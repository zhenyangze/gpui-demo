//! 浮层反馈:Modal(模态框)/ Toast(通知,全局队列)。

use std::time::{Duration, Instant};

use gpui::{
    div, prelude::FluentBuilder, px, App, AnimationExt as _, Context, Entity, Global, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, Styled, Window, prelude::*,
};

use crate::theme::{hsla, Theme};

// ------------------------- Modal -------------------------

#[derive(IntoElement)]
pub struct Modal {
    title: SharedString,
    width: Pixels,
    children: Vec<gpui::AnyElement>,
    footer: Option<gpui::AnyElement>,
    on_close: Box<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl Modal {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            width: px(440.),
            children: Vec::new(),
            footer: None,
            on_close: Box::new(|_, _| {}),
        }
    }

    pub fn width(mut self, w: Pixels) -> Self {
        self.width = w;
        self
    }

    pub fn footer(mut self, el: impl IntoElement) -> Self {
        self.footer = Some(el.into_any_element());
        self
    }

    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Box::new(f);
        self
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Modal {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let on_close = std::rc::Rc::new(self.on_close);
        let on_close_card = on_close.clone();
        let on_close_backdrop = on_close.clone();

        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0., 0., 0., 0.45))
            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                on_close_backdrop(window, cx);
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(self.width)
                    .max_h(gpui::relative(0.85))
                    .rounded(theme.radius_lg)
                    .bg(theme.bg_elevated)
                    .border_1()
                    .border_color(theme.border)
                    .shadow_lg()
                    .overflow_hidden()
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(
                        // 标题栏
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_4()
                            .py_3()
                            .border_b_1()
                            .border_color(theme.border)
                            .child(
                                div()
                                    .text_size(px(15.))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme.fg)
                                    .child(self.title),
                            )
                            .child(
                                crate::components::button::IconButton::new("modal-close", "x")
                                    .on_click(move |_, window, cx| on_close_card(window, cx)),
                            ),
                    )
                    .child(
                        div()
                            .id("modal-body")
                            .flex()
                            .flex_col()
                            .gap_3()
                            .px_4()
                            .py_4()
                            .overflow_y_scroll()
                            .children(self.children),
                    )
                    .children(self.footer.map(|f| {
                        div()
                            .flex()
                            .items_center()
                            .justify_end()
                            .gap_2()
                            .px_4()
                            .py_3()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(f)
                    })),
            )
    }
}

// ------------------------- Toast -------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastKind {
    fn icon(self) -> &'static str {
        match self {
            ToastKind::Success => "check-circle",
            ToastKind::Error => "alert",
            ToastKind::Warning => "alert",
            ToastKind::Info => "info",
        }
    }
    fn label(self) -> &'static str {
        match self {
            ToastKind::Success => "成功",
            ToastKind::Error => "错误",
            ToastKind::Warning => "警告",
            ToastKind::Info => "提示",
        }
    }
}

#[derive(Clone)]
struct Toast {
    id: u64,
    kind: ToastKind,
    message: SharedString,
    created: Instant,
    duration: Duration,
}

pub struct ToastState {
    toasts: Vec<Toast>,
    next_id: u64,
}

pub struct ToastGlobal(pub Entity<ToastState>);

impl Global for ToastGlobal {}

pub fn install_toasts(cx: &mut App) {
    let state = cx.new(|_| ToastState {
        toasts: Vec::new(),
        next_id: 1,
    });
    cx.set_global(ToastGlobal(state));
}

/// 弹出一条通知(默认 3 秒)
pub fn toast(kind: ToastKind, message: impl Into<SharedString>, cx: &mut App) {
    toast_with_duration(kind, message, Duration::from_secs(3), cx);
}

pub fn toast_with_duration(
    kind: ToastKind,
    message: impl Into<SharedString>,
    duration: Duration,
    cx: &mut App,
) {
    let Some(entity) = cx.try_global::<ToastGlobal>().map(|g| g.0.clone()) else {
        return;
    };
    let message = message.into();
    let id = entity.update(cx, |s, cx| {
        let id = s.next_id;
        s.next_id += 1;
        s.toasts.push(Toast {
            id,
            kind,
            message,
            created: Instant::now(),
            duration,
        });
        cx.notify();
        id
    });

    // 到期自动移除
    let weak = entity.downgrade();
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(duration + Duration::from_millis(60))
            .await;
        let _ = weak.update(cx, |s, cx| {
            let before = s.toasts.len();
            s.toasts.retain(|t| t.id != id);
            if s.toasts.len() != before {
                cx.notify();
            }
        });
    })
    .detach();
}

fn dismiss(id: u64, cx: &mut App) {
    let Some(entity) = cx.try_global::<ToastGlobal>().map(|g| g.0.clone()) else {
        return;
    };
    entity.update(cx, |s, cx| {
        s.toasts.retain(|t| t.id != id);
        cx.notify();
    });
}

/// 渲染通知层(根视图内调用一次;容器需 relative/absolute 根)
pub fn render_toasts(_window: &Window, cx: &App) -> Option<gpui::AnyElement> {
    let entity = cx.try_global::<ToastGlobal>()?.0.clone();
    let toasts = entity.read(cx).toasts.clone();
    if toasts.is_empty() {
        return None;
    }
    let theme = cx.global::<Theme>().clone();
    Some(
        div()
            .absolute()
            .bottom_4()
            .right_4()
            .flex()
            .flex_col()
            .gap_2()
            .w(px(340.))
            .children(toasts.into_iter().map(move |t| {
                let color = match t.kind {
                    ToastKind::Success => theme.success,
                    ToastKind::Error => theme.danger,
                    ToastKind::Warning => theme.warning,
                    ToastKind::Info => theme.info,
                };
                div()
                    .id(("toast", t.id))
                    .flex()
                    .gap_2()
                    .rounded(theme.radius_md)
                    .bg(theme.bg_elevated)
                    .border_1()
                    .border_color(theme.border)
                    .shadow_lg()
                    .overflow_hidden()
                    .child(
                        div().w(px(4.)).bg(color),
                    )
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .p_3()
                            .flex_1()
                            .child(
                                crate::icons::Icon::new(t.kind.icon())
                                    .size(px(15.))
                                    .text_color(color),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.))
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(theme.fg)
                                            .child(t.kind.label().to_string()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(theme.fg_muted)
                                            .child(t.message.to_string()),
                                    ),
                            )
                            .child(
                                div()
                                    .id(("toast-close", t.id))
                                    .cursor(gpui::CursorStyle::PointingHand)
                                    .text_color(theme.fg_subtle)
                                    .hover(|s| s.text_color(theme.fg))
                                    .on_click(move |_, _, cx| dismiss(t.id, cx))
                                    .child(crate::icons::Icon::new("x").size(px(12.))),
                            ),
                    )
                    .with_animation(
                        ("toast-enter", t.id),
                        gpui::Animation::new(Duration::from_millis(180)),
                        |el, delta| el.opacity(0.3 + 0.7 * delta),
                    )
                    .into_any_element()
            }))
            .into_any_element(),
    )
}
