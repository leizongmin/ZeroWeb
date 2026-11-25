//! # zero-css-parser
//!
//! 自建 CSS 解析器 — tokenizer + parser，支持完整选择器和属性解析。
//!
//! 不依赖任何 MPL 许可的 CSS 解析库（rust-cssparser、lightningcss），完全自建。
//!
//! ## 核心模块
//!
//! - [`tokenizer`] — CSS 词法分析器，将字符流转换为 token 流
//! - [`parser`] — CSS 语法解析器，将 token 流转换为 AST
//! - [`selector`] — 选择器解析和匹配
//! - [`values`] — CSS 属性值类型
//! - [`ast`] — CSS AST 数据结构
//! - [`media_query`] — CSS 媒体查询解析与评估

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_imports))]
#![allow(clippy::len_zero)]
#![allow(unused_comparisons)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::approx_constant)]
#![allow(clippy::bool_comparison)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::single_match)]
#![allow(clippy::comparison_to_empty)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::unnecessary_map_or)]

/// k113 css-mixins `@mixin`/`@apply` 解析期展开 face 指纹（css-mixins-1
/// §mixin-rule/§apply-rule：双程注册表＋@result 解包＋局部/参数卫生改名；
/// DCE 防剥同 k112 形——#[used] static 字节数组物化 bin 指纹，供 A/B 判代
/// provenance 法证 strings 直证）。
#[used]
#[allow(dead_code)]
static KC5R113_MIXIN_APPLY_FACE: [u8; 27] = *b"ZW-KC5R113-MIXIN-APPLY-FACE";

/// k114 css-mixins `@function` 定义文法 face 指纹（css-mixins-1 §2.1.1
/// `<function-parameter-list>` 严格门：类型分量白名单＋default 类型核验＋
/// returns 文法；防剥同 k113 形——#[used] static 字节数组物化 bin 指纹，
/// 供 A/B 判代 provenance 法证 strings 直证）。
#[used]
#[allow(dead_code)]
static KC5R114_FN_PRELUDE_FACE: [u8; 26] = *b"ZW-KC5R114-FN-PRELUDE-FACE";

/// k115 css-mixins `@function` CSSOM 结构化面 face 指纹（css-mixins-1 §3
/// CSSFunctionRule/CSSFunctionDeclarations/CSSFunctionDescriptors 接口面＋
/// canonical cssText 序列化域——escape_css_ident/type() 剥壳/body 折叠；
/// 防剥同 k114 形）。
#[used]
#[allow(dead_code)]
static KC5R115_FN_CSSOM_FACE: [u8; 24] = *b"ZW-KC5R115-FN-CSSOM-FACE";

/// k116 刀面指纹（mixin 跨表/导入聚合）——#[used] 防 DCE 剥 bin 指纹。
#[used]
static KC5R116_MIXIN_DOCSHEET_FACE: [u8; 27] = *b"ZW-KC5R116-MX-DOCSHEET-FACE";

pub mod ast;
pub mod media_query;
pub mod parser;
pub mod selector;
pub mod serialize;
pub mod supports_condition;
pub mod tokenizer;
pub mod values;

pub use ast::*;
pub use media_query::*;
pub use parser::MixinDefs;
pub use parser::Parser;
pub use selector::*;
pub use supports_condition::*;
pub use tokenizer::{
    CrossOriginValue, Spanned, Token, Tokenizer, UrlRequestModifier, canonical_url_modifier_list, escape_css_ident,
    line_column_from_offset,
};
pub use values::dashed_function::dashed_function_value_valid;
pub use values::parse_extended_visual::{
    canonicalize_url_modifier_value, url_modifier_declaration_valid, value_has_url_request_modifiers,
};

#[cfg(test)]
mod tests;
