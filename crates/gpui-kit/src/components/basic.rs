//! 基础展示组件:Badge / Tag / Avatar / Divider / Kbd / Spinner / ProgressBar /
//! Skeleton / Card / EmptyState / Stat / Tooltip 视图。

use gpui::{
    div, prelude::FluentBuilder, px, relative, App, AnyView, AnimationExt as _, Context, Div, Hsla,
    IntoElement, ParentElement, Pixels, Render, RenderOnce, SharedString, Styled, Window,
    prelude::*,
};

use crate::theme::Theme;

// ------------------------- Badge -------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Semantic {
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Neutral,
}

impl Semantic {
    pub fn color(self, theme: &Theme) -> Hsla {
        match self {
            Semantic::Primary => theme.primary,
            Semantic::Success => theme.success,
            Semantic::Warning => theme.warning,
            Semantic::Danger => theme.danger,
            Semantic::Info => theme.info,
            Semantic::Neutral => theme.fg_muted,
        }
    }
}

#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    semantic: Semantic,
    dot: bool,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            semantic: Semantic::Neutral,
            dot: false,
        }
    }
    pub fn semantic(mut self, s: Semantic) -> Self {
        self.semantic = s;
        self
    }
    pub fn dot(mut self) -> Self {
        self.dot = true;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let color = self.semantic.color(&theme);
        div()
            .flex()
            .items_center()
            .gap_1()
            .px_1p5()
            .py_px()
            .rounded_full()
            .text_size(px(11.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .bg(color.opacity(0.14))
            .text_color(color)
            .when(self.dot, |el| {
                el.child(
                    div()
                        .size(px(6.))
                        .rounded_full()
                        .bg(color),
                )
            })
            .child(self.label)
    }
}

// ------------------------- Tag -------------------------

#[derive(IntoElement)]
pub struct Tag {
    label: SharedString,
    closable: bool,
    on_close: Option<Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Tag {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            closable: false,
            on_close: None,
        }
    }
    pub fn closable(
        mut self,
        on_close: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.closable = true;
        self.on_close = Some(Box::new(on_close));
        self
    }
}

impl RenderOnce for Tag {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_px()
            .rounded(theme.radius_sm)
            .bg(theme.bg_hover)
            .border_1()
            .border_color(theme.border)
            .text_size(px(12.))
            .text_color(theme.fg_muted)
            .child(self.label)
            .when_some(self.on_close.filter(|_| self.closable), |el, cb| {
                el.child(
                    div()
                        .id("tag-close")
                        .cursor(gpui::CursorStyle::PointingHand)
                        .hover(|s| s.text_color(theme.fg))
                        .on_click(move |e, w, cx| cb(e, w, cx))
                        .child(crate::icons::Icon::new("x").size(px(10.))),
                )
            })
    }
}

// ------------------------- Avatar -------------------------

#[derive(IntoElement)]
pub struct Avatar {
    name: SharedString,
    size: Pixels,
}

impl Avatar {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            size: px(28.),
        }
    }
    pub fn size(mut self, s: Pixels) -> Self {
        self.size = s;
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let bg = crate::util::hash_color(&self.name);
        let initial: SharedString = self
            .name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "?".into())
            .into();
        let font_size = self.size * 0.42;
        div()
            .flex()
            .items_center()
            .justify_center()
            .size(self.size)
            .rounded_full()
            .bg(bg)
            .text_color(crate::theme::hsla(0., 0., 1., 0.95))
            .text_size(font_size)
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .when(!theme.is_dark(), |el| {
                el.text_color(crate::theme::hsla(0., 0., 1., 1.))
            })
            .child(initial)
    }
}

// ------------------------- Divider -------------------------

#[derive(IntoElement)]
pub struct Divider {
    vertical: bool,
}

pub fn divider() -> Divider {
    Divider { vertical: false }
}

pub fn divider_v() -> Divider {
    Divider { vertical: true }
}

impl RenderOnce for Divider {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        if self.vertical {
            div().w(px(1.)).h_full().bg(theme.border)
        } else {
            div().h(px(1.)).w_full().bg(theme.border)
        }
    }
}

// ------------------------- Kbd -------------------------

#[derive(IntoElement)]
pub struct Kbd {
    label: SharedString,
}

impl Kbd {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self { label: label.into() }
    }
}

impl RenderOnce for Kbd {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .px_1p5()
            .py(px(1.))
            .min_w(px(20.))
            .flex()
            .justify_center()
            .rounded(px(4.))
            .bg(theme.bg_hover)
            .border_1()
            .border_color(theme.border_strong)
            .text_size(px(11.))
            .font_family(theme.font_mono.clone())
            .text_color(theme.fg_muted)
            .child(self.label)
    }
}

// ------------------------- Spinner -------------------------

#[derive(IntoElement)]
pub struct Spinner {
    size: Pixels,
    color: Option<Hsla>,
}

impl Spinner {
    pub fn new() -> Self {
        Self { size: px(16.), color: None }
    }
    pub fn size(mut self, s: Pixels) -> Self {
        self.size = s;
        self
    }
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let color = self.color.unwrap_or(theme.primary);
        gpui::svg()
            .path("icons/refresh.svg")
            .size(self.size)
            .text_color(color)
            .with_animation(
                "spinner-anim",
                gpui::Animation::new(std::time::Duration::from_millis(800))
                    .repeat()
                    .with_easing(gpui::linear),
                |el, delta| {
                    el.with_transformation(gpui::Transformation::rotate(gpui::percentage(delta)))
                },
            )
    }
}

// ------------------------- ProgressBar -------------------------

#[derive(IntoElement)]
pub struct ProgressBar {
    value: f64,
    height: Pixels,
    color: Option<Hsla>,
    indeterminate: bool,
}

impl ProgressBar {
    pub fn new(value: f64) -> Self {
        Self {
            value: value.clamp(0., 1.),
            height: px(6.),
            color: None,
            indeterminate: false,
        }
    }
    pub fn height(mut self, h: Pixels) -> Self {
        self.height = h;
        self
    }
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
        self
    }
    pub fn indeterminate(mut self) -> Self {
        self.indeterminate = true;
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let color = self.color.unwrap_or(theme.primary);
        let fill_w = if self.indeterminate {
            relative(0.3)
        } else {
            relative(self.value as f32)
        };
        let fill_el = div().w(fill_w).h_full().rounded_full().bg(color);
        let fill_any = if self.indeterminate {
            fill_el
                .with_animation(
                    "progress-indet",
                    gpui::Animation::new(std::time::Duration::from_millis(1200))
                        .repeat()
                        .with_easing(gpui::ease_in_out),
                    |el, delta| {
                        let pos = delta * 0.7;
                        el.left(relative(pos as f32))
                    },
                )
                .into_any_element()
        } else {
            fill_el.into_any_element()
        };
        div()
            .w_full()
            .h(self.height)
            .rounded_full()
            .bg(theme.bg_hover)
            .overflow_hidden()
            .child(fill_any)
    }
}

// ------------------------- Skeleton -------------------------

#[derive(IntoElement)]
pub struct Skeleton {
    width: Pixels,
    height: Pixels,
    round: bool,
}

impl Skeleton {
    pub fn new(width: Pixels, height: Pixels) -> Self {
        Self { width, height, round: false }
    }
    pub fn text_line() -> Self {
        Self::new(px(220.), px(12.))
    }
    pub fn round(mut self) -> Self {
        self.round = true;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .w(self.width)
            .h(self.height)
            .rounded(if self.round { self.height / 2. } else { theme.radius_sm })
            .bg(theme.bg_hover)
            .with_animation(
                "skeleton-pulse",
                gpui::Animation::new(std::time::Duration::from_millis(1100))
                    .repeat()
                    .with_easing(gpui::ease_in_out),
                |el, delta| el.opacity(0.5 + 0.35 * delta),
            )
    }
}

// ------------------------- Card -------------------------

#[derive(IntoElement)]
pub struct Card {
    title: Option<SharedString>,
    children: Vec<gpui::AnyElement>,
    padding: Pixels,
    clickable: bool,
    on_click: Option<Box<dyn Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            title: None,
            children: Vec::new(),
            padding: px(16.),
            clickable: false,
            on_click: None,
        }
    }
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn padding(mut self, p: Pixels) -> Self {
        self.padding = p;
        self
    }
    pub fn on_click(
        mut self,
        f: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.clickable = true;
        self.on_click = Some(Box::new(f));
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let mut el = div()
            .id("card")
            .flex()
            .flex_col()
            .gap_2()
            .bg(theme.bg_panel)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_lg)
            .p(self.padding)
            .children(self.children);
        if let Some(title) = self.title {
            el = el.child(
                div()
                    .text_size(px(14.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(theme.fg)
                    .child(title),
            );
        }
        if let Some(cb) = self.on_click.filter(|_| self.clickable) {
            el = el
                .cursor(gpui::CursorStyle::PointingHand)
                .hover(|s| s.bg(theme.bg_hover))
                .on_click(move |e, w, cx| cb(e, w, cx));
        }
        el
    }
}

// ------------------------- EmptyState -------------------------

#[derive(IntoElement)]
pub struct EmptyState {
    icon: SharedString,
    title: SharedString,
    description: SharedString,
    action: Option<crate::components::button::Button>,
}

impl EmptyState {
    pub fn new(
        icon: impl Into<SharedString>,
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
    ) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            description: description.into(),
            action: None,
        }
    }
    pub fn action(mut self, b: crate::components::button::Button) -> Self {
        self.action = Some(b);
        self
    }
}

impl RenderOnce for EmptyState {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .size_full()
            .text_color(theme.fg_subtle)
            .child(
                crate::icons::Icon::new(self.icon)
                    .size(px(40.))
                    .opacity(0.5),
            )
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(theme.fg_muted)
                    .child(self.title),
            )
            .child(div().text_size(px(12.)).child(self.description))
            .children(self.action)
    }
}

// ------------------------- Stat -------------------------

#[derive(IntoElement)]
pub struct Stat {
    label: SharedString,
    value: SharedString,
    delta: Option<(f64, bool)>, // (百分比, 是否正向)
}

impl Stat {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            delta: None,
        }
    }
    pub fn delta(mut self, pct: f64, positive: bool) -> Self {
        self.delta = Some((pct, positive));
        self
    }
}

impl RenderOnce for Stat {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.fg_subtle)
                    .child(self.label),
            )
            .child(
                div()
                    .text_size(px(22.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme.fg)
                    .child(self.value),
            )
            .children(self.delta.map(|(pct, pos)| {
                div()
                    .text_size(px(11.))
                    .text_color(if pos { theme.success } else { theme.danger })
                    .child(format!("{}{}%", if pos { "↑ " } else { "↓ " }, pct))
            }))
    }
}

// ------------------------- Tooltip 视图 -------------------------

pub struct SimpleTooltip {
    text: SharedString,
}

impl Render for SimpleTooltip {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .px_2()
            .py_1()
            .rounded(theme.radius_sm)
            .bg(theme.bg_elevated)
            .border_1()
            .border_color(theme.border_strong)
            .text_size(px(12.))
            .text_color(theme.fg)
            .shadow_md()
            .child(self.text.clone())
    }
}

/// 供 `.tooltip(...)` 闭包调用
pub fn tooltip_view(text: &str, _window: &mut Window, cx: &mut App) -> AnyView {
    cx.new(|_| SimpleTooltip {
        text: text.to_string().into(),
    })
    .into()
}

// ------------------------- 色块 -------------------------

#[derive(IntoElement)]
pub struct ColorSwatch {
    color: Hsla,
    size: Pixels,
    label: Option<SharedString>,
}

impl ColorSwatch {
    pub fn new(color: Hsla) -> Self {
        Self {
            color,
            size: px(48.),
            label: None,
        }
    }
    pub fn size(mut self, s: Pixels) -> Self {
        self.size = s;
        self
    }
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
}

impl RenderOnce for ColorSwatch {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_1()
            .child(
                div()
                    .size(self.size)
                    .rounded(theme.radius_md)
                    .bg(self.color)
                    .border_1()
                    .border_color(theme.border),
            )
            .children(self.label.map(|l| {
                div()
                    .text_size(px(10.))
                    .text_color(theme.fg_subtle)
                    .child(l)
            }))
    }
}
