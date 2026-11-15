//! 2048 游戏状态:分数、历史、统计、持久化。

use crate::board::{self, Dir, Grid};
use gpui_kit::core::persist;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreRecord {
    pub score: u32,
    pub max_tile: u32,
    pub seconds: u64,
    pub time_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persisted {
    pub grid: Grid,
    pub score: u32,
    pub steps: u32,
    /// v2:合并次数(由 v1 的 merges 迁移而来)
    #[serde(default)]
    pub merge_ops: u32,
    /// v2:历史峰值方块(演示新增字段迁移)
    #[serde(default)]
    pub peak: u32,
    pub seconds: u64,
    pub best: u32,
    pub records: Vec<ScoreRecord>,
    pub night_theme: bool,
}

pub fn data_dir() -> PathBuf {
    let mut p = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    p.push(".gpui-demo");
    p.push("game2048");
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn path() -> PathBuf {
    let mut p = data_dir();
    p.push("save.json");
    p
}

/// 当前存档版本。v1:旧裸文件(无信封);v2:信封 + merges 语义改为 merge_ops 并新增 peak。
pub const SAVE_VERSION: u32 = 2;

/// 兼容迁移:裸文件(无 version 字段)→ 视为 v1 数据并包装信封
fn migrate_legacy_to_envelope() -> persist::Migration {
    Box::new(|mut v| {
        // persist::load 已保证传进来的是信封 data;此处处理"信封缺失"的旧格式由 load_any 完成,
        // 该迁移将 merges 字段重命名为 merge_ops(演示真实字段迁移)
        if let Some(obj) = v.as_object_mut() {
            if let Some(merges) = obj.remove("merges") {
                obj.insert("merge_ops".into(), merges);
            }
            obj.entry(String::from("peak")).or_insert(json!(0));
        }
        Ok(v)
    })
}

pub fn load() -> Option<Persisted> {
    // 旧版裸文件(无信封)→ 读出来后包一层 {version:1, data}
    let raw = std::fs::read_to_string(path()).ok()?;
    let is_envelope = serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| v.get("version").cloned())
        .is_some();
    let text = if is_envelope {
        raw
    } else {
        // 裸 v1 数据,字段名映射到 v2 前先保持原名;信封 version=1
        json!({ "version": 1, "data": serde_json::from_str::<serde_json::Value>(&raw).ok()? }).to_string()
    };
    let tmp = path().with_extension("migrating.json");
    std::fs::write(&tmp, text).ok();

    let result = persist::load::<Persisted>(
        &tmp,
        SAVE_VERSION,
        &[migrate_legacy_to_envelope()],
    );
    let _ = std::fs::remove_file(&tmp);
    match result {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("[game2048] 存档迁移失败:{e:?}");
            None
        }
    }
}

pub fn save(p: &Persisted) {
    let _ = persist::save(&path(), SAVE_VERSION, p);
}

pub fn default_records() -> Vec<ScoreRecord> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persist_roundtrip_v2() {
        let p = Persisted {
            grid: board::initial(),
            score: 1024,
            steps: 40,
            merge_ops: 20,
            peak: 256,
            seconds: 120,
            best: 2048,
            records: vec![ScoreRecord { score: 2048, max_tile: 2048, seconds: 300, time_unix: 1_000_000 }],
            night_theme: true,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: Persisted = serde_json::from_str(&json).unwrap();
        assert_eq!(back.score, 1024);
        assert_eq!(back.merge_ops, 20);
        assert_eq!(back.peak, 256);
        assert_eq!(back.grid, p.grid);
    }

    #[test]
    fn test_legacy_v1_file_migrates_to_v2() {
        // 构造 v1 裸文件(merges 字段)到真实数据目录(load() 读那里)
        let dir = data_dir();
        std::fs::create_dir_all(&dir).ok();
        let legacy = json!({
            "grid": [[2,0,0,0],[0,0,0,0],[0,0,0,0],[0,0,0,0]],
            "score": 64, "steps": 6, "merges": 3, "seconds": 42,
            "best": 128, "records": [], "night_theme": false
        });
        let p = path();
        std::fs::write(&p, serde_json::to_string(&legacy).unwrap()).unwrap();

        // 走真实 load()(裸文件 → 信封 → 迁移)
        let loaded = load().unwrap_or_else(|| panic!("迁移失败"));
        assert_eq!(loaded.score, 64);
        assert_eq!(loaded.merge_ops, 3, "merges 应迁移为 merge_ops");
        assert_eq!(loaded.peak, 0, "新增字段取默认值");
        // 清理,避免污染后续运行
        let _ = std::fs::remove_file(&p);
        let bak = p.with_extension("bak.json");
        let _ = std::fs::remove_file(&bak);
        let mig = p.with_extension("migrating.json");
        let _ = std::fs::remove_file(&mig);
    }
}
