//! 按钮组件:5 变体 × 3 尺寸 × disabled/loading + 图标按钮。

use gpui::{
    prelude::FluentBuilder, px, App, ClickEvent, ElementId, Hsla, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Styled, Window, div, prelude::*,
};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

impl ButtonSize {
    fn height(self) -> Pixels {
        match self {
            ButtonSize::Small => px(26.),
            ButtonSize::Medium => px(32.),
            ButtonSize::Large => px(40.),
        }
    }
    fn padding_x(self) -> Pixels {
        match self {
            ButtonSize::Small => px(8.),
            ButtonSize::Medium => px(12.),
            ButtonSize::Large => px(16.),
        }
    }
    fn text_size(self) -> Pixels {
        match self {
            ButtonSize::Small => px(12.),
            ButtonSize::Medium => px(13.),
            ButtonSize::Large => px(15.),
        }
    }
    fn gap(self) -> Pixels {
        match self {
            ButtonSize::Small => px(4.),
            _ => px(6.),
        }
    }
}

type OnClick = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    loading: bool,
    full_width: bool,
    on_click: Option<OnClick>,
    tooltip: Option<SharedString>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            variant: ButtonVariant::Secondary,
            size: ButtonSize::Medium,
            disabled: false,
            loading: false,
            full_width: false,
            on_click: None,
            tooltip: None,
        }
    }

    pub fn icon(mut self, name: impl Into<SharedString>) -> Self {
        self.icon = Some(name.into());
        self
    }

    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn link(mut self) -> Self {
        self.variant = ButtonVariant::Link;
        self
    }

    pub fn size(mut self, s: ButtonSize) -> Self {
        self.size = s;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = ButtonSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = ButtonSize::Large;
        self
    }

    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn loading(mut self, v: bool) -> Self {
        self.loading = v;
        self
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn tooltip(mut self, t: impl Into<SharedString>) -> Self {
        self.tooltip = Some(t.into());
        self
    }

    pub fn on_click(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let (bg, fg, border, hover_bg, active_bg): (Hsla, Hsla, Hsla, Hsla, Hsla) =
            match self.variant {
                ButtonVariant::Primary => (
                    theme.primary,
                    theme.primary_fg,
                    crate::theme::hsla(0., 0., 0., 0.),
                    theme.primary_hover,
                    theme.primary,
                ),
                ButtonVariant::Secondary => (
                    theme.bg_elevated,
                    theme.fg,
                    theme.border,
                    theme.bg_hover,
                    theme.bg_active,
                ),
                ButtonVariant::Danger => (
                    theme.danger,
                    theme.primary_fg,
                    crate::theme::hsla(0., 0., 0., 0.),
                    theme.danger_hover,
                    theme.danger,
                ),
                ButtonVariant::Ghost => (
                    crate::theme::hsla(0., 0., 0., 0.),
                    theme.fg_muted,
                    crate::theme::hsla(0., 0., 0., 0.),
                    theme.bg_hover,
                    theme.bg_active,
                ),
                ButtonVariant::Link => (
                    crate::theme::hsla(0., 0., 0., 0.),
                    theme.primary,
                    crate::theme::hsla(0., 0., 0., 0.),
                    theme.bg_hover,
                    theme.bg_active,
                ),
            };
        let dim = |c: Hsla| c.opacity(0.45);
        let (bg, fg, border) = if self.disabled || self.loading {
            (dim(bg), dim(fg), border)
        } else {
            (bg, fg, border)
        };

        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(self.size.gap())
            .h(self.size.height())
            .px(self.size.padding_x())
            .text_size(self.size.text_size())
            .font_weight(gpui::FontWeight::MEDIUM)
            .whitespace_nowrap()
            .rounded(theme.radius_md)
            .bg(bg)
            .text_color(fg)
            .when(!matches!(self.variant, ButtonVariant::Ghost | ButtonVariant::Link), |el| {
                el.border_1().border_color(border)
            })
            .when(self.full_width, |el| el.w_full())
            .when_some(self.on_click.filter(|_| !self.disabled && !self.loading), |el, cb| {
                el.cursor(gpui::CursorStyle::PointingHand)
                    .hover(move |s| s.bg(hover_bg))
                    .active(move |s| s.bg(active_bg))
                    .on_click(move |event, window, cx| cb(event, window, cx))
            });

        if let Some(icon) = self.icon {
            el = el.child(
                crate::icons::Icon::new(icon)
                    .size(match self.size {
                        ButtonSize::Small => px(12.),
                        ButtonSize::Medium => px(14.),
                        ButtonSize::Large => px(16.),
                    })
                    .map(|i| {
                        // 图标继承文字色:svg 用 text_color 渲染
                        i
                    }),
            );
        }
        if self.loading {
            el = el.child(crate::components::basic::Spinner::new().size(px(12.)));
        }
        el = el.child(self.label);
        if let Some(tip) = self.tooltip {
            let tip = tip.clone();
            el = el.tooltip(move |window, cx| {
                crate::components::basic::tooltip_view(&tip, window, cx)
            });
        }
        el
    }
}

/// 图标按钮(方形)
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: SharedString,
    size: Pixels,
    icon_px: Pixels,
    disabled: bool,
    danger: bool,
    on_click: Option<OnClick>,
    tooltip: Option<SharedString>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            size: px(28.),
            icon_px: px(16.),
            disabled: false,
            danger: false,
            on_click: None,
            tooltip: None,
        }
    }

    pub fn size(mut self, s: Pixels) -> Self {
        self.size = s;
        self
    }

    pub fn icon_size(mut self, s: Pixels) -> Self {
        self.icon_px = s;
        self
    }

    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn tooltip(mut self, t: impl Into<SharedString>) -> Self {
        self.tooltip = Some(t.into());
        self
    }

    pub fn on_click(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let fg = if self.disabled {
            theme.fg_subtle
        } else if self.danger {
            theme.danger
        } else {
            theme.fg_muted
        };
        let hover_bg = if self.danger {
            theme.danger.opacity(0.12)
        } else {
            theme.bg_hover
        };
        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(self.size)
            .rounded(theme.radius_md)
            .text_color(fg)
            .when(self.disabled, |el| el.opacity(0.4))
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |el, cb| {
                    el.cursor(gpui::CursorStyle::PointingHand)
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(move |event, window, cx| cb(event, window, cx))
                },
            )
            .when_some(self.tooltip, |el, tip| {
                el.tooltip(move |window, cx| {
                    crate::components::basic::tooltip_view(&tip, window, cx)
                })
            })
            .child(crate::icons::Icon::new(self.icon).size(self.icon_px))
    }
}
