//! # zero-ui-dsl
//!
//! 声明式 DSL（spec §8.4.1 `zero-ui-dsl` / FR-008 / IF-005）。
//!
//! - **M3 phase-1** 已落地：表达式引擎 [`engine`]（parse/typecheck/eval 三阶段 + sandbox + 资源上限）。
//! - **M3 phase-2** 已落地：受限 YAML 解析器 [`yaml`] + [`loader::YamlLoader`]（YAML→`WidgetSpec`，
//!   递归 component/id/props/bindings/actions/control/children，strict 模式加载时校验表达式语法）。
//! - **DC-10 桥** 已落地：[`i18n_bridge`]（DSL `i18n:` 对象 → `LocalizedText`，参数表达式求值）。
//! - **DC-6 phase-5** 已落地：action 简写（`command`/`navigate`/`open_overlay`/`close_overlay`，
//!   [`loader`]）+ [`asset_bridge`]（DSL `asset:` 对象 → `AssetId`）。
//! - **DC-6 列表渲染** 已落地：[`for_each::materialize_for_each`]（`for_each` 节点 → N 个具体子节点，
//!   item 作用域求值 bindings/visible_when/enabled_when，稳定 id，迭代受 `max_iterations` 约束）。
//!
//! DC-6 全部 phases（引擎/YAML loader/map+filter 嵌套路径投影/`for_each`/responsive/action 简写）
//! 已落地；counter/form/browser-shell-demo 示例已可构建运行（DC-14）。
//! 唯一不做：谓词过滤 `filter($items, field>x)`（需 lambda，明确超受控计算层范围）。

pub mod asset_bridge;
pub mod diagnostics;
pub mod engine;
pub mod expression;
pub mod for_each;
pub mod i18n_bridge;
pub mod loader;
pub mod yaml;

pub use asset_bridge::{asset_id_of, is_asset_object};
pub use diagnostics::DslError;
pub use engine::Engine;
pub use expression::{BinaryOp, Expression, PureFunctionId, UnaryOp};
pub use for_each::materialize_for_each;
pub use i18n_bridge::{i18n_value_to_message, is_i18n_object};
pub use loader::{EvalContext, ExpressionEngine, WidgetSpecLoader, YamlLoader};
pub use yaml::YamlValue;
