// demo-agentconsole —— AI 智能体对话工作台(智能体类 demo)
mod engine;
mod state;
mod ui;

use gpui::{
    px, size, App, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, prelude::*,
};
use gpui_kit::prelude::*;

use crate::ui::AgentApp;

gpui::actions!(agent, [NewChat, OpenPalette, Help, StopGen, Quit]);

fn main() {
    Application::new()
        .with_assets(gpui_kit::icons::asset_source())
        .run(|cx: &mut App| {
            gpui_kit::init_kit(cx);
            cx.bind_keys([
                KeyBinding::new("cmd-n", NewChat, Some("Agent")),
                KeyBinding::new("cmd-k", OpenPalette, Some("Agent")),
                KeyBinding::new("?", Help, Some("Agent")),
                KeyBinding::new("escape", StopGen, Some("Agent")),
                KeyBinding::new("cmd-q", Quit, None),
            ]);
            let bounds = Bounds::centered(None, size(px(1200.), px(780.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some("AgentConsole".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let arg = std::env::args().nth(1); // "tasks" 预置任务面板演示
                    cx.new(|cx| AgentApp::new_with_arg(cx, arg, window))
                },
            )
            .unwrap();
            cx.activate(true);
        });
}

pub const SHORTCUTS: &[(&str, &str)] = &[
    ("Enter", "发送消息"),
    ("Shift+Enter", "换行"),
    ("↑/↓", "翻阅历史输入(输入框聚焦时)"),
    ("Esc", "停止生成"),
    ("⌘N", "新建会话"),
    ("⌘K", "命令面板"),
    ("?", "本帮助"),
    ("⌘Q", "退出"),
];
