//! CSS 属性继承。
//!
//! 为元素计算继承样式：处理 inherit、initial、unset、revert 关键字，
//! 以及隐式继承（无显式值时从父元素继承）。

use std::collections::HashMap;

use zero_css_parser::values::FontWeightValue;
use zero_dom::QuirksMode;

use crate::property::{
    ComputedStyle, PropertyRegistry, apply_initial_value, apply_property_value_with_quirks, inherit_property,
};

/// 为元素计算继承样式。
///
/// # 参数
///
/// - `parent_style` — 父元素的计算样式（根元素为 None）
/// - `cascaded` — 级联后的属性值映射
/// - `quirks_mode` — 文档 quirks mode 状态
///
/// # 返回值
///
/// 返回包含继承和初始值的完整计算样式。
pub fn compute_inherited_style(
    parent_style: Option<&ComputedStyle>,
    cascaded: &HashMap<String, String>,
) -> ComputedStyle {
    compute_inherited_style_with_quirks(parent_style, cascaded, QuirksMode::NoQuirks, false)
}

/// 为元素计算继承样式（支持 quirks mode）。
///
/// `prefers_dark` 为用户颜色偏好（`prefers-color-scheme` 媒体查询 = dark），
/// 参与 `color-scheme` 属性的 used-scheme 合成（见 `parse_color_scheme_dark`）。
pub fn compute_inherited_style_with_quirks(
    parent_style: Option<&ComputedStyle>,
    cascaded: &HashMap<String, String>,
    quirks_mode: QuirksMode,
    prefers_dark: bool,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

    // 预解析 color-scheme：它影响本元素 light-dark() 颜色解析，须在颜色属性应用前确定
    //（CSS 规定 color-scheme 先于其他属性计算）。复用 keyword 解析保证 inherit/initial 等正确。
    // 主循环仍会按既有路径处理 color-scheme（idempotent）；未显式声明则继承父元素。
    if let Some(cs) = cascaded.get("color-scheme") {
        style.color_scheme_dark = match resolve_keyword(cs, "color-scheme", parent_style) {
            KeywordResolution::Concrete(v) => crate::property::apply::parse_color_scheme_dark(v, prefers_dark),
            KeywordResolution::Inherit | KeywordResolution::Unset | KeywordResolution::Revert => {
                parent_style.map(|p| p.color_scheme_dark).unwrap_or(false)
            }
            KeywordResolution::Initial | KeywordResolution::RevertLayer => false,
        };
    } else if let Some(parent) = parent_style {
        style.color_scheme_dark = parent.color_scheme_dark;
    }

    // 先处理所有级联属性
    for (property, value) in cascaded {
        let resolved = resolve_keyword(value, property, parent_style);
        match resolved {
            KeywordResolution::Inherit => {
                if let Some(parent) = parent_style {
                    inherit_property(parent, &mut style, property);
                } else {
                    // 根元素没有父，inherit 等于 initial
                    apply_initial_value(&mut style, property);
                }
            }
            KeywordResolution::Initial => {
                apply_initial_value(&mut style, property);
            }
            KeywordResolution::Unset => {
                if PropertyRegistry::is_inherited(property) {
                    if let Some(parent) = parent_style {
                        inherit_property(parent, &mut style, property);
                    } else {
                        apply_initial_value(&mut style, property);
                    }
                } else {
                    apply_initial_value(&mut style, property);
                }
            }
            KeywordResolution::Revert => {
                // revert 跳过作者级联值，回退到 User/UA 来源。
                // 简化实现：继承属性使用父元素计算值（模拟 User 来源行为），
                // 非继承属性使用初始值。
                if PropertyRegistry::is_inherited(property) {
                    if let Some(parent) = parent_style {
                        inherit_property(parent, &mut style, property);
                    } else {
                        apply_initial_value(&mut style, property);
                    }
                } else {
                    apply_initial_value(&mut style, property);
                }
            }
            KeywordResolution::RevertLayer => {
                // revert-layer 跳过当前 @layer 声明，回退到更低优先级层。
                // 简化实现：等同于 unset。
                if PropertyRegistry::is_inherited(property) {
                    if let Some(parent) = parent_style {
                        inherit_property(parent, &mut style, property);
                    } else {
                        apply_initial_value(&mut style, property);
                    }
                } else {
                    apply_initial_value(&mut style, property);
                }
            }
            KeywordResolution::Concrete(v) => {
                apply_property_value_with_quirks(
                    &mut style,
                    property,
                    v,
                    quirks_mode == QuirksMode::Quirks,
                    prefers_dark,
                );
            }
        }
    }

    // 对没有级联值的继承属性，从父元素继承
    if let Some(parent) = parent_style {
        for property in PropertyRegistry::known_properties() {
            if !cascaded.contains_key(*property) && PropertyRegistry::is_inherited(property) {
                inherit_property(parent, &mut style, property);
            }
        }
    }

    // https://drafts.csswg.org/css-fonts-4/#font-weight-prop
    // `bolder`/`lighter` 的 computed value 依赖父元素，须在继承完成后解析为绝对字重，
    // 使 layout、paint 与 getComputedStyle 消费同一结果。
    style.font_weight = resolve_relative_font_weight(&style.font_weight, parent_style);

    // CSS Text 3 §6.1：`match-parent` 在 compute 阶段按父元素定型——继承父元素 text-align，
    // 但父值为 start/end 时按**父 direction** 解析为 left/right（区别于普通 inherit：inherit
    // 保留 start/end 由子元素自身 direction 在 layout 解析）。根元素无父 → initial (start)。
    // 此处 parent 的 text-align 已是其 computed 值（不会再是 MatchParent）。
    if style.text_align == crate::property::TextAlignValue::MatchParent {
        style.text_align = match parent_style {
            Some(parent) => resolve_match_parent(&parent.text_align, &parent.direction),
            None => crate::property::TextAlignValue::Start,
        };
    }

    style
}

fn resolve_relative_font_weight(weight: &FontWeightValue, parent_style: Option<&ComputedStyle>) -> FontWeightValue {
    let parent_weight = parent_style
        .map(|parent| font_weight_absolute(&parent.font_weight))
        .unwrap_or(400);
    match weight {
        FontWeightValue::Bolder => FontWeightValue::Absolute(if parent_weight < 400 {
            400
        } else if parent_weight < 600 {
            700
        } else {
            900
        }),
        FontWeightValue::Lighter => FontWeightValue::Absolute(if parent_weight < 600 {
            100
        } else if parent_weight < 800 {
            400
        } else {
            700
        }),
        _ => weight.clone(),
    }
}

fn font_weight_absolute(weight: &FontWeightValue) -> u16 {
    match weight {
        FontWeightValue::Absolute(value) => *value,
        FontWeightValue::Normal => 400,
        FontWeightValue::Bold => 700,
        FontWeightValue::Bolder | FontWeightValue::Lighter => 400,
    }
}

/// 把 `match-parent` 解析为具体 text-align 值：继承 origin，但 start/end 按父 direction
/// 解析（CSS Text 3 §6.1）。origin 须为父元素 computed text-align（非 MatchParent）。
fn resolve_match_parent(
    origin: &crate::property::TextAlignValue,
    direction: &crate::property::DirectionValue,
) -> crate::property::TextAlignValue {
    use crate::property::{DirectionValue, TextAlignValue};
    let is_rtl = matches!(direction, DirectionValue::Rtl);
    match origin {
        TextAlignValue::Start => {
            if is_rtl {
                TextAlignValue::Right
            } else {
                TextAlignValue::Left
            }
        }
        TextAlignValue::End => {
            if is_rtl {
                TextAlignValue::Left
            } else {
                TextAlignValue::Right
            }
        }
        other => other.clone(),
    }
}

/// CSS 全局关键字解析结果。
#[derive(Debug, Clone, Copy, PartialEq)]
enum KeywordResolution<'a> {
    /// inherit 关键字。
    Inherit,
    /// initial 关键字。
    Initial,
    /// unset 关键字。
    Unset,
    /// revert 关键字 — 回退到上一层来源级联值。
    ///
    /// 简化实现：跳过作者级联，继承属性使用父元素值，
    /// 非继承属性使用初始值（即回退到 UA/User 来源的行为）。
    Revert,
    /// revert-layer 关键字 — 回退到上一个 @layer。
    ///
    /// 简化实现：等同于 unset（跳过当前层声明后按 unset 规则处理）。
    RevertLayer,
    /// 具体属性值（借用级联值，不克隆——热路径每属性每元素省一次 String 分配）。
    Concrete(&'a str),
}

/// 解析 CSS 全局关键字。
///
/// 返回关键字类型或具体值（具体值借用输入，调用方不得在 `cascaded` 生命周期
/// 之外使用——本函数仅在同一函数内消费）。
fn resolve_keyword<'a>(value: &'a str, _property: &str, _parent: Option<&ComputedStyle>) -> KeywordResolution<'a> {
    let trimmed = value.trim();
    match trimmed {
        "inherit" => KeywordResolution::Inherit,
        "initial" => KeywordResolution::Initial,
        "unset" => KeywordResolution::Unset,
        // revert 回退到上一层来源级联值（Author -> User -> UA），
        // 简化实现：跳过作者级联，继承属性使用父元素值，非继承属性使用初始值
        "revert" => KeywordResolution::Revert,
        // revert-layer 回退到上一个 @layer，
        // 简化实现：等同于 unset（跳过当前层声明后按 unset 规则处理）
        "revert-layer" => KeywordResolution::RevertLayer,
        _ => KeywordResolution::Concrete(trimmed),
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::property::{ComputedStyle, LineHeightValue};
    use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, PositionValue, VisibilityValue};

    /// 创建一个带自定义 color 和 font-size 的父样式。
    fn make_parent_style() -> ComputedStyle {
        let mut style = ComputedStyle::default();
        style.color = ColorValue::Rgba(255, 0, 0, 255);
        style.font_size = LengthValue::Px(20.0);
        style.visibility = VisibilityValue::Hidden;
        style
    }

    #[test]
    fn test_inherit_color_from_parent() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "inherit".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_initial_resets_to_default() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "initial".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255)); // initial = black
    }

    #[test]
    fn test_unset_inherited_property() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "unset".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // color 是继承属性，unset = inherit
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    fn test_unset_non_inherited_property() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("display".to_string(), "unset".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // display 不是继承属性，unset = initial
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    fn test_implicit_inheritance() {
        let parent = make_parent_style();
        let cascaded = HashMap::new(); // 没有任何级联属性

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // color 应该从父元素隐式继承
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
        // font-size 也应该继承
        assert_eq!(style.font_size, LengthValue::Px(20.0));
    }

    #[test]
    fn test_no_parent_uses_initial() {
        let cascaded = HashMap::new();
        let style = compute_inherited_style(None, &cascaded);
        // 根元素：所有属性使用初始值
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(style.font_size, LengthValue::Px(16.0));
    }

    #[test]
    fn test_concrete_value_applied() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "blue".to_string());
        cascaded.insert("display".to_string(), "flex".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 255, 255));
        assert_eq!(style.display, DisplayValue::Flex);
    }

    #[test]
    fn test_non_inherited_property_not_from_parent() {
        let mut parent = ComputedStyle::default();
        parent.display = DisplayValue::Flex;
        let cascaded = HashMap::new();

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // display 不继承，应该使用初始值
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    fn test_revert_keyword() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "revert".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // revert 跳过作者级联，color 是继承属性所以使用父元素值
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    /// revert 对继承属性使用父元素的值
    fn test_revert_uses_inherited_value() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "revert".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // color 是继承属性，revert 回退到父元素计算值（即 red）
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    /// revert 对非继承属性使用初始值
    fn test_revert_non_inherited_uses_initial() {
        let mut parent = ComputedStyle::default();
        parent.margin_top = LengthValue::Px(10.0);
        let mut cascaded = HashMap::new();
        cascaded.insert("margin-top".to_string(), "revert".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // margin-top 不是继承属性，revert 使用初始值 0
        assert_eq!(style.margin_top, LengthValue::Px(0.0));
    }

    #[test]
    /// revert 在根元素上回退到初始值
    fn test_revert_on_root_uses_initial() {
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "revert".to_string());

        let style = compute_inherited_style(None, &cascaded);
        // 根元素无父元素，revert 使用初始值 black
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    fn test_inherit_on_root_uses_initial() {
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "inherit".to_string());

        let style = compute_inherited_style(None, &cascaded);
        // 根元素没有父，inherit 等于 initial
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    fn test_multiple_properties_mixed() {
        let parent = make_parent_style();
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "green".to_string());
        cascaded.insert("display".to_string(), "block".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 128, 0, 255)); // green
        assert_eq!(style.display, DisplayValue::Block);
        // font-size 应该隐式继承
        assert_eq!(style.font_size, LengthValue::Px(20.0));
    }

    #[test]
    fn test_visibility_inherited() {
        let parent = make_parent_style();
        let cascaded = HashMap::new();

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.visibility, VisibilityValue::Hidden);
    }

    #[test]
    /// opacity 按 CSS 规范不是继承属性，子元素默认为 1.0
    fn test_opacity_not_inherited() {
        let mut parent = ComputedStyle::default();
        parent.opacity = 0.5;
        let cascaded = HashMap::new();

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // opacity 不继承，子元素默认 1.0
        assert_eq!(style.opacity, 1.0);
    }

    #[test]
    /// cursor 按 CSS 规范是继承属性
    fn test_cursor_inherited() {
        use crate::CursorValue;
        let mut parent = ComputedStyle::default();
        parent.cursor = CursorValue::Pointer;
        let cascaded = HashMap::new();

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.cursor, CursorValue::Pointer);
    }

    #[test]
    /// text-decoration 按 CSS 规范不是继承属性
    fn test_text_decoration_not_inherited() {
        use super::super::property::TextDecorationValue;
        let mut parent = ComputedStyle::default();
        parent.text_decoration = TextDecorationValue::Underline;
        let cascaded = HashMap::new();

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // text-decoration 不继承，子元素默认 None
        assert_eq!(style.text_decoration, TextDecorationValue::None);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增继承测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// color 通过 DOM 树多层继承
    fn test_color_inheritance_through_tree() {
        // grandparent 有 color: red
        let mut grandparent = ComputedStyle::default();
        grandparent.color = ColorValue::Rgba(255, 0, 0, 255);

        // parent 无级联值，隐式继承
        let parent_cascaded = HashMap::new();
        let parent = compute_inherited_style(Some(&grandparent), &parent_cascaded);
        assert_eq!(parent.color, ColorValue::Rgba(255, 0, 0, 255));

        // child 也无级联值，继续继承
        let child_cascaded = HashMap::new();
        let child = compute_inherited_style(Some(&parent), &child_cascaded);
        assert_eq!(child.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    /// font 系列属性继承（font-weight, font-style, line-height）
    fn test_font_properties_inheritance() {
        let mut parent = ComputedStyle::default();
        parent.font_weight = zero_css_parser::values::FontWeightValue::Bold;
        parent.font_style = zero_css_parser::values::FontStyleValue::Italic;
        parent.line_height = crate::property::LineHeightValue::Number(1.5);

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);

        assert_eq!(style.font_weight, zero_css_parser::values::FontWeightValue::Bold);
        assert_eq!(style.font_style, zero_css_parser::values::FontStyleValue::Italic);
        assert_eq!(style.line_height, crate::property::LineHeightValue::Number(1.5));
    }

    #[test]
    fn test_relative_font_weight_mapping() {
        let cases = [
            (100, 400, 100),
            (300, 400, 100),
            (400, 700, 100),
            (500, 700, 100),
            (600, 900, 400),
            (700, 900, 400),
            (800, 900, 700),
            (900, 900, 700),
        ];
        for (parent_weight, expected_bolder, expected_lighter) in cases {
            let mut parent = ComputedStyle::default();
            parent.font_weight = FontWeightValue::Absolute(parent_weight);

            let mut bolder = HashMap::new();
            bolder.insert("font-weight".to_string(), "bolder".to_string());
            assert_eq!(
                compute_inherited_style(Some(&parent), &bolder).font_weight,
                FontWeightValue::Absolute(expected_bolder)
            );

            let mut lighter = HashMap::new();
            lighter.insert("font-weight".to_string(), "lighter".to_string());
            assert_eq!(
                compute_inherited_style(Some(&parent), &lighter).font_weight,
                FontWeightValue::Absolute(expected_lighter)
            );
        }
    }

    #[test]
    fn test_relative_font_weight_resolves_through_parent_chain() {
        let mut grandparent = ComputedStyle::default();
        grandparent.font_weight = FontWeightValue::Absolute(300);

        let mut cascaded = HashMap::new();
        cascaded.insert("font-weight".to_string(), "bolder".to_string());
        let parent = compute_inherited_style(Some(&grandparent), &cascaded);
        let child = compute_inherited_style(Some(&parent), &cascaded);

        assert_eq!(parent.font_weight, FontWeightValue::Absolute(400));
        assert_eq!(child.font_weight, FontWeightValue::Absolute(700));
    }

    #[test]
    fn test_root_relative_font_weight_uses_normal_parent() {
        let mut cascaded = HashMap::new();
        cascaded.insert("font-weight".to_string(), "bolder".to_string());
        assert_eq!(
            compute_inherited_style(None, &cascaded).font_weight,
            FontWeightValue::Absolute(700)
        );

        cascaded.insert("font-weight".to_string(), "lighter".to_string());
        assert_eq!(
            compute_inherited_style(None, &cascaded).font_weight,
            FontWeightValue::Absolute(100)
        );
    }

    #[test]
    /// 非继承属性（border, margin, padding）不从父元素继承
    fn test_non_inherited_border_margin_padding() {
        let mut parent = ComputedStyle::default();
        parent.border_top_width = LengthValue::Px(5.0);
        parent.margin_top = LengthValue::Px(10.0);
        parent.padding_top = LengthValue::Px(8.0);

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);

        // 这些属性不继承，子元素使用初始值（border-width 初始 = medium=3px，CSS §8.5.1）
        assert_eq!(style.border_top_width, LengthValue::Px(3.0));
        assert_eq!(style.margin_top, LengthValue::Px(0.0));
        assert_eq!(style.padding_top, LengthValue::Px(0.0));
    }

    #[test]
    /// 显式 inherit 关键字获取父元素的值
    fn test_explicit_inherit_keyword() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(0, 128, 0, 255); // green
        parent.font_size = LengthValue::Px(24.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("font-size".to_string(), "inherit".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // 显式 inherit 取父元素的值
        assert_eq!(style.font_size, LengthValue::Px(24.0));
    }

    #[test]
    /// left/top/right/bottom 显式 inherit 取父元素计算值（CSS 2.1：inset 的 computed
    /// value 是 specified value，即使父 position:static 不应用，值仍保留供 inherit 取用）。
    /// 对应 WPT CSS2/visuren/inherit-static-offset-001/003、positioning/left-113。
    fn test_inset_inherit_keyword() {
        let mut parent = ComputedStyle::default();
        parent.left = LengthValue::Px(50.0);
        parent.top = LengthValue::Px(50.0);
        parent.right = LengthValue::Px(30.0);
        parent.bottom = LengthValue::Px(30.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("left".to_string(), "inherit".to_string());
        cascaded.insert("top".to_string(), "inherit".to_string());
        cascaded.insert("right".to_string(), "inherit".to_string());
        cascaded.insert("bottom".to_string(), "inherit".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.left, LengthValue::Px(50.0), "left:inherit 应取父 left");
        assert_eq!(style.top, LengthValue::Px(50.0), "top:inherit 应取父 top");
        assert_eq!(style.right, LengthValue::Px(30.0), "right:inherit 应取父 right");
        assert_eq!(style.bottom, LengthValue::Px(30.0), "bottom:inherit 应取父 bottom");
    }

    #[test]
    /// text-align 继承
    fn test_text_align_inherited() {
        let mut parent = ComputedStyle::default();
        parent.text_align = crate::property::TextAlignValue::Center;

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.text_align, crate::property::TextAlignValue::Center);
    }

    #[test]
    /// text-align: match-parent — 继承父值，但 start/end 按父 direction 解析（CSS Text 3 §6.1）
    fn test_text_align_match_parent() {
        use crate::property::{DirectionValue, TextAlignValue};

        let mut cascaded = HashMap::new();
        cascaded.insert("text-align".to_string(), "match-parent".to_string());

        // 父 LTR + start → match-parent 解析为 Left
        let mut parent_ltr = ComputedStyle::default();
        parent_ltr.text_align = TextAlignValue::Start;
        parent_ltr.direction = DirectionValue::Ltr;
        let style = compute_inherited_style(Some(&parent_ltr), &cascaded);
        assert_eq!(
            style.text_align,
            TextAlignValue::Left,
            "match-parent: parent LTR start → Left"
        );

        // 父 RTL + start → Right
        let mut parent_rtl = ComputedStyle::default();
        parent_rtl.text_align = TextAlignValue::Start;
        parent_rtl.direction = DirectionValue::Rtl;
        let style = compute_inherited_style(Some(&parent_rtl), &cascaded);
        assert_eq!(
            style.text_align,
            TextAlignValue::Right,
            "match-parent: parent RTL start → Right"
        );

        // 父 RTL + end → Left
        let mut parent_end = ComputedStyle::default();
        parent_end.text_align = TextAlignValue::End;
        parent_end.direction = DirectionValue::Rtl;
        let style = compute_inherited_style(Some(&parent_end), &cascaded);
        assert_eq!(
            style.text_align,
            TextAlignValue::Left,
            "match-parent: parent RTL end → Left"
        );

        // 父值非 start/end（Center）→ 直接继承 Center
        let mut parent_center = ComputedStyle::default();
        parent_center.text_align = TextAlignValue::Center;
        let style = compute_inherited_style(Some(&parent_center), &cascaded);
        assert_eq!(
            style.text_align,
            TextAlignValue::Center,
            "match-parent: parent center inherited as-is"
        );

        // 根元素（无父）match-parent → initial (start)
        let style = compute_inherited_style(None, &cascaded);
        assert_eq!(
            style.text_align,
            TextAlignValue::Start,
            "match-parent root → initial start"
        );
    }

    #[test]
    /// word-spacing 继承
    fn test_word_spacing_inherited() {
        let mut parent = ComputedStyle::default();
        parent.word_spacing = LengthValue::Px(4.0);

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.word_spacing, LengthValue::Px(4.0));
    }

    #[test]
    /// revert-layer 关键字简化为 unset 行为
    fn test_revert_layer_keyword() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255);

        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "revert-layer".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // revert-layer 等同于 unset，color 是继承属性所以使用父元素值
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    #[test]
    /// revert-layer 对非继承属性使用初始值
    fn test_revert_layer_non_inherited_uses_initial() {
        let mut parent = ComputedStyle::default();
        parent.margin_top = LengthValue::Px(10.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("margin-top".to_string(), "revert-layer".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // margin-top 不是继承属性，revert-layer 等同于 unset = initial
        assert_eq!(style.margin_top, LengthValue::Px(0.0));
    }

    // ═══════════════════════════════════════════════════════════════════
    // 新增继承边界条件测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// inherit 关键字对所有属性（包括非继承属性）都从父元素复制计算值
    fn test_explicit_inherit_on_non_inherited_property() {
        let mut parent = ComputedStyle::default();
        parent.margin_top = LengthValue::Px(42.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("margin-top".to_string(), "inherit".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // margin-top 虽非继承属性，但 inherit 关键字显式要求从父元素复制
        assert_eq!(style.margin_top, LengthValue::Px(42.0));
    }

    #[test]
    /// initial 关键字重置继承属性为初始值（不取父元素值）
    fn test_initial_resets_inherited_property() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255);
        parent.font_size = LengthValue::Px(32.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "initial".to_string());
        cascaded.insert("font-size".to_string(), "initial".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(style.font_size, LengthValue::Px(16.0));
    }

    #[test]
    /// unset 关键字：继承属性 = inherit，非继承属性 = initial
    fn test_unset_dual_behavior() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(0, 128, 0, 255);
        parent.display = DisplayValue::Flex;

        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "unset".to_string());
        cascaded.insert("display".to_string(), "unset".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // color 继承 → 取父元素值
        assert_eq!(style.color, ColorValue::Rgba(0, 128, 0, 255));
        // display 非继承 → 初始值 Inline
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    /// 隐式继承：无级联值的继承属性自动从父元素取值
    fn test_implicit_inheritance_multiple_properties() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255);
        parent.font_size = LengthValue::Px(24.0);
        parent.visibility = VisibilityValue::Hidden;
        parent.line_height = LineHeightValue::Number(1.8);

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);

        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
        assert_eq!(style.font_size, LengthValue::Px(24.0));
        assert_eq!(style.visibility, VisibilityValue::Hidden);
        assert_eq!(style.line_height, LineHeightValue::Number(1.8));
    }

    #[test]
    /// 根元素：inherit 等同于 initial（无父元素）
    fn test_root_inherit_equals_initial() {
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "inherit".to_string());
        cascaded.insert("font-size".to_string(), "inherit".to_string());

        let style = compute_inherited_style(None, &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(style.font_size, LengthValue::Px(16.0));
    }

    #[test]
    /// 根元素：unset 继承属性也等同于 initial（无父元素）
    fn test_root_unset_inherited_equals_initial() {
        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "unset".to_string());

        let style = compute_inherited_style(None, &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
    }

    #[test]
    /// 非继承属性在无级联值时使用初始值
    fn test_non_inherited_default_values() {
        let mut parent = ComputedStyle::default();
        parent.width = LengthValue::Px(500.0);
        parent.margin_top = LengthValue::Px(10.0);
        parent.padding_left = LengthValue::Px(5.0);
        parent.border_top_width = LengthValue::Px(3.0);
        parent.display = DisplayValue::Grid;

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);

        assert_eq!(style.width, LengthValue::Auto);
        assert_eq!(style.margin_top, LengthValue::Px(0.0));
        assert_eq!(style.padding_left, LengthValue::Px(0.0));
        // border-width 初始 = medium=3px（CSS §8.5.1）
        assert_eq!(style.border_top_width, LengthValue::Px(3.0));
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    /// 混合级联属性：一个显式值，一个 inherit，一个 unset
    fn test_mixed_keyword_resolution() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255);
        parent.font_size = LengthValue::Px(20.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "blue".to_string());
        cascaded.insert("font-size".to_string(), "inherit".to_string());
        cascaded.insert("display".to_string(), "unset".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(0, 0, 255, 255));
        assert_eq!(style.font_size, LengthValue::Px(20.0));
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    /// revert 对非继承属性使用初始值
    fn test_revert_non_inherited_uses_initial_display() {
        let mut parent = ComputedStyle::default();
        parent.display = DisplayValue::Grid;

        let mut cascaded = HashMap::new();
        cascaded.insert("display".to_string(), "revert".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.display, DisplayValue::Inline);
    }

    #[test]
    /// 值带前后空格时应正确解析关键字
    fn test_keyword_with_whitespace() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255);

        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "  inherit  ".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
    }

    // ── 新增边界测试 ──

    #[test]
    /// 根元素无父样式时，inherit 对非继承属性使用初始值
    fn test_inherit_no_parent_non_inherited() {
        let mut cascaded = HashMap::new();
        cascaded.insert("display".to_string(), "inherit".to_string());

        let style = compute_inherited_style(None, &cascaded);
        // display 非继承，无父样式 → 回退到 initial
        assert_eq!(
            style.display,
            DisplayValue::Inline,
            "无父样式时 inherit 对非继承属性应使用 initial"
        );
    }

    #[test]
    /// 空级联属性映射应使用所有默认初始值
    fn test_empty_cascaded_all_initial() {
        let cascaded = HashMap::new();
        let style = compute_inherited_style(None, &cascaded);

        assert_eq!(style.display, DisplayValue::Inline);
        assert_eq!(style.position, PositionValue::Static);
        assert_eq!(style.width, LengthValue::Auto);
    }

    #[test]
    /// initial 关键字对继承属性恢复默认值
    fn test_initial_on_inherited_property() {
        let mut parent = ComputedStyle::default();
        parent.color = ColorValue::Rgba(255, 0, 0, 255);
        parent.font_size = LengthValue::Px(24.0);

        let mut cascaded = HashMap::new();
        cascaded.insert("color".to_string(), "initial".to_string());
        cascaded.insert("font-size".to_string(), "initial".to_string());

        let style = compute_inherited_style(Some(&parent), &cascaded);
        // color initial = black (default), font-size initial = medium (16px)
        assert_eq!(
            style.color,
            ColorValue::Rgba(0, 0, 0, 255),
            "initial 应恢复 color 默认值"
        );
    }

    #[test]
    /// writing-mode 隐式继承和显式 inherit 关键字测试
    fn test_writing_mode_inheritance() {
        use crate::property::PropertyRegistry;
        use crate::property::types::WritingModeValue;

        // writing-mode 是继承属性
        assert!(
            PropertyRegistry::is_inherited("writing-mode"),
            "writing-mode should be in inherited property list"
        );

        // 隐式继承：子元素从父元素继承 writing-mode
        let mut parent = ComputedStyle::default();
        parent.writing_mode = WritingModeValue::VerticalRl;
        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(
            style.writing_mode,
            WritingModeValue::VerticalRl,
            "writing-mode should implicitly inherit from parent"
        );

        // 显式 inherit 关键字
        let mut cascaded2 = HashMap::new();
        cascaded2.insert("writing-mode".to_string(), "inherit".to_string());
        let style2 = compute_inherited_style(Some(&parent), &cascaded2);
        assert_eq!(
            style2.writing_mode,
            WritingModeValue::VerticalRl,
            "writing-mode with explicit 'inherit' should use parent value"
        );
    }
}
