//! 画廊各页面:主题 / 基础 / 表单 / 导航 / 反馈浮层 / 数据展示 / 关于。

use gpui::{canvas, px, prelude::*, App, Context, Div, MouseButton};
use std::collections::HashMap;

pub fn hl(h: f64, s: f64, l: f64) -> gpui::Hsla {
    gpui_kit::theme::hsla(h, s, l, 1.0)
}
use gpui_kit::prelude::*;

use crate::GalleryApp;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Overview,
    Basics,
    Forms,
    Navigation,
    Feedback,
    Data,
    Architecture,
    About,
}

impl Page {
    pub const NAV: [(Self, &'static str, &'static str); 8] = [
        (Self::Overview, "主题与色板", "palette"),
        (Self::Basics, "基础组件", "box"),
        (Self::Forms, "表单组件", "edit"),
        (Self::Navigation, "导航组件", "list"),
        (Self::Feedback, "反馈浮层", "info"),
        (Self::Data, "数据展示", "grid"),
        (Self::Architecture, "架构演示", "terminal"),
        (Self::About, "关于", "help"),
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "主题与色板",
            Self::Basics => "基础组件",
            Self::Forms => "表单组件",
            Self::Navigation => "导航组件",
            Self::Feedback => "反馈与浮层",
            Self::Data => "数据展示",
            Self::Architecture => "架构演示(L2 核心能力)",
            Self::About => "关于 gpui-kit",
        }
    }
}

/// LOGO 圆点
pub fn logo_mark(cx: &App) -> gpui::AnyElement {
    let theme = cx.global::<Theme>();
    div()
        .flex()
        .gap(px(3.))
        .child(div().size(px(10.)).rounded_full().bg(theme.primary))
        .child(
            div()
                .size(px(10.))
                .rounded_full()
                .bg(theme.primary.opacity(0.6)),
        )
        .child(
            div()
                .size(px(10.))
                .rounded_full()
                .bg(theme.primary.opacity(0.3)),
        )
        .into_any_element()
}

fn themed_section_header(name: &str, desc: &str, cx: &App) -> Div {
    let theme = cx.global::<Theme>();
    div()
        .flex()
        .flex_col()
        .gap_1()
        .mt_2()
        .child(
            div()
                .text_size(px(15.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .child(name.to_string()),
        )
        .child(
            div()
                .text_size(px(12.))
                .text_color(theme.fg_subtle)
                .child(desc.to_string()),
        )
}

// ------------------------- 主题与色板 -------------------------

pub fn render_overview(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let _ = app;
    let rows: Vec<(&str, gpui::Hsla, &str)> = vec![
        ("bg", theme.bg, "页面背景"),
        ("bg_panel", theme.bg_panel, "面板背景"),
        ("bg_elevated", theme.bg_elevated, "浮层背景"),
        ("bg_input", theme.bg_input, "输入背景"),
        ("fg", theme.fg, "主文字"),
        ("fg_muted", theme.fg_muted, "次要文字"),
        ("primary", theme.primary, "主色"),
        ("success", theme.success, "成功"),
        ("warning", theme.warning, "警告"),
        ("danger", theme.danger, "危险"),
        ("info", theme.info, "信息"),
        ("border", theme.border, "边框"),
    ];
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(themed_section_header(
            "语义色 Token",
            "所有组件从 Theme Global 读取;切换深浅/accent 即全局生效",
            cx,
        ))
        .child(
            div()
                .grid()
                .grid_cols(4)
                .gap_3()
                .children(rows.into_iter().map(|(token, color, desc)| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .h(px(56.))
                                .rounded(theme.radius_md)
                                .bg(color)
                                .border_1()
                                .border_color(theme.border),
                        )
                        .child(
                            div()
                                .font_family(theme.font_mono.clone())
                                .text_size(px(11.))
                                .text_color(theme.fg)
                                .child(token.to_string()),
                        )
                        .child(
                            div().text_size(px(11.)).text_color(theme.fg_subtle).child(desc),
                        )
                })),
        )
        .child(themed_section_header("Accent 主色", "六种预置主色", cx))
        .child(
            div()
                .flex()
                .gap_3()
                .children(Accent::ALL.iter().map(|a| {
                    ColorSwatch::new(a.base()).size(px(44.)).label(a.label())
                })),
        )
        .child(themed_section_header("圆角刻度", "sm / md / lg", cx))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .size(px(40.))
                        .rounded(theme.radius_sm)
                        .bg(theme.primary.opacity(0.3))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.))
                        .child("sm"),
                )
                .child(
                    div()
                        .size(px(40.))
                        .rounded(theme.radius_md)
                        .bg(theme.primary.opacity(0.3))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.))
                        .child("md"),
                )
                .child(
                    div()
                        .size(px(40.))
                        .rounded(theme.radius_lg)
                        .bg(theme.primary.opacity(0.3))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.))
                        .child("lg"),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(theme.fg_muted)
                        .child("spacing(n) = 4n px"),
                ),
        )
        .into_any_element()
}

// ------------------------- 基础组件 -------------------------

pub fn render_basics(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(themed_section_header("Button 按钮", "5 变体 × 3 尺寸 + 图标/禁用/加载", cx))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(Button::new("b1", "Primary").primary())
                .child(Button::new("b2", "Secondary"))
                .child(Button::new("b3", "Danger").danger())
                .child(Button::new("b4", "Ghost").ghost())
                .child(Button::new("b5", "Link").link())
                .child(Button::new("b6", "图标+主色").primary().icon("plus"))
                .child(
                    Button::new("b7", "加载中")
                        .loading(app.btn_loading)
                        .disabled(app.btn_loading),
                )
                .child(Button::new("b8", "禁用").disabled(app.btn_disabled))
                .child(Button::new("b9", "小号").small())
                .child(Button::new("b10", "大号").large().primary())
                .child(
                    Button::new("b-toggle", "切换禁用/加载")
                        .ghost()
                        .icon("settings")
                        .on_click(move |_, _, cx| {
                            entity.update(cx, |this, cx| {
                                this.btn_disabled = !this.btn_disabled;
                                this.btn_loading = !this.btn_loading;
                                cx.notify();
                            });
                        }),
                ),
        )
        .child(themed_section_header("IconButton 图标按钮", "方形图标交互", cx))
        .child(
            div()
                .flex()
                .gap_2()
                .child(IconButton::new("i1", "search"))
                .child(IconButton::new("i2", "plus"))
                .child(IconButton::new("i3", "settings"))
                .child(IconButton::new("i4", "trash").danger())
                .child(IconButton::new("i5", "download").disabled(true)),
        )
        .child(themed_section_header("Badge / Tag / Avatar", "状态标识与人像", cx))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_2()
                .child(Badge::new("默认"))
                .child(Badge::new("成功").semantic(Semantic::Success))
                .child(Badge::new("警告").semantic(Semantic::Warning))
                .child(Badge::new("危险").semantic(Semantic::Danger))
                .child(Badge::new("信息").semantic(Semantic::Info))
                .child(Badge::new("12").semantic(Semantic::Primary).dot())
                .child(Tag::new("可关闭标签").closable(|_, _, cx| {
                    toast(ToastKind::Info, "标签已关闭", cx);
                }))
                .child(Avatar::new("张三"))
                .child(Avatar::new("Ada Lovelace").size(px(36.)))
                .child(Avatar::new("Zed")),
        )
        .child(themed_section_header("Spinner / ProgressBar / Skeleton", "加载与占位", cx))
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(Spinner::new())
                .child(Spinner::new().size(px(24.)).color(theme.danger))
                .child(div().w(px(220.)).child(ProgressBar::new(0.64)))
                .child(
                    div()
                        .w(px(120.))
                        .child(ProgressBar::new(0.5).indeterminate()),
                )
                .child(Skeleton::new(px(110.), px(14.)))
                .child(Skeleton::new(px(28.), px(28.)).round()),
        )
        .child(themed_section_header("Card / Stat / EmptyState", "容器与数据", cx))
        .child(
            div()
                .grid()
                .grid_cols(3)
                .gap_3()
                .child(
                    Card::new()
                        .title("卡片标题")
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.fg_muted)
                                .child("卡片正文,支持标题与任意子元素。"),
                        )
                        .child(Tag::new("Card")),
                )
                .child(
                    Card::new()
                        .child(Stat::new("总请求", "128,934").delta(12.5, true)),
                )
                .child(
                    Card::new()
                        .child(Stat::new("错误率", "0.12%").delta(0.8, false)),
                ),
        )
        .child(
            div()
                .h(px(160.))
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .child(
                    EmptyState::new("box", "暂无数据", "这里会显示空状态与动作按钮").action(
                        Button::new("empty-act", "新建条目").primary(),
                    ),
                ),
        )
        .child(themed_section_header("Divider / Kbd / Tooltip", "细节元素", cx))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child("左侧")
                        .child(divider())
                        .child("右侧"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Kbd::new("⌘"))
                        .child(Kbd::new("K"))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.fg_muted)
                                .child("打开命令面板"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .child(
                            div()
                                .id("tt1")
                                .tooltip(|window, cx| tooltip_view("这是提示信息", window, cx))
                                .cursor_pointer()
                                .child("悬停我 ①"),
                        )
                        .child(
                            div()
                                .id("tt2")
                                .tooltip(|window, cx| tooltip_view("第二条提示", window, cx))
                                .cursor_pointer()
                                .child("悬停我 ②"),
                        ),
                ),
        )
        .into_any_element()
}

// ------------------------- 表单组件 -------------------------

pub fn render_forms(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();
    let error_input_len = app.input_error.read(cx).text().chars().count();

    let radio_labels = ["邮件", "短信", "推送"];

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(themed_section_header("TextInput 单行输入", "占位/前缀图标/清除/错误态;⌘A ⌘C ⌘X ⌘V 可用", cx))
        .child(text_input("f-input", &app.input_main.clone()).prefix_icon("search").clearable())
        .child(
            div()
                .text_size(px(12.))
                .text_color(theme.fg_muted)
                .child(format!("当前值:«{}»", app.input_main.read(cx).text())),
        )
        .child(
            text_input("f-input-error", &app.input_error.clone())
                .error(error_input_len > 0 && error_input_len < 3)
                .clearable()
                .suffix(
                    div()
                        .text_size(px(11.))
                        .text_color(if error_input_len > 0 && error_input_len < 3 {
                            theme.danger
                        } else {
                            theme.fg_subtle
                        })
                        .child("≥3 字符消除错误"),
                ),
        )
        .child(themed_section_header("TextArea 多行输入", "自适应行数,Shift+Enter 换行,↑↓ 移动光标", cx))
        .child(text_area("f-area", &app.input_area.clone()))
        .child(
            div()
                .text_size(px(12.))
                .text_color(theme.fg_muted)
                .child(format!(
                    "共 {} 字 / {} 行",
                    app.input_area.read(cx).text().chars().count(),
                    app.input_area.read(cx).text().lines().count().max(1)
                )),
        )
        .child(themed_section_header("Switch / Checkbox / Radio", "布尔与单选", cx))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child({
                    let e = entity.clone();
                    div().flex().items_center().gap_2().child(
                        Switch::new("f-sw", app.switch_a).on_change(move |_w, cx| {
                            e.update(cx, |this, cx| {
                                this.switch_a = !this.switch_a;
                                cx.notify();
                            });
                        }),
                    )
                })
                .child(div().text_size(px(12.)).child(if app.switch_a { "开关:开" } else { "开关:关" }))
                .child({
                    let e = entity.clone();
                    Checkbox::new("f-check", app.check_b)
                        .label("同意服务条款")
                        .on_change(move |_w, cx| {
                            e.update(cx, |this, cx| {
                                this.check_b = !this.check_b;
                                cx.notify();
                            });
                        })
                })
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .children(radio_labels.iter().enumerate().map(|(ix, label)| {
                            let e = entity.clone();
                            Radio::new(("f-radio", ix), app.radio_c == ix)
                                .label(*label)
                                .on_select(move |_w, cx| {
                                    e.update(cx, |this, cx| {
                                        this.radio_c = ix;
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        .child(themed_section_header("Slider 滑块", "拖拽/点击轨道调节(自绘拖拽)", cx))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child({
                    let e = entity.clone();
                    div().w(px(320.)).child(
                        Slider::new("f-slider", app.slider_v)
                            .range(0., 1.)
                            .step(0.01)
                            .on_change(move |v, _w, cx| {
                                e.update(cx, |this, cx| {
                                    this.slider_v = v;
                                    cx.notify();
                                });
                            }),
                    )
                })
                .child(
                    div()
                        .font_family(theme.font_mono.clone())
                        .text_size(px(12.))
                        .text_color(theme.primary)
                        .child(format!("{:.0}%", app.slider_v * 100.)),
                ),
        )
        .child(themed_section_header("Select 下拉选择", "受控弹出列表", cx))
        .child(select("f-select", &app.select_d.clone()).width(px(180.)))
        .child(themed_section_header("表单布局", "标签 + 控件 + 说明", cx))
        .child(
            Card::new().title("创建项目").child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(64.)).text_size(px(12.)).text_color(theme.fg_muted).child("名称"))
                            .child(div().flex_1().child(text_input("f-form-name", &app.input_main.clone()))),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(div().w(px(64.)).text_size(px(12.)).text_color(theme.fg_muted).child("可见性"))
                            .child(select("f-form-vis", &app.select_d.clone()).width(px(160.))),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap_2()
                            .child(Button::new("f-cancel", "取消").ghost())
                            .child(Button::new("f-save", "创建").primary().on_click(|_, _, cx| {
                                toast(ToastKind::Success, "项目已创建(演示)", cx);
                            })),
                    ),
            ),
        )
        .into_any_element()
}

// ------------------------- 导航 -------------------------

pub fn render_navigation(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();

    // 树数据(展平)
    #[derive(Clone)]
    struct Node {
        key: &'static str,
        label: &'static str,
        icon: &'static str,
        depth: usize,
        parent: Option<usize>,
    }
    let nodes = [
        Node { key: "src", label: "src", icon: "folder", depth: 0, parent: None },
        Node { key: "main.rs", label: "main.rs", icon: "file", depth: 1, parent: Some(0) },
        Node { key: "lib.rs", label: "lib.rs", icon: "file", depth: 1, parent: Some(0) },
        Node { key: "components", label: "components", icon: "folder", depth: 1, parent: Some(0) },
        Node { key: "button.rs", label: "button.rs", icon: "file", depth: 2, parent: Some(3) },
        Node { key: "input.rs", label: "input.rs", icon: "file", depth: 2, parent: Some(3) },
        Node { key: "docs", label: "docs", icon: "folder", depth: 0, parent: None },
        Node { key: "readme", label: "README.md", icon: "file", depth: 1, parent: Some(6) },
    ];
    let visible: Vec<(usize, &Node)> = nodes
        .iter()
        .enumerate()
        .filter(|(ix, n)| match n.parent {
            None => true,
            Some(p) => app.tree_expanded.get(p).copied().unwrap_or(true),
        })
        .collect();
    let _ = visible.len();
    let expanded_count = app.tree_expanded.iter().filter(|b| **b).count();
    let _ = expanded_count;

    let tree_rows: Vec<TreeRow> = nodes
        .iter()
        .enumerate()
        .filter(|(ix, n)| match n.parent {
            None => true,
            Some(p) => app.tree_expanded.get(p).copied().unwrap_or(true),
        })
        .map(|(ix, n)| TreeRow {
            key: n.key.into(),
            label: n.label.into(),
            icon: Some(n.icon.into()),
            depth: n.depth,
            has_children: n.icon == "folder",
            expanded: app.tree_expanded.get(ix).copied().unwrap_or(false),
            selected: ix == app.table_selected % nodes.len(),
            badge: None,
        })
        .collect();

    let tabs = vec!["全部".into(), "进行中".into(), "已完成".into()];
    let segs = vec!["日".into(), "周".into(), "月".into()];

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(themed_section_header("Tabs 页签", "点击切换", cx))
        .child({
            let e = entity.clone();
            Tabs::new("nav-tabs", tabs, app.tabs_e).on_change(move |ix, _w, cx| {
                e.update(cx, |this, cx| {
                    this.tabs_e = ix;
                    cx.notify();
                });
            })
        })
        .child(themed_section_header("Segmented 分段控件", "用于互斥视图切换", cx))
        .child({
            let e = entity.clone();
            Segmented::new("nav-seg", segs, app.seg_f).on_change(move |ix, _w, cx| {
                e.update(cx, |this, cx| {
                    this.seg_f = ix;
                    cx.notify();
                });
            })
        })
        .child(themed_section_header("TreeView 树形视图", "展开/折叠/选中(点击文件夹切换)", cx))
        .child({
            div()
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .p_2()
                .child(
                    TreeView::new(tree_rows).on_toggle({
                        let e = entity.clone();
                        move |key, _w, cx| {
                            e.update(cx, |this, cx| {
                                // key → 索引
                                let keys = ["src", "main.rs", "lib.rs", "components", "button.rs", "input.rs", "docs", "readme"];
                                if let Some(ix) = keys.iter().position(|k| *k == key) {
                                    if ix < this.tree_expanded.len() {
                                        this.tree_expanded[ix] = !this.tree_expanded[ix];
                                        cx.notify();
                                    }
                                }
                            });
                        }
                    })
                )
        })
        .child(themed_section_header("Breadcrumb / Pagination", "路径与分页", cx))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(Breadcrumb::new(vec!["工程".into(), "组件库".into(), "Button".into()]))
                .child(Pagination::new(2, 9).on_change(|page, _, cx| {
                    toast(ToastKind::Info, format!("跳转到第 {} 页", page + 1), cx);
                })),
        )
        .child(themed_section_header("Menu 菜单", "在此区域右键打开上下文菜单", cx))
        .child(
            div()
                .h(px(110.))
                .rounded(theme.radius_lg)
                .border_1()
                .border_color(theme.border_strong)
                .border_dashed()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(12.))
                .text_color(theme.fg_subtle)
                .on_mouse_down(MouseButton::Right, cx.listener(|this, event: &gpui::MouseDownEvent, _window, cx| {
                    let theme = cx.global::<Theme>().clone();
                    let _ = theme;
                    this.menu_open = Some((
                        event.position,
                        vec![
                            MenuItem::action("打开文件", |_, cx| toast(ToastKind::Info, "打开文件(演示)", cx)).icon("file"),
                            MenuItem::action("复制路径", |_, cx| toast(ToastKind::Success, "路径已复制", cx)).icon("copy").shortcut("⌘C"),
                            MenuItem::separator(),
                            MenuItem::action("删除", |_, cx| toast(ToastKind::Error, "已删除(演示)", cx)).danger(),
                        ],
                    ));
                    cx.notify();
                }))
                .child("在此右键 →"),
        )
        .into_any_element()
}

// ------------------------- 反馈浮层 -------------------------

pub fn render_feedback(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();
    let _ = theme;
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(themed_section_header("Toast 通知", "四种语义类型,自动消失,可手动关闭", cx))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(Button::new("t-success", "成功").on_click({
                    let e = entity.clone();
                    move |_, _, cx| {
                        e.update(cx, |this, _| {
                            let n = this.next_toast();
                            let _ = n;
                        });
                        toast(ToastKind::Success, "数据已保存", cx);
                    }
                }))
                .child(Button::new("t-error", "错误").on_click(|_, _, cx| {
                    toast(ToastKind::Error, "网络连接失败(演示)", cx);
                }))
                .child(Button::new("t-warn", "警告").on_click(|_, _, cx| {
                    toast(ToastKind::Warning, "磁盘空间不足 10%", cx);
                }))
                .child(Button::new("t-info", "信息").on_click(|_, _, cx| {
                    toast(ToastKind::Info, "新版本可用 v0.2.0", cx);
                })),
        )
        .child(themed_section_header("Modal 模态框", "遮罩 + 居中卡 + 关闭按钮", cx))
        .child(
            div().flex().child(
                Button::new("open-modal", "打开模态框").primary().on_click(cx.listener(
                    |this, _, _, cx| {
                        this.modal_open = true;
                        cx.notify();
                    },
                )),
            ),
        )
        .child(themed_section_header("CommandPalette 命令面板", "⌘K 打开,支持模糊搜索与键盘导航", cx))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new("open-palette", "打开命令面板 ⌘K").on_click(cx.listener(
                        |this, _, window, cx| {
                            this.palette.update(cx, |p, cx| p.open(window, cx));
                        },
                    )),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(theme.fg_subtle)
                        .child("支持 ↑↓ 选择、Enter 确认、Esc 关闭"),
                ),
        )
        .child(themed_section_header("MenuOverlay 下拉菜单", "右键上方演示区使用;这里是程序化打开", cx))
        .child(
            div().flex().child(
                Button::new("open-menu", "程序化打开菜单").on_click(cx.listener(|this, event: &gpui::ClickEvent, _, cx| {
                    this.menu_open = Some((
                        event.position(),
                        vec![
                            MenuItem::action("刷新", |_, cx| toast(ToastKind::Success, "已刷新", cx)).icon("refresh"),
                            MenuItem::action("导出", |_, cx| toast(ToastKind::Info, "已导出", cx)).icon("download"),
                            MenuItem::separator(),
                            MenuItem::action("危险操作", |_, cx| toast(ToastKind::Error, "危险!", cx)).danger(),
                        ],
                    ));
                    cx.notify();
                })),
            ),
        )
        .into_any_element()
}

// ------------------------- 数据展示 -------------------------

pub fn render_data(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();

    // 排序演示数据
    let mut rows: Vec<(String, i64, f64)> = vec![
        ("gpui-kit".into(), 5123, 4.8),
        ("flowmail".into(), 2891, 4.2),
        ("agent".into(), 4207, 4.6),
        ("2048".into(), 1948, 4.1),
        ("toolbox".into(), 3601, 4.4),
    ];
    let (sort_col, asc) = app.table_sort;
    match sort_col {
        1 => rows.sort_by_key(|r| if asc { r.1 } else { -r.1 }),
        2 => rows.sort_by(|a, b| if asc { a.2.partial_cmp(&b.2).unwrap() } else { b.2.partial_cmp(&a.2).unwrap() }),
        _ => rows.sort_by(|a, b| if asc { a.0.cmp(&b.0) } else { b.0.cmp(&a.0) }),
    }

    let spark_data = [3., 7., 4., 9., 6., 11., 8., 14., 10., 16., 13., 19.];

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(themed_section_header("Table 表格", "排序(点表头)/斑马纹/行选中", cx))
        .child({
            let cols = vec![
                TableColumn::new("项目", px(160.)).sortable(),
                TableColumn::new("星数", px(100.)).sortable(),
                TableColumn::new("评分", px(100.)).sortable(),
            ];
            let table_rows: Vec<Vec<gpui::AnyElement>> = rows
                .iter()
                .map(|(name, stars, score)| {
                    vec![
                        div().child(name.clone()).into_any_element(),
                        div().font_family(theme.font_mono.clone()).child(gpui_kit::util::format_thousands(*stars)).into_any_element(),
                        div().child(format!("{score:.1} ★")).into_any_element(),
                    ]
                })
                .collect();
            Table::new(cols, table_rows)
                .sort(app.table_sort.0, app.table_sort.1)
                .selected_row(app.table_selected.min(rows.len() - 1))
                .on_header_click({
                    let e = entity.clone();
                    move |ix, _w, cx| {
                        e.update(cx, |this, cx| {
                            if this.table_sort.0 == ix {
                                this.table_sort.1 = !this.table_sort.1;
                            } else {
                                this.table_sort = (ix, true);
                            }
                            cx.notify();
                        });
                    }
                })
                .on_row_click({
                    let e = entity.clone();
                    move |ix, _w, cx| {
                        e.update(cx, |this, cx| {
                            this.table_selected = ix;
                            cx.notify();
                        });
                    }
                })
        })
        .child(themed_section_header("Sparkline 迷你图", "canvas 绘制的迷你柱状趋势", cx))
        .child(
            Card::new()
                .title("近 12 周构建次数")
                .child(sparkline(spark_data.to_vec(), px(300.), px(48.), cx)),
        )
        .child(themed_section_header("Markdown 渲染", "标题/粗体/斜体/行内代码/代码块/列表/引用/链接", cx))
        .child(
            Card::new().child(
                MarkdownView::new(
                    "# gpui-kit\n支持 **加粗**、*斜体*、`inline_code` 与 [链接](https://gpui.rs)。\n\n- 列表项一\n- 列表项二\n\n> 引用块:GPUI 是 GPU 加速的 Rust UI 框架\n\n```rust\nlet app = Application::new().run(|cx| {\n    // ...\n});\n```\n\n1. 有序一\n2. 有序二",
                ),
            ),
        )
        .child(themed_section_header("KeyValue 键值对", "信息展示", cx))
        .child(
            Card::new().child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children([
                        ("框架", "GPUI 0.2.2 (runtime_shaders)"),
                        ("渲染", "Metal / GPU 加速"),
                        ("平台", "macOS 15.7 · x86_64"),
                        ("组件", "30 个"),
                    ].map(|(k, v)| {
                        div()
                            .flex()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(64.))
                                    .text_size(px(12.))
                                    .text_color(theme.fg_subtle)
                                    .child(k.to_string()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_family(theme.font_mono.clone())
                                    .child(v.to_string()),
                            )
                    })),
            ),
        )
        .into_any_element()
}

/// 迷你柱状图(canvas 绘制)
fn sparkline(data: Vec<f32>, w: gpui::Pixels, h: gpui::Pixels, cx: &App) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let max = data.iter().cloned().fold(0.0_f32, f32::max);
    let n = data.len();
    div().w(w).h(h).relative().child(
        canvas(
            move |_, _, _| (),
            move |bounds, _, window, _| {
                let unit = bounds.size.width / n as f32;
                let bar_w = unit * 0.6;
                let gap = unit * 0.4;
                for (ix, v) in data.iter().enumerate() {
                    let bh = (v / max) * bounds.size.height;
                    let x = bounds.left() + (bar_w + gap) * ix as f32;
                    let y = bounds.bottom() - bh;
                    window.paint_quad(gpui::fill(
                        gpui::Bounds::new(
                            gpui::point(x, y),
                            gpui::size(bar_w, bh),
                        ),
                        theme.primary.opacity(0.35 + 0.65 * (v / max)),
                    ));
                }
            },
        )
        .size_full(),
    ).into_any_element()
}

// ------------------------- 架构演示(L2 核心能力) -------------------------

pub fn render_architecture(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    let entity = cx.entity();
    let root_id = cx.entity().entity_id();

    // Store 演示:版本号 + 选择器派生值
    let store = app.arch_store.clone();
    let version = store.version(cx);
    let snapshot = store.read(cx);
    let clicks = snapshot.value().clicks;
    let note = snapshot.value().note.clone();
    let derived_len = note.chars().count();

    // 语法高亮演示:rust 源码 → spans → 着色行
    let code = "fn main() {\n    // GPUI 声明式 UI\n    let store = Store::new(42);\n    store.update(|v| *v += 1); // version + 1\n    assert_eq!(store.version(), 1);\n}";
    let spans = gpui_kit::text::highlight::tokenize("rust", code);
    let code_theme = highlight_theme(&theme);
    let code_lines = render_highlighted(code, &spans, &code_theme);

    // 可排序列表
    let sort_items = app.sort_items.clone();
    let list_id = 0xC0DE_u64;

    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(themed_section_header(
            "Store — 响应式状态容器",
            "版本号机制 + 选择器去重;点击按钮经 store.update 走完整数据流",
            cx,
        ))
        .child(
            Card::new()
                .title("Live Demo")
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .text_size(px(32.))
                                .font_weight(gpui::FontWeight::BOLD)
                                .font_family(theme.font_mono.clone())
                                .text_color(theme.primary)
                                .child(format!("{}", clicks)),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.fg_muted)
                                .font_family(theme.font_mono.clone())
                                .child(format!("store.version = {}", version)),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(theme.fg_subtle)
                                .child(format!("派生值 note.len = {} (选择器深比较去重)", derived_len)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child({
                            let st = store.clone();
                            Button::new("arch-inc", "store.update +1").primary().on_click(
                                move |_, _, cx| {
                                    st.update(cx, |v, _| {
                                        v.clicks += 1;
                                    });
                                    cx.notify(root_id);
                                },
                            )
                        })
                        .child({
                            let st = store.clone();
                            Button::new("arch-note", "更新 note(无关字段)")
                                .ghost()
                                .on_click(move |_, _, cx| {
                                    st.update(cx, |v, _| {
                                        v.note = format!("note@{}", v.clicks);
                                    });
                                    cx.notify(root_id);
                                })
                        }),
                ),
        )
        .child(themed_section_header(
            "Highlight — 语法高亮引擎",
            "独立 tokenizer(rust/json/toml)→ 着色区间;下例为真实渲染",
            cx,
        ))
        .child(
            Card::new().child(
                div()
                    .p_3()
                    .rounded(theme.radius_md)
                    .bg(theme.bg_input)
                    .font_family(theme.font_mono.clone())
                    .text_size(px(12.))
                    .flex()
                    .flex_col()
                    .children(code_lines),
            ),
        )
        .child(themed_section_header(
            "Sortable — 拖拽排序",
            "泛型 on_drag/on_drop 封装;拖动条目重排(纯逻辑 reorder 已单测)",
            cx,
        ))
        .child(
            div()
                .max_w(px(420.))
                .children(sort_items.iter().enumerate().map(|(ix, label)| {
                    gpui_kit::sortable::SortableRow::new(ix, label.clone(),
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .flex_1()
                            .min_w_0()
                            .child(Icon::new("list").size(px(12.)).text_color(theme.fg_subtle))
                            .child(div().flex_1().min_w_0().truncate().child(label.clone())),
                    )
                    .list_id(list_id)
                    .on_drop({
                        let e = entity.clone();
                        move |from, to, _w, cx| {
                            e.update(cx, |this, cx| {
                                if let Some(order) = gpui_kit::sortable::reorder(this.sort_items.len(), from, to) {
                                    this.sort_items = gpui_kit::sortable::apply_reorder(&this.sort_items, &order);
                                    cx.notify();
                                }
                            });
                        }
                    })
                })),
        )
        .child(themed_section_header("Router / History / Persist", "栈式导航 · 通用撤销 · 版本化迁移", cx))
        .child(
            div()
                .grid()
                .grid_cols(3)
                .gap_3()
                .child(
                    Card::new().title("Router").child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.fg_muted)
                            .child("DevToolbox 已迁移:侧栏点击入栈,⌘[ 后退,支持环形淘汰"),
                    ),
                )
                .child(
                    Card::new().title("History<T>").child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.fg_muted)
                            .child("2048 撤销由通用框架驱动(提交/合并/上限),纯逻辑 6 用例"),
                    ),
                )
                .child(
                    Card::new().title("Persist").child(
                        div()
                            .text_size(px(12.))
                            .text_color(theme.fg_muted)
                            .child("信封 {version,data} + 迁移链;2048 真实迁移 v1→v2 已单测"),
                    ),
                ),
        )
        .into_any_element()
}

/// 高亮配色(浅/深主题自适应)
fn highlight_theme(theme: &Theme) -> HashMap<gpui_kit::text::highlight::TokenKind, gpui::Hsla> {
    use gpui_kit::text::highlight::TokenKind as K;
    let mut m = HashMap::new();
    if theme.is_dark() {
        m.insert(K::Keyword, crate::pages::hl(0.78, 0.70, 0.72));
        m.insert(K::String, crate::pages::hl(0.35, 0.55, 0.62));
        m.insert(K::Number, crate::pages::hl(0.10, 0.60, 0.68));
        m.insert(K::Comment, theme.fg_subtle);
        m.insert(K::Type, crate::pages::hl(0.55, 0.62, 0.66));
        m.insert(K::Function, crate::pages::hl(0.60, 0.65, 0.68));
        m.insert(K::Punct, theme.fg_muted);
        m.insert(K::Plain, theme.fg);
    } else {
        m.insert(K::Keyword, crate::pages::hl(0.76, 0.65, 0.42));
        m.insert(K::String, crate::pages::hl(0.35, 0.55, 0.32));
        m.insert(K::Number, crate::pages::hl(0.08, 0.70, 0.38));
        m.insert(K::Comment, theme.fg_subtle);
        m.insert(K::Type, crate::pages::hl(0.55, 0.55, 0.40));
        m.insert(K::Function, crate::pages::hl(0.58, 0.55, 0.44));
        m.insert(K::Punct, theme.fg_muted);
        m.insert(K::Plain, theme.fg);
    }
    m
}

/// 按行渲染着色代码
fn render_highlighted(
    code: &str,
    spans: &[gpui_kit::text::highlight::Span],
    colors: &HashMap<gpui_kit::text::highlight::TokenKind, gpui::Hsla>,
) -> Vec<gpui::AnyElement> {
    // 按 \n 拆行,同时把 span 切到行内
    let mut line_elems = Vec::new();
    let mut line_start = 0usize;
    for line in code.split('\n') {
        let line_end = line_start + line.len();
        let mut children: Vec<gpui::AnyElement> = Vec::new();
        let mut cursor = line_start;
        for sp in spans.iter().filter(|s| s.end > line_start && s.start < line_end) {
            let s = sp.start.max(line_start);
            let e = sp.end.min(line_end);
            if s > cursor {
                children.push(
                    div()
                        .child(code[cursor..s].to_string())
                        .into_any_element(),
                );
            }
            children.push(
                div()
                    .text_color(*colors.get(&sp.kind).unwrap_or(&gpui::hsla(0., 0., 0.5, 1.)))
                    .child(code[s..e].to_string())
                    .into_any_element(),
            );
            cursor = e;
        }
        if cursor < line_end {
            children.push(div().child(code[cursor..line_end].to_string()).into_any_element());
        }
        line_elems.push(
            div()
                .flex()
                .flex_wrap()
                .min_h(px(16.))
                .children(if children.is_empty() {
                    vec![div().child(" ").into_any_element()]
                } else {
                    children
                })
                .into_any_element(),
        );
        line_start = line_end + 1;
    }
    line_elems
}

// ------------------------- 关于 -------------------------

pub fn render_about(cx: &mut Context<GalleryApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(logo_mark(cx))
                .child(
                    div()
                        .text_size(px(20.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("gpui-kit"),
                )
                .child(Badge::new(format!("v{}", gpui_kit::VERSION)).semantic(Semantic::Primary)),
        )
        .child(
            div()
                .text_size(px(13.))
                .text_color(theme.fg_muted)
                .child("GPUI 生态的可复用组件库与设计系统:30 个组件、主题 token、内嵌图标系统、命令面板与通知框架。本画廊即其官方 Showcase,亦作为新项目的起点模板。"),
        )
        .child(themed_section_header("使用方式", "三行接入", cx))
        .child(
            Card::new().child(
                MarkdownView::new("```rust\nApplication::new()\n    .with_assets(gpui_kit::icons::asset_source())\n    .run(|cx| {\n        gpui_kit::init_kit(cx);\n        // 打开窗口,root 视图里直接用 Button/InputState/Modal…\n    });\n```"),
            ),
        )
        .child(themed_section_header("组件统计", "覆盖基础/表单/导航/浮层/数据 五大类", cx))
        .child(
            div()
                .grid()
                .grid_cols(5)
                .gap_3()
                .child(Card::new().child(Stat::new("基础", "12")))
                .child(Card::new().child(Stat::new("表单", "7")))
                .child(Card::new().child(Stat::new("导航", "7")))
                .child(Card::new().child(Stat::new("浮层", "6")))
                .child(Card::new().child(Stat::new("数据", "6"))),
        )
        .into_any_element()
}
