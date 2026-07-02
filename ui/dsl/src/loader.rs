//! WidgetSpec 加载器（spec IF-005 `WidgetSpecLoader` / `ExpressionEngine`）。
//!
//! M1 只定义接口；YAML 解析 + 表达式 parse/typecheck/eval 在 M3 落地（spec FR-008）。

use crate::diagnostics::DslError;
use crate::expression::Expression;
use hashbrown::HashMap;
use zero_ui_core::binding::{Value, ValueType};
use zero_ui_core::widget::WidgetSpec;

/// YAML/字符串 → WidgetSpec 加载器（spec IF-005）。
pub trait WidgetSpecLoader {
    fn load_str(&self, source: &str) -> Result<WidgetSpec, DslError>;
}

/// 表达式引擎（spec IF-005 `ExpressionEngine`）。
pub trait ExpressionEngine {
    fn parse(&self, source: &str) -> Result<Expression, DslError>;
    fn typecheck(
        &self,
        expr: &Expression,
        schema: &zero_ui_core::binding::BindingSchema,
    ) -> Result<ValueType, DslError>;
    fn eval(&self, expr: &Expression, ctx: &EvalContext) -> Result<Value, DslError>;
}

/// 表达式求值上下文（持有状态快照 + 权限边界，spec 约束 7）。
///
/// `vars` 提供路径根（`$state`/`$props`/`$theme`/`$env` 等）的值快照；
/// 求值无副作用、确定性、可缓存。
#[derive(Debug, Clone)]
pub struct EvalContext {
    /// 是否允许调用纯函数（沙箱可关闭：false 时所有 Call → ForbiddenCapability）。
    pub allow_functions: bool,
    /// AST 最大节点数（防恶意 DSL 消耗 CPU，spec FR-008）。
    pub max_nodes: usize,
    /// 求值最大递归深度（防深度嵌套 AST 栈溢出）。
    pub max_depth: usize,
    /// 集合函数最大累计迭代数（防超长数组消耗 CPU）。
    pub max_iterations: usize,
    /// 路径根 → 值快照（如 `"state" → Value::Object{...}`）。
    pub vars: HashMap<String, Value>,
}

impl Default for EvalContext {
    fn default() -> EvalContext {
        EvalContext {
            allow_functions: true,
            max_nodes: 1024,
            max_depth: 64,
            max_iterations: 10_000,
            vars: HashMap::new(),
        }
    }
}

impl EvalContext {
    /// 注入一个路径根的值快照（builder 风格）。
    pub fn with_var(mut self, root: &str, value: Value) -> EvalContext {
        self.vars.insert(root.to_string(), value);
        self
    }
}

/// M1 占位加载器：YAML 解析在 M3 实现。
pub struct SkeletonLoader;

impl WidgetSpecLoader for SkeletonLoader {
    fn load_str(&self, _source: &str) -> Result<WidgetSpec, DslError> {
        Err(DslError::Parse("YAML loader not implemented until M3".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_loader_defers_to_m3() {
        let loader = SkeletonLoader;
        let err = loader.load_str("component: Button").unwrap_err();
        assert!(matches!(err, DslError::Parse(_)));
    }

    #[test]
    fn eval_context_defaults() {
        let ctx = EvalContext::default();
        assert!(ctx.allow_functions);
        assert!(ctx.max_nodes > 0);
    }
}
