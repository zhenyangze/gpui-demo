// demo-game2048 —— 2048 益智游戏(游戏类 demo)
mod board;
mod state;
mod ui;

use gpui::{
    px, size, App, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, prelude::*,
};
use gpui_kit::prelude::*;

use crate::ui::GameApp;

gpui::actions!(game, [
    MoveUp, MoveDown, MoveLeft, MoveRight, Undo, Restart, Pause, Help, Board, Quit,
]);

fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source())
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx);
            cx.bind_keys([
                KeyBinding::new("up", MoveUp, Some("Game")),
                KeyBinding::new("down", MoveDown, Some("Game")),
                KeyBinding::new("left", MoveLeft, Some("Game")),
                KeyBinding::new("right", MoveRight, Some("Game")),
                KeyBinding::new("w", MoveUp, Some("Game")),
                KeyBinding::new("s", MoveDown, Some("Game")),
                KeyBinding::new("a", MoveLeft, Some("Game")),
                KeyBinding::new("d", MoveRight, Some("Game")),
                KeyBinding::new("h", MoveLeft, Some("Game")),
                KeyBinding::new("j", MoveDown, Some("Game")),
                KeyBinding::new("k", MoveUp, Some("Game")),
                KeyBinding::new("l", MoveRight, Some("Game")),
                KeyBinding::new("u", Undo, Some("Game")),
                KeyBinding::new("r", Restart, Some("Game")),
                KeyBinding::new("p", Pause, Some("Game")),
                KeyBinding::new("?", Help, Some("Game")),
                KeyBinding::new("cmd-r", Board, Some("Game")),
                KeyBinding::new("cmd-q", Quit, None),
            ]);
            let bounds = Bounds::centered(None, size(px(680.), px(820.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("GPUI-2048".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| {
                    let arg = std::env::args().nth(1); // over | win | night
                    cx.new(|cx| GameApp::new(cx, arg))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}

pub const SHORTCUTS: &[(&str, &str)] = &[
    ("↑↓←→ / WASD / HJKL", "移动方块"),
    ("u", "撤销(最多 20 步)"),
    ("r", "重新开始"),
    ("p", "暂停/继续"),
    ("?", "本帮助"),
    ("⌘Q", "退出"),
];
