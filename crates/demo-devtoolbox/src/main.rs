// demo-devtoolbox —— 开发者小工具箱(小工具类 demo)
mod state;
mod tools;
mod ui;

use gpui::{
    px, size, App, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, prelude::*,
};
use gpui_kit::prelude::*;

use crate::ui::ToolboxApp;


fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source())
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx);
            cx.bind_keys([
                KeyBinding::new("cmd-k", OpenPalette, Some("Toolbox")),
                KeyBinding::new("cmd-1", Tool1, Some("Toolbox")),
                KeyBinding::new("cmd-2", Tool2, Some("Toolbox")),
                KeyBinding::new("cmd-3", Tool3, Some("Toolbox")),
                KeyBinding::new("cmd-4", Tool4, Some("Toolbox")),
                KeyBinding::new("cmd-5", Tool5, Some("Toolbox")),
                KeyBinding::new("cmd-6", Tool6, Some("Toolbox")),
                KeyBinding::new("cmd-q", Quit, None),
            ]);
            let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("DevToolbox".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    let arg = std::env::args().nth(1);
                    cx.new(|cx| ToolboxApp::new(cx, arg))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}

gpui::actions!(toolbox, [OpenPalette, Tool1, Tool2, Tool3, Tool4, Tool5, Tool6, Quit]);
