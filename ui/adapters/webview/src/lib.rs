//! # zero-ui-adapter-webview
//!
//! WebViewWidget — 把 `zero-webview` 包装为 UI SDK 的高级自定义组件（spec FR-005 / IF-004 / DC-3）。
//!
//! **架构边界**：UI SDK 只计算 WebViewWidget 的**外部矩形**，分配 viewport/clip/scale/theme/输入；
//! `zero-webview` 自处理 HTML/CSS/DOM/layout/paint，输出 RenderPrimitives/Texture/SceneNode；
//! 本组件把输出合成进 UI scene。**不得**把网页 DOM 映射为 UI widgets（spec FR-005 约束 5）。
//!
//! 这是 UI SDK **唯一**允许依赖 `zero-webview` 的浏览器耦合点（spec 约束 3 / DC-1）。
//! M1 skeleton：layout 输入 + scroll bridge；真实渲染合成在 M2。

pub mod scroll_bridge;
pub mod webview_widget;

pub use scroll_bridge::apply_scroll_command;
pub use webview_widget::{WebViewLayoutInput, WebViewPaintOutput, WebViewWidget, WebviewBackend};
