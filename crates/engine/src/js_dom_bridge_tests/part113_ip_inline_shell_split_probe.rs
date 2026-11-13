// 编辑命令兼容 ip 206 M1 主桶探针（2026-11-13，DC-4 div/p 容器子族 census
// ip-divp-census-mech.txt M1 三子形）——insertParagraph 拆分边界 inline
// formatting element 复制/丢弃语义钉测。
//
// Chrome 实录（e1-ip.log r2 post @ffdcd28af，div contenteditable 宿主，
// defaultparagraphseparator=div 前置，同语料 runConformanceTest setupDiv 形）：
// ① M1b 壳内 caret 拆分：<b>foobar</b>baz caret (text,3) →
//   <b>foo</b><div><b>bar</b>baz</div>（壳头侧闭合、新块重开壳链、壳外尾随
//   文本随新块）。现 got=<div><b>foobar</b>baz</div><div><br></div>——拆分点
//   落块尾非 caret（ipara 非 SLC wrap 臂 caret 语义缺失）。
// ② M1c span 壳同型：<span>foobar</span>baz caret (text,3) →
//   <span>foo</span><div><span>bar</span>baz</div>。现 got=<div><span>foobar
//   </span>baz</div><div><br></div>。
// ③ M1a 非折叠壳全含选区：foo<span style>[bar]</span>baz 选区 [bar] →
//   foo<div>baz</div>（选区删除拆分后**空壳丢弃**）。现 got 残留空
//   <span style></span> 壳（处方 c delete+split 后 clone 臂不丢空壳）。
// 红 = 现缺陷面（钉 Chrome 目标行为，修复后转绿）；绿 = 语义已到位。
// 断言面：SetInnerHtml mutation 串恰等（compare innerHTML 同源）。
// FIXME(ip-M1-knife)：三形 22:5x 红定谳（桥级 got=①②整段 wrap div 不拆 caret /
// ③空壳残留）——ipara 非 SLC wrap 臂 caret 语义 + 删后空壳丢弃修复刀转正去 ignore。
#[ignore]
#[test]
fn test_insertparagraph_caret_in_b_shell_split_m1() {
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
        "<html><body><div id=ce1 contenteditable><b>foobar</b>baz</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host1 = document.getElementById("ce1");
var bt1 = host1.childNodes[0].firstChild;
var r1 = document.createRange();
r1.setStart(bt1, 3);
r1.setEnd(bt1, 3);
getSelection().removeAllRanges();
getSelection().addRange(r1);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce1")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "<b>foo</b><div><b>bar</b>baz</div>"),
            "① b 壳内 caret 拆分须壳闭合+新块重开壳链（<b>foo</b><div><b>bar</b>baz</div>），实际: {found:?}"
        );
    }
}

#[ignore]
#[test]
fn test_insertparagraph_caret_in_span_shell_split_m1() {
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
        "<html><body><div id=ce2 contenteditable><span>foobar</span>baz</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host2 = document.getElementById("ce2");
var st2 = host2.childNodes[0].firstChild;
var r2 = document.createRange();
r2.setStart(st2, 3);
r2.setEnd(st2, 3);
getSelection().removeAllRanges();
getSelection().addRange(r2);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce2")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "<span>foo</span><div><span>bar</span>baz</div>"),
            "② span 壳内 caret 拆分须壳闭合+新块重开（<span>foo</span><div><span>bar</span>baz</div>），实际: {found:?}"
        );
    }
}

#[ignore]
#[test]
fn test_insertparagraph_shell_sel_empty_drop_m1() {
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
        "<html><body><div id=ce3 contenteditable>foo<span style=\"color:#aBcDeF\">bar</span>baz</div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 选区全含壳内容（[bar]）：删除 + 拆分，空 style 壳丢弃（Chrome 录音：
    // <p>foo<span>[bar]</span>baz → <p>foo</p><p>baz</p>，div 宿主同语义）。
    sandbox
        .execute(
            r##"
var host3 = document.getElementById("ce3");
var bt3 = host3.childNodes[1].firstChild;
var r3 = document.createRange();
r3.setStart(bt3, 0);
r3.setEnd(bt3, 3);
getSelection().removeAllRanges();
getSelection().addRange(r3);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce3")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "foo<div>baz</div>"),
            "③ 壳全含选区拆分须空壳丢弃（foo<div>baz</div>），实际: {found:?}"
        );
    }
}
