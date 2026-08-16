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
//! - [`animation`] — CSS 动画运行时，关键帧插值与动画时钟
//! - [`transition`] — CSS Transition 执行引擎，样式变化过渡插值

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(dead_code))]
#![cfg_attr(test, allow(unused_mut))]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::len_zero)]
#![allow(clippy::comparison_to_empty)]
#![allow(clippy::type_complexity)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::too_many_arguments)]

pub mod animation;
#[cfg(feature = "script-runtime")]
pub mod async_resolver;
pub mod composite;
pub mod dirty;
pub mod dom_bridge;
// P1b S1 原生 DOM 绑定（feature-gated v8；替换 polyfill 字符串桥，RFC p1b-v8-native-bindings）。
#[cfg(feature = "v8")]
pub mod dom_bindings;
// js-dom goal M6 S0q：QuickJS（rquickjs）原生 DOM 绑定骨架（镜像 V8 dom_bindings，DC-7 双引擎对等）。
pub mod element_from_point;
#[cfg(feature = "script-runtime")]
pub mod fetch_bridge;
#[cfg(feature = "script-runtime")]
pub mod font_load_bridge;
pub mod hit_test;
pub mod js_dom_bridge;
#[cfg(feature = "script-runtime")]
pub mod navigation_bridge;
pub mod paint;
pub mod pipeline;
mod pipeline_budget;
pub mod preload;
#[cfg(feature = "quickjs")]
pub mod quickjs_dom_bindings;
pub mod rect_bridge;
pub mod text_metrics;
#[cfg(feature = "script-runtime")]
pub mod timer_bridge;
pub mod transition;

pub use animation::*;
#[cfg(feature = "script-runtime")]
pub use async_resolver::*;
pub use composite::*;
pub use dirty::*;
pub use dom_bridge::*;
pub use element_from_point::*;
#[cfg(feature = "script-runtime")]
pub use fetch_bridge::*;
#[cfg(feature = "script-runtime")]
pub use font_load_bridge::*;
pub use hit_test::*;
pub use js_dom_bridge::*;
#[cfg(feature = "script-runtime")]
pub use navigation_bridge::*;
pub use paint::*;
pub use pipeline::*;
pub use pipeline_budget::{BudgetAdvance, BudgetStep, BudgetedRenderSession};
pub use preload::*;
pub use rect_bridge::*;
pub use text_metrics::{
    HmtxMeasureFn, TextShapeFn, font_variations_enabled, layout_estimate_char_width, measure_char_for_font,
    measure_text_hmtx_for_layout, set_char_measure_fn, set_hmtx_measure_fn, set_text_shape_fn, shape_text_for_paint,
};
#[cfg(feature = "script-runtime")]
pub use timer_bridge::*;
/// 渲染媒体类型（DC-12 @media print/screen；R1992 webview 生产接线）。
pub use zero_css_parser::media_query::MediaType;
pub use zero_css_parser::media_query::PrefersColorSchemeValue;
pub use zero_render_foundation::display_list::DisplayList;

#[cfg(test)]
mod tests;
