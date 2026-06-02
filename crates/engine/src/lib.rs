//! # zero-engine
//!
//! 页面内核 — HTML/DOM/CSSOM/样式/布局/绘制/脚本协调。
//!
//! 整合各子模块，实现完整的页面加载和渲染管线。
//!
//! ## 核心模块
//!
//! - [`paint`] — 绘制命令生成，将布局盒树转换为渲染图元
//! - [`dirty`] — 脏区域追踪，管理需要重绘的屏幕区域
//! - [`composite`] — 合成层逻辑，决定元素图层分配
//! - [`pipeline`] — 端到端渲染管线，编排 HTML→CSS→Layout→Paint
//! - [`preload`] — 资源预加载，解析 `<link rel="preload/prefetch">` 提示

#![warn(missing_docs)]

pub mod composite;
pub mod dirty;
pub mod dom_bridge;
pub mod paint;
pub mod pipeline;
pub mod preload;

pub use composite::*;
pub use dirty::*;
pub use dom_bridge::*;
pub use paint::*;
pub use pipeline::*;
pub use preload::*;

#[cfg(test)]
mod tests;
