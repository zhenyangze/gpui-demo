//! # 文本输入组件
//!
//! GPUI 无内置输入框,本模块基于官方 `examples/input.rs` 模式实现:
//! - [`InputState`]:受控状态实体(内容/光标/选区/回调);
//! - [`text_input`]:单行输入容器(自绘光标/选区,IME 支持);
//! - [`text_area`]:多行输入容器(按物理行 shape,自适应高度);
//! - [`bind_input_keys`]:注册编辑快捷键(上下文 `TextInput` / `TextArea`)。
//!
//! 多行的回车行为留给使用方绑定(如聊天发送),kit 提供 [`InputNewline`] 动作。

use std::ops::Range;

use gpui::{
    actions, div, fill, px, relative, size, App, AnimationExt as _, Bounds, ClipboardItem,
    Context, CursorStyle, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler,
    FocusHandle, Focusable, GlobalElementId, Hsla, IntoElement, KeyBinding, LayoutId, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point, RenderOnce, ShapedLine,
    SharedString, Style, TextRun, UTF16Selection, Window, prelude::*,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::theme::Theme;

actions!(
    kit_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Submit,
        Newline,
        Paste,
        Cut,
        Copy,
        Up,
        Down,
    ]
);

/// 注册输入类快捷键(由 init_kit 调用)
pub fn bind_input_keys(cx: &mut App) {
    use gpui::KeyBinding as KB;
    cx.bind_keys([
        KB::new("backspace", Backspace, Some("TextInput")),
        KB::new("delete", Delete, Some("TextInput")),
        KB::new("left", Left, Some("TextInput")),
        KB::new("right", Right, Some("TextInput")),
        KB::new("shift-left", SelectLeft, Some("TextInput")),
        KB::new("shift-right", SelectRight, Some("TextInput")),
        KB::new("cmd-a", SelectAll, Some("TextInput")),
        KB::new("cmd-v", Paste, Some("TextInput")),
        KB::new("cmd-c", Copy, Some("TextInput")),
        KB::new("cmd-x", Cut, Some("TextInput")),
        KB::new("home", Home, Some("TextInput")),
        KB::new("end", End, Some("TextInput")),
        KB::new("enter", Submit, Some("TextInput")),
        KB::new("shift-enter", Newline, Some("TextInput")),
        KB::new("up", Up, Some("TextInput")),
        KB::new("down", Down, Some("TextInput")),
        // 多行上下文:enter 不做提交,由使用方绑定(如聊天发送)
        KB::new("backspace", Backspace, Some("TextArea")),
        KB::new("delete", Delete, Some("TextArea")),
        KB::new("left", Left, Some("TextArea")),
        KB::new("right", Right, Some("TextArea")),
        KB::new("shift-left", SelectLeft, Some("TextArea")),
        KB::new("shift-right", SelectRight, Some("TextArea")),
        KB::new("cmd-a", SelectAll, Some("TextArea")),
        KB::new("cmd-v", Paste, Some("TextArea")),
        KB::new("cmd-c", Copy, Some("TextArea")),
        KB::new("cmd-x", Cut, Some("TextArea")),
        KB::new("home", Home, Some("TextArea")),
        KB::new("end", End, Some("TextArea")),
        KB::new("shift-enter", Newline, Some("TextArea")),
        KB::new("up", Up, Some("TextArea")),
        KB::new("down", Down, Some("TextArea")),
    ]);
}

type ChangeCb = Box<dyn Fn(&str, &mut Window, &mut App) + 'static>;
type SubmitCb = Box<dyn Fn(&str, bool, &mut Window, &mut App) + 'static>;

/// 输入框状态实体
pub struct InputState {
    focus_handle: FocusHandle,
    pub(crate) content: SharedString,
    pub(crate) placeholder: SharedString,
    pub(crate) selected_range: Range<usize>,
    pub(crate) selection_reversed: bool,
    pub(crate) marked_range: Option<Range<usize>>,
    pub(crate) last_rows: Option<Vec<(usize, ShapedLine)>>, // (line_start_offset, line)
    pub(crate) last_bounds: Option<Bounds<Pixels>>,
    pub(crate) is_selecting: bool,
    pub(crate) multi_line: bool,
    pub(crate) max_rows: usize,
    on_change: Option<ChangeCb>,
    on_submit: Option<SubmitCb>,
}

impl InputState {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: "".into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_rows: None,
            last_bounds: None,
            is_selecting: false,
            multi_line: false,
            max_rows: 8,
            on_change: None,
            on_submit: None,
        }
    }

    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn multi_line(mut self, rows: usize) -> Self {
        self.multi_line = true;
        self.max_rows = rows.max(2);
        self
    }

    pub fn on_change(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_submit(mut self, f: impl Fn(&str, bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    /// 当前选区(字节偏移)
    pub fn selected_range(&self) -> Range<usize> {
        self.selected_range.clone()
    }

    /// 是否有激活选区
    pub fn has_selection(&self) -> bool {
        !self.selected_range.is_empty()
    }

    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.marked_range = None;
        self.emit_change(window, cx);
        cx.notify();
    }

    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_text("", window, cx);
    }

    /// 无窗口上下文时更新内容(不触发 on_change 的 window 语义,直接置值并通知)
    pub fn set_text_quiet(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.marked_range = None;
        cx.notify();
    }

    pub fn focus(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    fn emit_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(cb) = self.on_change.as_ref() {
            let text = self.content.clone();
            cb(&text, window, cx);
        }
    }

    // ---- 动作处理 ----

    pub(crate) fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    pub(crate) fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    pub(crate) fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    pub(crate) fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    pub(crate) fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    pub(crate) fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        if self.multi_line {
            let line_start = self.content[..self.cursor_offset()]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            self.move_to(line_start, cx);
        } else {
            self.move_to(0, cx);
        }
    }

    pub(crate) fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        if self.multi_line {
            let offset = self.cursor_offset();
            let line_end = self.content[offset..]
                .find('\n')
                .map(|i| offset + i)
                .unwrap_or(self.content.len());
            self.move_to(line_end, cx);
        } else {
            self.move_to(self.content.len(), cx);
        }
    }

    pub(crate) fn move_up(&mut self, cx: &mut Context<Self>) {
        if !self.multi_line {
            return;
        }
        let offset = self.cursor_offset();
        let line_start = self.content[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
        if line_start == 0 {
            self.move_to(0, cx);
            return;
        }
        let col = offset - line_start;
        let prev_start = self.content[..line_start - 1]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let prev_len = line_start - 1 - prev_start;
        self.move_to(prev_start + col.min(prev_len), cx);
    }

    pub(crate) fn move_down(&mut self, cx: &mut Context<Self>) {
        if !self.multi_line {
            return;
        }
        let offset = self.cursor_offset();
        let line_end = self.content[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(self.content.len());
        if line_end == self.content.len() {
            self.move_to(self.content.len(), cx);
            return;
        }
        let col = offset
            - self.content[..offset]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
        let next_start = line_end + 1;
        let next_len = self.content[next_start..]
            .find('\n')
            .map(|i| next_start + i)
            .unwrap_or(self.content.len())
            - next_start;
        self.move_to(next_start + col.min(next_len), cx);
    }

    pub(crate) fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    pub(crate) fn newline(&mut self, _: &Newline, window: &mut Window, cx: &mut Context<Self>) {
        if self.multi_line {
            self.replace_text_in_range(None, "\n", window, cx);
        }
    }

    pub(crate) fn submit(&mut self, _: &Submit, window: &mut Window, cx: &mut Context<Self>) {
        if self.multi_line {
            return;
        }
        if let Some(cb) = self.on_submit.as_ref() {
            let text = self.content.clone();
            cb(&text, false, window, cx);
        }
    }

    pub(crate) fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            let text = if self.multi_line {
                text
            } else {
                text.replace('\n', " ")
            };
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    pub(crate) fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    pub(crate) fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    pub(crate) fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    pub(crate) fn on_mouse_up(
        &mut self,
        _: &MouseUpEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.is_selecting = false;
    }

    pub(crate) fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify();
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    pub(crate) fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let (Some(bounds), Some(rows)) = (self.last_bounds.as_ref(), self.last_rows.as_ref())
        else {
            return 0;
        };
        let line_height = bounds.size.height / rows.len().max(1) as f32;
        let row = ((position.y - bounds.top()) / line_height)
            .floor()
            .max(0.) as usize;
        let Some((line_start, line)) = rows.get(row.min(rows.len() - 1)) else {
            return 0;
        };
        let col = line.closest_index_for_x(position.x - bounds.left());
        line_start + col
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }
}

impl EntityInputHandler for InputState {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _w: &mut Window, _cx: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|r| self.range_to_utf16(r))
    }

    fn unmark_text(&mut self, _w: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content = (self.content[0..range.start].to_owned()
            + new_text
            + &self.content[range.end..])
            .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        self.emit_change(window, cx);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content = (self.content[0..range.start].to_owned()
            + new_text
            + &self.content[range.end..])
            .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|nr| nr.start + range.start..nr.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.emit_change(window, cx);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let rows = self.last_rows.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let (start, line) = rows.iter().find(|(s, l)| {
            range.start >= *s && range.start <= *s + l.text.len()
        })?;
        let x = line.x_for_index(range.start - *start);
        Some(Bounds::new(
            gpui::point(bounds.left() + x, bounds.top()),
            size(px(1.), bounds.size.height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let rows = self.last_rows.as_ref()?;
        let line_height = self.last_bounds?.size.height / rows.len().max(1) as f32;
        let row = ((line_point.y / line_height).floor().max(0.)) as usize;
        let (start, line) = rows.get(row.min(rows.len() - 1))?;
        let idx = line.index_for_x(line_point.x)?;
        Some(self.offset_to_utf16(start + idx))
    }
}

impl Focusable for InputState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// ------------------------- 自绘元素 -------------------------

struct InputElement {
    input: Entity<InputState>,
}

struct InputPrepaint {
    rows: Vec<(usize, ShapedLine)>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for InputElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for InputElement {
    type RequestLayoutState = ();
    type PrepaintState = InputPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let input = self.input.read(cx);
        let rows_count = if input.multi_line {
            input.content.lines().count().max(1)
        } else {
            1
        };
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height() * rows_count as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let multi = input.multi_line;
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let placeholder_color = cx.global::<Theme>().fg_subtle.opacity(0.7);

        // 拆行并逐行 shape
        let lines: Vec<(usize, SharedString)> = if multi {
            let mut v = Vec::new();
            let mut start = 0;
            for (i, part) in content.split('\n').enumerate() {
                let _ = i;
                v.push((start, part.to_string().into()));
                start += part.len() + 1;
            }
            if v.is_empty() {
                v.push((0, "".into()));
            }
            v
        } else {
            vec![(0, content.clone())]
        };

        let mut rows = Vec::with_capacity(lines.len());
        for (line_start, line_text) in &lines {
            let is_placeholder_line = content.is_empty() && *line_start == 0;
            let (display, color) = if is_placeholder_line {
                (input.placeholder.clone(), placeholder_color)
            } else {
                (line_text.clone(), style.color)
            };
            let run = TextRun {
                len: display.len(),
                font: style.font(),
                color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let shaped = window
                .text_system()
                .shape_line(display, font_size, &[run], None);
            rows.push((*line_start, shaped));
        }

        // 光标与选区(逐行求交)
        let mut cursor_quad = None;
        let mut selection_quads = Vec::new();
        for (line_start, line) in &rows {
            let line_end = *line_start + line.text.len();
            if !selected_range.is_empty() {
                let sel_start = selected_range.start.min(line_end);
                let sel_end = selected_range.end.min(line_end);
                if sel_start < sel_end && sel_end > *line_start {
                    let from = sel_start.max(*line_start);
                    let to = sel_end.max(*line_start);
                    let x1 = line.x_for_index(from - *line_start);
                    let x2 = line.x_for_index(to - *line_start);
                    let y_top = bounds.top()
                        + line_height
                            * rows
                                .iter()
                                .position(|(s, _)| s == line_start)
                                .unwrap_or(0) as f32;
                    selection_quads.push(fill(
                        Bounds::from_corners(
                            gpui::point(bounds.left() + x1, y_top),
                            gpui::point(bounds.left() + x2, y_top + line_height),
                        ),
                        cx.global::<Theme>().primary.opacity(0.30),
                    ));
                }
            }
            if cursor >= *line_start && cursor <= line_end {
                let row_ix = rows.iter().position(|(s, _)| s == line_start).unwrap_or(0);
                let y_top = bounds.top() + line_height * row_ix as f32;
                let x = line.x_for_index(cursor - *line_start);
                cursor_quad = Some(fill(
                    Bounds::new(
                        gpui::point(bounds.left() + x, y_top),
                        size(px(2.), line_height),
                    ),
                    cx.global::<Theme>().primary,
                ));
            }
        }

        let selection = selection_quads.first().cloned();
        InputPrepaint {
            rows,
            cursor: cursor_quad,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let line_height = window.line_height();
        let rows = std::mem::take(&mut prepaint.rows);
        let mut kept = Vec::with_capacity(rows.len());
        for (ix, (line_start, line)) in rows.into_iter().enumerate() {
            let origin = gpui::point(
                bounds.left(),
                bounds.top() + line_height * ix as f32,
            );
            line.paint(origin, line_height, window, cx).unwrap();
            kept.push((line_start, line));
        }

        let focused = focus_handle.is_focused(window);
        let blink_on = std::time::Instant::now().elapsed().as_millis() % 1000 < 600;
        if focused && blink_on {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _cx| {
            input.last_rows = Some(kept);
            input.last_bounds = Some(bounds);
        });
    }
}

// ------------------------- 容器组件 -------------------------

/// 单行输入容器(状态由 Entity 持有)
#[derive(IntoElement)]
pub struct TextInput {
    state: Entity<InputState>,
    id: ElementId,
    error: bool,
    disabled: bool,
    clearable: bool,
    prefix_icon: Option<SharedString>,
    suffix: Option<gpui::AnyElement>,
}

pub fn text_input(id: impl Into<ElementId>, state: &Entity<InputState>) -> TextInput {
    TextInput {
        id: id.into(),
        state: state.clone(),
        error: false,
        disabled: false,
        clearable: false,
        prefix_icon: None,
        suffix: None,
    }
}

/// 多行输入容器(状态需 `.multi_line(n)`)
pub fn text_area(id: impl Into<ElementId>, state: &Entity<InputState>) -> TextInput {
    text_input(id, state)
}

impl TextInput {
    pub fn error(mut self, e: bool) -> Self {
        self.error = e;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn clearable(mut self) -> Self {
        self.clearable = true;
        self
    }
    pub fn prefix_icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.prefix_icon = Some(icon.into());
        self
    }
    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let focused = self.state.read(cx).is_focused(window);
        let has_text = !self.state.read(cx).text().is_empty();
        let border_color = if self.error {
            theme.danger
        } else if focused {
            theme.primary
        } else {
            theme.border
        };
        let state = self.state.clone();
        let state_for_down = self.state.clone();
        let state_for_up = self.state.clone();
        let state_for_move = self.state.clone();
        let key_ctx = if self.state.read(cx).multi_line {
            "TextArea"
        } else {
            "TextInput"
        };

        let height = if self.state.read(cx).multi_line {
            let rows = self.state.read(cx).max_rows;
            px(10.) + window.line_height() * (rows as f32).min(6.)
        } else {
            px(32.)
        };

        let base = div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .h(height)
            .px_2()
            .py_1()
            .rounded(theme.radius_md)
            .bg(theme.bg_input)
            .border_1()
            .border_color(border_color)
            .when(self.disabled, |el| el.opacity(0.45))
            .cursor(CursorStyle::IBeam)
            .key_context(key_ctx)
            .track_focus(&state.focus_handle(cx))
            // 点击聚焦 + 光标定位
            .on_mouse_down(MouseButton::Left, {
                let st = state_for_down.clone();
                move |event: &MouseDownEvent, window, cx| {
                    window.focus(&st.focus_handle(cx));
                    st.update(cx, |s, cx| s.on_mouse_down(event, window, cx));
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let st = state_for_up.clone();
                move |event, window, cx| {
                    st.update(cx, |s, cx| s.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let st = state_for_up.clone();
                move |event, window, cx| {
                    st.update(cx, |s, cx| s.on_mouse_up(event, window, cx));
                }
            })
            .on_mouse_move({
                let st = state_for_move.clone();
                move |event, window, cx| {
                    if event.dragging() {
                        st.update(cx, |s, cx| s.on_mouse_move(event, window, cx));
                    }
                }
            })
            // 动作
            .on_action({
                let st = state.clone();
                move |_: &Backspace, window, cx| st.update(cx, |s, cx| s.backspace(&Backspace, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Delete, window, cx| st.update(cx, |s, cx| s.delete(&Delete, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Left, window, cx| st.update(cx, |s, cx| s.left(&Left, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Right, window, cx| st.update(cx, |s, cx| s.right(&Right, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &SelectLeft, window, cx| {
                    st.update(cx, |s, cx| s.select_left(&SelectLeft, window, cx))
                }
            })
            .on_action({
                let st = state.clone();
                move |_: &SelectRight, window, cx| {
                    st.update(cx, |s, cx| s.select_right(&SelectRight, window, cx))
                }
            })
            .on_action({
                let st = state.clone();
                move |_: &SelectAll, window, cx| {
                    st.update(cx, |s, cx| s.select_all(&SelectAll, window, cx))
                }
            })
            .on_action({
                let st = state.clone();
                move |_: &Home, window, cx| st.update(cx, |s, cx| s.home(&Home, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &End, window, cx| st.update(cx, |s, cx| s.end(&End, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Up, window, cx| st.update(cx, |s, cx| s.move_up(cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Down, window, cx| st.update(cx, |s, cx| s.move_down(cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Newline, window, cx| st.update(cx, |s, cx| s.newline(&Newline, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Submit, window, cx| st.update(cx, |s, cx| s.submit(&Submit, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Paste, window, cx| st.update(cx, |s, cx| s.paste(&Paste, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Copy, window, cx| st.update(cx, |s, cx| s.copy(&Copy, window, cx))
            })
            .on_action({
                let st = state.clone();
                move |_: &Cut, window, cx| st.update(cx, |s, cx| s.cut(&Cut, window, cx))
            })
            .children(self.prefix_icon.map(|i| {
                crate::icons::Icon::new(i)
                    .size(px(14.))
                    .text_color(theme.fg_subtle)
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(InputElement {
                        input: self.state.clone(),
                    }),
            )
            .children(self.suffix)
            .when(self.clearable && has_text && !self.disabled, |el| {
                let st = state.clone();
                el.child(
                    div()
                        .id("clear-btn")
                        .cursor(gpui::CursorStyle::PointingHand)
                        .hover(|s| s.text_color(theme.fg))
                        .text_color(theme.fg_subtle)
                        .on_click(move |_, window, cx| {
                            st.update(cx, |s, cx| s.clear(window, cx));
                        })
                        .child(crate::icons::Icon::new("x").size(px(12.))),
                )
            });

        // 动画挂载:驱动光标闪烁重绘(仅聚焦时;每窗口同时只有一个聚焦输入框,id 冲突不可能发生)
        if focused {
            base.with_animation(
                "input-blink",
                gpui::Animation::new(std::time::Duration::from_millis(1000)).repeat(),
                |el, _| el,
            )
            .into_any_element()
        } else {
            base.into_any_element()
        }
    }
}
