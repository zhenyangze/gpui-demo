//! DevToolbox 状态:当前工具、输入缓存、历史/收藏/设置持久化。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const TOOL_NAMES: [&str; 12] = [
    "JSON", "正则", "Base64", "URL", "哈希", "时间", "UUID", "进制", "颜色", "Diff", "文本", "计算",
];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Persisted {
    pub current_tool: usize,
    pub dark: bool,
    pub favorites: Vec<usize>,
    /// 每个工具最近 10 条输入
    pub history: std::collections::HashMap<usize, Vec<String>>,
}

pub fn data_dir() -> PathBuf {
    let mut p = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    p.push(".gpui-demo");
    p.push("devtoolbox");
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn path() -> PathBuf {
    let mut p = data_dir();
    p.push("state.json");
    p
}

pub fn load() -> Persisted {
    std::fs::read_to_string(path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(p: &Persisted) {
    if let Ok(json) = serde_json::to_string_pretty(p) {
        let _ = std::fs::write(path(), json);
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn push_history(hist: &mut Vec<String>, entry: &str) {
    if entry.trim().is_empty() {
        return;
    }
    hist.retain(|h| h != entry);
    hist.insert(0, entry.to_string());
    hist.truncate(10);
}
