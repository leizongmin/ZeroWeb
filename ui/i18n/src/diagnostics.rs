//! i18n 错误与诊断（spec IF-007 错误处理）。

use thiserror::Error;

/// i18n 硬错误（导致 resolve 失败）。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum I18nError {
    /// 模板引用了未提供的参数。
    #[error("missing parameter: {0}")]
    MissingParam(String),
    /// 参数类型与模板期望不符。
    #[error("invalid param type for: {0}")]
    InvalidParamType(String),
    /// catalog 加载失败（非法 locale/direction/格式）。
    #[error("catalog load error: {0}")]
    CatalogLoad(String),
}

/// 诊断种类（非致命）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// fallback chain 中无任何 locale 命中该 key。
    MissingKey,
    /// 命中了非首选 locale（fallback 生效）。
    FallbackUsed,
    /// plural form 缺失，用了默认 fallback form。
    PluralFallback,
}

/// i18n 诊断。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct I18nDiagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
}
