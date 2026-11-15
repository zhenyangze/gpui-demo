//! 数据表格(轻量,行数 <200 场景;大量数据请用 uniform_list 自行封装)。

use gpui::{
    div, px, App, ClickEvent, IntoElement, ParentElement, Pixels, RenderOnce, SharedString,
    Styled, Window, prelude::*,
};

use crate::theme::Theme;

pub struct TableColumn {
    pub label: SharedString,
    pub width: Pixels,
    pub sortable: bool,
}

impl TableColumn {
    pub fn new(label: impl Into<SharedString>, width: Pixels) -> Self {
        Self {
            label: label.into(),
            width,
            sortable: false,
        }
    }
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

#[derive(IntoElement)]
pub struct Table {
    columns: Vec<TableColumn>,
    rows: Vec<Vec<gpui::AnyElement>>,
    sort_column: Option<usize>,
    sort_ascending: bool,
    selected_row: Option<usize>,
    on_header_click: Option<std::rc::Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    on_row_click: Option<std::rc::Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    zebra: bool,
    max_height: Option<Pixels>,
}

impl Table {
    pub fn new(columns: Vec<TableColumn>, rows: Vec<Vec<gpui::AnyElement>>) -> Self {
        Self {
            columns,
            rows,
            sort_column: None,
            sort_ascending: true,
            selected_row: None,
            on_header_click: None,
            on_row_click: None,
            zebra: true,
            max_height: None,
        }
    }
    pub fn sort(mut self, column: usize, ascending: bool) -> Self {
        self.sort_column = Some(column);
        self.sort_ascending = ascending;
        self
    }
    pub fn selected_row(mut self, row: usize) -> Self {
        self.selected_row = Some(row);
        self
    }
    pub fn on_header_click(
        mut self,
        f: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_header_click = Some(std::rc::Rc::new(f));
        self
    }
    pub fn on_row_click(
        mut self,
        f: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_click = Some(std::rc::Rc::new(f));
        self
    }
    pub fn zebra(mut self, v: bool) -> Self {
        self.zebra = v;
        self
    }
    pub fn max_height(mut self, h: Pixels) -> Self {
        self.max_height = Some(h);
        self
    }
}

impl RenderOnce for Table {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let on_header = self.on_header_click.clone();
        let on_row = self.on_row_click.clone();

        div()
            .id("kit-table")
            .flex()
            .flex_col()
            .w_full()
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md)
            .overflow_hidden()
            .when_some(self.max_height, |el, h| el.max_h(h).overflow_y_scroll())
            // 表头
            .child(
                div()
                    .flex()
                    .flex_row()
                    .bg(theme.bg_hover)
                    .px_2()
                    .py_1p5()
                    .text_size(px(11.))
                    .text_color(theme.fg_muted)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .children(self.columns.iter().enumerate().map(|(ix, col)| {
                        let mut el = div()
                            .id(("table-header", ix))
                            .w(col.width)
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .gap_1()
                            .truncate()
                            .child(col.label.to_uppercase())
                            .when(self.sort_column == Some(ix), |el| {
                                el.child(
                                    crate::icons::Icon::new(if self.sort_ascending {
                                        "arrow-up"
                                    } else {
                                        "arrow-down"
                                    })
                                    .size(px(10.)),
                                )
                            });
                        if col.sortable && let Some(cb) = on_header.as_ref() {
                            let cb = cb.clone();
                            el = el
                                .cursor(gpui::CursorStyle::PointingHand)
                                .hover(|s| s.text_color(theme.fg))
                                .on_click(move |_: &ClickEvent, window, cx| cb(ix, window, cx));
                        }
                        el
                    })),
            )
            // 数据行
            .children(
                self.rows
                    .into_iter()
                    .enumerate()
                    .map(move |(row_ix, cells)| {
                        let on_row = on_row.clone();
                        let zebra_bg = if self.zebra && row_ix % 2 == 1 {
                            theme.bg_hover.opacity(0.5)
                        } else {
                            crate::theme::hsla(0., 0., 0., 0.)
                        };
                        let selected = self.selected_row == Some(row_ix);
                        div()
                            .id(("table-row", row_ix))
                            .flex()
                            .flex_row()
                            .px_2()
                            .py_1p5()
                            .text_size(px(12.))
                            .text_color(theme.fg)
                            .border_b_1()
                            .border_color(theme.border.opacity(0.5))
                            .bg(if selected { theme.primary.opacity(0.10) } else { zebra_bg })
                            .when(selected, |el| el.text_color(theme.primary))
                            .when_some(on_row, |el, cb| {
                                el.cursor(gpui::CursorStyle::PointingHand)
                                    .hover(move |s| {
                                        if !selected {
                                            s.bg(theme.bg_hover)
                                        } else {
                                            s
                                        }
                                    })
                                    .on_click(move |_: &ClickEvent, window, cx| {
                                        cb(row_ix, window, cx)
                                    })
                            })
                            .children(cells.into_iter().enumerate().map(|(col_ix, cell)| {
                                let width = self
                                    .columns
                                    .get(col_ix)
                                    .map(|c| c.width)
                                    .unwrap_or(px(80.));
                                div()
                                    .w(width)
                                    .flex_shrink_0()
                                    .min_w_0()
                                    .truncate()
                                    .child(cell)
                            }))
                    }),
            )
    }
}
