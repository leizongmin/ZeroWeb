//! 文本绘制主流程；列表、multicol、ruby 与 shaping 辅助位于 `text/` 子模块。

use std::collections::HashMap;

use zero_css_parser::values::types::FontStyleValue;
use zero_css_parser::values::{ColorValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::inline_finalization::{
    build_text_parent_override_map, resolve_text_align, resolve_text_align_last, resolve_text_indent,
    resolve_word_break_mode, subtree_has_text_decoration,
};
use zero_layout_engine::{FloatExclusion, InlineFormattingContext, LayoutBox};
use zero_render_foundation::color::Color;
use zero_render_foundation::font::TextDirection;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, TextControlBoundary};
use zero_style_system::{
    ComputedStyle, TabSizeValue, TextEmphasisPositionValue, TextEmphasisStyleValue, TextOverflowValue,
    TextTransformValue, WhiteSpaceValue,
};

use super::super::color::{color_value_to_render, resolve_color_current};
use super::super::helpers::PrimitiveCounts;
use super::super::helpers::apply_text_transform;
use super::text_image::{compute_object_fit_rect, get_img_intrinsic_size};

// 专属 helper 与单测按职责拆入 painter/text/ 子模块。
pub(crate) mod text_list;
mod text_multicol;
mod text_ruby;
mod text_shaping;

use text_multicol::compute_multicol_info_for_paint;
use text_multicol::multicol_balance_target_height;
use text_ruby::ruby_annotation_segments;
use text_shaping::{
    FragmentPaintWidths, ahem_uses_embox_position, collect_atomic_inline_sizes,
    configure_paint_ifc_advance as with_shaped_layout, fragment_advance_trace, fragment_font_size_adjustment,
    fragment_glyphs, glyph_sources, is_cc_control_char, logical_fragment_source, paint_ifc_baseline_offset,
    style_open_type_features,
};

impl super::Painter {
    /// 收集浮动子元素的排除区域（带样式映射版本）。
    ///
    /// 遍历 `box_node` 的直接子元素，找出带有 `float: left/right` 样式的子元素，
    /// 计算它们相对于容器内容区域的位置和尺寸。
    pub(super) fn collect_float_exclusions_with_styles(
        &self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<FloatExclusion> {
        let mut exclusions = Vec::new();

        // 容器内容区域的原点 y 偏移（浮动排除区域相对于内容区域顶部计算）
        let content_offset_y = box_node.border_top + box_node.padding_top;

        for child in &box_node.children {
            // 跳过绝对定位子元素（不参与浮动流）
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if let Some(node_id) = child.node_id
                && let Some(child_style) = styles.get(&node_id)
            {
                let is_left = matches!(child_style.float, FloatValue::Left | FloatValue::InlineStart);
                let is_right = matches!(child_style.float, FloatValue::Right | FloatValue::InlineEnd);

                if is_left || is_right {
                    // 浮动子元素相对于容器内容区域的位置
                    let rel_y = child.y - content_offset_y;
                    exclusions.push(FloatExclusion {
                        y: rel_y,
                        height: child.height,
                        width: child.width,
                        is_left,
                    });
                }
            }
        }

        exclusions
    }

    /// 绘制列表标记（disc/circle/square/decimal 等）。
    /// 绘制 CSS `content` 属性生成的文本内容。
    ///
    /// 当元素的 `content` 属性为 `String` 或 `Counter` 时，
    /// 在元素的内容区域起始位置绘制对应的文本。
    /// 支持计数器值的十进制、小写字母、大写字母、小写罗马、大写罗马格式化。
    pub(crate) fn paint_content(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        // R2573：content 文本解析抽出为 resolve_generated_content_text（paint_content 与
        // paint_list_marker 的 ::marker content 覆盖共用）。Normal/None/Attr/Url 无文本 → None。
        let text = match self.resolve_generated_content_text(&style.content) {
            Some(t) => t,
            // R1988：content:url() 由 inject_pseudo_text_nodes 注入 `<img>` 元素渲染，paint_content
            //（文本路径）不处理图片；Normal/None/Attr 亦无文本 → 均返回。
            None => return,
        };

        if text.is_empty() {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = super::super::color::color_value_to_render(&style.color);
        let (default_font_id, resolved_italic) = self.resolve_style_font_id(&style.font_family, style);
        let variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&variations);
        // R2497：font-style:italic/oblique 且 resolved face 非 italic → synthetic italic shear。
        // R3248：font-synthesis:style 为 false 时禁止合成斜体。
        let synthetic_italic = matches!(style.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_))
            && !resolved_italic
            && style.font_synthesis.style;
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let mut char_x = content_x;
        let char_y = content_y + font_size;
        for ch in text.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: char_y,
                font_size,
                color,
                glyph_id: ch as u32,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic,
            });
            char_x += self.measure_char_cached(default_font_id.0, ch, font_size, false);
        }
    }

    /// R1660：`<input>` 的 `value` 渲染为可见文本（form-control slice-2）。
    ///
    /// `<input>` 是 void 元素（无 DOM 文本子节点），其 `value` 属性的标签/预填内容此前
    /// 不渲染——submit/reset 按钮无可见文字、text 输入框的预填值不可见。R1659 已按 value
    /// 字符数 / `size` 属性给 input 正确几何宽；本方法在 paint 侧把 value 文本绘出，对齐
    /// Chromium UA 语义：
    /// - submit/reset/button：value（submit/reset 无 value 时默认 "Submit"/"Reset"）水平居中
    ///   （按钮标签居中）
    /// - text 类（text/search/email/url/tel/number + 默认无 type）：value 左对齐于内容盒
    /// - password：每字符渲染为 `•`（密码遮罩语义）
    /// - checkbox/radio/hidden/range/file/image/color/date 等：不渲染 value 文本
    ///
    /// 几何宽由 R1659 UA sizing 决定；此处只补 paint。值超 content 宽时自然溢出（与 chromium
    /// overflow 一致，本 slice 不做 ellipsis）。
    pub(crate) fn paint_input_value(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };
        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };
        let elem = match &node.kind {
            NodeKind::Element(e)
                if e.local_name().eq_ignore_ascii_case("input") || e.local_name().eq_ignore_ascii_case("textarea") =>
            {
                e
            }
            _ => return,
        };

        let itype = elem.get_attribute("type").unwrap_or_default().to_ascii_lowercase();
        let retained_value = self.form_control_values.get(&node_id).cloned();
        let default_value = if elem.local_name().eq_ignore_ascii_case("textarea") {
            doc.text_content(node_id).unwrap_or_default()
        } else {
            elem.get_attribute("value").unwrap_or_default()
        };
        let mut current_value = retained_value.unwrap_or(default_value);
        if let Some((text, start, end)) = self.form_control_compositions.get(&node_id) {
            current_value = replace_utf16_range(&current_value, *start, *end, text);
        }
        let boundary_value = current_value.clone();
        let current_value_empty = current_value.is_empty();
        // 决定渲染标签 + 是否水平居中。
        let (mut label, center): (String, bool) = if elem.local_name().eq_ignore_ascii_case("textarea") {
            if !self.form_control_values.contains_key(&node_id)
                && !self.form_control_compositions.contains_key(&node_id)
            {
                if current_value.is_empty() {
                    (elem.get_attribute("placeholder").unwrap_or_default(), false)
                } else {
                    return;
                }
            } else {
                (current_value, false)
            }
        } else {
            match itype.as_str() {
                "submit" => (
                    elem.get_attribute("value").unwrap_or_else(|| "Submit".to_string()),
                    true,
                ),
                "reset" => (elem.get_attribute("value").unwrap_or_else(|| "Reset".to_string()), true),
                "button" => (elem.get_attribute("value").unwrap_or_default(), true),
                "password" => (current_value.chars().map(|_| '\u{2022}').collect(), false),
                // 文本类（默认无 type 当 text）。
                "" | "text" | "search" | "email" | "url" | "tel" | "number" => (current_value, false),
                // checkbox/radio/hidden/range/file/image/color/date/... 不渲染 value 文本。
                _ => return,
            }
        };
        let placeholder = elem.get_attribute("placeholder");
        if label.is_empty()
            && let Some(placeholder) = placeholder.as_ref()
        {
            label = placeholder.clone();
        }
        let is_placeholder =
            current_value_empty && placeholder.as_deref().is_some_and(|placeholder| placeholder == label);
        let has_text_boundaries = elem.local_name().eq_ignore_ascii_case("textarea")
            || matches!(itype.as_str(), "" | "text" | "search" | "url" | "tel" | "password");
        if label.is_empty() && !has_text_boundaries {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = if is_placeholder {
            zero_render_foundation::color::Color {
                r: 117,
                g: 117,
                b: 117,
                a: 255,
            }
        } else {
            super::super::color::color_value_to_render(&style.color)
        };
        let (default_font_id, synthetic_italic) = self.resolve_style_font_id(&style.font_family, style);

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        // R3254-M11：textarea 值可含 '\n'——按行拆分绘制（此前把 '\n' 当普通 glyph 画成
        // 单行粘连）。line-height 简化为 1.2em；整个文本块在 content box 内垂直居中
        //（单行 input 行为与原实现一致）。font box 垂直居中于行盒：
        // https://drafts.csswg.org/css-inline-3/#line-height-property
        let line_height = font_size * 1.2;
        let lines: Vec<&str> = label.split('\n').collect();
        let boundary_lines: Vec<&str> = boundary_value.split('\n').collect();
        let node_handle = crate::hit_test::node_id_to_u64(node_id);
        let text_direction = match style.direction {
            zero_style_system::DirectionValue::Ltr => TextDirection::LeftToRight,
            zero_style_system::DirectionValue::Rtl => TextDirection::RightToLeft,
        };
        let shape_features = style_open_type_features(style);
        let shape_variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&shape_variations);
        let shape_adjustment = crate::text_metrics::font_size_adjustment(&style.font_size_adjust);
        let mut line_utf16_start = 0usize;
        let block_height = lines.len() as f32 * line_height;
        let block_top = content_y + (box_node.content_height - block_height).max(0.0) / 2.0;
        for (line_index, line) in lines.iter().enumerate() {
            let line_top = block_top + line_index as f32 * line_height;
            let baseline_y = line_top + (line_height - font_size).max(0.0) / 2.0 + font_size
                - if is_placeholder && !elem.local_name().eq_ignore_ascii_case("textarea") {
                    font_size * 0.875
                } else {
                    0.0
                };
            let shaped = (has_text_boundaries
                && !is_placeholder
                && itype != "password"
                && matches!(text_direction, TextDirection::LeftToRight))
            .then(|| {
                crate::shape_text_for_paint(
                    &[default_font_id.0],
                    line,
                    font_size,
                    text_direction,
                    &shape_features,
                    &shape_variations,
                    shape_adjustment,
                )
            })
            .flatten();
            // 居中按钮标签：先测总宽再定起始 x。
            let total_w: f32 = shaped
                .as_ref()
                .map(|glyphs| glyphs.iter().map(|glyph| glyph.advance_x).sum())
                .unwrap_or_else(|| {
                    line.chars()
                        .map(|ch| self.measure_char_cached(default_font_id.0, ch, font_size, false))
                        .sum()
                });
            let mut char_x = if center {
                content_x + (box_node.content_width - total_w).max(0.0) / 2.0
            } else {
                content_x
            };
            let boundary_line = boundary_lines.get(line_index).copied().unwrap_or_default();
            let mut utf16_offset = line_utf16_start;
            if has_text_boundaries {
                self.primitives.add_text_control_boundary(TextControlBoundary {
                    node_handle,
                    utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                    x: char_x,
                    y: line_top,
                    height: line_height,
                });
            }
            if let Some(shaped) = shaped {
                let sources = glyph_sources(line, &shaped, true).unwrap_or_else(|| vec![None; shaped.len()]);
                let mut cluster = None;
                for (glyph, source) in shaped.iter().zip(sources) {
                    if cluster != source.as_ref().map(|source| (source.start, source.end))
                        && let Some((_, previous_end)) = cluster
                    {
                        utf16_offset = line_utf16_start + line[..previous_end as usize].encode_utf16().count();
                        self.primitives.add_text_control_boundary(TextControlBoundary {
                            node_handle,
                            utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                            x: char_x,
                            y: line_top,
                            height: line_height,
                        });
                    }
                    cluster = source.as_ref().map(|source| (source.start, source.end));
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x + glyph.x_offset,
                        y: baseline_y - glyph.y_offset,
                        font_size: glyph.font_size,
                        color,
                        glyph_id: glyph.code_point as u32,
                        font_glyph_index: u16::try_from(glyph.glyph_id).ok(),
                        source,
                        font_id: glyph.font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic,
                    });
                    char_x += glyph.advance_x;
                }
                if let Some((_, end)) = cluster {
                    utf16_offset = line_utf16_start + line[..end as usize].encode_utf16().count();
                    self.primitives.add_text_control_boundary(TextControlBoundary {
                        node_handle,
                        utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                        x: char_x,
                        y: line_top,
                        height: line_height,
                    });
                }
            } else {
                let mut source_chars = boundary_line.chars();
                for ch in line.chars() {
                    let advance = self.measure_char_cached(default_font_id.0, ch, font_size, false);
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: baseline_y,
                        font_size,
                        color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic,
                    });
                    char_x += advance;
                    utf16_offset += source_chars.next().map(char::len_utf16).unwrap_or(1);
                    if has_text_boundaries && !is_placeholder {
                        self.primitives.add_text_control_boundary(TextControlBoundary {
                            node_handle,
                            utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                            x: char_x,
                            y: line_top,
                            height: line_height,
                        });
                    }
                }
            }
            line_utf16_start += boundary_line.encode_utf16().count();
            if line_index + 1 < boundary_lines.len() {
                line_utf16_start += 1;
            }
        }
    }

    /// 绘制 `<img>` 元素，根据 `object-fit` 属性决定图片如何适配容器。
    ///
    /// - `fill`：拉伸图片填满容器（默认）
    /// - `contain`：等比缩放，完整显示图片
    /// - `cover`：等比缩放，完全覆盖容器
    /// - `none`：原始尺寸
    /// - `scale-down`：取 none 和 contain 中较小的结果
    pub(crate) fn paint_img_element(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };

        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };

        // 获取图片 URL 作为键。R2439：`content:url(...)` 优先（element-becomes-replaced，
        // 覆盖任何元素含 `<img>` 的正常内容）——src 取自 style.content 的 Url；否则 `<img>`
        // 用 src 属性（src 缺失回退 srcset 首 URL，R2419）。build_subtree 已抑制 content:url
        // 元素的子节点，pipeline 已按 image 固有尺寸 sizing。
        let src = if let zero_style_system::property::types::ContentComputedValue::Url(u) = &style.content {
            u.clone()
        } else {
            match &node.kind {
                NodeKind::Element(elem) if elem.local_name() == "img" => {
                    let s = elem.get_attribute("src").unwrap_or_default();
                    if s.is_empty() {
                        elem.get_attribute("srcset")
                            .and_then(|s| crate::srcset_first_url(&s))
                            .unwrap_or_default()
                    } else {
                        s
                    }
                }
                _ => return,
            }
        };
        if src.is_empty() {
            return;
        }

        let container_w = box_node.content_width;
        let container_h = box_node.content_height;
        if container_w <= 0.0 || container_h <= 0.0 {
            return;
        }

        // 尝试获取图片的固有尺寸（从 width/height 属性或回退到容器尺寸）
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let image_hash = super::super::helpers::image_resource_key(&src, self.document_url.as_deref());
        let image_key = ImageKey::new(image_hash);

        // 与布局阶段保持一致：优先使用解码后的真实图片尺寸；若图片未进入缓存，再回退到
        // HTML width/height 属性，最后才退回容器尺寸。
        let decoded_size = self.get_image_size(image_hash);
        let (intrinsic_w, intrinsic_h) = get_img_intrinsic_size(node, decoded_size, container_w, container_h);

        let (img_x, img_y, img_w, img_h) = compute_object_fit_rect(
            &style.object_fit,
            &style.object_position,
            container_w,
            container_h,
            intrinsic_w,
            intrinsic_h,
            content_x,
            content_y,
        );

        self.primitives.add_image(ImagePrimitive {
            rect: Rect::new(img_x, img_y, img_w, img_h),
            image_key,
            clip: Some(Rect::new(content_x, content_y, container_w, container_h)),
        });
    }

    /// 绘制文本内容（生成多字符 GlyphPrimitive）。
    pub fn paint_text(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: Option<&Document>,
        styles: Option<&HashMap<NodeId, ComputedStyle>>,
    ) {
        if let (Some(node_id), Some(doc)) = (box_node.node_id, doc)
            && (self.form_control_values.contains_key(&node_id)
                || self.form_control_compositions.contains_key(&node_id))
            && doc.get(node_id).is_some_and(
                |node| matches!(&node.kind, NodeKind::Element(element) if element.local_name().eq_ignore_ascii_case("textarea")),
            )
        {
            return;
        }
        let abs_y = if matches!(
            style.appearance,
            zero_style_system::AppearanceComputedValue::Button
                | zero_style_system::AppearanceComputedValue::PushButton
                | zero_style_system::AppearanceComputedValue::SquareButton
        ) && box_node.node_id.is_some_and(|node_id| {
            doc.and_then(|doc| doc.get(node_id)).is_some_and(
                |node| matches!(&node.kind, NodeKind::Element(element) if element.local_name().eq_ignore_ascii_case("button")),
            )
        }) {
            abs_y + 2.0
        } else {
            abs_y
        };
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };

        if font_size <= 0.0 {
            return;
        }

        if style.color == ColorValue::CurrentColor {
            return;
        }

        let color = color_value_to_render(&style.color);

        let letter_spacing: f32 = match style.letter_spacing {
            LengthValue::Px(s) => s as f32,
            _ => 0.0,
        };
        let word_spacing: f32 = match style.word_spacing {
            LengthValue::Px(s) => s as f32,
            _ => 0.0,
        };

        // R2305：text-shadow 多阴影列表（CSS Text Decoration §3：`none | <shadow>#`）。
        // 预解析非零阴影（offset/blur 全零 = 不可见，跳过；与既有 has_text_shadow 语义一致），
        // 颜色预解析避免逐 glyph 重复解析。空 Vec = none。
        // R2364：颜色按元素 `color` 解析 currentColor（省略颜色默认 currentColor，CSS Text Deco §3）。
        let active_text_shadows: Vec<(f32, f32, Color)> = style
            .text_shadow
            .iter()
            .filter(|ts| !(ts.offset_x == 0.0 && ts.offset_y == 0.0 && ts.blur_radius == 0.0))
            .map(|ts| (ts.offset_x, ts.offset_y, resolve_color_current(&ts.color, &style.color)))
            .collect();

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        // R1717：+ valign_offset — 表格单元格文本的 vertical-align 内容偏移（仅 table-cell，
        // table.rs position_cells 设置；其他盒默认 0.0，零影响）。
        let content_y = abs_y + box_node.border_top + box_node.padding_top + box_node.valign_offset;

        let (tx, ty) = super::super::helpers::apply_transform_offset(style, abs_x, abs_y);

        let (default_font_id, default_resolved_italic) = self.resolve_style_font_id(&style.font_family, style);
        let default_variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let default_font_variation_id = self.primitives.intern_font_variations(&default_variations);
        // R2497：容器 font-style:italic/oblique → container_want_italic（macro 据 owner
        // per-fragment font_style 覆盖，缺省回落此值）。
        let container_want_italic = matches!(style.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
        // R1224：Ahem font_id 供 inline 元素字体≠容器时字形位图用（如 <span font:Ahem> 在
        // default div 内）。render_fragment macro 按 owner（片段父元素）font_family 选
        // frag_font_id——is_ahem 片段用 ahem_font_id 出 Ahem 方块，非 is_ahem 用 default。
        let ahem_font_id = self.resolve_style_font_id(&["Ahem".to_string()], style).0;
        // R1464/R2497：恢复 Path B 每节点 face/list 与 synthetic-italic 状态。
        let resolved_text_fonts = self.resolve_text_node_fonts(box_node, style);
        let text_node_font_ids = resolved_text_fonts.primary;
        let text_node_shaping_font_ids = resolved_text_fonts.shaping;
        let text_node_font_italic = resolved_text_fonts.italic;

        if let (Some(doc), Some(node_id)) = (doc, box_node.node_id) {
            // R109 §9.2.1.1：被 in-flow block 子元素拆分的 inline 父盒自身不渲染文本——
            // 其直接文本已由匿名块片段子盒（带 fragment_node_ids）渲染。避免与片段重叠。
            if box_node.is_r109_split && box_node.fragment_node_ids.is_none() {
                return;
            }
            if !has_direct_paintable_text(doc, node_id, styles) {
                return;
            }
            // R109：匿名块片段跳过 painted_inline_nodes 去重——多个片段共享 inline 的
            // node_id，首个片段渲染后会标记该 id，须放行后续片段各自渲染其片段文本。
            // R1548/R1549：vertical 容器内全 inset auto 的 abspos/fixed 盒由
            // fix_vertical_mode_abs_pos 在 postprocess 期精确定位（IFC 静态位置 + height
            // shrink-to-fit）+ 存入自身 font metrics（R1548），其文本须由自身 paint_text
            // 绘制。但其 containing-block 的 IFC 也会收集该文本片段（用于静态位置计算）并以
            // 容器色（R335/R358，常 transparent）绘在静态流位置 + 标记 node painted → 抑制
            // abspos 盒自身绘制 → 文本消失（R1547 root cause）。判据 = abspos 盒的
            // text_node_font_sizes 非空（**仅** fix_vertical_mode_abs_pos 会为 abspos 子盒
            // 填充它 = vertical 容器 + 全 inset auto + 盒位已修正；覆盖 span 继承 vertical 与
            // span 显式 horizontal-tb 两类）。非 auto inset 的 abspos 盒无此填充（定位未修正），
            // 仍走去重（旧行为）——避免绘在 taffy 错误位（vrl-038/042/044/048 top:1em）。
            let abspos_self_paint =
                (box_node.is_absolute || box_node.is_fixed) && !box_node.text_node_font_sizes.is_empty();
            if box_node.fragment_node_ids.is_none()
                && !abspos_self_paint
                && self.painted_inline_nodes.contains(&node_id)
            {
                return;
            }

            // R1099 Slice α-1（vertical-mode IFC 四层协调）：container_width WM-aware。
            // vertical-rl/lr 下 IFC 重跑须与 layout 侧（inline_finalization.rs）同取 content_height
            //（竖直 inline 尺寸 = 字符向下推进可用深度），非 content_width。horizontal-tb 零回归。
            // decoration-gate（TBD-2）：vertical 容器子树有 text-decoration/emphasis 时保持
            // content_width（旧行为），回避 Layer 4 装饰坐标耦合（α-3 未实施）。
            let is_vertical_wm = matches!(
                style.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
            let vertical_decoration_free = styles.is_some_and(|s| {
                box_node
                    .node_id
                    .is_some_and(|id| !subtree_has_text_decoration(doc, s, id))
            });
            let container_width = if is_vertical_wm && vertical_decoration_free {
                box_node.content_height
            } else {
                box_node.content_width
            };

            // 检测是否为多列容器（无块级子元素但有 inline 内容）
            // 如果是，使用列宽创建 IFC，并在渲染时将行分配到各列。
            // 条件：
            // 1. 无 inflow 子元素（纯 inline 内容）
            // 2. column-fill: balance（默认值，非 auto 顺序填充）
            // 注意：对于纯 inline 内容，有明确高度时 balance 模式仍需分配到各列。
            // column-fill: auto 的 inline 内容由 layout 层处理（有 height 限制时），
            // 此处仅处理 balance 模式（无论有无 height）。
            let has_in_flow_children = box_node
                .children
                .iter()
                .any(|c| !c.is_absolute && !c.is_fixed && c.is_block_level);
            let is_balance_mode = !matches!(style.column_fill, zero_style_system::ColumnFillComputedValue::Auto);
            // 仅对 height:auto 的纯行内 multicol 容器做列分配。明确高度的 balance 容器
            // （常见于嵌套 multicol / column-breaking 测试）涉及 column breaking，
            // 当前简单均衡分配会回归这类用例，回退到单块渲染。
            let height_auto = matches!(style.height, LengthValue::Auto);
            let multicol_info = if !has_in_flow_children && is_balance_mode && height_auto {
                compute_multicol_info_for_paint(style, container_width)
            } else {
                None
            };

            // 多列容器使用列宽创建 IFC
            let ifc_width = if let Some(ref mc) = multicol_info {
                mc.col_width
            } else {
                container_width
            };

            // R2577：word-break: break-word（CSS Text 3 legacy）≡ overflow-wrap: break-word。
            let break_word = matches!(
                style.overflow_wrap,
                zero_style_system::OverflowWrapValue::BreakWord | zero_style_system::OverflowWrapValue::Anywhere
            ) || matches!(style.word_break, zero_style_system::WordBreakValue::BreakWord);

            // 根据 white-space 属性设置换行和空白保留行为
            let (mut no_wrap, preserve_whitespace, break_at_newline) = match style.white_space {
                WhiteSpaceValue::Normal => (false, false, false),
                WhiteSpaceValue::Nowrap => (true, false, false),
                WhiteSpaceValue::Pre => (true, true, false),
                WhiteSpaceValue::PreWrap => (false, true, false),
                // pre-line：空白序列折叠但 `\n` 强制断行（CSS Text 3 §4.2）。
                // kill-switch ZW_PRELINE_NEWLINE_BREAK=0 恢复旧行为（与 inline_finalization 对称）。
                WhiteSpaceValue::PreLine => (
                    false,
                    false,
                    std::env::var("ZW_PRELINE_NEWLINE_BREAK").as_deref() != Ok("0"),
                ),
                WhiteSpaceValue::BreakSpaces => (false, true, false),
            };

            // CSS text-wrap: nowrap 覆盖换行行为
            if let Some(wrap_override) = super::Painter::resolve_text_wrap(style) {
                no_wrap = wrap_override;
            }

            // CSS line-clamp: 限制最大行数
            let max_lines = super::Painter::resolve_line_clamp(style);

            // R2191：word-break + line-break:anywhere 经共享 resolver（与 layout Path A 同源）。
            let word_break_mode = resolve_word_break_mode(style);

            // R958 双路径同源：text-align / text-align-last 经 layout 路径的共享 resolver
            //（inline_finalization::resolve_text_align[_last]）解析，消除 paint Path B 此前内联的
            // 重复 match（与 layout Path A 两份独立拷贝，曾潜伏 IFC Path A/B 分歧风险）。start/end
            // 方向感知（CSS Text 3 §6.1/§6.2）、Auto = 跟随 text-align，均由 resolver 统一实现。
            let text_align = resolve_text_align(Some(style));
            let text_align_last = resolve_text_align_last(Some(style));

            // R958 双路径同源：text-indent 经 layout 路径的共享 resolver（resolve_text_indent）解析，
            // 消除 paint Path B 此前内联的重复 match（与 layout Path A 两份独立拷贝，曾潜伏 IFC Path A/B
            // 分歧风险）。Px / Em（×font_size）/ Percentage（×container_width），CSS §10.3.1。Path B 的
            // font_size 已在 line 361 保证为 style.font_size 的 Px（非 Px 早 return），故与 resolver 内部
            // font_size_px（同取 style.font_size Px，16.0 防御回退在此不可达）等价。
            let text_indent_px = resolve_text_indent(&style.text_indent, &style.font_size, container_width);

            // CSS tab-size — 制表符展开宽度
            // Number(n) 表示 n 个空格宽度，Length 表示具体像素值
            let tab_size_px: f32 = match &style.tab_size {
                TabSizeValue::Number(n) => {
                    // 空格宽度约 font_size * 0.25，乘以空格数
                    *n as f32 * font_size * 0.25
                }
                TabSizeValue::Length(LengthValue::Px(v)) => *v as f32,
                TabSizeValue::Length(LengthValue::Em(v)) => *v as f32 * font_size,
                _ => font_size * 0.25 * 8.0, // 默认 8 个空格宽度
            };

            // 收集浮动子元素的排除区域
            let float_exclusions = styles
                .map(|s| self.collect_float_exclusions_with_styles(box_node, s))
                .unwrap_or_default();

            let is_vertical = matches!(
                style.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
            let is_vertical_rtl = matches!(style.writing_mode, zero_style_system::WritingModeValue::VerticalRl);

            // 尝试使用布局引擎存储的行内布局结果，避免重新运行 IFC。
            // 条件：(1) 非多列模式 (2) 有存储结果 (3) 容器宽度匹配
            // 宽度验证确保 table/multicol 后处理改变宽度时回退到 paint IFC。
            let width_matches = (box_node.inline_layout_width - ifc_width).abs() < 1.0;
            let use_stored = multicol_info.is_none() && box_node.inline_layout.is_some() && width_matches;

            // 从存储结果创建的扁平化片段列表（用于非多列渲染路径）
            struct PaintFragment {
                x: f32,
                y: f32,
                // R817 Phase 2：片段基线绝对 y（container-rel = line.y + line.baseline_y）。
                // 供 is_ahem glyph 定位用（见 stored 渲染循环），paint 非存储路径不读。
                baseline_y_abs: f32,
                width: f32,
                height: f32,
                font_size: f32,
                is_ahem: bool,
                is_ahem_font: bool,
                text: String,
                source: Option<zero_layout_engine::TextFragmentSource>,
                node_id: NodeId,
            }

            let stored_fragments: Vec<PaintFragment> = if use_stored {
                box_node
                    .inline_layout
                    .as_ref()
                    .unwrap()
                    .iter()
                    .flat_map(|line| {
                        // R355：多行存储需把行盒垂直偏移（line.y）加到片段 y 上——
                        // 存储片段 f.y 是行内相对（恒为 0），line.y 才是行盒在容器内的位置。
                        // R207 单行存储时 line.y==0 故无影响；R355 多行若不加 line.y，
                        // 所有行渲染在容器顶部 y=0 互相覆盖（ifc-008 底半红露白）。
                        let line_y = line.y;
                        line.fragments.iter().filter_map(move |f| {
                            f.node_id.map(|nid| PaintFragment {
                                x: f.x,
                                // R1456：垂直模式下 line.y 是**列 x 坐标**（inline/mod.rs:1551
                                // vertical_rtl 轴交换把列 x 存进 col.y/line.y），已在 f.x（= run.x
                                // = 列 x）中体现，**不可**再加到片段 y（深度）。旧行为 line_y+f.y
                                // 把列 x（如 764）误加到深度（0）→ frag_y=764 → 文本推到 viewport
                                // 外（vrl-011 等全 0 可见）。horizontal 仍 line_y+f.y（line.y 是行盒
                                // y 偏移，R355）。WM gate 零回归。
                                y: if is_vertical { f.y } else { line_y + f.y },
                                baseline_y_abs: if is_vertical {
                                    f.baseline_y
                                } else {
                                    line_y + f.baseline_y
                                },
                                width: f.width,
                                height: f.height,
                                font_size: f.font_size,
                                is_ahem: f.is_ahem,
                                is_ahem_font: f.is_ahem_font,
                                text: f.text.clone(),
                                source: f.source.clone(),
                                node_id: nid,
                            })
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // 非存储模式下运行 IFC
            let inline_ctx = if use_stored {
                InlineFormattingContext::new(ifc_width)
            } else {
                // R72: 恢复 override maps 机制。
                // 传递真实 styles 会导致 4 个测试回归（BFC-004, font-feature-002,
                // position-absolute-in-inline-005/006），虽然修复了 float-003。
                // override maps 方式是经过 R37-R71 验证的安全路径。
                // 仅纳入文本节点片段构建父级映射。
                // text_node_* 中混入了内联元素片段（如 <img>，其 font_size=0、height=96），
                // 它们与文本片段共享同一父元素；直接 collect 时 last-write-wins，
                // 结果随 HashMap 迭代顺序（每进程随机）变化 → 渲染非确定性（flaky reftest）。
                // 过滤为纯文本节点后，同一父元素的文本节点继承一致的字号/行高，结果确定。
                let is_text = |tn: zero_dom::NodeId| matches!(doc.get(tn).map(|n| &n.kind), Some(NodeKind::Text(_)));
                let parent_font_sizes: HashMap<zero_dom::NodeId, f32> =
                    build_text_parent_override_map(doc, &box_node.text_node_font_sizes);

                let parent_is_ahem: HashMap<zero_dom::NodeId, bool> = box_node
                    .text_node_is_ahem
                    .iter()
                    .filter_map(|(&tn, &is_ahem)| {
                        if is_text(tn) {
                            // 文本节点：键改写为其父元素（直接文本路径）。
                            doc.parent_node(tn).map(|pid| (pid, is_ahem))
                        } else {
                            // 已是元素（如 multicol col_ctx 把 inline 元素文本扁平化为
                            // node_id=元素 的片段）：键即元素自身（paint IFC flatten 路径
                            // 按 child_id=元素 查询）。R1446：multicol-basic span 文本经
                            // flatten 收集，is_ahem 须能传到 paint IFC 测宽。
                            Some((tn, is_ahem))
                        }
                    })
                    .collect();

                let parent_letter_spacing: HashMap<zero_dom::NodeId, f32> =
                    build_text_parent_override_map(doc, &box_node.text_node_letter_spacing);

                let parent_line_heights: HashMap<zero_dom::NodeId, f32> =
                    build_text_parent_override_map(doc, &box_node.text_node_line_heights);

                // R1012：text-transform 覆盖（re-key 文本节点 → 父元素），让 paint Path B
                // 空 styles IFC 也能在 collect_inline_items 期应用 transform，使行断用
                // 转换后文本宽度（与 layout IFC / chromium 一致）。None 不插入（保持默认）。
                let parent_text_transforms: HashMap<zero_dom::NodeId, TextTransformValue> = box_node
                    .text_node_text_transform
                    .iter()
                    .filter_map(|(&tn, &tt)| {
                        if !is_text(tn) || matches!(tt, TextTransformValue::None) {
                            return None;
                        }
                        doc.parent_node(tn).map(|pid| (pid, tt))
                    })
                    .collect();

                let inline_metrics = box_node.inline_element_metrics.clone();
                let margin_overrides = box_node.inline_element_margins.clone();

                let mut ctx = InlineFormattingContext::new(ifc_width)
                    // ZRG-2026-08-15 修复 A：paint IFC 也注入 font resolver——否则
                    // run.font_id=None，行断回退 estimate（布局↔绘制宽度错位）。
                    .with_font_resolver(std::rc::Rc::new(self.font_resolver.clone()))
                    .with_text_align(text_align)
                    .with_text_align_last(text_align_last)
                    .with_bidi_override_direction(zero_layout_engine::bidi_override_direction(style))
                    .with_plaintext_bidi(
                        matches!(style.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext),
                        matches!(
                            style.text_align,
                            zero_style_system::property::types::TextAlignValue::Start
                        ),
                    )
                    .with_plaintext_bidi_overrides(box_node.plaintext_bidi_nodes.clone())
                    .with_break_word(break_word)
                    .with_no_wrap(no_wrap)
                    .with_preserve_whitespace(preserve_whitespace)
                    .with_break_at_newline(break_at_newline)
                    .with_word_break(word_break_mode)
                    .with_text_autospace(style.text_autospace)
                    .with_text_indent(text_indent_px)
                    .with_float_exclusions(float_exclusions)
                    .with_tab_size(tab_size_px)
                    .with_vertical(is_vertical)
                    .with_vertical_rtl(is_vertical_rtl)
                    .with_block_extent(
                        if is_vertical
                            && styles.is_some_and(|s| {
                                box_node.node_id.is_some_and(|id| {
                                    s.get(&id).is_some_and(|st| {
                                        matches!(st.display, zero_css_parser::values::DisplayValue::TableCaption)
                                    })
                                })
                            })
                        {
                            box_node.content_width
                        } else {
                            container_width
                        },
                    )
                    .with_font_size_overrides(parent_font_sizes)
                    .with_is_ahem_overrides(parent_is_ahem)
                    .with_letter_spacing_overrides(parent_letter_spacing)
                    .with_line_height_overrides(parent_line_heights)
                    .with_text_transform_overrides(parent_text_transforms)
                    .with_inline_element_metrics(inline_metrics)
                    .with_margin_overrides(margin_overrides)
                    .with_inline_block_sizes(collect_atomic_inline_sizes(box_node, styles));
                ctx = with_shaped_layout(
                    ctx,
                    doc,
                    styles,
                    &text_node_font_ids,
                    &text_node_shaping_font_ids,
                    &box_node.text_node_font_size_adjust,
                    &self.generic_font_ids,
                );
                // R109 §9.2.1.1：匿名块盒片段——若此盒是 inline 被 block 子元素拆分后的
                // 匿名块片段，只收集该片段的 inline 内容（而非 inline 元素的全部子节点）。
                if let Some(ref frag) = box_node.fragment_node_ids {
                    ctx.set_fragment_node_ids(frag.clone());
                }
                ctx.layout(doc, node_id, &HashMap::new());
                ctx
            };

            // 多行块的片段 y 必须包含行盒 y 偏移（line.y），否则多行文本垂直堆叠。
            // `all_fragments()` 返回行内相对 y（恒 0），`all_fragments_with_line_y()`
            // 把 line.y 加到片段 y。
            //
            // **统一使用 with_line_y（R246 限制解除，2026-06-25）**：R246 曾把此修复限定在
            // preserve_whitespace（pre 族），因 auto-wrap 多行块的 test/ref 此前都堆叠同错，
            // 修后反致同源 reftest 净 -11 回归。但实测确认 auto-wrap 多行堆叠是真实 bug
            //（layout 算对多行 h，paint 把多行画在同一 y）——用户可见的"文字堆叠看不清"。
            // 同源 -11 是「test/ref 同错用例的诚实化暴露」（DC-14 视角为进步），非真退步；
            // product-smoke（真实网站）维度此修复为正收益。故统一对所有 Path B 应用 with_line_y。
            let fragments: Vec<zero_layout_engine::TextFragment> = if use_stored {
                Vec::new()
            } else {
                inline_ctx.all_fragments_with_line_y()
            };

            let has_content = use_stored && !stored_fragments.is_empty() || !fragments.is_empty();

            let needs_ellipsis = matches!(style.text_overflow, TextOverflowValue::Ellipsis)
                && !matches!(style.overflow_x, zero_css_parser::values::OverflowValue::Visible);

            // R2237：ellipsis '.' 测宽须用容器字体判定 is_ahem（Ahem 字体 '.' = 1em 方块，
            // 非真实点宽）。旧 measure_char_for_paint('.', fs, false) 硬编码 false → Ahem 容器
            // ellipsis 宽度过小 → 定位错（text-overflow-ellipsis-001）。driving: 同 container_is_ahem
            // 模式（text.rs:853）。
            let container_is_ahem = style
                .font_family
                .iter()
                .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));

            if has_content {
                let glyphs_before_fragments = self.primitives.glyphs.len();

                // writing-mode: vertical-rl/vertical-lr 时字符旋转 90°
                let rotation = if is_vertical { std::f32::consts::FRAC_PI_2 } else { 0.0 };

                if let Some(ref mc) = multicol_info {
                    // 多列布局：遍历行（带 line.y），将行分配到各列
                    let total_height: f32 = inline_ctx.lines.iter().map(|l| l.height).sum();
                    let num_lines = inline_ctx.lines.len();
                    // R1424：target_h 按 ceil(行数/列数) × 平均行高（front-loaded，匹配 chromium）。
                    let target_h = multicol_balance_target_height(num_lines, mc.col_count, total_height);

                    // 预计算每列首行 y，用于把每列内容 rebase 到列内 y=0。
                    // 旧实现 col_start_y = col_idx * target_h，当 target_h 不是行高整数倍时
                    // （如 29 行 / 2 列 → target_h=14.5 行）首行不在 y=0，列内内容整体偏移。
                    // 取每列实际首行 y 作 col_start_y 可消除该 fractional offset。
                    let col_first_y: Vec<f32> = (0..mc.col_count)
                        .map(|col_idx| {
                            if target_h <= 0.0 {
                                0.0
                            } else {
                                inline_ctx
                                    .lines
                                    .iter()
                                    .find(|l| ((l.y / target_h).floor() as usize).min(mc.col_count - 1) == col_idx)
                                    .map(|l| l.y)
                                    .unwrap_or(0.0)
                            }
                        })
                        .collect();

                    // 按列分组渲染：先收集每列的行索引，再按列渲染并裁剪
                    // 这样可以对每列独立裁剪，防止内容溢出到相邻列
                    for (col_idx, &col_start_y) in col_first_y.iter().enumerate() {
                        let col_x_offset = col_idx as f32 * (mc.col_width + mc.gap);

                        // 裁剪区域：列宽 + 右半间隙，允许内容延伸到间隙
                        let clip_rect = Rect::new(
                            content_x + col_x_offset,
                            content_y,
                            mc.col_width + mc.gap / 2.0,
                            box_node.content_height.max(0.0) + 1000.0,
                        );
                        let counts_before_col = PrimitiveCounts::snapshot(&self.primitives);

                        for line in &inline_ctx.lines {
                            // 根据行的 y 位置确定所在列
                            let line_col = if target_h > 0.0 {
                                (line.y / target_h).floor() as usize
                            } else {
                                0
                            }
                            .min(mc.col_count - 1);

                            if line_col != col_idx {
                                continue;
                            }

                            for fragment in &line.runs {
                                self.painted_inline_nodes.insert(fragment.node_id);

                                // 颜色：取片段所属 inline 元素的 color，绕过 inline ownership
                                // （多列分支统一绘制全部片段）。fragment.node_id 可能是 inline 元素
                                // 也可能是文本节点——文本节点时取其父元素。同时标记 owner 元素，
                                // 使 span 自身的 paint_text 跳过（避免在非列位置重绘）。
                                let owner_id = if doc
                                    .get(fragment.node_id)
                                    .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                                {
                                    doc.parent_node(fragment.node_id).unwrap_or(fragment.node_id)
                                } else {
                                    fragment.node_id
                                };
                                self.painted_inline_nodes.insert(owner_id);
                                let owner_style = styles.and_then(|s| s.get(&owner_id));
                                let owner_variations = crate::text_metrics::paint_font_variations(
                                    &owner_style.unwrap_or(style).font_variation_settings,
                                );
                                let owner_font_variation_id = self.primitives.intern_font_variations(&owner_variations);
                                let frag_color = owner_style
                                    .filter(|s| s.color != ColorValue::CurrentColor)
                                    .map(|s| color_value_to_render(&s.color))
                                    .unwrap_or(color);
                                // R2523：text-emphasis-color（CSS Text Decoration 3 §3.3）。
                                // 显式色覆盖 currentColor；默认 CurrentColor → 沿用 frag_color
                                //（标记随文字色，字节不变）。
                                let emphasis_color = owner_style
                                    .filter(|s| s.text_emphasis_color != ColorValue::CurrentColor)
                                    .map(|s| color_value_to_render(&s.text_emphasis_color))
                                    .unwrap_or(frag_color);
                                // R1021：text-emphasis 标记取自片段 owner 样式（<span> 上设的属性），
                                // 非容器 style。None/Char 判定 + 位置均来自 owner。
                                let emphasis_mark: Option<char> =
                                    owner_style.and_then(|s| match s.text_emphasis_style {
                                        TextEmphasisStyleValue::Char(c) => Some(c),
                                        TextEmphasisStyleValue::None => None,
                                    });
                                let emphasis_over = owner_style
                                    .map(|s| {
                                        matches!(
                                            s.text_emphasis_position,
                                            TextEmphasisPositionValue::OverRight | TextEmphasisPositionValue::OverLeft
                                        )
                                    })
                                    .unwrap_or(true);
                                // R1689：ruby per-segment annotation —— owner 为 <ruby> 时，
                                // 每个 rt 配对其前 base 段，annotation 居中于对应 base segment。
                                let ruby_segs: Option<Vec<(String, String)>> = ruby_annotation_segments(doc, owner_id);

                                let frag_base_x = content_x + fragment.x + col_x_offset + tx;
                                // 行盒顶部 = (line.y - col_start_y)；基线偏移 v_offset
                                // （Ahem 完美方块顶部对齐 → 0；普通字体 = font_size ≈ ascent）。
                                // is_ahem 用容器 font-family 判定（多列 IFC 的 fragment.is_ahem 不可靠）。
                                let container_is_ahem = style
                                    .font_family
                                    .iter()
                                    .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
                                let v_offset = if container_is_ahem { 0.0 } else { fragment.font_size };
                                let frag_base_y = content_y + (line.y - col_start_y) + v_offset + ty;

                                let transformed = apply_text_transform(&fragment.text, &style.text_transform);
                                let mut char_pos = frag_base_x;
                                let frag_is_ahem = fragment.is_ahem;

                                for ch in transformed.chars() {
                                    let glyph_x = char_pos;
                                    let glyph_y = frag_base_y;

                                    for &(shadow_ox, shadow_oy, shadow_color) in &active_text_shadows {
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: glyph_x + shadow_ox,
                                            y: glyph_y + shadow_oy,
                                            font_size: fragment.font_size,
                                            color: shadow_color,
                                            glyph_id: ch as u32,
                                            font_glyph_index: None,
                                            source: None,
                                            font_id: default_font_id,
                                            font_variation_id: owner_font_variation_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                            synthetic_italic: false,
                                        });
                                    }

                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: glyph_x,
                                        y: glyph_y,
                                        font_size: fragment.font_size,
                                        color: frag_color,
                                        glyph_id: ch as u32,
                                        font_glyph_index: None,
                                        source: None,
                                        font_id: default_font_id,
                                        font_variation_id: owner_font_variation_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                                        synthetic_italic: false,
                                    });

                                    let advance = self.measure_char_cached(
                                        default_font_id.0,
                                        ch,
                                        fragment.font_size,
                                        frag_is_ahem,
                                    ) + letter_spacing
                                        + if ch == ' ' { word_spacing } else { 0.0 };
                                    char_pos += advance;

                                    // R1021：text-emphasis 标记（CSS Text Decoration 3 §3）。
                                    // 每个非空白字符上方（over）或下方（under）居中绘一个小标记字符。
                                    if !ch.is_whitespace()
                                        && let Some(mark_ch) = emphasis_mark
                                    {
                                        let mark_fs = fragment.font_size * 0.5;
                                        let mark_advance =
                                            self.measure_char_cached(default_font_id.0, mark_ch, mark_fs, frag_is_ahem);
                                        // 居中于当前字符（char_pos 已前进 advance，故字符中心 = char_pos - advance/2）
                                        let mark_x = char_pos - advance / 2.0 - mark_advance / 2.0;
                                        let mark_y = if emphasis_over {
                                            frag_base_y - fragment.font_size
                                        } else {
                                            frag_base_y + fragment.font_size * 0.35
                                        };
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: mark_x,
                                            y: mark_y,
                                            font_size: mark_fs,
                                            color: emphasis_color,
                                            glyph_id: mark_ch as u32,
                                            font_glyph_index: None,
                                            source: None,
                                            font_id: default_font_id,
                                            font_variation_id: owner_font_variation_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                            synthetic_italic: false,
                                        });
                                    }

                                    // R1022 per-char ruby annotation 已移至 text_width 后的
                                    // segment-centered 块（R1688）—— 整 rt 注音居中于 base segment。
                                }

                                let text_width: f32 = transformed
                                    .chars()
                                    .map(|ch| {
                                        let w = self.measure_char_cached(
                                            default_font_id.0,
                                            ch,
                                            fragment.font_size,
                                            frag_is_ahem,
                                        ) + letter_spacing;
                                        if ch == ' ' { w + word_spacing } else { w }
                                    })
                                    .sum();
                                // R1689：ruby per-segment annotation —— 每个 rt 居中于其前 base 段
                                //（替代 R1688 整 base 扁平化居中，解 per-kanji Japanese ruby）。
                                // seg_x_start 按各 base 段字符宽累积，annotation 居中于 [start, start+seg_w]。
                                if let Some(segs) = ruby_segs.as_ref()
                                    && !segs.is_empty()
                                {
                                    let rt_fs = fragment.font_size * 0.5;
                                    let rt_y = frag_base_y - fragment.font_size;
                                    let mut seg_x = frag_base_x;
                                    for (base, annot) in segs {
                                        let seg_w: f32 = base
                                            .chars()
                                            .map(|c| {
                                                self.measure_char_cached(
                                                    default_font_id.0,
                                                    c,
                                                    fragment.font_size,
                                                    frag_is_ahem,
                                                )
                                            })
                                            .sum::<f32>()
                                            + letter_spacing * base.chars().count() as f32;
                                        if !annot.is_empty() {
                                            let annot_w: f32 = annot
                                                .chars()
                                                .map(|c| {
                                                    self.measure_char_cached(default_font_id.0, c, rt_fs, frag_is_ahem)
                                                })
                                                .sum();
                                            let mut ax = seg_x + (seg_w - annot_w) / 2.0;
                                            for rc in annot.chars() {
                                                self.primitives.add_glyph(GlyphPrimitive {
                                                    x: ax,
                                                    y: rt_y,
                                                    font_size: rt_fs,
                                                    color: frag_color,
                                                    glyph_id: rc as u32,
                                                    font_glyph_index: None,
                                                    source: None,
                                                    font_id: default_font_id,
                                                    font_variation_id: owner_font_variation_id,
                                                    bitmap_width: None,
                                                    bitmap_height: None,
                                                    rotation,
                                                    synthetic_italic: false,
                                                });
                                                ax += self.measure_char_cached(
                                                    default_font_id.0,
                                                    rc,
                                                    rt_fs,
                                                    frag_is_ahem,
                                                );
                                            }
                                        }
                                        seg_x += seg_w;
                                    }
                                }
                                self.paint_text_decoration_from_style(
                                    frag_base_x,
                                    frag_base_y,
                                    fragment.font_size,
                                    text_width,
                                    frag_color,
                                    style,
                                );
                            }
                        }

                        // 对本列的图元应用裁剪
                        super::super::helpers::clip_all_primitives_to_rect(
                            &mut self.primitives,
                            &counts_before_col,
                            &clip_rect,
                        );
                    }
                } else {
                    // 非多列布局：统一处理存储片段和 IFC 片段
                    // 宏化渲染逻辑，避免重复代码
                    macro_rules! render_fragment {
                        ($frag_x:expr, $frag_y:expr, $frag_width:expr, $baseline_offset:expr, $frag_fs:expr, $frag_text:expr, $frag_nid:expr, $is_ahem:expr, $frag_source:expr) => {{
                            // CSS 2.1 §9.2.1.1: an inline-block is an atomic inline-level box.
                            // Its parent IFC emits an empty placeholder solely for positioning; the
                            // inline-block's own box must still paint its text.
                            if !$frag_text.is_empty() {
                                self.painted_inline_nodes.insert($frag_nid);
                            }

                            // R358：per-fragment color（带 abs-pos guard）。
                            // 非多列路径此前所有片段用容器 color（丢失 span 自身 color，
                            // 如 multicol-count-computed-004 彩色 span 被渲成容器黑色）。
                            // 现解析每个片段所属元素的 color，**但 abs-pos/fixed 片段保留容器 color**——
                            // R335 实证 per-fragment color 作用于 abspos 文本会使绿色 X 更显眼地绘在
                            // 错误的 paint-IFC（正常流）位置 → abs-pos-non-replaced-vrl/vlr 4 case 回归。
                            // abspos 文本位置修复需 Phase A（R336 double-path），guard 维持当前行为。
                            let owner_id = if doc
                                .get($frag_nid)
                                .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                            {
                                doc.parent_node($frag_nid).unwrap_or($frag_nid)
                            } else {
                                $frag_nid
                            };
                            let frag_color = styles
                                .and_then(|s| s.get(&owner_id))
                                .filter(|s| {
                                    s.color != ColorValue::CurrentColor
                                        && !matches!(
                                            s.position,
                                            zero_css_parser::values::PositionValue::Absolute
                                                | zero_css_parser::values::PositionValue::Fixed
                                        )
                                })
                                .map(|s| color_value_to_render(&s.color))
                                .unwrap_or(color);

                            // R1021：text-emphasis 取自片段 owner 样式（<span> 上设）。
                            let owner_style_opt = styles.and_then(|s| s.get(&owner_id));
                            let shaping_style = owner_style_opt.unwrap_or(style);
                            let variations =
                                crate::text_metrics::paint_font_variations(&shaping_style.font_variation_settings);
                            let font_variation_id = self.primitives.intern_font_variations(&variations);
                            // R2523：text-emphasis-color（CSS Text Decoration 3 §3.3）。
                            // 显式色覆盖 currentColor；默认 CurrentColor → 沿用 frag_color
                            //（标记随文字色，字节不变）。
                            let emphasis_color = owner_style_opt
                                .filter(|s| s.text_emphasis_color != ColorValue::CurrentColor)
                                .map(|s| color_value_to_render(&s.text_emphasis_color))
                                .unwrap_or(frag_color);
                            // R1224/R1464：Path A 按 owner style 选 face；Path B 用 layout
                            // 保存的 per-run face，缺失时回退容器 face。
                            let is_ahem_frag = owner_style_opt.is_some_and(|s| {
                                s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
                            });
                            let frag_font_id = if is_ahem_frag {
                                ahem_font_id
                            } else {
                                text_node_font_ids
                                    .get(&$frag_nid)
                                    .copied()
                                    .unwrap_or(default_font_id)
                            };
                            // R2497：owner 请求 italic 且 resolved face 非 italic 时合成；
                            // Ahem 恒不合成，避免测试字体倾斜。
                            let frag_synthetic_italic = if is_ahem_frag {
                                false
                            } else {
                                let resolved_italic = text_node_font_italic
                                    .get(&$frag_nid)
                                    .copied()
                                    .unwrap_or(default_resolved_italic);
                                let want_it = owner_style_opt
                                    .is_some_and(|s| {
                                        matches!(s.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_))
                                    })
                                    || container_want_italic;
                                let synth_allowed = owner_style_opt
                                    .map(|s| s.font_synthesis.style)
                                    .unwrap_or(true);
                                want_it && !resolved_italic && synth_allowed
                            };
                            let emphasis_mark: Option<char> =
                                owner_style_opt.and_then(|s| match s.text_emphasis_style {
                                    TextEmphasisStyleValue::Char(c) => Some(c),
                                    TextEmphasisStyleValue::None => None,
                                });
                            let emphasis_over = owner_style_opt
                                .map(|s| matches!(
                                    s.text_emphasis_position,
                                    TextEmphasisPositionValue::OverRight | TextEmphasisPositionValue::OverLeft
                                ))
                                .unwrap_or(true);
                            // R1689：ruby per-segment annotation（替代 R1022 逐字符 + R1688 整 base 居中）。
                            let ruby_segs: Option<Vec<(String, String)>> = ruby_annotation_segments(doc, owner_id);

                            let (frag_base_x, frag_base_y, char_advance_is_y) = if is_vertical {
                                (content_x + $frag_x + tx, content_y + $frag_y + ty, true)
                            } else {
                                (
                                    content_x + $frag_x + tx,
                                    content_y + $frag_y + $baseline_offset + ty,
                                    false,
                                )
                            };
                            let mut char_pos = if char_advance_is_y {
                                frag_base_y
                            } else {
                                frag_base_x
                            };

                            let transformed = apply_text_transform(&$frag_text, &style.text_transform);

                            // R639：text_width 先算（glyph loop 之前），支持 inline bg 在 glyph 下绘制。
                            let text_width: f32 = transformed
                                .chars()
                                .map(|ch| {
                                    let w = self.measure_char_cached(frag_font_id.0, ch, $frag_fs, $is_ahem) + letter_spacing;
                                    if ch == ' ' { w + word_spacing } else { w }
                                })
                                .sum();
                            // R1689：ruby per-segment annotation —— 每个 rt 居中于其前 base 段
                            //（替代 R1688 整 base 扁平化居中，解 per-kanji Japanese ruby）。水平 only。
                            if !char_advance_is_y
                                && let Some(segs) = ruby_segs.as_ref()
                                && !segs.is_empty()
                            {
                                let rt_fs = $frag_fs * 0.5;
                                let rt_y = frag_base_y - $frag_fs;
                                let mut seg_x = frag_base_x;
                                for (base, annot) in segs {
                                    let seg_w: f32 = base
                                        .chars()
                                        .map(|c| self.measure_char_cached(frag_font_id.0, c, $frag_fs, $is_ahem))
                                        .sum::<f32>()
                                        + letter_spacing * base.chars().count() as f32;
                                    if !annot.is_empty() {
                                        let annot_w: f32 = annot
                                            .chars()
                                            .map(|c| self.measure_char_cached(frag_font_id.0, c, rt_fs, $is_ahem))
                                            .sum();
                                        let mut ax = seg_x + (seg_w - annot_w) / 2.0;
                                        for rc in annot.chars() {
                                            self.primitives.add_glyph(GlyphPrimitive {
                                                x: ax,
                                                y: rt_y,
                                                font_size: rt_fs,
                                                color: frag_color,
                                                glyph_id: rc as u32,
                                                font_glyph_index: None,
                                                source: None,
                                                font_id: frag_font_id,
                                                font_variation_id,
                                                bitmap_width: None,
                                                bitmap_height: None,
                                                rotation,
                    synthetic_italic: frag_synthetic_italic,
                                            });
                                            ax += self.measure_char_cached(frag_font_id.0, rc, rt_fs, $is_ahem);
                                        }
                                    }
                                    seg_x += seg_w;
                                }
                            }

                            // R639 Phase A slice：per-line-fragment inline background，仅对跨多行
                            // 的 inline 生效。关键修复（R638 锁定 blocker）：宏的 box_node 是 **IFC
                            // owner**（文本所在容器）非 inline 本身，故多行门控用 **owner inline 自身
                            // height**（self.inline_heights 按 owner_id 查），而非 box_node.height
                            //（IFC owner 的）——后者在 inline 文本处于父 IFC 时与 paint_node 抑制
                            //（inline 自身 box 上）分歧致 bg 消失。两处现均用 inline 自身 height → 一致。
                            // frag_base_x 已含 text-indent（IFC 首行 current_x=text_indent），首行从缩进后起。
                            let owner_h = self.inline_heights.get(&owner_id).copied().unwrap_or(0.0);
                            // R2160 Phase A slice 2 part2：env `ZW_PHASEA_MULTI_INLINE`（**R2198 default-on**；
                            // `=0` kill-switch。R2163 曾 REVERT default-on：orphan 丢 LayoutBox 致 hit_test.rs
                            // 漏收 `<a>`；R2197 slice 3 回填 + R2198 struct-check paint_skip-aware 后复开）
                            // 时，R639 per-fragment bg/border 也对 **orphan inline**（owner_h==0.0，即
                            // part1 skip-taffy 致 inline_heights 无条目）触发——补 part1 丢的 LayoutBox
                            // bg/border 绘制（解 R1492）。单行非 orphan（owner_h∈(0,1.5·fs]）不触发=
                            // 无双绘。orphan 信号（owner_h==0）天然耦合 part1。
                            let phasea_orphan_fire = std::env::var("ZW_PHASEA_MULTI_INLINE").as_deref() != Ok("0")
                                && owner_h == 0.0;
                            if !is_vertical
                                && !box_node.is_absolute
                                && !box_node.is_fixed
                                && (owner_h > $frag_fs * 1.5 || phasea_orphan_fire)
                                && let Some(owner_style) = styles.and_then(|s| s.get(&owner_id))
                                && matches!(owner_style.display, zero_css_parser::values::DisplayValue::Inline)
                            {
                                // R1442：inline padding/border 不入 line box 高度（CSS §10.8.1）但渲染于
                                // inline box 之外，可上溢/下溢覆盖邻接 line box（border-padding-bleed
                                // driving test）。R1441 定位旧 bg 用 `frag.y`（字形 run 顶 = line.y + f.y，
                                // f.y = baseline_y − line_h）非 line box top，偏移 ~ascent。has_bleed 时改用
                                // `frag.y + baseline_offset`（= glyph 顶 = line box top，Ahem lh:1 em-box）
                                // + 外延 padding+border；并 per-fragment 绘 border-top/bottom（003 border-only）。
                                // gate 不变（仍多行 owner_h>1.5·fs）→ 不触 single-line，避 R638 双计；
                                // bg-only 多行无 padding/border（R639）保持旧 frag.y 位不变。
                                let px_of = |lv: &LengthValue| match lv {
                                    LengthValue::Px(v) => *v as f32,
                                    _ => 0.0,
                                };
                                let bt_w = if matches!(
                                    owner_style.border_top_style,
                                    zero_style_system::property::types::BorderStyleValue::None
                                ) {
                                    0.0
                                } else {
                                    px_of(&owner_style.border_top_width)
                                };
                                let bb_w = if matches!(
                                    owner_style.border_bottom_style,
                                    zero_style_system::property::types::BorderStyleValue::None
                                ) {
                                    0.0
                                } else {
                                    px_of(&owner_style.border_bottom_width)
                                };
                                let pad_top = px_of(&owner_style.padding_top);
                                let pad_bot = px_of(&owner_style.padding_bottom);
                                let has_bg = owner_style.background_color != ColorValue::Transparent;
                                let has_bleed = pad_top > 0.0 || pad_bot > 0.0 || bt_w > 0.0 || bb_w > 0.0;
                                if has_bg || has_bleed {
                                    let line_h = box_node
                                        .text_node_line_heights
                                        .get(&$frag_nid)
                                        .copied()
                                        .unwrap_or($frag_fs * 1.164);
                                    let line_top = content_y + $frag_y + $baseline_offset + ty;
                                    let bleed_top = pad_top + bt_w;
                                    let bleed_bot = pad_bot + bb_w;
                                    // bg（has_bg 时）：has_bleed 外延对齐 line box 边，否则 R639 旧位（frag.y）。
                                    if has_bg {
                                        let (bg_y, bg_h) = if has_bleed {
                                            (line_top - bleed_top, line_h + bleed_top + bleed_bot)
                                        } else {
                                            (content_y + $frag_y + ty, line_h)
                                        };
                                        self.primitives.add_fill(
                                            Rect::new(frag_base_x, bg_y, text_width, bg_h),
                                            color_value_to_render(&owner_style.background_color),
                                        );
                                    }
                                    // per-fragment border-top/bottom（外延到 line box 之外覆盖邻接行）。
                                    if bt_w > 0.0 {
                                        let c = if matches!(owner_style.border_top_color, ColorValue::CurrentColor) {
                                            frag_color
                                        } else {
                                            color_value_to_render(&owner_style.border_top_color)
                                        };
                                        self.primitives
                                            .add_fill(Rect::new(frag_base_x, line_top - bt_w, text_width, bt_w), c);
                                    }
                                    if bb_w > 0.0 {
                                        let c = if matches!(owner_style.border_bottom_color, ColorValue::CurrentColor) {
                                            frag_color
                                        } else {
                                            color_value_to_render(&owner_style.border_bottom_color)
                                        };
                                        self.primitives.add_fill(
                                            Rect::new(frag_base_x, line_top + line_h + pad_bot, text_width, bb_w),
                                            c,
                                        );
                                    }
                                }
                            }

                            let size_adjust = fragment_font_size_adjustment(
                                owner_style_opt,
                                box_node.text_node_font_size_adjust.get(&$frag_nid),
                                &style.font_size_adjust,
                                !self.generic_font_ids.contains(&frag_font_id.0),
                            );
                            let shaped_text_eligible = !char_advance_is_y
                                && !$is_ahem
                                && letter_spacing == 0.0
                                && word_spacing == 0.0
                                && active_text_shadows.is_empty()
                                && emphasis_mark.is_none()
                                && ruby_segs.as_ref().is_none_or(Vec::is_empty)
                                && !frag_synthetic_italic
                                && !style.text_decoration_line.has_any()
                                && owner_style_opt.is_none_or(|owner| {
                                    !owner.text_decoration_line.has_any()
                                        && (owner.background_color == ColorValue::Transparent
                                            || text_shaping::font_size_adjustment_active(size_adjust))
                                })
                                && rotation.abs() < f32::EPSILON
                                && transformed.chars().all(|ch| !is_cc_control_char(ch));
                            let style_direction = match style.direction {
                                zero_style_system::DirectionValue::Ltr => TextDirection::LeftToRight,
                                zero_style_system::DirectionValue::Rtl => TextDirection::RightToLeft,
                            };
                            let text_direction = text_shaping::fragment_shape_direction(
                                $frag_source,
                                style_direction,
                                text_shaping::shaped_uba_rtl_enabled(),
                            );
                            let logical_source = logical_fragment_source(
                                $frag_source,
                                text_direction,
                                matches!(style.text_transform, TextTransformValue::None),
                            );
                            // https://drafts.csswg.org/css-fonts/#generic-font-families
                            let generic_font = self.generic_font_ids.contains(&frag_font_id.0);
                            let shaped_advance_eligible =
                                text_shaping::fragment_shaped_advance_eligible(generic_font, size_adjust);
                            let shaping_font_ids = self.fragment_shaping_font_ids(
                                owner_style_opt,
                                text_node_shaping_font_ids.get(&$frag_nid).map(Vec::as_slice),
                                frag_font_id,
                            );
                            let open_type_features = style_open_type_features(shaping_style);
                            let advance_trace = fragment_advance_trace(
                                &shaping_font_ids,
                                &transformed,
                                $frag_fs,
                                text_direction,
                                logical_source.as_ref(),
                                &open_type_features,
                                &variations,
                                size_adjust,
                            );
                            for glyph in fragment_glyphs(
                                &shaping_font_ids,
                                &transformed,
                                $frag_fs,
                                shaped_text_eligible,
                                text_direction,
                                shaped_advance_eligible,
                                logical_source,
                                &open_type_features,
                                &variations,
                                size_adjust,
                            ) {
                                let ch = glyph.code_point;
                                let glyph_font_id = glyph.font_id.unwrap_or(frag_font_id);
                                let glyph_font_size = glyph.font_size.unwrap_or($frag_fs);
                                let (glyph_x, glyph_y) = if char_advance_is_y {
                                    (frag_base_x, char_pos)
                                } else {
                                    (char_pos + glyph.x_offset, frag_base_y - glyph.y_offset)
                                };

                                for &(shadow_ox, shadow_oy, shadow_color) in &active_text_shadows {
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: glyph_x + shadow_ox,
                                        y: glyph_y + shadow_oy,
                                        font_size: glyph_font_size,
                                        color: shadow_color,
                                        glyph_id: ch as u32,
                                        font_glyph_index: None,
                                        source: None,
                                        font_id: glyph_font_id,
                                        font_variation_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                    synthetic_italic: frag_synthetic_italic,
                                    });
                                }

                                self.primitives.add_glyph(GlyphPrimitive {
                                    x: glyph_x,
                                    y: glyph_y,
                                    font_size: glyph_font_size,
                                    color: frag_color,
                                    glyph_id: ch as u32,
                                    font_glyph_index: glyph.font_glyph_index,
                                    source: glyph.source.clone(),
                                    font_id: glyph_font_id,
                                    font_variation_id,
                                    bitmap_width: None,
                                    bitmap_height: None,
                                    rotation,
                    synthetic_italic: frag_synthetic_italic,
                                });

                                // R644：Cc 控制字符可见性（CSS Text 3）——fontdue 对 Cc 无字形
                                //（.notdef 空），渲染可见占位框（em 方块），使 control-chars-* mismatch
                                // 测试 test != 空 ref（diff > min_mismatch_ratio 0.5%；fs×fs em 方块
                                // 在 4em=64px 下 ~0.85% diff，超阈值）。
                                if is_cc_control_char(ch) {
                                    self.primitives.add_fill(
                                        Rect::new(
                                            glyph_x,
                                            glyph_y - glyph_font_size,
                                            glyph_font_size,
                                            glyph_font_size,
                                        ),
                                        frag_color,
                                    );
                                }

                                let advance = glyph
                                    .advance_x
                                    .unwrap_or_else(|| self.measure_char_cached(frag_font_id.0, ch, $frag_fs, $is_ahem))
                                    + letter_spacing
                                    + if ch == ' ' { word_spacing } else { 0.0 };
                                char_pos += advance;

                                // R1021：text-emphasis 标记（水平书写模式；垂直暂不支持）。
                                if !char_advance_is_y
                                    && !ch.is_whitespace()
                                    && let Some(mark_ch) = emphasis_mark
                                {
                                    let mark_fs = $frag_fs * 0.5;
                                    let mark_advance = self.measure_char_cached(frag_font_id.0, mark_ch, mark_fs, $is_ahem);
                                    let mark_x = char_pos - advance / 2.0 - mark_advance / 2.0;
                                    // over：mark 基线在文本顶部之上（leading 区）；under：基线之下
                                    let mark_y = if emphasis_over {
                                        frag_base_y - $frag_fs - mark_fs * 0.4
                                    } else {
                                        frag_base_y + $frag_fs * 0.5
                                    };
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: mark_x,
                                        y: mark_y,
                                        font_size: mark_fs,
                                        color: emphasis_color,
                                        glyph_id: mark_ch as u32,
                                        font_glyph_index: None,
                                        source: None,
                                        font_id: frag_font_id,
                                        font_variation_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                    synthetic_italic: frag_synthetic_italic,
                                    });
                                }

                                // R1022 per-char ruby annotation 已移至 text_width 后的
                                // segment-centered 块（R1688）—— 整 rt 注音居中于 base segment。
                            }

                            if let Some(trace) = advance_trace {
                                trace.emit(
                                    if use_stored { "stored" } else { "paint-ifc" },
                                    frag_font_id.0,
                                    $frag_fs,
                                    &transformed,
                                    FragmentPaintWidths {
                                        fragment: $frag_width,
                                        legacy: text_width,
                                        consumed: char_pos - frag_base_x,
                                    },
                                );
                            }

                            self.paint_text_decoration_from_style(
                                frag_base_x,
                                frag_base_y,
                                $frag_fs,
                                text_width,
                                frag_color,
                                style,
                            );
                        }};
                    }

                    if use_stored {
                        for frag in &stored_fragments {
                            // R817 linebox 度量统一 Phase 2：is_ahem glyph 位图顶 = 片段基线 - font_size
                            // （Ahem 方块底边齐基线，ascent=font_size）。行基线（container-rel）=
                            // frag.baseline_y_abs。macro glyph_y = content_y + frag.y + v_offset，其中
                            // frag.y = line.y + f.y，故 v_offset = baseline_y_abs - font_size - frag.y
                            // → glyph_y = content_y + line.y + line.baseline_y - font_size（基线处）。
                            // 旧 v_offset=0 把 glyph 放在 f.y（=baseline_y-run.height，line-height>1 时
                            // 为负，glyph 越过行盒顶部错位）。line-height:1 时 f.y=baseline_y-font_size，
                            // v_offset 退化为 0（== 旧行为，A3，font-051 不回归）。
                            // 仅对**真正** Ahem 字形（is_ahem_font，来自 IFC run 实际字体）应用——
                            // 容器为 Ahem 但片段实为其它字体（font-051 的 serif span）时保留旧
                            // 容器级行为（is_ahem?0:font_size），避免按 ascent=font_size 错移。
                            let v_offset = if frag.is_ahem_font {
                                // R841：line-height-aware Ahem 方块位（见 ahem_uses_embox_position）。
                                // half-leading≈0（lh≈fs）→ em-box 位 baseline−0.8·fs；否则 R817 baseline−fs。
                                if ahem_uses_embox_position(frag.height, frag.font_size) {
                                    frag.baseline_y_abs - 0.8 * frag.font_size - frag.y
                                } else {
                                    frag.baseline_y_abs - frag.font_size - frag.y
                                }
                            } else if frag.is_ahem {
                                0.0
                            } else {
                                frag.font_size
                            };
                            render_fragment!(
                                frag.x,
                                frag.y,
                                frag.width,
                                v_offset,
                                frag.font_size,
                                frag.text,
                                frag.node_id,
                                frag.is_ahem,
                                frag.source.as_ref()
                            );
                        }
                    } else {
                        for fragment in fragments.iter() {
                            // IFC 片段（空 styles）：frag.y 基于 16px 默认值，
                            // 使用存储的 font_size（来自 layout IFC）计算基线偏移。
                            // 如果无存储值，回退到 16px 默认值（保持原有行为）。
                            let stored_fs = box_node.text_node_font_sizes.get(&fragment.node_id).copied();
                            let baseline_fs = stored_fs.unwrap_or(fragment.font_size);
                            // GlyphPrimitive.y is a baseline coordinate. The IFC fragment y is the
                            // run top (`baseline_y - run.height`), so text adds the full fragment
                            // height. Subtracting an estimated ascent here passes the em-box top as a
                            // baseline and shifts every rasterized glyph upward by about 0.8em.
                            let baseline_offset = if fragment.font_size > 0.0 {
                                paint_ifc_baseline_offset(fragment.height)
                            } else {
                                baseline_fs
                            };
                            render_fragment!(
                                fragment.x,
                                fragment.y,
                                fragment.width,
                                baseline_offset,
                                stored_fs.unwrap_or(fragment.font_size),
                                fragment.text,
                                fragment.node_id,
                                fragment.is_ahem,
                                fragment.source.as_ref()
                            );
                        }
                    }
                } // end non-multicol else block

                // text-overflow: ellipsis 后处理
                if needs_ellipsis && container_width > 0.0 {
                    let content_right = content_x + container_width + tx;

                    let glyphs = &mut self.primitives.glyphs;
                    let fragment_glyphs = &mut glyphs[glyphs_before_fragments..];

                    let mut last_visible_idx: Option<usize> = None;
                    let mut has_overflow = false;

                    for (i, g) in fragment_glyphs.iter().enumerate() {
                        if g.font_size == 0.0 {
                            continue;
                        }
                        if g.x >= content_right {
                            has_overflow = true;
                            last_visible_idx = if i > 0 { Some(i - 1) } else { None };
                            break;
                        }
                        last_visible_idx = Some(i);
                    }

                    if has_overflow {
                        let ellipsis_char_width =
                            crate::measure_char_for_font(default_font_id.0, '.', font_size, container_is_ahem);
                        let total_ellipsis_width = ellipsis_char_width * 3.0 + letter_spacing * 2.0;
                        let ellipsis_end_x = content_right;
                        let ellipsis_start_x = ellipsis_end_x - total_ellipsis_width;

                        let cutoff_start = if let Some(idx) = last_visible_idx {
                            let mut cut = idx + 1;
                            for j in (0..=idx).rev() {
                                if fragment_glyphs[j].x < ellipsis_start_x && fragment_glyphs[j].font_size > 0.0 {
                                    cut = j + 1;
                                    break;
                                }
                                cut = j;
                            }
                            cut
                        } else {
                            0
                        };

                        for g in fragment_glyphs.iter_mut().skip(cutoff_start) {
                            g.glyph_id = 0;
                            g.font_size = 0.0;
                        }

                        let first_glyph = fragment_glyphs.iter().find(|g| g.font_size > 0.0);
                        let base_y = first_glyph.map(|g| g.y).unwrap_or(content_y + font_size + ty);

                        for (i, ch) in ['.', '.', '.'].iter().enumerate() {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: ellipsis_start_x + ellipsis_char_width * i as f32 + letter_spacing * i as f32,
                                y: base_y,
                                font_size,
                                color,
                                glyph_id: *ch as u32,
                                font_glyph_index: None,
                                source: None,
                                font_id: default_font_id,
                                font_variation_id: default_font_variation_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation: 0.0,
                                synthetic_italic: false,
                            });
                        }
                    }
                }

                // CSS line-clamp 后处理：限制可见行数并在截断处添加省略号
                if let Some(max) = max_lines {
                    let glyphs = &self.primitives.glyphs;
                    let fragment_glyphs = &glyphs[glyphs_before_fragments..];

                    // 收集唯一的行 Y 坐标（用于计算总行数）
                    let mut line_ys: Vec<f32> = fragment_glyphs
                        .iter()
                        .filter(|g| g.font_size > 0.0)
                        .map(|g| g.y)
                        .collect();
                    line_ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    line_ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);

                    // R2467 line-clamp slice 2：触发条件双源。
                    // - `exceeded`：paint 看到的行数 > max（non-stored 路径：paint IFC 用空 styles
                    //   重跑不 cap → 全量行）。
                    // - `box_node.line_clamp_clamped`：layout 期 IFC `apply_line_clamp_cap` 真截断
                    //   （stored 路径：pure-Ahem 容器 inline_layout 已被 cap 到 max 行，paint 看到
                    //   行数 == max → exceeded=false → 须读此标志才能补 ellipsis）。
                    // max==0（CSS line-clamp:0 视同 none）或无行 → 不触发。
                    let exceeded = max >= 1 && line_ys.len() > max as usize;
                    let clamped = max >= 1 && box_node.line_clamp_clamped;
                    if exceeded || clamped {
                        // 截断：移除第 max 行之后的所有 glyph。stored 路径已被 layout cap → 无超出行 →
                        // 此处 exceeded=false 跳过；non-stored 路径 exceeded=true 主动截。
                        if exceeded {
                            let cutoff_y = line_ys[max as usize];
                            let glyphs = &mut self.primitives.glyphs;
                            for g in glyphs[glyphs_before_fragments..].iter_mut() {
                                if g.y >= cutoff_y - 0.5 {
                                    g.font_size = 0.0;
                                    g.glyph_id = 0;
                                }
                            }
                        }

                        // 在最后一可见行（第 max 行）末尾渲 U+2026 ellipsis（与 WPT line-clamp refs
                        // 一致：单字符 `…`，非 3 个 ASCII '.'）。`max.min(line_ys.len())` 防越界
                        //（stored 路径行数 == max）。
                        let last_idx = (max as usize).min(line_ys.len()).saturating_sub(1);
                        let last_line_y = line_ys[last_idx];

                        // 末行最右可见 glyph 及其字符 advance（求末行文本 end x）。
                        let last_glyph = self.primitives.glyphs[glyphs_before_fragments..]
                            .iter()
                            .filter(|g| g.font_size > 0.0 && (g.y - last_line_y).abs() < 0.5)
                            .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
                        let last_adv = last_glyph
                            .and_then(|g| char::from_u32(g.glyph_id))
                            .map(|c| self.measure_char_cached(default_font_id.0, c, font_size, container_is_ahem))
                            .unwrap_or(0.0);
                        let last_text_end_x = last_glyph.map(|g| g.x + last_adv).unwrap_or(content_x + tx);

                        let default_font_id = self.resolve_style_font_id(&style.font_family, style).0;
                        let ellipsis_char = '\u{2026}';
                        let ellipsis_width =
                            self.measure_char_cached(default_font_id.0, ellipsis_char, font_size, container_is_ahem);
                        let content_right = content_x + container_width + tx;

                        // 定位：紧跟末行文本末尾；若 + ellipsis 宽超 content_right（末行已占满），
                        // 回退到 content_right 右对齐 + 截掉末行尾部 glyph 让位（镜像 text-overflow
                        // cutoff，text.rs:1464-1481）。
                        let ellipsis_x = if last_text_end_x + ellipsis_width <= content_right + 0.5 {
                            last_text_end_x
                        } else {
                            let cut_x = content_right - ellipsis_width;
                            let glyphs = &mut self.primitives.glyphs;
                            for g in glyphs[glyphs_before_fragments..].iter_mut() {
                                if g.font_size > 0.0 && (g.y - last_line_y).abs() < 0.5 && g.x + 0.5 >= cut_x {
                                    g.font_size = 0.0;
                                    g.glyph_id = 0;
                                }
                            }
                            cut_x
                        };

                        self.primitives.add_glyph(GlyphPrimitive {
                            x: ellipsis_x,
                            y: last_line_y,
                            font_size,
                            color,
                            glyph_id: ellipsis_char as u32,
                            font_glyph_index: None,
                            source: None,
                            font_id: default_font_id,
                            font_variation_id: default_font_variation_id,
                            bitmap_width: None,
                            bitmap_height: None,
                            rotation: 0.0,
                            synthetic_italic: false,
                        });
                    }
                }

                return;
            }
        }

        // 退化为单个占位 glyph
        let glyph_x = content_x + tx;
        let glyph_y = content_y + ty;

        for &(shadow_ox, shadow_oy, shadow_color) in &active_text_shadows {
            self.primitives.add_glyph(GlyphPrimitive {
                x: glyph_x + shadow_ox,
                y: glyph_y + font_size + shadow_oy,
                font_size,
                color: shadow_color,
                glyph_id: 0,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id: default_font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            });
        }

        self.primitives.add_glyph(GlyphPrimitive {
            x: glyph_x,
            y: glyph_y + font_size,
            font_size,
            color,
            glyph_id: 0,
            font_glyph_index: None,
            source: None,
            font_id: default_font_id,
            font_variation_id: default_font_variation_id,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });

        self.paint_text_decoration_from_style(
            glyph_x,
            glyph_y + font_size,
            font_size,
            self.measure_char_cached(default_font_id.0, 'A', font_size, false),
            color,
            style,
        );
    }

    /// 绘制匿名文本项（flex/grid 容器中的文本节点）。
    ///
    /// 与 paint_text 不同，此方法直接渲染 node_id 指向的文本节点内容，
    /// 而非查找子文本节点。匿名文本项没有独立的 ComputedStyle，
    /// 使用父元素的样式。
    pub fn paint_anonymous_text_item(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
        node_id: NodeId,
    ) {
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 || style.color == ColorValue::CurrentColor {
            return;
        }

        let color = color_value_to_render(&style.color);
        let default_font_id = self.resolve_style_font_id(&style.font_family, style).0;
        let variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&variations);
        let content_x = abs_x;
        let content_y = abs_y;

        // 直接从文本节点获取内容
        let text = match doc.get(node_id) {
            Some(node) => match &node.kind {
                NodeKind::Text(data) => data.content.trim().to_string(),
                _ => return,
            },
            None => return,
        };
        if text.is_empty() {
            return;
        }

        // 渲染文本字符为 glyph primitives
        let is_ahem = style
            .font_family
            .iter()
            .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
        let mut char_x = content_x;
        for ch in text.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: content_y + font_size,
                font_size,
                color,
                glyph_id: ch as u32,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            });
            char_x += self.measure_char_cached(default_font_id.0, ch, font_size, is_ahem);
        }
    }
}

/// 将数字转换为罗马数字字符串（1-based）。
pub(super) fn has_direct_paintable_text(
    doc: &Document,
    node_id: NodeId,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
) -> bool {
    let direct = doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if !text.content.trim().is_empty()
        )
    });
    if direct {
        return true;
    }
    // PHASEA stored-line-boxes 路径（默认启用；env PHASEA_STORE_EXT=0 关闭，与 compute_final 存储扩展配套）：仅对
    // **纯 inline 内容**容器（有 inline-level 元素子节点且**无 block-level 元素子节点**）返回
    // true。排除 block 子节点（独立渲染）与混合 inline+block 内容（block-in-inline / span+h4
    // 等存储路径与重跑分歧致回归：inline-box-001/002、multicol-block-no-clip-001）。
    if std::env::var("PHASEA_STORE_EXT").as_deref() != Ok("0")
        && let Some(styles) = styles
    {
        use zero_css_parser::values::DisplayValue;
        let is_inline_display = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::Inline
                    | DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        let child_ids: Vec<zero_dom::NodeId> = doc.child_nodes(node_id);
        let child_displays: Vec<Option<&DisplayValue>> =
            child_ids.iter().map(|c| styles.get(c).map(|s| &s.display)).collect();
        let has_inline_elem = child_displays.iter().any(|d| d.is_some_and(is_inline_display));
        // R1280：float 子元素是 out-of-flow（CSS §9.5），不属于 in-flow block 内容。
        // 含 [inline 文本 + float] 的容器其 inline 内容仍经 IFC 排版并绕 float 流动。
        // 旧实现把 blockified float（display:Block + float≠none）误计为 block-level →
        // 此处返回 false → 容器 paint_text 早退 → inline 元素文本（floats-006 的 <span>X</span>）
        // 经 span 自身 Path B 在非 float-excluded 位渲染。float 子不计为 block，让容器 paint_text
        // 跑（Path A 存储见 inline_finalization，或 Path B collect_float_exclusions），IFC 正确绕
        // float，并经 painted_inline_nodes 自动抑制 inline 子 Path B（避免双绘）。
        // kill-switch `ZW_FLOAT_INLINE_PAINT=0` 回退旧行为（default-on，全 dir A/B net 0 零回归）。
        let float_not_block = std::env::var("ZW_FLOAT_INLINE_PAINT").as_deref() != Ok("0");
        let has_block_elem = if float_not_block {
            child_ids.iter().any(|c| {
                styles.get(c).is_some_and(|s| {
                    !is_inline_display(&s.display) && matches!(s.float, zero_css_parser::values::FloatValue::None)
                })
            })
        } else {
            child_displays
                .iter()
                .any(|d| d.is_some_and(|dd| !is_inline_display(dd)))
        };
        // inline-level 子元素须为叶文本容器（无元素子节点），排除 block-in-inline（R109 碎片化）。
        let inline_children_have_elem = child_ids.iter().any(|c| {
            styles.get(c).is_some_and(|s| is_inline_display(&s.display))
                && doc
                    .child_nodes(*c)
                    .iter()
                    .any(|gc| doc.get(*gc).is_some_and(|n| matches!(&n.kind, NodeKind::Element(_))))
        });
        has_inline_elem && !has_block_elem && !inline_children_have_elem
    } else {
        false
    }
}

fn replace_utf16_range(value: &str, start: usize, end: usize, replacement: &str) -> String {
    fn byte_offset(value: &str, utf16_offset: usize) -> usize {
        let mut units = 0;
        for (byte, ch) in value.char_indices() {
            if units >= utf16_offset {
                return byte;
            }
            units += ch.len_utf16();
        }
        value.len()
    }

    let range_start = start.min(end);
    let range_end = start.max(end);
    let start_byte = byte_offset(value, range_start);
    let end_byte = byte_offset(value, range_end);
    format!("{}{}{}", &value[..start_byte], replacement, &value[end_byte..])
}

/// R2303 object-position 定位测试。
#[cfg(test)]
mod r2303_object_position_tests;
/// R841 em-box 定位测试。
#[cfg(test)]
mod r841_tests;
