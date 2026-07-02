//! # zero-ui-dsl
//!
//! 声明式 DSL（spec §8.4.1 `zero-ui-dsl` / FR-008 / IF-005）。
//!
//! M1 skeleton：表达式 AST（[`expression`]）、DSL 错误（[`diagnostics`]）、
//! WidgetSpec 加载器与表达式引擎 trait（[`loader`]）。
//!
//! 完整 YAML→WidgetSpec 解析、表达式 parse/validate/typecheck/eval、sandbox negative tests、
//! 资源上限在 M3 落地。

pub mod diagnostics;
pub mod expression;
pub mod loader;

pub use diagnostics::DslError;
pub use expression::{BinaryOp, Expression, PureFunctionId, UnaryOp};
pub use loader::{EvalContext, ExpressionEngine, SkeletonLoader, WidgetSpecLoader};
