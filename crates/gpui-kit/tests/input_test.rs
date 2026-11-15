//! 输入组件无头集成测试:验证与真实打字相同的管线
//! (dispatch_keystroke → ElementInputHandler → replace_text_in_range)。

use gpui::{prelude::*, TestAppContext};
use gpui_kit::components::input::{text_input, InputState};
use gpui_kit::prelude::*;

struct Harness {
    input: Entity<InputState>,
    submitted: Entity<std::cell::RefCell<Option<String>>>,
    changes: Entity<std::cell::RefCell<usize>>,
}

impl Render for Harness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(text_input("t", &self.input.clone()))
    }
}

#[gpui::test]
fn input_typing_via_ime_pipeline(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init_kit(cx);
    });

    let window = cx.add_window(|_window, cx| {
        let submitted = cx.new(|_| std::cell::RefCell::new(None));
        let changes = cx.new(|_| std::cell::RefCell::new(0));
        let submitted_for_cb = submitted.clone();
        let changes_for_cb = changes.clone();
        let input = cx.new(|cx| {
            InputState::new(cx)
                .on_change(move |_text, _window, _cx| {
                    changes_for_cb.update(_cx, |c, _| *c.borrow_mut() += 1);
                })
                .on_submit(move |text, _shift, _window, _cx| {
                    submitted_for_cb.update(_cx, |c, _| {
                        *c.borrow_mut() = Some(text.to_string());
                    });
                })
        });
        Harness {
            input,
            submitted,
            changes,
        }
    });

    // 聚焦输入框
    window
        .update(cx, |root, window, app| {
            root.input.read(app).focus(window);
            assert!(root.input.read(app).is_focused(window));
        })
        .unwrap();

    // 模拟真实打字(与 IME 同管线)
    cx.simulate_input(window.into(), "hello 你好");
    window
        .update(cx, |root, _window, cx| {
            assert_eq!(root.input.read(cx).text(), "hello 你好");
        })
        .unwrap();

    // 退格删除一个字形(grapheme 边界,不应删半个汉字)
    cx.simulate_keystrokes(window.into(), "backspace");
    window
        .update(cx, |root, _window, cx| {
            assert_eq!(root.input.read(cx).text(), "hello 你");
        })
        .unwrap();

    // 光标移动 + 选区全选
    cx.simulate_keystrokes(window.into(), "cmd-a");
    window
        .update(cx, |root, _window, cx| {
            assert!(root.input.read(cx).has_selection());
            assert_eq!(root.input.read(cx).selected_range(), 0..9);
        })
        .unwrap();

    // 提交(enter)
    cx.simulate_keystrokes(window.into(), "enter");
    window
        .update(cx, |root, _window, cx| {
            let submitted = root.submitted.read(cx).borrow().clone();
            assert_eq!(submitted, Some("hello 你".to_string()));
            let changes = root.changes.read(cx).borrow().clone();
            assert!(changes >= 5, "on_change 应触发多次,实际 {changes}");
        })
        .unwrap();
}

#[gpui::test]
fn input_clear_and_set_text(cx: &mut TestAppContext) {
    cx.update(|cx| {
        gpui_kit::init_kit(cx);
    });
    let window = cx.add_window(|_window, cx| {
        let input = cx.new(|cx| InputState::new(cx));
        Harness {
            input,
            submitted: cx.new(|_| std::cell::RefCell::new(None)),
            changes: cx.new(|_| std::cell::RefCell::new(0)),
        }
    });

    window
        .update(cx, |root, window, cx| {
            root.input.update(cx, |s, cx| {
                s.set_text("abc", window, cx);
            });
            assert_eq!(root.input.read(cx).text(), "abc");
            // 清空
            root.input.update(cx, |s, cx| {
                s.clear(window, cx);
            });
            assert_eq!(root.input.read(cx).text(), "");
        })
        .unwrap();
}
