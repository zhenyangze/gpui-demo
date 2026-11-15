//! 表单控件:Switch / Checkbox / Radio / Slider(自绘拖拽)。

use std::rc::Rc;

use gpui::{
    canvas, div, prelude::FluentBuilder, px, relative, App, ClickEvent, ElementId, IntoElement,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Pixels, RenderOnce, SharedString,
    Styled, Window, prelude::*,
};

use crate::theme::Theme;

type ChangeCb = Box<dyn Fn(&mut Window, &mut App) + 'static>;

// ------------------------- Switch -------------------------

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    checked: bool,
    disabled: bool,
    on_change: Option<ChangeCb>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            disabled: false,
            on_change: None,
        }
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn on_change(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let (track, knob) = if self.checked {
            (theme.primary, crate::theme::hsla(0., 0., 1., 1.))
        } else {
            (theme.border_strong, theme.bg_elevated)
        };
        div()
            .id(self.id)
            .relative()
            .w(px(36.))
            .h(px(20.))
            .rounded_full()
            .bg(track)
            .when(self.disabled, |el| el.opacity(0.4))
            .when_some(self.on_change.filter(|_| !self.disabled), |el, cb| {
                el.cursor(gpui::CursorStyle::PointingHand)
                    .on_click(move |_: &ClickEvent, window, cx| cb(window, cx))
            })
            .child(
                // 旋钮:选中时右移 16px
                div()
                    .absolute()
                    .top(px(2.))
                    .left(px(2. + if self.checked { 16. } else { 0. }))
                    .size(px(16.))
                    .rounded_full()
                    .bg(knob)
                    .shadow_sm(),
            )
    }
}

// ------------------------- Checkbox -------------------------

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    label: Option<SharedString>,
    indeterminate: bool,
    disabled: bool,
    on_change: Option<ChangeCb>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            label: None,
            indeterminate: false,
            disabled: false,
            on_change: None,
        }
    }
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn indeterminate(mut self, v: bool) -> Self {
        self.indeterminate = v;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn on_change(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let active = self.checked || self.indeterminate;
        let bg = if active { theme.primary } else { theme.bg_input };
        let border = if active { theme.primary } else { theme.border_strong };
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .when(self.disabled, |el| el.opacity(0.4))
            .when_some(self.on_change.filter(|_| !self.disabled), |el, cb| {
                el.cursor(gpui::CursorStyle::PointingHand)
                    .on_click(move |_: &ClickEvent, window, cx| cb(window, cx))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(16.))
                    .rounded(px(4.))
                    .bg(bg)
                    .border_1()
                    .border_color(border)
                    .child(if self.indeterminate {
                        crate::icons::Icon::new("minus").size(px(10.)).text_color(theme.primary_fg)
                    } else if self.checked {
                        crate::icons::Icon::new("check").size(px(10.)).text_color(theme.primary_fg)
                    } else {
                        crate::icons::Icon::new("minus").size(px(10.)).text_color(crate::theme::hsla(0., 0., 0., 0.))
                    }),
            )
            .children(self.label.map(|l| {
                div()
                    .text_size(px(13.))
                    .text_color(theme.fg_muted)
                    .child(l)
            }))
    }
}

// ------------------------- Radio -------------------------

#[derive(IntoElement)]
pub struct Radio {
    id: ElementId,
    selected: bool,
    label: Option<SharedString>,
    on_select: Option<ChangeCb>,
}

impl Radio {
    pub fn new(id: impl Into<ElementId>, selected: bool) -> Self {
        Self {
            id: id.into(),
            selected,
            label: None,
            on_select: None,
        }
    }
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn on_select(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Radio {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let (ring, dot) = if self.selected {
            (theme.primary, theme.primary)
        } else {
            (theme.border_strong, crate::theme::hsla(0., 0., 0., 0.))
        };
        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap_2()
            .when_some(self.on_select, |el, cb| {
                el.cursor(gpui::CursorStyle::PointingHand)
                    .on_click(move |_: &ClickEvent, window, cx| cb(window, cx))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(16.))
                    .rounded_full()
                    .border_2()
                    .border_color(ring)
                    .child(div().size(px(8.)).rounded_full().bg(dot)),
            )
            .children(self.label.map(|l| {
                div().text_size(px(13.)).text_color(theme.fg_muted).child(l)
            }))
    }
}

// ------------------------- Slider -------------------------

type SliderCb = Rc<dyn Fn(f64, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    on_change: Option<SliderCb>,
}

impl Slider {
    pub fn new(id: impl Into<ElementId>, value: f64) -> Self {
        Self {
            id: id.into(),
            value,
            min: 0.,
            max: 1.,
            step: 0.01,
            on_change: None,
        }
    }
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, s: f64) -> Self {
        self.step = s;
        self
    }
    pub fn on_change(mut self, f: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let pct = if (self.max - self.min).abs() < f64::EPSILON {
            0.
        } else {
            ((self.value - self.min) / (self.max - self.min)).clamp(0., 1.)
        };
        let Some(on_change) = self.on_change.clone() else {
            return div().w_full().h(px(20.)).into_any_element();
        };

        // 通过 canvas 拿到布局 bounds,再用 window.on_mouse_event 注册拖拽
        let drag_state: Rc<std::cell::Cell<Option<f64>>> = Rc::new(std::cell::Cell::new(None));
        let min = self.min;
        let max = self.max;
        let step = self.step;
        let id = self.id.clone();

        div()
            .id(id.clone())
            .relative()
            .w_full()
            .h(px(20.))
            .child(
                div()
                    .absolute()
                    .top(px(8.))
                    .left_0()
                    .right_0()
                    .h(px(4.))
                    .rounded_full()
                    .bg(theme.bg_hover)
                    .child(
                        div()
                            .w(relative(pct as f32))
                            .h_full()
                            .rounded_full()
                            .bg(theme.primary),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(5.))
                    .left(relative(pct as f32))
                    .ml(px(-7.))
                    .size(px(14.))
                    .rounded_full()
                    .bg(theme.bg_elevated)
                    .border_2()
                    .border_color(theme.primary)
                    .shadow_sm(),
            )
            .child(
                canvas(
                    move |_, _, _| (),
                    move |bounds, _, window, _| {
                        {
                            let drag_state = drag_state.clone();
                            let on_change = on_change.clone();
                            window.on_mouse_event(move |ev: &MouseDownEvent, _, window, cx| {
                                if bounds.contains(&ev.position) {
                                    let value = position_to_value(
                                        ev.position.x - bounds.left(),
                                        bounds.size.width,
                                        min,
                                        max,
                                    );
                                    drag_state.set(Some(value));
                                    on_change(value, window, cx);
                                }
                            });
                        }
                        {
                            let drag_state = drag_state.clone();
                            let on_change = on_change.clone();
                            window.on_mouse_event(move |ev: &MouseMoveEvent, _, window, cx| {
                                if ev.dragging() && drag_state.get().is_some() {
                                    let mut value = position_to_value(
                                        ev.position.x - bounds.left(),
                                        bounds.size.width,
                                        min,
                                        max,
                                    );
                                    value = (value / step).round() * step;
                                    on_change(value, window, cx);
                                }
                            });
                        }
                        {
                            let drag_state = drag_state.clone();
                            window.on_mouse_event(move |_: &MouseUpEvent, _, _, _| {
                                drag_state.set(None);
                            });
                        }
                    },
                )
                .absolute()
                .inset_0(),
            )
            .cursor(gpui::CursorStyle::PointingHand)
            .into_any_element()
    }
}

fn position_to_value(x: Pixels, width: Pixels, min: f64, max: f64) -> f64 {
    let ratio = ((x / width.max(gpui::px(1.))) as f64).clamp(0., 1.);
    min + ratio * (max - min)
}
