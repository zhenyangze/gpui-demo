// demo-gallery —— gpui-kit 组件画廊(开发框架类 demo)
mod pages;

use gpui::{
    px, size, App, Application, Bounds, Context, FocusHandle, Focusable, KeyBinding, Window,
    WindowBounds, WindowOptions, prelude::*,
};
use gpui_kit::prelude::*;

use crate::pages::Page;

gpui::actions!(gallery, [OpenPalette, Quit]);

fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source())
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx);
            cx.bind_keys([
                KeyBinding::new("cmd-k", OpenPalette, Some("Gallery")),
                KeyBinding::new("cmd-q", Quit, None),
            ]);
            let bounds = Bounds::centered(None, size(px(1080.), px(720.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("gpui-kit Gallery".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    // 支持启动参数直达页面:demo-gallery [overview|basics|forms|navigation|feedback|data|about]
                    let page = std::env::args()
                        .nth(1)
                        .and_then(|arg| GalleryApp::page_from_name(&arg));
                    cx.new(|cx| GalleryApp::new_with_page(cx, page.unwrap_or(Page::Overview)))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}

pub struct GalleryApp {
    focus: FocusHandle,
    page: Page,
    search: Entity<InputState>,
    palette: Entity<PaletteState>,
    accent_menu: Entity<SelectState>,
    // 演示状态
    btn_disabled: bool,
    btn_loading: bool,
    switch_a: bool,
    check_b: bool,
    radio_c: usize,
    slider_v: f64,
    select_d: Entity<SelectState>,
    tabs_e: usize,
    seg_f: usize,
    modal_open: bool,
    toast_seq: usize,
    tree_expanded: [bool; 5],
    input_main: Entity<InputState>,
    input_error: Entity<InputState>,
    input_area: Entity<InputState>,
    table_sort: (usize, bool),
    table_selected: usize,
    menu_open: Option<(gpui::Point<gpui::Pixels>, Vec<MenuItem>)>,
    /// 架构演示:响应式 Store(计数器)
    arch_store: gpui_kit::core::store::Store<ArchDemo>,
    /// 架构演示:可排序列表项
    sort_items: Vec<String>,
}

/// Store 演示状态(任意可克隆结构)
#[derive(Clone, Debug, Default)]
pub struct ArchDemo {
    pub clicks: u64,
    pub note: String,
}

impl GalleryApp {
    fn page_from_name(name: &str) -> Option<Page> {
        Page::NAV
            .iter()
            .find(|(p, n, _)| {
                format!("{p:?}").to_lowercase() == name.to_lowercase() || *n == name
            })
            .map(|(p, _, _)| *p)
    }

    fn new_with_page(cx: &mut Context<Self>, page: Page) -> Self {
        let mut app = Self::new(cx);
        app.page = page;
        app
    }

    fn new(cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(cx).placeholder("搜索组件…"));
        let palette = cx.new(|cx| PaletteState::new(cx));
        let accent_menu = cx.new(|cx| {
            SelectState::new(
                cx,
                Accent::ALL
                    .iter()
                    .map(|a| a.label().to_string().into())
                    .collect(),
            )
            .on_change(|ix, _window, cx| {
                cx.update_global::<Theme, _>(|theme, _| theme.set_accent(Accent::ALL[ix]));
            })
        });
        let select_d = cx.new(|cx| {
            SelectState::new(cx, vec!["紧凑".into(), "舒适".into(), "宽敞".into()])
        });
        let input_main = cx.new(|cx| InputState::new(cx).placeholder("单行输入,试试 ⌘A/⌘C/⌘V"));
        let input_error = cx.new(|cx| InputState::new(cx).placeholder("错误状态示例"));
        let input_area = cx.new(|cx| InputState::new(cx).multi_line(4).placeholder("多行输入…(Shift+Enter 换行)"));
        Self {
            focus: cx.focus_handle(),
            input_main,
            input_error,
            input_area,
            page: Page::Overview,
            search,
            palette,
            accent_menu,
            btn_disabled: false,
            btn_loading: false,
            switch_a: true,
            check_b: false,
            radio_c: 1,
            slider_v: 0.7,
            select_d,
            tabs_e: 0,
            seg_f: 1,
            modal_open: false,
            toast_seq: 0,
            tree_expanded: [true, false, false, true, false],
            table_sort: (0, true),
            table_selected: 1,
            menu_open: None,
            arch_store: gpui_kit::core::store::create_store(ArchDemo::default(), cx),
            sort_items: vec!["需求分析".into(), "架构设计".into(), "编码实现".into(), "测试验收".into()],
        }
    }

    fn next_toast(&mut self) -> usize {
        self.toast_seq += 1;
        self.toast_seq
    }

}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let query = self.search.read(cx).text().trim().to_lowercase();

        // 命令面板命令(每次渲染重建,动作捕获实体)
        let entity = cx.entity();
        let mut palette_items = vec![PaletteCommand::new("theme", "切换深浅主题", |_, cx| {
            Theme::toggle_mode(cx);
        })];
        for (page, name, _) in Page::NAV.iter().cloned() {
            let entity = entity.clone();
            palette_items.push(PaletteCommand::new(
                leak_id(page),
                format!("跳转:{name}"),
                move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.page = page;
                        cx.notify();
                    });
                },
            ));
        }
        palette_items.push(PaletteCommand::new("toast", "弹一条通知", |_, cx| {
            toast(ToastKind::Success, "操作成功(演示)", cx);
        }));
        palette_items.push(PaletteCommand::new("modal", "打开示例模态框", |_, cx| {
            toast(ToastKind::Info, "在「反馈浮层」页打开模态框", cx);
        }));
        self.palette.update(cx, |p, cx| p.set_items(palette_items, cx));

        // 侧栏目录
        let nav_items: Vec<(Page, &'static str, &'static str)> = Page::NAV
            .iter()
            .filter(|(_, name, _)| {
                query.is_empty() || name.to_lowercase().contains(&query) || name.contains(&query)
            })
            .cloned()
            .collect();

        let page = self.page;

        let content: gpui::AnyElement = match page {
            Page::Overview => pages::render_overview(self, cx),
            Page::Basics => pages::render_basics(self, cx),
            Page::Forms => pages::render_forms(self, cx),
            Page::Navigation => pages::render_navigation(self, cx),
            Page::Feedback => pages::render_feedback(self, cx),
            Page::Data => pages::render_data(self, cx),
            Page::Architecture => pages::render_architecture(self, cx),
            Page::About => pages::render_about(cx),
        };

        div()
            .id("gallery-root")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.bg)
            .text_color(theme.fg)
            .font_family(theme.font_ui.clone())
            .key_context("Gallery")
            .track_focus(&self.focus.clone())
            .on_action(cx.listener(
                |this: &mut GalleryApp, _: &OpenPalette, window: &mut Window, cx: &mut Context<GalleryApp>| {
                    let pal = this.palette.clone();
                    pal.update(cx, |p, cx| p.open(window, cx));
                },
            ))
            .on_action(|_: &Quit, _window, cx| cx.quit())
            // 工具栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .h(px(52.))
                    .flex_shrink_0()
                    .border_b_1()
                    .border_color(theme.border)
                    .bg(theme.bg_panel)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(pages::logo_mark(cx))
                            .child(
                                div()
                                    .text_size(px(16.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("gpui-kit Gallery"),
                            )
                            .child(
                                Badge::new(format!("v{}", gpui_kit::VERSION))
                                    .semantic(Semantic::Primary),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("cmdk", "命令 ⌘K")
                                    .ghost()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.palette.update(cx, |p, cx| p.open(window, cx));
                                    })),
                            )
                            .child(
                                Button::new("toggle-theme", if theme.is_dark() { "浅色" } else { "深色" })
                                    .icon(if theme.is_dark() { "sun" } else { "moon" })
                                    .on_click(|_, _, cx| Theme::toggle_mode(cx)),
                            )
                            .child(select("accent-select", &self.accent_menu.clone()).width(px(110.))),
                    ),
            )
            // 主体
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(
                        div()
                            .w(px(210.))
                            .flex_shrink_0()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_2()
                            .border_r_1()
                            .border_color(theme.border)
                            .bg(theme.bg_panel)
                            .child(text_input("search", &self.search.clone()).prefix_icon("search"))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .children(nav_items.into_iter().map(|(p, name, icon)| {
                                        NavItem::new(("nav", p as usize), name)
                                            .icon(icon)
                                            .selected(page == p)
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.page = p;
                                                cx.notify();
                                            }))
                                    })),
                            )
                            .child(
                                div()
                                    .mt_auto()
                                    .text_size(px(11.))
                                    .text_color(theme.fg_subtle)
                                    .child("30 组件 · Rust 1.94"),
                            ),
                    )
                    .child(
                        div()
                            .id("content")
                            .flex_1()
                            .min_w_0()
                            .overflow_y_scroll()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_4()
                                    .p_6()
                                    .max_w(px(900.))
                                    .child(
                                        div()
                                            .text_size(px(22.))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child(page.title()),
                                    )
                                    .child(content),
                            ),
                    ),
            )
            // 右键菜单
            .children(self.menu_open.clone().map(|(pos, items)| {
                let weak = cx.weak_entity();
                MenuOverlay::new(pos, items).on_close(move |_, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.menu_open = None;
                        cx.notify();
                    });
                })
            }))
            // 模态
            .when(self.modal_open, |el| {
                el.child(
                    Modal::new("示例模态框")
                        .on_close({
                            let weak = cx.weak_entity();
                            move |_, cx| {
                                weak.update(cx, |this, cx| {
                                    this.modal_open = false;
                                    cx.notify();
                                }).ok();
                            }
                        })
                        .child(div().child("这是一个 gpui-kit Modal 组件,支持遮罩点击与关闭按钮。"))
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(Badge::new("Modal"))
                                .child(Badge::new("遮罩").semantic(Semantic::Info)),
                        )
                        .footer(
                            div().child(
                                Button::new("modal-ok", "好的")
                                    .primary()
                                    .on_click({
                                        let weak = cx.weak_entity();
                                        move |_, _, cx| {
                                            weak.update(cx, |this, cx| {
                                                this.modal_open = false;
                                                cx.notify();
                                            }).ok();
                                        }
                                    }),
                            ),
                        ),
                )
            })
            .child(self.palette.clone())
            .children(render_toasts(window, cx))
    }
}

impl Focusable for GalleryApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

fn leak_id(page: Page) -> &'static str {
    match page {
        Page::Architecture => "nav-architecture",
        Page::Overview => "nav-overview",
        Page::Basics => "nav-basics",
        Page::Forms => "nav-forms",
        Page::Navigation => "nav-navigation",
        Page::Feedback => "nav-feedback",
        Page::Data => "nav-data",
        Page::About => "nav-about",
    }
}
