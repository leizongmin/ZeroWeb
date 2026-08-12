//! StyleSystem 端到端补充覆盖测试。
//!
//! 覆盖 StyleSystem 的视口单位计算、ComputedStyle 默认值验证、
//! 级联优先级、继承等场景。

use super::super::*;
use zero_css_parser::ast::{
    ComplexSelector, CompoundSelector, Declaration, Rule, Selector, StyleRule, SubclassSelector, TypeSelector,
};
use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, OverflowValue, PositionValue};
use zero_dom::{Document, NodeId};

/// 辅助：创建包含 body 的文档，返回 (doc, body_id)。
fn make_doc_with_body() -> (Document, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, body)
}

/// 辅助：创建标签选择器。
fn make_tag_selector(tag: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    }
}

/// 辅助：创建类选择器。
fn make_class_selector(class: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class(class.to_string())],
                },
                None,
            )],
        },
    }
}

/// 辅助：创建 ID 选择器。
fn make_id_selector(id: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id(id.to_string())],
                },
                None,
            )],
        },
    }
}

/// 辅助：从属性+值构建 StyleRule。
fn make_style_rule(selectors: Vec<Selector>, decls: Vec<(&str, &str)>) -> Rule {
    Rule::Style(StyleRule {
        selectors,
        declarations: decls
            .into_iter()
            .map(|(p, v)| Declaration {
                property: p.to_string(),
                value: v.to_string(),
                important: false,
            })
            .collect(),
    })
}

fn make_stylesheet(rules: Vec<Rule>) -> Stylesheet {
    Stylesheet { rules }
}

// ═══════════════════════════════════════════════════════════════════════
// ComputedStyle 默认值测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_computed_style_default_display_inline() {
    let style = ComputedStyle::default();
    assert!(matches!(style.display, DisplayValue::Inline));
}

#[test]
fn test_computed_style_default_position_static() {
    let style = ComputedStyle::default();
    assert!(matches!(style.position, PositionValue::Static));
}

#[test]
fn test_computed_style_default_overflow_visible() {
    let style = ComputedStyle::default();
    assert!(matches!(style.overflow_x, OverflowValue::Visible));
    assert!(matches!(style.overflow_y, OverflowValue::Visible));
}

#[test]
fn test_computed_style_default_margin_zero() {
    let style = ComputedStyle::default();
    assert!(matches!(style.margin_top, LengthValue::Px(0.0)));
    assert!(matches!(style.margin_right, LengthValue::Px(0.0)));
    assert!(matches!(style.margin_bottom, LengthValue::Px(0.0)));
    assert!(matches!(style.margin_left, LengthValue::Px(0.0)));
}

#[test]
fn test_computed_style_default_padding_zero() {
    let style = ComputedStyle::default();
    assert!(matches!(style.padding_top, LengthValue::Px(0.0)));
    assert!(matches!(style.padding_right, LengthValue::Px(0.0)));
    assert!(matches!(style.padding_bottom, LengthValue::Px(0.0)));
    assert!(matches!(style.padding_left, LengthValue::Px(0.0)));
}

#[test]
fn test_computed_style_default_border_width_medium() {
    // border-width 初始值 = medium（CSS §8.5.1，ZeroWeb 取 3px）。实际无布局边框，
    // 因为 border-style 初始 = none，converter 在 style=none 时把 width 抑制为 0。
    let style = ComputedStyle::default();
    assert!(matches!(style.border_top_width, LengthValue::Px(3.0)));
    assert!(matches!(style.border_right_width, LengthValue::Px(3.0)));
    assert!(matches!(style.border_bottom_width, LengthValue::Px(3.0)));
    assert!(matches!(style.border_left_width, LengthValue::Px(3.0)));
}

#[test]
fn test_computed_style_default_sizing_auto() {
    let style = ComputedStyle::default();
    assert!(matches!(style.width, LengthValue::Auto));
    assert!(matches!(style.height, LengthValue::Auto));
}

#[test]
fn test_computed_style_default_font_size_16px() {
    let style = ComputedStyle::default();
    if let LengthValue::Px(v) = style.font_size {
        assert_eq!(v, 16.0, "默认字体大小应为 16px");
    } else {
        panic!("font_size 应为 Px(16.0)");
    }
}

#[test]
fn test_computed_style_default_float_none() {
    let style = ComputedStyle::default();
    assert!(matches!(style.float, zero_css_parser::values::FloatValue::None));
}

#[test]
fn test_computed_style_default_clear_none() {
    let style = ComputedStyle::default();
    assert!(matches!(style.clear, zero_css_parser::values::ClearValue::None));
}

#[test]
fn test_computed_style_default_color_is_rgba() {
    let style = ComputedStyle::default();
    assert!(matches!(style.color, ColorValue::Rgba(..)));
}

#[test]
fn test_computed_style_clone() {
    let style = ComputedStyle::default();
    let cloned = style.clone();
    assert!(matches!(cloned.display, DisplayValue::Inline));
    assert!(matches!(cloned.position, PositionValue::Static));
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 视口设置
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_viewport_affects_vw() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("width", "50vw")],
    )]);

    let mut sys = StyleSystem::new();
    sys.set_viewport(1000.0, 800.0);
    let styles = sys.compute_styles(&doc, &[ss]);
    let div_style = styles.get(&div).expect("div style");

    if let LengthValue::Px(v) = div_style.width {
        assert!((v - 500.0).abs() < 1.0, "50vw 在 1000px 视口应为 ~500px, got {v}");
    }
}

#[test]
fn test_style_system_viewport_affects_vh() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("height", "25vh")],
    )]);

    let mut sys = StyleSystem::new();
    sys.set_viewport(1000.0, 800.0);
    let styles = sys.compute_styles(&doc, &[ss]);
    let div_style = styles.get(&div).expect("div style");

    if let LengthValue::Px(v) = div_style.height {
        assert!((v - 200.0).abs() < 1.0, "25vh 在 800px 视口应为 ~200px, got {v}");
    }
}

#[test]
fn test_style_system_vmin_unit() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("width", "10vmin")],
    )]);

    let mut sys = StyleSystem::new();
    sys.set_viewport(1000.0, 800.0);
    let styles = sys.compute_styles(&doc, &[ss]);
    let div_style = styles.get(&div).expect("div style");

    if let LengthValue::Px(v) = div_style.width {
        // 10vmin = 10% * min(1000, 800) = 80
        assert!((v - 80.0).abs() < 1.0, "10vmin 应为 ~80px, got {v}");
    }
}

#[test]
fn test_style_system_vmax_unit() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("width", "10vmax")],
    )]);

    let mut sys = StyleSystem::new();
    sys.set_viewport(1000.0, 800.0);
    let styles = sys.compute_styles(&doc, &[ss]);
    let div_style = styles.get(&div).expect("div style");

    if let LengthValue::Px(v) = div_style.width {
        // 10vmax = 10% * max(1000, 800) = 100
        assert!((v - 100.0).abs() < 1.0, "10vmax 应为 ~100px, got {v}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 基本属性计算
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_color_red() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("color", "red")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let div_style = styles.get(&div).expect("div style");

    assert_eq!(div_style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
fn test_style_system_background_color_blue() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("background-color", "blue")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let div_style = styles.get(&div).expect("div style");

    assert_eq!(div_style.background_color, ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_style_system_display_none() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("display", "none")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().display, DisplayValue::None));
}

/// R3290：`<dialog>` 元素的 UA display 规则。
/// HTML Living Standard UA 样式表：`dialog:not([open]) { display: none }`——
/// 无 open 属性的 dialog 默认不渲染；`dialog[open]`（经 show/showModal 或 open 内容属性）应渲染。
/// https://html.spec.whatwg.org/multipage/rendering.html#the-dialog-element-2
#[test]
fn test_dialog_display_open_attribute_r3290() {
    // 无 open 属性 → display:none（ua_default_display 基础规则，未变）。
    let (mut doc, body) = make_doc_with_body();
    let dialog_closed = doc.create_element("dialog");
    doc.append_child(body, dialog_closed).unwrap();
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    assert!(
        matches!(styles.get(&dialog_closed).unwrap().display, DisplayValue::None),
        "<dialog> 无 open 属性 → display:none"
    );

    // 有 open 属性 → display:block（R3290 显式 UA 覆盖，UA 优先级 0,0,0）。
    let (mut doc2, body2) = make_doc_with_body();
    let dialog_open = doc2.create_element("dialog");
    doc2.set_attribute(dialog_open, "open", "");
    doc2.append_child(body2, dialog_open).unwrap();
    let mut sys2 = StyleSystem::new();
    let styles2 = sys2.compute_styles(&doc2, &[]);
    assert!(
        matches!(styles2.get(&dialog_open).unwrap().display, DisplayValue::Block),
        "<dialog open> → display:block（R3290 UA 覆盖）"
    );

    // 作者样式 display:flex 覆盖 UA（author 优先级 > UA 0,0,0）——验证 UA 覆盖可被作者样式盖过。
    let (mut doc3, body3) = make_doc_with_body();
    let dialog_author = doc3.create_element("dialog");
    doc3.set_attribute(dialog_author, "open", "");
    doc3.append_child(body3, dialog_author).unwrap();
    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("dialog")],
        vec![("display", "flex")],
    )]);
    let mut sys3 = StyleSystem::new();
    let styles3 = sys3.compute_styles(&doc3, &[ss]);
    assert!(
        matches!(styles3.get(&dialog_author).unwrap().display, DisplayValue::Flex),
        "作者 display:flex 覆盖 dialog[open] UA block（author > UA 优先级）"
    );
}

#[test]
fn test_style_system_display_flex() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("display", "flex")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().display, DisplayValue::Flex));
}

#[test]
fn test_style_system_display_grid() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("display", "grid")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().display, DisplayValue::Grid));
}

#[test]
fn test_style_system_display_inline() {
    let (mut doc, body) = make_doc_with_body();
    let span = doc.create_element("span");
    doc.append_child(body, span).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("span")],
        vec![("display", "inline")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&span).unwrap().display, DisplayValue::Inline));
}

#[test]
fn test_style_system_position_absolute() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("position", "absolute")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().position, PositionValue::Absolute));
}

#[test]
fn test_style_system_position_fixed() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("position", "fixed")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().position, PositionValue::Fixed));
}

#[test]
fn test_style_system_position_relative() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("position", "relative")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().position, PositionValue::Relative));
}

#[test]
fn test_style_system_position_sticky() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("position", "sticky")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().position, PositionValue::Sticky));
}

#[test]
fn test_style_system_overflow_hidden() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("overflow", "hidden")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();
    assert!(matches!(s.overflow_x, OverflowValue::Hidden));
    assert!(matches!(s.overflow_y, OverflowValue::Hidden));
}

#[test]
fn test_style_system_overflow_scroll() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("overflow", "scroll")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();
    assert!(matches!(s.overflow_x, OverflowValue::Scroll));
    assert!(matches!(s.overflow_y, OverflowValue::Scroll));
}

#[test]
fn test_style_system_px_width_height() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("width", "200px"), ("height", "100px")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    if let LengthValue::Px(w) = s.width {
        assert!((w - 200.0).abs() < 0.1, "width 应为 ~200px, got {w}");
    }
    if let LengthValue::Px(h) = s.height {
        assert!((h - 100.0).abs() < 0.1, "height 应为 ~100px, got {h}");
    }
}

#[test]
fn test_style_system_em_font_size() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("font-size", "2em")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    if let LengthValue::Px(v) = s.font_size {
        assert!((v - 32.0).abs() < 0.1, "2em 应为 ~32px, got {v}");
    }
}

#[test]
fn test_style_system_rem_font_size() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("font-size", "1.5rem")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    if let LengthValue::Px(v) = s.font_size {
        assert!((v - 24.0).abs() < 0.1, "1.5rem 应为 ~24px, got {v}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 级联优先级
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_later_rule_wins() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![
        make_style_rule(vec![make_tag_selector("div")], vec![("color", "red")]),
        make_style_rule(vec![make_tag_selector("div")], vec![("color", "blue")]),
    ]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    assert_eq!(s.color, ColorValue::Rgba(0, 0, 255, 255), "后声明的规则应胜出");
}

#[test]
fn test_style_system_higher_specificity_wins() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "myid");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![
        make_style_rule(vec![make_tag_selector("div")], vec![("color", "red")]),
        make_style_rule(vec![make_id_selector("myid")], vec![("color", "green")]),
    ]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    assert_eq!(s.color, ColorValue::Rgba(0, 128, 0, 255), "高特异性 ID 选择器应胜出");
}

#[test]
fn test_style_system_important_overrides() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "myid");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![
        Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: true,
            }],
        }),
        make_style_rule(vec![make_id_selector("myid")], vec![("color", "green")]),
    ]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    assert_eq!(s.color, ColorValue::Rgba(0, 0, 255, 255), "!important 应胜过更高特异性");
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 多元素测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_multiple_elements_different_styles() {
    let (mut doc, body) = make_doc_with_body();
    let div1 = doc.create_element("div");
    doc.set_attribute(div1, "class", "a");
    let div2 = doc.create_element("div");
    doc.set_attribute(div2, "class", "b");
    doc.append_child(body, div1).unwrap();
    doc.append_child(body, div2).unwrap();

    let ss = make_stylesheet(vec![
        make_style_rule(vec![make_class_selector("a")], vec![("color", "red")]),
        make_style_rule(vec![make_class_selector("b")], vec![("color", "blue")]),
    ]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);

    assert_eq!(styles.get(&div1).unwrap().color, ColorValue::Rgba(255, 0, 0, 255));
    assert_eq!(styles.get(&div2).unwrap().color, ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
fn test_style_system_nested_em_inheritance() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.set_attribute(outer, "class", "outer");
    let inner = doc.create_element("span");
    doc.set_attribute(inner, "class", "inner");
    doc.append_child(body, outer).unwrap();
    doc.append_child(outer, inner).unwrap();

    let ss = make_stylesheet(vec![
        make_style_rule(vec![make_class_selector("outer")], vec![("font-size", "20px")]),
        make_style_rule(vec![make_class_selector("inner")], vec![("font-size", "2em")]),
    ]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let inner_style = styles.get(&inner).expect("inner");

    // 2em 相对于父级 20px = 40px
    if let LengthValue::Px(v) = inner_style.font_size {
        assert!((v - 40.0).abs() < 0.1, "嵌套 2em 应为 ~40px, got {v}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 继承测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_color_inheritance() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(body, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("color", "green")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);

    assert_eq!(styles.get(&parent).unwrap().color, ColorValue::Rgba(0, 128, 0, 255));
    assert_eq!(
        styles.get(&child).unwrap().color,
        ColorValue::Rgba(0, 128, 0, 255),
        "color 应从父元素继承"
    );
}

#[test]
fn test_style_system_font_size_inheritance() {
    let (mut doc, body) = make_doc_with_body();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(body, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("font-size", "24px")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);

    if let LengthValue::Px(v) = styles.get(&child).unwrap().font_size {
        assert!((v - 24.0).abs() < 0.1, "font-size 应继承父级的 24px, got {v}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem flex 属性
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_flex_direction_column() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("display", "flex"), ("flex-direction", "column")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(
        styles.get(&div).unwrap().flex_direction,
        zero_css_parser::values::FlexDirectionValue::Column
    ));
}

#[test]
fn test_style_system_flex_direction_row_reverse() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("display", "flex"), ("flex-direction", "row-reverse")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(
        styles.get(&div).unwrap().flex_direction,
        zero_css_parser::values::FlexDirectionValue::RowReverse
    ));
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 边框测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_border_width_shorthand() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("border", "2px solid black")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    if let LengthValue::Px(v) = s.border_top_width {
        assert!((v - 2.0).abs() < 0.1, "border-top-width 应为 ~2px, got {v}");
    }
    if let LengthValue::Px(v) = s.border_left_width {
        assert!((v - 2.0).abs() < 0.1, "border-left-width 应为 ~2px, got {v}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_no_matching_rules() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_class_selector("nonexistent")],
        vec![("color", "red")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    let s = styles.get(&div).unwrap();

    // 应使用默认值
    if let LengthValue::Px(v) = s.font_size {
        assert!(v > 0.0, "font-size 应有合理默认值");
    }
}

#[test]
fn test_style_system_empty_stylesheet() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(styles.get(&div).unwrap().display, DisplayValue::Block));
}

#[test]
fn test_style_system_multiple_stylesheets() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss1 = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("width", "100px")],
    )]);
    let ss2 = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("height", "200px")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss1, ss2]);
    let s = styles.get(&div).unwrap();

    if let LengthValue::Px(w) = s.width {
        assert!((w - 100.0).abs() < 0.1, "width 应为 ~100px, got {w}");
    }
    if let LengthValue::Px(h) = s.height {
        assert!((h - 200.0).abs() < 0.1, "height 应为 ~200px, got {h}");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// StyleSystem float 和 clear
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_style_system_float_left() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("float", "left")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(
        styles.get(&div).unwrap().float,
        zero_css_parser::values::FloatValue::Left
    ));
}

#[test]
fn test_style_system_float_right() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("float", "right")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(
        styles.get(&div).unwrap().float,
        zero_css_parser::values::FloatValue::Right
    ));
}

#[test]
fn test_style_system_clear_both() {
    let (mut doc, body) = make_doc_with_body();
    let div = doc.create_element("div");
    doc.append_child(body, div).unwrap();

    let ss = make_stylesheet(vec![make_style_rule(
        vec![make_tag_selector("div")],
        vec![("clear", "both")],
    )]);

    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[ss]);
    assert!(matches!(
        styles.get(&div).unwrap().clear,
        zero_css_parser::values::ClearValue::Both
    ));
}

/// M3-S9：增量样式（compute_styles_incremental）与全量 compute_styles 一致性。
/// 变更节点 + 子树重算后，完整 styles map 与全量重算完全一致（含继承链/custom/伪元素）。
#[test]
fn test_incremental_styles_match_full_after_change() {
    use zero_css_parser::Parser as CssParser;

    let html = r#"<html><body>
        <div id="a" style="color:red"><span class="c">s1</span><span>s2</span></div>
        <div id="b"><p style="color:blue">p</p></div>
        <section id="s"><div class="c">x</div></section>
    </body></html>"#;
    let css = r#"
        .c { font-size: 14px; }
        #b p { margin: 5px; }
        div { display: block; }
        div::before { content: "pre"; }
    "#;
    let mut doc = zero_dom::parse_html(html);
    let stylesheets = vec![CssParser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let full = sys.compute_styles(&doc, &stylesheets);

    // 变更 #a 的 style 属性（color + 新增自定义属性）
    let a = doc.query_selector(doc.root(), "#a").expect("#a");
    doc.set_attribute(a, "style", "color:green; --x: 10px");
    let a2 = doc.query_selector(doc.root(), "#a").expect("#a");
    assert_eq!(a, a2);

    // 增量重算 #a
    let mut incr = full.clone();
    let mut sys2 = StyleSystem::new();
    sys2.set_viewport(800.0, 600.0);
    sys2.compute_styles_incremental(&doc, &stylesheets, &[a], &mut incr);

    // 全量参考
    let full2 = sys.compute_styles(&doc, &stylesheets);

    // 一致性：全部元素样式相同
    assert_eq!(full2.len(), incr.len(), "map size: {} vs {}", full2.len(), incr.len());
    for (nid, style) in &full2 {
        let incr_style = incr.get(nid).expect("incremental has node");
        assert_eq!(style.color, incr_style.color, "node {nid:?} color");
        assert_eq!(style.font_size, incr_style.font_size, "node {nid:?} font-size");
    }
    // 变更元素 color 生效
    let a_style = incr.get(&a).expect("#a style");
    assert_eq!(
        a_style.color,
        zero_css_parser::values::ColorValue::Rgba(0, 128, 0, 255),
        "green"
    );
}
