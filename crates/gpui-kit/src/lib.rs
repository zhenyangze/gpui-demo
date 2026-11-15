//! # gpui-kit
//!
//! GPUI(https://gpui.rs)可复用组件库与设计系统。
//!
//! ## 快速上手
//!
//! ```ignore
//! // 最小骨架(示意;完整可运行示例见 kit examples/hello_kit.rs)
//! use gpui::{prelude::*, Application};
//!
//! fn main() {
//!     Application::new()
//!         .with_assets(gpui_kit::icons::asset_source()) // 注册内嵌图标
//!         .run(|cx: &mut App| {
//!             gpui_kit::init_kit(cx); // 注册主题/快捷键/通知
//!             cx.open_window(WindowOptions::default(), |_, cx| cx.new(|_| MyRoot)).unwrap();
//!         });
//! }
//!
//! struct MyRoot;
//! impl gpui::Render for MyRoot {
//!     fn render(&mut self, _: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
//!         gpui::div().child("Hello gpui-kit")
//!     }
//! }
//! ```
//!
//! ## 设计约定
//! - `Theme` 为全局 Global,所有组件读取它取 token;
//! - 交互组件无内部业务状态,通过回调回传;
//! - `Application::new().with_assets(...)` + `init_kit(cx)` 必须在打开窗口前完成。

pub mod components;
pub mod core;
pub mod icons;
pub mod markdown;
pub mod nav;
pub mod perf;
pub mod sortable;
pub mod text;
pub mod theme;
pub mod util;

pub mod prelude {
    // 透传 gpui 常用项,业务 demo 只需 `use gpui_kit::prelude::*`
    pub use gpui::{
        div, prelude::*, px, App, Application, Context, Entity, IntoElement, Pixels, Render,
        RenderOnce, SharedString, Styled, Window,
    };
    pub use crate::components::basic::*;
    pub use crate::components::button::{Button, ButtonSize, ButtonVariant, IconButton};
    pub use crate::components::controls::{Checkbox, Radio, Slider, Switch};
    pub use crate::components::input::{text_area, text_input, InputState, TextInput};
    pub use crate::components::nav::{
        Breadcrumb, NavItem, Pagination, Segmented, Tabs, Toolbar,
    };
    pub use crate::components::overlay::{render_toasts, toast, toast_with_duration, Modal, ToastKind};
    pub use crate::components::palette::{PaletteCommand, PaletteState};
    pub use crate::components::select::{select, MenuItem, MenuOverlay, Select, SelectState};
    pub use crate::components::table::{Table, TableColumn};
    pub use crate::components::tree::{TreeRow, TreeView};
    pub use crate::icons::{icon, Icon};
    pub use crate::markdown::{render_markdown, MarkdownView};
    pub use crate::theme::{hsla, spacing, Accent, Theme, ThemeMode};
    pub use crate::components::basic::Semantic;
}

use gpui::App;

/// 一步完成 kit 初始化:默认深色主题、快捷键、通知系统、命令中心、性能计数。
/// 图标资源请在 `Application::new().with_assets(gpui_kit::icons::asset_source())` 注册。
pub fn init_kit(cx: &mut App) {
    init_kit_with(theme::Theme::dark(), cx);
}

/// 使用自定义主题初始化
pub fn init_kit_with(theme: theme::Theme, cx: &mut App) {
    cx.set_global(theme);
    cx.set_global(core::command::CommandRegistry::default());
    perf::init(cx);
    components::input::bind_input_keys(cx);
    components::palette::bind_palette_keys(cx);
    components::overlay::install_toasts(cx);
}

/// kit 版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
