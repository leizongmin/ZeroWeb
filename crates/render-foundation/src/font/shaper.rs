//! 文本整形 — 将 Unicode 文本转换为 Glyph 序列，支持简单换行。
//!
//! TextShaper 基于 rustybuzz 进行 OpenType shaping（连字、kerning、GSUB/GPOS），
//! 回退到 fontdue 的逐字符映射。

use crate::font::loader::FontLoader;
use crate::primitive::FontId;
use unicode_segmentation::UnicodeSegmentation;

fn shaped_fallback_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    // R3243-F 曾默认开启（!= "0"）：中文长文逐字 fragment 每帧多 face fallback
    // shaping 重排，perf-gate morning paint 3x 回归。改回显式 opt-in（R3243-F 之前
    // 语义）；reftest 实测 fallback 关闭零回归（css-fonts 17/20 与全量均一致）。
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_FALLBACK").as_deref() == Ok("1"))
}

pub(crate) fn per_face_features_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_PER_FACE_FEATURES").as_deref() != Ok("0"))
}

/// 文本 shaping 方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextDirection {
    /// 由 rustybuzz 根据文本猜测方向。
    Auto,
    /// 从左到右。
    LeftToRight,
    /// 从右到左。
    RightToLeft,
}

/// CSS `font-size-adjust` 使用的字体 metric。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSizeAdjustMetric {
    /// x-height / em。
    ExHeight,
    /// cap-height / em。
    CapHeight,
    /// "0" glyph horizontal advance / em。
    ChWidth,
    /// U+6C34 horizontal advance / em。
    IcWidth,
    /// U+6C34 vertical advance / em。
    IcHeight,
}

/// CSS `font-size-adjust` 对 shaping face 的字号调整。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSizeAdjustment {
    /// 不调整字号。
    None,
    /// 使用指定 metric；`target=None` 表示从 primary face 读取。
    Adjust {
        /// 调整依据。
        metric: FontSizeAdjustMetric,
        /// 显式 aspect value，或 `None`（`from-font`）。
        target: Option<f32>,
    },
}

impl FontSizeAdjustment {
    pub(crate) fn cache_key(self) -> (u8, u32) {
        match self {
            Self::None => (0, 0),
            Self::Adjust { metric, target } => {
                let metric = match metric {
                    FontSizeAdjustMetric::ExHeight => 0,
                    FontSizeAdjustMetric::CapHeight => 1,
                    FontSizeAdjustMetric::ChWidth => 2,
                    FontSizeAdjustMetric::IcWidth => 3,
                    FontSizeAdjustMetric::IcHeight => 4,
                };
                (
                    1 + metric * 2 + u8::from(target.is_none()),
                    target.map_or(0, f32::to_bits),
                )
            }
        }
    }
}

/// 应用于完整 shaping run 的 OpenType feature。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpenTypeFeature {
    /// 四字节 OpenType feature tag（如 `liga`）。
    pub tag: [u8; 4],
    /// feature 值；`0` 表示关闭，`1` 表示开启，其余值供索引型 feature 使用。
    pub value: u32,
}

impl OpenTypeFeature {
    /// 创建一个作用于完整 shaping run 的 feature。
    pub const fn new(tag: [u8; 4], value: u32) -> Self {
        Self { tag, value }
    }
}

/// 应用于完整 shaping run 的 OpenType variation axis。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenTypeVariation {
    /// 四字节 OpenType axis tag（如 `wght`）。
    pub tag: [u8; 4],
    /// axis 坐标。
    pub value: f32,
}

impl OpenTypeVariation {
    /// 创建一个作用于完整 shaping run 的 variation axis。
    pub const fn new(tag: [u8; 4], value: f32) -> Self {
        Self { tag, value }
    }

    pub(crate) fn cache_key(self) -> ([u8; 4], u32) {
        (self.tag, self.value.to_bits())
    }
}

/// 一次文本整形调用的可选输入。
#[derive(Debug, Clone, Copy)]
pub struct TextShapingOptions<'a> {
    /// 文本方向。
    pub direction: TextDirection,
    /// 元素级 OpenType feature overrides。
    pub features: &'a [OpenTypeFeature],
    /// 元素级 OpenType variation axis overrides。
    pub variations: &'a [OpenTypeVariation],
    /// per-face `font-size-adjust`。
    pub adjustment: FontSizeAdjustment,
    /// R34xx：BCP47 语言标签（canvas ctx.lang 等——shaping 语言系统选择；
    /// 'tr' 走 GSUB TRK lang sys 关闭 fi 连字。None = 字体默认语言）。
    pub language: Option<&'a str>,
}

impl Default for TextShapingOptions<'_> {
    fn default() -> Self {
        Self {
            direction: TextDirection::Auto,
            features: &[],
            variations: &[],
            adjustment: FontSizeAdjustment::None,
            language: None,
        }
    }
}

/// 单个整形后的 Glyph 信息
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// 字体内部的 glyph 索引
    pub glyph_id: u32,
    /// 字体 ID
    pub font_id: FontId,
    /// resolved face 应用 `font-size-adjust` 后的实际字号。
    pub font_size: f32,
    /// 相对于行首的水平前进宽度（像素）
    pub advance_x: f32,
    /// 同一源码字符未应用 kerning/GPOS 时的裸 glyph advance（像素）。
    pub unshaped_advance_x: f32,
    /// 水平偏移量（像素，用于 mark positioning / kerning）
    pub x_offset: f32,
    /// 垂直偏移量（像素）
    pub y_offset: f32,
    /// rustybuzz 返回的原始 UTF-8 cluster 字节偏移。
    pub cluster: u32,
    /// 该字符的 Unicode 码点（用于回退标识）
    pub code_point: char,
}

/// 一行整形结果
#[derive(Debug, Clone)]
pub struct ShapedLine {
    /// 该行的 glyph 序列
    pub glyphs: Vec<ShapedGlyph>,
    /// 行的总前进宽度
    pub width: f32,
}

/// 文本整形器 — 将文本字符串转换为带位置的 glyph 序列
pub struct TextShaper<'a> {
    /// 字体加载器引用
    font_loader: &'a FontLoader,
    /// 默认字体 ID（整形时使用）
    default_font_id: Option<FontId>,
}

impl<'a> TextShaper<'a> {
    /// 使用指定字体加载器创建整形器。
    ///
    /// `default_font_id` 为整形使用的默认字体 ID，可以为 None（此时整形仅产生占位 glyph）。
    pub fn new(font_loader: &'a FontLoader, default_font_id: Option<FontId>) -> Self {
        Self {
            font_loader,
            default_font_id,
        }
    }

    /// 将文本整形为 glyph 序列，不进行换行（单行模式）。
    ///
    /// 优先使用 rustybuzz 进行 OpenType shaping（连字、kerning、GSUB/GPOS），
    /// 如果字体数据不可用则回退到 fontdue 的逐字符映射。
    pub fn shape_single_line(&self, text: &str, font_size: f32) -> Vec<ShapedGlyph> {
        self.shape_single_line_with_direction(text, font_size, TextDirection::Auto)
    }

    /// 使用显式方向将文本整形为 glyph 序列。
    pub fn shape_single_line_with_direction(
        &self,
        text: &str,
        font_size: f32,
        direction: TextDirection,
    ) -> Vec<ShapedGlyph> {
        self.shape_single_line_with_features(text, font_size, direction, &[])
    }

    /// 使用显式方向与 OpenType feature 将文本整形为 glyph 序列。
    pub fn shape_single_line_with_features(
        &self,
        text: &str,
        font_size: f32,
        direction: TextDirection,
        features: &[OpenTypeFeature],
    ) -> Vec<ShapedGlyph> {
        let font_ids = self.default_font_id.map_or_else(Vec::new, |font_id| vec![font_id.0]);
        self.shape_single_line_with_font_ids(&font_ids, text, font_size, direction, features)
    }

    /// 使用有序 CSS face 列表整形文本。
    pub fn shape_single_line_with_font_ids(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        direction: TextDirection,
        features: &[OpenTypeFeature],
    ) -> Vec<ShapedGlyph> {
        self.shape_single_line_with_font_ids_and_adjustment(
            font_ids,
            text,
            font_size,
            direction,
            features,
            FontSizeAdjustment::None,
        )
    }

    /// 使用有序 CSS face 列表和 per-face `font-size-adjust` 整形文本。
    pub fn shape_single_line_with_font_ids_and_adjustment(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        direction: TextDirection,
        features: &[OpenTypeFeature],
        adjustment: FontSizeAdjustment,
    ) -> Vec<ShapedGlyph> {
        self.shape_single_line_with_font_ids_and_options(
            font_ids,
            text,
            font_size,
            TextShapingOptions {
                direction,
                features,
                adjustment,
                ..TextShapingOptions::default()
            },
        )
    }

    /// 使用有序 CSS face 列表、variation axes 和 per-face `font-size-adjust` 整形文本。
    pub fn shape_single_line_with_font_ids_and_options(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        options: TextShapingOptions<'_>,
    ) -> Vec<ShapedGlyph> {
        let TextShapingOptions {
            direction,
            features,
            variations,
            adjustment,
            language,
        } = options;
        let primary_id = font_ids
            .first()
            .copied()
            .map(FontId)
            .or(self.default_font_id)
            .unwrap_or(FontId(0));
        let primary_size = self
            .font_loader
            .adjusted_font_size(primary_id.0, primary_id.0, font_size, adjustment);
        let per_face_features = per_face_features_enabled();
        let primary_features = if per_face_features {
            self.font_loader.resolved_font_features(primary_id.0, features)
        } else {
            features.to_vec()
        };
        let primary_variations = self.font_loader.resolved_font_variations(primary_id.0, variations);

        // 尝试 rustybuzz shaping
        if !font_ids.is_empty() || self.default_font_id.is_some() {
            if (font_ids.len() > 1 || shaped_fallback_enabled() || !primary_variations.is_empty())
                && let Some(glyphs) = self.shape_with_fallback_runs(font_ids, text, font_size, options)
            {
                return glyphs;
            }
            if let Some(glyphs) = self.shape_with_rustybuzz(
                primary_id,
                text,
                primary_size,
                direction,
                &primary_features,
                &primary_variations,
                language,
            ) {
                return glyphs;
            }
        }

        // 回退：fontdue 逐字符映射
        let mut glyphs = self.shape_fallback(text, primary_size, primary_id);
        if direction == TextDirection::RightToLeft {
            glyphs.reverse();
        }
        glyphs
    }

    /// 按 grapheme 覆盖范围切分连续同 face run，再分别 shaping。
    ///
    /// https://drafts.csswg.org/css-fonts-4/#font-matching-algorithm
    fn shape_with_fallback_runs(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        options: TextShapingOptions<'_>,
    ) -> Option<Vec<ShapedGlyph>> {
        let TextShapingOptions {
            direction,
            features,
            variations,
            adjustment,
            language,
        } = options;
        let per_face_features = per_face_features_enabled();
        if text.is_empty() || direction != TextDirection::LeftToRight || !per_face_features && !features.is_empty() {
            return None;
        }
        let primary_id = FontId(*font_ids.first()?);
        // OPTIMIZATION: fallback 链预构建一次（去重），避免每 grapheme 重建
        //（resolve_font_for_code_point_in 每次重新拼 chain + contains 去重——
        // 中文长文逐字扫描下 O(graphemes × chain) 分配/比较，perf-gate morning paint 回归）。
        let mut chain = Vec::with_capacity(font_ids.len() + self.font_loader.fallback_chain().len());
        for &font_id in font_ids.iter().chain(self.font_loader.fallback_chain()) {
            if !chain.contains(&font_id) {
                chain.push(font_id);
            }
        }

        let mut runs: Vec<(usize, usize, FontId)> = Vec::new();
        let mut used_fallback = false;
        for (start, grapheme) in text.grapheme_indices(true) {
            let selector = grapheme
                .chars()
                .find(|ch| !is_face_ignorable(*ch))
                .or_else(|| grapheme.chars().next())?;
            let font_id = FontId(
                self.font_loader
                    .resolve_font_for_code_point_in_chain(&chain, selector)?,
            );
            let end = start + grapheme.len();
            used_fallback |= font_id != primary_id;
            if let Some((_, run_end, run_font_id)) = runs.last_mut()
                && *run_font_id == font_id
            {
                *run_end = end;
            } else {
                runs.push((start, end, font_id));
            }
        }
        tracing::debug!(
            target: "zero_render_foundation::shaped_fallback",
            text,
            font_ids = ?font_ids,
            used_fallback,
            runs = ?runs,
            "ZW_SHAPED_FALLBACK"
        );
        if !used_fallback {
            return None;
        }
        let mut result = Vec::new();
        for (start, end, font_id) in runs {
            let run_text = text.get(start..end)?;
            let resolved_size = self
                .font_loader
                .adjusted_font_size(primary_id.0, font_id.0, font_size, adjustment);
            // https://drafts.csswg.org/css-fonts-4/#feature-precedence
            let resolved_features = if per_face_features {
                self.font_loader.resolved_font_features(font_id.0, features)
            } else {
                features.to_vec()
            };
            let resolved_variations = self.font_loader.resolved_font_variations(font_id.0, variations);
            let mut glyphs = self.shape_with_rustybuzz(
                font_id,
                run_text,
                resolved_size,
                direction,
                &resolved_features,
                &resolved_variations,
                language,
            )?;
            let cluster_base = u32::try_from(start).ok()?;
            for glyph in &mut glyphs {
                glyph.cluster = glyph.cluster.checked_add(cluster_base)?;
            }
            result.extend(glyphs);
        }
        Some(result)
    }

    /// 使用 rustybuzz 进行 OpenType shaping。
    #[allow(clippy::too_many_arguments)]
    fn shape_with_rustybuzz(
        &self,
        font_id: FontId,
        text: &str,
        font_size: f32,
        direction: TextDirection,
        features: &[OpenTypeFeature],
        variations: &[OpenTypeVariation],
        language: Option<&str>,
    ) -> Option<Vec<ShapedGlyph>> {
        let font_data = self.font_loader.get_font_data(font_id.0)?;

        let mut face = rustybuzz::Face::from_slice(font_data, self.font_loader.face_index(font_id.0))?;
        // https://drafts.csswg.org/css-fonts-4/#font-variation-settings-def
        let variations = variations
            .iter()
            .map(|variation| rustybuzz::Variation {
                tag: rustybuzz::ttf_parser::Tag::from_bytes(&variation.tag),
                value: variation.value,
            })
            .collect::<Vec<_>>();
        face.set_variations(&variations);

        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        match direction {
            TextDirection::Auto => {}
            TextDirection::LeftToRight => buffer.set_direction(rustybuzz::Direction::LeftToRight),
            TextDirection::RightToLeft => buffer.set_direction(rustybuzz::Direction::RightToLeft),
        }
        // R34xx：shaping 语言（canvas ctx.lang 等）——GSUB 语言系统选择（'tr' → TRK
        // 关闭 fi 连字）。rustybuzz::Language 接受 BCP47 串（内部映射 OT lang tag）。
        if let Some(lang) = language
            && let Ok(l) = lang.parse::<rustybuzz::Language>()
        {
            buffer.set_language(l);
        }

        // https://drafts.csswg.org/css-fonts-4/#feature-precedence
        let features = features
            .iter()
            .map(|feature| {
                rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(&feature.tag), feature.value, ..)
            })
            .collect::<Vec<_>>();
        let glyph_buffer = rustybuzz::shape(&face, &features, buffer);

        let glyph_infos = glyph_buffer.glyph_infos();
        let glyph_positions = glyph_buffer.glyph_positions();

        // rustybuzz 的 glyph_position 字段为字体设计单位（UPM 刻度），
        // 转换为像素须乘 font_size / units_per_em。
        let upem = face.units_per_em() as f32;
        let px_per_unit = if upem > 0.0 { font_size / upem } else { 0.0 };

        // https://www.w3.org/TR/css-text-3/#text-shaping
        // rustybuzz 的 cluster 是原始 UTF-8 字节偏移，不是 Unicode 标量索引。
        let source_chars: Vec<(usize, char)> = text.char_indices().collect();
        let ordinal_source_mapping = glyph_infos.len() == source_chars.len()
            && glyph_infos.windows(2).all(|pair| pair[0].cluster <= pair[1].cluster);
        let mut glyphs = Vec::with_capacity(glyph_infos.len());
        for (i, (info, pos)) in glyph_infos.iter().zip(glyph_positions.iter()).enumerate() {
            let code_point = if ordinal_source_mapping {
                source_chars[i].1
            } else {
                text.get(info.cluster as usize..)
                    .and_then(|cluster| cluster.chars().next())
                    .or_else(|| source_chars.get(i).map(|(_, ch)| *ch))
                    .unwrap_or('\u{FFFD}')
            };

            // x_advance 已包含 kerning、GPOS 和连字调整；用 fontdue 的裸 glyph
            // advance 覆盖它会撤销 shaping 结果。glyph_id=0 时保留原估算回退。
            let advance_x = if info.glyph_id == 0 {
                font_size * 0.6
            } else {
                pos.x_advance as f32 * px_per_unit
            };
            let unshaped_advance_x = self
                .query_glyph_metrics(font_id.0, code_point, font_size)
                .map_or(font_size * 0.6, |(_, advance)| advance);

            glyphs.push(ShapedGlyph {
                glyph_id: info.glyph_id,
                font_id,
                font_size,
                advance_x,
                unshaped_advance_x,
                x_offset: pos.x_offset as f32 * px_per_unit,
                y_offset: pos.y_offset as f32 * px_per_unit,
                cluster: info.cluster,
                code_point,
            });
        }

        Some(glyphs)
    }

    /// fontdue 逐字符映射回退路径。
    fn shape_fallback(&self, text: &str, font_size: f32, font_id: FontId) -> Vec<ShapedGlyph> {
        let mut glyphs = Vec::with_capacity(text.len());

        for (cluster, ch) in text.char_indices() {
            let (glyph_id, advance_x) = if let Some(fid) = self.default_font_id {
                match self.query_glyph_metrics(fid.0, ch, font_size) {
                    Some((gid, adv)) => (gid, adv),
                    None => (0u32, font_size * 0.6),
                }
            } else {
                (ch as u32, font_size * 0.6)
            };

            glyphs.push(ShapedGlyph {
                glyph_id,
                font_id,
                font_size,
                advance_x,
                unshaped_advance_x: advance_x,
                x_offset: 0.0,
                y_offset: 0.0,
                cluster: cluster as u32,
                code_point: ch,
            });
        }

        glyphs
    }

    /// 将文本整形为多行 glyph 序列，在指定行宽处换行。
    ///
    /// 换行规则：逐字符累积前进宽度，当累积宽度超过 `max_line_width` 时
    /// 在最后一个空格处折行；如果没有空格则在超限处折行。
    /// 显式换行符 `'\n'` 强制折行。
    pub fn shape_with_line_wrap(&self, text: &str, font_size: f32, max_line_width: f32) -> Vec<ShapedLine> {
        if max_line_width <= 0.0 || text.is_empty() {
            return vec![ShapedLine {
                glyphs: vec![],
                width: 0.0,
            }];
        }

        let glyphs = self.shape_single_line(text, font_size);

        let mut lines: Vec<ShapedLine> = Vec::new();
        let mut current_line: Vec<ShapedGlyph> = Vec::new();
        let mut current_width: f32 = 0.0;
        let mut last_space_idx: Option<usize> = None;
        let mut width_at_last_space: f32 = 0.0;

        for glyph in glyphs.iter() {
            // 显式换行符
            if glyph.code_point == '\n' {
                lines.push(ShapedLine {
                    glyphs: std::mem::take(&mut current_line),
                    width: current_width,
                });
                current_width = 0.0;
                last_space_idx = None;
                width_at_last_space = 0.0;
                continue;
            }

            // 记录空格位置
            if glyph.code_point == ' ' {
                last_space_idx = Some(current_line.len());
                width_at_last_space = current_width;
            }

            let new_width = current_width + glyph.advance_x;

            if new_width > max_line_width && !current_line.is_empty() {
                // 需要换行
                if let Some(space_idx) = last_space_idx {
                    // 在最后一个空格处折行
                    let remaining: Vec<ShapedGlyph> = current_line.split_off(space_idx);
                    lines.push(ShapedLine {
                        glyphs: current_line,
                        width: width_at_last_space,
                    });
                    // 跳过折行处的空格
                    current_line = remaining.into_iter().skip(1).collect();
                    current_width = current_line.iter().map(|g| g.advance_x).sum();
                } else {
                    // 没有空格，在超限处折行
                    lines.push(ShapedLine {
                        glyphs: std::mem::take(&mut current_line),
                        width: current_width,
                    });
                    current_line.push(glyph.clone());
                    current_width = glyph.advance_x;
                }
                last_space_idx = None;
                width_at_last_space = 0.0;
            } else {
                current_line.push(glyph.clone());
                current_width = new_width;
            }
        }

        // 最后一行
        if !current_line.is_empty() {
            lines.push(ShapedLine {
                glyphs: current_line,
                width: current_width,
            });
        }

        if lines.is_empty() {
            lines.push(ShapedLine {
                glyphs: vec![],
                width: 0.0,
            });
        }

        lines
    }

    /// 查询指定字符在字体中的 glyph 索引和前进宽度。
    fn query_glyph_metrics(&self, font_id: u32, code_point: char, font_size: f32) -> Option<(u32, f32)> {
        let data = self.font_loader.get_font_data(font_id)?;
        let face = rustybuzz::ttf_parser::Face::parse(data, self.font_loader.face_index(font_id)).ok()?;
        let glyph_id = face.glyph_index(code_point).filter(|glyph_id| glyph_id.0 != 0)?;
        let units_per_em = f32::from(face.units_per_em());
        if units_per_em <= 0.0 {
            return None;
        }
        let advance = f32::from(face.glyph_hor_advance(glyph_id)?) * font_size / units_per_em;
        Some((u32::from(glyph_id.0), advance))
    }
}

fn is_face_ignorable(ch: char) -> bool {
    matches!(ch, '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}')
        || matches!(ch as u32, 0xFE00..=0xFE0F | 0xE0100..=0xE01EF)
}

/// 计算文本在指定字体大小下单行渲染所需的总宽度（像素）。
///
/// 如果没有可用字体则返回每个字符约 0.6 * font_size 的估算宽度。
pub fn measure_text_width(shaper: &TextShaper, text: &str, font_size: f32) -> f32 {
    shaper
        .shape_single_line(text, font_size)
        .iter()
        .map(|g| g.advance_x)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::loader::FontLoader;

    const LATO_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");

    /// 创建空的 TextShaper（无字体）。
    fn make_empty_shaper() -> TextShaper<'static> {
        static LOADER: std::sync::OnceLock<FontLoader> = std::sync::OnceLock::new();
        let loader = LOADER.get_or_init(FontLoader::new);
        // OnceLock 中存储的对象生命周期为 'static，无需 unsafe
        TextShaper::new(loader, None)
    }

    /// 查找一个可用的系统字体文件
    fn find_system_font() -> Option<std::path::PathBuf> {
        let candidates = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return Some(std::path::PathBuf::from(path));
            }
        }
        None
    }

    /// 加载系统字体数据（如果可用）
    fn load_system_font_data() -> Option<Vec<u8>> {
        let path = find_system_font()?;
        std::fs::read(path).ok()
    }

    /// 测试空文本整形。
    #[test]
    fn test_shape_empty_text() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("", 16.0);
        assert!(glyphs.is_empty());
    }

    /// 测试无字体时整形产生占位 glyph。
    #[test]
    fn test_shape_no_font_placeholder() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("AB", 16.0);
        assert_eq!(glyphs.len(), 2);
        // 无字体时 glyph_id 等于 code_point
        assert_eq!(glyphs[0].glyph_id, 'A' as u32);
        assert_eq!(glyphs[1].glyph_id, 'B' as u32);
        // 占位 advance 约 0.6 * font_size
        assert!((glyphs[0].advance_x - 16.0 * 0.6).abs() < 0.01);
    }

    /// 测试 measure_text_width 对空文本返回 0。
    #[test]
    fn test_measure_empty_text() {
        let shaper = make_empty_shaper();
        let width = measure_text_width(&shaper, "", 16.0);
        assert_eq!(width, 0.0);
    }

    /// 测试 measure_text_width 对非空文本返回正值。
    #[test]
    fn test_measure_nonempty_text() {
        let shaper = make_empty_shaper();
        let width = measure_text_width(&shaper, "Hello", 16.0);
        assert!(width > 0.0);
    }

    /// 测试换行：短文本不换行。
    #[test]
    fn test_line_wrap_short_text() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("Hi", 16.0, 1000.0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].glyphs.len(), 2);
    }

    /// 测试换行：显式换行符折行。
    #[test]
    fn test_line_wrap_newline() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("A\nB", 16.0, 1000.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].glyphs.len(), 1);
        assert_eq!(lines[0].glyphs[0].code_point, 'A');
        assert_eq!(lines[1].glyphs.len(), 1);
        assert_eq!(lines[1].glyphs[0].code_point, 'B');
    }

    /// 测试换行：宽度不足时在空格处折行。
    #[test]
    fn test_line_wrap_at_space() {
        let shaper = make_empty_shaper();
        // 每个字符 advance ≈ 0.6 * 16 = 9.6
        // "A B C D" = 7 chars, 总宽度 ≈ 67.2
        // 设 max_width = 25，第 3 个字符 "B" 时超出（28.8 > 25）
        // 在空格处折行 → 应产生多行
        let lines = shaper.shape_with_line_wrap("A B C D", 16.0, 25.0);
        assert!(lines.len() >= 2, "应在空格处换行，实际 {} 行", lines.len());
    }

    /// 测试换行：空文本返回单空行。
    #[test]
    fn test_line_wrap_empty() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("", 16.0, 100.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].glyphs.is_empty());
    }

    /// 测试换行：零宽度返回空行。
    #[test]
    fn test_line_wrap_zero_width() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("Hello", 16.0, 0.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].glyphs.is_empty());
    }

    /// 测试整形：跳过换行符本身不产生 glyph。
    #[test]
    fn test_shape_newline_not_in_glyphs() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("X\nY", 16.0, 1000.0);
        // 换行符不应出现在任何行的 glyph 中
        for line in &lines {
            assert!(!line.glyphs.iter().any(|g| g.code_point == '\n'));
        }
    }

    /// 测试 ShapedLine width 字段。
    #[test]
    fn test_shaped_line_width() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("ABC", 16.0, 1000.0);
        assert_eq!(lines.len(), 1);
        let expected_width: f32 = lines[0].glyphs.iter().map(|g| g.advance_x).sum();
        assert!((lines[0].width - expected_width).abs() < 0.01);
    }

    /// 测试连续换行符产生空行。
    #[test]
    fn test_consecutive_newlines() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("A\n\nB", 16.0, 1000.0);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].glyphs.len(), 1);
        assert_eq!(lines[1].glyphs.len(), 0);
        assert_eq!(lines[2].glyphs.len(), 1);
    }

    // ── 使用真实字体的测试（有系统字体时执行）──────────

    /// 测试使用真实字体整形。
    #[test]
    fn test_shape_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        let glyphs = shaper.shape_single_line("Hello", 16.0);
        assert_eq!(glyphs.len(), 5, "应为 5 个字符生成 5 个 glyph");

        // 每个 glyph 的 advance 应该是正数
        for glyph in &glyphs {
            assert!(glyph.advance_x > 0.0, "advance_x 应为正数");
        }

        // 总宽度应该合理（5 个字符大约 40-80px）
        let total: f32 = glyphs.iter().map(|g| g.advance_x).sum();
        assert!(total > 20.0 && total < 200.0, "总宽度应合理，实际 {}", total);
    }

    /// rustybuzz cluster 使用 UTF-8 字节偏移，多字节字符后仍须映射到正确码点。
    #[test]
    fn test_rustybuzz_cluster_uses_utf8_byte_offsets() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        let code_points: Vec<char> = shaper
            .shape_single_line("éAB", 16.0)
            .into_iter()
            .map(|glyph| glyph.code_point)
            .collect();

        assert_eq!(code_points, vec!['é', 'A', 'B']);
    }

    #[test]
    fn test_ltr_shared_cluster_preserves_source_code_points() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));
        let glyphs = shaper.shape_single_line("x\u{301}", 16.0);

        assert_eq!(glyphs.len(), 2);
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.code_point).collect::<Vec<_>>(),
            vec!['x', '\u{301}']
        );
        assert_eq!(glyphs.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>(), vec![0, 0]);
    }

    #[test]
    fn test_ltr_ligature_preserves_source_cluster() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));
        let glyphs = shaper.shape_single_line("fi", 16.0);

        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].cluster, 0);
        assert_eq!(glyphs[0].code_point, 'f');
    }

    #[test]
    fn test_ligature_feature_can_be_disabled_per_run() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        let enabled = shaper.shape_single_line_with_features(
            "fi",
            16.0,
            TextDirection::LeftToRight,
            &[OpenTypeFeature::new(*b"liga", 1)],
        );
        let disabled = shaper.shape_single_line_with_features(
            "fi",
            16.0,
            TextDirection::LeftToRight,
            &[OpenTypeFeature::new(*b"liga", 0)],
        );

        assert_eq!(enabled.len(), 1, "liga=1 should form the bundled Lato fi ligature");
        assert_eq!(disabled.len(), 2, "liga=0 should retain separate f and i glyphs");
    }

    #[test]
    fn fallback_runs_preserve_resolved_face_and_absolute_clusters() {
        let Some(fallback_data) = load_system_font_data() else {
            eprintln!("skipping: no system fallback font found");
            return;
        };
        let mut loader = FontLoader::new();
        let primary = loader.load_font(LATO_TTF).expect("load bundled Lato");
        let fallback = loader.load_font(&fallback_data).expect("load system fallback");
        let primary_font = loader.get(primary).expect("primary face");
        let fallback_font = loader.get(fallback).expect("fallback face");
        let Some(ch) = ['א', 'ش', '☃', '⌘']
            .into_iter()
            .find(|ch| !primary_font.has_glyph(*ch) && fallback_font.has_glyph(*ch))
        else {
            eprintln!("skipping: no coverage difference between test faces");
            return;
        };
        loader.set_fallback_chain(vec![fallback]);
        let shaper = TextShaper::new(&loader, Some(FontId(primary)));
        assert!(
            shaper
                .shape_with_fallback_runs(
                    &[primary, fallback],
                    "f\u{200C}i",
                    16.0,
                    TextShapingOptions {
                        direction: TextDirection::LeftToRight,
                        ..TextShapingOptions::default()
                    },
                )
                .is_none(),
            "ZWNJ must remain in the primary grapheme run"
        );

        let text = format!("A{ch}B");
        let glyphs = shaper
            .shape_with_fallback_runs(
                &[primary, fallback],
                &text,
                16.0,
                TextShapingOptions {
                    direction: TextDirection::LeftToRight,
                    ..TextShapingOptions::default()
                },
            )
            .expect("fallback run");

        assert_eq!(glyphs.len(), 3);
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.font_id).collect::<Vec<_>>(),
            vec![FontId(primary), FontId(fallback), FontId(primary)]
        );
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>(),
            vec![0, 1, 1 + ch.len_utf8() as u32]
        );
        assert!(glyphs.iter().all(|glyph| glyph.glyph_id > 0 && glyph.advance_x > 0.0));
    }

    /// shaping advance 必须保留 rustybuzz 的 kerning/GPOS 结果。
    #[test]
    fn test_rustybuzz_position_is_authoritative_advance() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("should load bundled Lato font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));
        let font_data = loader
            .get_font_data(font_id)
            .expect("font bytes should remain available");
        let face = rustybuzz::Face::from_slice(font_data, 0).expect("valid bundled Lato face");
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str("AV");
        let expected = rustybuzz::shape(&face, &[], buffer)
            .glyph_positions()
            .iter()
            .map(|position| position.x_advance as f32 * 16.0 / face.units_per_em() as f32)
            .sum::<f32>();

        let glyphs = shaper.shape_single_line("AV", 16.0);
        let actual: f32 = glyphs.iter().map(|glyph| glyph.advance_x).sum();
        let unshaped: f32 = glyphs.iter().map(|glyph| glyph.unshaped_advance_x).sum();
        let isolated = measure_text_width(&shaper, "A", 16.0) + measure_text_width(&shaper, "V", 16.0);

        assert!(
            (actual - expected).abs() < 0.001,
            "actual={actual}, expected={expected}"
        );
        assert!(
            (unshaped - isolated).abs() < 0.001,
            "unshaped={unshaped}, isolated={isolated}"
        );
        assert!(actual < unshaped, "Lato AV kerning must reduce advance");
    }

    /// 测试使用真实字体的换行。
    #[test]
    fn test_line_wrap_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        // "Hello World" 在 50px 宽度内应该换行
        let lines = shaper.shape_with_line_wrap("Hello World", 16.0, 50.0);
        assert!(lines.len() >= 2, "应在 50px 内换行，实际 {} 行", lines.len());

        // 每行的实际宽度不应超过 max_width 太多
        for line in &lines {
            // 允许单字符行超过 max_width（无法再分割）
            if line.glyphs.len() > 1 {
                assert!(line.width <= 60.0, "行宽度应接近 max_width，实际 {}", line.width);
            }
        }
    }

    /// 空字符串整形应不产生任何 glyph。
    #[test]
    fn test_shaper_empty_string() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("", 16.0);
        assert!(glyphs.is_empty(), "空字符串不应产生 glyph");

        // 换行模式也应返回空 glyph
        let lines = shaper.shape_with_line_wrap("", 16.0, 1000.0);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].glyphs.is_empty(), "换行模式空字符串 glyph 应为空");
        assert_eq!(lines[0].width, 0.0);
    }

    /// 单字符整形应精确产生一个 glyph。
    #[test]
    fn test_shaper_single_character() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("A", 16.0);
        assert_eq!(glyphs.len(), 1, "单字符应产生恰好一个 glyph");
        assert_eq!(glyphs[0].code_point, 'A');
        assert_eq!(glyphs[0].glyph_id, 'A' as u32); // 无字体时 glyph_id = code_point
        assert!(glyphs[0].advance_x > 0.0, "advance_x 应为正值");
    }

    /// 测试使用真实字体测量文本宽度。
    #[test]
    fn test_measure_with_real_font() {
        let font_data = match load_system_font_data() {
            Some(data) => data,
            None => {
                eprintln!("skipping: no system font found");
                return;
            }
        };

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(&font_data).expect("should load system font");
        let shaper = TextShaper::new(&loader, Some(FontId(font_id)));

        let width_16 = measure_text_width(&shaper, "Hello", 16.0);
        let width_32 = measure_text_width(&shaper, "Hello", 32.0);

        assert!(width_16 > 0.0, "16px 文本宽度应为正");
        assert!(width_32 > width_16, "32px 应比 16px 宽: {} vs {}", width_32, width_16);
    }

    /// 测试混合 ASCII 与 CJK 字符的整形。
    ///
    /// ASCII 字符和 CJK 字符混合时，每个字符都应产生对应的 glyph，
    /// 且所有 glyph 的 advance_x 均为正值。
    #[test]
    fn test_shaper_mixed_ascii_cjk() {
        let shaper = make_empty_shaper();
        let text = "Hello世界";
        let glyphs = shaper.shape_single_line(text, 16.0);
        assert_eq!(glyphs.len(), 7, "应为 7 个字符产生 7 个 glyph");

        // 无字体时 glyph_id 等于 code_point
        assert_eq!(glyphs[0].code_point, 'H');
        assert_eq!(glyphs[0].glyph_id, 'H' as u32);
        assert_eq!(glyphs[4].code_point, 'o');
        // CJK 字符
        assert_eq!(glyphs[5].code_point, '世');
        assert_eq!(glyphs[5].glyph_id, '世' as u32);
        assert_eq!(glyphs[6].code_point, '界');
        assert_eq!(glyphs[6].glyph_id, '界' as u32);

        // 所有 advance_x 应为正
        for glyph in &glyphs {
            assert!(glyph.advance_x > 0.0, "advance_x 应为正数");
        }
    }

    /// 测试显式换行符 \\n 产生多行输出。
    ///
    /// 文本中包含多个 \n 时，应产生对应数量的行，且换行符本身不出现在 glyph 中。
    #[test]
    fn test_shaper_explicit_newline() {
        let shaper = make_empty_shaper();
        let text = "第一行\n第二行\n第三行";
        let lines = shaper.shape_with_line_wrap(text, 16.0, 1000.0);
        assert_eq!(lines.len(), 3, "应在 \\n 处产生 3 行");

        // 每行应包含对应的 CJK 字符
        assert_eq!(lines[0].glyphs.len(), 3, "第一行应有 3 个字符");
        assert_eq!(lines[1].glyphs.len(), 3, "第二行应有 3 个字符");
        assert_eq!(lines[2].glyphs.len(), 3, "第三行应有 3 个字符");

        // 换行符不应出现在任何行的 glyph 中
        for line in &lines {
            assert!(
                !line.glyphs.iter().any(|g| g.code_point == '\n'),
                "换行符不应出现在 glyph 中"
            );
        }

        // 每行宽度应为正
        for line in &lines {
            assert!(line.width > 0.0, "每行宽度应为正");
        }
    }

    /// 测试负 font_size 时 shape_single_line 不 panic 且 advance 为负值或零。
    ///
    /// 无字体时 advance = font_size * 0.6，当 font_size 为负时 advance 也为负。
    /// 验证整形器在极端输入下不崩溃。
    #[test]
    fn test_shape_negative_font_size() {
        let shaper = make_empty_shaper();
        let glyphs = shaper.shape_single_line("A", -16.0);
        assert_eq!(glyphs.len(), 1, "负 font_size 仍应产生 glyph");
        // advance = -16.0 * 0.6 = -9.6
        let expected_advance = -16.0 * 0.6;
        assert!(
            (glyphs[0].advance_x - expected_advance).abs() < 0.01,
            "负 font_size 的 advance 应为 {expected_advance}，实际 {}",
            glyphs[0].advance_x
        );
    }

    /// 测试仅含空格的文本在换行模式下产生正确的 glyph 数量。
    ///
    /// shape_single_line 会为每个空格字符生成 glyph，
    /// 但 shape_with_line_wrap 在宽度不足时会尝试在空格处折行。
    /// 验证纯空格文本不会导致无限循环或空结果。
    #[test]
    fn test_shape_with_line_wrap_spaces_only() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("   ", 16.0, 1000.0);
        assert_eq!(lines.len(), 1, "纯空格文本应产生单行");
        assert_eq!(lines[0].glyphs.len(), 3, "纯空格文本应有 3 个空格 glyph");
        // 每个空格 glyph 的 code_point 应为 ' '
        for glyph in &lines[0].glyphs {
            assert_eq!(glyph.code_point, ' ');
        }
    }

    /// 测试非常长的字符串整形不 panic
    ///
    /// 对 10000 个字符的字符串执行单行整形，验证不发生栈溢出或 panic。
    #[test]
    fn test_shape_very_long_string() {
        let shaper = make_empty_shaper();
        let long_text: String = "A".repeat(10_000);
        let glyphs = shaper.shape_single_line(&long_text, 16.0);
        assert_eq!(glyphs.len(), 10_000, "应产生 10000 个 glyph");
        // 所有 advance 应一致
        for g in &glyphs {
            assert!((g.advance_x - 16.0 * 0.6).abs() < 0.01);
        }
    }

    /// 测试 shape_with_line_wrap 使用负行宽返回空行
    ///
    /// 传入负数 max_line_width 时，shape_with_line_wrap 应返回单空行。
    #[test]
    fn test_shape_with_line_wrap_negative_width() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("Hello", 16.0, -100.0);
        assert_eq!(lines.len(), 1, "负宽度应返回单行");
        assert!(lines[0].glyphs.is_empty(), "负宽度应返回空 glyph");
    }

    /// 测试仅含换行符的文本
    ///
    /// 多个连续换行符应产生多个空行，且没有 glyph。
    #[test]
    fn test_shape_only_newlines() {
        let shaper = make_empty_shaper();
        let lines = shaper.shape_with_line_wrap("\n\n\n", 16.0, 1000.0);
        assert_eq!(lines.len(), 3, "3 个换行符应产生 3 行");
        for line in &lines {
            assert!(line.glyphs.is_empty(), "仅换行符的行不应有 glyph");
            assert_eq!(line.width, 0.0, "仅换行符的行宽度应为 0");
        }
    }

    /// 测试无空格的长文本在窄宽度下强制折行
    ///
    /// 当文本没有空格且宽度不足时，应在超限处强制折行。
    #[test]
    fn test_shape_wrap_no_spaces_narrow() {
        let shaper = make_empty_shaper();
        // "ABCDEFGH" 每个字符 advance ≈ 9.6，max_width = 20
        // 第 3 个字符后宽度 ≈ 28.8 > 20，应强制折行
        let lines = shaper.shape_with_line_wrap("ABCDEFGH", 16.0, 20.0);
        assert!(lines.len() >= 2, "无空格窄宽度应产生多行，实际 {} 行", lines.len());
        // 所有 glyph 都应存在
        let total_glyphs: usize = lines.iter().map(|l| l.glyphs.len()).sum();
        assert_eq!(total_glyphs, 8, "总共应有 8 个 glyph");
    }

    /// 测试 Unicode 特殊字符（表情符号、控制字符）整形不 panic
    ///
    /// 包含零宽连接符、组合字符等特殊 Unicode 字符的文本应能整形，
    /// 每个码点对应一个 glyph。
    #[test]
    fn test_shape_unicode_special_characters() {
        let shaper = make_empty_shaper();
        let text = "a\u{0308}\u{20DD}"; // a + 组合分音符 + 组合圆圈
        let glyphs = shaper.shape_single_line(text, 16.0);
        assert_eq!(glyphs.len(), 3, "3 个码点应产生 3 个 glyph");
        // 无字体时 glyph_id 等于 code_point
        assert_eq!(glyphs[0].glyph_id, 'a' as u32);
    }
}
