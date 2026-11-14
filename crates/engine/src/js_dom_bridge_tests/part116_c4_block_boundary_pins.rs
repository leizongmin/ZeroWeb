// C4 刀钉测（2026-11-14）：insertParagraph 拆分边界块级后随兄弟不入新块 +
// bare-br 行界双空块物化。corpus C4 族 14 键（insertparagraph.html gate v3 残面
// ipm2-v3-ip.log census 17 键中宿主级 14 键；dt/blockquote p 尾/li 尾 3 键归
// list/容器链刀域不越界）。三臂实录 exp：① boundaryText wrap 臂表跨面
// `fo[o<table…b]ar`（块级 table 留宿主、新块空垫）；① 宿主元素边界臂 bare-br
// 行界面（`</p>{}<br>` 族双空块形，行中 br 窄门不开）；part01f 主拆分 host 级
// caret 整选删后 `{<h1>…}</p>` 面（extract 收窄首块级相含节点前，空新块 pad）。

#[test]
fn test_insertparagraph_c4_table_cross_div_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21a contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21a = document.getElementById("ce21a");
h21a.innerHTML = "foo<table><tr><td>bar</table>";
var r21a = document.createRange();
r21a.setStart(h21a.childNodes[0], 2);
r21a.setEnd(h21a.querySelector("td").firstChild, 1);
getSelection().removeAllRanges();
getSelection().addRange(r21a);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21a\").innerHTML")
            .unwrap()
            .value,
        "<div>fo</div><div><br></div><table><tbody><tr><td>ar</td></tr></tbody></table>",
        "㉑ C4 表跨选区（dPS div）：块级 table 留宿主不入新块，新块空垫 br（corpus `fo[o<table…b]ar` dPS div 实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_c4_table_cross_p_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21b contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21b = document.getElementById("ce21b");
h21b.innerHTML = "foo<table><tr><td>bar</table>";
var r21b = document.createRange();
r21b.setStart(h21b.childNodes[0], 2);
r21b.setEnd(h21b.querySelector("td").firstChild, 1);
getSelection().removeAllRanges();
getSelection().addRange(r21b);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21b\").innerHTML")
            .unwrap()
            .value,
        "<p>fo</p><p><br></p><table><tbody><tr><td>ar</td></tr></tbody></table>",
        "㉒ C4 表跨选区（dPS p）：sepTag 双面制同形（corpus dPS p 实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_c4_bare_br_tail_double_empty_div_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21c contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21c = document.getElementById("ce21c");
h21c.innerHTML = "<p>foo</p><br>";
var r21c = document.createRange();
r21c.setStart(h21c, 1);
r21c.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r21c);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21c\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><div><br></div><div><br></div>",
        "㉓ C4 bare-br 行界（dPS div）：caret 块界位首裸 br 物化双空块（新块 pad + br 行成块；corpus `</p>{{}}<br>` 实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_c4_bare_br_tail_double_empty_p_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21d contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21d = document.getElementById("ce21d");
h21d.innerHTML = "<p>foo</p><br>";
var r21d = document.createRange();
r21d.setStart(h21d, 1);
r21d.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r21d);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21d\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><p><br></p><p><br></p>",
        "㉔ C4 bare-br 行界（dPS p）：sepTag 双面制同形（corpus dPS p 实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_c4_bare_br_leading_side_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21e contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21e = document.getElementById("ce21e");
h21e.innerHTML = "<br><p>foo</p>";
var r21e = document.createRange();
r21e.setStart(h21e, 0);
r21e.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r21e);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21e\").innerHTML")
            .unwrap()
            .value,
        "<div><br></div><div><br></div><p>foo</p>",
        "㉕ C4 bare-br 行界首侧（dPS div）：`{{}}<br><p>` 双空块 + 后随块级 p 留宿主（corpus 实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_c4_bare_br_following_block_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21f contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21f = document.getElementById("ce21f");
h21f.innerHTML = "<p>foo</p><br><h1>bar</h1>";
var r21f = document.createRange();
r21f.setStart(h21f, 1);
r21f.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r21f);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21f\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><div><br></div><div><br></div><h1>bar</h1>",
        "㉖ C4 bare-br 行界（尾随 h1）：双空块 + h1 留宿主不卷入（corpus `</p>{{}}<br><h1>` 实录 exp0；h1/foo 前缀面同 19/21 键形）"
    );
}

#[test]
fn test_insertparagraph_c4_bare_br_mid_line_gate_closed_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21g contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21g = document.getElementById("ce21g");
h21g.innerHTML = "<b>foo</b><br>";
var r21g = document.createRange();
r21g.setStart(h21g, 1);
r21g.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r21g);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21g\").innerHTML")
            .unwrap()
            .value,
        "<b>foo</b><div><br></div>",
        "㉗ C4 窄门 guard：caret 前行内壳（`<b>foo[]</b><br>` 行中 br 形）双空块物化不开（无语料锚保守现径——br 入新块单块形）"
    );
}

#[test]
fn test_insertparagraph_c4_full_block_select_div_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21h contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21h = document.getElementById("ce21h");
h21h.innerHTML = "<p>foo</p><h1>bar</h1><p>baz</p>";
var r21h = document.createRange();
r21h.setStart(h21h, 1);
r21h.setEnd(h21h, 2);
getSelection().removeAllRanges();
getSelection().addRange(r21h);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21h\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><div><br></div><p>baz</p>",
        "㉘ C4 整块选删后 host 级 caret（dPS div）：extract 收窄首块级相含节点前——新块空垫 + p-baz 留宿主（corpus `{{<h1>…}}</p>` got `<div><p>baz</p></div>` 嵌套违规形修复）"
    );
}

#[test]
fn test_insertparagraph_c4_full_block_select_p_m3c4() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=ce21i contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h21i = document.getElementById("ce21i");
h21i.innerHTML = "<p>foo</p><h1>bar</h1><p>baz</p>";
var r21i = document.createRange();
r21i.setStart(h21i, 1);
r21i.setEnd(h21i, 2);
getSelection().removeAllRanges();
getSelection().addRange(r21i);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce21i\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><p><br></p><p>baz</p>",
        "㉙ C4 整块选删后 host 级 caret（dPS p）：同形 p 面不产 p 嵌套违规（corpus got `<p><p>baz</p></p>` 修复）"
    );
}
