// 编辑命令兼容 ip 206 M1 主桶探针（2026-11-13，DC-4 div/p 容器子族 census
// ip-divp-census-mech.txt M1 三子形）——insertParagraph 拆分边界 inline
// formatting element 复制/丢弃语义钉测（ip-M1 刀收口转正常驻）。
//
// Chrome 实录（e1-ip.log r2 post @ffdcd28af，div contenteditable 宿主，
// defaultparagraphseparator=div 前置，同语料 runConformanceTest setupDiv 形）：
// ① M1b 壳内 caret 拆分：<b>foobar</b>baz caret (text,3) →
//   <div><b>foo</b></div><div><b>bar</b>baz</div>（双包裹形：左块壳链闭合、
//   新块重开壳链、壳外尾随文本随新块）。修复 = ipara 非 SLC 臂壳链窄门直构
//   （cloneNode(false)/createTextNode/先摘后挂，绕开 wrap+extract 管线在
//   wrap 生成块上的序列化分叉）。
// ② M1c span 壳同型：<span>foobar</span>baz caret (text,3) →
//   <div><span>foo</span></div><div><span>bar</span>baz</div>。
// ③ M1a 非折叠壳全含选区：foo<span style>[bar]</span>baz 选区 [bar] →
//   <div>foo</div><div>baz</div>（选区删除拆分后**空壳丢弃**）。目标为外推
//   形（census 12键 span-style 选区簇 p 宿主实录 `<p>foo</p><p>baz</p>` 同
//   语义，div 宿主 setupDiv 形无逐字实录行）。
// 断言面：沙箱 execute 直读最终宿主 innerHTML 恰等（compare innerHTML 同源
// parser）。窄门宿主落位经 wired write（直构块序列化后一次 innerHTML= 整域
// 重排，SetInnerHtml 单域契约）：直构含 parse 家族原节点先摘后挂 move，细粒
// 度 mutation 流不可表达 plain move（摘除有 RC 记录、重挂无记录），宿主
// base+overlay 视图滞留旧位且 apply 镜像丢节点——直挂 append 版桥级实证
// `baz` 残留宿主首位（DBG2：直构块 outerHTML 本身恰等，唯宿主视图融合滞
// 留）。占位 br 标记随重解析丢失由门内写后重打补回（A3 typing 消费面）。
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
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce1\").innerHTML")
            .unwrap()
            .value,
        "<div><b>foo</b></div><div><b>bar</b>baz</div>",
        "① b 壳内 caret 拆分须双包裹形（左块壳链闭合+新块重开壳链+壳外尾随文本随新块）"
    );
}

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
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce2\").innerHTML")
            .unwrap()
            .value,
        "<div><span>foo</span></div><div><span>bar</span>baz</div>",
        "② span 壳内 caret 拆分须双包裹形（左块壳链闭合+新块重开壳链+壳外尾随文本随新块）"
    );
}

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
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce3\").innerHTML")
            .unwrap()
            .value,
        "<div>foo</div><div>baz</div>",
        "③ 壳全含选区拆分须空壳丢弃（左块纯文本+新块纯文本）"
    );
}
