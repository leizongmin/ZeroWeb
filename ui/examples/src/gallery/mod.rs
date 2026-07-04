//! 组件画廊示例（DC-14）—— 验证通用 UI SDK 导航切换 + 组件预览 + 源码展示。
//!
//! 布局：Header（标题 + 语言切换）+ Body（左侧导航 + 右侧 Demo 区）。
//! 导航项点击会 emit action → GalleryApp reducer 切换 current_page → 重建 widget tree。

pub mod app;
pub mod highlight;
pub mod model;
pub mod pages;

pub use app::{GalleryApp, register_gallery_factories};
