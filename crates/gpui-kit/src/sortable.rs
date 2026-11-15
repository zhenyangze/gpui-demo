//! # 拖拽排序框架(L2)— gpui on_drag/on_drop 的泛型封装
//!
//! `SortableRow` 渲染"可拖拽行":行内容由调用方提供,框架负责拖拽浮层、
//! 落点高亮与 drop 回调;`reorder/apply_reorder` 为纯逻辑重排(单测覆盖)。

use gpui::{div, prelude::*, px, App, ClickEvent, Context, IntoElement, ParentElement, Render, RenderOnce, SharedString, Styled, Window};
use std::rc::Rc;

/// 拖拽载荷:列表 id + 条目当前下标
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SortDrag {
    pub list_id: u64,
    pub index: usize,
}

/// 拖拽时的浮层视图(半透明胶囊)
pub struct SortDragGhost {
    pub label: SharedString,
}

impl Render for SortDragGhost {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<crate::theme::Theme>();
        div()
            .px_3()
            .py_2()
            .rounded(theme.radius_md)
            .bg(theme.primary.opacity(0.85))
            .text_color(theme.primary_fg)
            .text_size(px(12.))
            .shadow_lg()
            .child(self.label.clone())
    }
}

type RowCb = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type DropCb = Rc<dyn Fn(usize, usize, &mut Window, &mut App) + 'static>;

/// 可拖拽行:行内容任意(图标/文本…),框架注入拖拽/落点/点击行为
#[derive(IntoElement)]
pub struct SortableRow {
    pub list_id: u64,
    pub index: usize,
    /// 拖拽浮层显示的文本
    pub ghost_label: SharedString,
    /// 行内容元素
    pub content: gpui::AnyElement,
    /// 当前行是否为落点目标(高亮)
    pub drop_target: bool,
    pub on_click: Option<RowCb>,
    pub on_drop: Option<DropCb>,
}

impl SortableRow {
    pub fn new(index: usize, ghost_label: impl Into<SharedString>, content: impl IntoElement) -> Self {
        Self {
            list_id: 0,
            index,
            ghost_label: ghost_label.into(),
            content: content.into_any_element(),
            drop_target: false,
            on_click: None,
            on_drop: None,
        }
    }
    pub fn list_id(mut self, id: u64) -> Self {
        self.list_id = id;
        self
    }
    pub fn drop_target(mut self, v: bool) -> Self {
        self.drop_target = v;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
    pub fn on_drop(mut self, f: impl Fn(usize, usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_drop = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SortableRow {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<crate::theme::Theme>();
        let Self { list_id, index, ghost_label, content, drop_target, on_click, on_drop } = self;

        div()
            .id(("sort-row", (list_id as usize).wrapping_mul(10000) + index))
            .flex()
            .items_center()
            .gap_2()
            .px_2()
            .py_2()
            .mb_1()
            .rounded(theme.radius_md)
            .bg(if drop_target {
                theme.primary.opacity(0.16)
            } else {
                theme.bg_input
            })
            .border_1()
            .border_color(if drop_target {
                theme.primary
            } else {
                theme.border
            })
            .text_size(px(12.))
            .cursor_move()
            .hover(|s| s.border_color(theme.border_strong))
            .on_drag(
                SortDrag { list_id, index },
                move |drag, _, _, cx| {
                    cx.new(|_| SortDragGhost {
                        label: format!("↕ {}", ghost_label).into(),
                    })
                },
            )
            .on_drop(move |drag: &SortDrag, window, cx| {
                if drag.list_id == list_id
                    && let Some(cb) = on_drop.as_ref()
                {
                    cb(drag.index, index, window, cx);
                }
            })
            .when_some(on_click, |el, cb| {
                el.cursor_pointer().on_click(move |e, w, cx| cb(e, w, cx))
            })
            .child(content)
    }
}

/// 核心纯逻辑:计算重排后的索引顺序
pub fn reorder(len: usize, from: usize, to: usize) -> Option<Vec<usize>> {
    if len == 0 || from >= len || to >= len || from == to {
        return None;
    }
    let mut order: Vec<usize> = (0..len).collect();
    let item = order.remove(from);
    order.insert(to, item);
    Some(order)
}

/// 按索引映射重排一个 Vec
pub fn apply_reorder<T: Clone>(items: &[T], order: &[usize]) -> Vec<T> {
    order.iter().map(|i| items[*i].clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reorder_down() {
        assert_eq!(reorder(5, 0, 2).unwrap(), vec![1, 2, 0, 3, 4]);
    }

    #[test]
    fn test_reorder_up() {
        assert_eq!(reorder(5, 3, 1).unwrap(), vec![0, 3, 1, 2, 4]);
    }

    #[test]
    fn test_reorder_same_is_none() {
        assert!(reorder(5, 2, 2).is_none());
    }

    #[test]
    fn test_reorder_out_of_range() {
        assert!(reorder(3, 3, 0).is_none());
        assert!(reorder(3, 0, 3).is_none());
        assert!(reorder(0, 0, 0).is_none());
    }

    #[test]
    fn test_apply_reorder() {
        let items = vec!["a", "b", "c", "d"];
        let order = reorder(4, 0, 2).unwrap();
        assert_eq!(apply_reorder(&items, &order), vec!["b", "c", "a", "d"]);
    }
}
