//! # perf — 渲染性能观测(L1)
//!
//! RenderCounter 记录每个实体的渲染次数;`perf_overlay` 用于开发期可视化,
//! 以及虚拟化正确性测试(断言长列表只渲染可视行)。

use std::collections::HashMap;

use gpui::{div, prelude::*, px, App, Context, Global, IntoElement, Render, Window};

#[derive(Default)]
pub struct RenderCounter {
    counts: HashMap<String, u64>,
    enabled: bool,
}

impl Global for RenderCounter {}

impl RenderCounter {
    pub fn bump(cx: &mut App, key: impl Into<String>) {
        let c = cx.global_mut::<RenderCounter>();
        *c.counts.entry(key.into()).or_insert(0) += 1;
    }

    pub fn get(cx: &App, key: &str) -> u64 {
        cx.try_global::<RenderCounter>()
            .map(|c| c.counts.get(key).copied().unwrap_or(0))
            .unwrap_or(0)
    }

    pub fn set_enabled(cx: &mut App, enabled: bool) {
        cx.global_mut::<RenderCounter>().enabled = enabled;
    }

    pub fn is_enabled(cx: &App) -> bool {
        cx.try_global::<RenderCounter>().map(|c| c.enabled).unwrap_or(false)
    }

    pub fn reset(cx: &mut App) {
        cx.global_mut::<RenderCounter>().counts.clear();
    }

    pub fn snapshot(cx: &App) -> Vec<(String, u64)> {
        let mut v: Vec<(String, u64)> = cx
            .try_global::<RenderCounter>()
            .map(|c| c.counts.iter().map(|(k, n)| (k.clone(), *n)).collect())
            .unwrap_or_default();
        v.sort_by(|a, b| b.1.cmp(&a.1));
        v
    }
}

/// 开发期右上角渲染计数浮层
pub struct PerfOverlay;

impl Render for PerfOverlay {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !RenderCounter::is_enabled(cx) {
            return div().into_any_element();
        }
        let theme = cx.global::<crate::theme::Theme>();
        let rows = RenderCounter::snapshot(cx)
            .into_iter()
            .take(6)
            .map(|(k, n)| {
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(10.))
                    .font_family(theme.font_mono.clone())
                    .child(k)
                    .child(format!("{}", n))
            })
            .collect::<Vec<_>>();
        div()
            .absolute()
            .top(px(4.))
            .right(px(4.))
            .p_2()
            .rounded(theme.radius_sm)
            .bg(crate::theme::hsla(0., 0., 0., 0.75))
            .text_color(crate::theme::hsla(0., 0., 0.9, 1.))
            .flex()
            .flex_col()
            .gap(px(1.))
            .children(rows)
            .into_any_element()
    }
}

/// init:注册 RenderCounter Global
pub fn init(cx: &mut App) {
    cx.set_global(RenderCounter::default());
}
