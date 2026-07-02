//! # zero-ui-render
//!
//! 通用 UI SDK 的渲染/场景抽象层（spec §8.4.1 `zero-ui-render` / FR-004 / DC-3）。
//!
//! 定义 Render·Scene tree（[`render_node`]、[`scene`]）、paint 上下文（[`paint_ctx`]）、
//! 裁剪栈（[`clip`]）、合成层（[`layer`]）与命中测试（[`hit_test`]）。
//!
//! M1 不直接依赖 `render-foundation` 后端（spec TBD-2）；通过 [`SceneRecorder`] 实现
//! [`zero_ui_core::widget::PaintRecorder`]，把 widget paint 调用记录为图元。M2 通过
//! [`backend::RenderBackend`] trait + [`backend::paint_scene`] 把 Scene 派发给具体光栅后端
//! （render-foundation 后续实现），并把文本绘制消费 [`zero_text_foundation::TextBlob`]（DC-11）。

pub mod backend;
pub mod clip;
pub mod hit_test;
pub mod layer;
pub mod paint_ctx;
pub mod render_node;
pub mod scene;

pub use backend::{RenderBackend, paint_scene};
pub use layer::Layer;
pub use paint_ctx::SceneRecorder;
pub use render_node::{RenderNode, RenderPrimitive};
pub use scene::{Scene, SceneEntry};
