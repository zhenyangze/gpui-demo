//! AgentConsole 引擎层:AgentEngine trait、MockEngine、工具系统、意图路由。

use serde::{Deserialize, Serialize};

/// 一轮回复的流式事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    /// 追加文本增量
    TextDelta(String),
    /// 工具调用(带结果与耗时)
    ToolCall {
        id: u64,
        name: String,
        args: String,
        result: String,
        ok: bool,
        dur_ms: u64,
    },
    /// 任务清单更新(工具 task_planner 产生)
    Tasks(Vec<String>),
    /// 回复结束
    Done {
        tokens_in: usize,
        tokens_out: usize,
    },
}

/// 引擎 trait:实现它即可接入任意真实 LLM(见 README 的接入指引)
pub trait AgentEngine: 'static {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// 依据用户输入产出事件序列(同步产出;流式播放由 UI 层控制节奏)
    fn respond(&self, input: &str, model: &str, temperature: f64) -> Vec<StreamEvent>;
}

// ------------------------- 工具实现 -------------------------

pub mod tools {
    use std::time::Instant;

    pub struct ToolResult {
        pub name: &'static str,
        pub args: String,
        pub result: String,
        pub ok: bool,
        pub dur_ms: u64,
    }

    /// 工具定义:名称/描述/执行函数 —— 注册制,新增工具只需追加一项
    pub struct ToolDef {
        pub name: &'static str,
        pub description: &'static str,
        pub run: fn(&str) -> ToolResult,
    }

    /// 工具注册表(静态,启动期构建)
    pub fn registry() -> &'static [ToolDef] {
        &[
            ToolDef {
                name: "calculator",
                description: "四则/括号/幂/取模表达式求值",
                run: run_calculator,
            },
            ToolDef { name: "clock", description: "查询当前本地日期时间", run: run_clock },
            ToolDef { name: "roll", description: "掷骰子,格式 NdM", run: run_roll },
        ]
    }

    /// 按名称执行注册表中的工具
    pub fn execute(name: &str, args: &str) -> Option<ToolResult> {
        registry()
            .iter()
            .find(|t| t.name == name)
            .map(|t| (t.run)(args))
    }

    pub fn run_calculator(expr: &str) -> ToolResult {
        let t = Instant::now();
        match gpui_kit::util::eval_math(expr) {
            Ok(v) => ToolResult {
                name: "calculator",
                args: expr.to_string(),
                result: {
                    if v.fract() == 0.0 && v.abs() < 1e15 {
                        format!("{}", gpui_kit::util::format_thousands(v as i64))
                    } else {
                        format!("{v}")
                    }
                },
                ok: true,
                dur_ms: t.elapsed().as_millis() as u64 + 2,
            },
            Err(e) => ToolResult {
                name: "calculator",
                args: expr.to_string(),
                result: format!("错误:{e}"),
                ok: false,
                dur_ms: t.elapsed().as_millis() as u64 + 2,
            },
        }
    }

    pub fn run_clock(_args: &str) -> ToolResult {
        let t = Instant::now();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        ToolResult {
            name: "clock",
            args: "local".into(),
            result: format!("{}(本地)", gpui_kit::util::unix_to_datetime(now)),
            ok: true,
            dur_ms: t.elapsed().as_millis() as u64 + 1,
        }
    }

    pub fn run_roll(args: &str) -> ToolResult {
        let t = Instant::now();
        // 形如 "2d6" 或 "1d20"
        let mut parts = args.trim().to_lowercase();
        parts.retain(|c| c != ' ');
        let ok = parts.contains('d');
        let result = if ok {
            let (n, faces) = parts.split_once('d').unwrap();
            let n: u32 = n.parse().unwrap_or(1).clamp(1, 20);
            let faces: u32 = faces.parse().unwrap_or(6).clamp(2, 100);
            let rolls: Vec<u32> = (0..n)
                .map(|_| 1 + rand::random::<u32>() % faces)
                .collect();
            format!("🎲 {} => {:?} 合计 {}", parts, rolls, rolls.iter().sum::<u32>())
        } else {
            "错误:格式应为 NdM,如 2d6".into()
        };
        ToolResult {
            name: "roll",
            args: args.to_string(),
            result,
            ok,
            dur_ms: t.elapsed().as_millis() as u64 + 1,
        }
    }
}

// ------------------------- Mock 引擎 -------------------------

pub struct MockEngine;

/// 意图类型(测试可见)
#[derive(Debug, PartialEq, Eq)]
pub enum Intent {
    Math,
    Clock,
    Task,
    Roll,
    Greeting,
    Explain,
    Code,
    General,
}

/// 规则意图路由(供测试与引擎共用)
pub fn classify(input: &str) -> Intent {
    let s = input.trim();
    let lower = s.to_lowercase();
    if s.is_empty() {
        return Intent::General;
    }
    // 算式:包含数字与运算符
    if lower.contains(|c: char| c.is_ascii_digit())
        && (lower.contains('+') || lower.contains('-') || lower.contains('*') || lower.contains('/') || lower.contains('^') || lower.contains('%'))
        && !lower.contains(' ')
    {
        return Intent::Math;
    }
    if lower.contains("几点") || lower.contains("时间") || lower.contains("日期") || lower.contains("today") || lower.contains("now") {
        return Intent::Clock;
    }
    if lower.contains("待办") || lower.contains("任务") || lower.contains("计划") || lower.contains("todo") || lower.contains("task") {
        return Intent::Task;
    }
    if lower.starts_with("roll") || lower.contains("掷骰") || lower.contains("d6") || lower.contains("d20") {
        return Intent::Roll;
    }
    if lower.contains("你好") || lower.contains("hi") || lower.contains("hello") {
        return Intent::Greeting;
    }
    if lower.contains("写代码") || lower.contains("示例") || lower.contains("example") || lower.contains("code") || lower.contains("rust") {
        return Intent::Code;
    }
    if lower.contains("解释") || lower.contains("什么是") || lower.contains("为什么") || lower.contains("how") || lower.contains("what") {
        return Intent::Explain;
    }
    Intent::General
}

/// 任务规划模板
pub fn plan_tasks(goal: &str) -> Vec<String> {
    vec![
        format!("澄清目标:{goal}"),
        "拆解为可执行步骤".to_string(),
        "执行并验证结果".to_string(),
        "输出总结报告".to_string(),
    ]
}

impl AgentEngine for MockEngine {
    fn id(&self) -> &'static str {
        "mock"
    }

    fn name(&self) -> &'static str {
        "Mock 引擎(本地规则)"
    }

    fn respond(&self, input: &str, model: &str, temperature: f64) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        let mut tokens_out = 0usize;
        let tid = rand::random::<u64>();
        let flavor = if temperature > 0.7 { "(有创造力模式)" } else { "" };
        match classify(input) {
            Intent::Math => {
                let expr = input.trim();
                let r = tools::run_calculator(expr);
                events.push(StreamEvent::ToolCall {
                    id: tid,
                    name: r.name.to_string(),
                    args: r.args.clone(),
                    result: r.result.clone(),
                    ok: r.ok,
                    dur_ms: r.dur_ms,
                });
                let text = if r.ok {
                    format!("计算结果:**{}** = {}\n\n需要我再帮你换算单位或写公式吗?", r.args, r.result)
                } else {
                    format!("抱歉,表达式无法求值:{}\n\n支持 `+ - * / % ^` 与括号,试试 `(12+8)*3`。", r.result)
                };
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
            Intent::Clock => {
                let r = tools::execute("clock", "").unwrap_or_else(|| tools::run_clock(""));
                events.push(StreamEvent::ToolCall {
                    id: tid,
                    name: r.name.to_string(),
                    args: r.args.clone(),
                    result: r.result.clone(),
                    ok: true,
                    dur_ms: r.dur_ms,
                });
                let text: String = format!("当前时间如上所示 {}。\n\n需要我帮你设个提醒或倒计时吗?", flavor);
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
            Intent::Task => {
                let goal = input.trim().trim_start_matches("帮我").trim_start_matches("计划").trim();
                let tasks = plan_tasks(if goal.is_empty() { "未命名目标" } else { goal });
                events.push(StreamEvent::ToolCall {
                    id: tid,
                    name: "task_planner".to_string(),
                    args: format!("goal=\"{}\"", goal),
                    result: format!("生成 {} 项任务", tasks.len()),
                    ok: true,
                    dur_ms: 3,
                });
                events.push(StreamEvent::Tasks(tasks.clone()));
                let mut text = String::from("已为你生成任务清单,右侧任务面板可勾选进度:\n\n");
                for (ix, t) in tasks.iter().enumerate() {
                    text.push_str(&format!("{}. {}\n", ix + 1, t));
                }
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
            Intent::Roll => {
                let r = tools::execute("roll", input.trim().trim_start_matches("roll").trim())
                    .unwrap_or_else(|| tools::run_roll("2d6"));
                events.push(StreamEvent::ToolCall {
                    id: tid,
                    name: r.name.to_string(),
                    args: r.args.clone(),
                    result: r.result.clone(),
                    ok: r.ok,
                    dur_ms: r.dur_ms,
                });
                let text: String = if r.ok { "掷骰完成,结果见上。".to_string() } else { r.result.clone() };
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
            Intent::Greeting => {
                let text: String = format!("你好!我是运行在 **{}** 上的 Mock 智能体。\n\n我可以:\n- 🧮 计算:`(12+8)*3`\n- 🕐 查时间:`现在几点`\n- 📋 规划任务:`帮我计划一次发布`\n- 🎲 掷骰:`roll 2d6`\n\n试试看!", model);
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
            Intent::Code => {
                let text: &str = "这是一个用 GPUI 渲染按钮的最小示例:\n\n```rust\nuse gpui::prelude::*;\n\nfn main() {\n    Application::new().run(|cx: &mut App| {\n        cx.open_window(WindowOptions::default(), |_, cx| {\n            cx.new(|_| HelloWorld)\n        }).unwrap();\n    });\n}\n```\n\n要点:\n1. `Application::new()` 启动事件循环\n2. `open_window` 创建窗口与根视图\n3. 实现 `Render` trait 即为视图\n\n需要更完整的工程模板请看本仓库的 gpui-kit。";
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text.to_string()));
            }
            Intent::Explain => {
                let text: String = format!("关于「{}」,Mock 引擎的离线解释如下:\n\n- 这是一个**演示环境**,回答来自本地规则模板\n- 真实接入 LLM 只需实现 `AgentEngine` trait(见项目 README)\n- 知识性问题的准确性取决于所接入的模型 {}\n\n换个说法再问我一次,或试试工具类指令。", input.trim(), flavor);
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
            Intent::General => {
                let text: String = format!("收到:「{}」\n\n我目前是离线 Mock 引擎,擅长以下指令:\n\n- 算式求值(如 `(12+8)*3`)\n- 查询当前时间\n- 生成任务清单(如「帮我计划发布」)\n- 掷骰(如 `roll 2d6`)\n\n> 接入真实模型:实现 `AgentEngine` trait 并注册到引擎注册表。", input.trim());
                tokens_out += text.len() / 4;
                events.push(StreamEvent::TextDelta(text));
            }
        }
        events.push(StreamEvent::Done {
            tokens_in: input.len() / 4,
            tokens_out,
        });
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        assert_eq!(classify("(12+8)*3"), Intent::Math);
        assert_eq!(classify("2^10"), Intent::Math);
        assert_eq!(classify("现在几点"), Intent::Clock);
        assert_eq!(classify("今天日期"), Intent::Clock);
        assert_eq!(classify("帮我计划一次发布"), Intent::Task);
        assert_eq!(classify("生成待办"), Intent::Task);
        assert_eq!(classify("roll 2d6"), Intent::Roll);
        assert_eq!(classify("你好"), Intent::Greeting);
        assert_eq!(classify("写个 rust 示例"), Intent::Code);
        assert_eq!(classify("什么是 GPUI"), Intent::Explain);
        assert_eq!(classify("随便说点什么"), Intent::General);
        assert_eq!(classify(""), Intent::General);
    }

    #[test]
    fn test_engine_math_toolcall() {
        let e = MockEngine;
        let evs = e.respond("(12+8)*3", "mock-m", 0.5);
        let has_tool = evs.iter().any(|ev| matches!(ev, StreamEvent::ToolCall { ok: true, name, .. } if name == "calculator"));
        assert!(has_tool);
        let done = evs.iter().any(|ev| matches!(ev, StreamEvent::Done { tokens_out, .. } if *tokens_out > 0));
        assert!(done);
    }

    #[test]
    fn test_engine_task_events() {
        let e = MockEngine;
        let evs = e.respond("帮我计划产品发布", "mock-s", 0.2);
        let tasks = evs.iter().find_map(|ev| match ev {
            StreamEvent::Tasks(t) => Some(t.clone()),
            _ => None,
        });
        assert!(tasks.is_some());
        assert_eq!(tasks.unwrap().len(), 4);
    }

    #[test]
    fn test_engine_roll() {
        let e = MockEngine;
        let evs = e.respond("roll 2d6", "mock-s", 0.2);
        let ok = evs.iter().any(|ev| matches!(ev, StreamEvent::ToolCall { ok: true, name, .. } if name == "roll"));
        assert!(ok);
    }

    #[test]
    fn test_engine_greeting_general() {
        let e = MockEngine;
        assert!(e.respond("你好", "mock-s", 0.2).len() >= 2);
        assert!(e.respond("天气如何", "mock-s", 0.2).len() >= 2);
    }

    #[test]
    fn test_tools() {
        assert!(tools::run_calculator("1+2").ok);
        assert!(!tools::run_calculator("1/0").ok);
        assert!(tools::run_clock("").ok);
        assert!(tools::run_roll("2d6").ok);
        assert!(!tools::run_roll("xyz").ok);
    }
}
