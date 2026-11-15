//! gpui-kit 最小可运行示例:按钮 / 输入框 / 开关 / Toast / 主题。
//!
//! 运行:cargo run --example hello_kit -p gpui-kit

use gpui::{prelude::*, px, App, Application, Bounds, Context, Window, WindowBounds, WindowOptions};
use gpui_kit::prelude::*;

struct HelloKit {
    input: Entity<InputState>,
    area: Entity<InputState>,
    switched: bool,
    count: usize,
}

impl Render for HelloKit {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        div()
            .relative()
            .size_full()
            .bg(theme.bg)
            .text_color(theme.fg)
            .font_family(theme.font_ui.clone())
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("gpui-kit hello"),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        Button::new("inc", "计数 +1")
                            .primary()
                            .on_click(cx.listener(|this, _: &gpui::ClickEvent, _, cx| {
                                this.count += 1;
                                toast(
                                    ToastKind::Success,
                                    format!("当前计数 {}", this.count),
                                    cx,
                                );
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("danger", "危险操作")
                            .danger()
                            .on_click(cx.listener(|_, _, _, cx| {
                                toast(ToastKind::Error, "演示危险操作", cx);
                                cx.notify();
                            })),
                    )
                    .child(Button::new("dis", "禁用").disabled(true)),
            )
            .child(text_input("hello-input", &self.input).prefix_icon("search").clearable())
            .child(text_area("hello-area", &self.area))
            .child(
                div().flex().items_center().gap_2().child(format!(
                    "输入内容:«{}» / 多行:{} 字",
                    self.input.read(cx).text(),
                    self.area.read(cx).text().chars().count()
                )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child({
                        let entity = cx.entity();
                        Switch::new("sw", self.switched).on_change(move |_window, cx| {
                            entity.update(cx, |this, cx| {
                                this.switched = !this.switched;
                                cx.notify();
                            });
                        })
                    })
                    .child(if self.switched { "开" } else { "关" }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(gpui_kit::components::button::IconButton::new("ic1", "search"))
                    .child(gpui_kit::components::button::IconButton::new("ic2", "plus"))
                    .child(Icon::new("star").size(px(20.)))
                    .child(Icon::new("check").size(px(20.)).text_color(theme.success))
                    .child("图标实验(平铺)"),
            )
            .child(
                div()
                    .id("scroll-area")
                    .h(px(90.))
                    .overflow_y_scroll()
                    .border_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .p_2()
                            .child(gpui_kit::components::button::IconButton::new("ic3", "search"))
                            .child(Icon::new("star").size(px(20.)))
                            .child(Icon::new("check").size(px(20.)).text_color(theme.success))
                            .child("图标实验(滚动容器内)"),
                    ),
            )
            .children(render_toasts(_window, cx))
    }
}

fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source())
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx);
            let bounds = Bounds::centered(None, gpui::size(px(520.), px(560.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    cx.new(|cx| HelloKit {
                        input: cx.new(|cx| InputState::new(cx).placeholder("搜点什么…")),
                        area: cx.new(|cx| InputState::new(cx).multi_line(4).placeholder("多行输入…")),
                        switched: true,
                        count: 0,
                    })
                },
            )
            .unwrap();
            cx.activate(true);
        });
}
