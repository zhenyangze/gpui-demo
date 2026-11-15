//! 命令面板(Cmd+K):模糊搜索动作,键盘导航。

use gpui::{
    actions, div, px, App, Context, Entity, Focusable, IntoElement, KeyBinding, ParentElement,
    Render, SharedString, Styled, Window, prelude::*,
};

use crate::components::input::InputState;
use crate::theme::Theme;

actions!(kit_palette, [PaletteUp, PaletteDown, PaletteConfirm, PaletteClose]);

pub fn bind_palette_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", PaletteUp, Some("Palette")),
        KeyBinding::new("down", PaletteDown, Some("Palette")),
        KeyBinding::new("enter", PaletteConfirm, Some("Palette")),
        KeyBinding::new("escape", PaletteClose, Some("Palette")),
    ]);
}

#[derive(Clone)]
pub struct PaletteCommand {
    pub id: &'static str,
    pub label: SharedString,
    pub hint: Option<SharedString>,
    pub handler: std::rc::Rc<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl PaletteCommand {
    pub fn new(
        id: &'static str,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            hint: None,
            handler: std::rc::Rc::new(handler),
        }
    }
    pub fn hint(mut self, h: impl Into<SharedString>) -> Self {
        self.hint = Some(h.into());
        self
    }
}

pub struct PaletteState {
    pub open: bool,
    input: Entity<InputState>,
    items: Vec<PaletteCommand>,
    selected: usize,
    on_close: Option<std::rc::Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl PaletteState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| InputState::new(cx).placeholder("输入命令或搜索…"));
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self {
            open: false,
            input,
            items: Vec::new(),
            selected: 0,
            on_close: None,
        }
    }

    pub fn set_items(&mut self, items: Vec<PaletteCommand>, cx: &mut Context<Self>) {
        self.items = items;
        self.selected = 0;
        cx.notify();
    }

    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(std::rc::Rc::new(f));
        self
    }

    pub fn open(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = true;
        self.input.update(cx, |s, cx| s.set_text("", window, cx));
        self.selected = 0;
        cx.notify();
        let handle = self.input.focus_handle(cx);
        window.focus(&handle);
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open = false;
        if let Some(cb) = self.on_close.as_ref() {
            cb(window, cx);
        }
        cx.notify();
    }

    fn query(&self, cx: &App) -> String {
        self.input.read(cx).text().to_lowercase()
    }

    /// 简单模糊匹配:子串或子序列
    fn matches(&self, label: &str, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let label = label.to_lowercase();
        if label.contains(query) {
            return true;
        }
        let mut it = label.chars();
        query.chars().all(|qc| it.any(|lc| lc == qc))
    }

    fn filtered(&self, cx: &App) -> Vec<usize> {
        let query = self.query(cx);
        self.items
            .iter()
            .enumerate()
            .filter(|(_, cmd)| self.matches(&cmd.label, &query))
            .map(|(ix, _)| ix)
            .collect()
    }
}

impl Render for PaletteState {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div().into_any_element();
        }
        let theme = cx.global::<Theme>().clone();
        let filtered = self.filtered(cx);
        let selected = if self.selected >= filtered.len() {
            0
        } else {
            self.selected
        };
        let items = self.items.clone();
        let count = filtered.len();

        let on_up = cx.listener(|this, _: &PaletteUp, _window, cx| {
            if this.selected > 0 {
                this.selected -= 1;
                cx.notify();
            }
        });
        let on_down = cx.listener(|this, _: &PaletteDown, _window, cx| {
            let count = this.filtered(cx).len();
            if this.selected + 1 < count {
                this.selected += 1;
                cx.notify();
            }
        });
        let on_confirm = cx.listener(move |this, _: &PaletteConfirm, window, cx| {
            let filtered = this.filtered(cx);
            if let Some(&ix) = filtered.get(this.selected) {
                let handler = this.items[ix].handler.clone();
                let close = this.on_close.clone();
                if let Some(cb) = close {
                    cb(window, cx);
                }
                handler(window, cx);
            }
        });
        let on_esc = cx.listener(move |this, _: &PaletteClose, window, cx| {
            this.close(window, cx);
        });

        div()
            .id("palette-root")
            .absolute()
            .inset_0()
            .flex()
            .items_start()
            .justify_center()
            .pt(px(80.))
            .bg(crate::theme::hsla(0., 0., 0., 0.35))
            .key_context("Palette")
            .track_focus(&self.input.focus_handle(cx))
            .on_action(on_up)
            .on_action(on_down)
            .on_action(on_confirm)
            .on_action(on_esc)
            .on_mouse_down(gpui::MouseButton::Left, {
                let this = cx.weak_entity();
                move |_, window, cx| {
                    let _ = this.update(cx, |s, cx| s.close(window, cx));
                }
            })
            .child(
                div()
                    .id("palette-panel")
                    .flex()
                    .flex_col()
                    .w(px(480.))
                    .max_h(px(420.))
                    .rounded(theme.radius_lg)
                    .bg(theme.bg_elevated)
                    .border_1()
                    .border_color(theme.border_strong)
                    .shadow_lg()
                    .overflow_hidden()
                    .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(theme.border)
                            .child(crate::components::input::text_input("palette-input", &self.input)),
                    )
                    .child(
                        div()
                            .id("palette-list")
                            .flex()
                            .flex_col()
                            .overflow_y_scroll()
                            .py_1()
                            .children(filtered.into_iter().enumerate().map(move |(vis_ix, item_ix)| {
                                let cmd = items[item_ix].clone();
                                let is_sel = vis_ix == selected;
                                let handler = cmd.handler.clone();
                                let this_close = self.on_close.clone();
                                div()
                                    .id(("palette-item", item_ix))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .mx_2()
                                    .px_3()
                                    .py_2()
                                    .rounded(theme.radius_sm)
                                    .text_size(px(13.))
                                    .cursor(gpui::CursorStyle::PointingHand)
                                    .when(is_sel, |el| {
                                        el.bg(theme.primary.opacity(0.14))
                                    })
                                    .hover(|s| s.bg(theme.bg_hover))
                                    .on_click(move |_, window, cx| {
                                        if let Some(cb) = this_close.as_ref() {
                                            cb(window, cx);
                                        }
                                        handler(window, cx);
                                    })
                                    .child(
                                        crate::icons::Icon::new("zap")
                                            .size(px(13.))
                                            .text_color(if is_sel { theme.primary } else { theme.fg_subtle }),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_color(if is_sel { theme.primary } else { theme.fg })
                                            .child(cmd.label.to_string()),
                                    )
                                    .children(cmd.hint.clone().map(|h: SharedString| {
                                        div().text_size(px(11.)).text_color(theme.fg_subtle).child(h.to_string())
                                    }))
                                    .when(is_sel, |el| {
                                        el.child(
                                            crate::components::basic::Kbd::new("↵"),
                                        )
                                    })
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_1p5()
                            .border_t_1()
                            .border_color(theme.border)
                            .text_size(px(11.))
                            .text_color(theme.fg_subtle)
                            .child(div().flex().items_center().gap_1()
                                .child(crate::components::basic::Kbd::new("↑↓"))
                                .child(crate::components::basic::Kbd::new("↵"))
                                .child(crate::components::basic::Kbd::new("esc")))
                            .child(format!("{} 条匹配", count)),
                    ),
            )
            .into_any_element()
    }
}
