// Test file split from matcher.rs — core selector matching tests
use super::super::*;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, NthPattern,
    PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
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
                        case: AttrCaseModifier::Default,
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
                        case: AttrCaseModifier::Default,
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
                        case: AttrCaseModifier::Default,
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
                        case: AttrCaseModifier::Insensitive,
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

/// Selectors Level 4 `[attr=val s]`：`s` 修饰符强制大小写敏感，覆盖文档语言默认。
/// 在 HTML 模式（默认大小写不敏感）下，`s` 应让 `[title="es"]` 不匹配 `title="ES"`。
fn attr_selector_cs(name: &str, matcher: AttributeMatcher) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: name.to_string(),
                        matcher,
                        case: AttrCaseModifier::Sensitive,
                    })],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_matches_attribute_case_sensitive_flag_s() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    doc.set_attribute(div, "title", "ES");
    // HTML 模式（content_is_xml = false，默认大小写不敏感）。
    assert!(!doc.content_is_xml(), "默认 doc 应为 HTML 模式");

    // HTML 基线：无修饰符大小写不敏感，[title="es"] 匹配 title="ES"。
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector("title", AttributeMatcher::Exact("es".to_string()))
        ),
        "HTML 基线：无修饰符 [title=\"es\"] 应匹配 title=\"ES\"（大小写不敏感）"
    );

    // `s` 修饰符：强制大小写敏感，覆盖 HTML 默认 → 不匹配。
    assert!(
        !matches_selector(
            &doc,
            div,
            &attr_selector_cs("title", AttributeMatcher::Exact("es".to_string()))
        ),
        "Selectors L4：[title=\"es\" s] 应强制大小写敏感、不匹配 title=\"ES\""
    );
    // 大小写一致时仍匹配。
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector_cs("title", AttributeMatcher::Exact("ES".to_string()))
        ),
        "[title=\"ES\" s] 应匹配 title=\"ES\""
    );

    // `s` 对 Includes/DashMatch 等 matcher 同样强制敏感。
    doc.set_attribute(div, "class", "Btn Active");
    assert!(
        !matches_selector(
            &doc,
            div,
            &attr_selector_cs("class", AttributeMatcher::Includes("active".to_string()))
        ),
        "[class~=\"active\" s] 不应匹配 class=\"Btn Active\""
    );
    assert!(
        matches_selector(
            &doc,
            div,
            &attr_selector_cs("class", AttributeMatcher::Includes("Active".to_string()))
        ),
        "[class~=\"Active\" s] 应匹配 class=\"Btn Active\""
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
fn test_matches_disabled_fieldset_select_propagation_r3277() {
    // R3277：HTML spec §4.10.18 禁用传播——CSS :disabled 与 DOM 选择器一致。
    // <fieldset disabled> body 内控件传播禁用，首个 <legend> 内除外；
    // <select disabled> 内 <option> 传播（§4.10.10）；<fieldset disabled> 自身匹配 :disabled。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    let fs = doc.create_element("fieldset");
    doc.set_attribute(fs, "disabled", "");
    doc.append_child(body, fs).unwrap();

    let legend = doc.create_element("legend");
    doc.append_child(fs, legend).unwrap();
    let legend_in = doc.create_element("input");
    doc.append_child(legend, legend_in).unwrap();

    let body_in = doc.create_element("input");
    doc.append_child(fs, body_in).unwrap();

    let sel_dis = doc.create_element("select");
    doc.set_attribute(sel_dis, "disabled", "");
    doc.append_child(body, sel_dis).unwrap();
    let sel_opt = doc.create_element("option");
    doc.append_child(sel_dis, sel_opt).unwrap();

    // fieldset 自身（可禁用元素 + disabled 属性）匹配 :disabled。
    assert!(
        matches_selector(&doc, fs, &simple_pseudo("disabled")),
        "fieldset[disabled] 自身应匹配 :disabled"
    );
    // body 内控件传播禁用。
    assert!(
        matches_selector(&doc, body_in, &simple_pseudo("disabled")),
        "禁用 fieldset body 内 input 应传播匹配 :disabled"
    );
    assert!(
        !matches_selector(&doc, body_in, &simple_pseudo("enabled")),
        "禁用 fieldset body 内 input 不应匹配 :enabled"
    );
    // 首个 legend 内控件豁免。
    assert!(
        !matches_selector(&doc, legend_in, &simple_pseudo("disabled")),
        "首个 legend 内控件应豁免 fieldset disabled 传播"
    );
    assert!(
        matches_selector(&doc, legend_in, &simple_pseudo("enabled")),
        "首个 legend 内控件应匹配 :enabled"
    );
    // select disabled → option 传播。
    assert!(
        matches_selector(&doc, sel_opt, &simple_pseudo("disabled")),
        "禁用 select 内 option 应传播匹配 :disabled"
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

#[test]
fn test_matches_read_write_fieldset_disabled_propagation_r3278() {
    // R3278：`:read-write` 经 Document::is_effectively_read_write——含 `<fieldset disabled>`
    // 祖先传播禁用判定（与 DOM 选择器同源）。fieldset 内文本 input 自身无 disabled，
    // 但经传播禁用 → 只读；首个 legend 内豁免。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    let fs = doc.create_element("fieldset");
    doc.set_attribute(fs, "disabled", "");
    doc.append_child(body, fs).unwrap();

    let legend = doc.create_element("legend");
    doc.append_child(fs, legend).unwrap();
    let legend_in = doc.create_element("input");
    doc.append_child(legend, legend_in).unwrap();

    let fs_in = doc.create_element("input");
    doc.append_child(fs, fs_in).unwrap();

    // fieldset body 内 input 经传播禁用 → 只读。
    assert!(
        !matches_selector(&doc, fs_in, &simple_pseudo("read-write")),
        "禁用 fieldset body 内 input 经传播禁用应为只读"
    );
    assert!(
        matches_selector(&doc, fs_in, &simple_pseudo("read-only")),
        "禁用 fieldset body 内 input 经传播禁用应匹配 :read-only"
    );
    // 首个 legend 内 input 豁免（未禁用）→ 可编辑。
    assert!(
        matches_selector(&doc, legend_in, &simple_pseudo("read-write")),
        "首个 legend 内 input 豁免传播，应匹配 :read-write"
    );
    assert!(
        !matches_selector(&doc, legend_in, &simple_pseudo("read-only")),
        "首个 legend 内 input 豁免传播，不应匹配 :read-only"
    );
}

#[test]
fn test_matches_placeholder_shown() {
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // <input placeholder="x">（无 value）→ 显示 placeholder
    let in_ph = doc.create_element("input");
    doc.set_attribute(in_ph, "placeholder", "name");
    doc.append_child(body, in_ph).unwrap();

    // <input placeholder="x" value="">（空 value）→ 仍显示 placeholder
    let in_empty = doc.create_element("input");
    doc.set_attribute(in_empty, "placeholder", "name");
    doc.set_attribute(in_empty, "value", "");
    doc.append_child(body, in_empty).unwrap();

    // <input placeholder="x" value="hi">（有值）→ 不显示
    let in_val = doc.create_element("input");
    doc.set_attribute(in_val, "placeholder", "name");
    doc.set_attribute(in_val, "value", "hi");
    doc.append_child(body, in_val).unwrap();

    // <input value="hi">（无 placeholder）→ 不匹配
    let in_no_ph = doc.create_element("input");
    doc.set_attribute(in_no_ph, "value", "hi");
    doc.append_child(body, in_no_ph).unwrap();

    // <textarea placeholder="x"></textarea>（空内容）→ 显示
    let ta_empty = doc.create_element("textarea");
    doc.set_attribute(ta_empty, "placeholder", "name");
    doc.append_child(body, ta_empty).unwrap();

    // <textarea placeholder="x">text</textarea>（有内容）→ 不显示
    let ta_val = doc.create_element("textarea");
    doc.set_attribute(ta_val, "placeholder", "name");
    let text = doc.create_text_node("text");
    doc.append_child(ta_val, text).unwrap();
    doc.append_child(body, ta_val).unwrap();

    // <p placeholder="x">（非 input/textarea）→ 不匹配
    let p_ph = doc.create_element("p");
    doc.set_attribute(p_ph, "placeholder", "x");
    doc.append_child(body, p_ph).unwrap();

    let sel = simple_pseudo("placeholder-shown");
    assert!(
        matches_selector(&doc, in_ph, &sel),
        "input[placeholder] 无 value 应匹配 :placeholder-shown"
    );
    assert!(
        matches_selector(&doc, in_empty, &sel),
        "input[placeholder][value=\"\"] 应匹配 :placeholder-shown"
    );
    assert!(
        !matches_selector(&doc, in_val, &sel),
        "input[placeholder][value=\"hi\"] 不应匹配 :placeholder-shown"
    );
    assert!(
        !matches_selector(&doc, in_no_ph, &sel),
        "无 placeholder 的 input 不应匹配 :placeholder-shown"
    );
    assert!(
        matches_selector(&doc, ta_empty, &sel),
        "textarea[placeholder] 空内容应匹配 :placeholder-shown"
    );
    assert!(
        !matches_selector(&doc, ta_val, &sel),
        "textarea[placeholder] 有内容不应匹配 :placeholder-shown"
    );
    assert!(
        !matches_selector(&doc, p_ph, &sel),
        "p 非输入元素，不应匹配 :placeholder-shown"
    );
}

#[test]
fn test_matches_default_pseudo_class() {
    // :default = option[selected] + 默认选中的 checkbox/radio（checked 内容属性）
    //          + 表单内树序首个 submit 按钮（button[type=submit|缺省]、input[type=submit|image]）。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // 表单 1：两个 submit，树序首个 input[type=submit] 匹配，第二个 button 不匹配。
    let form1 = doc.create_element("form");
    doc.append_child(body, form1).unwrap();
    let sub1 = doc.create_element("input");
    doc.set_attribute(sub1, "type", "submit");
    doc.append_child(form1, sub1).unwrap();
    let sub2 = doc.create_element("button");
    doc.set_attribute(sub2, "type", "submit");
    doc.append_child(form1, sub2).unwrap();

    // 表单 2：reset 非 submit 候选不匹配；无 type 的 button（默认 submit）为首个 → 匹配。
    let form2 = doc.create_element("form");
    doc.append_child(body, form2).unwrap();
    let reset_btn = doc.create_element("button");
    doc.set_attribute(reset_btn, "type", "reset");
    doc.append_child(form2, reset_btn).unwrap();
    let btn_default = doc.create_element("button");
    doc.append_child(form2, btn_default).unwrap();

    // option[selected] → :default；无 selected → 不匹配。
    let opt_sel = doc.create_element("option");
    doc.set_attribute(opt_sel, "selected", "");
    doc.append_child(body, opt_sel).unwrap();
    let opt_plain = doc.create_element("option");
    doc.append_child(body, opt_plain).unwrap();

    // 默认选中的 checkbox/radio（checked 内容属性）→ :default。
    let cb = doc.create_element("input");
    doc.set_attribute(cb, "type", "checkbox");
    doc.set_attribute(cb, "checked", "");
    doc.append_child(body, cb).unwrap();
    let rd = doc.create_element("input");
    doc.set_attribute(rd, "type", "radio");
    doc.set_attribute(rd, "checked", "");
    doc.append_child(body, rd).unwrap();

    // 无 form 宿主的 submit：非任何 form 的默认按钮 → 不匹配。
    let img = doc.create_element("input");
    doc.set_attribute(img, "type", "image");
    doc.append_child(body, img).unwrap();
    // 非 submit 的 input：text 不匹配。
    let txt = doc.create_element("input");
    doc.set_attribute(txt, "type", "text");
    doc.append_child(body, txt).unwrap();

    let sel = simple_pseudo("default");
    assert!(
        matches_selector(&doc, sub1, &sel),
        "form 内树序首个 submit 应匹配 :default"
    );
    assert!(
        !matches_selector(&doc, sub2, &sel),
        "form 内非首个 submit 不应匹配 :default"
    );
    assert!(
        matches_selector(&doc, btn_default, &sel),
        "无 type 的 button（默认 submit）作为首个应匹配 :default"
    );
    assert!(
        !matches_selector(&doc, reset_btn, &sel),
        "button[type=reset] 非 submit 候选，不应匹配 :default"
    );
    assert!(
        matches_selector(&doc, opt_sel, &sel),
        "option[selected] 应匹配 :default"
    );
    assert!(
        !matches_selector(&doc, opt_plain, &sel),
        "无 selected 的 option 不应匹配 :default"
    );
    assert!(
        matches_selector(&doc, cb, &sel),
        "input[type=checkbox][checked] 默认选中应匹配 :default"
    );
    assert!(
        matches_selector(&doc, rd, &sel),
        "input[type=radio][checked] 默认选中应匹配 :default"
    );
    assert!(
        !matches_selector(&doc, img, &sel),
        "无 form 宿主的 submit 不应匹配 :default"
    );
    assert!(
        !matches_selector(&doc, txt, &sel),
        "input[type=text] 非 submit/checkbox/radio，不应匹配 :default"
    );
}

#[test]
fn test_matches_indeterminate_pseudo_class() {
    // :indeterminate = <progress> 无 value 属性 + radio 组（同 name + 同 form 宿主）无任何 checked。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // <progress> 无 value → indeterminate；有 value → 非 indeterminate。
    let prog_ind = doc.create_element("progress");
    doc.append_child(body, prog_ind).unwrap();
    let prog_det = doc.create_element("progress");
    doc.set_attribute(prog_det, "value", "50");
    doc.append_child(body, prog_det).unwrap();

    // radio 组 g（同 form + 同 name）无 checked → 全部 indeterminate。
    let form = doc.create_element("form");
    doc.append_child(body, form).unwrap();
    let r1 = doc.create_element("input");
    doc.set_attribute(r1, "type", "radio");
    doc.set_attribute(r1, "name", "g");
    doc.append_child(form, r1).unwrap();
    let r2 = doc.create_element("input");
    doc.set_attribute(r2, "type", "radio");
    doc.set_attribute(r2, "name", "g");
    doc.append_child(form, r2).unwrap();

    // radio 组 h 有 checked 成员 → 组内均非 indeterminate。
    let r3 = doc.create_element("input");
    doc.set_attribute(r3, "type", "radio");
    doc.set_attribute(r3, "name", "h");
    doc.set_attribute(r3, "checked", "");
    doc.append_child(form, r3).unwrap();
    let r4 = doc.create_element("input");
    doc.set_attribute(r4, "type", "radio");
    doc.set_attribute(r4, "name", "h");
    doc.append_child(form, r4).unwrap();

    // checkbox 不匹配（indeterminate 为动态 IDL 状态，静态不可知）。
    let cb = doc.create_element("input");
    doc.set_attribute(cb, "type", "checkbox");
    doc.append_child(body, cb).unwrap();

    let sel = simple_pseudo("indeterminate");
    assert!(
        matches_selector(&doc, prog_ind, &sel),
        "无 value 的 progress 应匹配 :indeterminate"
    );
    assert!(
        !matches_selector(&doc, prog_det, &sel),
        "有 value 的 progress 不应匹配 :indeterminate"
    );
    assert!(
        matches_selector(&doc, r1, &sel),
        "radio 组（name=g）无 checked，成员应匹配 :indeterminate"
    );
    assert!(
        matches_selector(&doc, r2, &sel),
        "radio 组（name=g）无 checked，成员应匹配 :indeterminate"
    );
    assert!(
        !matches_selector(&doc, r3, &sel),
        "radio 组（name=h）有 checked，checked 成员不应匹配 :indeterminate"
    );
    assert!(
        !matches_selector(&doc, r4, &sel),
        "radio 组（name=h）有 checked，未选成员也不应匹配 :indeterminate"
    );
    assert!(
        !matches_selector(&doc, cb, &sel),
        "checkbox 静态下不匹配 :indeterminate"
    );
}

/// `:dir(<dir>)` 选择器辅助：subclass 为 PseudoClass(Dir(dir))。
fn dir_selector(dir: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Dir(dir.to_string()))],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_matches_dir_pseudo_class() {
    // :dir(ltr|rtl) 按元素方向性匹配：显式 dir、祖先继承、缺省默认 LTR、dir=auto 按内容。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    // 显式 dir="rtl" / dir="ltr"。
    let rtl_el = doc.create_element("div");
    doc.set_attribute(rtl_el, "dir", "rtl");
    doc.append_child(body, rtl_el).unwrap();
    let ltr_el = doc.create_element("div");
    doc.set_attribute(ltr_el, "dir", "ltr");
    doc.append_child(body, ltr_el).unwrap();

    // 继承：section[dir=rtl] 内的 p。
    let sec = doc.create_element("section");
    doc.set_attribute(sec, "dir", "rtl");
    doc.append_child(body, sec).unwrap();
    let p_in_rtl = doc.create_element("p");
    doc.append_child(sec, p_in_rtl).unwrap();

    // 无 dir（默认 LTR）。
    let no_dir = doc.create_element("span");
    doc.append_child(body, no_dir).unwrap();

    // dir="auto" + 阿拉伯文 → RTL。
    let auto_rtl = doc.create_element("div");
    doc.set_attribute(auto_rtl, "dir", "auto");
    let ar_text = doc.create_text_node("مرحبا");
    doc.append_child(auto_rtl, ar_text).unwrap();
    doc.append_child(body, auto_rtl).unwrap();

    // dir="auto" + 拉丁文 → LTR。
    let auto_ltr = doc.create_element("div");
    doc.set_attribute(auto_ltr, "dir", "auto");
    let en_text = doc.create_text_node("Hello");
    doc.append_child(auto_ltr, en_text).unwrap();
    doc.append_child(body, auto_ltr).unwrap();

    let sel_ltr = dir_selector("ltr");
    let sel_rtl = dir_selector("rtl");
    assert!(matches_selector(&doc, rtl_el, &sel_rtl), "dir=rtl 应匹配 :dir(rtl)");
    assert!(!matches_selector(&doc, rtl_el, &sel_ltr), "dir=rtl 不应匹配 :dir(ltr)");
    assert!(matches_selector(&doc, ltr_el, &sel_ltr), "dir=ltr 应匹配 :dir(ltr)");
    assert!(!matches_selector(&doc, ltr_el, &sel_rtl), "dir=ltr 不应匹配 :dir(rtl)");
    assert!(
        matches_selector(&doc, p_in_rtl, &sel_rtl),
        "继承 section[dir=rtl] 的 p 应匹配 :dir(rtl)"
    );
    assert!(
        !matches_selector(&doc, p_in_rtl, &sel_ltr),
        "继承 RTL 的 p 不应匹配 :dir(ltr)"
    );
    assert!(
        matches_selector(&doc, no_dir, &sel_ltr),
        "无 dir 默认 LTR 应匹配 :dir(ltr)"
    );
    assert!(
        !matches_selector(&doc, no_dir, &sel_rtl),
        "无 dir 默认 LTR 不应匹配 :dir(rtl)"
    );
    assert!(
        matches_selector(&doc, auto_rtl, &sel_rtl),
        "dir=auto + 阿拉伯文应匹配 :dir(rtl)"
    );
    assert!(
        matches_selector(&doc, auto_ltr, &sel_ltr),
        "dir=auto + 拉丁文应匹配 :dir(ltr)"
    );
}

/// `:nth-child(an+b of S)` 选择器辅助。
fn nth_child_of_selector(a: i32, b: i32, of: Vec<Selector>, last: bool) -> Selector {
    let pseudo = if last {
        PseudoClassSelector::NthLastChildOf(NthPattern { a, b }, of)
    } else {
        PseudoClassSelector::NthChildOf(NthPattern { a, b }, of)
    };
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(pseudo)],
                },
                None,
            )],
        },
    }
}

#[test]
fn test_matches_nth_child_of_selector() {
    // :nth-child(an+b of S) 仅在匹配 S 的兄弟中计数。
    // 父代子序：div.a, p, div.a, span —— div 兄弟位置 1/2。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    let parent = doc.create_element("div");
    doc.append_child(body, parent).unwrap();
    let div1 = doc.create_element("div");
    doc.set_attribute(div1, "class", "a");
    doc.append_child(parent, div1).unwrap();
    let mid_p = doc.create_element("p");
    doc.append_child(parent, mid_p).unwrap();
    let div2 = doc.create_element("div");
    doc.set_attribute(div2, "class", "a");
    doc.append_child(parent, div2).unwrap();
    let span = doc.create_element("span");
    doc.append_child(parent, span).unwrap();

    let of_div = vec![make_tag_selector("div")];
    let first_of_div = nth_child_of_selector(0, 1, of_div.clone(), false);
    let second_of_div = nth_child_of_selector(0, 2, of_div.clone(), false);
    let last_of_div = nth_child_of_selector(0, 1, of_div.clone(), true);
    let second_last_of_div = nth_child_of_selector(0, 2, of_div.clone(), true);
    let even_of_div = nth_child_of_selector(2, 0, of_div.clone(), false);

    assert!(
        matches_selector(&doc, div1, &first_of_div),
        "div1 是首个 div 兄弟，应匹配 :nth-child(1 of div)"
    );
    assert!(
        !matches_selector(&doc, div2, &first_of_div),
        "div2 非首个 div 兄弟，不应匹配 :nth-child(1 of div)"
    );
    assert!(
        matches_selector(&doc, div2, &second_of_div),
        "div2 是第 2 个 div 兄弟，应匹配 :nth-child(2 of div)"
    );
    assert!(
        !matches_selector(&doc, mid_p, &first_of_div),
        "p 不匹配 of 选择器(div)，不应匹配"
    );
    assert!(
        matches_selector(&doc, div2, &last_of_div),
        "div2 是最后一个 div，应匹配 :nth-last-child(1 of div)"
    );
    assert!(
        matches_selector(&doc, div1, &second_last_of_div),
        "div1 是倒数第 2 个 div，应匹配 :nth-last-child(2 of div)"
    );
    assert!(
        matches_selector(&doc, div2, &even_of_div),
        "div2 在 div 兄弟中位置 2，应匹配 :nth-child(2n of div)"
    );
    assert!(
        !matches_selector(&doc, div1, &even_of_div),
        "div1 在 div 兄弟中位置 1，不应匹配 :nth-child(2n of div)"
    );
}

#[test]
fn test_matches_any_link_and_link_pseudo_class() {
    // :any-link / :link 匹配 a/area/link 带 href；静态下 :link 等价 :any-link。
    let (mut doc, _html, body, _div, _p) = make_test_dom();

    let a_href = doc.create_element("a");
    doc.set_attribute(a_href, "href", "/x");
    doc.append_child(body, a_href).unwrap();
    let a_nohref = doc.create_element("a");
    doc.append_child(body, a_nohref).unwrap();
    let area = doc.create_element("area");
    doc.set_attribute(area, "href", "/y");
    doc.append_child(body, area).unwrap();
    let link = doc.create_element("link");
    doc.set_attribute(link, "href", "/z");
    doc.append_child(body, link).unwrap();
    let para = doc.create_element("p");
    doc.append_child(body, para).unwrap();

    let any_link = simple_pseudo("any-link");
    let link_sel = simple_pseudo("link");
    assert!(matches_selector(&doc, a_href, &any_link), "a[href] 应匹配 :any-link");
    assert!(
        matches_selector(&doc, a_href, &link_sel),
        "a[href] 应匹配 :link（静态等价 :any-link）"
    );
    assert!(
        !matches_selector(&doc, a_nohref, &any_link),
        "无 href 的 a 不应匹配 :any-link"
    );
    assert!(matches_selector(&doc, area, &any_link), "area[href] 应匹配 :any-link");
    assert!(matches_selector(&doc, link, &any_link), "link[href] 应匹配 :any-link");
    assert!(
        !matches_selector(&doc, para, &any_link),
        "p 非链接元素，不应匹配 :any-link"
    );
}

#[test]
fn test_matches_scope_pseudo_class() {
    // :scope 在文档样式表中等价 :root（匹配文档根元素 html）。
    let (doc, html, _body, div, p) = make_test_dom();
    let sel = simple_pseudo("scope");
    assert!(matches_selector(&doc, html, &sel), ":scope 应匹配文档根元素 html");
    assert!(!matches_selector(&doc, div, &sel), "div 非文档根，不应匹配 :scope");
    assert!(!matches_selector(&doc, p, &sel), "p 非文档根，不应匹配 :scope");
}

#[test]
fn test_matches_target_pseudo_class_r3283() {
    // R3283：:target（CSS Selectors L3 §6.6.2）——当前文档 URL fragment 指向的唯一元素。
    // 此前 CSS 解析器识别但 style-system matcher 走 `_ => false` → CSS `:target` 恒不匹配，
    // 与 DOM querySelector 不一致。补全为委派 Document::is_target_element（dom/document/target.rs）。
    // make_test_dom：div id="main"（div 变量），p 无 id。
    let (mut doc, _html, _body, div, p) = make_test_dom();
    let sel = simple_pseudo("target");

    // 无 URL → 无 :target。
    assert!(!matches_selector(&doc, div, &sel), "无 URL 时 :target 不应匹配任何元素");

    // URL 无 fragment → 无 :target。
    doc.set_url(Some("https://example.com/page".to_string()));
    assert!(
        !matches_selector(&doc, div, &sel),
        "URL 无 fragment 时 :target 不应匹配"
    );

    // URL fragment=#main → div（id=main）成为 :target；p（无 id）不匹配。
    doc.set_url(Some("https://example.com/page#main".to_string()));
    assert!(
        matches_selector(&doc, div, &sel),
        "#main fragment 应使 id=main 的 div 匹配 :target"
    );
    assert!(!matches_selector(&doc, p, &sel), "无 id 的 p 不应匹配 :target");

    // fragment 指向不存在的 id → 无 :target。
    doc.set_url(Some("https://example.com/page#missing".to_string()));
    assert!(!matches_selector(&doc, div, &sel), "不存在的 fragment 不应匹配 :target");

    // 百分号编码 fragment：#m%61%69n（%6D=... 实测 #main 的 'a'=%61）解码为 main 命中。
    doc.set_url(Some("https://example.com/page#m%61in".to_string()));
    assert!(
        matches_selector(&doc, div, &sel),
        "百分号编码 #m%61in 解码为 main 应使 div 匹配 :target"
    );
}

#[test]
fn test_matches_validation_pseudo_classes_r3284() {
    // R3284：:valid/:invalid/:in-range/:out-of-range（HTML §4.10.20 + CSS Selectors L4）。
    // 此前 CSS 解析器识别但 style-system matcher 走 `_ => false` → CSS 这四个伪类恒不匹配，
    // 与 DOM querySelector 不一致。补全为委派 Document 权威方法（dom/document/validation.rs）。
    let mut doc = zero_dom::Document::new();
    let root = doc.root();

    // required 空 input → :invalid（valueMissing）。
    let req_empty = doc.create_element("input");
    doc.set_attribute(req_empty, "id", "req-empty");
    doc.set_attribute(req_empty, "required", "");
    doc.append_child(root, req_empty).unwrap();

    // required 已填值 input → :valid。
    let req_filled = doc.create_element("input");
    doc.set_attribute(req_filled, "id", "req-filled");
    doc.set_attribute(req_filled, "required", "");
    doc.set_attribute(req_filled, "value", "x");
    doc.append_child(root, req_filled).unwrap();

    // number input 在范围内 → :valid + :in-range。
    let num_in = doc.create_element("input");
    doc.set_attribute(num_in, "id", "num-in");
    doc.set_attribute(num_in, "type", "number");
    doc.set_attribute(num_in, "min", "1");
    doc.set_attribute(num_in, "max", "10");
    doc.set_attribute(num_in, "value", "5");
    doc.append_child(root, num_in).unwrap();

    // number input 越下界 → :invalid + :out-of-range。
    let num_lo = doc.create_element("input");
    doc.set_attribute(num_lo, "id", "num-lo");
    doc.set_attribute(num_lo, "type", "number");
    doc.set_attribute(num_lo, "min", "1");
    doc.set_attribute(num_lo, "max", "10");
    doc.set_attribute(num_lo, "value", "0");
    doc.append_child(root, num_lo).unwrap();

    // number input 越上界 → :invalid + :out-of-range。
    let num_hi = doc.create_element("input");
    doc.set_attribute(num_hi, "id", "num-hi");
    doc.set_attribute(num_hi, "type", "number");
    doc.set_attribute(num_hi, "min", "1");
    doc.set_attribute(num_hi, "max", "10");
    doc.set_attribute(num_hi, "value", "11");
    doc.append_child(root, num_hi).unwrap();

    // 无约束无边界 input → :valid，非 :in-range。
    let opt_empty = doc.create_element("input");
    doc.set_attribute(opt_empty, "id", "opt-empty");
    doc.append_child(root, opt_empty).unwrap();

    // disabled required input → barred（既不 :valid 也不 :invalid）。
    let disabled_req = doc.create_element("input");
    doc.set_attribute(disabled_req, "id", "disabled-req");
    doc.set_attribute(disabled_req, "required", "");
    doc.set_attribute(disabled_req, "disabled", "");
    doc.append_child(root, disabled_req).unwrap();

    // 非表单控件 p → barred。
    let para = doc.create_element("p");
    doc.set_attribute(para, "id", "para");
    doc.append_child(root, para).unwrap();

    let sel = |name: &str| simple_pseudo(name);

    // :invalid。
    assert!(
        matches_selector(&doc, req_empty, &sel("invalid")),
        "required 空应匹配 :invalid"
    );
    assert!(
        matches_selector(&doc, num_lo, &sel("invalid")),
        "越下界 number 应匹配 :invalid"
    );
    assert!(
        matches_selector(&doc, num_hi, &sel("invalid")),
        "越上界 number 应匹配 :invalid"
    );
    assert!(
        !matches_selector(&doc, req_filled, &sel("invalid")),
        "required 已填值不应匹配 :invalid"
    );
    assert!(
        !matches_selector(&doc, disabled_req, &sel("invalid")),
        "disabled 不应匹配 :invalid（barred）"
    );
    assert!(
        !matches_selector(&doc, para, &sel("invalid")),
        "p 不应匹配 :invalid（barred）"
    );

    // :valid。
    assert!(
        matches_selector(&doc, req_filled, &sel("valid")),
        "required 已填值应匹配 :valid"
    );
    assert!(
        matches_selector(&doc, num_in, &sel("valid")),
        "在范围内 number 应匹配 :valid"
    );
    assert!(
        matches_selector(&doc, opt_empty, &sel("valid")),
        "无约束 input 应匹配 :valid"
    );
    assert!(
        !matches_selector(&doc, req_empty, &sel("valid")),
        "required 空不应匹配 :valid"
    );
    assert!(
        !matches_selector(&doc, para, &sel("valid")),
        "p 不应匹配 :valid（barred）"
    );

    // :in-range。
    assert!(
        matches_selector(&doc, num_in, &sel("in-range")),
        "区间内 number 应匹配 :in-range"
    );
    assert!(
        !matches_selector(&doc, num_lo, &sel("in-range")),
        "越界 number 不应匹配 :in-range"
    );
    assert!(
        !matches_selector(&doc, opt_empty, &sel("in-range")),
        "无边界 number 不应匹配 :in-range"
    );

    // :out-of-range。
    assert!(
        matches_selector(&doc, num_lo, &sel("out-of-range")),
        "越下界应匹配 :out-of-range"
    );
    assert!(
        matches_selector(&doc, num_hi, &sel("out-of-range")),
        "越上界应匹配 :out-of-range"
    );
    assert!(
        !matches_selector(&doc, num_in, &sel("out-of-range")),
        "区间内 number 不应匹配 :out-of-range"
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

#[test]
fn test_matches_defined_pseudo_class_r3299() {
    // R3299：:defined（HTML §3.1.3 + CSS Selectors §10）——内置元素或已升级 custom element 匹配；
    // 未升级（合法 CE 名）不匹配。此前 CSS 解析器识别但 style-system matcher 走 `_ => false` →
    // CSS `:defined` 恒不匹配，与 DOM querySelector 不一致。补全为复用 dom `is_valid_custom_element_name`
    // 静态近似（合法 CE 名 → 未升级 → 不匹配），与 DOM 选择器同源。
    let mut doc = zero_dom::Document::new();
    let root = doc.root();

    // 内置 div（无连字符）→ :defined。
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    // 未升级 custom element（合法 CE 名 my-widget）→ 非 :defined。
    let widget = doc.create_element("my-widget");
    doc.append_child(root, widget).unwrap();

    // 合法 CE 名（多连字符）x-foo-bar → 非 :defined。
    let nested = doc.create_element("x-foo-bar");
    doc.append_child(root, nested).unwrap();

    // 无连字符（mywidget）→ 视内置/未知 → :defined。
    let builtin_like = doc.create_element("mywidget");
    doc.append_child(root, builtin_like).unwrap();

    let sel = simple_pseudo("defined");
    assert!(matches_selector(&doc, div, &sel), "内置 div 应匹配 :defined");
    assert!(
        !matches_selector(&doc, widget, &sel),
        "未升级 custom element（my-widget）不应匹配 :defined"
    );
    assert!(
        !matches_selector(&doc, nested, &sel),
        "未升级 custom element（x-foo-bar）不应匹配 :defined"
    );
    assert!(
        matches_selector(&doc, builtin_like, &sel),
        "无连字符 tag（mywidget）应匹配 :defined（非合法 CE 名）"
    );
}

#[test]
fn test_matches_blank_pseudo_class_r3300() {
    // R3300：:blank（CSS UI L4 / Selectors L4 §12）——值空或纯空白的文本输入控件。
    // 此前 CSS 解析器识别但 style-system matcher 走 `_ => false` → CSS `:blank` 恒不匹配，
    // 与 DOM querySelector 不一致。补全为委派 Document::is_blank_element（与 :placeholder-shown
    // 空值检测同源，但不要求 placeholder 属性）。
    let mut doc = zero_dom::Document::new();
    let root = doc.root();

    // 空 input（无 value）→ :blank。
    let empty_in = doc.create_element("input");
    doc.append_child(root, empty_in).unwrap();

    // 有值 input → 非 :blank。
    let filled_in = doc.create_element("input");
    doc.set_attribute(filled_in, "value", "text");
    doc.append_child(root, filled_in).unwrap();

    // 空 textarea → :blank。
    let empty_ta = doc.create_element("textarea");
    doc.append_child(root, empty_ta).unwrap();

    // 纯空白 textarea → :blank。
    let ws_ta = doc.create_element("textarea");
    let ws_text = doc.create_text_node("   \n\t  ");
    doc.append_child(ws_ta, ws_text).unwrap();
    doc.append_child(root, ws_ta).unwrap();

    // 有内容 textarea → 非 :blank。
    let filled_ta = doc.create_element("textarea");
    let content = doc.create_text_node("content");
    doc.append_child(filled_ta, content).unwrap();
    doc.append_child(root, filled_ta).unwrap();

    // 非表单元素 div → 非 :blank。
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    let sel = simple_pseudo("blank");
    assert!(matches_selector(&doc, empty_in, &sel), "空 input 应匹配 :blank");
    assert!(!matches_selector(&doc, filled_in, &sel), "有值 input 不应匹配 :blank");
    assert!(matches_selector(&doc, empty_ta, &sel), "空 textarea 应匹配 :blank");
    assert!(matches_selector(&doc, ws_ta, &sel), "纯空白 textarea 应匹配 :blank");
    assert!(
        !matches_selector(&doc, filled_ta, &sel),
        "有内容 textarea 不应匹配 :blank"
    );
    assert!(!matches_selector(&doc, div, &sel), "非表单元素 div 不应匹配 :blank");
}
