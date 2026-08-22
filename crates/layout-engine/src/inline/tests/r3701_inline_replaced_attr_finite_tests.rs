use super::super::*;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_style_system::ComputedStyle;

fn collect_single_replaced_attr_item(tag: &str, width: &str, height: &str) -> Vec<InlineItem> {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let element = doc.create_element(tag);
    doc.set_attribute(element, "width", width);
    doc.set_attribute(element, "height", height);
    doc.append_child(root, element).unwrap();

    let root = doc.root();
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineBlock;
    style.width = LengthValue::Auto;
    style.height = LengthValue::Auto;
    let mut styles = std::collections::HashMap::new();
    styles.insert(element, style);

    let ctx = InlineFormattingContext::new(800.0);
    ctx.collect_inline_items(&doc, root, &styles)
}

/// R3701: inline replaced collection consumes HTML dimension attrs directly; non-finite
/// dimension attrs must not enter inline atomic box sizing.
#[test]
fn r3701_inline_replaced_attr_intrinsic_rejects_non_finite_size() {
    for (tag, width, height) in [
        ("canvas", "Infinity", "100"),
        ("canvas", "100", "Infinity"),
        ("video", "Infinity", "100"),
        ("iframe", "100", "Infinity"),
        ("embed", "Infinity", "100"),
        ("object", "100", "Infinity"),
        ("applet", "Infinity", "100"),
        ("img", "Infinity", "100"),
        ("img", "100", "Infinity"),
    ] {
        for item in collect_single_replaced_attr_item(tag, width, height) {
            assert!(
                !matches!(item, InlineItem::InlineBlock(_)),
                "{tag} {width}x{height} should not produce an attr-driven inline-block, got {item:?}"
            );
        }
    }
}

#[test]
fn r3701_inline_replaced_attr_intrinsic_preserves_finite_size() {
    let items = collect_single_replaced_attr_item("canvas", "120", "80");
    let Some(InlineItem::InlineBlock(block)) = items.first() else {
        panic!("expected finite attrs to produce inline-block, got {items:?}");
    };
    assert_eq!(block.width, 120.0);
    assert_eq!(block.height, 80.0);
}
