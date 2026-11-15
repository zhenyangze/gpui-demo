//! 2048 UI:棋盘绘制、动画、顶栏、覆盖层、排行榜。

use gpui::{px, prelude::*, AnimationExt as _, App, AppContext, Context, Entity, FocusHandle, Focusable, Hsla, Render, Window};
use gpui_kit::prelude::*;

use gpui_kit::core::history::History;

use crate::board::{self, Dir, Grid};
use crate::state::{self, Persisted, ScoreRecord};
use crate::{Help, MoveDown, MoveLeft, MoveRight, MoveUp, Pause, Quit, Restart, SHORTCUTS, Undo};

pub struct GameApp {
    focus: FocusHandle,
    grid: Grid,
    score: u32,
    best: u32,
    steps: u32,
    merge_ops: u32,
    peak: u32,
    seconds: u64,
    history: History<(Grid, u32)>,
    records: Vec<ScoreRecord>,
    night: bool,
    paused: bool,
    over: bool,
    won: bool,
    keep_playing: bool,
    new_tile: Option<(usize, usize)>,
    merged_tiles: Vec<(usize, usize)>,
    shake: bool,
    help_open: bool,
    board_open: bool,
    tick_task: Option<gpui::Task<()>>,
}

impl GameApp {
    pub fn new(cx: &mut Context<Self>, arg: Option<String>) -> Self {
        let saved = state::load();
        let (grid, score, steps, merge_ops, peak, seconds, best, records, night) = match saved {
            Some(p) => (p.grid, p.score, p.steps, p.merge_ops, p.peak, p.seconds, p.best, p.records, p.night_theme),
            None => (board::initial(), 0, 0, 0, 0, 0, 0, vec![], false),
        };

        let mut theme = if night { Theme::dark() } else { Theme::light() };
        theme.set_accent(Accent::Orange);
        cx.set_global(theme);

        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();

        // 每秒计时(游戏未暂停且未结束时)
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_secs(1))
                    .await;
                let ok = this.update(cx, |this, cx| {
                    if !this.paused && !this.over {
                        this.seconds += 1;
                        cx.notify();
                    }
                });
                if ok.is_err() {
                    break;
                }
            }
        });

        let mut over = false;
        if !board::any_move_possible(&grid) {
            over = true;
        }

        let mut app = Self {
            focus: cx.focus_handle(),
            grid,
            score,
            best,
            steps,
            merge_ops,
            peak,
            seconds,
            history: History::new(20),
            records,
            night,
            paused: false,
            over,
            won: false,
            keep_playing: false,
            new_tile: None,
            merged_tiles: vec![],
            shake: false,
            help_open: false,
            board_open: false,
            tick_task: Some(tick),
        };

        if arg.as_deref() == Some("over") {
            // 演示 Game Over:构造死局
            app.grid = [[2, 4, 8, 16], [32, 64, 128, 256], [2, 4, 8, 16], [32, 64, 128, 256]];
            app.score = 12340;
            app.over = true;
        }
        if arg.as_deref() == Some("win") {
            app.grid = [[1024, 512, 256, 128], [64, 32, 16, 8], [4, 2, 2, 0], [0, 0, 0, 0]];
            app.score = 88916;
            app.won = true;
        }
        app
    }

    fn persist(&self) {
        state::save(&Persisted {
            grid: self.grid,
            score: self.score,
            steps: self.steps,
            merge_ops: self.merge_ops,
            peak: self.peak,
            seconds: self.seconds,
            best: self.best,
            records: self.records.clone(),
            night_theme: self.night,
        });
    }

    fn move_dir(&mut self, dir: Dir, cx: &mut Context<Self>) {
        if self.paused || (self.over && !self.keep_playing) {
            return;
        }
        let result = board::slide(&self.grid, dir);
        if !result.moved {
            // 无效移动:抖动
            self.shake = true;
            cx.notify();
            let this = cx.weak_entity();
            cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(220))
                    .await;
                let _ = this.update(cx, |this, cx| {
                    this.shake = false;
                    cx.notify();
                });
            })
            .detach();
            return;
        }
        // 历史(通用 History 框架)
        self.history.commit((self.grid, self.score));
        let gained = result.gained;
        self.grid = result.grid;
        self.score += gained;
        self.steps += 1;
        self.merge_ops += result.merged.len() as u32;
        self.merged_tiles = result.merged.clone();
        self.peak = self.peak.max(board::max_tile(&self.grid));
        if result.max_merge >= 2048 && !self.won {
            self.won = true;
        }
        if self.score > self.best {
            self.best = self.score;
        }
        let spawn_pos = board::spawn(&mut self.grid);
        self.new_tile = spawn_pos;
        self.over = !board::any_move_possible(&self.grid);
        if self.over {
            // 记录成绩
            self.records.push(ScoreRecord {
                score: self.score,
                max_tile: board::max_tile(&self.grid),
                seconds: self.seconds,
                time_unix: now_ts(),
            });
            self.records.sort_by(|a, b| b.score.cmp(&a.score));
            self.records.truncate(5);
            self.persist();
            toast(ToastKind::Warning, format!("游戏结束,得分 {}", self.score), cx);
        }
        self.persist();
        cx.notify();
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if self.paused {
            return;
        }
        let mut current = (self.grid, self.score);
        if self.history.undo(&mut current) {
            self.grid = current.0;
            self.score = current.1;
            self.over = false;
            self.won = false;
            self.new_tile = None;
            self.merged_tiles.clear();
            self.persist();
            toast(ToastKind::Info, format!("已撤销(剩 {} 步)", self.history.undo_len()), cx);
            cx.notify();
        }
    }

    fn restart(&mut self, cx: &mut Context<Self>) {
        self.grid = board::initial();
        self.score = 0;
        self.steps = 0;
        self.merge_ops = 0;
        self.peak = 0;
        self.seconds = 0;
        self.history.clear();
        self.over = false;
        self.won = false;
        self.keep_playing = false;
        self.paused = false;
        self.new_tile = None;
        self.merged_tiles.clear();
        self.persist();
        toast(ToastKind::Success, "新对局开始!", cx);
        cx.notify();
    }

    fn reset_all(&mut self, cx: &mut Context<Self>) {
        self.records.clear();
        self.best = 0;
        self.restart(cx);
    }

    fn share_score(&self, cx: &mut Context<Self>) {
        let text = format!(
            "我在 GPUI-2048 得了 {} 分(最高块 {},用时 {} 秒)!",
            self.score, board::max_tile(&self.grid), self.seconds
        );
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
        toast(ToastKind::Success, "成绩已复制到剪贴板", cx);
    }
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn tile_colors(v: u32, night: bool) -> (Hsla, Hsla) {
    // (背景, 文字色)
    let _ = night;
    match v {
        2 => (gpui_kit::theme::hsla(0.10, 0.45, 0.86, 1.), gpui_kit::theme::hsla(0., 0., 0.25, 1.)),
        4 => (gpui_kit::theme::hsla(0.08, 0.55, 0.80, 1.), gpui_kit::theme::hsla(0., 0., 0.25, 1.)),
        8 => (gpui_kit::theme::hsla(0.07, 0.72, 0.72, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        16 => (gpui_kit::theme::hsla(0.05, 0.78, 0.66, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        32 => (gpui_kit::theme::hsla(0.03, 0.80, 0.60, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        64 => (gpui_kit::theme::hsla(0.01, 0.82, 0.55, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        128 => (gpui_kit::theme::hsla(0.12, 0.70, 0.60, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        256 => (gpui_kit::theme::hsla(0.12, 0.72, 0.54, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        512 => (gpui_kit::theme::hsla(0.12, 0.74, 0.48, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        1024 => (gpui_kit::theme::hsla(0.58, 0.65, 0.55, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        2048 => (gpui_kit::theme::hsla(0.50, 0.75, 0.45, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
        _ => (gpui_kit::theme::hsla(0.75, 0.70, 0.35, 1.), gpui_kit::theme::hsla(0., 0., 1., 1.)),
    }
}

impl Render for GameApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>().clone();
        let entity = cx.entity();
        let grid = self.grid;
        let score = self.score;
        let best = self.best;
        let steps = self.steps;
        let merge_ops = self.merge_ops;
        let peak = self.peak;
        let seconds = self.seconds;
        let paused = self.paused;
        let over = self.over && !self.keep_playing;
        let won = self.won;
        let history_len = self.history.undo_len();
        let shake = self.shake;
        let max_tile_v = board::max_tile(&grid);

        // 渲染计数(perf 观测)
        gpui_kit::perf::RenderCounter::bump(cx, "game.board");

        // 棋盘
        let cell = px(96.);
        let gap = px(10.);
        let board_size = cell * 4. + gap * 3.;

        let mut tiles = Vec::new();
        for r in 0..4 {
            for c in 0..4 {
                let v = grid[r][c];
                let is_new = self.new_tile == Some((r, c));
                let is_merged = self.merged_tiles.contains(&(r, c));
                tiles.push(tile_view(v, r, c, cell, gap, is_new, is_merged, &theme, entity.clone()));
            }
        }

        div()
            .id("game-root")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(theme.bg)
            .text_color(theme.fg)
            .font_family(theme.font_ui.clone())
            .key_context("Game")
            .track_focus(&self.focus.clone())
            .on_action(cx.listener(|this: &mut GameApp, _: &MoveUp, _: &mut Window, cx: &mut Context<GameApp>| this.move_dir(Dir::Up, cx)))
            .on_action(cx.listener(|this: &mut GameApp, _: &MoveDown, _: &mut Window, cx: &mut Context<GameApp>| this.move_dir(Dir::Down, cx)))
            .on_action(cx.listener(|this: &mut GameApp, _: &MoveLeft, _: &mut Window, cx: &mut Context<GameApp>| this.move_dir(Dir::Left, cx)))
            .on_action(cx.listener(|this: &mut GameApp, _: &MoveRight, _: &mut Window, cx: &mut Context<GameApp>| this.move_dir(Dir::Right, cx)))
            .on_action(cx.listener(|this: &mut GameApp, _: &Undo, _: &mut Window, cx: &mut Context<GameApp>| this.undo(cx)))
            .on_action(cx.listener(|this: &mut GameApp, _: &Restart, _: &mut Window, cx: &mut Context<GameApp>| this.restart(cx)))
            .on_action(cx.listener(|this: &mut GameApp, _: &Pause, _: &mut Window, cx: &mut Context<GameApp>| {
                this.paused = !this.paused;
                cx.notify();
            }))
            .on_action(cx.listener(|this: &mut GameApp, _: &Help, _: &mut Window, cx: &mut Context<GameApp>| {
                this.help_open = !this.help_open;
                cx.notify();
            }))
            .on_action(|_: &Quit, _, cx| cx.quit())
            // 顶栏
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .flex_shrink_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(28.))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(theme.primary)
                                    .child("2048"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(theme.fg_subtle)
                                    .child(format!("最高块 {}", max_tile_v)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(score_box("分数", &gpui_kit::util::format_thousands(score as i64), &theme))
                            .child(score_box("最佳", &gpui_kit::util::format_thousands(best as i64), &theme)),
                    ),
            )
            // 工具条
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .pb_2()
                    .flex_shrink_0()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new("g-undo", "撤销")
                                    .icon("undo")
                                    .disabled(history_len == 0)
                                    .tooltip(format!("剩余 {} 步", history_len))
                                    .on_click(cx.listener(|this: &mut GameApp, _, _: &mut Window, cx: &mut Context<GameApp>| this.undo(cx))),
                            )
                            .child(
                                Button::new("g-restart", "重开").icon("refresh").on_click(cx.listener(
                                    |this, _, _, cx| this.restart(cx),
                                )),
                            )
                            .child(
                                Button::new("g-pause", if paused { "继续" } else { "暂停" })
                                    .icon(if paused { "play" } else { "pause" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.paused = !this.paused;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new("g-board", "🏆").tooltip("排行榜").on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.board_open = true;
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(
                                Button::new("g-theme", if theme.is_dark() { "经典" } else { "暗夜" })
                                    .icon(if theme.is_dark() { "sun" } else { "moon" })
                                    .on_click(cx.listener(|this: &mut GameApp, _, _: &mut Window, cx: &mut Context<GameApp>| {
                                        this.night = !this.night;
                                        let mode = if this.night { gpui_kit::theme::ThemeMode::Light } else { gpui_kit::theme::ThemeMode::Dark };
                                        cx.update_global::<Theme, _>(|t, _| *t = t.clone().with_mode(mode));
                                        this.persist();
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // 统计条
            .child(
                div()
                    .flex()
                    .justify_center()
                    .gap_4()
                    .pb_2()
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(theme.fg_subtle)
                    .child(format!("步数 {}", steps))
                    .child(format!("合并 {}", merge_ops))
                    .child(format!("峰值 {}", peak))
                    .child(format!("用时 {}s", seconds)),
            )
            // 棋盘
            .child(
                div()
                    .flex()
                    .justify_center()
                    .flex_1()
                    .items_center()
                    .child(shake_wrap(shake, board_size, &theme, tiles)),
            )
            // 提示
            .child(
                div()
                    .flex()
                    .justify_center()
                    .py_3()
                    .flex_shrink_0()
                    .text_size(px(11.))
                    .text_color(theme.fg_subtle)
                    .child("方向键 / WASD / HJKL 移动 · U 撤销 · R 重开 · P 暂停 · ? 帮助"),
            )
            // 覆盖层
            .when(paused, |el| {
                el.child(overlay("已暂停", "按 P 继续游戏", &theme, vec![]))
            })
            .when(over, |el| {
                el.child(overlay(
                    "Game Over",
                    &format!("得分 {} · 最高块 {} · 步数 {} · 合并 {} · 用时 {}s", score, max_tile_v, steps, merge_ops, seconds),
                    &theme,
                    vec![
                        Button::new("o-restart", "再来一局").primary().on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.restart(cx))
                        }),
                        Button::new("o-share", "分享成绩")
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| e.update(cx, |this, cx| this.share_score(cx))
                            }),
                    ],
                ))
            })
            .when(won && !over && !self.keep_playing, |el| {
                el.child(overlay(
                    "🎉 你赢了!",
                    &format!("达成 2048!当前得分 {}", score),
                    &theme,
                    vec![
                        Button::new("o-continue", "继续挑战").primary().on_click({
                            let e = entity.clone();
                            move |_, _, cx| {
                                e.update(cx, |this, cx| {
                                    this.keep_playing = true;
                                    cx.notify();
                                });
                            }
                        }),
                        Button::new("o-newgame", "重新开始").on_click({
                            let e = entity.clone();
                            move |_, _, cx| e.update(cx, |this, cx| this.restart(cx))
                        }),
                    ],
                ))
            })
            // 排行榜
            .when(self.board_open, |el| {
                let records = self.records.clone();
                let best = self.best;
                el.child(board_modal(records, best, entity.clone(), cx))
            })
            // 帮助
            .when(self.help_open, |el| el.child(help_modal(entity.clone(), cx)))
            .children(render_toasts(window, cx))
    }
}

impl Focusable for GameApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

// ---------- 组件 ----------

#[allow(clippy::too_many_arguments)]
fn tile_view(
    v: u32,
    r: usize,
    c: usize,
    cell: Pixels,
    gap: Pixels,
    is_new: bool,
    is_merged: bool,
    theme: &Theme,
    entity: Entity<super::ui::GameApp>,
) -> gpui::AnyElement {
    let _ = entity;
    let (bg, fg) = if v == 0 {
        (gpui::hsla(0., 0., 0., 0.), gpui::hsla(0., 0., 0., 0.))
    } else {
        tile_colors(v, false)
    };
    let font_size = if v >= 1024 {
        px(30.)
    } else if v >= 128 {
        px(36.)
    } else {
        px(40.)
    };
    // inner 容器无 padding:起点 = c*(cell+gap)
    let x = (cell + gap) * c as f32;
    let y = (cell + gap) * r as f32;

    let base = div()
        .absolute()
        .left(x)
        .top(y)
        .size(cell)
        .rounded(px(8.))
        .bg(bg)
        .flex()
        .items_center()
        .justify_center()
        .text_size(font_size)
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(fg)
        .font_family(theme.font_mono.clone())
        .child(if v == 0 { "".to_string() } else { gpui_kit::util::format_thousands(v as i64) });

    if v == 0 {
        return base.into_any_element();
    }
    if is_new {
        let cell = cell;
        base.with_animation(
            ("tile-new", (r * 4 + c) as usize),
            gpui::Animation::new(std::time::Duration::from_millis(160)),
            move |el, delta| el.opacity(0.3 + 0.7 * delta).size(cell * (0.7 + 0.3 * delta)),
        )
        .into_any_element()
    } else if is_merged {
        let cell = cell;
        base.with_animation(
            ("tile-merge", (r * 4 + c) as usize),
            gpui::Animation::new(std::time::Duration::from_millis(150)),
            move |el, delta| {
                // 1.0 → 1.15 → 1.0 脉冲(尺寸脉冲)
                let scale = if delta < 0.5 {
                    1.0 + delta * 0.3
                } else {
                    1.15 - (delta - 0.5) * 0.3
                };
                el.size(cell * scale)
            },
        )
        .into_any_element()
    } else {
        base.into_any_element()
    }
}

/// 棋盘容器(可抖动)
fn shake_wrap(
    shake: bool,
    board_size: Pixels,
    theme: &Theme,
    tiles: Vec<gpui::AnyElement>,
) -> gpui::AnyElement {
    let inner = div()
        .relative()
        .size(board_size)
        .child(
            div()
                .grid()
                .grid_cols(4)
                .gap(px(10.))
                .size_full()
                .children((0..16).map(|_| {
                    div().rounded(px(8.)).bg(theme.bg_hover.opacity(0.7))
                })),
        )
        .children(tiles);
    let gap = px(10.);
    let board = div()
        .p(gap)
        .rounded(px(16.))
        .bg(theme.bg_panel)
        .shadow_lg()
        .child(inner);
    if shake {
        board
            .with_animation(
                "shake",
                gpui::Animation::new(std::time::Duration::from_millis(200)),
                |el, delta| {
                    let off = ((delta * 3.0) as f64).sin() * 4.0;
                    el.left(px(off as f32))
                },
            )
            .into_any_element()
    } else {
        board.into_any_element()
    }
}

fn score_box(label: &str, value: &str, theme: &Theme) -> gpui::AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .px_3()
        .py_1p5()
        .rounded(theme.radius_md)
        .bg(theme.bg_panel)
        .child(
            div()
                .text_size(px(10.))
                .text_color(theme.fg_subtle)
                .child(label.to_string()),
        )
        .child(
            div()
                .text_size(px(16.))
                .font_weight(gpui::FontWeight::BOLD)
                .font_family(theme.font_mono.clone())
                .child(value.to_string()),
        )
        .into_any_element()
}

fn overlay(title: &str, desc: &str, theme: &Theme, buttons: Vec<Button>) -> gpui::AnyElement {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(gpui_kit::theme::hsla(0., 0., 0., 0.55))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                .p_6()
                .rounded(px(16.))
                .bg(theme.bg_elevated)
                .shadow_lg()
                .child(
                    div()
                        .text_size(px(26.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.fg_muted)
                        .child(desc.to_string()),
                )
                .child(
                    div().flex().gap_2().children(buttons.into_iter().map(|b| {
                        b.into_any_element()
                    })),
                ),
        )
        .into_any_element()
}

fn board_modal(
    records: Vec<ScoreRecord>,
    best: u32,
    entity: Entity<GameApp>,
    cx: &Context<GameApp>,
) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    Modal::new("本地排行榜 🏆")
        .on_close({
            let e = entity.clone();
            move |_, cx| {
                e.update(cx, |this, cx| {
                    this.board_open = false;
                    cx.notify();
                });
            }
        })
        .child(if records.is_empty() {
            div().h(px(140.)).child(
                EmptyState::new("trophy", "还没有成绩", "完成一局后即可上榜"),
            )
        } else {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .children(records.iter().enumerate().map(|(ix, rec)| {
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_2()
                        .rounded(theme.radius_md)
                        .bg(if ix == 0 { theme.primary.opacity(0.12) } else { theme.bg_hover.opacity(0.5) })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(16.))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(if ix == 0 { theme.warning } else { theme.fg_muted })
                                        .child(match ix {
                                            0 => "👑".to_string(),
                                            i => format!("#{}", i + 1),
                                        }),
                                )
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .font_family(theme.font_mono.clone())
                                        .child(gpui_kit::util::format_thousands(rec.score as i64)),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(theme.fg_subtle)
                                .child(format!(
                                    "最高块 {} · {}s · {}",
                                    rec.max_tile,
                                    rec.seconds,
                                    gpui_kit::util::unix_to_date(rec.time_unix)
                                )),
                        )
                }))
        })
        .child(
            div()
                .flex()
                .justify_between()
                .text_size(px(12.))
                .text_color(theme.fg_subtle)
                .child(format!("历史最佳:{}", gpui_kit::util::format_thousands(best as i64)))
                .child("Top 5"),
        )
        .footer(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("b-reset", "清空全部数据").danger().on_click({
                        let e = entity.clone();
                        move |_, _, cx| {
                            e.update(cx, |this, cx| this.reset_all(cx));
                        }
                    }),
                )
                .child(
                    Button::new("b-close", "关闭").primary().on_click({
                        let e = entity.clone();
                        move |_, _, cx| {
                            e.update(cx, |this, cx| {
                                this.board_open = false;
                                cx.notify();
                            });
                        }
                    }),
                ),
        )
        .into_any_element()
}

fn help_modal(entity: Entity<GameApp>, cx: &Context<GameApp>) -> gpui::AnyElement {
    let theme = cx.global::<Theme>().clone();
    Modal::new("玩法与快捷键")
        .on_close({
            let e = entity.clone();
            move |_, cx| {
                e.update(cx, |this, cx| {
                    this.help_open = false;
                    cx.notify();
                });
            }
        })
        .child(
            div()
                .text_size(px(13.))
                .text_color(theme.fg_muted)
                .child("使用方向键移动方块,相同数字合并翻倍,凑出 2048 即胜利!棋盘满且无法合并则游戏结束。"),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .mt_2()
                .children(SHORTCUTS.iter().map(|(k, desc)| {
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(div().text_size(px(13.)).child(desc.to_string()))
                        .child(Kbd::new(*k))
                })),
        )
        .footer(
            div().child(
                Button::new("gh-close", "关闭").primary().on_click({
                    let e = entity.clone();
                    move |_, _, cx| {
                        e.update(cx, |this, cx| {
                            this.help_open = false;
                            cx.notify();
                        });
                    }
                }),
            ),
        )
        .into_any_element()
}
