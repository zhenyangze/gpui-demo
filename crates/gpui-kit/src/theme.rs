//! # gpui-kit 主题系统
//!
//! 设计 token:色板、字体、字号、圆角。`Theme` 是一个 GPUI Global,
//! 通过 [`init_kit`] 安装,任意组件内部用 [`Theme::global`] 读取。

use gpui::{px, App, Global, Hsla, Pixels, SharedString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    Blue,
    Purple,
    Green,
    Orange,
    Pink,
    Teal,
}

impl Accent {
    pub fn base(self) -> Hsla {
        match self {
            Accent::Blue => hsla(0.58, 0.72, 0.58, 1.0),
            Accent::Purple => hsla(0.72, 0.60, 0.62, 1.0),
            Accent::Green => hsla(0.40, 0.55, 0.50, 1.0),
            Accent::Orange => hsla(0.07, 0.80, 0.58, 1.0),
            Accent::Pink => hsla(0.93, 0.65, 0.62, 1.0),
            Accent::Teal => hsla(0.48, 0.60, 0.48, 1.0),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Accent::Blue => "蓝",
            Accent::Purple => "紫",
            Accent::Green => "绿",
            Accent::Orange => "橙",
            Accent::Pink => "粉",
            Accent::Teal => "青",
        }
    }
    pub const ALL: [Accent; 6] = [
        Accent::Blue,
        Accent::Purple,
        Accent::Green,
        Accent::Orange,
        Accent::Pink,
        Accent::Teal,
    ];
}

/// hsl 构造辅助(0-1 区间)
pub fn hsla(h: f64, s: f64, l: f64, a: f64) -> Hsla {
    gpui::hsla(h as f32, s as f32, l as f32, a as f32)
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub accent: Accent,
    pub font_ui: SharedString,
    pub font_mono: SharedString,
    pub font_size_base: Pixels,
    // 表面
    pub bg: Hsla,
    pub bg_panel: Hsla,
    pub bg_elevated: Hsla,
    pub bg_hover: Hsla,
    pub bg_active: Hsla,
    pub bg_input: Hsla,
    // 前景
    pub fg: Hsla,
    pub fg_muted: Hsla,
    pub fg_subtle: Hsla,
    // 边框
    pub border: Hsla,
    pub border_strong: Hsla,
    // 语义色
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_fg: Hsla,
    pub danger: Hsla,
    pub danger_hover: Hsla,
    pub warning: Hsla,
    pub success: Hsla,
    pub info: Hsla,
    // 圆角
    pub radius_sm: Pixels,
    pub radius_md: Pixels,
    pub radius_lg: Pixels,
}

impl Global for Theme {}

impl Theme {
    pub fn dark() -> Self {
        let accent = Accent::Blue;
        Self {
            mode: ThemeMode::Dark,
            accent,
            font_ui: font_ui_default().into(),
            font_mono: font_mono_default().into(),
            font_size_base: px(14.),
            bg: hsla(0., 0., 0.10, 1.),
            bg_panel: hsla(0., 0., 0.14, 1.),
            bg_elevated: hsla(0., 0., 0.18, 1.),
            bg_hover: hsla(0., 0., 1., 0.06),
            bg_active: hsla(0., 0., 1., 0.10),
            bg_input: hsla(0., 0., 0.08, 1.),
            fg: hsla(0., 0., 0.92, 1.),
            fg_muted: hsla(0., 0., 0.65, 1.),
            fg_subtle: hsla(0., 0., 0.45, 1.),
            border: hsla(0., 0., 1., 0.09),
            border_strong: hsla(0., 0., 1., 0.18),
            primary: accent.base(),
            primary_hover: hsla(0.58, 0.72, 0.66, 1.),
            primary_fg: hsla(0., 0., 1., 1.),
            danger: hsla(0.01, 0.72, 0.58, 1.),
            danger_hover: hsla(0.01, 0.72, 0.66, 1.),
            warning: hsla(0.10, 0.80, 0.58, 1.),
            success: hsla(0.40, 0.58, 0.50, 1.),
            info: hsla(0.55, 0.62, 0.58, 1.),
            radius_sm: px(4.),
            radius_md: px(8.),
            radius_lg: px(12.),
        }
    }

    pub fn light() -> Self {
        let accent = Accent::Blue;
        Self {
            mode: ThemeMode::Light,
            accent,
            font_ui: font_ui_default().into(),
            font_mono: font_mono_default().into(),
            font_size_base: px(14.),
            bg: hsla(0., 0., 0.97, 1.),
            bg_panel: hsla(0., 0., 1., 1.),
            bg_elevated: hsla(0., 0., 1., 1.),
            bg_hover: hsla(0., 0., 0., 0.05),
            bg_active: hsla(0., 0., 0., 0.08),
            bg_input: hsla(0., 0., 1., 1.),
            fg: hsla(0., 0., 0.13, 1.),
            fg_muted: hsla(0., 0., 0.40, 1.),
            fg_subtle: hsla(0., 0., 0.60, 1.),
            border: hsla(0., 0., 0., 0.10),
            border_strong: hsla(0., 0., 0., 0.20),
            primary: accent.base(),
            primary_hover: hsla(0.58, 0.72, 0.50, 1.),
            primary_fg: hsla(0., 0., 1., 1.),
            danger: hsla(0.01, 0.68, 0.50, 1.),
            danger_hover: hsla(0.01, 0.68, 0.42, 1.),
            warning: hsla(0.10, 0.85, 0.45, 1.),
            success: hsla(0.40, 0.55, 0.40, 1.),
            info: hsla(0.55, 0.60, 0.48, 1.),
            radius_sm: px(4.),
            radius_md: px(8.),
            radius_lg: px(12.),
        }
    }

    /// 主题的变体切换
    pub fn with_mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        let (accent, base, font) = (self.accent, self.mode, (self.font_ui.clone(), self.font_mono.clone(), self.font_size_base));
        let mut next = match base {
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Light => Self::light(),
        };
        next.accent = accent;
        next.set_accent(accent);
        next.font_ui = font.0;
        next.font_mono = font.1;
        next.font_size_base = font.2;
        next
    }

    pub fn set_accent(&mut self, accent: Accent) {
        self.accent = accent;
        self.primary = accent.base();
        // hover 色随 accent 亮度微调
        let base = accent.base();
        let lift = match self.mode {
            ThemeMode::Dark => 0.08,
            ThemeMode::Light => -0.08,
        };
        self.primary_hover = hsla(base.h as f64, base.s as f64, (base.l as f64 + lift).clamp(0., 1.), 1.);
    }

    pub fn is_dark(&self) -> bool {
        self.mode == ThemeMode::Dark
    }

    pub fn global(cx: &App) -> &Theme {
        cx.global::<Theme>()
    }

    /// 切换主题(全局替换,所有 observe_global 的视图会重绘)
    pub fn set_global(theme: Theme, cx: &mut App) {
        cx.set_global(theme);
    }

    pub fn toggle_mode(cx: &mut App) {
        let next = {
            let t = cx.global::<Theme>().clone();
            let mode = match t.mode {
                ThemeMode::Dark => ThemeMode::Light,
                ThemeMode::Light => ThemeMode::Dark,
            };
            t.with_mode(mode)
        };
        cx.set_global(next);
    }
}

#[cfg(target_os = "macos")]
pub fn font_ui_default() -> &'static str {
    ".SystemUIFont"
}

#[cfg(target_os = "macos")]
pub fn font_mono_default() -> &'static str {
    "Menlo"
}

#[cfg(not(target_os = "macos"))]
pub fn font_ui_default() -> &'static str {
    "Segoe UI, Roboto, sans-serif"
}

#[cfg(not(target_os = "macos"))]
pub fn font_mono_default() -> &'static str {
    "Consolas, monospace"
}

/// 间距刻度(4px 基准)
pub const fn spacing(n: f32) -> Pixels {
    px(4. * n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_mode_preserves_accent_and_fonts() {
        let mut t = Theme::dark();
        t.set_accent(Accent::Pink);
        t.font_size_base = px(16.);
        let t2 = t.with_mode(ThemeMode::Light);
        assert_eq!(t2.mode, ThemeMode::Light);
        assert_eq!(t2.accent, Accent::Pink);
        assert_eq!(t2.font_size_base, px(16.));
        assert!(t2.is_dark() == false);
    }

    #[test]
    fn test_toggle_preserves_accent() {
        let mut t = Theme::light();
        t.set_accent(Accent::Teal);
        let t2 = t.with_mode(ThemeMode::Dark);
        assert_eq!(t2.accent, Accent::Teal);
    }

    #[test]
    fn test_spacing_scale() {
        assert_eq!(spacing(1.0), px(4.));
        assert_eq!(spacing(6.0), px(24.));
    }
}
