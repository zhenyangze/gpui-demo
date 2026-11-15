//! # gpui-kit 图标系统
//!
//! 内嵌 ~40 枚 24x24 线性图标(SVG 字符串,风格参考 Feather Icons,MIT)。
//! gpui 的 svg 渲染仅取 alpha 通道并以 `text_color` 着色,因此图标单色可调。
//! 通过 [`KitAssetSource`] 在 [`init_kit`](crate::init_kit) 时注册,
//! 使用方式:`Icon::new("mail")` 或 `icon("mail")`。

use gpui::{
    svg, App, AssetSource, IntoElement, RenderOnce, SharedString, Styled, prelude::FluentBuilder,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

pub struct KitAssetSource {
    icons: HashMap<SharedString, String>,
}

impl AssetSource for KitAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>, anyhow::Error> {
        let name = path
            .strip_prefix("icons/")
            .and_then(|s| s.strip_suffix(".svg"))
            .unwrap_or(path);
        if let Some(body) = self.icons.get(name) {
            return Ok(Some(Cow::Owned(body.as_bytes().to_vec())));
        }
        Ok(None)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>, anyhow::Error> {
        let _ = path;
        Ok(self.icons.keys().map(|k| format!("icons/{k}.svg").into()).collect())
    }
}

/// 线性图标模板(stroke 风格)
fn stroke(body: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"black\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{body}</svg>"
    )
}

/// 填充模板
fn filled(body: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"black\" stroke=\"none\">{body}</svg>"
    )
}

/// 收录的全部图标(名称 → svg body)
pub fn icon_map() -> HashMap<SharedString, String> {
    let mut m: HashMap<SharedString, String> = HashMap::new();
    let mut add = |k: &'static str, body: String| {
        m.insert(k.into(), body);
    };
    add("mail", stroke("<rect x=\"2\" y=\"4\" width=\"20\" height=\"16\" rx=\"2\"/><path d=\"m22 6-10 7L2 6\"/>"));
    add("inbox", stroke("<path d=\"M22 12h-6l-2 3h-4l-2-3H2\"/><path d=\"M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z\"/>"));
    add("star", stroke("<polygon points=\"12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2\"/>"));
    add("send", stroke("<line x1=\"22\" y1=\"2\" x2=\"11\" y2=\"13\"/><polygon points=\"22 2 15 22 11 13 2 9 22 2\"/>"));
    add("file", stroke("<path d=\"M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z\"/><path d=\"M14 2v6h6\"/>"));
    add("trash", stroke("<path d=\"M3 6h18\"/><path d=\"M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6\"/><path d=\"M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2\"/>"));
    add("folder", stroke("<path d=\"M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z\"/>"));
    add("folder-plus", stroke("<path d=\"M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z\"/><line x1=\"12\" y1=\"11\" x2=\"12\" y2=\"17\"/><line x1=\"9\" y1=\"14\" x2=\"15\" y2=\"14\"/>"));
    add("search", stroke("<circle cx=\"11\" cy=\"11\" r=\"8\"/><line x1=\"21\" y1=\"21\" x2=\"16.65\" y2=\"16.65\"/>"));
    add("settings", stroke("<line x1=\"4\" y1=\"21\" x2=\"4\" y2=\"14\"/><line x1=\"4\" y1=\"10\" x2=\"4\" y2=\"3\"/><line x1=\"12\" y1=\"21\" x2=\"12\" y2=\"12\"/><line x1=\"12\" y1=\"8\" x2=\"12\" y2=\"3\"/><line x1=\"20\" y1=\"21\" x2=\"20\" y2=\"16\"/><line x1=\"20\" y1=\"12\" x2=\"20\" y2=\"3\"/><line x1=\"1\" y1=\"14\" x2=\"7\" y2=\"14\"/><line x1=\"9\" y1=\"8\" x2=\"15\" y2=\"8\"/><line x1=\"17\" y1=\"16\" x2=\"23\" y2=\"16\"/>"));
    add("plus", stroke("<line x1=\"12\" y1=\"5\" x2=\"12\" y2=\"19\"/><line x1=\"5\" y1=\"12\" x2=\"19\" y2=\"12\"/>"));
    add("minus", stroke("<line x1=\"5\" y1=\"12\" x2=\"19\" y2=\"12\"/>"));
    add("x", stroke("<line x1=\"18\" y1=\"6\" x2=\"6\" y2=\"18\"/><line x1=\"6\" y1=\"6\" x2=\"18\" y2=\"18\"/>"));
    add("chevron-down", stroke("<polyline points=\"6 9 12 15 18 9\"/>"));
    add("chevron-right", stroke("<polyline points=\"9 18 15 12 9 6\"/>"));
    add("chevron-left", stroke("<polyline points=\"15 18 9 12 15 6\"/>"));
    add("chevron-up", stroke("<polyline points=\"18 15 12 9 6 15\"/>"));
    add("check", stroke("<polyline points=\"20 6 9 17 4 12\"/>"));
    add("check-circle", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><polyline points=\"16 9 11 15 8 12\"/>"));
    add("clock", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><polyline points=\"12 6 12 12 16 14\"/>"));
    add("bot", stroke("<rect x=\"4\" y=\"8\" width=\"16\" height=\"12\" rx=\"2\"/><path d=\"M12 8V5\"/><circle cx=\"12\" cy=\"4\" r=\"1\"/><circle cx=\"9\" cy=\"14\" r=\"1\"/><circle cx=\"15\" cy=\"14\" r=\"1\"/>"));
    add("user", stroke("<path d=\"M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2\"/><circle cx=\"12\" cy=\"7\" r=\"4\"/>"));
    add("users", stroke("<path d=\"M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2\"/><circle cx=\"9\" cy=\"7\" r=\"4\"/><path d=\"M23 21v-2a4 4 0 0 0-3-3.87\"/><path d=\"M16 3.13a4 4 0 0 1 0 7.75\"/>"));
    add("wrench", stroke("<path d=\"M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z\"/>"));
    add("sun", stroke("<circle cx=\"12\" cy=\"12\" r=\"5\"/><line x1=\"12\" y1=\"1\" x2=\"12\" y2=\"3\"/><line x1=\"12\" y1=\"21\" x2=\"12\" y2=\"23\"/><line x1=\"4.22\" y1=\"4.22\" x2=\"5.64\" y2=\"5.64\"/><line x1=\"18.36\" y1=\"18.36\" x2=\"19.78\" y2=\"19.78\"/><line x1=\"1\" y1=\"12\" x2=\"3\" y2=\"12\"/><line x1=\"21\" y1=\"12\" x2=\"23\" y2=\"12\"/><line x1=\"4.22\" y1=\"19.78\" x2=\"5.64\" y2=\"18.36\"/><line x1=\"18.36\" y1=\"5.64\" x2=\"19.78\" y2=\"4.22\"/>"));
    add("moon", stroke("<path d=\"M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z\"/>"));
    add("copy", stroke("<rect x=\"9\" y=\"9\" width=\"13\" height=\"13\" rx=\"2\"/><path d=\"M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1\"/>"));
    add("download", stroke("<path d=\"M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4\"/><polyline points=\"7 10 12 15 17 10\"/><line x1=\"12\" y1=\"15\" x2=\"12\" y2=\"3\"/>"));
    add("upload", stroke("<path d=\"M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4\"/><polyline points=\"17 8 12 3 7 8\"/><line x1=\"12\" y1=\"3\" x2=\"12\" y2=\"15\"/>"));
    add("play", filled("<polygon points=\"6 3 21 12 6 21 6 3\"/>"));
    add("pause", filled("<rect x=\"6\" y=\"4\" width=\"4\" height=\"16\" rx=\"1\"/><rect x=\"14\" y=\"4\" width=\"4\" height=\"16\" rx=\"1\"/>"));
    add("stop", filled("<rect x=\"5\" y=\"5\" width=\"14\" height=\"14\" rx=\"2\"/>"));
    add("undo", stroke("<polyline points=\"1 4 1 10 7 10\"/><path d=\"M3.51 15a9 9 0 1 0 2.13-9.36L1 10\"/>"));
    add("refresh", stroke("<polyline points=\"23 4 23 10 17 10\"/><path d=\"M20.49 15a9 9 0 1 1-2.12-9.36L23 10\"/>"));
    add("trophy", stroke("<path d=\"M6 4h12v6a6 6 0 0 1-12 0z\"/><path d=\"M6 5H3v2a3 3 0 0 0 3 3\"/><path d=\"M18 5h3v2a3 3 0 0 1-3 3\"/><path d=\"M12 16v4\"/><path d=\"M8 20h8\"/>"));
    add("list", stroke("<line x1=\"8\" y1=\"6\" x2=\"21\" y2=\"6\"/><line x1=\"8\" y1=\"12\" x2=\"21\" y2=\"12\"/><line x1=\"8\" y1=\"18\" x2=\"21\" y2=\"18\"/><line x1=\"3\" y1=\"6\" x2=\"3.01\" y2=\"6\"/><line x1=\"3\" y1=\"12\" x2=\"3.01\" y2=\"12\"/><line x1=\"3\" y1=\"18\" x2=\"3.01\" y2=\"18\"/>"));
    add("hash", stroke("<line x1=\"4\" y1=\"9\" x2=\"20\" y2=\"9\"/><line x1=\"4\" y1=\"15\" x2=\"20\" y2=\"15\"/><line x1=\"10\" y1=\"3\" x2=\"8\" y2=\"21\"/><line x1=\"16\" y1=\"3\" x2=\"14\" y2=\"21\"/>"));
    add("calendar", stroke("<rect x=\"3\" y=\"4\" width=\"18\" height=\"18\" rx=\"2\"/><line x1=\"16\" y1=\"2\" x2=\"16\" y2=\"6\"/><line x1=\"8\" y1=\"2\" x2=\"8\" y2=\"6\"/><line x1=\"3\" y1=\"10\" x2=\"21\" y2=\"10\"/>"));
    add("zap", filled("<polygon points=\"13 2 3 14 12 14 11 22 21 10 12 10 13 2\"/>"));
    add("alert", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><line x1=\"12\" y1=\"8\" x2=\"12\" y2=\"12\"/><line x1=\"12\" y1=\"16\" x2=\"12.01\" y2=\"16\"/>"));
    add("info", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><line x1=\"12\" y1=\"16\" x2=\"12\" y2=\"12\"/><line x1=\"12\" y1=\"8\" x2=\"12.01\" y2=\"8\"/>"));
    add("eye", stroke("<path d=\"M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>"));
    add("arrow-right", stroke("<line x1=\"5\" y1=\"12\" x2=\"19\" y2=\"12\"/><polyline points=\"12 5 19 12 12 19\"/>"));
    add("arrow-left", stroke("<line x1=\"19\" y1=\"12\" x2=\"5\" y2=\"12\"/><polyline points=\"12 19 5 12 12 5\"/>"));
    add("arrow-up", stroke("<line x1=\"12\" y1=\"19\" x2=\"12\" y2=\"5\"/><polyline points=\"5 12 12 5 19 12\"/>"));
    add("arrow-down", stroke("<line x1=\"12\" y1=\"5\" x2=\"12\" y2=\"19\"/><polyline points=\"19 12 12 19 5 12\"/>"));
    add("more", filled("<circle cx=\"5\" cy=\"12\" r=\"1.6\"/><circle cx=\"12\" cy=\"12\" r=\"1.6\"/><circle cx=\"19\" cy=\"12\" r=\"1.6\"/>"));
    add("terminal", stroke("<polyline points=\"4 17 10 11 4 5\"/><line x1=\"12\" y1=\"19\" x2=\"20\" y2=\"19\"/>"));
    add("code", stroke("<polyline points=\"16 18 22 12 16 6\"/><polyline points=\"8 6 2 12 8 18\"/>"));
    add("book", stroke("<path d=\"M4 19.5A2.5 2.5 0 0 1 6.5 17H20\"/><path d=\"M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z\"/>"));
    add("keyboard", stroke("<rect x=\"2\" y=\"6\" width=\"20\" height=\"12\" rx=\"2\"/><line x1=\"6\" y1=\"10\" x2=\"6.01\" y2=\"10\"/><line x1=\"10\" y1=\"10\" x2=\"10.01\" y2=\"10\"/><line x1=\"14\" y1=\"10\" x2=\"14.01\" y2=\"10\"/><line x1=\"18\" y1=\"10\" x2=\"18.01\" y2=\"10\"/><line x1=\"8\" y1=\"14\" x2=\"16\" y2=\"14\"/>"));
    add("palette", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><circle cx=\"8.5\" cy=\"10.5\" r=\"1\"/><circle cx=\"12\" cy=\"7.5\" r=\"1\"/><circle cx=\"15.5\" cy=\"10.5\" r=\"1\"/><path d=\"M12 22a10 10 0 0 0 10-10\"/><path d=\"M12 22a10 10 0 0 1 0-20\"/>"));
    add("globe", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><line x1=\"2\" y1=\"12\" x2=\"22\" y2=\"12\"/><path d=\"M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z\"/>"));
    add("shield", stroke("<path d=\"M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z\"/>"));
    add("help", stroke("<circle cx=\"12\" cy=\"12\" r=\"10\"/><path d=\"M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3\"/><line x1=\"12\" y1=\"17\" x2=\"12.01\" y2=\"17\"/>"));
    add("archive", stroke("<polyline points=\"21 8 21 21 3 21 3 8\"/><rect x=\"1\" y=\"3\" width=\"22\" height=\"5\"/><line x1=\"10\" y1=\"12\" x2=\"14\" y2=\"12\"/>"));
    add("edit", stroke("<path d=\"M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7\"/><path d=\"M18.5 2.5a2.12 2.12 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z\"/>"));
    add("reply", stroke("<polyline points=\"9 17 4 12 9 7\"/><path d=\"M20 18v-2a4 4 0 0 0-4-4H4\"/>"));
    add("forward", stroke("<polyline points=\"15 17 20 12 15 7\"/><path d=\"M4 18v-2a4 4 0 0 1 4-4h12\"/>"));
    add("dice", stroke("<rect x=\"3\" y=\"3\" width=\"18\" height=\"18\" rx=\"2\"/><circle cx=\"8.5\" cy=\"8.5\" r=\"1\"/><circle cx=\"15.5\" cy=\"8.5\" r=\"1\"/><circle cx=\"12\" cy=\"12\" r=\"1\"/><circle cx=\"8.5\" cy=\"15.5\" r=\"1\"/><circle cx=\"15.5\" cy=\"15.5\" r=\"1\"/>"));
    add("grid", stroke("<rect x=\"3\" y=\"3\" width=\"7\" height=\"7\"/><rect x=\"14\" y=\"3\" width=\"7\" height=\"7\"/><rect x=\"14\" y=\"14\" width=\"7\" height=\"7\"/><rect x=\"3\" y=\"14\" width=\"7\" height=\"7\"/>"));
    add("box", stroke("<path d=\"M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z\"/><polyline points=\"3.27 6.96 12 12.01 20.73 6.96\"/><line x1=\"12\" y1=\"22.08\" x2=\"12\" y2=\"12\"/>"));
    m
}

/// 包装 Arc<dyn AssetSource> 以便 `Application::new().with_assets(...)`
pub struct ArcAssetSource(pub Arc<dyn AssetSource>);

impl AssetSource for ArcAssetSource {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>, anyhow::Error> {
        self.0.load(path)
    }
    fn list(&self, path: &str) -> Result<Vec<SharedString>, anyhow::Error> {
        self.0.list(path)
    }
}

/// 供 `Application::new().with_assets(...)` 使用
pub fn asset_source() -> ArcAssetSource {
    ArcAssetSource(Arc::new(KitAssetSource {
        icons: icon_map(),
    }))
}

/// 图标组件
#[derive(IntoElement)]
pub struct Icon {
    name: SharedString,
    size: gpui::Pixels,
    color: Option<gpui::Hsla>,
    opacity: f32,
}

impl Icon {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            size: gpui::px(16.),
            color: None,
            opacity: 1.0,
        }
    }

    pub fn size(mut self, size: gpui::Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn text_color(mut self, color: gpui::Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        // 未显式指定颜色时,默认继承主题 fg(实测 svg 不沿 text 样式栈继承父色)
        let default_color = cx.global::<crate::theme::Theme>().fg_muted;
        svg()
            .path(format!("icons/{}.svg", self.name))
            .size(self.size)
            .flex_shrink_0()
            .map(|el| match self.color {
                Some(c) => el.text_color(c),
                None => el.text_color(default_color),
            })
            .map(|el| {
                if self.opacity < 1.0 {
                    el.opacity(self.opacity)
                } else {
                    el
                }
            })
    }
}

/// 便捷函数
pub fn icon(name: impl Into<SharedString>) -> Icon {
    Icon::new(name)
}
