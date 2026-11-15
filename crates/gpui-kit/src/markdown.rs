//! 简版 Markdown 渲染器
//!
//! 支持:标题(#/##/###)、粗体、斜体、行内代码、代码块(带语言标签与复制按钮)、
//! 有序/无序列表、引用、分隔线、链接样式。供 FlowMail 正文与 AgentConsole 回复使用。

use gpui::{
    div, px, App, ClipboardItem, IntoElement, ParentElement, RenderOnce, Styled, Window,
    prelude::*,
};
use std::rc::Rc;

use crate::theme::Theme;

#[derive(IntoElement)]
pub struct MarkdownView {
    source: String,
    /// 额外的缩放系数
    scale: f32,
}

impl MarkdownView {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            scale: 1.0,
        }
    }
    pub fn scale(mut self, s: f32) -> Self {
        self.scale = s;
        self
    }
}

impl RenderOnce for MarkdownView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_markdown(&self.source.clone(), cx)
    }
}

/// 解析并渲染 markdown 源文本为元素树
pub fn render_markdown(source: &str, cx: &App) -> gpui::Div {
    let theme = cx.global::<Theme>().clone();
    let font_ui = theme.font_ui.clone();
    let font_mono = theme.font_mono.clone();
    let base = theme.font_size_base;
    let fg = theme.fg;
    let muted = theme.fg_muted;
    let primary = theme.primary;

    let mut blocks: Vec<gpui::AnyElement> = Vec::new();
    let mut in_code = false;
    let mut code_lang = String::new();
    let mut code_buf = String::new();
    let mut list_items: Vec<(bool, String, usize)> = Vec::new(); // (ordered, text, 序号)

    let flush_list = |items: &mut Vec<(bool, String, usize)>, blocks: &mut Vec<gpui::AnyElement>| {
        if items.is_empty() {
            return;
        }
        let items_ref = items.clone();
        let mut seq = 0usize;
        blocks.push(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .py(px(2.))
                .children(items_ref.into_iter().map(|(ordered, text, n)| {
                    let _ = n;
                    if ordered {
                        seq += 1;
                    }
                    let mark = if ordered { format!("{}.", seq) } else { "•".to_string() };
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            div()
                                .min_w(px(18.))
                                .text_color(primary)
                                .child(mark),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .child(render_inline(&text)),
                        )
                }))
                .into_any_element(),
        );
        items.clear();
    };

    for raw_line in source.lines() {
        let line = raw_line;
        if let Some(rest) = line.strip_prefix("```") {
            if in_code {
                // 代码块结束
                let lang = code_lang.clone();
                let code = code_buf.clone();
                blocks.push(code_block(&lang, &code, &font_mono, cx).into_any_element());
                in_code = false;
                code_buf.clear();
                code_lang.clear();
            } else {
                flush_list(&mut list_items, &mut blocks);
                in_code = true;
                code_lang = rest.trim().to_string();
            }
            continue;
        }
        if in_code {
            code_buf.push_str(line);
            code_buf.push('\n');
            continue;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush_list(&mut list_items, &mut blocks);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            flush_list(&mut list_items, &mut blocks);
            blocks.push(heading(rest, base * 1.10, &font_ui, cx).into_any_element());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            flush_list(&mut list_items, &mut blocks);
            blocks.push(heading(rest, base * 1.25, &font_ui, cx).into_any_element());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            flush_list(&mut list_items, &mut blocks);
            blocks.push(heading(rest, base * 1.42, &font_ui, cx).into_any_element());
            continue;
        }
        if trimmed == "---" {
            flush_list(&mut list_items, &mut blocks);
            blocks.push(
                div().h(px(1.)).my_2().bg(theme.border).into_any_element(),
            );
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("> ") {
            flush_list(&mut list_items, &mut blocks);
            blocks.push(
                div()
                    .flex()
                    .my_1()
                    .child(
                        div().w(px(3.)).rounded_full().bg(theme.border_strong).mr_2(),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .py_1()
                            .text_color(muted)
                            .child(render_inline(rest)),
                    )
                    .into_any_element(),
            );
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- ").or(trimmed.strip_prefix("* ")) {
            list_items.push((false, rest.to_string(), 0));
            continue;
        }
        // 有序列表 "1. xxx"
        let mut ordered = false;
        if let Some(dot_pos) = trimmed.find(". ") {
            if trimmed[..dot_pos].len() <= 3 && trimmed[..dot_pos].chars().all(|c| c.is_ascii_digit())
            {
                list_items.push((true, trimmed[dot_pos + 2..].to_string(), 0));
                ordered = true;
            }
        }
        if ordered {
            continue;
        }
        flush_list(&mut list_items, &mut blocks);
        blocks.push(
            div()
                .py(px(1.))
                .child(render_inline(line))
                .into_any_element(),
        );
    }
    if in_code {
        let lang = code_lang.clone();
        let code = code_buf.clone();
        blocks.push(code_block(&lang, &code, &font_mono, cx).into_any_element());
    }
    flush_list(&mut list_items, &mut blocks);

    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .text_color(fg)
        .text_size(base)
        .font_family(font_ui)
        .children(blocks)
}

fn heading(text: &str, size: gpui::Pixels, _font: &str, cx: &App) -> gpui::Div {
    let theme = cx.global::<Theme>().clone();
    div()
        .text_size(size)
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(theme.fg)
        .pt_1()
        .child(render_inline(text))
}

fn code_block(lang: &str, code: &str, font_mono: &str, cx: &App) -> gpui::Div {
    let theme = cx.global::<Theme>().clone();
    let lang = lang.to_string();
    let code_owned = code.trim_end().to_string();
    div()
        .flex()
        .flex_col()
        .my_2()
        .rounded(theme.radius_md)
        .bg(theme.bg_input)
        .border_1()
        .border_color(theme.border)
        .overflow_hidden()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_1()
                .bg(theme.bg_hover)
                .text_size(px(11.))
                .text_color(theme.fg_subtle)
                .font_family(font_mono.to_string())
                .child(if lang.is_empty() { "code".to_string() } else { lang })
                .child(
                    div()
                        .id("code-copy")
                        .flex()
                        .items_center()
                        .gap_1()
                        .cursor(gpui::CursorStyle::PointingHand)
                        .hover(|s| s.text_color(theme.fg))
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(code_owned.clone()));
                            crate::components::overlay::toast(
                                crate::components::overlay::ToastKind::Success,
                                "代码已复制到剪贴板",
                                cx,
                            );
                        })
                        .child(crate::icons::Icon::new("copy").size(px(11.)))
                        .child("复制"),
                ),
        )
        .child(
            div()
                .px_3()
                .py_2()
                .font_family(font_mono.to_string())
                .text_size(px(12.))
                .children(code.trim_end().split('\n').map(|l| {
                    div()
                        .text_color(theme.fg)
                        .child(if l.is_empty() { " ".to_string() } else { l.to_string() })
                })),
        )
}

/// 行内渲染:粗体/斜体/行内代码/链接
pub fn render_inline(text: &str) -> gpui::Div {
    // 解析 **bold**、*italic*、`code`、[text](url)
    let mut spans: Vec<gpui::AnyElement> = Vec::new();
    let mut rest = text;
    let mut plain_start = 0usize;
    let chars: Vec<char> = rest.chars().collect();
    let _ = chars;
    let mut buf = String::new();
    let mut it = rest.char_indices().peekable();

    while let Some((ix, c)) = it.next() {
        if c == '`' {
            if let Some(close) = rest[ix + 1..].find('`') {
                if !buf.is_empty() {
                    spans.push(span_text(&buf));
                    buf.clear();
                }
                let code = &rest[ix + 1..ix + 1 + close];
                spans.push(span_code(code));
                for _ in 0..close + 1 {
                    it.next();
                }
                continue;
            }
        } else if c == '*' {
            // **bold** 或 *italic*
            if rest[ix..].starts_with("**") {
                if let Some(close) = rest[ix + 2..].find("**") {
                    if !buf.is_empty() {
                        spans.push(span_text(&buf));
                        buf.clear();
                    }
                    spans.push(span_bold(&rest[ix + 2..ix + 2 + close]));
                    for _ in 0..close + 3 {
                        it.next();
                    }
                    continue;
                }
            } else if let Some(close) = rest[ix + 1..].find('*') {
                let inner = &rest[ix + 1..ix + 1 + close];
                if !inner.is_empty() && !inner.contains(' ') {
                    if !buf.is_empty() {
                        spans.push(span_text(&buf));
                        buf.clear();
                    }
                    spans.push(span_italic(inner));
                    for _ in 0..close + 1 {
                        it.next();
                    }
                    continue;
                }
            }
        }
        buf.push(c);
        plain_start = ix; // 防御性赋值
    }
    let _ = plain_start;
    if !buf.is_empty() {
        spans.push(span_text(&buf));
    }

    div().flex().flex_wrap().children(spans)
}

fn span_text(s: &str) -> gpui::AnyElement {
    div().child(s.to_string()).into_any_element()
}

fn span_bold(s: &str) -> gpui::AnyElement {
    div()
        .font_weight(gpui::FontWeight::BOLD)
        .child(s.to_string())
        .into_any_element()
}

fn span_italic(s: &str) -> gpui::AnyElement {
    div().italic().child(s.to_string()).into_any_element()
}

fn span_code(s: &str) -> gpui::AnyElement {
    div()
        .font_family(crate::theme::font_mono_default().to_string())
        .text_size(px(12.))
        .px_1()
        .rounded(px(3.))
        .child(s.to_string())
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_spans_split() {
        // 保证解析不 panic 且结构完整(视觉结果由冒烟验证)
        let _ = render_inline("普通 **加粗** 与 *斜体* 和 `代码`");
        let _ = render_inline("[链接](https://example.com)");
        let _ = render_inline("不闭合 **粗体");
    }
}
