//! 选择器与菜单:Select(下拉选择)/ Menu(下拉/右键菜单,父级状态驱动)。

use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, Context, ElementId, Entity, IntoElement,
    ParentElement, Pixels, Point, RenderOnce, SharedString, Styled, Window, prelude::*,
};
use gpui::AnimationExt as _;

use crate::theme::Theme;

// ------------------------- Select -------------------------

/// 下拉选择状态实体
pub struct SelectState {
    pub open: bool,
    pub selected: Option<usize>,
    pub options: Vec<SharedString>,
    on_change: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl SelectState {
    pub fn new(cx: &mut Context<Self>, options: Vec<SharedString>) -> Self {
        Self {
            open: false,
            selected: if options.is_empty() { None } else { Some(0) },
            options,
            on_change: None,
        }
    }

    pub fn placeholder(mut self) -> Self {
        self.selected = None;
        self
    }

    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn set_options(&mut self, options: Vec<SharedString>, cx: &mut Context<Self>) {
        self.options = options;
        self.selected = self.selected.min(self.options.len().checked_sub(1));
        cx.notify();
    }

    pub fn selected_label(&self) -> SharedString {
        self.selected
            .and_then(|ix| self.options.get(ix))
            .cloned()
            .unwrap_or_else(|| "请选择…".into())
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    pub fn choose(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.open = false;
        let changed = self.selected != Some(ix);
        self.selected = Some(ix);
        cx.notify();
        if changed && let Some(cb) = self.on_change.as_ref() {
            cb(ix, window, cx);
        }
    }
}

/// 下拉选择组件(菜单为触发器下方的绝对定位面板)
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    state: Entity<SelectState>,
    width: Pixels,
}

pub fn select(id: impl Into<ElementId>, state: &Entity<SelectState>) -> Select {
    Select {
        id: id.into(),
        state: state.clone(),
        width: px(160.),
    }
}

impl Select {
    pub fn width(mut self, w: Pixels) -> Self {
        self.width = w;
        self
    }
}

impl RenderOnce for Select {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let state = self.state.clone();
        let state_for_click = state.clone();
        let state_for_item = state.clone();
        let state_for_close = state.clone();
        let open = state.read(cx).open;

        div()
            .id(self.id)
            .relative()
            .w(self.width)
            .child(
                // 触发器
                div()
                    .id("select-trigger")
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(30.))
                    .px_2()
                    .rounded(theme.radius_md)
                    .bg(theme.bg_input)
                    .border_1()
                    .border_color(if open { theme.primary } else { theme.border })
                    .cursor(gpui::CursorStyle::PointingHand)
                    .hover(|s| s.border_color(theme.border_strong))
                    .text_size(px(13.))
                    .text_color(theme.fg)
                    .on_click(move |_: &ClickEvent, _window, cx| {
                        state_for_click.update(cx, |s, cx| s.toggle(cx));
                    })
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(state.read(cx).selected_label().to_string()),
                    )
                    .child(
                        crate::icons::Icon::new(if open { "chevron-up" } else { "chevron-down" })
                            .size(px(12.))
                            .text_color(theme.fg_subtle),
                    ),
            )
            .when(open, |el| {
                el.child(
                    // 菜单面板
                    div()
                        .id("select-menu")
                        .absolute()
                        .top(px(34.))
                        .left_0()
                        .w_full()
                        .max_h(px(240.))
                        .overflow_y_scroll()
                        .rounded(theme.radius_md)
                        .bg(theme.bg_elevated)
                        .border_1()
                        .border_color(theme.border)
                        .shadow_lg()
                        .p_1()
                        .children({
                            let options = state.read(cx).options.clone();
                            let selected = state.read(cx).selected;
                            let st = state_for_item.clone();
                            options
                                .into_iter()
                                .enumerate()
                                .map(move |(ix, label)| {
                                    let st = st.clone();
                                    let is_sel = selected == Some(ix);
                                    div()
                                        .id(("select-item", ix))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px_2()
                                        .py_1()
                                        .rounded(theme.radius_sm)
                                        .text_size(px(13.))
                                        .cursor(gpui::CursorStyle::PointingHand)
                                        .hover(|s| s.bg(theme.bg_hover))
                                        .text_color(if is_sel {
                                            theme.primary
                                        } else {
                                            theme.fg
                                        })
                                        .on_click(move |_, window, cx| {
                                            st.update(cx, |s, cx| s.choose(ix, window, cx));
                                        })
                                        .child(label.to_string())
                                        .when(is_sel, |el| {
                                            el.child(crate::icons::Icon::new("check").size(px(12.)))
                                        })
                                })
                                .collect::<Vec<_>>()
                        }),
                )
                .child(
                    // 点击外部关闭的透明遮罩
                    div()
                        .id("select-backdrop")
                        .absolute()
                        .top(px(34.))
                        .left_0()
                        .w_full()
                        .h(px(400.))
                        .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                            state_for_close.update(cx, |s, cx| s.close(cx));
                        }),
                )
            })
    }
}

// ------------------------- Menu(父级状态驱动) -------------------------

pub type MenuHandler = std::rc::Rc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub enum MenuItem {
    Separator,
    Action {
        label: SharedString,
        icon: Option<SharedString>,
        shortcut: Option<SharedString>,
        danger: bool,
        disabled: bool,
        handler: MenuHandler,
    },
}

impl MenuItem {
    pub fn action(
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        MenuItem::Action {
            label: label.into(),
            icon: None,
            shortcut: None,
            danger: false,
            disabled: false,
            handler: std::rc::Rc::new(handler),
        }
    }
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        if let MenuItem::Action { icon: i, .. } = &mut self {
            *i = Some(icon.into());
        }
        self
    }
    pub fn shortcut(mut self, s: impl Into<SharedString>) -> Self {
        if let MenuItem::Action { shortcut, .. } = &mut self {
            *shortcut = Some(s.into());
        }
        self
    }
    pub fn danger(mut self) -> Self {
        if let MenuItem::Action { danger, .. } = &mut self {
            *danger = true;
        }
        self
    }
    pub fn disabled(mut self) -> Self {
        if let MenuItem::Action { disabled, .. } = &mut self {
            *disabled = true;
        }
        self
    }
    pub fn separator() -> Self {
        MenuItem::Separator
    }
}

/// 渲染菜单覆盖层(放置于根容器内,position 为窗口坐标)
#[derive(IntoElement)]
pub struct MenuOverlay {
    position: Point<Pixels>,
    items: Vec<MenuItem>,
    on_close: Box<dyn Fn(&mut Window, &mut App) + 'static>,
}

impl MenuOverlay {
    pub fn new(position: Point<Pixels>, items: Vec<MenuItem>) -> Self {
        Self {
            position,
            items,
            on_close: Box::new(|_, _| {}),
        }
    }

    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Box::new(f);
        self
    }
}

impl RenderOnce for MenuOverlay {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let on_close = std::rc::Rc::new(self.on_close);
        let on_close_backdrop = on_close.clone();

        div()
            .id("menu-root")
            .absolute()
            .inset_0()
            .child(
                // 背景:点击关闭
                div()
                    .absolute()
                    .inset_0()
                    .on_mouse_down(gpui::MouseButton::Left, {
                        let cb = on_close_backdrop.clone();
                        move |_, window, cx| cb(window, cx)
                    })
                    .on_mouse_down(gpui::MouseButton::Right, {
                        let cb = on_close_backdrop.clone();
                        move |_, window, cx| cb(window, cx)
                    }),
            )
            .child(
                div()
                    .absolute()
                    .left(self.position.x)
                    .top(self.position.y)
                    .w(px(200.))
                    .rounded(theme.radius_md)
                    .bg(theme.bg_elevated)
                    .border_1()
                    .border_color(theme.border)
                    .shadow_lg()
                    .p_1()
                    .children(self.items.into_iter().enumerate().map(move |(ix, item)| {
                        let on_close = on_close.clone();
                        match item {
                            MenuItem::Separator => div().h(px(1.)).my_1().bg(theme.border).into_any_element(),
                            MenuItem::Action {
                                label,
                                icon,
                                shortcut,
                                danger,
                                disabled,
                                handler,
                            } => {
                                let fg = if disabled {
                                    theme.fg_subtle
                                } else if danger {
                                    theme.danger
                                } else {
                                    theme.fg
                                };
                                div()
                                    .id(("menu-item", ix))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_2()
                                    .py_1p5()
                                    .rounded(theme.radius_sm)
                                    .text_size(px(13.))
                                    .text_color(fg)
                                    .when(!disabled, |el| {
                                        el.cursor(gpui::CursorStyle::PointingHand)
                                            .hover(move |s| {
                                                s.bg(if danger {
                                                    theme.danger.opacity(0.12)
                                                } else {
                                                    theme.bg_hover
                                                })
                                            })
                                            .on_click(move |_, window, cx| {
                                                (on_close.clone())(window, cx);
                                                handler(window, cx);
                                            })
                                    })
                                    .children(icon.map(|i| crate::icons::Icon::new(i).size(px(14.))))
                                    .child(div().flex_1().truncate().child(label.to_string()))
                                    .children(shortcut.map(|s| {
                                        crate::components::basic::Kbd::new(s)
                                    }))
                                    .into_any_element()
                            }
                        }
                    })),
            )
    }
}
