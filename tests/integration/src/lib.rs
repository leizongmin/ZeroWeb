//! 跨 crate 集成测试
//!
//! 测试多个 crate 之间的协作，验证端到端的管线正确性。

#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(dead_code))]
#![cfg_attr(test, allow(unused_mut))]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::len_zero)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::useless_vec)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::for_kv_map)]
#![allow(unused_comparisons)]

#[cfg(test)]
mod dom_css;

#[cfg(test)]
mod css_style;

#[cfg(test)]
mod render_pipeline;

#[cfg(test)]
mod runtime_conformance;

#[cfg(test)]
mod b3_load_mechanism;

#[cfg(test)]
mod net_security;

#[cfg(test)]
mod network_loading;

#[cfg(test)]
mod storage;

#[cfg(test)]
mod protocol_navigation;

#[cfg(test)]
mod canvas_render;

#[cfg(test)]
mod wasm_sandbox;

#[cfg(test)]
mod webview_full_pipeline;

#[cfg(test)]
mod cross_crate_integration;

#[cfg(test)]
mod cross_crate_pipeline;

#[cfg(test)]
mod dom_bridge_polyfill;

#[cfg(test)]
mod browser_shell_integration;

#[cfg(test)]
mod e2e_canvas_dom;

// js-dom M3（R90）：Web Components 端到端验收资产（DC-2 WC 首切片）。
#[cfg(test)]
mod e2e_web_components;

// js-dom M3（R95）：真实 lit 库端到端验收资产（DC-2 WC 收口——lit/LitElement 全链路）。
#[cfg(test)]
mod e2e_lit_library;
mod e2e_vue_library;

#[cfg(test)]
mod e2e_rendering;

#[cfg(test)]
mod headless_protocol;

#[cfg(test)]
mod web_api_pipeline;

#[cfg(test)]
mod multi_process;

#[cfg(test)]
mod navigation_paint;

#[cfg(test)]
mod html_compat;

#[cfg(test)]
mod security_pipeline;

#[cfg(test)]
mod real_website_compat;

#[cfg(test)]
mod webview_product_smoke;

#[cfg(test)]
mod product_level_smoke;

#[cfg(test)]
mod viewport_adaptive;

#[cfg(test)]
mod font_fallback_render;
