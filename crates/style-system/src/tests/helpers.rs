//! 共享测试辅助函数。

#[allow(unused_imports)]
use super::super::*;
pub use zero_css_parser::ast::{
    Combinator, ComplexSelector, CompoundSelector, Declaration, Rule, Selector, StyleRule, SubclassSelector,
    TypeSelector,
};
#[allow(unused_imports)]
pub use zero_css_parser::values::{ColorValue, DisplayValue, LengthValue, OverflowValue};
pub use zero_dom::{Document, NodeId};

/// 创建测试 DOM：html > body > div#main > p.text
pub fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();

    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "main");
    doc.append_child(body, div).unwrap();

    let p = doc.create_element("p");
    doc.set_attribute(p, "class", "text");
    doc.append_child(div, p).unwrap();

    (doc, html, body, div, p)
}

pub fn make_tag_selector(tag: &str) -> Selector {
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

pub fn make_compound_selector(
    type_selector: Option<TypeSelector>,
    subclass_selectors: Vec<SubclassSelector>,
) -> CompoundSelector {
    CompoundSelector {
        type_selector,
        subclass_selectors,
    }
}

pub fn make_complex_selector(parts: Vec<(CompoundSelector, Option<zero_css_parser::ast::Combinator>)>) -> Selector {
    Selector {
        complex: ComplexSelector { parts },
    }
}
