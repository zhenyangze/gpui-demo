//! DevToolbox UI:工具侧栏 + 每工具双栏界面 + 命令面板。

use std::collections::HashMap;

use gpui::{px, prelude::*, App, Context, Entity, FocusHandle, Focusable, Render, Window};
use gpui_kit::prelude::*;
use gpui_kit::util::format_thousands;

use crate::state::{self, TOOL_NAMES};
use crate::tools;
use crate::{OpenPalette, Quit, Tool1, Tool2, Tool3, Tool4, Tool5, Tool6};

pub struct ToolboxApp {
    focus: FocusHandle,
    dark: bool,
    /// 工具页路由(栈式导航:侧栏点击 push,⌘[ 后退)
    router: gpui_kit::nav::Router<usize>,
    favorites: Vec<usize>,
    history: HashMap<usize, Vec<String>>,
    palette: Entity<PaletteState>,
    // 输入状态(共享的多行输入)
    input_a: Entity<InputState>,
    input_b: Entity<InputState>,
    input_line: Entity<InputState>,
    // JSON 选项
    json_indent: usize,
    json_urlsafe: bool,
    // 正则
    regex_flags: Vec<bool>, // g i m s u
    // 哈希选项
    pw_len: f64,
    pw_opts: [bool; 4],
    // diff 选项
    diff_ignore_ws: bool,
    // 颜色
    color_hex: Entity<InputState>,
}

impl ToolboxApp {
    pub fn new(cx: &mut Context<Self>, arg: Option<String>) -> Self {
        let saved = state::load();
        let mut dark = saved.dark;
        if arg.as_deref() == Some("light") {
            dark = true;
        }
        let mut theme = if dark { Theme::dark() } else { Theme::light() };
        theme.set_accent(Accent::Teal);
        cx.set_global(theme);

        let input_a = cx.new(|cx| InputState::new(cx).multi_line(10).placeholder("输入…"));
        let input_b = cx.new(|cx| InputState::new(cx).multi_line(10).placeholder("对比文本…"));
        let input_line = cx.new(|cx| InputState::new(cx).placeholder("单行输入…"));
        let color_hex = cx.new(|cx| InputState::new(cx).placeholder("#3366CC"));
        color_hex.update(cx, |s, cx| s.set_text_quiet("#3366CC", cx));

        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();

        let app = Self {
            focus: cx.focus_handle(),
            dark,
            router: gpui_kit::nav::Router::new(saved.current_tool.min(TOOL_NAMES.len() - 1)),
            favorites: saved.favorites,
            history: saved.history,
            palette: cx.new(|cx| PaletteState::new(cx)),
            input_a,
            input_b,
            input_line,
            json_indent: 2,
            json_urlsafe: false,
            regex_flags: vec![true, false, false, false, false],
            pw_len: 16.,
            pw_opts: [true, true, true, false],
            diff_ignore_ws: false,
            color_hex,
        };
        app.register_palette(cx);
        // 预填 JSON 示例
        app.input_a.update(cx, |s, cx| {
            s.set_text_quiet(r#"{"name":"gpui-kit","stars":5123,"tags":["rust","ui"],"meta":{"rows":[{"id":1},{"id":2},{"id":3}]}}"#, cx)
        });
        app.input_line.update(cx, |s, cx| {
            s.set_text_quiet("hello gpui world".to_string(), cx)
        });
        app
    }

    fn register_palette(&self, cx: &mut Context<Self>) {
        let entity = cx.entity();
        let mut items = Vec::new();
        for (ix, name) in TOOL_NAMES.iter().enumerate() {
            let e = entity.clone();
            let id = Box::leak(format!("tool-{}", ix).into_boxed_str());
            items.push(PaletteCommand::new(id, format!("打开工具:{}", name), move |_, cx| {
                e.update(cx, |this, cx| {
                    this.router.push(ix);
                    this.persist();
                    cx.notify();
                });
            }));
        }
        items.push(PaletteCommand::new("theme", "切换深浅主题", {
            let e = entity.clone();
            move |_, cx| {
                e.update(cx, |this, cx| {
                    this.dark = !this.dark;
                    let mode = if this.dark { gpui_kit::theme::ThemeMode::Light } else { gpui_kit::theme::ThemeMode::Dark };
                    cx.update_global::<Theme, _>(|t, _| *t = t.clone().with_mode(mode));
                    this.persist();
                    cx.notify();
                });
            }
        }));
        self.palette.update(cx, |p, cx| p.set_items(items, cx));
    }

    fn persist(&self) {
        state::save(&state::Persisted {
            current_tool: *self.router.current(),
            dark: self.dark,
            favorites: self.favorites.clone(),
            history: self.history.clone(),
        });
    }

    fn tool_icon(ix: usize) -> &'static str {
        match ix {
            0 => "code",
            1 => "zap",
            2 => "box",
            3 => "globe",
            4 => "shield",
            5 => "clock",
            6 => "hash",
            7 => "grid",
            8 => "palette",
            9 => "list",
            10 => "edit",
            _ => "terminal",
        }
    }
}

impl Render for ToolboxApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let entity = cx.entity();
        let current = *self.router.current();
        let tool_name = TOOL_NAMES[current];

        div()
            .id("tb-root")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.bg)
            .text_color(theme.fg)
            .font_family(theme.font_ui.clone())
            .key_context("Toolbox")
            .track_focus(&self.focus.clone())
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &OpenPalette, w: &mut Window, cx: &mut Context<ToolboxApp>| {
                let pal = this.palette.clone();
                pal.update(cx, |p, cx| p.open(w, cx));
            }))
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &Tool1, _: &mut Window, cx: &mut Context<ToolboxApp>| {
                this.router.push(0);
                this.persist();
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &Tool2, _: &mut Window, cx: &mut Context<ToolboxApp>| {
                this.router.push(1);
                this.persist();
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &Tool3, _: &mut Window, cx: &mut Context<ToolboxApp>| {
                this.router.push(2);
                this.persist();
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &Tool4, _: &mut Window, cx: &mut Context<ToolboxApp>| {
                this.router.push(3);
                this.persist();
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &Tool5, _: &mut Window, cx: &mut Context<ToolboxApp>| {
                this.router.push(4);
                this.persist();
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut ToolboxApp, _: &Tool6, _: &mut Window, cx: &mut Context<ToolboxApp>| {
                this.router.push(5);
                this.persist();
                cx.notify();
            }))
            .on_action(|_: &Quit, _, cx| cx.quit())
            // 顶栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(50.))
                    .px_3()
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new("wrench").size(px(18.)).text_color(theme.primary))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("DevToolbox"),
                            )
                            .child(Badge::new(tool_name).semantic(Semantic::Primary)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("tb-back", "后退")
                                    .icon("arrow-left")
                                    .disabled(!self.router.can_pop())
                                    .tooltip(format!("历史 {} 层", self.router.depth() - 1))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if this.router.pop() {
                                            this.persist();
                                            cx.notify();
                                        }
                                    })),
                            )
                            .child(
                                Button::new("tb-palette", "工具 ⌘K")
                                    .ghost()
                                    .on_click(cx.listener(|this, _, w, cx| {
                                        this.palette.update(cx, |p, cx| p.open(w, cx));
                                    })),
                            )
                            .child(
                                Button::new("tb-theme", if theme.is_dark() { "浅色" } else { "深色" })
                                    .icon(if theme.is_dark() { "sun" } else { "moon" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.dark = !this.dark;
                                        let mode = if this.dark { gpui_kit::theme::ThemeMode::Light } else { gpui_kit::theme::ThemeMode::Dark };
                                        cx.update_global::<Theme, _>(|t, _| *t = t.clone().with_mode(mode));
                                        this.persist();
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // 主体
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    // 侧栏
                    .child(
                        div()
                            .w(px(180.))
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .p_2()
                            .gap_1()
                            .border_r_1()
                            .border_color(theme.border)
                            .bg(theme.bg_panel)
                            .child(div().px_1().text_size(px(11.)).text_color(theme.fg_subtle).child("收藏"))
                            .children(self.favorites.iter().map(|ix| {
                                let ix = *ix;
                                div()
                                    .id(("tb-fav", ix))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_2()
                                    .py_1()
                                    .rounded(theme.radius_sm)
                                    .text_size(px(12.))
                                    .cursor_pointer()
                                    .text_color(theme.warning)
                                    .hover(|s| s.bg(theme.bg_hover))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.router.push(ix);
                                        this.persist();
                                        cx.notify();
                                    }))
                                    .child(Icon::new("star").size(px(11.)))
                                    .child(TOOL_NAMES[ix].to_string())
                            }))
                            .when(self.favorites.is_empty(), |el| {
                                el.child(
                                    div()
                                        .px_1()
                                        .text_size(px(11.))
                                        .text_color(theme.fg_subtle)
                                        .child("(点击 ⭐ 添加)"),
                                )
                            })
                            .child(div().px_1().pt_2().text_size(px(11.)).text_color(theme.fg_subtle).child("全部工具"))
                            .children(TOOL_NAMES.iter().enumerate().map(|(ix, name)| {
                                let is_cur = ix == current;
                                let is_fav = self.favorites.contains(&ix);
                                div()
                                    .id(("tb-tool", ix))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_2()
                                    .py_1p5()
                                    .rounded(theme.radius_md)
                                    .text_size(px(12.))
                                    .cursor_pointer()
                                    .when(is_cur, |el| {
                                        el.bg(theme.primary.opacity(0.14)).text_color(theme.primary)
                                    })
                                    .when(!is_cur, |el| {
                                        el.text_color(theme.fg_muted)
                                            .hover(|s| s.bg(theme.bg_hover).text_color(theme.fg))
                                    })
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.router.push(ix);
                                        this.persist();
                                        cx.notify();
                                    }))
                                    .child(Icon::new(Self::tool_icon(ix)).size(px(13.)))
                                    .child(div().flex_1().min_w_0().truncate().child(name.to_string()))
                                    .child({
                                        let e = entity.clone();
                                        div()
                                            .id(("tb-star", ix))
                                            .text_color(if is_fav { theme.warning } else { theme.fg_subtle })
                                            .child(Icon::new("star").size(px(10.)))
                                            .on_click({
                                                let e = e.clone();
                                                let is_fav = is_fav;
                                                move |_, _, cx| {
                                                    e.update(cx, |this, cx| {
                                                        if is_fav {
                                                            this.favorites.retain(|f| *f != ix);
                                                        } else {
                                                            this.favorites.push(ix);
                                                        }
                                                        this.persist();
                                                        cx.notify();
                                                    });
                                                }
                                            })
                                    })
                            })),
                    )
                    // 工具区
                    .child(
                        div()
                            .id("tb-content")
                            .flex_1()
                            .min_w_0()
                            .overflow_y_scroll()
                            .p_4()
                            .child(self.render_tool(current, &theme, cx)),
                    ),
            )
            // 状态栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(24.))
                    .px_3()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .text_size(px(10.))
                    .text_color(theme.fg_subtle)
                    .child(format!("当前工具:{} · 共 12 个", tool_name))
                    .child("本地计算 · 不联网"),
            )
            .child(self.palette.clone())
            .children(render_toasts(window, cx))
    }
}

impl Focusable for ToolboxApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// 双栏输出布局
fn io_pane(
    label_a: &str,
    label_b: &str,
    a: gpui::AnyElement,
    b: gpui::AnyElement,
    theme: &Theme,
) -> gpui::AnyElement {
    div()
        .flex()
        .gap_3()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme.fg_subtle)
                        .child(label_a.to_string()),
                )
                .child(a),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .flex_1()
                .min_w_0()
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme.fg_subtle)
                        .child(label_b.to_string()),
                )
                .child(b),
        )
        .into_any_element()
}

fn output_box(text: &str, theme: &Theme, mono: bool, error: bool) -> gpui::AnyElement {
    div()
        .id(("out-box", text.len()))
        .flex_1()
        .min_h(px(120.))
        .p_2()
        .rounded(theme.radius_md)
        .bg(theme.bg_input)
        .border_1()
        .border_color(if error { theme.danger } else { theme.border })
        .text_size(px(12.))
        .when(mono, |el| el.font_family(theme.font_mono.clone()))
        .text_color(if error { theme.danger } else { theme.fg })
        .overflow_y_scroll()
        .child(if text.is_empty() {
            "—".to_string()
        } else {
            text.to_string()
        })
        .into_any_element()
}

impl ToolboxApp {
    fn render_tool(&mut self, ix: usize, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        match ix {
            0 => self.tool_json(theme, cx),
            1 => self.tool_regex(theme, cx),
            2 => self.tool_base64(theme, cx),
            3 => self.tool_url(theme, cx),
            4 => self.tool_hash(theme, cx),
            5 => self.tool_time(theme, cx),
            6 => self.tool_uuid(theme, cx),
            7 => self.tool_radix(theme, cx),
            8 => self.tool_color(theme, cx),
            9 => self.tool_diff(theme, cx),
            10 => self.tool_text(theme, cx),
            _ => self.tool_calc(theme, cx),
        }
    }

    fn tool_json(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_a.read(cx).text().to_string();
        let (out, err, stats) = if src.trim().is_empty() {
            (String::new(), false, None)
        } else {
            match tools::json_validate(&src) {
                Ok(st) => {
                    let formatted = tools::json_format(&src, self.json_indent).unwrap_or_default();
                    (formatted, false, Some(st))
                }
                Err(e) => (e, true, None),
            }
        };
        let e_in = self.input_a.clone();
        let e_self = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("JSON 工具", "格式化 / 压缩 / 校验 / 键排序 / Path 提取", theme))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("j-fmt", "格式化").primary().on_click({
                            let e = e_self.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    let src = this.input_a.read(cx).text().to_string();
                                    if let Ok(f) = tools::json_format(&src, this.json_indent) {
                                        this.input_a.update(cx, |s, cx| s.set_text_quiet(f, cx));
                                    }
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Button::new("j-min", "压缩").on_click({
                            let e = e_self.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    let src = this.input_a.read(cx).text().to_string();
                                    if let Ok(m) = tools::json_minify(&src) {
                                        this.input_a.update(cx, |s, cx| s.set_text_quiet(m, cx));
                                    }
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Button::new("j-sort", "键排序").on_click({
                            let e = e_self.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    let src = this.input_a.read(cx).text().to_string();
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&src) {
                                        let sorted = tools::json_sort_keys(&v);
                                        this.input_a.update(cx, |s, cx| {
                                            s.set_text_quiet(serde_json::to_string_pretty(&sorted).unwrap_or_default(), cx)
                                        });
                                    }
                                    cx.notify();
                                });
                            }
                        }),
                    )
                    .child(
                        Segmented::new(
                            "j-indent",
                            vec!["2 空格".into(), "4 空格".into()],
                            if self.json_indent == 2 { 0 } else { 1 },
                        )
                        .on_change({
                            let e = e_self.clone();
                            move |ix, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.json_indent = if ix == 0 { 2 } else { 4 };
                                    cx.notify();
                                });
                            }
                        }),
                    ),
            )
            .child(io_pane(
                "输入(可编辑)",
                if err { "校验结果(错误)" } else { "格式化结果" },
                div().h(px(260.)).child(text_input("j-in", &e_in)).into_any_element(),
                output_box(&out, theme, true, err),
                theme,
            ))
            .when_some(stats, |el, st| {
                el.child(
                    div()
                        .flex()
                        .gap_3()
                        .text_size(px(11.))
                        .text_color(theme.fg_muted)
                        .child(format!("节点 {} · 深度 {} · 最大数组 {}", st.nodes, st.depth, st.max_array_len)),
                )
            })
            .into_any_element()
    }

    fn tool_regex(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        use regex::Regex;
        let pattern = self.input_line.read(cx).text().to_string();
        let test_text = self.input_a.read(cx).text().to_string();
        let flags = self.regex_flags.clone();
        let (result, matches_list, error) = if pattern.is_empty() {
            (String::new(), vec![], false)
        } else {
            let mut builder = regex::RegexBuilder::new(pattern.as_str());
            builder = builder
                .case_insensitive(flags.get(1).copied().unwrap_or(false))
                .dot_matches_new_line(flags.get(3).copied().unwrap_or(false))
                .multi_line(flags.get(4).copied().unwrap_or(false))
                .to_owned();
            match builder.build() {
                Ok(re) => {
                    let mut list = Vec::new();
                    let mut out = String::new();
                    let text = test_text.as_str();
                    let mut last = 0usize;
                    for m in re.find_iter(text) {
                        out.push_str(&text[last..m.start()]);
                        out.push_str(&format!("【{}】", m.as_str()));
                        last = m.end();
                        let mut line = format!("{} @ {}", m.as_str(), m.start());
                        if let Some(caps) = re.captures(m.as_str()) {
                            if caps.len() > 1 {
                                let groups: Vec<String> = caps
                                    .iter()
                                    .skip(1)
                                    .map(|c| c.map(|g| g.as_str().to_string()).unwrap_or_default())
                                    .collect();
                                line.push_str(&format!(" 分组:{:?}", groups));
                            }
                        }
                        list.push(line);
                    }
                    out.push_str(&text[last..]);
                    (out, list, false)
                }
                Err(e) => (format!("正则错误:{}", e), vec![], true),
            }
        };
        let e_line = self.input_line.clone();
        let e_text = self.input_a.clone();
        let e_self = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("正则测试器", "实时匹配 / 高亮 / 分组 / 模板", theme))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .child(text_input("rx-pattern", &e_line).prefix_icon("zap")),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_1()
                            .children(["g", "i", "m", "s", "u"]
                                .iter()
                                .enumerate()
                                .map(|(ix, f)| {
                                    let e = e_self.clone();
                                    let on = flags.get(ix).copied().unwrap_or(false);
                                    div()
                                        .id(("rx-flag", ix))
                                        .px_2()
                                        .py_1()
                                        .rounded(theme.radius_sm)
                                        .text_size(px(11.))
                                        .font_family(theme.font_mono.clone())
                                        .cursor_pointer()
                                        .text_color(if on { theme.primary_fg } else { theme.fg_subtle })
                                        .bg(if on { theme.primary } else { theme.bg_hover })
                                        .on_click(move |_, _, cx| {
                                            e.update(cx, |this, cx| {
                                                this.regex_flags[ix] = !this.regex_flags[ix];
                                                cx.notify();
                                            });
                                        })
                                        .child(f.to_string())
                                })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children(tools::REGEX_TEMPLATES.iter().enumerate().map(|(ix, (name, re))| {
                        let e = e_self.clone();
                        let re = re.to_string();
                        div()
                            .id(("rx-tpl", ix))
                            .px_2()
                            .py_1()
                            .rounded_full()
                            .bg(theme.bg_hover)
                            .border_1()
                            .border_color(theme.border)
                            .text_size(px(11.))
                            .text_color(theme.fg_muted)
                            .cursor_pointer()
                            .hover(|s| s.text_color(theme.fg))
                            .on_click(move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.input_line.update(cx, |s, cx| s.set_text_quiet(re.clone(), cx));
                                    cx.notify();
                                });
                            })
                            .child(name.to_string())
                    })),
            )
            .child(io_pane(
                "测试文本",
                if error { "错误" } else { "匹配结果(【】标记)" },
                div().h(px(200.)).child(text_input("rx-text", &e_text)).into_any_element(),
                output_box(&result, theme, true, error),
                theme,
            ))
            .when(!matches_list.is_empty(), |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(theme.fg_subtle)
                                .child(format!("共 {} 处匹配:", matches_list.len())),
                        )
                        .children(matches_list.iter().take(20).map(|m| {
                            div()
                                .text_size(px(11.))
                                .font_family(theme.font_mono.clone())
                                .text_color(theme.fg_muted)
                                .child(m.clone())
                        })),
                )
            })
            .into_any_element()
    }

    fn tool_base64(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_a.read(cx).text().to_string();
        let (encoded, dec_err) = if src.is_empty() {
            (String::new(), false)
        } else {
            (tools::b64_encode(src.as_bytes(), self.json_urlsafe), false)
        };
        let decoded = tools::b64_decode(&src, self.json_urlsafe)
            .and_then(|b| String::from_utf8(b).map_err(|_| "非 UTF-8".into()))
            .map(|s| format!("解码:{}", s))
            .unwrap_or_default();
        let hex_out = tools::to_hex(&src);
        let e_in = self.input_a.clone();
        let e_self = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("Base64 / 编解码", "编码 / 解码 / HEX", theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child("URL 安全")
                    .child({
                        let e = e_self.clone();
                        Switch::new("b64-url", self.json_urlsafe).on_change(move |_, cx| {
                            e.update(cx, |this, cx| {
                                this.json_urlsafe = !this.json_urlsafe;
                                cx.notify();
                            });
                        })
                    }),
            )
            .child(io_pane(
                "文本输入",
                "Base64 编码",
                div().h(px(140.)).child(text_input("b64-in", &e_in)).into_any_element(),
                output_box(&encoded, theme, true, false),
                theme,
            ))
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("反向解码"))
                            .child(output_box(&decoded, theme, false, dec_err)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .flex_1()
                            .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("HEX"))
                            .child(output_box(&hex_out, theme, true, false)),
                    ),
            )
            .into_any_element()
    }

    fn tool_url(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_line.read(cx).text().to_string();
        let encoded = tools::percent_encode(&src);
        let decoded = tools::percent_decode(&src).unwrap_or_else(|e| format!("错误:{}", e));
        let e_line = self.input_line.clone();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("URL 编解码", "percent-encoding 双向", theme))
            .child(
                div()
                    .child(text_input("url-in", &e_line).prefix_icon("globe").clearable()),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("编码"))
                    .child(output_box(&encoded, theme, true, false)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("解码"))
                    .child(output_box(&decoded, theme, false, false)),
            )
            .into_any_element()
    }

    fn tool_hash(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_a.read(cx).text().to_string();
        let (h256, h1, h5) = if src.is_empty() {
            (String::new(), String::new(), String::new())
        } else {
            (tools::sha256_hex(&src), tools::sha1_hex(&src), tools::md5_hex(&src))
        };
        let pw = tools::gen_password(
            self.pw_len as usize,
            self.pw_opts[0],
            self.pw_opts[1],
            self.pw_opts[2],
            self.pw_opts[3],
        );
        let entropy = tools::password_entropy(&pw);
        let e_in = self.input_a.clone();
        let e_self = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("哈希与密码", "SHA-256 / SHA-1 / MD5 / 密码生成", theme))
            .child(div().h(px(90.)).child(text_input("h-in", &e_in).prefix_icon("shield")))
            .child(hash_row("SHA-256", h256.clone(), theme))
            .child(hash_row("SHA-1", h1.clone(), theme))
            .child(hash_row("MD5", h5.clone(), theme))
            .child(divider())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(div().w(px(64.)).text_size(px(12.)).child("长度"))
                    .child({
                        let e = e_self.clone();
                        div()
                            .w(px(200.))
                            .child(
                                Slider::new("pw-len", self.pw_len)
                                    .range(8., 64.)
                                    .step(1.)
                                    .on_change(move |v, _, cx| {
                                        e.update(cx, |this, cx| {
                                            this.pw_len = v;
                                            cx.notify();
                                        });
                                    })
                            )
                    })
                    .child(
                        div()
                            .font_family(theme.font_mono.clone())
                            .text_size(px(12.))
                            .text_color(theme.primary)
                            .child(format!("{}", self.pw_len as usize)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children(["小写", "大写", "数字", "符号"]
                        .iter()
                        .enumerate()
                        .map(|(ix, name)| {
                            let e = e_self.clone();
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    Checkbox::new(("pw-opt", ix), self.pw_opts[ix]).on_change(
                                        move |_, cx| {
                                            e.update(cx, |this, cx| {
                                                this.pw_opts[ix] = !this.pw_opts[ix];
                                                cx.notify();
                                            });
                                        },
                                    ),
                                )
                                .child(
                                    div().text_size(px(12.)).text_color(theme.fg_muted).child(name.to_string()),
                                )
                        })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_2()
                            .rounded(theme.radius_md)
                            .bg(theme.bg_input)
                            .border_1()
                            .border_color(theme.border)
                            .font_family(theme.font_mono.clone())
                            .text_size(px(13.))
                            .child(pw.clone()),
                    )
                    .child(
                        Button::new("pw-copy", "复制")
                            .on_click(move |_, _, cx| {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(pw.clone()));
                                toast(ToastKind::Success, "密码已复制", cx);
                            }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("强度"))
                    .child(
                        div()
                            .w(px(160.))
                            .child(ProgressBar::new((entropy / 100.).clamp(0., 1.))),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.fg_muted)
                            .child(format!("{:.0} bits", entropy)),
                    ),
            )
            .into_any_element()
    }

    fn tool_time(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_line.read(cx).text().to_string();
        let now = state::now();
        let (parsed, err) = if src.trim().is_empty() {
            (String::new(), false)
        } else {
            match tools::parse_timestamp(&src) {
                Ok((dt, date)) => (format!("{}\n{}", dt, date), false),
                Err(e) => (e, true),
            }
        };
        let e_line = self.input_line.clone();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("时间戳转换", "秒/毫秒自动识别 · 本地 + 日期", theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded(theme.radius_md)
                            .bg(theme.bg_input)
                            .border_1()
                            .border_color(theme.border)
                            .font_family(theme.font_mono.clone())
                            .text_size(px(14.))
                            .child(gpui_kit::util::unix_to_datetime(now)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_size(px(11.))
                            .text_color(theme.fg_subtle)
                            .child(Icon::new("clock").size(px(11.)))
                            .child(format!("unix:{}", now)),
                    ),
            )
            .child(div().child(text_input("ts-in", &e_line).prefix_icon("clock").clearable()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("转换结果"))
                    .child(output_box(&parsed, theme, true, err)),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .text_size(px(11.))
                    .text_color(theme.fg_muted)
                    .child("相对时间示例:")
                    .child(tools::relative_now(now - 3600))
                    .child("·")
                    .child(tools::relative_now(now - 86400 * 3)),
            )
            .into_any_element()
    }

    fn tool_uuid(&mut self, theme: &Theme, _cx: &mut Context<Self>) -> gpui::AnyElement {
        let uuids: Vec<String> = (0..5).map(|_| tools::gen_uuid_v4()).collect();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("UUID 生成器", "v4 批量生成", theme))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(uuids.iter().enumerate().map(|(ix, u)| {
                        div()
                            .id(("uuid-row", ix))
                            .flex()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .rounded(theme.radius_md)
                            .bg(theme.bg_input)
                            .border_1()
                            .border_color(theme.border)
                            .font_family(theme.font_mono.clone())
                            .text_size(px(13.))
                            .cursor_pointer()
                            .hover(|s| s.border_color(theme.primary))
                            .on_click({
                                let u = u.clone();
                                move |_, _, cx| {
                                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(u.clone()));
                                    toast(ToastKind::Success, "UUID 已复制", cx);
                                }
                            })
                            .child(u.clone())
                    })),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.fg_subtle)
                    .child("点击任一 UUID 复制到剪贴板"),
            )
            .into_any_element()
    }

    fn tool_radix(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_line.read(cx).text().trim().to_string();
        let n: i64 = src.parse().unwrap_or(0);
        let [b, o, d, h] = tools::radix_convert(n);
        let e_line = self.input_line.clone();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("进制转换", "2/8/10/16 四向联动", theme))
            .child(div().child(text_input("radix-in", &e_line).prefix_icon("hash").clearable()))
            .child(hash_row("二进制", b.clone(), theme))
            .child(hash_row("八进制", o.clone(), theme))
            .child(hash_row("十进制", d.clone(), theme))
            .child(hash_row("十六进制", h.clone(), theme))
            .into_any_element()
    }

    fn tool_color(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let hex = self.color_hex.read(cx).text().trim().to_string();
        let (rgb, hsl, err) = match gpui_kit::util::parse_hex_color(&hex) {
            Some((r, g, b)) => {
                let (h, s, l) = gpui_kit::util::rgb_to_hsl(r, g, b);
                (
                    format!("rgb({}, {}, {})", (r * 255.) as u8, (g * 255.) as u8, (b * 255.) as u8),
                    format!("hsl({:.0}, {:.0}%, {:.0}%)", h * 360., s * 100., l * 100.),
                    false,
                )
            }
            None => (String::new(), String::new(), !hex.is_empty()),
        };
        let swatch_color = gpui_kit::util::parse_hex_color(&hex)
            .map(|(r, g, b)| gpui::hsla(r as f32, g as f32, b as f32, 1.0))
            .unwrap_or(theme.bg_hover);
        let e_hex = self.color_hex.clone();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("颜色转换", "HEX ↔ RGB ↔ HSL", theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .w(px(120.))
                            .child(text_input("color-in", &e_hex).clearable()),
                    )
                    .child(
                        div()
                            .size(px(48.))
                            .rounded(theme.radius_lg)
                            .bg(swatch_color)
                            .border_1()
                            .border_color(theme.border),
                    ),
            )
            .when(err, |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(theme.danger)
                        .child("非法 HEX 颜色,示例:#3366CC"),
                )
            })
            .child(hash_row("RGB", rgb.clone(), theme))
            .child(hash_row("HSL", hsl.clone(), theme))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children((0..5).map(|i| {
                        let l = 0.25 + i as f64 * 0.125;
                        let base = gpui_kit::util::parse_hex_color(&hex).unwrap_or((0.2, 0.4, 0.8));
                        let (h, s, l0) = gpui_kit::util::rgb_to_hsl(base.0, base.1, base.2);
                        div()
                            .flex_1()
                            .h(px(28.))
                            .rounded(theme.radius_sm)
                            .bg(gpui::hsla(h as f32, s as f32, l.clamp(0.05, 0.95) as f32, 1.0))
                            .border_1()
                            .border_color(theme.border)
                    })),
            )
            .into_any_element()
    }

    fn tool_diff(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let a = self.input_a.read(cx).text().to_string();
        let b = self.input_b.read(cx).text().to_string();
        let d = tools::diff_lines(&a, &b, self.diff_ignore_ws);
        let adds = d.iter().filter(|(op, _)| *op == '+').count();
        let dels = d.iter().filter(|(op, _)| *op == '-').count();
        let e_a = self.input_a.clone();
        let e_b = self.input_b.clone();
        let e_self = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("文本 Diff", "LCS 行级对比(红删绿增)", theme))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child({
                        let e = e_self.clone();
                        Checkbox::new("diff-ws", self.diff_ignore_ws)
                            .label("忽略行尾空白")
                            .on_change(move |_, cx| {
                                e.update(cx, |this, cx| {
                                    this.diff_ignore_ws = !this.diff_ignore_ws;
                                    cx.notify();
                                });
                            })
                    })
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme.fg_muted)
                            .child(format!("+{} -{}", adds, dels)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("原文 A"))
                            .child(div().h(px(180.)).child(text_input("d-a", &e_a))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("对比 B"))
                            .child(div().h(px(180.)).child(text_input("d-b", &e_b))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .flex_1()
                    .child(div().text_size(px(11.)).text_color(theme.fg_subtle).child("Diff 结果"))
                    .child(
                        div()
                            .id("diff-out")
                            .flex_1()
                            .p_2()
                            .rounded(theme.radius_md)
                            .bg(theme.bg_input)
                            .border_1()
                            .border_color(theme.border)
                            .font_family(theme.font_mono.clone())
                            .text_size(px(12.))
                            .overflow_y_scroll()
                            .children(d.iter().map(|(op, line)| {
                                let color = match op {
                                    '+' => theme.success,
                                    '-' => theme.danger,
                                    _ => theme.fg_muted,
                                };
                                let prefix = match op {
                                    '+' => "+ ",
                                    '-' => "- ",
                                    _ => "  ",
                                };
                                div()
                                    .flex()
                                    .gap_2()
                                    .text_color(color)
                                    .child(div().w(px(14.)).child(prefix.to_string()))
                                    .child(div().flex_1().min_w_0().child(line.clone()))
                            })),
                    ),
            )
            .into_any_element()
    }

    fn tool_text(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let src = self.input_a.read(cx).text().to_string();
        let (chars, words, lines) = tools::text_stats(&src);
        let e_in = self.input_a.clone();
        let e_self = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("文本工具", "统计 / 大小写 / 命名风格 / Lorem", theme))
            .child(div().h(px(120.)).child(text_input("t-in", &e_in).clearable()))
            .child(
                div()
                    .flex()
                    .gap_4()
                    .text_size(px(12.))
                    .text_color(theme.fg_muted)
                    .child(format!("字符 {}", chars))
                    .child(format!("单词 {}", words))
                    .child(format!("行 {}", lines)),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(["camel", "snake", "kebab", "upper", "lower"].iter().map(|mode| {
                        let e = e_self.clone();
                        let m = mode.to_string();
                        Button::new(("case", mode.len() + mode.chars().next().map(|c| c as usize).unwrap_or(0)), *mode)
                            .small()
                            .on_click(move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    let src = this.input_a.read(cx).text().to_string();
                                    let out = tools::case_convert(&src, &m);
                                    this.input_a.update(cx, |s, cx| s.set_text_quiet(out, cx));
                                    cx.notify();
                                });
                            })
                    })),
            )
            .child(hash_row("驼峰", tools::case_convert(&src, "camel"), theme))
            .child(hash_row("下划线", tools::case_convert(&src, "snake"), theme))
            .child(hash_row("Lorem 示例", tools::lorem(1), theme))
            .into_any_element()
    }

    fn tool_calc(&mut self, theme: &Theme, cx: &mut Context<Self>) -> gpui::AnyElement {
        let root_id = cx.entity().entity_id();
        let src = self.input_line.read(cx).text().trim().to_string();
        let (result, err) = if src.is_empty() {
            (String::new(), false)
        } else {
            match gpui_kit::util::eval_math(&src) {
                Ok(v) => {
                    let s = if v.fract() == 0.0 && v.abs() < 1e15 {
                        format_thousands(v as i64)
                    } else {
                        format!("{}", v)
                    };
                    (s, false)
                }
                Err(e) => (e, true),
            }
        };
        let e_line = self.input_line.clone();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .size_full()
            .child(section_title("计算器", "四则/括号/幂/取模 + 常量 pi/e/tau", theme))
            .child(div().child(text_input("calc-in", &e_line).prefix_icon("terminal").clearable()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(28.))
                            .font_weight(gpui::FontWeight::BOLD)
                            .font_family(theme.font_mono.clone())
                            .text_color(if err { theme.danger } else { theme.primary })
                            .child(if result.is_empty() { "—".to_string() } else { result.clone() }),
                    )
                    .children([
                        Button::new("c-pi", "pi").small().ghost().on_click({
                            let e = e_line.clone();
                            move |_, _, cx| {
                                e.update(cx, |s, cx| s.set_text_quiet(format!("{}*pi", s.text()), cx));
                                cx.notify(root_id);
                            }
                        }),
                        Button::new("c-tau", "tau").small().ghost().on_click({
                            let e = e_line.clone();
                            move |_, _, cx| {
                                e.update(cx, |s, cx| s.set_text_quiet(format!("{}*tau", s.text()), cx));
                                cx.notify(root_id);
                            }
                        }),
                    ]),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.fg_subtle)
                    .child("支持:+ - * / % ^ 与括号,例:(12+8)*3 / 2^10 / 10%3"),
            )
            .into_any_element()
    }
}

fn section_title(title: &str, desc: &str, theme: &Theme) -> gpui::AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_size(px(16.))
                .font_weight(gpui::FontWeight::BOLD)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(theme.fg_subtle)
                .child(desc.to_string()),
        )
        .into_any_element()
}

fn hash_row(label: &str, value: String, theme: &Theme) -> gpui::AnyElement {
    let label_id = label.len() + label.chars().next().map(|c| c as usize).unwrap_or(0);
    let label = label.to_string();
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(70.))
                .text_size(px(12.))
                .text_color(theme.fg_subtle)
                .child(label.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .px_2()
                .py_1p5()
                .rounded(theme.radius_sm)
                .bg(theme.bg_input)
                .border_1()
                .border_color(theme.border)
                .font_family(theme.font_mono.clone())
                .text_size(px(11.))
                .truncate()
                .child(value.clone()),
        )
        .child(
            IconButton::new(("copy", label_id), "copy")
                .size(px(22.))
                .icon_size(px(11.))
                .tooltip("复制")
                .on_click({
                    let v = value.clone();
                    move |_, _, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(v.clone()));
                        toast(ToastKind::Success, format!("{} 已复制", label), cx);
                    }
                }),
        )
        .into_any_element()
}
