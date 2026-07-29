// Test file split from matcher.rs — core selector matching tests
use super::super::*;
use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, PseudoClassSelector, Selector,
    SubclassSelector, TypeSelector,
};
use zero_dom::Document;

// ── 辅助函数 ──

pub(super) fn make_tag_selector(tag: &str) -> Selector {
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

pub(super) fn make_id_selector(id: &str) -> Selector {
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

pub(super) fn make_class_selector(cls: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class(cls.to_string())],
                },
                None,
            )],
        },
    }
}

/// 创建一个简单的测试 DOM：html > body > div#main.container > p.text
pub(super) fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();

    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "main");
    doc.set_attribute(div, "class", "container");
    doc.append_child(body, div).unwrap();

    let p = doc.create_element("p");
    doc.set_attribute(p, "class", "text");
    doc.append_child(div, p).unwrap();

    (doc, html, body, div, p)
}
#[test]
fn test_matches_tag_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_tag_selector("div");
    assert!(matches_selector(&doc, div, &sel));

    let sel_p = make_tag_selector("p");
    assert!(!matches_selector(&doc, div, &sel_p));
}

#[test]
fn test_matches_id_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_id_selector("main");
    assert!(matches_selector(&doc, div, &sel));

    let sel_not_found = make_id_selector("other");
    assert!(!matches_selector(&doc, div, &sel_not_found));
}

#[test]
fn test_matches_class_selector() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let sel = make_class_selector("container");
    assert!(matches_selector(&doc, div, &sel));

    let sel_text = make_class_selector("text");
    assert!(matches_selector(&doc, p, &sel_text));
    assert!(!matches_selector(&doc, div, &sel_text));
}

#[test]
fn test_matches_universal_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));
}

#[test]
fn test_matches_descendant_combinator() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    // div p
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Descendant),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_matches_child_combinator() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    // div > p
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Child),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(matches_selector(&doc, p, &sel));

    // body > p 不应该匹配（p 是 div 的子元素，不是 body 的直接子元素）
    let sel2 = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("body".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Child),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(!matches_selector(&doc, p, &sel2));
}

#[test]
fn test_matches_attribute_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    // [id]
    let sel_exists = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "id".to_string(),
                        matcher: AttributeMatcher::Exists,
                        case_insensitive: false,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel_exists));

    // [id=main]
    let sel_exact = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "id".to_string(),
                        matcher: AttributeMatcher::Exact("main".to_string()),
                        case_insensitive: false,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel_exact));
}

/// 辅助：构造 `[name=matcher]` 单一属性选择器。
fn attr_selector(name: &str, matcher: AttributeMatcher) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: name.to_string(),
                        matcher,
                        case_insensitive: false,
                    })],
                },
                None,
            )],
        },
    }
}

/// CSS Selectors §6.3：属性值选择器大小写敏感性由文档语言决定。
/// HTML 不敏感（`[title="es"]` 匹配 `title="ES"`），XML/XHTML 敏感（不匹配）。
/// 验证 WPT attribute-value-selector-007（HTML 不敏感）与 008/009（XHTML 敏感）语义。
#[test]
fn test_matches_attribute_case_sensitivity_html_vs_xml() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    doc.set_attribute(div, "title", "ES");

    // HTML 模式（content_is_xml = false，默认）：大小写不敏感
    assert!(!doc.content_is_xml(), "默认 doc 应为 HTML 模式");
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::Exact("es".to_string()))
        ),
        "HTML 模式：[title=\"es\"] 应匹配 title=\"ES\"（大小写不敏感）"
    );
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::Exact("ES".to_string()))
        ),
        "HTML 模式：[title=\"ES\"] 应匹配 title=\"ES\""
    );

    // XML/XHTML 模式（content_is_xml = true）：大小写敏感
    doc.set_content_is_xml(true);
    assert!(doc.content_is_xml(), "置位后应为 XML 模式");
    assert!(
        !matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::Exact("es".to_string()))
        ),
        "XML 模式：[title=\"es\"] 不应匹配 title=\"ES\"（大小写敏感，WPT 008/009）"
    );
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::Exact("ES".to_string()))
        ),
        "XML 模式：[title=\"ES\"] 应匹配 title=\"ES\""
    );
}

/// Selectors Level 4 `[attr=val i]`：`i` 修饰符强制 ASCII 大小写不敏感，覆盖文档语言默认。
/// 在 XML 模式（默认大小写敏感）下，`i` 应让 `[title="es"]` 匹配 `title="ES"`。
fn attr_selector_ci(name: &str, matcher: AttributeMatcher) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: name.to_string(),
                        matcher,
                        case_insensitive: true,
                    })],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_matches_attribute_case_insensitive_flag_i() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    doc.set_attribute(div, "title", "ES");
    doc.set_content_is_xml(true);

    // XML 模式基线：无修饰符大小写敏感，[title="es"] 不匹配 title="ES"。
    assert!(
        !matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::Exact("es".to_string()))
        ),
        "XML 基线：无修饰符 [title=\"es\"] 不应匹配 title=\"ES\""
    );

    // `i` 修饰符：强制大小写不敏感，覆盖 XML 默认 → 应匹配。
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector_ci("title", AttributeMatcher::Exact("es".to_string()))
        ),
        "Selectors L4：[title=\"es\" i] 应强制大小写不敏感、匹配 title=\"ES\""
    );

    // `i` 对 Includes/Substring 等其他 matcher 同样生效。
    doc.set_attribute(div, "class", "Btn Active");
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector_ci("class", AttributeMatcher::Includes("active".to_string()))
        ),
        "[class~=\"active\" i] 应匹配 class=\"Btn Active\""
    );
}

/// `:checked` 选择器辅助：subclass 为 PseudoClass(Simple("checked"))。
fn checked_selector() -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "checked".to_string(),
                    ))],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_matches_checked_pseudo_class() {
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // <input type="checkbox" checked> → :checked
    let cb_on = doc.create_element("input");
    doc.set_attribute(cb_on, "type", "checkbox");
    doc.set_attribute(cb_on, "checked", "");
    doc.append_child(body, cb_on).unwrap();

    // <input type="checkbox">（无 checked）→ 不匹配
    let cb_off = doc.create_element("input");
    doc.set_attribute(cb_off, "type", "checkbox");
    doc.append_child(body, cb_off).unwrap();

    // <input type="text" checked> → 不匹配（text 输入非 :checked）
    let text_on = doc.create_element("input");
    doc.set_attribute(text_on, "type", "text");
    doc.set_attribute(text_on, "checked", "");
    doc.append_child(body, text_on).unwrap();

    // <option selected> → :checked
    let opt = doc.create_element("option");
    doc.set_attribute(opt, "selected", "");
    doc.append_child(body, opt).unwrap();

    assert!(
        matches_selector(&doc, cb_on, &checked_selector()),
        "input[type=checkbox][checked] 应匹配 :checked"
    );
    assert!(
        !matches_selector(&doc, cb_off, &checked_selector()),
        "无 checked 属性的 checkbox 不应匹配 :checked"
    );
    assert!(
        !matches_selector(&doc, text_on, &checked_selector()),
        "input[type=text] 即使有 checked 也不应匹配 :checked"
    );
    assert!(
        matches_selector(&doc, opt, &checked_selector()),
        "option[selected] 应匹配 :checked"
    );
}

/// 简单伪类选择器辅助：`:<name>`（subclass 为 PseudoClass(Simple(name))）。
fn simple_pseudo(name: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        name.to_string(),
                    ))],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_matches_disabled_enabled_required_optional() {
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // <input disabled>、<input>、<button disabled>、<button>、<div disabled>
    let in_dis = doc.create_element("input");
    doc.set_attribute(in_dis, "disabled", "");
    doc.append_child(body, in_dis).unwrap();
    let in_en = doc.create_element("input");
    doc.append_child(body, in_en).unwrap();
    let btn_dis = doc.create_element("button");
    doc.set_attribute(btn_dis, "disabled", "");
    doc.append_child(body, btn_dis).unwrap();
    let btn_en = doc.create_element("button");
    doc.append_child(body, btn_en).unwrap();
    let div_dis = doc.create_element("div");
    doc.set_attribute(div_dis, "disabled", "");
    doc.append_child(body, div_dis).unwrap();

    // :disabled
    assert!(
        matches_selector(&doc, in_dis, &simple_pseudo("disabled")),
        "input[disabled] 应匹配 :disabled"
    );
    assert!(
        !matches_selector(&doc, in_en, &simple_pseudo("disabled")),
        "无 disabled 的 input 不应匹配 :disabled"
    );
    assert!(
        matches_selector(&doc, btn_dis, &simple_pseudo("disabled")),
        "button[disabled] 应匹配 :disabled"
    );
    assert!(
        !matches_selector(&doc, div_dis, &simple_pseudo("disabled")),
        "div 非可禁用元素，即使有 disabled 也不应匹配 :disabled"
    );

    // :enabled
    assert!(
        matches_selector(&doc, in_en, &simple_pseudo("enabled")),
        "input（无 disabled）应匹配 :enabled"
    );
    assert!(
        matches_selector(&doc, btn_en, &simple_pseudo("enabled")),
        "button（无 disabled）应匹配 :enabled"
    );
    assert!(
        !matches_selector(&doc, in_dis, &simple_pseudo("enabled")),
        "input[disabled] 不应匹配 :enabled"
    );
    assert!(
        !matches_selector(&doc, div_dis, &simple_pseudo("enabled")),
        "div 非可禁用元素，不应匹配 :enabled"
    );

    // :required / :optional
    let in_req = doc.create_element("input");
    doc.set_attribute(in_req, "required", "");
    doc.append_child(body, in_req).unwrap();
    let sel_req = doc.create_element("select");
    doc.set_attribute(sel_req, "required", "");
    doc.append_child(body, sel_req).unwrap();
    let div_req = doc.create_element("div");
    doc.set_attribute(div_req, "required", "");
    doc.append_child(body, div_req).unwrap();

    assert!(
        matches_selector(&doc, in_req, &simple_pseudo("required")),
        "input[required] 应匹配 :required"
    );
    assert!(
        matches_selector(&doc, sel_req, &simple_pseudo("required")),
        "select[required] 应匹配 :required"
    );
    assert!(
        !matches_selector(&doc, in_en, &simple_pseudo("required")),
        "无 required 的 input 不应匹配 :required"
    );
    assert!(
        !matches_selector(&doc, div_req, &simple_pseudo("required")),
        "div 非可约束元素，不应匹配 :required"
    );

    assert!(
        matches_selector(&doc, in_en, &simple_pseudo("optional")),
        "input（无 required）应匹配 :optional"
    );
    assert!(
        !matches_selector(&doc, in_req, &simple_pseudo("optional")),
        "input[required] 不应匹配 :optional"
    );
    assert!(
        !matches_selector(&doc, div_req, &simple_pseudo("optional")),
        "div 非可约束元素，不应匹配 :optional"
    );
}

#[test]
fn test_matches_read_only_read_write() {
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // <input>（默认 text，可编辑）→ :read-write；非 :read-only
    let in_text = doc.create_element("input");
    doc.append_child(body, in_text).unwrap();

    // <input type="password">（可编辑）→ :read-write
    let in_pw = doc.create_element("input");
    doc.set_attribute(in_pw, "type", "password");
    doc.append_child(body, in_pw).unwrap();

    // <input readonly> → :read-only
    let in_ro = doc.create_element("input");
    doc.set_attribute(in_ro, "readonly", "");
    doc.append_child(body, in_ro).unwrap();

    // <input disabled> → :read-only（不可编辑）
    let in_dis = doc.create_element("input");
    doc.set_attribute(in_dis, "disabled", "");
    doc.append_child(body, in_dis).unwrap();

    // <input type="checkbox">（非文本可编辑类型）→ :read-only
    let in_cb = doc.create_element("input");
    doc.set_attribute(in_cb, "type", "checkbox");
    doc.append_child(body, in_cb).unwrap();

    // <textarea>（可编辑）→ :read-write
    let ta = doc.create_element("textarea");
    doc.append_child(body, ta).unwrap();

    // <textarea readonly> → :read-only
    let ta_ro = doc.create_element("textarea");
    doc.set_attribute(ta_ro, "readonly", "");
    doc.append_child(body, ta_ro).unwrap();

    // <p>（非表单控件，默认不可编辑）→ :read-only
    let p = doc.create_element("p");
    doc.append_child(body, p).unwrap();

    // :read-write
    assert!(
        matches_selector(&doc, in_text, &simple_pseudo("read-write")),
        "input（默认 text 可编辑）应匹配 :read-write"
    );
    assert!(
        matches_selector(&doc, in_pw, &simple_pseudo("read-write")),
        "input[type=password] 应匹配 :read-write"
    );
    assert!(
        matches_selector(&doc, ta, &simple_pseudo("read-write")),
        "textarea 应匹配 :read-write"
    );
    assert!(
        !matches_selector(&doc, in_ro, &simple_pseudo("read-write")),
        "input[readonly] 不应匹配 :read-write"
    );
    assert!(
        !matches_selector(&doc, in_dis, &simple_pseudo("read-write")),
        "input[disabled] 不应匹配 :read-write"
    );
    assert!(
        !matches_selector(&doc, in_cb, &simple_pseudo("read-write")),
        "input[type=checkbox] 非可编辑类型，不应匹配 :read-write"
    );
    assert!(
        !matches_selector(&doc, p, &simple_pseudo("read-write")),
        "p 非表单控件，不应匹配 :read-write"
    );

    // :read-only（:read-write 的补集）
    assert!(
        matches_selector(&doc, p, &simple_pseudo("read-only")),
        "p 应匹配 :read-only"
    );
    assert!(
        matches_selector(&doc, in_ro, &simple_pseudo("read-only")),
        "input[readonly] 应匹配 :read-only"
    );
    assert!(
        matches_selector(&doc, in_cb, &simple_pseudo("read-only")),
        "input[type=checkbox] 应匹配 :read-only"
    );
    assert!(
        !matches_selector(&doc, in_text, &simple_pseudo("read-only")),
        "可编辑 input 不应匹配 :read-only"
    );
    assert!(
        !matches_selector(&doc, ta, &simple_pseudo("read-only")),
        "可编辑 textarea 不应匹配 :read-only"
    );
}

/// DashMatch (`|=`) 在 XML 模式下也应大小写敏感（WPT attribute-value-selector-009）。
#[test]
fn test_matches_attribute_dashmatch_case_sensitivity_xml() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    doc.set_attribute(div, "title", "ES");

    // HTML 模式：[title|="es"] 匹配 "ES"（大小写不敏感，"ES" 整体匹配）
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::DashMatch("es".to_string()))
        ),
        "HTML 模式：[title|=\"es\"] 应匹配 title=\"ES\""
    );

    // XML 模式：[title|="es"] 不匹配 "ES"（大小写敏感）
    doc.set_content_is_xml(true);
    assert!(
        !matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::DashMatch("es".to_string()))
        ),
        "XML 模式：[title|=\"es\"] 不应匹配 title=\"ES\"（大小写敏感，WPT 009）"
    );
}

#[test]
fn test_matches_pseudo_first_child() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "first-child".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    // div 是 body 的第一个子元素
    assert!(matches_selector(&doc, div, &sel));
}

#[test]
fn test_matches_pseudo_root() {
    let (doc, html, _body, _div, _p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "root".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    // html 是文档根元素
    assert!(matches_selector(&doc, html, &sel));
}

#[test]
fn test_matches_pseudo_empty() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "empty".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));

    // 添加文本节点后不再是 empty
    let text = doc.create_text_node("hello");
    doc.append_child(div, text).unwrap();
    assert!(!matches_selector(&doc, div, &sel));
}

#[test]
fn test_matches_not_pseudo() {
    let (doc, _html, _body, div, p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        make_id_selector("main"),
                    ]))],
                },
                None,
            )],
        },
    };
    // div#main 不匹配 :not(#main)
    assert!(!matches_selector(&doc, div, &sel));
    // p 匹配 :not(#main)
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_matches_is_pseudo() {
    let (doc, _html, _body, div, p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Is(vec![
                        make_tag_selector("div"),
                        make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));
    assert!(!matches_selector(&doc, p, &sel));
}
