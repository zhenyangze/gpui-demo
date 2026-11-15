//! AgentConsole 数据层:会话、消息、设置、持久化。

use crate::engine::{StreamEvent, AgentEngine};
use gpui::{AppContext, UpdateGlobal};
use gpui_kit::theme::{Theme, ThemeMode, Accent};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: u64,
    pub name: String,
    pub args: String,
    pub result: String,
    pub ok: bool,
    pub dur_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub time_unix: i64,
    pub tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub text: String,
    pub done: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: u64,
    pub title: String,
    pub messages: Vec<Message>,
    pub tasks: Vec<Task>,
    pub unread: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub dark: bool,
    pub accent_index: usize,
    pub model_index: usize, // 0 mock-s / 1 mock-m / 2 mock-l
    pub speed_index: usize, // 0 慢 / 1 中 / 2 快
    pub temperature: f64,
    pub system_prompt: String,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            dark: true,
            accent_index: 0,
            model_index: 1,
            speed_index: 1,
            temperature: 0.7,
            system_prompt: "你是一个乐于助人的桌面智能体。".into(),
        }
    }
}

pub const MODELS: [&str; 3] = ["mock-s (轻量)", "mock-m (均衡)", "mock-l (加强)"];
pub const SPEEDS: [&str; 3] = ["慢速", "中速", "快速"];

pub fn speed_delay_ms(speed_index: usize, model_index: usize) -> u64 {
    let base = match speed_index {
        0 => 60,
        1 => 28,
        _ => 10,
    };
    let model_factor = match model_index {
        0 => 1,
        1 => 1,
        _ => 2,
    };
    (base * model_factor) as u64
}

pub fn data_dir() -> PathBuf {
    let mut p = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    p.push(".gpui-demo");
    p.push("agentconsole");
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn sessions_path() -> PathBuf {
    let mut p = data_dir();
    p.push("sessions.json");
    p
}

pub fn settings_path() -> PathBuf {
    let mut p = data_dir();
    p.push("settings.json");
    p
}

pub fn load_sessions() -> Option<Vec<Conversation>> {
    let t = std::fs::read_to_string(sessions_path()).ok()?;
    serde_json::from_str(&t).ok()
}

pub fn save_sessions(sessions: &[Conversation]) {
    if let Ok(json) = serde_json::to_string_pretty(sessions) {
        let _ = std::fs::write(sessions_path(), json);
    }
}

pub fn load_settings() -> AgentSettings {
    std::fs::read_to_string(settings_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_settings(s: &AgentSettings) {
    if let Ok(json) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(settings_path(), json);
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn approx_tokens(s: &str) -> usize {
    (s.chars().count() / 4).max(1)
}

pub fn seed_sessions() -> Vec<Conversation> {
    vec![Conversation {
        id: 1,
        title: "欢迎使用 AgentConsole".into(),
        messages: vec![Message {
            role: Role::Assistant,
            content: "你好!我是 Mock 智能体 👋\n\n试试这些指令:\n- `(12+8)*3` — 计算器\n- `现在几点` — 时钟\n- `帮我计划一次产品发布` — 任务规划\n- `roll 2d6` — 掷骰\n\n顶部 ⌘K 打开命令面板,右侧是任务面板。".into(),
            tool_calls: vec![],
            time_unix: now(),
            tokens: approx_tokens("seed"),
        }],
        tasks: vec![],
        unread: false,
    }]
}

/// 流式播放引擎事件:逐段 update 会话(由 UI 的 spawn 驱动)
pub fn apply_event(
    conv: &mut Conversation,
    ev: &StreamEvent,
    streaming_msg: &mut Option<Message>,
) {
    match ev {
        StreamEvent::TextDelta(d) => {
            let msg = streaming_msg.get_or_insert_with(|| Message {
                role: Role::Assistant,
                content: String::new(),
                tool_calls: vec![],
                time_unix: now(),
                tokens: 0,
            });
            msg.content.push_str(d);
        }
        StreamEvent::ToolCall { id, name, args, result, ok, dur_ms } => {
            let msg = streaming_msg.get_or_insert_with(|| Message {
                role: Role::Assistant,
                content: String::new(),
                tool_calls: vec![],
                time_unix: now(),
                tokens: 0,
            });
            msg.tool_calls.push(ToolCallRecord {
                id: *id,
                name: name.clone(),
                args: args.clone(),
                result: result.clone(),
                ok: *ok,
                dur_ms: *dur_ms,
            });
        }
        StreamEvent::Tasks(tasks) => {
            conv.tasks = tasks
                .iter()
                .map(|t| Task { text: t.clone(), done: false, active: false })
                .collect();
        }
        StreamEvent::Done { tokens_in, tokens_out } => {
            if let Some(msg) = streaming_msg.take() {
                let mut msg = msg;
                msg.tokens = *tokens_out;
                conv.messages.push(msg);
            }
            let _ = tokens_in;
        }
    }
}

/// 应用主题设置(启动/设置变更时)
pub fn apply_theme(s: &AgentSettings, cx: &mut gpui::App) {
    let mut theme = if s.dark { Theme::dark() } else { Theme::light() };
    theme.set_accent(Accent::ALL.get(s.accent_index).copied().unwrap_or(Accent::Blue));
    cx.set_global(theme);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_and_persist_roundtrip() {
        let s = seed_sessions();
        let json = serde_json::to_string(&s).unwrap();
        let back: Vec<Conversation> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].messages.len(), 1);
    }

    #[test]
    fn test_apply_event_streaming() {
        let mut conv = seed_sessions().pop().unwrap();
        let mut streaming = None;
        apply_event(&mut conv, &StreamEvent::TextDelta("你好,".into()), &mut streaming);
        apply_event(&mut conv, &StreamEvent::TextDelta("世界!".into()), &mut streaming);
        assert!(streaming.is_some());
        assert_eq!(streaming.as_ref().unwrap().content, "你好,世界!");
        apply_event(
            &mut conv,
            &StreamEvent::Done { tokens_in: 1, tokens_out: 2 },
            &mut streaming,
        );
        assert!(streaming.is_none());
        assert_eq!(conv.messages.len(), 2);
        assert_eq!(conv.messages[1].tokens, 2);
    }

    #[test]
    fn test_apply_event_tools_and_tasks() {
        let mut conv = seed_sessions().pop().unwrap();
        let mut streaming = None;
        apply_event(
            &mut conv,
            &StreamEvent::ToolCall { id: 1, name: "calculator".into(), args: "1+2".into(), result: "3".into(), ok: true, dur_ms: 3 },
            &mut streaming,
        );
        apply_event(&mut conv, &StreamEvent::Tasks(vec!["a".into(), "b".into()]), &mut streaming);
        apply_event(&mut conv, &StreamEvent::Done { tokens_in: 0, tokens_out: 0 }, &mut streaming);
        assert_eq!(conv.messages[1].tool_calls.len(), 1);
        assert_eq!(conv.tasks.len(), 2);
    }

    #[test]
    fn test_speed_delay() {
        assert!(speed_delay_ms(2, 0) < speed_delay_ms(0, 0));
        assert!(speed_delay_ms(1, 2) > speed_delay_ms(1, 0));
    }

    #[test]
    fn test_approx_tokens() {
        assert!(approx_tokens("hello world!") >= 2);
        assert_eq!(approx_tokens(""), 1);
    }
}
