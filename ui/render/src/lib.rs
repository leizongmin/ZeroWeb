//! # zero-ui-render
//!
//! 通用 UI SDK 的渲染/场景抽象层（spec §8.4.1 `zero-ui-render` / FR-004 / DC-3）。
//!
//! 定义 Render·Scene tree（[`render_node`]、[`scene`]）、paint 上下文（[`paint_ctx`]）、
//! 裁剪栈（[`clip`]）、合成层（[`layer`]）与命中测试（[`hit_test`]）。
//!
//! M1 不直接依赖 `render-foundation` 后端（spec TBD-2）；通过 [`SceneRecorder`] 实现
//! [`zero_ui_core::widget::PaintRecorder`]，把 widget paint 调用记录为图元，再由 M2 桥接
//! 到 render-foundation 的 GPU/CPU 后端。

pub mod clip;
pub mod hit_test;
pub mod layer;
pub mod paint_ctx;
pub mod render_node;
pub mod scene;

pub use layer::Layer;
pub use paint_ctx::SceneRecorder;
pub use render_node::{RenderNode, RenderPrimitive};
pub use scene::{Scene, SceneEntry};
