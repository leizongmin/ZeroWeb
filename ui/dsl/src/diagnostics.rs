//! DSL 错误（spec IF-005 错误处理 / FR-008 sandbox）。

use thiserror::Error;

/// DSL 错误。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DslError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("validation error: {0}")]
    Validate(String),
    #[error("typecheck error: {0}")]
    Typecheck(String),
    #[error("unknown function: {0}")]
    UnknownFunction(String),
    /// 访问了禁止能力（文件/网络/进程/时钟/随机/递归/状态写入）。
    #[error("forbidden capability: {0}")]
    ForbiddenCapability(String),
    /// 求值超限（AST 深度/节点数/迭代数）。
    #[error("eval resource limit: {0}")]
    EvalResourceLimit(String),
    /// 任意脚本字段（spec IF-005：禁止任意脚本）。
    #[error("sandbox violation: {0}")]
    SandboxViolation(String),
}
