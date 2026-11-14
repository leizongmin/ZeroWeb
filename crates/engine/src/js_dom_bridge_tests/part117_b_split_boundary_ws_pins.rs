// B 族刀钉测（2026-11-14）：insertParagraph 拆点块界空白/nbsp 归一。corpus B 族
// 15 键（insertparagraph.html gate v3 残面 ipm3-c4-v3-ip.log census B 族 17 键中
// 拆点归一面 15 键；`foo&nbsp;[]<br>` a/b 壳携 2 键归 C2 方向性壳携刀域不越界）。
// 两臂实录 exp：② sep2 同型块拆臂（`<div>a…[]…b</div>` 族 14 键——折叠空格行界
// 翻 `&nbsp;`、whitespace-only 段退化空块占位 br、单侧退化不翻对侧）；⑦ 宿主
// 元素边界臂（CE-false 跨行 1 键——尾侧 whitespace-only 留宿主根层、首文本行界
// 翻 `&nbsp;` 仅头侧有行内内容时）。

#[test]
fn test_insertparagraph_b_ws_tail_flip_div_m3b() {
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
        "<html><body><div id=ce22a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22a = document.getElementById("ce22a");
h22a.innerHTML = "<div>a b</div>";
var r22a = document.createRange();
r22a.setStart(h22a.firstChild.firstChild, 1);
r22a.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22a);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22a\").innerHTML")
            .unwrap()
            .value,
        "<div>a</div><div>&nbsp;b</div>",
        "① B 族 `a[] b`：右段行首折叠空格翻 `&nbsp;`（corpus `a[] b` dPS div 实录 exp0 唯一接受形）"
    );
}

#[test]
fn test_insertparagraph_b_ws_head_flip_div_m3b() {
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
        "<html><body><div id=ce22b contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22b = document.getElementById("ce22b");
h22b.innerHTML = "<div>a b</div>";
var r22b = document.createRange();
r22b.setStart(h22b.firstChild.firstChild, 2);
r22b.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22b);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22b\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;</div><div>b</div>",
        "② B 族 `a []b`：左段行尾折叠空格翻 `&nbsp;`（corpus `a []b` dPS div 实录 exp0 唯一接受形）"
    );
}

#[test]
fn test_insertparagraph_b_ws_nbsp_head_tail_div_m3b() {
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
        "<html><body><div id=ce22c contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22c = document.getElementById("ce22c");
h22c.innerHTML = "<div>a&nbsp; b</div>";
var r22c = document.createRange();
r22c.setStart(h22c.firstChild.firstChild, 2);
r22c.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22c);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22c\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;</div><div>&nbsp;b</div>",
        "③ B 族 `a&nbsp;[] b`：左段尾 nbsp 保持 + 右段首空格翻 nbsp（corpus 实录 exp0 唯一接受形）"
    );
}

#[test]
fn test_insertparagraph_b_ws_head_flip_nbsp_tail_div_m3b() {
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
        "<html><body><div id=ce22d contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22d = document.getElementById("ce22d");
h22d.innerHTML = "<div>a &nbsp;b</div>";
var r22d = document.createRange();
r22d.setStart(h22d.firstChild.firstChild, 2);
r22d.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22d);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22d\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;</div><div>&nbsp;b</div>",
        "④ B 族 `a []&nbsp;b`：左段尾空格翻 nbsp + 右段首 nbsp 保持（corpus 实录 exp0 唯一接受形）"
    );
}

#[test]
fn test_insertparagraph_b_ws_lead_run_swap_div_m3b() {
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
        "<html><body><div id=ce22e contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22e = document.getElementById("ce22e");
h22e.innerHTML = "<div>a&nbsp; &nbsp;b</div>";
var r22e = document.createRange();
r22e.setStart(h22e.firstChild.firstChild, 2);
r22e.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22e);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22e\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;</div><div>&nbsp; b</div>",
        "⑤ B 族 `a&nbsp;[] &nbsp;b`：右段行首空白串 ` nbsp` 翻转 `nbsp `（首字翻 nbsp 余位换空格；corpus 实录 exp0 唯一接受形）"
    );
}

#[test]
fn test_insertparagraph_b_ws_lead_run_swap_tail_nbsp_head_div_m3b() {
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
        "<html><body><div id=ce22f contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22f = document.getElementById("ce22f");
h22f.innerHTML = "<div>a &nbsp; b</div>";
var r22f = document.createRange();
r22f.setStart(h22f.firstChild.firstChild, 2);
r22f.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22f);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22f\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;</div><div>&nbsp; b</div>",
        "⑥ B 族 `a []&nbsp; b`：左段尾空格翻 nbsp + 右段首串首字翻 nbsp（corpus 实录 exp0 唯一接受形）"
    );
}

#[test]
fn test_insertparagraph_b_ws_trailing_space_flip_div_m3b() {
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
        "<html><body><div id=ce22g contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22g = document.getElementById("ce22g");
h22g.innerHTML = "<div>a&nbsp; &nbsp; b</div>";
var r22g = document.createRange();
r22g.setStart(h22g.firstChild.firstChild, 3);
r22g.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22g);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22g\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;&nbsp;</div><div>&nbsp; b</div>",
        "⑦ B 族 `a&nbsp; []&nbsp; b`：左段行尾折叠空格翻 nbsp（nbsp 保持）对齐接受形首项（corpus assert_in_array exp0）"
    );
}

#[test]
fn test_insertparagraph_b_ws_double_nbsp_head_div_m3b() {
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
        "<html><body><div id=ce22h contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22h = document.getElementById("ce22h");
h22h.innerHTML = "<div>a&nbsp;&nbsp; &nbsp;b</div>";
var r22h = document.createRange();
r22h.setStart(h22h.firstChild.firstChild, 3);
r22h.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22h);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22h\").innerHTML")
            .unwrap()
            .value,
        "<div>a&nbsp;&nbsp;</div><div>&nbsp; b</div>",
        "⑧ B 族 `a&nbsp;&nbsp;[] &nbsp;b`：左段尾 nbsp 保持右段首字翻（corpus assert_in_array exp0）"
    );
}

#[test]
fn test_insertparagraph_b_ws_space_nbsp_head_div_m3b() {
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
        "<html><body><div id=ce22i contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22i = document.getElementById("ce22i");
h22i.innerHTML = "<div>a &nbsp; &nbsp;b</div>";
var r22i = document.createRange();
r22i.setStart(h22i.firstChild.firstChild, 3);
r22i.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22i);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22i\").innerHTML")
            .unwrap()
            .value,
        "<div>a &nbsp;</div><div>&nbsp; b</div>",
        "⑨ B 族 `a &nbsp;[] &nbsp;b`：左段行尾 nbsp 保持（前位空格不动）右段首字翻（corpus assert_in_array exp1）"
    );
}

#[test]
fn test_insertparagraph_b_ws_tail_degenerate_div_m3b() {
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
        "<html><body><div id=ce22j contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22j = document.getElementById("ce22j");
h22j.innerHTML = "<div>abc </div>";
var r22j = document.createRange();
r22j.setStart(h22j.firstChild.firstChild, 3);
r22j.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22j);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22j\").innerHTML")
            .unwrap()
            .value,
        "<div>abc</div><div><br></div>",
        "⑩ B 族 `abc[] `：右段 whitespace-only 退化，新块占位 br（corpus assert_in_array exp0）"
    );
}

#[test]
fn test_insertparagraph_b_ws_tail_degenerate_two_div_m3b() {
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
        "<html><body><div id=ce22k contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22k = document.getElementById("ce22k");
h22k.innerHTML = "<div>abc  </div>";
var r22k = document.createRange();
r22k.setStart(h22k.firstChild.firstChild, 3);
r22k.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22k);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22k\").innerHTML")
            .unwrap()
            .value,
        "<div>abc</div><div><br></div>",
        "⑪ B 族 `abc[]  `：右段多空白 whitespace-only 退化同形（corpus assert_in_array exp0）"
    );
}

#[test]
fn test_insertparagraph_b_ws_head_content_tail_degenerate_div_m3b() {
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
        "<html><body><div id=ce22l contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22l = document.getElementById("ce22l");
h22l.innerHTML = "<div>abc  </div>";
var r22l = document.createRange();
r22l.setStart(h22l.firstChild.firstChild, 4);
r22l.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22l);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22l\").innerHTML")
            .unwrap()
            .value,
        "<div>abc </div><div><br></div>",
        "⑫ B 族 `abc [] `：右段退化时左段尾空格不翻（单侧退化跳过对侧归一；corpus assert_in_array exp1）"
    );
}

#[test]
fn test_insertparagraph_b_ws_head_degenerate_div_m3b() {
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
        "<html><body><div id=ce22m contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22m = document.getElementById("ce22m");
h22m.innerHTML = "<div>  abc</div>";
var r22m = document.createRange();
r22m.setStart(h22m.firstChild.firstChild, 1);
r22m.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22m);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22m\").innerHTML")
            .unwrap()
            .value,
        "<div><br></div><div> abc</div>",
        "⑬ B 族 ` [] abc`：左段 whitespace-only 退化占位 br，右段首空格不翻（corpus assert_in_array exp1）"
    );
}

#[test]
fn test_insertparagraph_b_ws_head_degenerate_two_div_m3b() {
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
        "<html><body><div id=ce22n contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22n = document.getElementById("ce22n");
h22n.innerHTML = "<div>  abc</div>";
var r22n = document.createRange();
r22n.setStart(h22n.firstChild.firstChild, 2);
r22n.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22n);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22n\").innerHTML")
            .unwrap()
            .value,
        "<div><br></div><div>abc</div>",
        "⑭ B 族 `  []abc`：左段多空白退化，右段实文首字不翻（corpus assert_in_array exp0）"
    );
}

#[test]
fn test_insertparagraph_b_ws_cefalse_host_boundary_m3b() {
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
        "<html><body><div id=ce22o contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var h22o = document.getElementById("ce22o");
h22o.innerHTML = ' <span contenteditable="false">A</span> ; <span contenteditable="false">B</span> ; <span contenteditable="false">C</span> ';
var r22o = document.createRange();
r22o.setStart(h22o.childNodes[2], 0);
r22o.collapse(true);
getSelection().removeAllRanges();
getSelection().addRange(r22o);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce22o\").innerHTML")
            .unwrap()
            .value,
        " <div><span contenteditable=\"false\">A</span></div><div>&nbsp;; <span contenteditable=\"false\">B</span> ; <span contenteditable=\"false\">C</span></div> ",
        "⑮ B 族 CE-false 跨行（wrap 臂，caret 规约进后随文本 offset 0 同 corpus）：新块首文本行界空格翻 nbsp + 尾侧 whitespace-only 出块留宿主原层（corpus assert_in_array exp0）"
    );
}
