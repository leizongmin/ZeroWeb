//! Paint 文本整形的受限接入与旧逐字符回退。

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::sync::Arc;
use unicode_bidi::{BidiClass, bidi_class};
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::{InlineFormattingContext, LayoutBox, TextFragmentSource};
use zero_render_foundation::font::{OpenTypeFeature, ShapedGlyph, TextDirection};
use zero_render_foundation::primitive::{FontId, GlyphSource};
use zero_style_system::ComputedStyle;

pub(super) struct ResolvedTextNodeFonts {
    pub(super) primary: HashMap<NodeId, zero_render_foundation::primitive::FontId>,
    pub(super) shaping: HashMap<NodeId, Vec<u32>>,
    pub(super) italic: HashMap<NodeId, bool>,
}

pub(super) fn collect_atomic_inline_sizes(
    layout: &LayoutBox,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
) -> HashMap<NodeId, (f32, f32)> {
    fn visit(
        layout: &LayoutBox,
        styles: Option<&HashMap<NodeId, ComputedStyle>>,
        sizes: &mut HashMap<NodeId, (f32, f32)>,
    ) {
        if let Some(node_id) = layout.node_id
            && layout.width > 0.0
            && layout.height > 0.0
            && styles.and_then(|styles| styles.get(&node_id)).is_some_and(|style| {
                matches!(
                    style.display,
                    DisplayValue::InlineBlock
                        | DisplayValue::InlineFlex
                        | DisplayValue::InlineGrid
                        | DisplayValue::InlineTable
                )
            })
        {
            sizes.insert(node_id, (layout.width, layout.height));
        }
        for child in &layout.children {
            visit(child, styles, sizes);
        }
    }

    let mut sizes = HashMap::new();
    visit(layout, styles, &mut sizes);
    sizes
}

impl super::super::Painter {
    /// 根据 CSS family/weight/style/stretch 解析主 face。
    pub(crate) fn resolve_font_id(
        &self,
        font_family: &[String],
        font_weight: &zero_css_parser::values::FontWeightValue,
        font_style: &zero_css_parser::values::types::FontStyleValue,
        font_stretch: f32,
    ) -> (FontId, bool) {
        use zero_css_parser::values::FontWeightValue;
        use zero_css_parser::values::types::FontStyleValue;

        let want_bold = matches!(font_weight, FontWeightValue::Bold | FontWeightValue::Bolder)
            || matches!(font_weight, FontWeightValue::Absolute(weight) if *weight >= 600);
        let want_italic = matches!(font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
        // OPTIMIZATION（2026-08-19）：结果 memoize（见 Painter::font_id_cache 注释）。
        // 键用「原始 family 列表 + bool×2 + stretch 位串」——完整保留解析输入，
        // 不做语义归并（避免与下方解析逻辑产生等价类偏差）。
        let oblique_angle = match font_style {
            FontStyleValue::Oblique(Some(deg)) => deg.to_bits(),
            _ => 0,
        };
        let cache_key = (
            font_family.to_vec(),
            want_bold as u8,
            want_italic as u8,
            (font_stretch.to_bits(), oblique_angle),
        );
        if let Some(hit) = self.font_id_cache.borrow().get(&cache_key) {
            return (FontId(hit.0), hit.1);
        }
        let resolved = self.resolve_font_id_uncached(font_family, want_bold, want_italic, font_stretch);
        self.font_id_cache
            .borrow_mut()
            .insert(cache_key, (resolved.0.0, resolved.1));
        resolved
    }

    /// `resolve_font_id` 的未缓存实现（原逻辑原样保留）。
    fn resolve_font_id_uncached(
        &self,
        font_family: &[String],
        want_bold: bool,
        want_italic: bool,
        font_stretch: f32,
    ) -> (FontId, bool) {
        const GENERIC_FAMILIES: &[&str] = &[
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "ui-serif",
            "ui-sans-serif",
            "ui-monospace",
            "ui-rounded",
            "emoji",
            "math",
            "fangsong",
        ];
        for family in font_family {
            let is_quoted = family.starts_with('"') || family.starts_with('\'');
            let name = family.trim_matches('"').trim_matches('\'');
            // https://drafts.csswg.org/css-fonts-4/#family-name-value
            if is_quoted
                && GENERIC_FAMILIES
                    .iter()
                    .any(|generic| generic.eq_ignore_ascii_case(name))
            {
                continue;
            }
            if let Some((id, resolved_italic)) = zero_render_foundation::font::resolve_font_face(
                &self.font_resolver_lower,
                &name.to_ascii_lowercase(),
                want_bold,
                want_italic,
                font_stretch,
            ) {
                return (FontId(id), resolved_italic);
            }
        }
        if let Some((id, resolved_italic)) = zero_render_foundation::font::resolve_font_face(
            &self.font_resolver_lower,
            "sans-serif",
            want_bold,
            want_italic,
            font_stretch,
        ) {
            return (FontId(id), resolved_italic);
        }
        (FontId(0), false)
    }

    pub(crate) fn resolve_style_font_id(&self, font_family: &[String], style: &ComputedStyle) -> (FontId, bool) {
        self.resolve_font_id(font_family, &style.font_weight, &style.font_style, style.font_stretch)
    }

    /// 按 CSS `font-family` 顺序解析可用 face ID，供 shaping fallback 使用。
    pub(crate) fn resolve_font_ids(
        &self,
        font_family: &[String],
        font_weight: &zero_css_parser::values::FontWeightValue,
        font_style: &zero_css_parser::values::types::FontStyleValue,
        font_stretch: f32,
    ) -> Vec<u32> {
        zero_layout_engine::font_resolution::resolve_font_ids_for_style(
            &self.font_resolver,
            font_family,
            font_weight,
            font_style,
            font_stretch,
        )
    }

    pub(crate) fn resolve_style_font_ids(&self, font_family: &[String], style: &ComputedStyle) -> Vec<u32> {
        self.resolve_font_ids(font_family, &style.font_weight, &style.font_style, style.font_stretch)
    }

    pub(super) fn resolve_text_node_fonts(
        &self,
        box_node: &zero_layout_engine::LayoutBox,
        style: &ComputedStyle,
    ) -> ResolvedTextNodeFonts {
        let mut primary = HashMap::with_capacity(box_node.text_node_font_families.len());
        let mut shaping = HashMap::with_capacity(box_node.text_node_font_families.len());
        let mut italic = HashMap::with_capacity(box_node.text_node_font_families.len());
        for (&node_id, families) in &box_node.text_node_font_families {
            let (font_id, resolved_italic) = self.resolve_style_font_id(families, style);
            primary.insert(node_id, font_id);
            shaping.insert(node_id, self.resolve_style_font_ids(families, style));
            italic.insert(node_id, resolved_italic);
        }
        ResolvedTextNodeFonts {
            primary,
            shaping,
            italic,
        }
    }

    pub(super) fn fragment_shaping_font_ids(
        &self,
        owner_style: Option<&ComputedStyle>,
        stored_font_ids: Option<&[u32]>,
        fragment_font_id: zero_render_foundation::primitive::FontId,
    ) -> Vec<u32> {
        let mut font_ids = owner_style.map_or_else(
            || stored_font_ids.map_or_else(|| vec![fragment_font_id.0], <[u32]>::to_vec),
            |owner| self.resolve_style_font_ids(&owner.font_family, owner),
        );
        if font_ids.first() != Some(&fragment_font_id.0) {
            font_ids.retain(|font_id| *font_id != fragment_font_id.0);
            font_ids.insert(0, fragment_font_id.0);
        }
        // R3243-F 曾默认开（`!= "0"`）：fallback 多 face shaping 每帧全量重排
        // （perf-gate morning paint 回归）；改回显式 opt-in（R3243-F 之前语义）。
        if !preserve_font_fallback_faces(&font_ids, &self.generic_font_ids) {
            font_ids.truncate(1);
        }
        font_ids
    }
}

fn preserve_font_fallback_faces(font_ids: &[u32], generic_font_ids: &HashSet<u32>) -> bool {
    // https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
    // OPTIMIZATION: keep R3243's single-face path for generic/system text, but
    // preserve explicit author faces so missing glyphs can continue down the
    // declared family list without reopening the CJK product perf regression.
    preserve_font_fallback_faces_with_policy(
        shaped_fallback_enabled(),
        author_font_fallback_enabled(),
        font_ids.len() > 1 && font_ids.iter().all(|font_id| !generic_font_ids.contains(font_id)),
    )
}

fn preserve_font_fallback_faces_with_policy(
    global_fallback: bool,
    author_fallback: bool,
    all_faces_are_author_faces: bool,
) -> bool {
    global_fallback || author_fallback && all_faces_are_author_faces
}

pub(super) fn fragment_font_size_adjustment(
    owner_style: Option<&ComputedStyle>,
    stored: Option<&zero_style_system::FontSizeAdjustValue>,
    fallback: &zero_style_system::FontSizeAdjustValue,
    author_face: bool,
) -> zero_render_foundation::font::FontSizeAdjustment {
    let adjustment = crate::text_metrics::font_size_adjustment(
        owner_style
            .map(|style| &style.font_size_adjust)
            .or(stored)
            .unwrap_or(fallback),
    );
    if preserve_font_fallback_faces_with_policy(shaped_fallback_enabled(), author_font_fallback_enabled(), author_face)
    {
        adjustment
    } else {
        zero_render_foundation::font::FontSizeAdjustment::None
    }
}

pub(super) fn font_size_adjustment_active(adjustment: zero_render_foundation::font::FontSizeAdjustment) -> bool {
    !matches!(adjustment, zero_render_foundation::font::FontSizeAdjustment::None)
}

fn shaped_text_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_TEXT").as_deref() != Ok("0"))
}

fn indexed_glyph_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_GLYPH_INDEX").as_deref() != Ok("0"))
}

fn shaped_positioning_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_POSITIONING").as_deref() == Ok("1"))
}

fn shaped_advance_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| shaped_positioning_enabled() || std::env::var("ZW_SHAPED_ADVANCE").as_deref() != Ok("0"))
}

fn shaped_offsets_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| shaped_positioning_enabled() || std::env::var("ZW_SHAPED_OFFSETS").as_deref() != Ok("0"))
}

fn shaped_complex_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_COMPLEX").as_deref() != Ok("0"))
}

fn shaped_rtl_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_RTL").as_deref() != Ok("0"))
}

pub(super) fn shaped_uba_rtl_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_UBA_RTL").as_deref() != Ok("0"))
}

fn shaped_advance_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_ADVANCE_TRACE").as_deref() == Ok("1"))
}

fn shaped_generic_paint_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_GENERIC_PAINT").as_deref() != Ok("0"))
}

fn shaped_fallback_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    crate::text_metrics::paint_env_value(crate::text_metrics::paint_env_snapshot_enabled(), &ENABLED, || {
        std::env::var("ZW_SHAPED_FALLBACK").as_deref() == Ok("1")
    })
}

fn author_font_fallback_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    crate::text_metrics::paint_env_value(crate::text_metrics::paint_env_snapshot_enabled(), &ENABLED, || {
        std::env::var("ZW_AUTHOR_FONT_FALLBACK").as_deref() != Ok("0")
    })
}

fn shaped_layout_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    crate::text_metrics::paint_env_value(crate::text_metrics::paint_env_snapshot_enabled(), &ENABLED, || {
        std::env::var("ZW_SHAPED_LAYOUT").as_deref() == Ok("1")
    })
}

fn author_shaped_layout_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    crate::text_metrics::paint_env_value(crate::text_metrics::paint_env_snapshot_enabled(), &ENABLED, || {
        std::env::var("ZW_AUTHOR_SHAPED_LAYOUT").as_deref() != Ok("0")
    })
}

fn adjusted_generic_advance_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    crate::text_metrics::paint_env_value(crate::text_metrics::paint_env_snapshot_enabled(), &ENABLED, || {
        std::env::var("ZW_SHAPED_ADJUSTED_GENERIC_ADVANCE").as_deref() != Ok("0")
    })
}

fn shaped_advance_policy(generic_font: bool, size_adjusted: bool, adjusted_generic_enabled: bool) -> bool {
    !generic_font || size_adjusted && adjusted_generic_enabled
}

pub(super) fn fragment_shaped_advance_eligible(
    generic_font: bool,
    size_adjust: zero_render_foundation::font::FontSizeAdjustment,
) -> bool {
    shaped_advance_policy(
        generic_font,
        font_size_adjustment_active(size_adjust),
        adjusted_generic_advance_enabled(),
    )
}

pub(super) fn configure_paint_ifc_advance<S: BuildHasher>(
    context: InlineFormattingContext,
    doc: &Document,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
    text_node_font_ids: &HashMap<NodeId, FontId>,
    text_node_shaping_font_ids: &HashMap<NodeId, Vec<u32>>,
    text_node_font_size_adjust: &HashMap<NodeId, zero_style_system::FontSizeAdjustValue, S>,
    generic_font_ids: &HashSet<u32>,
) -> InlineFormattingContext {
    let shaped_layout = shaped_layout_enabled();
    let author_shaped_layout = author_shaped_layout_enabled();
    if std::env::var("ZW_SHAPED_TEXT").as_deref() == Ok("0") || !shaped_layout && !author_shaped_layout {
        return context;
    }
    // Keep generic/system text on the established estimate path; only explicit
    // author faces need layout and paint to share their resolved face advances.
    let author_only = author_shaped_layout && !shaped_layout;
    let mut primary_ids: HashMap<NodeId, u32> = text_node_font_ids
        .iter()
        .filter_map(|(&text_node, &font_id)| {
            let parent_id = doc.parent_node(text_node)?;
            let owner = styles?.get(&parent_id)?;
            const GENERIC_FAMILIES: [&str; 6] = ["sans-serif", "serif", "monospace", "cursive", "fantasy", "system-ui"];
            let no_spacing = matches!(owner.letter_spacing, LengthValue::Px(v) if v == 0.0)
                && matches!(owner.word_spacing, LengthValue::Px(v) if v == 0.0);
            let declared_generic = owner.font_family.iter().any(|family| {
                GENERIC_FAMILIES
                    .iter()
                    .any(|generic| family.eq_ignore_ascii_case(generic))
            });
            let eligible = matches!(owner.direction, zero_style_system::DirectionValue::Ltr)
                && matches!(owner.writing_mode, zero_style_system::WritingModeValue::HorizontalTb)
                && no_spacing
                && declared_generic
                && generic_font_ids.contains(&font_id.0)
                && !owner
                    .font_family
                    .iter()
                    .any(|family| family.trim_matches('"').eq_ignore_ascii_case("Ahem"))
                && !doc.get(parent_id).is_some_and(
                    |node| matches!(&node.kind, NodeKind::Element(element) if element.local_name() == "ruby"),
                );
            eligible.then_some((parent_id, font_id.0))
        })
        .collect();
    for (&node_id, size_adjust) in text_node_font_size_adjust {
        if matches!(size_adjust, zero_style_system::FontSizeAdjustValue::None) {
            continue;
        }
        let Some(&font_id) = text_node_font_ids.get(&node_id) else {
            continue;
        };
        let owner_id = if doc
            .get(node_id)
            .is_some_and(|node| matches!(node.kind, NodeKind::Element(_)))
        {
            node_id
        } else if let Some(parent_id) = doc.parent_node(node_id) {
            parent_id
        } else {
            continue;
        };
        primary_ids.entry(owner_id).or_insert(font_id.0);
    }
    if author_only {
        primary_ids.clear();
    }
    let mut shaping_ids = if !shaped_fallback_enabled() {
        text_node_shaping_font_ids.clone()
    } else {
        HashMap::new()
    };
    if author_only {
        shaping_ids.retain(|_, font_ids| {
            !font_ids.is_empty() && font_ids.iter().all(|font_id| !generic_font_ids.contains(font_id))
        });
    }
    let mut size_adjust: HashMap<NodeId, zero_style_system::FontSizeAdjustValue> = if !shaped_fallback_enabled() {
        text_node_font_size_adjust
            .iter()
            .map(|(&node_id, &value)| (node_id, value))
            .collect()
    } else {
        HashMap::new()
    };
    if author_only {
        size_adjust.retain(|node_id, _| shaping_ids.contains_key(node_id));
    }
    context
        .with_font_id_overrides(std::rc::Rc::new(primary_ids))
        .with_font_ids_overrides(std::rc::Rc::new(shaping_ids))
        .with_font_size_adjust_overrides(std::rc::Rc::new(size_adjust))
        .with_advance_source(std::rc::Rc::new(crate::text_metrics::ShapedAdvanceSource::new(
            generic_font_ids.clone(),
        )))
}

/// 同一 shaping 输入的三种 advance，用于与 fragment/paint 宽度对账。
pub(super) struct FragmentAdvanceTrace {
    pub(super) layout_estimate: f32,
    pub(super) unshaped: f32,
    pub(super) shaped: f32,
    pub(super) resolved_font_ids: Vec<u32>,
    pub(super) resolved_font_sizes: Vec<f32>,
    pub(super) size_adjust: zero_render_foundation::font::FontSizeAdjustment,
}

pub(super) struct FragmentPaintWidths {
    pub(super) fragment: f32,
    pub(super) legacy: f32,
    pub(super) consumed: f32,
}

impl FragmentAdvanceTrace {
    pub(super) fn emit(self, path: &'static str, font_id: u32, font_size: f32, text: &str, paint: FragmentPaintWidths) {
        tracing::info!(
            target: "zero_engine::shaped_advance",
            path,
            font_id,
            font_size,
            text = ?text,
            layout_estimate = self.layout_estimate,
            unshaped = self.unshaped,
            shaped = self.shaped,
            resolved_font_ids = ?self.resolved_font_ids,
            resolved_font_sizes = ?self.resolved_font_sizes,
            size_adjust = ?self.size_adjust,
            fragment_width = paint.fragment,
            legacy_paint = paint.legacy,
            paint_consumed = paint.consumed,
            "ZW_SHAPED_ADVANCE_TRACE"
        );
    }
}

/// Paint 循环消费的单个字形。
pub(super) struct FragmentGlyph {
    pub(super) code_point: char,
    pub(super) font_id: Option<zero_render_foundation::primitive::FontId>,
    pub(super) font_size: Option<f32>,
    pub(super) font_glyph_index: Option<u16>,
    pub(super) source: Option<GlyphSource>,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
    pub(super) advance_x: Option<f32>,
}

/// 整形结果或零分配的旧逐字符迭代器。
pub(super) enum FragmentGlyphs<'a> {
    Shaped {
        glyphs: std::vec::IntoIter<ShapedGlyph>,
        sources: std::vec::IntoIter<Option<GlyphSource>>,
        indexed: bool,
        advanced: bool,
        offset: bool,
    },
    Legacy(std::str::Chars<'a>),
}

/// 可安全交给单方向 shaper 的逻辑文本片段。
pub(super) struct LogicalFragmentSource<'a> {
    text: &'a str,
    source_text: Arc<str>,
    source_start: u32,
}

pub(super) fn fragment_shape_direction(
    source: Option<&TextFragmentSource>,
    style_direction: TextDirection,
    uba_rtl_enabled: bool,
) -> TextDirection {
    // https://www.w3.org/TR/css-writing-modes-3/#bidi-algo
    if uba_rtl_enabled && source.and_then(TextFragmentSource::uniform_resolved_rtl) == Some(true) {
        TextDirection::RightToLeft
    } else {
        style_direction
    }
}

/// 从布局片段恢复 RTL logical shaping 输入。
pub(super) fn logical_fragment_source(
    source: Option<&TextFragmentSource>,
    direction: TextDirection,
    text_transform_none: bool,
) -> Option<LogicalFragmentSource<'_>> {
    if direction != TextDirection::RightToLeft || !text_transform_none {
        return None;
    }
    let source = source?;
    let range = source.logical_range()?;
    let text = source.text.get(range.clone())?;
    if !single_rtl_script(text) {
        return None;
    }
    Some(LogicalFragmentSource {
        text,
        source_text: source.text.clone(),
        source_start: u32::try_from(range.start).ok()?,
    })
}

fn single_rtl_script(text: &str) -> bool {
    let mut has_rtl = false;
    for ch in text.chars() {
        match bidi_class(ch) {
            BidiClass::R | BidiClass::AL => has_rtl = true,
            BidiClass::L
            | BidiClass::EN
            | BidiClass::AN
            | BidiClass::LRE
            | BidiClass::RLE
            | BidiClass::LRO
            | BidiClass::RLO
            | BidiClass::LRI
            | BidiClass::RLI
            | BidiClass::FSI
            | BidiClass::PDI
            | BidiClass::PDF => return false,
            _ => {}
        }
    }
    has_rtl
}

fn set_feature(features: &mut Vec<OpenTypeFeature>, tag: [u8; 4], value: u32) {
    if let Some(feature) = features.iter_mut().find(|feature| feature.tag == tag) {
        feature.value = value;
    } else {
        features.push(OpenTypeFeature::new(tag, value));
    }
}

/// 按 CSS Fonts feature precedence 生成元素级 caller overrides。
///
/// Face defaults 由 FontLoader 在更低优先级合并；此处顺序为
/// font-variant → letter-spacing → font-feature-settings。
/// https://drafts.csswg.org/css-fonts-4/#feature-precedence
pub(super) fn style_open_type_features(style: &zero_style_system::ComputedStyle) -> Vec<OpenTypeFeature> {
    let mut features = Vec::new();
    // https://drafts.csswg.org/css-fonts-4/#font-kerning-prop
    use zero_style_system::FontKerningValue;
    match style.font_kerning {
        FontKerningValue::Auto => {}
        FontKerningValue::Normal => {
            let tag = if style.writing_mode.is_vertical_block_flow() {
                *b"vkrn"
            } else {
                *b"kern"
            };
            set_feature(&mut features, tag, 1);
        }
        FontKerningValue::None => {
            set_feature(&mut features, *b"kern", 0);
            set_feature(&mut features, *b"vkrn", 0);
        }
    }

    let ligatures = style.font_variant_ligatures;
    if let Some(enabled) = ligatures.common {
        set_feature(&mut features, *b"liga", enabled as u32);
        set_feature(&mut features, *b"clig", enabled as u32);
    }
    if let Some(enabled) = ligatures.discretionary {
        set_feature(&mut features, *b"dlig", enabled as u32);
    }
    if let Some(enabled) = ligatures.historical {
        set_feature(&mut features, *b"hlig", enabled as u32);
    }
    if let Some(enabled) = ligatures.contextual {
        set_feature(&mut features, *b"calt", enabled as u32);
    }

    // https://drafts.csswg.org/css-fonts-4/#feature-precedence
    // Only non-zero letter-spacing suppresses ligatures; explicit `0em` does not.
    let has_nonzero_spacing = match style.letter_spacing {
        zero_style_system::LengthValue::Px(v) => v.abs() > f64::EPSILON,
        _ => !style.letter_spacing_normal,
    };
    if has_nonzero_spacing {
        set_feature(&mut features, *b"liga", 0);
        set_feature(&mut features, *b"clig", 0);
    }

    // https://drafts.csswg.org/css-fonts-4/#font-variant-numeric-prop
    use zero_style_system::FontVariantNumericValue;
    match &style.font_variant_numeric {
        FontVariantNumericValue::Normal => {}
        FontVariantNumericValue::Ordinal => set_feature(&mut features, *b"ordn", 1),
        FontVariantNumericValue::SlashedZero => set_feature(&mut features, *b"zero", 1),
        FontVariantNumericValue::LiningNums => set_feature(&mut features, *b"lnum", 1),
        FontVariantNumericValue::OldstyleNums => set_feature(&mut features, *b"onum", 1),
        FontVariantNumericValue::ProportionalNums => set_feature(&mut features, *b"pnum", 1),
        FontVariantNumericValue::TabularNums => set_feature(&mut features, *b"tnum", 1),
        FontVariantNumericValue::DiagonalFractions => set_feature(&mut features, *b"frac", 1),
        FontVariantNumericValue::StackedFractions => set_feature(&mut features, *b"afrc", 1),
    }

    // https://drafts.csswg.org/css-fonts-4/#font-variant-caps-prop
    use zero_style_system::FontVariantCapsValue;
    match style.font_variant_caps {
        FontVariantCapsValue::Normal => {}
        FontVariantCapsValue::SmallCaps => set_feature(&mut features, *b"smcp", 1),
        FontVariantCapsValue::AllSmallCaps => {
            set_feature(&mut features, *b"smcp", 1);
            set_feature(&mut features, *b"c2sc", 1);
        }
        FontVariantCapsValue::PetiteCaps => set_feature(&mut features, *b"pcap", 1),
        FontVariantCapsValue::AllPetiteCaps => {
            set_feature(&mut features, *b"pcap", 1);
            set_feature(&mut features, *b"c2pc", 1);
        }
        FontVariantCapsValue::Unicase => set_feature(&mut features, *b"unic", 1),
        FontVariantCapsValue::TitlingCaps => set_feature(&mut features, *b"titl", 1),
    }

    // https://drafts.csswg.org/css-fonts-4/#font-variant-east-asian-prop
    use zero_style_system::FontVariantEastAsianValue;
    match style.font_variant_east_asian {
        FontVariantEastAsianValue::Normal => {}
        FontVariantEastAsianValue::Jis78 => set_feature(&mut features, *b"jp78", 1),
        FontVariantEastAsianValue::Jis83 => set_feature(&mut features, *b"jp83", 1),
        FontVariantEastAsianValue::Jis90 => set_feature(&mut features, *b"jp90", 1),
        FontVariantEastAsianValue::Jis04 => set_feature(&mut features, *b"jp04", 1),
        FontVariantEastAsianValue::Simplified => set_feature(&mut features, *b"smpl", 1),
        FontVariantEastAsianValue::Traditional => set_feature(&mut features, *b"trad", 1),
        FontVariantEastAsianValue::FullWidth => set_feature(&mut features, *b"fwid", 1),
        FontVariantEastAsianValue::ProportionalWidth => set_feature(&mut features, *b"pwid", 1),
        FontVariantEastAsianValue::Ruby => set_feature(&mut features, *b"ruby", 1),
    }

    // https://drafts.csswg.org/css-fonts-4/#font-variant-position-prop
    use zero_style_system::FontVariantPositionValue;
    match style.font_variant_position {
        FontVariantPositionValue::Normal => {}
        FontVariantPositionValue::Sub => set_feature(&mut features, *b"subs", 1),
        FontVariantPositionValue::Super => set_feature(&mut features, *b"sups", 1),
    }

    // https://drafts.csswg.org/css-fonts-4/#font-variant-alternates-prop
    for setting in &style.font_variant_alternates_features {
        set_feature(&mut features, setting.tag, setting.value);
    }

    if let zero_style_system::FontFeatureSettingsValue::Features(settings) = &style.font_feature_settings {
        for setting in settings {
            set_feature(&mut features, setting.tag, setting.value);
        }
    }
    features
}

impl Iterator for FragmentGlyphs<'_> {
    type Item = FragmentGlyph;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Shaped {
                glyphs,
                sources,
                indexed,
                advanced,
                offset,
            } => {
                let glyph = glyphs.next()?;
                Some(FragmentGlyph {
                    code_point: glyph.code_point,
                    font_id: Some(glyph.font_id),
                    font_size: Some(glyph.font_size),
                    font_glyph_index: (*indexed).then(|| u16::try_from(glyph.glyph_id).ok()).flatten(),
                    source: sources.next().flatten(),
                    x_offset: if *offset { glyph.x_offset } else { 0.0 },
                    y_offset: if *offset { glyph.y_offset } else { 0.0 },
                    advance_x: (*advanced).then_some(glyph.advance_x),
                })
            }
            Self::Legacy(chars) => Some(FragmentGlyph {
                code_point: chars.next()?,
                font_id: None,
                font_size: None,
                font_glyph_index: None,
                source: None,
                x_offset: 0.0,
                y_offset: 0.0,
                advance_x: None,
            }),
        }
    }
}

/// 返回受限 shaping 结果；未启用、不满足边界或 cluster 非一一映射时回退旧路径。
///
/// https://www.w3.org/TR/css-text-3/#text-shaping
pub(super) fn fragment_glyphs<'a>(
    font_ids: &[u32],
    text: &'a str,
    font_size: f32,
    eligible: bool,
    direction: TextDirection,
    advance_eligible: bool,
    logical_source: Option<LogicalFragmentSource<'a>>,
    features: &[OpenTypeFeature],
    variations: &[zero_render_foundation::font::OpenTypeVariation],
    size_adjust: zero_render_foundation::font::FontSizeAdjustment,
) -> FragmentGlyphs<'a> {
    if font_ids.is_empty() {
        return FragmentGlyphs::Legacy(text.chars());
    }
    let complex_enabled = complex_run_enabled(
        direction,
        logical_source.is_some(),
        shaped_complex_enabled(),
        shaped_rtl_enabled(),
    );
    let shape_direction = effective_shape_direction(direction, complex_enabled);
    let logical_source = logical_source.filter(|_| complex_enabled && direction == TextDirection::RightToLeft);
    let shaping_text = logical_source.as_ref().map_or(text, |source| source.text);
    if eligible
        && shaped_text_enabled()
        && (direction != TextDirection::RightToLeft || complex_enabled)
        && let Some(mut glyphs) = crate::shape_text_for_paint(
            font_ids,
            shaping_text,
            font_size,
            shape_direction,
            features,
            variations,
            size_adjust,
        )
    {
        let Some(complex_mapping) = mapping_mode(shaping_text, &glyphs, complex_enabled) else {
            return FragmentGlyphs::Legacy(text.chars());
        };
        let simple_mapping = !complex_mapping;
        let indexed = indexed_glyph_enabled();
        if complex_mapping && !indexed {
            return FragmentGlyphs::Legacy(text.chars());
        }
        let offsets_enabled = shaped_offsets_enabled();
        if simple_mapping
            && crate::text_metrics::source_mapping_requires_offsets(shaping_text, &glyphs)
            && !offsets_enabled
        {
            return FragmentGlyphs::Legacy(text.chars());
        }
        let sources = if let Some(source) = logical_source {
            glyph_sources_in_run(
                shaping_text,
                &glyphs,
                complex_mapping,
                source.source_text,
                source.source_start,
            )
        } else {
            glyph_sources(shaping_text, &glyphs, complex_mapping)
        };
        let Some(sources) = sources else {
            return FragmentGlyphs::Legacy(text.chars());
        };
        let adjusted_glyph_size = crate::text_metrics::glyph_sizes_adjusted(font_size, &glyphs);
        let advance_eligible = advance_eligible || adjusted_glyph_size;
        let generic_contextual = simple_mapping && !advance_eligible && shaped_generic_paint_enabled();
        if generic_contextual {
            for glyph in &mut glyphs {
                // ZRG-2026-08-15：paint_base 按字形实际字体测量（glyph.font_id），
                // 保证与 shaping 同源——此前用 primary 字体测，webfont 字距错乱。
                let paint_base =
                    crate::measure_char_for_font(glyph.font_id.0, glyph.code_point, glyph.font_size, false);
                glyph.advance_x = crate::text_metrics::paint_base_with_contextual_delta(
                    paint_base,
                    glyph.advance_x,
                    glyph.unshaped_advance_x,
                );
            }
        }
        return FragmentGlyphs::Shaped {
            glyphs: glyphs.into_iter(),
            sources: sources.into_iter(),
            indexed,
            advanced: complex_mapping
                || generic_contextual
                || shaped_advance_enabled() && (advance_eligible || shaped_positioning_enabled()),
            offset: complex_mapping || offsets_enabled,
        };
    }
    FragmentGlyphs::Legacy(text.chars())
}

pub(super) fn fragment_advance_trace(
    font_ids: &[u32],
    text: &str,
    font_size: f32,
    direction: TextDirection,
    logical_source: Option<&LogicalFragmentSource<'_>>,
    features: &[OpenTypeFeature],
    variations: &[zero_render_foundation::font::OpenTypeVariation],
    size_adjust: zero_render_foundation::font::FontSizeAdjustment,
) -> Option<FragmentAdvanceTrace> {
    if !shaped_advance_trace_enabled() {
        return None;
    }
    let complex_enabled = complex_run_enabled(
        direction,
        logical_source.is_some(),
        shaped_complex_enabled(),
        shaped_rtl_enabled(),
    );
    let shape_direction = effective_shape_direction(direction, complex_enabled);
    let shaping_text = logical_source.map_or(text, |source| source.text);
    let glyphs = crate::shape_text_for_paint(
        font_ids,
        shaping_text,
        font_size,
        shape_direction,
        features,
        variations,
        size_adjust,
    )?;
    Some(advance_trace_from_glyphs(shaping_text, font_size, &glyphs, size_adjust))
}

fn advance_trace_from_glyphs(
    text: &str,
    font_size: f32,
    glyphs: &[ShapedGlyph],
    size_adjust: zero_render_foundation::font::FontSizeAdjustment,
) -> FragmentAdvanceTrace {
    let mut resolved_font_ids = Vec::new();
    let mut resolved_font_sizes = Vec::new();
    for glyph in glyphs {
        if !resolved_font_ids.contains(&glyph.font_id.0) {
            resolved_font_ids.push(glyph.font_id.0);
        }
        if !resolved_font_sizes.contains(&glyph.font_size) {
            resolved_font_sizes.push(glyph.font_size);
        }
    }
    FragmentAdvanceTrace {
        layout_estimate: text
            .chars()
            .map(|ch| crate::text_metrics::layout_estimate_char_width(ch, font_size, false))
            .sum(),
        unshaped: glyphs.iter().map(|glyph| glyph.unshaped_advance_x).sum(),
        shaped: glyphs.iter().map(|glyph| glyph.advance_x).sum(),
        resolved_font_ids,
        resolved_font_sizes,
        size_adjust,
    }
}

fn complex_run_enabled(
    direction: TextDirection,
    has_logical_source: bool,
    complex_enabled: bool,
    rtl_enabled: bool,
) -> bool {
    complex_enabled || rtl_enabled && has_logical_source && direction == TextDirection::RightToLeft
}

fn effective_shape_direction(direction: TextDirection, complex_enabled: bool) -> TextDirection {
    if complex_enabled {
        direction
    } else {
        TextDirection::Auto
    }
}

fn mapping_mode(text: &str, glyphs: &[ShapedGlyph], allow_complex: bool) -> Option<bool> {
    if crate::text_metrics::one_to_one_source_mapping(text, glyphs) {
        Some(false)
    } else if allow_complex && source_clusters_valid(text, glyphs) {
        Some(true)
    } else {
        None
    }
}

fn source_clusters_valid(text: &str, glyphs: &[ShapedGlyph]) -> bool {
    let Ok(text_len) = u32::try_from(text.len()) else {
        return false;
    };
    !glyphs.is_empty()
        && glyphs.iter().any(|glyph| glyph.cluster == 0)
        && glyphs.iter().all(|glyph| {
            glyph.glyph_id > 0
                && u16::try_from(glyph.glyph_id).is_ok()
                && glyph.cluster < text_len
                && text.is_char_boundary(glyph.cluster as usize)
        })
}

pub(super) fn glyph_sources(
    text: &str,
    glyphs: &[ShapedGlyph],
    all_clusters: bool,
) -> Option<Vec<Option<GlyphSource>>> {
    glyph_sources_in_run(text, glyphs, all_clusters, Arc::from(text), 0)
}

fn glyph_sources_in_run(
    text: &str,
    glyphs: &[ShapedGlyph],
    all_clusters: bool,
    source_text: Arc<str>,
    source_start: u32,
) -> Option<Vec<Option<GlyphSource>>> {
    if !all_clusters && !crate::text_metrics::source_mapping_requires_offsets(text, glyphs) {
        return Some(vec![None; glyphs.len()]);
    }
    if !source_clusters_valid(text, glyphs) {
        return None;
    }
    let text_len = u32::try_from(text.len()).ok()?;
    let source_end = source_start.checked_add(text_len)?;
    if source_text.get(source_start as usize..source_end as usize) != Some(text) {
        return None;
    }
    let mut starts: Vec<u32> = glyphs.iter().map(|glyph| glyph.cluster).collect();
    starts.sort_unstable();
    starts.dedup();
    let mut cluster_counts = HashMap::new();
    for glyph in glyphs {
        *cluster_counts.entry(glyph.cluster).or_insert(0usize) += 1;
    }
    Some(
        glyphs
            .iter()
            .map(|glyph| {
                let end = starts
                    .iter()
                    .copied()
                    .find(|start| *start > glyph.cluster)
                    .unwrap_or(text_len);
                GlyphSource::new(
                    source_text.clone(),
                    source_start.checked_add(glyph.cluster)?,
                    source_start.checked_add(end)?,
                )
                .filter(|source| {
                    all_clusters || cluster_counts[&glyph.cluster] > 1 || source.as_str().chars().nth(1).is_some()
                })
            })
            .collect(),
    )
}

/// 判断是否为须绘制占位框的非空白 Cc 控制字符。
pub(super) fn is_cc_control_char(ch: char) -> bool {
    let cp = ch as u32;
    ((cp <= 0x1F) || (0x7F..=0x9F).contains(&cp)) && !matches!(cp, 0x09 | 0x0A | 0x0C | 0x0D)
}

/// Ahem 在 half-leading 近零时使用 em-box 定位。
pub(super) fn ahem_uses_embox_position(line_height: f32, font_size: f32) -> bool {
    let half_leading = (line_height - font_size) / 2.0;
    half_leading.abs() < 0.5
}

pub(super) fn paint_ifc_baseline_offset(fragment_height: f32) -> f32 {
    fragment_height
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::primitive::FontId;

    fn glyph() -> ShapedGlyph {
        ShapedGlyph {
            glyph_id: 7,
            font_id: FontId(1),
            font_size: 16.0,
            advance_x: 8.0,
            unshaped_advance_x: 9.0,
            x_offset: 2.0,
            y_offset: 3.0,
            cluster: 0,
            code_point: 'A',
        }
    }

    #[test]
    fn shaped_advance_and_offsets_are_independent() {
        let mut offsets_only = FragmentGlyphs::Shaped {
            glyphs: vec![glyph()].into_iter(),
            sources: vec![None].into_iter(),
            indexed: true,
            advanced: false,
            offset: true,
        };
        let offset = offsets_only.next().expect("offset glyph");
        assert_eq!(offset.advance_x, None);
        assert_eq!((offset.x_offset, offset.y_offset), (2.0, 3.0));

        let mut advance_only = FragmentGlyphs::Shaped {
            glyphs: vec![glyph()].into_iter(),
            sources: vec![None].into_iter(),
            indexed: true,
            advanced: true,
            offset: false,
        };
        let advance = advance_only.next().expect("advance glyph");
        assert_eq!(advance.advance_x, Some(8.0));
        assert_eq!((advance.x_offset, advance.y_offset), (0.0, 0.0));
    }

    #[test]
    fn shaped_fragment_preserves_resolved_font_id() {
        let mut resolved = glyph();
        resolved.font_id = FontId(9);
        let mut glyphs = FragmentGlyphs::Shaped {
            glyphs: vec![resolved].into_iter(),
            sources: vec![None].into_iter(),
            indexed: true,
            advanced: false,
            offset: false,
        };

        assert_eq!(glyphs.next().expect("resolved glyph").font_id, Some(FontId(9)));
    }

    #[test]
    fn advance_trace_compares_estimated_unshaped_and_shaped_widths() {
        let mut first = glyph();
        first.advance_x = 8.5;
        let mut second = glyph();
        second.code_point = 'V';
        second.advance_x = 8.0;

        let trace = advance_trace_from_glyphs(
            "AV",
            16.0,
            &[first, second],
            zero_render_foundation::font::FontSizeAdjustment::None,
        );

        assert_eq!(trace.layout_estimate, 17.6);
        assert_eq!(trace.unshaped, 18.0);
        assert_eq!(trace.shaped, 16.5);
    }

    #[test]
    fn contextual_glyph_deltas_sum_to_the_contextual_fragment_width() {
        let paint_bases = [10.0, 11.0];
        let shaped = [9.5, 10.25];
        let unshaped = [10.0, 10.5];
        let glyph_width: f32 = paint_bases
            .iter()
            .zip(shaped)
            .zip(unshaped)
            .map(|((&base, shaped), unshaped)| {
                crate::text_metrics::paint_base_with_contextual_delta(base, shaped, unshaped)
            })
            .sum();

        assert_eq!(
            glyph_width,
            crate::text_metrics::paint_base_with_contextual_delta(21.0, 19.75, 20.5)
        );
    }

    #[test]
    fn adjusted_generic_font_uses_shaped_advance_policy() {
        assert!(shaped_advance_policy(false, false, false));
        assert!(!shaped_advance_policy(true, false, true));
        assert!(!shaped_advance_policy(true, true, false));
        assert!(shaped_advance_policy(true, true, true));
    }

    #[test]
    fn rtl_gate_only_enables_runs_with_logical_source() {
        assert!(!complex_run_enabled(TextDirection::RightToLeft, true, false, false));
        assert!(complex_run_enabled(TextDirection::RightToLeft, true, false, true));
        assert!(!complex_run_enabled(TextDirection::RightToLeft, false, false, true));
        assert!(!complex_run_enabled(TextDirection::LeftToRight, true, false, true));
        assert!(complex_run_enabled(TextDirection::LeftToRight, false, true, false));
    }

    #[test]
    fn uba_rtl_gate_overrides_ltr_style_only_for_uniform_rtl_fragment() {
        let rtl = TextFragmentSource {
            text: Arc::<str>::from("aאבb"),
            visual_to_logical: vec![Some(3..5), Some(1..3)],
            visual_is_rtl: vec![true, true],
        };
        assert_eq!(
            fragment_shape_direction(Some(&rtl), TextDirection::LeftToRight, false),
            TextDirection::LeftToRight
        );
        assert_eq!(
            fragment_shape_direction(Some(&rtl), TextDirection::LeftToRight, true),
            TextDirection::RightToLeft
        );
        let logical = logical_fragment_source(
            Some(&rtl),
            fragment_shape_direction(Some(&rtl), TextDirection::LeftToRight, true),
            true,
        )
        .expect("uniform RTL fragment in LTR container");
        assert_eq!(logical.text, "אב");

        let mixed = TextFragmentSource {
            text: Arc::<str>::from("aאב"),
            visual_to_logical: vec![Some(0..1), Some(3..5), Some(1..3)],
            visual_is_rtl: vec![false, true, true],
        };
        assert_eq!(
            fragment_shape_direction(Some(&mixed), TextDirection::LeftToRight, true),
            TextDirection::LeftToRight
        );
    }

    #[test]
    fn style_features_follow_variant_spacing_and_explicit_precedence() {
        let mut style = zero_style_system::ComputedStyle::default();
        style.font_variant_ligatures.common = Some(true);
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"liga", 1), OpenTypeFeature::new(*b"clig", 1),]
        );

        // letter-spacing: 0em — explicit zero does NOT suppress ligatures
        style.letter_spacing_normal = false;
        style.letter_spacing = zero_style_system::LengthValue::Px(0.0);
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"liga", 1), OpenTypeFeature::new(*b"clig", 1),]
        );

        // letter-spacing: 0.1em — non-zero DOES suppress ligatures
        style.letter_spacing = zero_style_system::LengthValue::Px(3.2);
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"liga", 0), OpenTypeFeature::new(*b"clig", 0),]
        );

        // font-feature-settings: 'liga' on — highest priority, re-enables
        style.font_feature_settings =
            zero_style_system::FontFeatureSettingsValue::Features(vec![zero_style_system::FontFeatureSetting {
                tag: *b"liga",
                value: 1,
            }]);
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"liga", 1), OpenTypeFeature::new(*b"clig", 0),]
        );

        style.font_feature_settings = zero_style_system::FontFeatureSettingsValue::Normal;
        style.font_variant_ligatures = zero_style_system::FontVariantLigaturesValue::default();
        style.letter_spacing_normal = true;
        style.letter_spacing = zero_style_system::LengthValue::Px(0.0);
        style.font_variant_alternates_features = vec![zero_style_system::FontFeatureSetting {
            tag: *b"hist",
            value: 1,
        }];
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"hist", 1)]
        );
    }

    #[test]
    fn author_faces_preserve_fallback_without_enabling_global_fallback() {
        assert!(!preserve_font_fallback_faces_with_policy(false, true, false));
        assert!(preserve_font_fallback_faces_with_policy(false, true, true));
        assert!(!preserve_font_fallback_faces_with_policy(false, false, true));
        assert!(preserve_font_fallback_faces_with_policy(true, false, false));
    }

    #[test]
    fn style_features_map_font_kerning_by_writing_mode_and_allow_explicit_override() {
        let mut style = zero_style_system::ComputedStyle {
            font_kerning: zero_style_system::FontKerningValue::Normal,
            ..Default::default()
        };
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"kern", 1)]
        );

        style.writing_mode = zero_style_system::WritingModeValue::VerticalRl;
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"vkrn", 1)]
        );

        style.font_kerning = zero_style_system::FontKerningValue::None;
        style.font_feature_settings =
            zero_style_system::FontFeatureSettingsValue::Features(vec![zero_style_system::FontFeatureSetting {
                tag: *b"vkrn",
                value: 1,
            }]);
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"kern", 0), OpenTypeFeature::new(*b"vkrn", 1),]
        );
    }

    #[test]
    fn explicit_feature_settings_override_resolved_alternates() {
        let style = zero_style_system::ComputedStyle {
            font_variant_alternates_features: vec![
                zero_style_system::FontFeatureSetting {
                    tag: *b"salt",
                    value: 2,
                },
                zero_style_system::FontFeatureSetting {
                    tag: *b"ss03",
                    value: 1,
                },
            ],
            font_feature_settings: zero_style_system::FontFeatureSettingsValue::Features(vec![
                zero_style_system::FontFeatureSetting {
                    tag: *b"salt",
                    value: 0,
                },
            ]),
            ..Default::default()
        };
        assert_eq!(
            style_open_type_features(&style),
            vec![OpenTypeFeature::new(*b"salt", 0), OpenTypeFeature::new(*b"ss03", 1),]
        );
    }

    #[test]
    fn glyph_sources_share_utf8_cluster_ranges_within_one_text_run() {
        let mut base = glyph();
        base.code_point = 'A';
        let mut mark = glyph();
        mark.code_point = '\u{301}';
        let mut trailing = glyph();
        trailing.code_point = 'B';
        trailing.cluster = 3;

        let sources = glyph_sources("A\u{301}B", &[base, mark, trailing], false).expect("valid sources");
        let base = sources[0].as_ref().expect("base source");
        let mark = sources[1].as_ref().expect("mark source");

        assert_eq!(base.as_str(), "A\u{301}");
        assert!(base.same_cluster(mark));
        assert!(sources[2].is_none());
    }

    #[test]
    fn logical_fragment_source_requires_rtl_without_text_transform() {
        let text = Arc::<str>::from("xאבגy");
        let source = TextFragmentSource {
            text: text.clone(),
            visual_to_logical: vec![Some(5..7), Some(3..5), Some(1..3)],
            visual_is_rtl: vec![true, true, true],
        };
        let logical =
            logical_fragment_source(Some(&source), TextDirection::RightToLeft, true).expect("contiguous RTL source");
        assert_eq!(logical.text, "אבג");
        assert_eq!(logical.source_start, 1);
        assert!(Arc::ptr_eq(&logical.source_text, &text));
        assert!(logical_fragment_source(Some(&source), TextDirection::LeftToRight, true).is_none());
        assert!(logical_fragment_source(Some(&source), TextDirection::RightToLeft, false).is_none());
    }

    #[test]
    fn logical_fragment_source_rejects_mixed_mapping() {
        let source = TextFragmentSource {
            text: Arc::<str>::from("aאב"),
            visual_to_logical: vec![Some(0..1), Some(3..5), Some(1..3)],
            visual_is_rtl: vec![false, true, true],
        };
        assert!(logical_fragment_source(Some(&source), TextDirection::RightToLeft, true).is_none());

        let monotonic_mixed = TextFragmentSource {
            text: Arc::<str>::from("aאב"),
            visual_to_logical: vec![Some(3..5), Some(1..3), Some(0..1)],
            visual_is_rtl: vec![true, true, true],
        };
        assert!(logical_fragment_source(Some(&monotonic_mixed), TextDirection::RightToLeft, true).is_none());
    }

    #[test]
    fn complex_glyph_sources_cover_ligature_and_decreasing_clusters() {
        assert_eq!(
            effective_shape_direction(TextDirection::RightToLeft, false),
            TextDirection::Auto
        );
        assert_eq!(
            effective_shape_direction(TextDirection::RightToLeft, true),
            TextDirection::RightToLeft
        );
        let mut ligature = glyph();
        ligature.code_point = 'f';
        assert!(source_clusters_valid("fi", &[ligature.clone()]));
        assert_eq!(mapping_mode("fi", &[ligature.clone()], false), None);
        assert_eq!(mapping_mode("fi", &[ligature.clone()], true), Some(true));
        let ligature_sources = glyph_sources("fi", &[ligature], true).expect("ligature sources");
        assert_eq!(ligature_sources[0].as_ref().expect("ligature source").as_str(), "fi");

        let mut gimel = glyph();
        gimel.cluster = 4;
        gimel.code_point = 'ג';
        let mut bet = glyph();
        bet.cluster = 2;
        bet.code_point = 'ב';
        let mut alef = glyph();
        alef.code_point = 'א';
        let rtl = [gimel, bet, alef];
        assert!(source_clusters_valid("אבג", &rtl));
        let sources = glyph_sources("אבג", &rtl, true).expect("RTL sources");
        assert_eq!(sources[0].as_ref().expect("gimel source").as_str(), "ג");
        assert_eq!(sources[1].as_ref().expect("bet source").as_str(), "ב");
        assert_eq!(sources[2].as_ref().expect("alef source").as_str(), "א");

        let full_run = Arc::<str>::from("xאבגy");
        let sources = glyph_sources_in_run("אבג", &rtl, true, full_run.clone(), 1).expect("full-run RTL sources");
        let gimel = sources[0].as_ref().expect("gimel full-run source");
        assert_eq!((gimel.start, gimel.end), (5, 7));
        assert!(Arc::ptr_eq(&gimel.text, &full_run));
        assert!(sources.iter().flatten().all(|source| source.same_text_run(gimel)));
        assert!(glyph_sources_in_run("אבג", &rtl, true, full_run, 0).is_none());
    }

    #[test]
    fn complex_glyph_sources_reject_non_boundary_cluster() {
        let mut invalid = glyph();
        invalid.cluster = 1;
        assert!(!source_clusters_valid("אב", &[invalid]));
    }

    #[test]
    fn paint_ifc_offset_places_glyph_at_fragment_baseline() {
        assert_eq!(paint_ifc_baseline_offset(24.0), 24.0);
        assert_eq!(paint_ifc_baseline_offset(48.0), 48.0);
    }
}
