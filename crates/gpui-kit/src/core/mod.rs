//! # L2 核心服务层
//! - [`store`]:响应式状态容器
//! - [`history`]:撤销/重做
//! - [`persist`]:版本化持久化与迁移
//! - [`command`]:命令注册中心

pub mod command;
pub mod history;
pub mod persist;
pub mod store;
