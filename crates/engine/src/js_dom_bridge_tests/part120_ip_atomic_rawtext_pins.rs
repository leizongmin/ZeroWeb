// ip 非文本原子拆分边界刀钉测（2026-11-15）：残 20F 普查 C3+C6 族 4 键
// （insertparagraph.js :390-399 script rawtext 拆分 + :1250-1257 注释原子，
// dPS div/p 双面）。C3 = extract sc===ec/contained 面对 Comment/PI 的
// cloneSlice+deleteData 切片语义（原节点原地清空残 `<!---->` 空壳）；C6 =
// 宿主直子 rawtext 壳链 caret 走 wrap ViaHtml 整树重建致拆分位引用脱挂、
// c1InDom 兜底退化空拆分。控制钉 = M1 门 b 壳中点 + 注释前界两绿键防回归。

#[test]
fn test_insertparagraph_c6_script_mid_div() {
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
        "<html><body><div id=ce120a contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h120a = document.getElementById("ce120a");
h120a.innerHTML = '<script>foobar<\/script>baz';
var t120a = h120a.firstChild.firstChild;
var r120a = document.createRange();
r120a.setStart(t120a, 3);
r120a.setEnd(t120a, 3);
getSelection().removeAllRanges();
getSelection().addRange(r120a);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce120a\").innerHTML")
            .unwrap()
            .value,
        "<div><script>foo</script><br></div><div><script>bar</script>baz</div>",
        "① C6 `<script>foo[]bar</script>baz` dPS div：rawtext 中点拆进双 script 元素（corpus :390-394 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c6_script_mid_p() {
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
        "<html><body><div id=ce120b contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h120b = document.getElementById("ce120b");
h120b.innerHTML = '<script>foobar<\/script>baz';
var t120b = h120b.firstChild.firstChild;
var r120b = document.createRange();
r120b.setStart(t120b, 3);
r120b.setEnd(t120b, 3);
getSelection().removeAllRanges();
getSelection().addRange(r120b);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce120b\").innerHTML")
            .unwrap()
            .value,
        "<p><script>foo</script><br></p><p><script>bar</script>baz</p>",
        "② C6 同上 dPS p 面（corpus :395-399 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c3_comment_atomic_div() {
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
        "<html><body><div id=ce120c contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h120c = document.getElementById("ce120c");
h120c.innerHTML = '<p>foo<!--bar-->';
var t120c = h120c.firstChild.firstChild;
var r120c = document.createRange();
r120c.setStart(t120c, 3);
r120c.setEnd(t120c, 3);
getSelection().removeAllRanges();
getSelection().addRange(r120c);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce120c\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><p><!--bar--><br></p>",
        "③ C3 `<p>foo[]<!--bar-->` dPS div：注释原子整体留右段（corpus :1250-1254 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c3_comment_atomic_p() {
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
        "<html><body><div id=ce120d contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h120d = document.getElementById("ce120d");
h120d.innerHTML = '<p>foo<!--bar-->';
var t120d = h120d.firstChild.firstChild;
var r120d = document.createRange();
r120d.setStart(t120d, 3);
r120d.setEnd(t120d, 3);
getSelection().removeAllRanges();
getSelection().addRange(r120d);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce120d\").innerHTML")
            .unwrap()
            .value,
        "<p>foo</p><p><!--bar--><br></p>",
        "④ C3 同上 dPS p 面（corpus :1255-1259 exp 唯一形）"
    );
}

#[test]
fn test_insertparagraph_c6_ctrl_b_mid_div() {
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
        "<html><body><div id=ce120e contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h120e = document.getElementById("ce120e");
h120e.innerHTML = '<b>foobar</b>baz';
var t120e = h120e.firstChild.firstChild;
var r120e = document.createRange();
r120e.setStart(t120e, 3);
r120e.setEnd(t120e, 3);
getSelection().removeAllRanges();
getSelection().addRange(r120e);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce120e\").innerHTML")
            .unwrap()
            .value,
        "<div><b>foo</b></div><div><b>bar</b>baz</div>",
        "⑤ 控制 M1 门 b 壳中点绿键防回归（corpus :763-767 exp）"
    );
}

#[test]
fn test_insertparagraph_c3_ctrl_comment_before_div() {
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
        "<html><body><div id=ce120f contenteditable></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h120f = document.getElementById("ce120f");
h120f.innerHTML = '<p><!--foo-->bar';
var t120f = h120f.firstChild.childNodes[1];
var r120f = document.createRange();
r120f.setStart(t120f, 0);
r120f.setEnd(t120f, 0);
getSelection().removeAllRanges();
getSelection().addRange(r120f);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce120f\").innerHTML")
            .unwrap()
            .value,
        "<p><!--foo--><br></p><p>bar</p>",
        "⑥ 控制 注释前界绿键防回归——注释不入选区恒左留（corpus :1260-1264 exp）"
    );
}
