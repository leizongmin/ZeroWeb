//! 文本错误与诊断（spec IF-008 错误处理）。

use thiserror::Error;

/// 文本基础层错误。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TextError {
    /// 请求的字体族全部缺失，fallback chain 也无法满足。
    #[error("font not found for request")]
    FontNotFound,
    /// shaping 失败（无效字体表/不支持脚本）。
    #[error("shaping failed")]
    ShapeFailed,
    /// 测量失败。
    #[error("measure failed")]
    MeasureFailed,
    /// glyph atlas 已满且无法驱逐。
    #[error("glyph atlas full")]
    AtlasFull,
    /// 非法请求（零字号、空族等）。
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

/// 诊断种类（非致命，不阻断渲染）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// 字体缺失，使用了 fallback。
    FallbackUsed,
    /// 字体中无对应 glyph（tofu）。
    MissingGlyph,
    /// 不支持的 font feature / variation axis 被忽略。
    UnsupportedFeatureIgnored,
}

/// 文本诊断（供调用方收集、上报）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextDiagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
}
