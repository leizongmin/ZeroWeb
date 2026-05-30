//! CSS 属性继承。
//!
//! 为元素计算继承样式：处理 inherit、initial、unset、revert 关键字，
//! 以及隐式继承（无显式值时从父元素继承）。

use std::collections::HashMap;

use crate::property::{
    ComputedStyle, PropertyRegistry, apply_initial_value, apply_property_value, inherit_property,
};

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
    /// 具体属性值。
    Concrete(String),
}

/// 解析 CSS 全局关键字。
///
/// 返回关键字类型或具体值。
fn resolve_keyword(
    value: &str,
    _property: &str,
    _parent: Option<&ComputedStyle>,
) -> KeywordResolution {
    let trimmed = value.trim();
    match trimmed {
        "inherit" => KeywordResolution::Inherit,
        "initial" => KeywordResolution::Initial,
        "unset" => KeywordResolution::Unset,
        // revert 需要回退到上一层来源的级联值，
        // 简化实现：等同于 unset
        "revert" => KeywordResolution::Unset,
        // revert-layer 需要回退到上一个 @layer，
        // 简化实现：等同于 unset
        "revert-layer" => KeywordResolution::Unset,
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
        // revert 简化为 unset，color 是继承属性所以 inherit from parent
        assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
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
    fn test_opacity_inherited() {
        let mut parent = ComputedStyle::default();
        parent.opacity = 0.5;
        let cascaded = HashMap::new();

        let style = compute_inherited_style(Some(&parent), &cascaded);
        assert_eq!(style.opacity, 0.5);
    }
}
