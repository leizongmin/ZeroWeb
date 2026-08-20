//! query.rs 覆盖率补充测试：parse_simple_selector 和通过 Document API 测试匹配。

use crate::*;

// ═══════════════════════════════════════════════════════════════════════
// parse_simple_selector — 错误和边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_empty_id_selector() {
    assert!(parse_simple_selector("div#").is_none());
}

#[test]
fn test_parse_empty_class_selector() {
    assert!(parse_simple_selector("div.").is_none());
}

#[test]
fn test_parse_unclosed_bracket() {
    assert!(parse_simple_selector("[attr").is_none());
}

#[test]
fn test_parse_unrecognized_char_after_attribute() {
    // 属性选择器后跟普通文本 → 无法识别的选择器部分 → 返回 None
    assert!(parse_simple_selector("div[attr]unknown").is_none());
}

#[test]
fn test_parse_attribute_tilde_equals() {
    let sel = parse_simple_selector("[data-x~=foo]").unwrap();
    let attr = sel.attribute.unwrap();
    assert_eq!(attr.name, "data-x");
    assert!(matches!(attr.matcher, AttributeMatcher::Includes(v) if v == "foo"));
}

#[test]
fn test_parse_attribute_with_spaces() {
    let sel = parse_simple_selector("[ data-x = bar ]").unwrap();
    let attr = sel.attribute.unwrap();
    assert_eq!(attr.name, "data-x");
    assert!(matches!(attr.matcher, AttributeMatcher::Exact(v) if v == "bar"));
}

#[test]
fn test_parse_attribute_css3_operators() {
    // 四个 CSS3 子串/连字符运算符。
    let mk = |sel: &str| parse_simple_selector(sel).unwrap().attribute.unwrap();
    assert!(matches!(
        mk("[href^=https]").matcher,
        AttributeMatcher::Prefix(v) if v == "https"
    ));
    assert!(matches!(
        mk("[href$=pdf]").matcher,
        AttributeMatcher::Suffix(v) if v == "pdf"
    ));
    assert!(matches!(
        mk("[class*=icon]").matcher,
        AttributeMatcher::Substring(v) if v == "icon"
    ));
    assert!(matches!(
        mk("[lang|=en]").matcher,
        AttributeMatcher::DashMatch(v) if v == "en"
    ));
    // 带引号值应去引号（双引号 / 单引号）。
    assert!(matches!(
        mk("[href^=\"https\"]").matcher,
        AttributeMatcher::Prefix(v) if v == "https"
    ));
    assert!(matches!(
        mk("[data-x^='x-']").matcher,
        AttributeMatcher::Prefix(v) if v == "x-"
    ));
    // 两字符运算符须先于单字符 `=`：`^=` 不能被拆成 Exact(name 含 `^`)。
    let a = mk("[a^=b]");
    assert_eq!(a.name, "a");
    assert!(matches!(a.matcher, AttributeMatcher::Prefix(v) if v == "b"));
}

#[test]
fn test_parse_combined_with_attribute() {
    let sel = parse_simple_selector("div#id.cls[data-x=val]").unwrap();
    assert_eq!(sel.tag.as_deref(), Some("div"));
    assert_eq!(sel.id.as_deref(), Some("id"));
    assert_eq!(sel.classes, vec!["cls"]);
    let attr = sel.attribute.unwrap();
    assert_eq!(attr.name, "data-x");
}

#[test]
fn test_parse_tag_only() {
    let sel = parse_simple_selector("span").unwrap();
    assert_eq!(sel.tag.as_deref(), Some("span"));
    assert!(sel.id.is_none());
    assert!(sel.classes.is_empty());
    assert!(sel.attribute.is_none());
}

#[test]
fn test_parse_id_then_class_then_attribute() {
    let sel = parse_simple_selector("#myid.myclass[data-test]").unwrap();
    assert!(sel.tag.is_none());
    assert_eq!(sel.id.as_deref(), Some("myid"));
    assert_eq!(sel.classes, vec!["myclass"]);
    assert!(sel.attribute.is_some());
}

#[test]
fn test_parse_tag_then_class() {
    let sel = parse_simple_selector("div.myclass").unwrap();
    assert_eq!(sel.tag.as_deref(), Some("div"));
    assert_eq!(sel.classes, vec!["myclass"]);
}

#[test]
fn test_parse_tag_then_id() {
    let sel = parse_simple_selector("div#myid").unwrap();
    assert_eq!(sel.tag.as_deref(), Some("div"));
    assert_eq!(sel.id.as_deref(), Some("myid"));
}

#[test]
fn test_parse_class_then_id() {
    let sel = parse_simple_selector(".cls#id").unwrap();
    assert!(sel.tag.is_none());
    assert_eq!(sel.classes, vec!["cls"]);
    assert_eq!(sel.id.as_deref(), Some("id"));
}

#[test]
fn test_parse_class_then_attribute() {
    let sel = parse_simple_selector(".cls[data-x]").unwrap();
    assert!(sel.tag.is_none());
    assert_eq!(sel.classes, vec!["cls"]);
    assert!(sel.attribute.is_some());
}

#[test]
fn test_parse_id_then_attribute() {
    let sel = parse_simple_selector("#id[data-x=val]").unwrap();
    assert!(sel.tag.is_none());
    assert_eq!(sel.id.as_deref(), Some("id"));
    let attr = sel.attribute.unwrap();
    assert!(matches!(attr.matcher, AttributeMatcher::Exact(v) if v == "val"));
}

// ═══════════════════════════════════════════════════════════════════════
// SimpleSelector::matches — 通过 Document.query_selector 间接测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_query_selector_tag_match() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(doc.root(), div).unwrap();
    doc.append_child(doc.root(), span).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "div"), Some(div));
    assert_eq!(doc.query_selector(doc.root(), "span"), Some(span));
}

#[test]
fn test_query_selector_tag_case_insensitive() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    doc.append_child(doc.root(), div).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "DIV"), Some(div));
}

#[test]
fn test_query_selector_tag_no_match() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    doc.append_child(doc.root(), div).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "span"), None);
}

#[test]
fn test_query_selector_id_match() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "main");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "#main"), Some(el));
}

#[test]
fn test_query_selector_id_no_match() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "other");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "#main"), None);
}

#[test]
fn test_query_selector_id_missing() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "#main"), None);
}

#[test]
fn test_query_selector_class_match() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "active");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), ".active"), Some(el));
}

#[test]
fn test_query_selector_class_no_match() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "inactive");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), ".active"), None);
}

#[test]
fn test_query_selector_class_missing() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), ".active"), None);
}

#[test]
fn test_query_selector_multiple_classes() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "a b c");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), ".a.b"), Some(el));
    // 缺少 b 类
    assert_eq!(doc.query_selector(doc.root(), ".a.x"), None);
}

#[test]
fn test_query_selector_attribute_exists() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "data-x", "val");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "[data-x]"), Some(el));
    assert_eq!(doc.query_selector(doc.root(), "[data-y]"), None);
}

#[test]
fn test_query_selector_attribute_exact() {
    let mut doc = Document::new();
    let el = doc.create_element("input");
    doc.set_attribute(el, "type", "text");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "[type=text]"), Some(el));
    assert_eq!(doc.query_selector(doc.root(), "[type=password]"), None);
}

#[test]
fn test_query_selector_attribute_includes() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "data", "foo bar baz");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "[data~=bar]"), Some(el));
    assert_eq!(doc.query_selector(doc.root(), "[data~=qux]"), None);
}

#[test]
fn test_query_selector_combined_tag_id_class() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "main");
    doc.set_attribute(el, "class", "active");
    doc.set_attribute(el, "data-x", "val");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "div#main.active[data-x]"), Some(el));
}

#[test]
fn test_query_selector_combined_wrong_tag() {
    let mut doc = Document::new();
    let el = doc.create_element("span");
    doc.set_attribute(el, "id", "main");
    doc.set_attribute(el, "class", "active");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "div#main.active"), None);
}

#[test]
fn test_query_selector_combined_missing_class() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "main");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), "div#main.active"), None);
}

#[test]
fn test_query_selector_all_multiple_matches() {
    let mut doc = Document::new();
    let el1 = doc.create_element("div");
    let el2 = doc.create_element("div");
    let el3 = doc.create_element("span");
    doc.append_child(doc.root(), el1).unwrap();
    doc.append_child(doc.root(), el2).unwrap();
    doc.append_child(doc.root(), el3).unwrap();
    let results = doc.query_selector_all(doc.root(), "div");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_query_selector_all_no_matches() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();
    let results = doc.query_selector_all(doc.root(), "span");
    assert!(results.is_empty());
}

#[test]
fn test_query_selector_invalid_selector() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();
    assert_eq!(doc.query_selector(doc.root(), ""), None);
    assert_eq!(doc.query_selector(doc.root(), "div#"), None);
}
