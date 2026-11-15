//! AgentConsole UI:会话列表 / 对话流 / 工具卡 / 输入区 / 任务面板 / 设置。
//!
//! 渲染期间只读局部状态与其他实体,根实体数据在 render 入口一次性借出。

use std::cell::RefCell;

use gpui::{
    px, prelude::*, App, Context, Entity, FocusHandle, Focusable, KeyDownEvent, Pixels, Render,
    UniformListScrollHandle, Window,
};
use gpui_kit::prelude::*;

use crate::engine::{AgentEngine, MockEngine};
use crate::state::{
    self, apply_theme, approx_tokens, load_sessions, load_settings, save_sessions, save_settings,
    speed_delay_ms, AgentSettings, Conversation, Message, Role, Task, MODELS, SPEEDS,
};
use crate::{Help, NewChat, OpenPalette, Quit, SHORTCUTS, StopGen};

pub struct AgentApp {
    focus: FocusHandle,
    engine: Box<dyn AgentEngine>,
    sessions: Vec<Conversation>,
    active: usize,
    next_id: u64,
    settings: AgentSettings,
    input: Entity<InputState>,
    prompt: Entity<InputState>,
    history: RefCell<Vec<String>>,
    generating: bool,
    streaming_msg: Option<Message>,
    gen_task: RefCell<Option<gpui::Task<()>>>,
    palette: Entity<PaletteState>,
    tasks_open: bool,
    help_open: bool,
    settings_open: bool,
    tokens_in_total: u64,
    tokens_out_total: u64,
    chat_scroll: UniformListScrollHandle,
}

impl AgentApp {
    pub fn new_with_arg(cx: &mut Context<Self>, arg: Option<String>, window: &mut Window) -> Self {
        let mut app = Self::new(cx, arg.clone());
        // 启动自动聚焦输入框(A-077)
        {
            let h = app.input.focus_handle(cx);
            window.focus(&h);
        }
        if arg.as_deref() == Some("tasks") {
            if let Some(c) = app.sessions.get_mut(app.active) {
                c.tasks = crate::engine::plan_tasks("发布 gpui-kit v0.2")
                    .into_iter()
                    .enumerate()
                    .map(|(ix, t)| Task { text: t, done: ix < 1, active: ix == 1 })
                    .collect();
            }
        }
        app
    }

    pub fn new(cx: &mut Context<Self>, arg: Option<String>) -> Self {
        let settings = load_settings();
        apply_theme(&settings, cx);
        let sessions = load_sessions().unwrap_or_else(state::seed_sessions);
        let next_id = sessions.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        let input = cx.new(|cx| {
            InputState::new(cx)
                .multi_line(6)
                .placeholder("给智能体发消息…(Enter 发送,Shift+Enter 换行)")
        });
        let prompt = cx.new(|cx| InputState::new(cx).multi_line(3));
        let prompt_text = settings.system_prompt.clone();
        prompt.update(cx, |s, cx| s.set_text_quiet(prompt_text, cx));
        let palette = cx.new(|cx| PaletteState::new(cx));
        let engine = Box::new(MockEngine);

        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();

        let mut app = Self {
            focus: cx.focus_handle(),
            engine,
            sessions,
            active: 0,
            next_id,
            settings,
            input,
            prompt,
            history: RefCell::new(vec![]),
            generating: false,
            streaming_msg: None,
            gen_task: RefCell::new(None),
            palette,
            tasks_open: true,
            help_open: false,
            settings_open: false,
            tokens_in_total: 0,
            tokens_out_total: 0,
            chat_scroll: UniformListScrollHandle::new(),
        };
        app.register_palette(cx);

        if arg.as_deref() == Some("tasks") {
            if let Some(c) = app.sessions.get_mut(app.active) {
                c.tasks = crate::engine::plan_tasks("发布 gpui-kit v0.2")
                    .into_iter()
                    .enumerate()
                    .map(|(ix, t)| Task { text: t, done: ix < 1, active: ix == 1 })
                    .collect();
            }
        }
        app
    }

    fn register_palette(&self, cx: &mut Context<Self>) {
        let entity = cx.entity();
        let items = vec![
            PaletteCommand::new("new", "新建会话 ⌘N", {
                let e = entity.clone();
                move |_, cx| e.update(cx, |this, cx| this.new_chat(cx))
            }),
            PaletteCommand::new("theme", "切换深浅主题", {
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.settings.dark = !this.settings.dark;
                        apply_theme(&this.settings, cx);
                        this.persist_settings();
                        cx.notify();
                    });
                }
            }),
            PaletteCommand::new("tasks", "收起/展开任务面板", {
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.tasks_open = !this.tasks_open;
                        cx.notify();
                    });
                }
            }),
            PaletteCommand::new("settings", "打开设置", {
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.settings_open = true;
                        cx.notify();
                    });
                }
            }),
            PaletteCommand::new("stats", "重置 token 统计", {
                let e = entity.clone();
                move |_, cx| {
                    e.update(cx, |this, cx| {
                        this.tokens_in_total = 0;
                        this.tokens_out_total = 0;
                        cx.notify();
                        toast(ToastKind::Success, "统计已清零", cx);
                    });
                }
            }),
        ];
        self.palette.update(cx, |p, cx| p.set_items(items, cx));
    }

    fn persist(&self) {
        save_sessions(&self.sessions);
    }

    fn persist_settings(&self) {
        save_settings(&self.settings);
    }

    fn active_conv(&self) -> Option<&Conversation> {
        self.sessions.get(self.active)
    }

    fn active_conv_mut(&mut self) -> Option<&mut Conversation> {
        self.sessions.get_mut(self.active)
    }

    fn new_chat(&mut self, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;
        self.sessions.insert(
            0,
            Conversation {
                id,
                title: format!("新会话 {}", id),
                messages: vec![],
                tasks: vec![],
                unread: false,
            },
        );
        self.active = 0;
        self.streaming_msg = None;
        self.generating = false;
        self.persist();
        cx.notify();
    }

    fn delete_chat(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.sessions.remove(ix);
        if self.sessions.is_empty() {
            let id = self.next_id;
            self.next_id += 1;
            self.sessions.push(Conversation {
                id,
                title: "新会话".into(),
                messages: vec![],
                tasks: vec![],
                unread: false,
            });
        }
        if self.active >= self.sessions.len() {
            self.active = self.sessions.len() - 1;
        }
        self.persist();
        cx.notify();
    }

    /// 发送消息并启动流式生成
    fn send(&mut self, cx: &mut Context<Self>) {
        if self.generating {
            return;
        }
        let text = self.input.read(cx).text().trim().to_string();
        if text.is_empty() {
            return;
        }
        // 斜杠命令
        if let Some(cmd) = text.strip_prefix('/') {
            match cmd.trim() {
                "clear" => {
                    if let Some(c) = self.active_conv_mut() {
                        c.messages.clear();
                        c.tasks.clear();
                    }
                    self.streaming_msg = None;
                    self.persist();
                    cx.notify();
                    return;
                }
                "help" => {
                    self.help_open = true;
                    cx.notify();
                    return;
                }
                "stats" => {
                    toast(
                        ToastKind::Info,
                        format!("累计输入 {} tok / 输出 {} tok", self.tokens_in_total, self.tokens_out_total),
                        cx,
                    );
                    return;
                }
                "tools" => {
                    let list: Vec<String> = crate::engine::tools::registry()
                        .iter()
                        .map(|t| format!("{} — {}", t.name, t.description))
                        .collect();
                    toast(ToastKind::Info, format!("注册工具 {} 个:{}", list.len(), list.join("; ")), cx);
                    return;
                }
                _ => {
                    toast(ToastKind::Warning, format!("未知命令 /{}", cmd), cx);
                    return;
                }
            }
        }

        self.history.borrow_mut().push(text.clone());
        self.input.update(cx, |s, cx| s.set_text_quiet("", cx));

        let tokens = approx_tokens(&text);
        self.tokens_in_total += tokens as u64;
        if let Some(c) = self.active_conv_mut() {
            c.messages.push(Message {
                role: Role::User,
                content: text.clone(),
                tool_calls: vec![],
                time_unix: state::now(),
                tokens,
            });
            if c.messages.len() == 1 {
                c.title = text.chars().take(18).collect();
            }
        }
        self.persist();

        // 预计算引擎响应(工具卡 + 完整文本)
        let model_name = MODELS[self.settings.model_index]
            .split(' ')
            .next()
            .unwrap_or("mock")
            .to_string();
        let events = self.engine.respond(&text, &model_name, self.settings.temperature);
        let mut tool_calls = vec![];
        let mut content = String::new();
        let mut tasks: Option<Vec<Task>> = None;
        let mut tokens_out = 0usize;
        let mut tokens_in = 0usize;
        for ev in &events {
            match ev {
                crate::engine::StreamEvent::ToolCall { id, name, args, result, ok, dur_ms } => {
                    tool_calls.push(state::ToolCallRecord {
                        id: *id, name: name.clone(), args: args.clone(), result: result.clone(), ok: *ok, dur_ms: *dur_ms,
                    });
                }
                crate::engine::StreamEvent::TextDelta(d) => content.push_str(d),
                crate::engine::StreamEvent::Tasks(t) => {
                    tasks = Some(t.iter().map(|x| Task { text: x.clone(), done: false, active: false }).collect());
                }
                crate::engine::StreamEvent::Done { tokens_in: ti, tokens_out: to } => {
                    tokens_in = *ti;
                    tokens_out = *to;
                }
            }
        }

        self.generating = true;
        self.streaming_msg = Some(Message {
            role: Role::Assistant,
            content: String::new(),
            tool_calls: tool_calls.clone(),
            time_unix: state::now(),
            tokens: 0,
        });

        // 流式播放:按字符块推进
        let delay = speed_delay_ms(self.settings.speed_index, self.settings.model_index);
        let chunk_len = if delay < 15 { 6 } else { 2 };
        let this = cx.weak_entity();
        let task = cx.spawn(async move |this, cx| {
            let chars: Vec<char> = content.chars().collect();
            let mut pos = 0usize;
            // 先显示工具卡
            let _ = this.update(cx, |this, cx| {
                if let Some(msg) = this.streaming_msg.as_mut() {
                    msg.tool_calls = tool_calls.clone();
                }
                if let Some(c) = this.active_conv_mut() {
                    c.tasks = tasks.clone().unwrap_or_default();
                }
                cx.notify();
            });
            while pos < chars.len() {
                if delay > 0 {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(delay))
                        .await;
                }
                let end = (pos + chunk_len).min(chars.len());
                let chunk: String = chars[pos..end].iter().collect();
                pos = end;
                let ok = this.update(cx, |this, cx| {
                    if !this.generating {
                        return;
                    }
                    if let Some(msg) = this.streaming_msg.as_mut() {
                        msg.content.push_str(&chunk);
                    }
                    cx.notify();
                });
                if ok.is_err() {
                    return;
                }
            }
            this.update(cx, |this, cx| {
                this.tokens_in_total += tokens_in as u64;
                this.tokens_out_total += tokens_out as u64;
                if let (Some(msg), Some(c)) = (this.streaming_msg.take(), this.active_conv_mut()) {
                    let mut msg = msg;
                    msg.tokens = tokens_out.max(1);
                    c.messages.push(msg);
                }
                this.generating = false;
                this.persist();
                cx.notify();
            })
            .ok();
        });
        *self.gen_task.borrow_mut() = Some(task);
        cx.notify();
    }

    fn stop_generation(&mut self, cx: &mut Context<Self>) {
        if self.generating {
            self.generating = false;
            // gpui Task 无显式 cancel;置 generating=false 后循环内自检退出
            let _ = self.gen_task.borrow_mut().take();
            if let Some(msg) = self.streaming_msg.take() {
                if let Some(c) = self.active_conv_mut() {
                    let mut msg = msg;
                    if !msg.content.is_empty() {
                        msg.content.push_str("\n\n*(已停止生成)*");
                    } else {
                        msg.content = "*(已停止生成)*".into();
                    }
                    msg.tokens = 1;
                    c.messages.push(msg);
                }
            }
            self.persist();
            toast(ToastKind::Warning, "已停止生成", cx);
            cx.notify();
        }
    }

    fn regenerate(&mut self, cx: &mut Context<Self>) {
        if self.generating {
            return;
        }
        let last_user = self
            .active_conv()
            .and_then(|c| c.messages.iter().rev().find(|m| m.role == Role::User))
            .map(|m| m.content.clone());
        if let Some(text) = last_user {
            if let Some(c) = self.active_conv_mut() {
                while c.messages.last().map(|m| m.role == Role::Assistant).unwrap_or(false) {
                    c.messages.pop();
                }
            }
            self.input.update(cx, |s, cx| s.set_text_quiet(text, cx));
            self.send(cx);
        }
    }

    fn toggle_task(&mut self, ix: usize, cx: &mut Context<Self>) {
        if let Some(c) = self.active_conv_mut()
            && let Some(t) = c.tasks.get_mut(ix)
        {
            t.done = !t.done;
            t.active = false;
            self.persist();
        }
        cx.notify();
    }

    fn clear_done_tasks(&mut self, cx: &mut Context<Self>) {
        if let Some(c) = self.active_conv_mut() {
            c.tasks.retain(|t| !t.done);
            self.persist();
        }
        cx.notify();
    }
}

impl Render for AgentApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let entity = cx.entity();
        let conv = self.active_conv().cloned();
        let generating = self.generating;
        let streaming = self.streaming_msg.clone();
        let model_label = MODELS[self.settings.model_index].to_string();
        let temp_label = format!("{:.1}", self.settings.temperature);
        let engine_label = self.engine.name().to_string();
        let tin = gpui_kit::util::format_thousands(self.tokens_in_total as i64);
        let tout = gpui_kit::util::format_thousands(self.tokens_out_total as i64);
        let sessions = self.sessions.clone();
        let active_ix = self.active;
        let tasks_open = self.tasks_open;
        let input_len = self.input.read(cx).text().chars().count();

        div()
            .id("agent-root")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.bg)
            .text_color(theme.fg)
            .font_family(theme.font_ui.clone())
            .key_context("Agent")
            .track_focus(&self.focus.clone())
            .on_action(cx.listener(|this: &mut AgentApp, _: &NewChat, _: &mut Window, cx: &mut Context<AgentApp>| this.new_chat(cx)))
            .on_action(cx.listener(|this: &mut AgentApp, _: &OpenPalette, w: &mut Window, cx: &mut Context<AgentApp>| {
                let pal = this.palette.clone();
                pal.update(cx, |p, cx| p.open(w, cx));
            }))
            .on_action(cx.listener(|this: &mut AgentApp, _: &Help, _: &mut Window, cx: &mut Context<AgentApp>| {
                this.help_open = !this.help_open;
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut AgentApp, _: &StopGen, _: &mut Window, cx: &mut Context<AgentApp>| this.stop_generation(cx)))
            .on_action(|_: &Quit, _, cx| cx.quit())
            // 顶栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(50.))
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
                            .child(Icon::new("bot").size(px(18.)).text_color(theme.primary))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("AgentConsole"),
                            )
                            .child(Badge::new(engine_label.clone()).semantic(Semantic::Primary))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(theme.fg_subtle)
                                    .child(model_label.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                IconButton::new("a-settings", "settings")
                                    .tooltip("设置")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings_open = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                IconButton::new("a-help", "help")
                                    .tooltip("快捷键 ?")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.help_open = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("a-palette", "⌘K").ghost().on_click(cx.listener(
                                    |this, _, w, cx| {
                                        this.palette.update(cx, |p, cx| p.open(w, cx));
                                    },
                                )),
                            ),
                    ),
            )
            // 三栏
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_sessions(&theme, &sessions, active_ix, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .bg(theme.bg)
                            .child(chat_pane(&conv, &streaming, generating, &theme, entity.clone()))
                            .child(input_bar(input_len, generating, &theme, entity.clone(), self.input.clone(), cx)),
                    )
                    .when(tasks_open, |el| {
                        el.child(task_pane(&conv, &theme, entity.clone(), cx))
                    }),
            )
            // 状态栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.))
                    .px_3()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .text_size(px(10.))
                    .text_color(theme.fg_subtle)
                    .child(format!(
                        "引擎 {} · 模型 {} · temp {} · {}",
                        engine_label,
                        model_label,
                        temp_label,
                        if generating { "生成中…" } else { "在线" },
                    ))
                    .child(format!("tokens 输入 {} / 输出 {}", tin, tout)),
            )
            // Modals
            .when(self.help_open, |el| el.child(help_modal(entity.clone(), cx)))
            .when(self.settings_open, |el| {
                let (model_ix, speed_ix, temp, dark, prompt_input) = (
                    self.settings.model_index,
                    self.settings.speed_index,
                    self.settings.temperature,
                    self.settings.dark,
                    self.prompt.clone(),
                );
                el.child(settings_modal(entity.clone(), model_ix, speed_ix, temp, dark, prompt_input, cx))
            })
            .child(self.palette.clone())
            .children(render_toasts(window, cx))
    }
}

impl AgentApp {
    fn render_sessions(
        &self,
        theme: &Theme,
        sessions: &[Conversation],
        active_ix: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        div()
            .w(px(230.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .p_2()
            .gap_2()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.bg_panel)
            .child(
                Button::new("a-newchat", "新建会话")
                    .primary()
                    .icon("plus")
                    .full_width()
                    .on_click(cx.listener(|this, _, _, cx| this.new_chat(cx))),
            )
            .child(
                div()
                    .id("a-chat-list")
                    .flex()
                    .flex_col()
                    .gap_1()
                    .overflow_y_scroll()
                    .children(sessions.iter().enumerate().map(|(ix, c)| {
                        let is_active = ix == active_ix;
                        let c_id = c.id;
                        let c_title = c.title.clone();
                        let c_n = c.messages.len();
                        div()
                            .id(("a-chat", c_id))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_2()
                            .py_1p5()
                            .rounded(theme.radius_md)
                            .text_size(px(12.))
                            .cursor_pointer()
                            .when(is_active, |el| {
                                el.bg(theme.primary.opacity(0.14)).text_color(theme.primary)
                            })
                            .when(!is_active, |el| {
                                el.text_color(theme.fg_muted)
                                    .hover(|s| s.bg(theme.bg_hover).text_color(theme.fg))
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.active = ix;
                                this.streaming_msg = None;
                                this.generating = false;
                                this.persist();
                                cx.notify();
                            }))
                            .child(Icon::new("list").size(px(12.)))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .child(format!("{} ({})", c_title, c_n)),
                            )
                            .child(
                                IconButton::new(("a-delchat", c_id), "trash")
                                    .size(px(16.))
                                    .icon_size(px(10.))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.delete_chat(ix, cx);
                                        toast(ToastKind::Success, "会话已删除", cx);
                                    })),
                            )
                    })),
            )
            .into_any_element()
    }
}

impl Focusable for AgentApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

// ------------------------- 聊天区 -------------------------

fn chat_pane(
    conv: &Option<Conversation>,
    streaming: &Option<Message>,
    generating: bool,
    theme: &Theme,
    entity: Entity<AgentApp>,
) -> gpui::AnyElement {
    let Some(conv) = conv else {
        return div().into_any_element();
    };
    let items: Vec<gpui::AnyElement> = conv
        .messages
        .iter()
        .enumerate()
        .map(|(ix, m)| message_bubble(m, ix, theme, entity.clone()))
        .collect();

    div()
        .id("chat-scroll")
        .flex_1()
        .min_h_0()
        .overflow_y_scroll()
        .px_4()
        .py_3()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .when(conv.messages.is_empty() && streaming.is_none(), |el| {
                    el.child(
                        div().h(px(200.)).child(
                            EmptyState::new(
                                "bot",
                                "开始对话",
                                "试试:(12+8)*3 · 现在几点 · 帮我计划发布 · roll 2d6",
                            ),
                        ),
                    )
                })
                .children(items)
                .children(streaming.as_ref().map(|m| message_bubble(m, usize::MAX, theme, entity.clone())))
                .when(generating && streaming.as_ref().map(|m| m.content.is_empty()).unwrap_or(true) == false && false, |el| el)
                .when(generating, |el| {
                    el.child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(px(12.))
                            .text_color(theme.fg_subtle)
                            .child(Spinner::new().size(px(12.)))
                            .child("正在生成…(Esc 停止)"),
                    )
                }),
        )
        .into_any_element()
}

fn message_bubble(m: &Message, ix: usize, theme: &Theme, entity: Entity<AgentApp>) -> gpui::AnyElement {
    let is_user = m.role == Role::User;
    let tool_cards: Vec<gpui::AnyElement> = m
        .tool_calls
        .iter()
        .map(|tc| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .my_1()
                .p_2()
                .rounded(theme.radius_md)
                .bg(if tc.ok { theme.bg_input } else { theme.danger.opacity(0.12) })
                .border_1()
                .border_color(if tc.ok { theme.border } else { theme.danger.opacity(0.5) })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .text_size(px(11.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .child(
                                    Icon::new(if tc.ok { "wrench" } else { "alert" })
                                        .size(px(11.))
                                        .text_color(if tc.ok { theme.info } else { theme.danger }),
                                )
                                .child(format!("工具 {}", tc.name)),
                        )
                        .child(
                            div()
                                .text_size(px(10.))
                                .text_color(theme.fg_subtle)
                                .child(format!("{} ms", tc.dur_ms)),
                        ),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .font_family(theme.font_mono.clone())
                        .text_color(theme.fg_subtle)
                        .child(format!("args: {}", tc.args)),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .font_family(theme.font_mono.clone())
                        .text_color(if tc.ok { theme.fg } else { theme.danger })
                        .child(format!("→ {}", tc.result)),
                )
                .into_any_element()
        })
        .collect();

    let time_str = {
        let secs = m.time_unix.rem_euclid(86400);
        format!("{:02}:{:02}", secs / 3600, (secs % 3600) / 60)
    };

    if is_user {
        div()
            .flex()
            .justify_end()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_end()
                    .gap(px(2.))
                    .max_w(px(560.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div().text_size(px(10.)).text_color(theme.fg_subtle).child(time_str),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("我"),
                            )
                            .child(Avatar::new("我").size(px(22.))),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded(theme.radius_lg)
                            .rounded_br(theme.radius_sm)
                            .bg(theme.primary)
                            .text_color(theme.primary_fg)
                            .text_size(px(13.))
                            .child(m.content.clone()),
                    ),
            )
            .into_any_element()
    } else {
        div()
            .flex()
            .justify_start()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(2.))
                    .max_w(px(640.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Avatar::new("AI").size(px(22.)))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("智能体"),
                            )
                            .child(div().text_size(px(10.)).text_color(theme.fg_subtle).child(time_str))
                            .child(
                                div()
                                    .text_size(px(10.))
                                    .text_color(theme.fg_subtle)
                                    .child(format!("{} tok", m.tokens)),
                            ),
                    )
                    .children(tool_cards)
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded(theme.radius_lg)
                            .rounded_bl(theme.radius_sm)
                            .bg(theme.bg_panel)
                            .border_1()
                            .border_color(theme.border)
                            .text_size(px(13.))
                            .min_w(px(80.))
                            .max_w(px(640.))
                            .child(MarkdownView::new(m.content.clone())),
                    )
                    .when(ix != usize::MAX, |el| {
                        el.child(
                            div()
                                .flex()
                                .gap_3()
                                .child({
                                    let e = entity.clone();
                                    div()
                                        .id(("msg-copy", ix))
                                        .text_size(px(10.))
                                        .text_color(theme.fg_subtle)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(theme.fg))
                                        .on_click(move |_, _, cx| {
                                            let text = e
                                                .read(cx)
                                                .active_conv()
                                                .and_then(|c| c.messages.get(ix))
                                                .map(|m| m.content.clone())
                                                .unwrap_or_default();
                                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
                                            toast(ToastKind::Success, "消息已复制", cx);
                                        })
                                        .child("复制")
                                })
                                .child({
                                    let e = entity.clone();
                                    div()
                                        .id(("msg-regen", ix))
                                        .text_size(px(10.))
                                        .text_color(theme.fg_subtle)
                                        .cursor_pointer()
                                        .hover(|s| s.text_color(theme.fg))
                                        .on_click(move |_, _, cx| {
                                            e.update(cx, |this, cx| this.regenerate(cx));
                                        })
                                        .child("重新生成")
                                }),
                        )
                    }),
            )
            .into_any_element()
    }
}

// ------------------------- 输入区 -------------------------

fn input_bar(
    input_len: usize,
    generating: bool,
    theme: &Theme,
    entity: Entity<AgentApp>,
    e_input: Entity<InputState>,
    cx: &Context<AgentApp>,
) -> gpui::AnyElement {
    let root_id = entity.entity_id();
    let suggestions = ["(12+8)*3", "现在几点", "帮我计划发布", "roll 2d6"];

    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .flex_shrink_0()
        .border_t_1()
        .border_color(theme.border)
        .bg(theme.bg_panel)
        .child(
            div()
                .flex()
                .gap_2()
                .children(suggestions.iter().enumerate().map(|(ix, s)| {
                    let e_input2 = e_input.clone();
                    let text = s.to_string();
                    div()
                        .id(("suggest", ix))
                        .px_2()
                        .py_1()
                        .rounded_full()
                        .bg(theme.bg_hover)
                        .border_1()
                        .border_color(theme.border)
                        .text_size(px(11.))
                        .text_color(theme.fg_muted)
                        .cursor_pointer()
                        .hover(|s| s.text_color(theme.fg).border_color(theme.primary))
                        .on_click(move |_, _, cx| {
                            e_input2.update(cx, |st, cx| st.set_text_quiet(text.clone(), cx));
                            cx.notify(root_id);
                        })
                        .child(s.to_string())
                })),
        )
        .child(
            div()
                .flex()
                .items_end()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .on_key_down({
                            let e_send = entity.clone();
                            move |ev: &KeyDownEvent, _, cx| {
                                let ks = &ev.keystroke;
                                if ks.key == "enter" && !ks.modifiers.shift && !ks.modifiers.control && !ks.modifiers.platform
                                {
                                    e_send.update(cx, |this, cx| this.send(cx));
                                    cx.stop_propagation();
                                }
                            }
                        })
                        .child(text_area("a-input", &e_input).clearable()),
                )
                .child(if generating {
                    Button::new("a-stop", "停止").danger().icon("stop").on_click({
                        let e = entity.clone();
                        move |_, _, cx| e.update(cx, |this, cx| this.stop_generation(cx))
                    })
                } else {
                    Button::new("a-send", "发送")
                        .primary()
                        .icon("send")
                        .disabled(input_len == 0)
                        .on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.send(cx))
                        })
                }),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(10.))
                .text_color(theme.fg_subtle)
                .child("Enter 发送 · Shift+Enter 换行 · 生成中 Esc 停止")
                .child(format!("{} 字", input_len)),
        )
        .into_any_element()
}

// ------------------------- 任务面板 -------------------------

fn task_pane(
    conv: &Option<Conversation>,
    theme: &Theme,
    entity: Entity<AgentApp>,
    cx: &Context<AgentApp>,
) -> gpui::AnyElement {
    let Some(conv) = conv else {
        return div().into_any_element();
    };
    let tasks = conv.tasks.clone();
    let total = tasks.len();
    let done = tasks.iter().filter(|t| t.done).count();
    let pct = if total == 0 { 0. } else { done as f64 / total as f64 };

    div()
        .w(px(260.))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .p_2()
        .gap_2()
        .border_l_1()
        .border_color(theme.border)
        .bg(theme.bg_panel)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("任务面板"),
                )
                .child(
                    IconButton::new("a-tasks-close", "x")
                        .size(px(18.))
                        .icon_size(px(10.))
                        .on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.tasks_open = false;
                                    cx.notify();
                                });
                            }
                        }),
                ),
        )
        .child(
            div()
                .id("a-task-list")
                .flex()
                .flex_col()
                .gap_1()
                .overflow_y_scroll()
                .children(tasks.iter().enumerate().map(|(ix, t)| {
                    gpui_kit::sortable::SortableRow::new(ix, t.text.clone(),
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .flex_1()
                            .min_w_0()
                            .child(
                                Icon::new(if t.done {
                                    "check-circle"
                                } else if t.active {
                                    "clock"
                                } else {
                                    "list"
                                })
                                .size(px(13.))
                                .text_color(if t.done {
                                    theme.success
                                } else if t.active {
                                    theme.warning
                                } else {
                                    theme.fg_subtle
                                }),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(12.))
                                    .text_color(if t.done { theme.fg_subtle } else { theme.fg })
                                    .child(t.text.clone()),
                            ),
                    )
                    .list_id(0x51D0_u64)
                    .on_click({
                        let e = entity.clone();
                        move |_, _, cx| e.update(cx, |this, cx| this.toggle_task(ix, cx))
                    })
                    .on_drop({
                        let e = entity.clone();
                        move |from: usize, to: usize, _w, cx| {
                            e.update(cx, |this, cx| {
                                if let Some(c) = this.active_conv_mut()
                                    && let Some(order) = gpui_kit::sortable::reorder(c.tasks.len(), from, to)
                                {
                                    c.tasks = gpui_kit::sortable::apply_reorder(&c.tasks, &order);
                                    this.persist();
                                }
                                cx.notify();
                            });
                        }
                    })
                })),
        )
        .when(total == 0, |el| {
            el.child(
                div().h(px(120.)).child(
                    EmptyState::new("list", "暂无任务", "发送「帮我计划 xxx」让智能体生成任务"),
                ),
            )
        })
        .child(
            div()
                .mt_auto()
                .flex()
                .flex_col()
                .gap_2()
                .child(ProgressBar::new(pct))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_size(px(11.))
                        .text_color(theme.fg_subtle)
                        .child(format!("{}/{} 完成", done, total))
                        .child(
                            Button::new("a-clear-done", "清除已完成")
                                .small()
                                .ghost()
                                .disabled(done == 0)
                                .on_click({
                                    let e = entity.clone();
                                    move |_, _, cx| e.update(cx, |this, cx| this.clear_done_tasks(cx))
                                }),
                        ),
                ),
        )
        .into_any_element()
}

// ------------------------- Modals -------------------------

fn help_modal(entity: Entity<AgentApp>, cx: &Context<AgentApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    Modal::new("键盘快捷键 / 斜杠命令")
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
                        .child(div().text_size(px(13.)).child(desc.to_string()))
                        .child(Kbd::new(*k))
                })),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(theme.fg_muted)
                .mt_2()
                .child("斜杠命令:/help · /clear · /stats · /tools"),
        )
        .footer(
            div().child(
                Button::new("ah-close", "关闭").primary().on_click({
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

fn settings_modal(
    entity: Entity<AgentApp>,
    model_ix: usize,
    speed_ix: usize,
    temp: f64,
    dark: bool,
    prompt: Entity<InputState>,
    cx: &Context<AgentApp>,
) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    Modal::new("设置")
        .width(px(480.))
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
                .child("模型")
                .child(
                    Segmented::new(
                        "s-model",
                        MODELS.iter().map(|m| SharedString::from(m.split(' ').next().unwrap())).collect(),
                        model_ix,
                    )
                    .on_change({
                        let e = entity.clone();
                        move |ix, _, cx| {
                            e.update(cx, |this, cx| {
                                this.settings.model_index = ix;
                                this.persist_settings();
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
                .child("回复速度")
                .child(
                    Segmented::new("s-speed", SPEEDS.iter().map(|s| SharedString::from(*s)).collect(), speed_ix)
                        .on_change({
                            let e = entity.clone();
                            move |ix, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.settings.speed_index = ix;
                                    this.persist_settings();
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
                .gap_3()
                .child(div().w(px(90.)).text_size(px(12.)).child("temperature"))
                .child({
                    let e = entity.clone();
                    div().flex_1().child(
                        Slider::new("s-temp", temp)
                            .range(0., 1.)
                            .step(0.1)
                            .on_change(move |v, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.settings.temperature = v;
                                    this.persist_settings();
                                    cx.notify();
                                });
                            })
                    )
                })
                .child(
                    div()
                        .font_family(theme.font_mono.clone())
                        .text_size(px(12.))
                        .text_color(theme.primary)
                        .child(format!("{temp:.1}")),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child("深浅主题")
                .child({
                    let e = entity.clone();
                    Switch::new("s-dark", dark).on_change(move |_, cx| {
                        e.update(cx, |this, cx| {
                            this.settings.dark = !this.settings.dark;
                            apply_theme(&this.settings, cx);
                            this.persist_settings();
                            cx.notify();
                        });
                    })
                }),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(div().text_size(px(12.)).child("系统提示词"))
                .child(text_area("s-prompt", &prompt)),
        )
        .footer(
            div().child(
                Button::new("as-done", "完成").primary().on_click({
                    let e = entity.clone();
                    move |_, _, cx| {
                        e.update(cx, |this, cx| {
                            // 读取提示词
                            let text = this.prompt.read(cx).text().to_string();
                            this.settings.system_prompt = text;
                            this.persist_settings();
                            this.settings_open = false;
                            cx.notify();
                        });
                    }
                }),
            ),
        )
        .into_any_element()
}
