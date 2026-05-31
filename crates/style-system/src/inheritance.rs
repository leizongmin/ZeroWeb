//! CSS 属性继承。
//!
//! 为元素计算继承样式：处理 inherit、initial、unset、revert 关键字，
//! 以及隐式继承（无显式值时从父元素继承）。

use std::collections::HashMap;

use crate::property::{ComputedStyle, PropertyRegistry, apply_initial_value, apply_property_value, inherit_property};

/// 为元素计算继承样式。
///
/// # 参数
///
/// - `parent_style` — 父元素的计算样式（根元素为 None）
/// - `cascaded` — 级联后的属性值映射
///
/// # 返回值
///
/// 返回包含继承和初始值的完整计算样式。
pub fn compute_inherited_style(
    parent_style: Option<&ComputedStyle>,
    cascaded: &HashMap<String, String>,
) -> ComputedStyle {
    let mut style = ComputedStyle::default();

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
                apply_property_value(&mut style, property, &v);
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

    style
}

/// CSS 全局关键字解析结果。
#[derive(Debug, Clone, PartialEq)]
enum KeywordResolution {
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
    /// 具体属性值。
    Concrete(String),
}

/// 解析 CSS 全局关键字。
///
/// 返回关键字类型或具体值。
fn resolve_keyword(value: &str, _property: &str, _parent: Option<&ComputedStyle>) -> KeywordResolution {
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
        _ => KeywordResolution::Concrete(trimmed.to_string()),
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::property::ComputedStyle;
    use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, VisibilityValue};

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
    /// 非继承属性（border, margin, padding）不从父元素继承
    fn test_non_inherited_border_margin_padding() {
        let mut parent = ComputedStyle::default();
        parent.border_top_width = LengthValue::Px(5.0);
        parent.margin_top = LengthValue::Px(10.0);
        parent.padding_top = LengthValue::Px(8.0);

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);

        // 这些属性不继承，子元素使用初始值
        assert_eq!(style.border_top_width, LengthValue::Px(0.0));
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
    /// text-align 继承
    fn test_text_align_inherited() {
        let mut parent = ComputedStyle::default();
        parent.text_align = crate::property::TextAlignValue::Center;

        let cascaded = HashMap::new();
        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.text_align, crate::property::TextAlignValue::Center);
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
}
