//! 集成测试模块

mod types_stub;

#[cfg(feature = "wasmi")]
mod advanced;
#[cfg(feature = "wasmi")]
mod basic;
#[cfg(feature = "wasmi")]
mod module_instance;
#[cfg(feature = "wasmi")]
mod review_r3347;
