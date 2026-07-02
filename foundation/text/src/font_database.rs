//! 字体发现与解析（spec IF-008 `FontProvider`）。
//!
//! M1 只定义接口与 `FontMatch` 数据模型；具体 fontdue/fontdb 后端实现由 M2 桥接
//! （从 `crates/render-foundation/src/font` 迁移/复用）。

use crate::diagnostics::TextError;
use crate::font_request::{FontFamily, FontId, FontRequest, FontStretch, FontStyle, FontWeight};
use serde::{Deserialize, Serialize};

/// 字体来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontSource {
    /// 系统安装字体（按路径加载）。
    System,
    /// 内存字体（随应用打包/WebView 注入）。
    Memory,
}

/// 解析后的字体匹配。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontMatch {
    pub id: FontId,
    pub family: FontFamily,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub source: FontSource,
}

/// 字体提供者（spec IF-008 `FontProvider`）。
///
/// - `query`：按请求解析**首选**字体（用于整段主字体）。
/// - `fallback_chain`：为文本中可能存在的缺字符返回 **fallback 序列**（含首选），
///   shaping 时按字符逐个选用能覆盖该字符的字体。
pub trait FontProvider {
    fn query(&self, request: &FontRequest) -> Result<FontMatch, TextError>;
    fn fallback_chain(&self, text: &str, request: &FontRequest) -> Result<Vec<FontMatch>, TextError>;
}
