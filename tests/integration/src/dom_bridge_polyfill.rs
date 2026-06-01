//! DOM Bridge polyfill 行为测试。
//!
//! 通过 V8 引擎实际执行 polyfill 代码，验证 JS 侧 DOM API 的运行时行为。

use zero_engine::dom_bridge::generate_dom_api_polyfill;
use zero_script_sandbox::V8Sandbox;

/// 辅助：在 V8 中执行 polyfill + 测试代码，返回原始结果字符串。
fn eval_polyfill(test_code: &str) -> String {
    let mut sandbox = V8Sandbox::new().expect("V8 init");
    let polyfill = generate_dom_api_polyfill();
    let full_code = format!("{polyfill}\n{test_code}");
    let result = sandbox.execute(&full_code).expect("execute");
    result.value
}

#[test]
fn test_polyfill_create_element() {
    let result = eval_polyfill("var div = document.createElement('div');\nJSON.stringify(div.tagName);");
    assert_eq!(result.trim_matches('"'), "DIV", "tagName should be 'DIV'");
}

#[test]
fn test_polyfill_create_text_node() {
    let result = eval_polyfill("var t = document.createTextNode('hello');\nJSON.stringify(t.textContent);");
    assert_eq!(result.trim_matches('"'), "hello");
}

#[test]
fn test_polyfill_set_get_attribute() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'test-id');
        el.setAttribute('class', 'a b');
        var r = {id: el.getAttribute('id'), cls: el.getAttribute('class'), has: el.hasAttribute('id')};
        JSON.stringify(r);
    "#,
    );
    assert!(result.contains("test-id"), "id 应为 'test-id': {result}");
    assert!(result.contains("a b"), "class 应为 'a b': {result}");
}

#[test]
fn test_polyfill_remove_attribute() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('data-x', '123');
        el.removeAttribute('data-x');
        JSON.stringify(el.hasAttribute('data-x'));
    "#,
    );
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_polyfill_append_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var c = document.createElement('span');
        p.appendChild(c);
        JSON.stringify(p.children.length);
    "#,
    );
    assert_eq!(result.trim(), "1");
}

#[test]
fn test_polyfill_remove_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var c = document.createElement('span');
        p.appendChild(c);
        p.removeChild(c);
        JSON.stringify(p.children.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_polyfill_child_parent_node() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var c = document.createElement('span');
        p.appendChild(c);
        JSON.stringify(c.parentNode === p);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_polyfill_insert_before() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var a = document.createElement('a');
        var b = document.createElement('b');
        var c = document.createElement('c');
        p.appendChild(a);
        p.appendChild(c);
        p.insertBefore(b, c);
        JSON.stringify(p.children[1].tagName);
    "#,
    );
    assert_eq!(result.trim_matches('"'), "B");
}

#[test]
fn test_polyfill_clone_node_shallow() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'orig');
        el.appendChild(document.createElement('span'));
        var clone = el.cloneNode(false);
        JSON.stringify([clone.getAttribute('class'), clone.children.length]);
    "#,
    );
    assert!(result.contains("orig"), "浅拷贝应保留属性: {result}");
    assert!(result.contains("0"), "浅拷贝不应含子节点: {result}");
}

#[test]
fn test_polyfill_clone_node_deep() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'src');
        var child = document.createElement('span');
        child.setAttribute('class', 'inner');
        el.appendChild(child);
        var clone = el.cloneNode(true);
        JSON.stringify([clone.getAttribute('id'), clone.children.length]);
    "#,
    );
    assert!(result.contains("src"), "深拷贝应保留属性: {result}");
    assert!(result.contains("1"), "深拷贝应含子节点: {result}");
}

#[test]
fn test_polyfill_text_content_getter() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.appendChild(document.createTextNode('Hello '));
        el.appendChild(document.createTextNode('World'));
        JSON.stringify(el.getTextContent());
    "#,
    );
    assert_eq!(result.trim_matches('"'), "Hello World");
}

#[test]
fn test_polyfill_has_child_nodes() {
    let result = eval_polyfill(
        r#"
        var e = document.createElement('div');
        var f = document.createElement('div');
        f.appendChild(document.createElement('span'));
        JSON.stringify([e.hasChildNodes(), f.hasChildNodes()]);
    "#,
    );
    assert!(result.contains("false"), "空元素 hasChildNodes 应 false: {result}");
    assert!(result.contains("true"), "有子节点 hasChildNodes 应 true: {result}");
}

#[test]
fn test_polyfill_replace_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var old = document.createElement('span');
        var nw = document.createElement('b');
        p.appendChild(old);
        p.replaceChild(nw, old);
        JSON.stringify(p.children[0].tagName);
    "#,
    );
    assert_eq!(result.trim_matches('"'), "B");
}

#[test]
fn test_polyfill_document_fragment_type() {
    let result = eval_polyfill("var frag = document.createDocumentFragment(); JSON.stringify(frag.nodeType);");
    assert_eq!(result.trim(), "11", "DocumentFragment nodeType 应为 11");
}

#[test]
fn test_polyfill_id_classname() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'myId');
        el.setAttribute('class', 'foo bar');
        JSON.stringify([el.id, el.className]);
    "#,
    );
    assert!(result.contains("myId"), "el.id 应为 'myId': {result}");
    assert!(result.contains("foo bar"), "el.className 应为 'foo bar': {result}");
}

#[test]
fn test_polyfill_get_element_by_id() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'target');
        document.body.appendChild(el);
        JSON.stringify(document.getElementById('target') !== null);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_polyfill_get_element_by_id_not_found() {
    let result = eval_polyfill("JSON.stringify(document.getElementById('nonexistent'));");
    assert_eq!(result.trim(), "null");
}
