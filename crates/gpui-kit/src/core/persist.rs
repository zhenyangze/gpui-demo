//! # Persist — 版本化持久化 + 迁移链(L2)
//!
//! 文件信封:`{ "version": N, "data": <应用数据> }`。
//! 加载时按迁移链逐级升级到当前版本;损坏文件回退默认值并保留 `.bak`。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub type Migration = Box<dyn Fn(Value) -> Result<Value, String> + 'static>;

#[derive(Serialize, Deserialize)]
struct Envelope {
    version: u32,
    data: Value,
}

#[derive(Debug)]
pub enum LoadError {
    NotFound,
    Corrupted(String),
}

/// 读取并迁移到 current_version
pub fn load<D: for<'de> Deserialize<'de>>(
    path: &Path,
    current_version: u32,
    migrations: &[Migration],
) -> Result<D, LoadError> {
    let text = std::fs::read_to_string(path).map_err(|_| LoadError::NotFound)?;

    let envelope: Envelope = serde_json::from_str(&text)
        .map_err(|e| LoadError::Corrupted(format!("信封解析失败:{e}")))?;

    let mut version = envelope.version;
    let mut data = envelope.data;

    // 逐级迁移
    while version < current_version {
        let idx = (version as usize).saturating_sub(1);
        let Some(mig) = migrations.get(idx) else {
            return Err(LoadError::Corrupted(format!("缺少迁移:v{version} → v{}", version + 1)));
        };
        data = mig(data).map_err(|e| LoadError::Corrupted(format!("v{version} 迁移失败:{e}")))?;
        version += 1;
    }
    if version > current_version {
        return Err(LoadError::Corrupted(format!("文件版本 v{version} 高于程序 v{current_version}")));
    }

    serde_json::from_value(data).map_err(|e| LoadError::Corrupted(format!("目标结构解析失败:{e}")))
}

/// 保存(写入当前版本信封);写入前对旧文件做 .bak 备份
pub fn save<D: Serialize>(path: &Path, current_version: u32, data: &D) -> std::io::Result<()> {
    let envelope = Envelope {
        version: current_version,
        data: serde_json::to_value(data).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
    };
    let json = serde_json::to_string_pretty(&envelope)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if path.exists() {
        let mut bak = PathBuf::from(path);
        bak.set_extension("bak.json");
        let _ = std::fs::copy(path, bak);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, json)
}

/// 便捷迁移:重命名顶层键
pub fn migrate_map_keys(from: &'static str, to: &'static str) -> Migration {
    Box::new(move |mut v: Value| {
        if let Value::Object(map) = &mut v {
            if let Some(val) = map.remove(from) {
                map.insert(to.to_string(), val);
            }
        }
        Ok(v)
    })
}

/// 便捷迁移:插入/覆盖一个默认字段
pub fn migrate_add_field(key: &'static str, value: serde_json::Value) -> Migration {
    Box::new(move |mut v: Value| {
        if let Value::Object(map) = &mut v {
            map.insert(key.to_string(), value.clone());
        }
        Ok(v)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("gpui-kit-persist-test-{}", name));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct AppV1 {
        title: String,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct AppV2 {
        title: String,
        count: u32,
    }

    #[test]
    fn test_save_load_roundtrip_current_version() {
        let p = tmp("rt.json");
        let app = AppV2 { title: "hi".into(), count: 7 };
        save(&p, 2, &app).unwrap();
        let back: AppV2 = load(&p, 2, &[]).unwrap();
        assert_eq!(back, app);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_migration_v1_to_v2() {
        let p = tmp("mig.json");
        // 手写 v1 文件(无信封 version → 用信封包)
        let v1 = json!({ "version": 1, "data": { "title": "old" } });
        std::fs::write(&p, serde_json::to_string(&v1).unwrap()).unwrap();

        let migrate: Migration = Box::new(|mut v: Value| {
            if let Value::Object(map) = &mut v {
                map.insert("count".into(), json!(42));
            }
            Ok(v)
        });
        let back: AppV2 = load(&p, 2, &[migrate]).unwrap();
        assert_eq!(back, AppV2 { title: "old".into(), count: 42 });
        // 迁移后应自动落盘为新版本?——设计为只读迁移,由调用方决定保存。此处不落盘。
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_chain_of_two_migrations() {
        let p = tmp("chain.json");
        let v1 = json!({ "version": 1, "data": { "old_key": 1 } });
        std::fs::write(&p, serde_json::to_string(&v1).unwrap()).unwrap();

        let rename = migrate_map_keys("old_key", "new_key");
        let add_field: Migration = Box::new(|mut v: Value| {
            if let Value::Object(map) = &mut v {
                map.insert("extra".into(), json!(true));
            }
            Ok(v)
        });
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct V3 {
            new_key: u32,
            extra: bool,
        }
        let back: V3 = load(&p, 3, &[rename, add_field]).unwrap();
        assert_eq!(back, V3 { new_key: 1, extra: true });
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_missing_migration_is_error() {
        let p = tmp("nomig.json");
        let v1 = json!({ "version": 1, "data": {} });
        std::fs::write(&p, serde_json::to_string(&v1).unwrap()).unwrap();
        let res: Result<AppV2, _> = load(&p, 2, &[]);
        assert!(matches!(res, Err(LoadError::Corrupted(_))));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_future_version_is_error() {
        let p = tmp("future.json");
        let v99 = json!({ "version": 99, "data": {} });
        std::fs::write(&p, serde_json::to_string(&v99).unwrap()).unwrap();
        let res: Result<AppV2, _> = load(&p, 2, &[]);
        assert!(matches!(res, Err(LoadError::Corrupted(_))));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_corrupted_file() {
        let p = tmp("bad.json");
        std::fs::write(&p, "not json {{{").unwrap();
        let res: Result<AppV2, _> = load(&p, 2, &[]);
        assert!(matches!(res, Err(LoadError::Corrupted(_))));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn test_not_found() {
        let res: Result<AppV2, _> = load(&tmp("absent.json"), 2, &[]);
        assert!(matches!(res, Err(LoadError::NotFound)));
    }

    #[test]
    fn test_bak_created_on_save() {
        let p = tmp("bak.json");
        save(&p, 2, &AppV2 { title: "a".into(), count: 1 }).unwrap();
        save(&p, 2, &AppV2 { title: "b".into(), count: 2 }).unwrap();
        let mut bak = p.clone();
        bak.set_extension("bak.json");
        assert!(bak.exists());
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_file(&bak);
    }

    #[test]
    fn test_migrate_map_keys_helper() {
        let v = json!({ "old_key": 3 });
        let out = migrate_map_keys("old_key", "new_key")(v).unwrap();
        assert_eq!(out, json!({ "new_key": 3 }));
    }
}
