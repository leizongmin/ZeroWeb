//! # zero-ui-dsl
//!
//! 声明式 DSL（spec §8.4.1 `zero-ui-dsl` / FR-008 / IF-005）。
//!
//! - **M3 phase-1** 已落地：表达式引擎 [`engine`]（parse/typecheck/eval 三阶段 + sandbox + 资源上限）。
//! - **M3 phase-2** 已落地：受限 YAML 解析器 [`yaml`] + [`loader::YamlLoader`]（YAML→`WidgetSpec`，
//!   递归 component/id/props/bindings/actions/control/children，strict 模式加载时校验表达式语法）。
//! - **DC-10 桥** 已落地：[`i18n_bridge`]（DSL `i18n:` 对象 → `LocalizedText`，参数表达式求值）。
//!
//! 剩余：map/filter 嵌套路径投影（follow-up）、DSL command·route·overlay·asset 引用、
//! counter/form/browser-shell-demo 示例（DC-14）。

pub mod diagnostics;
pub mod engine;
pub mod expression;
pub mod i18n_bridge;
pub mod loader;
pub mod yaml;

pub use diagnostics::DslError;
pub use engine::Engine;
pub use expression::{BinaryOp, Expression, PureFunctionId, UnaryOp};
pub use i18n_bridge::{i18n_value_to_message, is_i18n_object};
pub use loader::{EvalContext, ExpressionEngine, WidgetSpecLoader, YamlLoader};
pub use yaml::YamlValue;
