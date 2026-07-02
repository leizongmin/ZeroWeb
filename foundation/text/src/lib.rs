//! # zero-text-foundation
//!
//! 共享文本/字体基础层（浏览器无关）。UI SDK（`ui/render`、`ui/widgets::TextInput`）与
//! WebView（`zero-webview`/`zero-engine`）通过本 crate 得到一致的 font fallback、shaping
//! 与 glyph cache（spec FR-014 / IF-008 / DC-11）。
//!
//! 覆盖 spec §8.4.1 `zero-text-foundation` 全部模块：
//! [`font_request`]、[`font_database`]、[`font_fallback`]、[`shaping`]、[`text_measure`]、
//! [`glyph_cache`]、[`glyph_atlas`]、[`bidi`]、[`line_break`]、[`grapheme`]、[`text_blob`]、
//! [`diagnostics`]。
//!
//! M1 提供接口与最小实现/纯逻辑；M2 起 [`backend`] 提供 fontdue + rustybuzz 真实后端。
//! 本 crate 不依赖任何 UI/浏览器业务 crate。

pub mod backend;
pub mod bidi;
pub mod diagnostics;
pub mod font_database;
pub mod font_fallback;
pub mod font_request;
pub mod glyph_atlas;
pub mod glyph_cache;
pub mod grapheme;
pub mod line_break;
pub mod shaping;
pub mod text_blob;
pub mod text_measure;

// 顶层再导出常用接口，方便调用方 `use zero_text_foundation::{FontProvider, TextShaper, ...}`。
pub use backend::FontdueBackend;
pub use diagnostics::TextError;
pub use font_database::{FontMatch, FontProvider, FontSource};
pub use font_request::{
    FontFamily, FontId, FontRequest, FontStretch, FontStyle, FontWeight, LocaleId, Script, TextDirection,
};
pub use glyph_atlas::{AtlasRect, GlyphAtlasEntry};
pub use glyph_cache::{GlyphCache, GlyphKey, InMemoryGlyphCache};
pub use shaping::{GlyphRun, PositionedGlyph, ShapeInput, ShapedText, TextShaper};
pub use text_measure::{TextMeasureInput, TextMeasurer, TextMetrics};
