//! # zero-script-sandbox
//!
//! 扩展/用户脚本引擎（V8/QuickJS feature gate）。
//!
//! 提供 JavaScript 脚本执行沙箱，用于扩展脚本、用户脚本和自动化脚本。
//! 通过 feature gate 选择后端引擎。

#![warn(missing_docs)]

/// 脚本沙箱
pub struct ScriptSandbox {
    // TODO: 在 M6+ 里程碑中实现
}

#[cfg(test)]
mod tests {
    #[test]
    fn placeholder() {
        // M6+ 里程碑将在此处实现脚本沙箱测试
    }
}
