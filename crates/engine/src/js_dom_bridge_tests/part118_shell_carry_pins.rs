// C2/F/I6 方向性壳携刀钉测（2026-11-15）：insertParagraph 空块垫底 br 壳链
// 方向性语义——样式壳携全链、`<a>` 链接壳弃携，左空块（offset-0 形，左臂
// 携链不带重开门）与右空块（携带链归一臂：含 a 全弃 / 纯样式浅链扩全）同域。
// 16 键逐键断言 corpus exp 接受形（insertparagraph.html census C2 8 + F 6 +
// I6 2，ipm3-b-v6-ip.log 16F 全集；探针定谳 br 新建/携带谱与 part118 探针档
// 一致：C2 臂 L 新建、F/I6 携带剥/扩后垫底新建）。

#[test]
fn test_insertparagraph_c2_left_b_p_shell_carry() {
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
        "<html><body><div id=ce118a contenteditable><p><b>foo</b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118a = document.getElementById("ce118a");
var t118a = h118a.firstChild.firstChild.firstChild;
var r118a = document.createRange();
r118a.setStart(t118a, 0);
r118a.setEnd(t118a, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118a);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118a\").innerHTML")
            .unwrap()
            .value,
        "<p><b><br></b></p><p><b>foo</b></p>",
        "① C2 `<p><b>[]foo</b></p>`：左空块垫底 br 携 b 链（corpus :830 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c2_left_b_div_shell_carry() {
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
        "<html><body><div id=ce118b contenteditable><div><b>foo</b></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118b = document.getElementById("ce118b");
var t118b = h118b.firstChild.firstChild.firstChild;
var r118b = document.createRange();
r118b.setStart(t118b, 0);
r118b.setEnd(t118b, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118b);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118b\").innerHTML")
            .unwrap()
            .value,
        "<div><b><br></b></div><div><b>foo</b></div>",
        "② C2 `<div><b>[]foo</b></div>` div 宿主镜像（corpus :835 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c2_left_bi_p_shell_carry() {
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
        "<html><body><div id=ce118c contenteditable><p><b><i>foo</i></b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118c = document.getElementById("ce118c");
var t118c = h118c.firstChild.firstChild.firstChild.firstChild;
var r118c = document.createRange();
r118c.setStart(t118c, 0);
r118c.setEnd(t118c, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118c);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118c\").innerHTML")
            .unwrap()
            .value,
        "<p><b><i><br></i></b></p><p><b><i>foo</i></b></p>",
        "③ C2 `<p><b><i>[]foo</i></b></p>`：双壳全链携（链方向保序，corpus :840 exp）"
    );
}

#[test]
fn test_insertparagraph_c2_left_bi_div_shell_carry() {
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
        "<html><body><div id=ce118d contenteditable><div><b><i>foo</i></b></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118d = document.getElementById("ce118d");
var t118d = h118d.firstChild.firstChild.firstChild.firstChild;
var r118d = document.createRange();
r118d.setStart(t118d, 0);
r118d.setEnd(t118d, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118d);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118d\").innerHTML")
            .unwrap()
            .value,
        "<div><b><i><br></i></b></div><div><b><i>foo</i></b></div>",
        "④ C2 `<div><b><i>[]foo</i></b></div>` div 宿主镜像（corpus :845 exp）"
    );
}

#[test]
fn test_insertparagraph_c2_left_ib_p_shell_carry() {
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
        "<html><body><div id=ce118e contenteditable><p><i><b>foo</b></i></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118e = document.getElementById("ce118e");
var t118e = h118e.firstChild.firstChild.firstChild.firstChild;
var r118e = document.createRange();
r118e.setStart(t118e, 0);
r118e.setEnd(t118e, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118e);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118e\").innerHTML")
            .unwrap()
            .value,
        "<p><i><b><br></b></i></p><p><i><b>foo</b></i></p>",
        "⑤ C2 `<p><i><b>[]foo</b></i></p>`：i<b 方向镜像全链携（corpus :850 exp）"
    );
}

#[test]
fn test_insertparagraph_c2_left_ib_div_shell_carry() {
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
        "<html><body><div id=ce118f contenteditable><div><i><b>foo</b></i></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118f = document.getElementById("ce118f");
var t118f = h118f.firstChild.firstChild.firstChild.firstChild;
var r118f = document.createRange();
r118f.setStart(t118f, 0);
r118f.setEnd(t118f, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118f);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118f\").innerHTML")
            .unwrap()
            .value,
        "<div><i><b><br></b></i></div><div><i><b>foo</b></i></div>",
        "⑥ C2 `<div><i><b>[]foo</b></i></div>` div 宿主镜像（corpus :855 exp）"
    );
}

#[test]
fn test_insertparagraph_c2_left_baouter_p_shell_carry() {
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
        "<html><body><div id=ce118g contenteditable><p><b><a href=foo>foo</a></b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118g = document.getElementById("ce118g");
var t118g = h118g.firstChild.firstChild.firstChild.firstChild;
var r118g = document.createRange();
r118g.setStart(t118g, 0);
r118g.setEnd(t118g, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118g);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118g\").innerHTML")
            .unwrap()
            .value,
        "<p><b><br></b></p><p><b><a href=\"foo\">foo</a></b></p>",
        "⑦ C2 `<p><b><a href>[]foo</a></b></p>`：左块 b 携 a 弃、右块 a 携内容（corpus :1092 exp）"
    );
}

#[test]
fn test_insertparagraph_c2_left_baouter_div_shell_carry() {
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
        "<html><body><div id=ce118h contenteditable><div><b><a href=foo>foo</a></b></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118h = document.getElementById("ce118h");
var t118h = h118h.firstChild.firstChild.firstChild.firstChild;
var r118h = document.createRange();
r118h.setStart(t118h, 0);
r118h.setEnd(t118h, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118h);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118h\").innerHTML")
            .unwrap()
            .value,
        "<div><b><br></b></div><div><b><a href=\"foo\">foo</a></b></div>",
        "⑧ C2 `<div><b><a href>[]foo</a></b></div>` div 宿主镜像（corpus :1097 exp）"
    );
}

#[test]
fn test_insertparagraph_c2_left_aouter_p_bare_pad() {
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
        "<html><body><div id=ce118i contenteditable><p><a href=foo><b>foo</b></a></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118i = document.getElementById("ce118i");
var t118i = h118i.firstChild.firstChild.firstChild.firstChild;
var r118i = document.createRange();
r118i.setStart(t118i, 0);
r118i.setEnd(t118i, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118i);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118i\").innerHTML")
            .unwrap()
            .value,
        "<p><br></p><p><a href=\"foo\"><b>foo</b></a></p>",
        "⑰ C2 a-outer `<p><a href><b>[]foo</b></a></p>`：最外层 a 之上无壳＝左块裸垫（corpus :1072 exp；v1 批 P2F 回归根因面）"
    );
}

#[test]
fn test_insertparagraph_c2_left_aouter_div_bare_pad() {
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
        "<html><body><div id=ce118j contenteditable><div><a href=foo><b>foo</b></a></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118j = document.getElementById("ce118j");
var t118j = h118j.firstChild.firstChild.firstChild.firstChild;
var r118j = document.createRange();
r118j.setStart(t118j, 0);
r118j.setEnd(t118j, 0);
getSelection().removeAllRanges();
getSelection().addRange(r118j);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118j\").innerHTML")
            .unwrap()
            .value,
        "<div><br></div><div><a href=\"foo\"><b>foo</b></a></div>",
        "⑱ C2 a-outer div 宿主镜像＝左块裸垫（corpus :1077 exp）"
    );
}

#[test]
fn test_insertparagraph_f_right_a_p_chain_drop() {
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
        "<html><body><div id=ce118i contenteditable><p><a href=foo>foo&nbsp;<br></a></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118i = document.getElementById("ce118i");
var t118i = h118i.firstChild.firstChild.firstChild;
var r118i = document.createRange();
r118i.setStart(t118i, 4);
r118i.setEnd(t118i, 4);
getSelection().removeAllRanges();
getSelection().addRange(r118i);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118i\").innerHTML")
            .unwrap()
            .value,
        "<p><a href=\"foo\">foo&nbsp;</a></p><p><br></p>",
        "⑨ F `<p><a href>foo&nbsp;[]<br></a></p>`：右空块 a 链全弃裸 br（corpus :1145 exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_f_right_a_div_chain_drop() {
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
        "<html><body><div id=ce118j contenteditable><div><a href=foo>foo&nbsp;<br></a></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118j = document.getElementById("ce118j");
var t118j = h118j.firstChild.firstChild.firstChild;
var r118j = document.createRange();
r118j.setStart(t118j, 4);
r118j.setEnd(t118j, 4);
getSelection().removeAllRanges();
getSelection().addRange(r118j);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118j\").innerHTML")
            .unwrap()
            .value,
        "<div><a href=\"foo\">foo&nbsp;</a></div><div><br></div>",
        "⑩ F `<div><a href>foo&nbsp;[]<br></a></div>` div 宿主镜像（corpus :1151 exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_f_right_ab_p_chain_drop() {
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
        "<html><body><div id=ce118k contenteditable><p><a href=foo><b>foo&nbsp;<br></b></a></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118k = document.getElementById("ce118k");
var t118k = h118k.firstChild.firstChild.firstChild.firstChild;
var r118k = document.createRange();
r118k.setStart(t118k, 4);
r118k.setEnd(t118k, 4);
getSelection().removeAllRanges();
getSelection().addRange(r118k);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118k\").innerHTML")
            .unwrap()
            .value,
        "<p><a href=\"foo\"><b>foo&nbsp;</b></a></p><p><br></p>",
        "⑪ F `<p><a href><b>foo&nbsp;[]<br></b></a></p>`：a 内 b 同弃（corpus :1157 exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_f_right_ab_div_chain_drop() {
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
        "<html><body><div id=ce118l contenteditable><div><a href=foo><b>foo&nbsp;<br></b></a></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118l = document.getElementById("ce118l");
var t118l = h118l.firstChild.firstChild.firstChild.firstChild;
var r118l = document.createRange();
r118l.setStart(t118l, 4);
r118l.setEnd(t118l, 4);
getSelection().removeAllRanges();
getSelection().addRange(r118l);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118l\").innerHTML")
            .unwrap()
            .value,
        "<div><a href=\"foo\"><b>foo&nbsp;</b></a></div><div><br></div>",
        "⑫ F `<div><a href><b>foo&nbsp;[]<br></b></a></div>` div 宿主镜像（corpus :1163 exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_f_right_ba_p_chain_drop() {
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
        "<html><body><div id=ce118m contenteditable><p><b><a href=foo>foo&nbsp;<br></a></b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118m = document.getElementById("ce118m");
var t118m = h118m.firstChild.firstChild.firstChild.firstChild;
var r118m = document.createRange();
r118m.setStart(t118m, 4);
r118m.setEnd(t118m, 4);
getSelection().removeAllRanges();
getSelection().addRange(r118m);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118m\").innerHTML")
            .unwrap()
            .value,
        "<p><b><a href=\"foo\">foo&nbsp;</a></b></p><p><br></p>",
        "⑬ F `<p><b><a href>foo&nbsp;[]<br></a></b></p>`：b 外 a 同弃裸 br（corpus :1169 exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_f_right_ba_div_chain_drop() {
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
        "<html><body><div id=ce118n contenteditable><div><b><a href=foo>foo&nbsp;<br></a></b></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118n = document.getElementById("ce118n");
var t118n = h118n.firstChild.firstChild.firstChild.firstChild;
var r118n = document.createRange();
r118n.setStart(t118n, 4);
r118n.setEnd(t118n, 4);
getSelection().removeAllRanges();
getSelection().addRange(r118n);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118n\").innerHTML")
            .unwrap()
            .value,
        "<div><b><a href=\"foo\">foo&nbsp;</a></b></div><div><br></div>",
        "⑭ F `<div><b><a href>foo&nbsp;[]<br></a></b></div>` div 宿主镜像（corpus :1178 exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_i6_double_span_full_chain() {
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
        "<html><body><div id=ce118o contenteditable><div><span style=\"color:rgb(0, 0, 255)\"><span style=\"background-color:rgb(0, 0, 255)\">foo</span><br></span></div></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118o = document.getElementById("ce118o");
var t118o = h118o.firstChild.firstChild.firstChild.firstChild;
var r118o = document.createRange();
r118o.setStart(t118o, 3);
r118o.setEnd(t118o, 3);
getSelection().removeAllRanges();
getSelection().addRange(r118o);
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118o\").innerHTML")
            .unwrap()
            .value,
        "<div><span style=\"color:rgb(0, 0, 255)\"><span style=\"background-color:rgb(0, 0, 255)\">foo</span></span></div><div><span style=\"color:rgb(0, 0, 255)\"><span style=\"background-color:rgb(0, 0, 255)\"><br></span></span></div>",
        "⑮ I6 `<div><span color><span bg>foo[]</span><br></span></div>`：右块双壳全链（corpus exp 备选 1）"
    );
}

#[test]
fn test_insertparagraph_i6_double_span_li_full_chain() {
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
        "<html><body><div id=ce118p contenteditable><ul><li><span style=\"color:rgb(0, 0, 255)\"><span style=\"background-color:rgb(0, 0, 255)\">foo</span><br></span></li></ul></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h118p = document.getElementById("ce118p");
var t118p = h118p.firstChild.firstChild.firstChild.firstChild.firstChild;
var r118p = document.createRange();
r118p.setStart(t118p, 3);
r118p.setEnd(t118p, 3);
getSelection().removeAllRanges();
getSelection().addRange(r118p);
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce118p\").innerHTML")
            .unwrap()
            .value,
        "<ul><li><span style=\"color:rgb(0, 0, 255)\"><span style=\"background-color:rgb(0, 0, 255)\">foo</span></span></li><li><span style=\"color:rgb(0, 0, 255)\"><span style=\"background-color:rgb(0, 0, 255)\"><br></span></span></li></ul>",
        "⑯ I6 ul>li 变体：li 拆分双壳全链（corpus exp 备选 1）"
    );
}
