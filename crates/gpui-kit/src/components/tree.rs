//! 树形视图(父级持有展开状态;数据由调用方展平)。

use gpui::{
    div, prelude::FluentBuilder, px, App, IntoElement, Pixels, RenderOnce, SharedString, Styled,
    Window, prelude::*,
};

use crate::theme::Theme;

/// 展平后的树行数据
#[derive(Clone)]
pub struct TreeRow {
    pub key: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
    pub selected: bool,
    pub badge: Option<SharedString>,
}

#[derive(IntoElement)]
pub struct TreeView {
    rows: Vec<TreeRow>,
    indent: Pixels,
    on_toggle: Option<std::rc::Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_select: Option<std::rc::Rc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
}

impl TreeView {
    pub fn new(rows: Vec<TreeRow>) -> Self {
        Self {
            rows,
            indent: px(16.),
            on_toggle: None,
            on_select: None,
        }
    }

    pub fn on_toggle(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(std::rc::Rc::new(f));
        self
    }

    pub fn on_select(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(f));
        self
    }
}

impl RenderOnce for TreeView {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let on_toggle = self.on_toggle.clone();
        let on_select = self.on_select.clone();

        div()
            .flex()
            .flex_col()
            .gap_px()
            .children(self.rows.into_iter().map(move |row| {
                let on_toggle = on_toggle.clone();
                let on_select = on_select.clone();
                let key = row.key.clone();
                div()
                    .id(row.key.clone())
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .pl(self.indent * row.depth as f32)
                    .pr_2()
                    .py_1()
                    .rounded(theme.radius_sm)
                    .text_size(px(13.))
                    .when(row.selected, |el| {
                        el.bg(theme.primary.opacity(0.14)).text_color(theme.primary)
                    })
                    .when(!row.selected, |el| {
                        el.text_color(theme.fg_muted)
                            .hover(|s| s.bg(theme.bg_hover).text_color(theme.fg))
                    })
                    .cursor(gpui::CursorStyle::PointingHand)
                    .on_click(move |_, window, cx| {
                        if let Some(cb) = on_select.as_ref() {
                            cb(&key, window, cx);
                        }
                        if row.has_children && let Some(cb) = on_toggle.as_ref() {
                            cb(&key, window, cx);
                        }
                    })
                    // 展开箭头
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(14.))
                            .child(if row.has_children {
                                crate::icons::Icon::new(if row.expanded {
                                    "chevron-down"
                                } else {
                                    "chevron-right"
                                })
                                .size(px(12.))
                                .text_color(theme.fg_subtle)
                            } else {
                                crate::icons::Icon::new("minus")
                                    .size(px(12.))
                                    .text_color(crate::theme::hsla(0., 0., 0., 0.))
                            }),
                    )
                    .children(row.icon.map(|i| {
                        crate::icons::Icon::new(i)
                            .size(px(14.))
                            .text_color(if row.selected {
                                theme.primary
                            } else {
                                theme.fg_subtle
                            })
                    }))
                    .child(div().flex_1().min_w_0().truncate().child(row.label.to_string()))
                    .children(row.badge.map(|b| {
                        crate::components::basic::Badge::new(b).semantic(crate::components::basic::Semantic::Neutral)
                    }))
            }))
    }
}
