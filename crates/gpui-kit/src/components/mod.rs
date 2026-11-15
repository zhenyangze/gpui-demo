//! gpui-kit 组件库
//!
//! 所有组件从 [`crate::theme::Theme`] Global 取设计 token。
//! 交互组件一律要求外部传入回调(闭包拿到 `&mut App`),保持"kit 无业务状态"的框架定位。

pub mod basic;
pub mod button;
pub mod controls;
pub mod input;
pub mod nav;
pub mod overlay;
pub mod palette;
pub mod select;
pub mod table;
pub mod tree;
