//! # zero-ui-dsl
//!
//! 声明式 DSL（spec §8.4.1 `zero-ui-dsl` / FR-008 / IF-005）。
//!
//! M3 phase-1 已落地：表达式引擎 [`engine`]（parse/typecheck/eval 三阶段 + sandbox + 资源上限）。
//! 剩余：完整 YAML→WidgetSpec 解析（`WidgetSpecLoader`）、map/filter（需 Lambda，spec 枚举 TBD）、
//! DSL `i18n:` message id / responsive branch / command/route 引用在 M3 phase-2+。

pub mod diagnostics;
pub mod engine;
pub mod expression;
pub mod loader;

pub use diagnostics::DslError;
pub use engine::Engine;
pub use expression::{BinaryOp, Expression, PureFunctionId, UnaryOp};
pub use loader::{EvalContext, ExpressionEngine, SkeletonLoader, WidgetSpecLoader};
