// demo-flowmail —— 三栏式桌面邮件客户端(客户端类 demo)
mod state;
mod ui;

use gpui::{
    px, size, App, Application, Bounds, KeyBinding, WindowBounds, WindowOptions,
    prelude::*,
};
use gpui_kit::prelude::*;

use crate::ui::FlowMailApp;

fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source())
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx);
            bind_actions(cx);
            let bounds = Bounds::centered(None, size(px(1200.), px(780.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("FlowMail".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    // 启动参数:demo-flowmail [compose|<mail_id>]
                    let arg = std::env::args().nth(1);
                    cx.new(|cx| FlowMailApp::new_with_arg(cx, arg))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}

gpui::actions!(flowmail, [
    Compose, FocusSearch, NextMail, PrevMail, ToggleRead, ToggleStar, DeleteMail,
    Help, Settings, Quit,
]);

fn bind_actions(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("c", Compose, Some("FlowMail")),
        KeyBinding::new("/", FocusSearch, Some("FlowMail")),
        KeyBinding::new("j", NextMail, Some("FlowMail")),
        KeyBinding::new("k", PrevMail, Some("FlowMail")),
        KeyBinding::new("e", ToggleRead, Some("FlowMail")),
        KeyBinding::new("s", ToggleStar, Some("FlowMail")),
        KeyBinding::new("delete", DeleteMail, Some("FlowMail")),
        KeyBinding::new("backspace", DeleteMail, Some("FlowMail")),
        KeyBinding::new("?", Help, Some("FlowMail")),
        KeyBinding::new("cmd-n", Compose, Some("FlowMail")),
        KeyBinding::new("cmd-e", Settings, Some("FlowMail")),
        KeyBinding::new("cmd-q", Quit, None),
    ]);
}

/// 快捷键速查表(帮助 Modal 用)
pub const SHORTCUTS: &[(&str, &str)] = &[
    ("c / ⌘N", "写邮件"),
    ("/", "搜索邮件"),
    ("j / k", "下一封 / 上一封"),
    ("e", "标记已读/未读"),
    ("s", "星标切换"),
    ("Del", "移入垃圾箱"),
    ("⌘E", "设置"),
    ("?", "本帮助"),
    ("⌘Q", "退出"),
];
