//! FlowMail UI:三栏布局、虚拟化列表、详情、写信/设置/帮助 Modal、动作与模拟收信。

use std::collections::HashSet;

use gpui::{
    canvas, px, prelude::*, App, Context, Entity, FocusHandle, Focusable,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Render, UniformListScrollHandle,
    Window, uniform_list,
};

fn fid_hash(s: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}
use gpui_kit::prelude::*;

use crate::state::{self, Folder, Mail, Persisted, Settings};
use crate::{SHORTCUTS, Compose, DeleteMail, FocusSearch, Help, NextMail, PrevMail, Quit, Settings as SettingsAction, ToggleRead, ToggleStar};

pub struct FlowMailApp {
    focus: FocusHandle,
    mails: Vec<Mail>,
    custom_folders: Vec<Folder>,
    settings: Settings,
    selected_folder: String,
    selected_mail: Option<u64>,
    checked: HashSet<u64>,
    next_id: u64,
    search: Entity<InputState>,
    list_scroll: UniformListScrollHandle,
    compose_open: bool,
    compose_to: Entity<InputState>,
    compose_cc: Entity<InputState>,
    compose_subject: Entity<InputState>,
    compose_body: Entity<InputState>,
    compose_editing_draft: Option<u64>,
    settings_open: bool,
    help_open: bool,
    folder_modal_open: bool,
    folder_name: Entity<InputState>,
    folder_rename_id: Option<String>,
    confirm: Option<ConfirmKind>,
    sidebar_w: Pixels,
    list_w: Pixels,
    drag: Option<DragPane>,
}

enum DragPane {
    Sidebar { start_x: Pixels, start_w: Pixels },
    List { start_x: Pixels, start_w: Pixels },
}

#[derive(Clone)]
pub enum ConfirmKind {
    PermanentDelete,
    ResetAll,
    DeleteFolder(String),
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

impl FlowMailApp {
    pub fn new_with_arg(cx: &mut Context<Self>, arg: Option<String>) -> Self {
        let mut app = Self::new(cx);
        match arg.as_deref() {
            Some("compose") => {
                app.compose_open = true;
            }
            Some(id) => {
                if let Ok(id) = id.parse::<u64>() {
                    app.selected_mail = Some(id);
                    if let Some(m) = app.mails.iter_mut().find(|m| m.id == id) {
                        m.read = true;
                    }
                }
            }
            None => {}
        }
        app
    }

    pub fn new(cx: &mut Context<Self>) -> Self {
        let persisted = state::load_state();
        let (mails, custom, settings, selected_folder, next_id) = match persisted {
            Some(p) => (p.mails, p.custom_folders, p.settings, p.selected_folder, p.next_id),
            None => {
                let mails = state::seed_mails(now());
                let next = mails.iter().map(|m| m.id).max().unwrap_or(0) + 1;
                (mails, vec![], Settings::default(), "inbox".to_string(), next)
            }
        };

        let mut theme = if settings.theme_dark { Theme::dark() } else { Theme::light() };
        theme.set_accent(Accent::ALL.get(settings.accent_index).copied().unwrap_or(Accent::Blue));
        cx.set_global(theme);

        let search = cx.new(|cx| InputState::new(cx).placeholder("搜索邮件(主题/发件人/正文)…"));
        let compose_to = cx.new(|cx| InputState::new(cx).placeholder("收件人,逗号分隔"));
        let compose_cc = cx.new(|cx| InputState::new(cx).placeholder("抄送(可选)"));
        let compose_subject = cx.new(|cx| InputState::new(cx).placeholder("主题"));
        let compose_body = cx.new(|cx| InputState::new(cx).multi_line(8).placeholder("正文…"));
        let folder_name = cx.new(|cx| InputState::new(cx).placeholder("文件夹名称"));

        // 主题变化 → 全局重绘
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();

        let app = Self {
            focus: cx.focus_handle(),
            mails,
            custom_folders: custom,
            settings,
            selected_folder,
            selected_mail: None,
            checked: HashSet::new(),
            next_id,
            search,
            list_scroll: UniformListScrollHandle::new(),
            compose_open: false,
            compose_to,
            compose_cc,
            compose_subject,
            compose_body,
            compose_editing_draft: None,
            settings_open: false,
            help_open: false,
            folder_modal_open: false,
            folder_name,
            folder_rename_id: None,
            confirm: None,
            sidebar_w: px(200.),
            list_w: px(380.),
            drag: None,
        };

        // 模拟收信循环
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(45))
                    .await;
                if this.update(cx, |this, cx| this.receive_mail(cx)).is_err() {
                    break;
                }
            }
        })
        .detach();

        app
    }

    // ---------- 数据 ----------

    fn folders_all(&self) -> Vec<Folder> {
        let mut v = state::builtin_folders();
        v.extend(self.custom_folders.iter().cloned());
        v
    }

    fn folder_name_of(&self, id: &str) -> String {
        self.folders_all()
            .iter()
            .find(|f| f.id == id)
            .map(|f| f.name.clone())
            .unwrap_or_else(|| id.to_string())
    }

    fn visible_mails(&self, cx: &App) -> Vec<u64> {
        let q = self.search.read(cx).text().trim().to_lowercase();
        let folder = self.selected_folder.clone();
        let mut list: Vec<&Mail> = self
            .mails
            .iter()
            .filter(|m| {
                let in_folder = if folder == "starred" {
                    m.starred && m.folder != "trash"
                } else {
                    m.folder == folder
                };
                let hit = q.is_empty()
                    || m.subject.to_lowercase().contains(&q)
                    || m.from.to_lowercase().contains(&q)
                    || m.body.to_lowercase().contains(&q);
                in_folder && hit
            })
            .collect();
        if self.settings.sort_by_sender {
            list.sort_by(|a, b| a.from.cmp(&b.from));
        } else {
            list.sort_by_key(|m| if self.settings.sort_desc { -m.time_unix } else { m.time_unix });
        }
        list.into_iter().map(|m| m.id).collect()
    }

    fn unread_count(&self, folder: &str) -> usize {
        self.mails.iter().filter(|m| m.folder == folder && !m.read).count()
    }

    fn mail(&self, id: u64) -> Option<&Mail> {
        self.mails.iter().find(|m| m.id == id)
    }

    fn persist(&self) {
        state::save_state(&Persisted {
            mails: self.mails.clone(),
            custom_folders: self.custom_folders.clone(),
            settings: self.settings.clone(),
            selected_folder: self.selected_folder.clone(),
            next_id: self.next_id,
        });
    }

    // ---------- 动作 ----------

    fn select_mail(&mut self, id: u64, cx: &mut Context<Self>) {
        self.selected_mail = Some(id);
        if let Some(m) = self.mails.iter_mut().find(|m| m.id == id) {
            if !m.read {
                m.read = true;
                self.persist();
            }
        }
        cx.notify();
    }

    fn toggle_read(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_mail
            && let Some(m) = self.mails.iter_mut().find(|m| m.id == id)
        {
            m.read = !m.read;
            self.persist();
            cx.notify();
        }
    }

    fn toggle_star(&mut self, id: u64, cx: &mut Context<Self>) {
        if let Some(m) = self.mails.iter_mut().find(|m| m.id == id) {
            m.starred = !m.starred;
            self.persist();
            cx.notify();
        }
    }

    fn batch_move_trash(&mut self, ids: &[u64], cx: &mut Context<Self>) {
        let n = ids.len();
        for id in ids {
            if let Some(m) = self.mails.iter_mut().find(|m| m.id == *id) {
                m.folder = "trash".into();
            }
        }
        self.selected_mail = None;
        self.checked.clear();
        self.persist();
        toast(ToastKind::Success, format!("已将 {n} 封邮件移入垃圾箱"), cx);
        cx.notify();
    }

    fn receive_mail(&mut self, cx: &mut Context<Self>) {
        let m = state::random_incoming(now(), self.next_id);
        self.next_id += 1;
        let subject = m.subject.clone();
        self.mails.push(m);
        self.persist();
        toast(ToastKind::Info, format!("新邮件:{}", subject), cx);
        cx.notify();
    }

    fn open_compose_window(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.compose_editing_draft = None;
        self.compose_to.update(cx, |s, cx| s.set_text_quiet("", cx));
        self.compose_cc.update(cx, |s, cx| s.set_text_quiet("", cx));
        self.compose_subject.update(cx, |s, cx| s.set_text_quiet("", cx));
        self.compose_body.update(cx, |s, cx| s.set_text_quiet("", cx));
        self.compose_open = true;
        cx.notify();
    }

    fn compose_snapshot(&self, cx: &App) -> (String, String, String, String) {
        (
            self.compose_to.read(cx).text().to_string(),
            self.compose_cc.read(cx).text().to_string(),
            self.compose_subject.read(cx).text().to_string(),
            self.compose_body.read(cx).text().to_string(),
        )
    }

    fn compose_reply(&mut self, id: u64, reply_all: bool, is_reply: bool, cx: &mut Context<Self>) {
        let Some(m) = self.mail(id).cloned() else { return };
        let addr = m
            .from
            .rsplit('<')
            .next()
            .map(|s| s.trim_end_matches('>').trim())
            .unwrap_or("someone@example.com")
            .to_string();
        let to = addr;
        let cc = if reply_all { m.cc.join(", ") } else { String::new() };
        let subject = if is_reply {
            if m.subject.starts_with("Re:") { m.subject.clone() } else { format!("Re: {}", m.subject) }
        } else {
            format!("Fwd: {}", m.subject)
        };
        let quoted: String = m
            .body
            .lines()
            .map(|l| format!("> {}\n", l))
            .collect::<String>();
        let body = format!("\n\n{}\n\n—— 在 {} 的原邮件 ——\n{}", "", m.from, quoted);
        self.compose_to.update(cx, |s, cx| s.set_text_quiet(to, cx));
        self.compose_cc.update(cx, |s, cx| s.set_text_quiet(cc, cx));
        self.compose_subject.update(cx, |s, cx| s.set_text_quiet(subject, cx));
        self.compose_body.update(cx, |s, cx| s.set_text_quiet(body, cx));
        self.compose_editing_draft = None;
        self.compose_open = true;
        cx.notify();
    }

    fn send_mail(&mut self, cx: &mut Context<Self>) {
        let (to, cc, subject, body) = self.compose_snapshot(cx);
        if to.trim().is_empty() || subject.trim().is_empty() {
            toast(ToastKind::Error, "收件人与主题不能为空", cx);
            return;
        }
        if let Some(draft_id) = self.compose_editing_draft {
            self.mails.retain(|m| m.id != draft_id);
        }
        let m = Mail {
            id: self.next_id,
            from: "我 <me@example.com>".into(),
            to: to.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            cc: cc.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            subject,
            body,
            time_unix: now(),
            folder: "sent".into(),
            read: true,
            starred: false,
            attachments: vec![],
            tags: vec![],
        };
        self.next_id += 1;
        self.mails.push(m);
        self.compose_open = false;
        self.persist();
        toast(ToastKind::Success, "邮件已发送", cx);
        cx.notify();
    }

    fn save_draft(&mut self, cx: &mut Context<Self>) {
        let (to, cc, subject, body) = self.compose_snapshot(cx);
        if let Some(draft_id) = self.compose_editing_draft {
            if let Some(m) = self.mails.iter_mut().find(|m| m.id == draft_id) {
                m.to = vec![to];
                m.cc = vec![cc];
                m.subject = subject;
                m.body = body;
            }
        } else {
            let m = Mail {
                id: self.next_id,
                from: "我 <me@example.com>".into(),
                to: vec![to],
                cc: vec![cc],
                subject: if subject.is_empty() { "(无主题)".into() } else { subject },
                body,
                time_unix: now(),
                folder: "drafts".into(),
                read: true,
                starred: false,
                attachments: vec![],
                tags: vec![],
            };
            self.next_id += 1;
            self.mails.push(m);
        }
        self.compose_open = false;
        self.persist();
        toast(ToastKind::Success, "草稿已保存", cx);
        cx.notify();
    }

    fn edit_draft(&mut self, id: u64, cx: &mut Context<Self>) {
        let Some(m) = self.mail(id).cloned() else { return };
        self.compose_to.update(cx, |s, cx| s.set_text_quiet(m.to.join(", "), cx));
        self.compose_cc.update(cx, |s, cx| s.set_text_quiet(m.cc.join(", "), cx));
        self.compose_subject.update(cx, |s, cx| s.set_text_quiet(m.subject.clone(), cx));
        self.compose_body.update(cx, |s, cx| s.set_text_quiet(m.body.clone(), cx));
        self.compose_editing_draft = Some(id);
        self.compose_open = true;
        cx.notify();
    }

    fn commit_folder(&mut self, cx: &mut Context<Self>) {
        let name = self.folder_name.read(cx).text().trim().to_string();
        if name.is_empty() {
            toast(ToastKind::Error, "文件夹名称不能为空", cx);
            return;
        }
        if let Some(rid) = self.folder_rename_id.clone() {
            if let Some(f) = self.custom_folders.iter_mut().find(|f| f.id == rid) {
                f.name = name;
            }
        } else {
            let id = format!("custom-{}", self.next_id);
            self.custom_folders.push(Folder { id: id.clone(), name, builtin: false });
            self.next_id += 1;
        }
        self.folder_modal_open = false;
        self.persist();
        cx.notify();
    }

    fn confirm_execute(&mut self, cx: &mut Context<Self>) {
        let kind = self.confirm.clone();
        self.confirm = None;
        match kind {
            Some(ConfirmKind::PermanentDelete) => {
                if let Some(id) = self.selected_mail {
                    self.mails.retain(|m| m.id != id);
                    self.selected_mail = None;
                    self.persist();
                    toast(ToastKind::Success, "已永久删除", cx);
                }
            }
            Some(ConfirmKind::ResetAll) => {
                self.mails = state::seed_mails(now());
                self.custom_folders.clear();
                self.settings = Settings::default();
                self.selected_folder = "inbox".into();
                self.selected_mail = None;
                self.next_id = self.mails.iter().map(|m| m.id).max().unwrap_or(0) + 1;
                self.persist();
                toast(ToastKind::Success, "已恢复初始演示数据", cx);
            }
            Some(ConfirmKind::DeleteFolder(id)) => {
                self.mails.iter_mut().for_each(|m| {
                    if m.folder == id {
                        m.folder = "trash".into();
                    }
                });
                self.custom_folders.retain(|f| f.id != id);
                if self.selected_folder == id {
                    self.selected_folder = "inbox".into();
                }
                self.persist();
                toast(ToastKind::Success, "文件夹已删除,邮件移入垃圾箱", cx);
            }
            _ => {}
        }
        cx.notify();
    }

    fn row_h(&self) -> Pixels {
        if self.settings.compact { px(34.) } else { px(48.) }
    }
}

/// 虚拟化邮件列表面板
fn list_pane(
    theme: Theme,
    visible: Vec<u64>,
    row_h: Pixels,
    now_ts: i64,
    scroll: UniformListScrollHandle,
    cx: &mut Context<FlowMailApp>,
) -> gpui::AnyElement {
    div()
        .flex_1()
        .min_h_0()
        .child(
            uniform_list(
                "fm-list",
                visible.len(),
                cx.processor(move |this, range, _window, cx| {
                    let mut items = Vec::new();
                    for ix in range {
                        let Some(id) = visible.get(ix).copied() else { continue };
                        let Some(m) = this.mail(id) else { continue };
                        let m = m.clone();
                        let is_checked = this.checked.contains(&id);
                        let is_selected = this.selected_mail == Some(id);
                        let entity = cx.entity();
                        items.push(mail_row(
                            &m, ix, is_checked, is_selected, row_h, &theme, entity, now_ts,
                        ));
                    }
                    items
                }),
            )
            .track_scroll(scroll)
            .size_full(),
        )
        .into_any_element()
}

impl Render for FlowMailApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let entity = cx.entity();
        let now_ts = now();
        let visible = self.visible_mails(cx);
        let selected = self.selected_mail;
        let base = self.font_base();
        let row_h = self.row_h();
        let _ = base;

        div()
            .id("flowmail-root")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.bg)
            .text_color(theme.fg)
            .font_family(theme.font_ui.clone())
            .key_context("FlowMail")
            .track_focus(&self.focus.clone())
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &Compose, window: &mut Window, cx: &mut Context<FlowMailApp>| this.open_compose_window(window, cx)))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &FocusSearch, window: &mut Window, _cx: &mut Context<FlowMailApp>| {
                let h = this.search.focus_handle(_cx);
                window.focus(&h);
            }))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &NextMail, _w: &mut Window, cx: &mut Context<FlowMailApp>| {
                let vis = this.visible_mails(cx);
                let next = match this.selected_mail {
                    None => vis.first().copied(),
                    Some(id) => {
                        let ix = vis.iter().position(|v| *v == id).map(|i| i + 1).unwrap_or(0);
                        vis.get(ix).copied()
                    }
                };
                if let Some(id) = next {
                    this.select_mail(id, cx);
                }
            }))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &PrevMail, _w: &mut Window, cx: &mut Context<FlowMailApp>| {
                let vis = this.visible_mails(cx);
                let prev = match this.selected_mail {
                    None => None,
                    Some(id) => {
                        let ix = vis.iter().position(|v| *v == id).unwrap_or(1);
                        if ix > 0 { vis.get(ix - 1).copied() } else { None }
                    }
                };
                if let Some(id) = prev {
                    this.select_mail(id, cx);
                }
            }))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &ToggleRead, _w: &mut Window, cx: &mut Context<FlowMailApp>| this.toggle_read(cx)))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &ToggleStar, _w: &mut Window, cx: &mut Context<FlowMailApp>| {
                if let Some(id) = this.selected_mail {
                    this.toggle_star(id, cx);
                }
            }))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &DeleteMail, _w: &mut Window, cx: &mut Context<FlowMailApp>| {
                if let Some(id) = this.selected_mail {
                    this.batch_move_trash(&[id], cx);
                }
            }))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &Help, _: &mut Window, cx: &mut Context<FlowMailApp>| {
                this.help_open = !this.help_open;
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut FlowMailApp, _: &SettingsAction, _: &mut Window, cx: &mut Context<FlowMailApp>| {
                this.settings_open = !this.settings_open;
                cx.notify();
            }))
            .on_action(|_: &Quit, _w, cx| cx.quit())
            // 顶栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(52.))
                    .px_3()
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Icon::new("mail").size(px(18.)).text_color(theme.primary),
                            )
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("FlowMail"),
                            )
                            .child(
                                div()
                                    .w(px(280.))
                                    .child(text_input("fm-search", &self.search.clone()).prefix_icon("search").clearable()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("compose", "写邮件")
                                    .primary()
                                    .icon("edit")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_compose_window(window, cx);
                                    })),
                            )
                            .child(
                                IconButton::new("fm-settings", "settings")
                                    .tooltip("设置 ⌘E")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings_open = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                IconButton::new("fm-help", "help")
                                    .tooltip("快捷键 ?")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.help_open = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                IconButton::new("fm-theme", if theme.is_dark() { "sun" } else { "moon" })
                                    .tooltip("切换主题")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.theme_dark = !this.settings.theme_dark;
                                        let mode = if this.settings.theme_dark {
                                            gpui_kit::theme::ThemeMode::Dark
                                        } else {
                                            gpui_kit::theme::ThemeMode::Light
                                        };
                                        cx.update_global::<Theme, _>(|t, _| *t = t.clone().with_mode(mode));
                                        this.persist();
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // 三栏
            .child(self.render_body(&theme, &visible, now_ts, row_h, cx))
            // 状态栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(26.))
                    .px_3()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .text_size(px(11.))
                    .text_color(theme.fg_subtle)
                    .child(format!(
                        "文件夹:{} · 视图 {} 封 · 未读 {} 封",
                        self.folder_name_of(&self.selected_folder),
                        visible.len(),
                        self.unread_count(&self.selected_folder)
                    ))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(Icon::new("refresh").size(px(10.)))
                            .child("每 45 秒模拟收信"),
                    ),
            )
            // Modals
            .when(self.compose_open, |el| {
                let snap = self.compose_snapshot(cx);
                let e_to = self.compose_to.clone();
                let e_cc = self.compose_cc.clone();
                let e_sub = self.compose_subject.clone();
                let e_body = self.compose_body.clone();
                el.child(compose_modal(entity.clone(), self.compose_editing_draft, snap, e_to, e_cc, e_sub, e_body, cx))
            })
            .when(self.settings_open, |el| el.child(settings_modal(entity.clone(), &self.settings.clone(), cx)))
            .when(self.help_open, |el| el.child(help_modal(cx)))
            .when(self.folder_modal_open, |el| {
                let input = self.folder_name.clone();
                el.child(folder_modal(entity.clone(), self.folder_rename_id.clone(), input, cx))
            })
            .when_some(self.confirm.clone(), |el, kind| {
                el.child(confirm_modal(kind, entity.clone(), cx))
            })
            .children(render_toasts(window, cx))
    }
}

impl FlowMailApp {
    fn font_base(&self) -> Pixels {
        px(13. + self.settings.font_scale as f32)
    }

    fn render_body(
        &self,
        theme: &Theme,
        visible: &[u64],
        now_ts: i64,
        row_h: Pixels,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let entity = cx.entity();
        let selected = self.selected_mail;
        let theme_owned = theme.clone();
        let visible = visible.to_vec();
        div()
            .flex()
            .flex_1()
            .min_h_0()
            // 侧栏
            .child(self.render_sidebar(&theme, cx))
            .child(drag_handle(DragSide::Sidebar, entity.clone()))
            // 列表
            .child(
                div()
                    .w(self.list_w)
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .border_r_1()
                    .border_color(theme.border)
                    .bg(theme.bg)
                    .child(self.render_list_bar(&theme, &visible, cx))
                    .child(list_pane(
                        theme_owned,
                        visible,
                        row_h,
                        now_ts,
                        self.list_scroll.clone(),
                        cx,
                    )),
            )
            .child(drag_handle(DragSide::List, entity.clone()))
            // 详情
            .child(self.render_detail(&theme, selected, cx))
            .into_any_element()
    }

    fn render_sidebar(&self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let entity = cx.entity();
        div()
            .w(self.sidebar_w)
            .flex_shrink_0()
            .flex()
            .flex_col()
            .p_2()
            .gap_1()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.bg_panel)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .pb_1()
                    .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("文件夹"))
                    .child(
                        IconButton::new("fm-newfolder", "folder-plus")
                            .size(px(20.))
                            .icon_size(px(12.))
                            .tooltip("新建文件夹")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.folder_modal_open = true;
                                this.folder_rename_id = None;
                                cx.notify();
                            })),
                    ),
            )
            .children(self.folders_all().into_iter().map(|f| {
                let unread = if f.id == "starred" {
                    self.mails.iter().filter(|m| m.starred && m.folder != "trash").count()
                } else {
                    self.unread_count(&f.id)
                };
                let icon = match f.id.as_str() {
                    "inbox" => "inbox",
                    "starred" => "star",
                    "sent" => "send",
                    "drafts" => "file",
                    "trash" => "trash",
                    _ => "folder",
                };
                let is_selected = self.selected_folder == f.id;
                let fid = f.id.clone();
                let fid_del = f.id.clone();
                div()
                    .id(("fm-folder", fid_hash(&fid)))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_2()
                    .py_1p5()
                    .rounded(theme.radius_md)
                    .text_size(px(13.))
                    .cursor_pointer()
                    .when(is_selected, |el| {
                        el.bg(theme.primary.opacity(0.14)).text_color(theme.primary)
                    })
                    .when(!is_selected, |el| {
                        el.text_color(theme.fg_muted)
                            .hover(|s| s.bg(theme.bg_hover).text_color(theme.fg))
                    })
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected_folder = fid.clone();
                        this.selected_mail = None;
                        this.checked.clear();
                        cx.notify();
                    }))
                    .child(Icon::new(icon).size(px(14.)))
                    .child(div().flex_1().min_w_0().truncate().child(f.name.clone()))
                    .when(!f.builtin, |el| {
                        el.child(
                            IconButton::new(("fm-delfolder", fid_hash(&fid_del)), "x")
                                .size(px(16.))
                                .icon_size(px(10.))
                                .tooltip("删除文件夹")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.confirm = Some(ConfirmKind::DeleteFolder(fid_del.clone()));
                                    cx.notify();
                                })),
                        )
                    })
                    .when(unread > 0, |el| {
                        el.child(
                            Badge::new(format!("{}", unread))
                                .semantic(if is_selected { Semantic::Primary } else { Semantic::Neutral })
                                .dot(),
                        )
                    })
            }))
            .child(
                div()
                    .mt_auto()
                    .text_size(px(10.))
                    .text_color(theme.fg_subtle)
                    .child(format!(
                        "共 {} 封 · 未读 {} 封",
                        self.mails.iter().filter(|m| m.folder != "trash").count(),
                        self.mails.iter().filter(|m| !m.read && m.folder != "trash").count()
                    )),
            )
            .into_any_element()
    }

    fn render_list_bar(&self, theme: &Theme, visible: &[u64], cx: &mut Context<Self>) -> gpui::AnyElement {
        let checked_n = self.checked.len();
        div()
            .flex()
            .items_center()
            .justify_between()
            .px_2()
            .h(px(40.))
            .flex_shrink_0()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.bg_panel)
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(theme.fg_subtle)
                    .child(if checked_n > 0 {
                        format!("已选 {} 封", checked_n)
                    } else {
                        format!("{} 封邮件", visible.len())
                    }),
            )
            .child(
                div()
                    .flex()
                    .gap_1()
                    .when(checked_n > 0, |el| {
                        el.child(
                            Button::new("batch-read", "标已读").small().ghost().on_click(cx.listener(|this, _, _, cx| {
                                let ids: Vec<u64> = this.checked.iter().copied().collect();
                                for id in ids {
                                    if let Some(m) = this.mails.iter_mut().find(|m| m.id == id) {
                                        m.read = true;
                                    }
                                }
                                this.checked.clear();
                                this.persist();
                                toast(ToastKind::Success, "已标记为已读", cx);
                                cx.notify();
                            })),
                        )
                        .child(
                            Button::new("batch-star", "加星").small().ghost().on_click(cx.listener(|this, _, _, cx| {
                                let ids: Vec<u64> = this.checked.iter().copied().collect();
                                for id in ids {
                                    if let Some(m) = this.mails.iter_mut().find(|m| m.id == id) {
                                        m.starred = true;
                                    }
                                }
                                this.checked.clear();
                                this.persist();
                                toast(ToastKind::Success, "已加星", cx);
                                cx.notify();
                            })),
                        )
                        .child(
                            Button::new("batch-del", "删除").small().ghost().danger().on_click(cx.listener(|this, _, _, cx| {
                                let ids: Vec<u64> = this.checked.iter().copied().collect();
                                this.batch_move_trash(&ids, cx);
                            })),
                        )
                    })
                    .when(checked_n == 0 && !visible.is_empty(), |el| {
                        el.child(
                            Button::new("all-read", "全部已读").small().ghost().on_click(cx.listener(|this, _, _, cx| {
                                let folder = this.selected_folder.clone();
                                for m in this.mails.iter_mut() {
                                    if m.folder == folder || (folder == "starred" && m.starred) {
                                        m.read = true;
                                    }
                                }
                                this.persist();
                                toast(ToastKind::Success, "全部标为已读", cx);
                                cx.notify();
                            })),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_detail(&self, theme: &Theme, selected: Option<u64>, cx: &mut Context<Self>) -> gpui::AnyElement {
        let Some(id) = selected else {
            return div()
                .flex_1()
                .child(EmptyState::new("mail", "选择一封邮件", "点击列表中的邮件查看详情;按 ? 查看快捷键"))
                .into_any_element();
        };
        let Some(m) = self.mail(id).cloned() else {
            return div().flex_1().into_any_element();
        };
        let entity = cx.entity();
        let name = m.from.split('<').next().unwrap_or(&m.from).trim().to_string();
        let in_trash = m.folder == "trash";
        let mail_id = m.id;
        let body_source = m.body.clone();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            // 操作条
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py_1p5()
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .child(
                        IconButton::new("d-read", if m.read { "info" } else { "check" })
                            .tooltip(if m.read { "标记未读" } else { "标记已读" })
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.toggle_read(cx))
                            }),
                    )
                    .child(
                        IconButton::new("d-star", "star")
                            .tooltip("星标切换")
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.toggle_star(mail_id, cx))
                            }),
                    )
                    .child(div().h(px(16.)).child(divider_v()))
                    .child(
                        IconButton::new("d-reply", "reply")
                            .tooltip("回复")
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.compose_reply(mail_id, false, true, cx))
                            }),
                    )
                    .child(
                        IconButton::new("d-replyall", "users")
                            .tooltip("回复全部")
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.compose_reply(mail_id, true, true, cx))
                            }),
                    )
                    .child(
                        IconButton::new("d-forward", "forward")
                            .tooltip("转发")
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.compose_reply(mail_id, false, false, cx))
                            }),
                    )
                    .child(
                        IconButton::new("d-trash", "trash")
                            .tooltip("移入垃圾箱")
                            .danger()
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.batch_move_trash(&[mail_id], cx))
                            }),
                    )
                    .child(
                        IconButton::new("d-copy", "copy")
                            .tooltip("复制正文")
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| {
                                    let text = e.read(cx).mail(mail_id).map(|m| m.body.clone()).unwrap_or_default();
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                                    toast(ToastKind::Success, "正文已复制到剪贴板", cx);
                                }
                            }),
                    )
                    .when(in_trash, |el| {
                        el.child(div().h(px(16.)).child(divider_v()))
                            .child(
                                IconButton::new("d-restore", "undo")
                                    .tooltip("恢复到收件箱")
                                    .on_click({
                                        let e = entity.clone();
                                        move |_, _, cx| {
                                            e.update(cx, |this, cx| {
                                                if let Some(m) = this.mails.iter_mut().find(|m| m.id == mail_id) {
                                                    m.folder = "inbox".into();
                                                    this.persist();
                                                    this.selected_mail = None;
                                                    toast(ToastKind::Success, "已恢复到收件箱", cx);
                                                }
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                IconButton::new("d-purge", "x")
                                    .tooltip("永久删除")
                                    .danger()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.confirm = Some(ConfirmKind::PermanentDelete);
                                        cx.notify();
                                    }))
                            )
                    })
                    .child(
                        IconButton::new("d-archive", "folder-plus")
                            .tooltip("移动到文件夹")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.folder_modal_open = true;
                                this.folder_rename_id = None;
                                let _ = cx;
                            })),
                    ),
            )
            // 头部
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .text_size(px(18.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(m.subject.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Avatar::new(name.clone()).size(px(34.)))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(1.))
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .child(name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(theme.fg_subtle)
                                            .child(format!("收件人:{}", m.to.join(", "))),
                                    )
                                    .when(!m.cc.is_empty(), |el| {
                                        el.child(
                                            div()
                                                .text_size(px(11.))
                                                .text_color(theme.fg_subtle)
                                                .child(format!("抄送:{}", m.cc.join(", "))),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .ml_auto()
                                    .text_size(px(11.))
                                    .text_color(theme.fg_subtle)
                                    .child(gpui_kit::util::unix_to_datetime(m.time_unix)),
                            ),
                    )
                    .when(!m.tags.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_1()
                                .children(m.tags.iter().map(|t| Tag::new(t.clone()))),
                        )
                    }),
            )
            // 附件
            .when(!m.attachments.is_empty(), |el| {
                el.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .children(m.attachments.iter().map(|a| {
                            div()
                                .id(("fm-att", fid_hash(&a.name)))
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_2()
                                .py_1p5()
                                .rounded(theme.radius_md)
                                .bg(theme.bg_hover)
                                .border_1()
                                .border_color(theme.border)
                                .text_size(px(12.))
                                .cursor_pointer()
                                .hover(|s| s.border_color(theme.primary))
                                .on_click({
                                    let a_name = a.name.clone();
                                    move |_, _, cx| {
                                        toast(ToastKind::Info, format!("演示环境无法下载附件「{}」", a_name), cx);
                                    }
                                })
                                .child(Icon::new("archive").size(px(13.)).text_color(theme.info))
                                .child(a.name.clone())
                                .child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(theme.fg_subtle)
                                        .child(gpui_kit::util::format_bytes(a.size)),
                                )
                        })),
                )
            })
            // 正文
            .child(
                div()
                    .id("detail-body")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_4()
                    .py_3()
                    .child(MarkdownView::new(m.body.clone())),
            )
            .into_any_element()
    }
}

impl Focusable for FlowMailApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

// ---------- 组件 ----------

#[derive(Clone, Copy)]
enum DragSide {
    Sidebar,
    List,
}

fn drag_handle(side: DragSide, entity: Entity<FlowMailApp>) -> gpui::AnyElement {
    div()
        .id(match side {
            DragSide::Sidebar => "drag-side",
            DragSide::List => "drag-list",
        })
        .w(px(4.))
        .h_full()
        .flex_shrink_0()
        .cursor(gpui::CursorStyle::ResizeLeftRight)
        .hover(|s| s.bg(gpui::hsla(0.58, 0.7, 0.6, 0.5)))
        .child(
            canvas(
                move |_, _, _| (),
                move |bounds, _, window, _| {
                    let entity_down = entity.clone();
                    window.on_mouse_event(move |ev: &MouseDownEvent, _, _, cx| {
                        if bounds.contains(&ev.position) {
                            entity_down.update(cx, |this, _| {
                                this.drag = Some(match side {
                                    DragSide::Sidebar => DragPane::Sidebar { start_x: ev.position.x, start_w: this.sidebar_w },
                                    DragSide::List => DragPane::List { start_x: ev.position.x, start_w: this.list_w },
                                });
                            });
                        }
                    });
                    let entity2 = entity.clone();
                    window.on_mouse_event(move |ev: &MouseMoveEvent, _, _, cx| {
                        if ev.dragging() {
                            entity2.update(cx, |this, cx| {
                                match this.drag {
                                    Some(DragPane::Sidebar { start_x, start_w }) => {
                                        let delta = ev.position.x - start_x;
                                        this.sidebar_w = (start_w + delta).clamp(px(160.), px(320.));
                                        cx.notify();
                                    }
                                    Some(DragPane::List { start_x, start_w }) => {
                                        let delta = ev.position.x - start_x;
                                        this.list_w = (start_w + delta).clamp(px(280.), px(560.));
                                        cx.notify();
                                    }
                                    None => {}
                                }
                            });
                        }
                    });
                    let entity3 = entity.clone();
                    window.on_mouse_event(move |_: &MouseUpEvent, _, _, cx| {
                        entity3.update(cx, |this, _| {
                            this.drag = None;
                        });
                    });
                },
            )
            .size_full(),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn mail_row(
    m: &Mail,
    ix: usize,
    is_checked: bool,
    is_selected: bool,
    row_h: Pixels,
    theme: &Theme,
    entity: Entity<FlowMailApp>,
    now_ts: i64,
) -> gpui::AnyElement {
    let id = m.id;
    let name = m.from.split('<').next().unwrap_or(&m.from).trim().to_string();
    let entity_row = entity.clone();
    div()
        .id(("fm-row", id))
        .w_full()
        .flex()
        .items_center()
        .gap_2()
        .px_2()
        .h(row_h)
        .border_b_1()
        .border_color(theme.border.opacity(0.5))
        .cursor_pointer()
        .when(is_selected, |el| el.bg(theme.primary.opacity(0.10)))
        .when(!is_selected && ix % 2 == 1, |el| el.bg(theme.bg_hover.opacity(0.4)))
        .hover(|s| s.bg(theme.bg_hover))
        .on_click(move |_, _, cx| {
            entity_row.update(cx, |this, cx| this.select_mail(id, cx));
        })
        .child({
            let e = entity.clone();
            div()
                .id(("fm-check", id))
                .flex()
                .items_center()
                .justify_center()
                .size(px(14.))
                .rounded(px(3.))
                .border_1()
                .border_color(if is_checked { theme.primary } else { theme.border_strong })
                .bg(if is_checked { theme.primary } else { gpui::hsla(0., 0., 0., 0.) })
                .on_click(move |_, _, cx| {
                    e.update(cx, |this, cx| {
                        if this.checked.contains(&id) {
                            this.checked.remove(&id);
                        } else {
                            this.checked.insert(id);
                        }
                        cx.notify();
                    });
                })
                .child(if is_checked {
                    Icon::new("check").size(px(9.)).text_color(theme.primary_fg)
                } else {
                    Icon::new("minus").size(px(9.)).text_color(gpui::hsla(0., 0., 0., 0.))
                })
        })
        .child(Avatar::new(name.clone()).size(px(26.)))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(1.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_size(px(12.))
                                .text_color(if m.read { theme.fg_muted } else { theme.fg })
                                .font_weight(if m.read { gpui::FontWeight::NORMAL } else { gpui::FontWeight::SEMIBOLD })
                                .child(name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(theme.fg_subtle)
                                .child(format_mail_time(m.time_unix, now_ts)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_size(px(12.))
                                .text_color(if m.read { theme.fg_subtle } else { theme.fg })
                                .child(m.subject.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .children((!m.attachments.is_empty()).then(|| {
                                    Icon::new("archive").size(px(11.)).text_color(theme.fg_subtle)
                                }))
                                .children(m.starred.then(|| {
                                    Icon::new("star").size(px(12.)).text_color(theme.warning)
                                })),
                        ),
                ),
        )
        .child(
            div()
                .w(px(8.))
                .flex()
                .justify_center()
                .child(if !m.read {
                    div().size(px(7.)).rounded_full().bg(theme.primary)
                } else {
                    div()
                }),
        )
        .into_any_element()
}

pub fn format_mail_time(unix: i64, now: i64) -> String {
    let diff = now - unix;
    if diff < 86400 && gpui_kit::util::unix_to_date(unix) == gpui_kit::util::unix_to_date(now) {
        let secs = unix.rem_euclid(86400);
        format!("{:02}:{:02}", secs / 3600, (secs % 3600) / 60)
    } else if diff < 172800 {
        "昨天".to_string()
    } else {
        gpui_kit::util::unix_to_date(unix)
    }
}

// ---------- Modals ----------

fn compose_modal(
    entity: Entity<FlowMailApp>,
    editing_draft: Option<u64>,
    snapshot: (String, String, String, String),
    e_to: Entity<InputState>,
    e_cc: Entity<InputState>,
    e_subject: Entity<InputState>,
    e_body: Entity<InputState>,
    cx: &Context<FlowMailApp>,
) -> gpui::AnyElement {
    let (to, _cc, subject, body) = snapshot;
    let theme = cx.global::<Theme>().clone();
    let can_send = !to.trim().is_empty() && !subject.trim().is_empty();
    Modal::new(if editing_draft.is_some() { "编辑草稿" } else { "写邮件" })
            .width(px(560.))
            .on_close({
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.compose_open = false;
                        cx.notify();
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(52.)).text_size(px(12.)).text_color(theme.fg_muted).child("收件人"))
                    .child(div().flex_1().child(text_input("c-to", &e_to))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(52.)).text_size(px(12.)).text_color(theme.fg_muted).child("抄送"))
                    .child(div().flex_1().child(text_input("c-cc", &e_cc))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(52.)).text_size(px(12.)).text_color(theme.fg_muted).child("主题"))
                    .child(div().flex_1().child(text_input("c-subject", &e_subject))),
            )
            .child(div().child(text_area("c-body", &e_body)))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(11.))
                    .child(
                        div()
                            .text_color(if can_send { theme.fg_subtle } else { theme.danger })
                            .child(if can_send {
                                format!("正文 {} 字", body.chars().count())
                            } else {
                                "收件人与主题不能为空".to_string()
                            }),
                    )
                    .child("⌘Enter 发送"),
            )
            .footer(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("c-draft", "存草稿").ghost().on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.save_draft(cx))
                        }),
                    )
                    .child(
                        Button::new("c-cancel", "取消").on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.compose_open = false;
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Button::new("c-send", "发送").primary().disabled(!can_send).on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.send_mail(cx))
                        }),
                    ),
            )
    .into_any_element()
}

fn settings_modal(entity: Entity<FlowMailApp>, settings: &Settings, cx: &Context<FlowMailApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    Modal::new("设置")
            .on_close({
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.settings_open = false;
                        cx.notify();
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("深浅主题")
                    .child(
                        Button::new("s-theme", if theme.is_dark() { "切换到浅色" } else { "切换到深色" })
                            .icon(if theme.is_dark() { "sun" } else { "moon" })
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| {
                                    e.update(cx, |this, cx| {
                                        this.settings.theme_dark = !this.settings.theme_dark;
                                        let mode = if this.settings.theme_dark {
                                            gpui_kit::theme::ThemeMode::Dark
                                        } else {
                                            gpui_kit::theme::ThemeMode::Light
                                        };
                                        cx.update_global::<Theme, _>(|t, _| *t = t.clone().with_mode(mode));
                                        this.persist();
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("紧凑列表")
                    .child({
                        let e = entity.clone();
                        Switch::new("s-compact", settings.compact).on_change(move |_, cx| {
                            e.update(cx, |this, cx| {
                                this.settings.compact = !this.settings.compact;
                                this.persist();
                                cx.notify();
                            });
                        })
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("正文字号")
                    .child(
                        Segmented::new(
                            "s-font",
                            vec!["小".into(), "中".into(), "大".into()],
                            settings.font_scale,
                        )
                        .on_change({
                            let e = entity.clone();
                            move |ix, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.settings.font_scale = ix;
                                    this.persist();
                                    cx.notify();
                                });
                            }
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child("主色")
                    .child({
                        let e = entity.clone();
                        div().flex().gap_2().children(Accent::ALL.iter().enumerate().map(|(ix, a)| {
                            let e2 = e.clone();
                            div()
                                .id(("accent", ix))
                                .size(px(22.))
                                .rounded_full()
                                .bg(a.base())
                                .border_2()
                                .border_color(if settings.accent_index == ix { theme.fg } else { gpui::hsla(0., 0., 0., 0.) })
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    e2.update(cx, |this, cx| {
                                        this.settings.accent_index = ix;
                                        cx.update_global::<Theme, _>(|t, _| t.set_accent(Accent::ALL[ix]));
                                        this.persist();
                                        cx.notify();
                                    });
                                })
                        }))
                    }),
            )
            .footer(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("s-reset", "清空本地数据").danger().on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.confirm = Some(ConfirmKind::ResetAll);
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Button::new("s-done", "完成").primary().on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.settings_open = false;
                                    cx.notify();
                                });
                            }
                        }),
                    ),
            )
    .into_any_element()
}

fn help_modal(cx: &Context<FlowMailApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();
    Modal::new("键盘快捷键")
            .on_close({
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.help_open = false;
                        cx.notify();
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(SHORTCUTS.iter().map(|(k, desc)| {
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div().text_size(px(13.)).text_color(theme.fg).child(desc.to_string()),
                            )
                            .child(Kbd::new(*k))
                    })),
            )
            .footer(
                div().child(
                    Button::new("h-close", "关闭").primary().on_click({
                        let e = entity.clone();
                        move |_, _, cx| {
                            e.update(cx, |this, cx| {
                                this.help_open = false;
                                cx.notify();
                            });
                        }
                    }),
                ),
            )
    .into_any_element()
}

fn folder_modal(
    entity: Entity<FlowMailApp>,
    rename_id: Option<String>,
    input: Entity<InputState>,
    cx: &Context<FlowMailApp>,
) -> gpui::AnyElement {
    let title = if rename_id.is_some() { "重命名文件夹" } else { "新建文件夹" };
    Modal::new(title)
            .width(px(360.))
            .on_close({
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.folder_modal_open = false;
                        cx.notify();
                    });
                }
            })
            .child(div().child(text_input("folder-name", &input)))
            .footer(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("f-cancel", "取消").on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.folder_modal_open = false;
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Button::new("f-ok", "确定").primary().on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.commit_folder(cx))
                        }),
                    ),
            )
    .into_any_element()
}

fn confirm_modal(kind: ConfirmKind, entity: Entity<FlowMailApp>, cx: &Context<FlowMailApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let (title, desc) = match &kind {
        ConfirmKind::PermanentDelete => ("永久删除".to_string(), "该邮件将从垃圾箱中彻底删除,不可撤销。".to_string()),
        ConfirmKind::ResetAll => ("清空本地数据".to_string(), "所有邮件、文件夹与设置将恢复为初始演示状态。".to_string()),
        ConfirmKind::DeleteFolder(name) => ("删除文件夹".to_string(), format!("文件夹「{name}」将被删除,其中邮件移入垃圾箱。")),
    };
    Modal::new(title)
            .width(px(380.))
            .on_close({
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.confirm = None;
                        cx.notify();
                    });
                }
            })
            .child(div().text_size(px(13.)).text_color(theme.fg_muted).child(desc))
            .footer(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("cf-cancel", "取消").on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.confirm = None;
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Button::new("cf-ok", "确认执行").danger().on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.confirm_execute(cx))
                        }),
                    ),
            )
    .into_any_element()
}
