//! 导航组件:Tabs / Segmented / Breadcrumb / Toolbar / Sidebar 导航项 / Pagination。

use gpui::{
    div, prelude::FluentBuilder, px, App, ClickEvent, ElementId, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Styled, Window, prelude::*,
};
use gpui::AnimationExt as _;

use crate::theme::Theme;

// ------------------------- Tabs -------------------------

#[derive(IntoElement)]
pub struct Tabs {
    tabs: Vec<SharedString>,
    selected: usize,
    on_change: Option<std::rc::Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    id: &'static str,
}

impl Tabs {
    pub fn new(id: &'static str, tabs: Vec<SharedString>, selected: usize) -> Self {
        Self {
            id,
            tabs,
            selected,
            on_change: None,
        }
    }
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for Tabs {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let id_base = self.id;
        div()
            .flex()
            .border_b_1()
            .border_color(theme.border)
            .children(self.tabs.into_iter().enumerate().map(move |(ix, label)| {
                let selected = self.selected == ix;
                let mut el = div()
                    .id((id_base, ix))
                    .px_3()
                    .py_2()
                    .text_size(px(13.))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .text_color(if selected { theme.primary } else { theme.fg_muted })
                    .border_b_2()
                    .border_color(if selected { theme.primary } else { crate::theme::hsla(0., 0., 0., 0.) })
                    .hover(|s| s.text_color(theme.fg))
                    .child(label.to_string());
                if let Some(cb) = self.on_change.as_ref() {
                    let cb = cb.clone();
                    el = el.on_click(move |_: &ClickEvent, window, cx| cb(ix, window, cx));
                }
                el
            }))
    }
}

// ------------------------- Segmented -------------------------

#[derive(IntoElement)]
pub struct Segmented {
    options: Vec<SharedString>,
    selected: usize,
    on_change: Option<std::rc::Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    id: &'static str,
}

impl Segmented {
    pub fn new(id: &'static str, options: Vec<SharedString>, selected: usize) -> Self {
        Self {
            id,
            options,
            selected,
            on_change: None,
        }
    }
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for Segmented {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let id_base = self.id;
        div()
            .flex()
            .items_center()
            .p(px(2.))
            .gap(px(2.))
            .rounded(theme.radius_md)
            .bg(theme.bg_hover)
            .children(self.options.into_iter().enumerate().map(move |(ix, label)| {
                let selected = self.selected == ix;
                let mut el = div()
                    .id((id_base, ix))
                    .px_3()
                    .py_1()
                    .rounded(theme.radius_sm)
                    .text_size(px(12.))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .text_color(if selected { theme.fg } else { theme.fg_muted })
                    .when(selected, |el| el.bg(theme.bg_elevated).shadow_sm())
                    .hover(|s| s.text_color(theme.fg))
                    .child(label.to_string());
                if let Some(cb) = self.on_change.as_ref() {
                    let cb = cb.clone();
                    el = el.on_click(move |_: &ClickEvent, window, cx| cb(ix, window, cx));
                }
                el
            }))
    }
}

// ------------------------- Breadcrumb -------------------------

#[derive(IntoElement)]
pub struct Breadcrumb {
    parts: Vec<SharedString>,
}

impl Breadcrumb {
    pub fn new(parts: Vec<SharedString>) -> Self {
        Self { parts }
    }
}

impl RenderOnce for Breadcrumb {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_size(px(12.))
            .children(self.parts.into_iter().enumerate().flat_map(|(ix, part)| {
                let mut v = Vec::new();
                if ix > 0 {
                    v.push(
                        crate::icons::Icon::new("chevron-right")
                            .size(px(11.))
                            .text_color(theme.fg_subtle)
                            .into_any_element(),
                    );
                }
                v.push(
                    div()
                        .text_color(if ix == 0 { theme.fg } else { theme.fg_subtle })
                        .child(part.to_string())
                        .into_any_element(),
                );
                v
            }))
    }
}

// ------------------------- Toolbar -------------------------

#[derive(IntoElement)]
pub struct Toolbar {
    children: Vec<gpui::AnyElement>,
    gap: Pixels,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            gap: px(8.),
        }
    }
    pub fn gap(mut self, g: Pixels) -> Self {
        self.gap = g;
        self
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .flex()
            .items_center()
            .px_2()
            .py_1()
            .gap(self.gap)
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.bg_panel)
            .children(self.children)
    }
}

// ------------------------- Sidebar 导航项 -------------------------

#[derive(IntoElement)]
pub struct NavItem {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    badge: Option<SharedString>,
    selected: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl NavItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            selected: false,
            on_click: None,
        }
    }
    pub fn icon(mut self, i: impl Into<SharedString>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn badge(mut self, b: impl Into<SharedString>) -> Self {
        self.badge = Some(b.into());
        self
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
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

impl RenderOnce for NavItem {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_1p5()
            .rounded(theme.radius_md)
            .text_size(px(13.))
            .when(self.selected, |el| {
                el.bg(theme.primary.opacity(0.14)).text_color(theme.primary)
            })
            .when(!self.selected, |el| {
                el.text_color(theme.fg_muted)
                    .hover(|s| s.bg(theme.bg_hover).text_color(theme.fg))
            })
            .when_some(self.on_click, |el, cb| {
                el.cursor(gpui::CursorStyle::PointingHand)
                    .on_click(move |e, window, cx| cb(e, window, cx))
            })
            .children(self.icon.map(|i| {
                crate::icons::Icon::new(i)
                    .size(px(15.))
                    .text_color(if self.selected {
                        theme.primary
                    } else {
                        theme.fg_subtle
                    })
            }))
            .child(div().flex_1().min_w_0().truncate().child(self.label.to_string()))
            .children(self.badge.map(|b| crate::components::basic::Badge::new(b)))
    }
}

// ------------------------- Pagination -------------------------

#[derive(IntoElement)]
pub struct Pagination {
    page: usize,
    total_pages: usize,
    on_change: Option<std::rc::Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
}

impl Pagination {
    pub fn new(page: usize, total_pages: usize) -> Self {
        Self {
            page,
            total_pages,
            on_change: None,
        }
    }
    pub fn on_change(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for Pagination {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let on_change = self.on_change.clone();
        let on_change_prev = self.on_change.clone();
        let on_change_next = self.on_change.clone();
        let page = self.page;
        div()
            .flex()
            .items_center()
            .gap_1()
            .text_size(px(12.))
            .child(
                crate::components::button::IconButton::new("page-prev", "chevron-left")
                    .disabled(page == 0)
                    .on_click(move |_, window, cx| {
                        if page > 0 && let Some(cb) = on_change_prev.as_ref() {
                            cb(page - 1, window, cx);
                        }
                    }),
            )
            .child(
                div()
                    .text_color(theme.fg_muted)
                    .child(format!("{} / {}", self.page + 1, self.total_pages.max(1))),
            )
            .child(
                crate::components::button::IconButton::new("page-next", "chevron-right")
                    .disabled(page + 1 >= self.total_pages)
                    .on_click(move |_, window, cx| {
                        if page + 1 < self.total_pages && let Some(cb) = on_change_next.as_ref() {
                            cb(page + 1, window, cx);
                        }
                    }),
            )
    }
}
