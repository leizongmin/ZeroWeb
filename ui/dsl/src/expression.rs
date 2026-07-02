//! 表达式 AST（spec IF-005 `Expression`）。
//!
//! M1 提供 AST 数据模型；parser/typecheck/eval + sandbox 在 M3 落地（spec FR-008）。
//! 表达式无副作用、可缓存、受 EvalContext 权限边界限制（spec 约束 7）。

use compact_str::CompactString;
use zero_ui_core::binding::{StatePath, Value};

/// 受控纯函数 id（白名单注册，禁止任意函数）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PureFunctionId(pub CompactString);

impl PureFunctionId {
    pub fn new(name: &str) -> PureFunctionId {
        PureFunctionId(CompactString::new(name))
    }
}

/// 一元运算符。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// 二元运算符（含算术/比较/布尔/空值合并）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    /// `??` 空值合并。
    NullCoalesce,
}

/// 表达式 AST（spec IF-005）。
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Path(StatePath),
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
    Call {
        function: PureFunctionId,
        args: Vec<Expression>,
    },
    Array(Vec<Expression>),
    Object(Vec<(CompactString, Expression)>),
}

impl Expression {
    /// 字面量快捷构造。
    pub fn literal(value: Value) -> Expression {
        Expression::Literal(value)
    }

    /// 状态路径快捷构造。
    pub fn path(dot_path: &str) -> Expression {
        Expression::Path(StatePath::parse(dot_path))
    }

    /// AST 节点数（用于 M3 资源上限检查）。
    pub fn node_count(&self) -> usize {
        match self {
            Expression::Literal(_) | Expression::Path(_) => 1,
            Expression::Unary { expr, .. } => 1 + expr.node_count(),
            Expression::Binary { left, right, .. } => 1 + left.node_count() + right.node_count(),
            Expression::Conditional {
                condition,
                then_expr,
                else_expr,
            } => 1 + condition.node_count() + then_expr.node_count() + else_expr.node_count(),
            Expression::Call { args, .. } => 1 + args.iter().map(|a| a.node_count()).sum::<usize>(),
            Expression::Array(items) => 1 + items.iter().map(|a| a.node_count()).sum::<usize>(),
            Expression::Object(items) => 1 + items.iter().map(|(_, e)| e.node_count()).sum::<usize>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_node_count() {
        // count + 1 == path('tabs.count') + literal(1)
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::path("tabs.count")),
            right: Box::new(Expression::literal(Value::Int(1))),
        };
        // Binary(1) + Path(1) + Literal(1) = 3。
        assert_eq!(expr.node_count(), 3);
    }

    #[test]
    fn conditional_construct() {
        let cond = Expression::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expression::path("tabs.len")),
            right: Box::new(Expression::literal(Value::Int(0))),
        };
        let c = Expression::Conditional {
            condition: Box::new(cond),
            then_expr: Box::new(Expression::literal(Value::Text("has".into()))),
            else_expr: Box::new(Expression::literal(Value::Text("none".into()))),
        };
        assert!(c.node_count() >= 5);
    }

    #[test]
    fn node_count_for_unary_call_array_object() {
        // Unary: Not(path) → 1 + 1 = 2。
        let unary = Expression::Unary {
            op: UnaryOp::Not,
            expr: Box::new(Expression::path("flag")),
        };
        assert_eq!(unary.node_count(), 2);

        // Call: clamp(0, 1) → 1 + 2 = 3。
        let call = Expression::Call {
            function: PureFunctionId::new("clamp"),
            args: vec![Expression::literal(Value::Int(0)), Expression::literal(Value::Int(1))],
        };
        assert_eq!(call.node_count(), 3);

        // Array: [1, 2] → 1 + 2 = 3。
        let arr = Expression::Array(vec![
            Expression::literal(Value::Int(1)),
            Expression::literal(Value::Int(2)),
        ]);
        assert_eq!(arr.node_count(), 3);

        // Object: {k: true} → 1 + 1 = 2。
        let obj = Expression::Object(vec![(CompactString::new("k"), Expression::literal(Value::Bool(true)))]);
        assert_eq!(obj.node_count(), 2);

        // PureFunctionId 构造与相等。
        assert_eq!(PureFunctionId::new("clamp"), PureFunctionId::new("clamp"));
    }
}
