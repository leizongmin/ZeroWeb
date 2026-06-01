//! # zero-wasm-sandbox
//!
//! 非页面 WASM 运行时（wasmi）。
//!
//! 用于插件、扩展能力或受控计算任务。
//! 基于 wasmi 纯 Rust WASM 解释器实现。

#![warn(missing_docs)]

mod types;
pub use types::*;

#[cfg(feature = "wasmi")]
mod wasmi_backend;

#[cfg(feature = "wasmi")]
pub use wasmi_backend::*;

#[cfg(not(feature = "wasmi"))]
mod stub_backend;

#[cfg(not(feature = "wasmi"))]
pub use stub_backend::*;

#[cfg(test)]
mod tests;
