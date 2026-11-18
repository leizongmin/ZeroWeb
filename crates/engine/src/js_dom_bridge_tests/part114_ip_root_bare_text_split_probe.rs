// IP-M3 刀 A 族探针（2026-11-14）：可编辑根层非首行裸文本 caret insertParagraph
// 拆分——legacy ① 臂 boundaryText 裸留门。corpus A 族 22 键（insertparagraph.html
// gate v3 残面 ipm2-v3-ip.log）实录 Chrome 三子形：行中尾位拆分（`bar[]<br>`）左
// 段裸留根层仅右半包新块；词中切片（`ba[]r`）原文本节点截尾留 host；行首拆分
// （`[]bar`，前一兄弟 br）根层补裸 br 行界。首行面（caret 前无 br）维持 F2 wrap
// 径（mt rich 组 `foo[]bar` 23 案录音 `<p>foo</p><p>{}bar</p>`）。

#[test]
fn test_insertparagraph_root_bare_mid_line_m3a() {
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
        "<html><body><div id=ce16a contenteditable>foo<br>bar<br><span>baz</span></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host16 = document.getElementById("ce16a");
var t16 = host16.childNodes[2];
var r16 = document.createRange();
r16.setStart(t16, 3);
r16.setEnd(t16, 3);
getSelection().removeAllRanges();
getSelection().addRange(r16);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce16a\").innerHTML")
            .unwrap()
            .value,
        "foo<br>bar<div><br><span>baz</span></div>",
        "⑯ M3-A 行中尾位拆分：非首行裸文本 caret 左段裸留根层，tail 段与后随兄弟入新块（corpus A 族实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_root_bare_line_start_m3a() {
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
        "<html><body><div id=ce16b contenteditable>foo<br>bar<br><span>baz</span></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host17 = document.getElementById("ce16b");
var t17 = host17.childNodes[2];
var r17 = document.createRange();
r17.setStart(t17, 0);
r17.setEnd(t17, 0);
getSelection().removeAllRanges();
getSelection().addRange(r17);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce16b\").innerHTML")
            .unwrap()
            .value,
        "foo<br><br><div>bar<br><span>baz</span></div>",
        "⑰ M3-A 行首拆分：caret 文本节点整体入新块 + 根层补裸 br 行界（corpus `foo<br>[]bar…` 5 案实录）"
    );
}

#[test]
fn test_insertparagraph_root_bare_mid_word_m3a() {
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
        "<html><body><div id=ce16c contenteditable>foo<br>bar<br><span>baz</span></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host18 = document.getElementById("ce16c");
var t18 = host18.childNodes[2];
var r18 = document.createRange();
r18.setStart(t18, 2);
r18.setEnd(t18, 2);
getSelection().removeAllRanges();
getSelection().addRange(r18);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce16c\").innerHTML")
            .unwrap()
            .value,
        "foo<br>ba<div>r<br><span>baz</span></div>",
        "⑱ M3-A 词中切片：原文本节点截尾留 host（head 段），tail 段入新块（corpus `ba[]r` 实录 exp0）"
    );
}

#[test]
fn test_insertparagraph_root_first_line_wrap_kept_m3a() {
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
        "<html><body><div id=ce16d contenteditable>foobar</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host19 = document.getElementById("ce16d");
var t19 = host19.childNodes[0];
var r19 = document.createRange();
r19.setStart(t19, 3);
r19.setEnd(t19, 3);
getSelection().removeAllRanges();
getSelection().addRange(r19);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce16d\").innerHTML")
            .unwrap()
            .value,
        "<div>foo</div><div>bar</div>",
        "⑲ M3-A 首行面 guard：caret 前无 br 维持 F2 wrap 径（mt rich 组 foo[]bar 录音形，裸留门不开）"
    );
}

#[test]
fn test_insertparagraph_root_bare_dps_p_m3a() {
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
        "<html><body><div id=ce16e contenteditable>foo<br>bar<br><span>baz</span></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host20 = document.getElementById("ce16e");
var t20 = host20.childNodes[2];
var r20 = document.createRange();
r20.setStart(t20, 3);
r20.setEnd(t20, 3);
getSelection().removeAllRanges();
getSelection().addRange(r20);
document.execCommand("defaultparagraphseparator", false, "p");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce16e\").innerHTML")
            .unwrap()
            .value,
        "foo<br>bar<p><br><span>baz</span></p>",
        "⑳ M3-A dPS p 面：sepTag 双面制显式 setter 一票胜出（corpus dPS p 11 键同形）"
    );
}
