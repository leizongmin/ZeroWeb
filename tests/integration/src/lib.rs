//! 跨 crate 集成测试
//!
//! 测试多个 crate 之间的协作，验证端到端的管线正确性。

#[cfg(test)]
mod dom_css;

#[cfg(test)]
mod css_style;

#[cfg(test)]
mod render_pipeline;

#[cfg(test)]
mod net_security;

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

#[cfg(test)]
mod e2e_rendering;
