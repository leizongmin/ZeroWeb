//! # zero-wasm-sandbox
//!
//! WASM 运行时沙箱。
//!
//! 支持两种后端：
//! - **wasmi**（默认）— 纯 Rust 解释器，适用于插件和扩展
//! - **wasmtime** — JIT 编译器，适用于页面级 WASM 执行
//!
//! 同时启用两者时，使用 wasmtime 后端（JIT 性能更优）。

#![warn(missing_docs)]

mod types;
pub use types::*;

#[cfg(feature = "wasmtime")]
mod wasmtime_backend;

#[cfg(feature = "wasmtime")]
pub use wasmtime_backend::*;

#[cfg(all(feature = "wasmi", not(feature = "wasmtime")))]
mod wasmi_backend;

#[cfg(all(feature = "wasmi", not(feature = "wasmtime")))]
pub use wasmi_backend::*;

#[cfg(not(any(feature = "wasmi", feature = "wasmtime")))]
mod stub_backend;

#[cfg(not(any(feature = "wasmi", feature = "wasmtime")))]
pub use stub_backend::*;

#[cfg(test)]
mod tests;
