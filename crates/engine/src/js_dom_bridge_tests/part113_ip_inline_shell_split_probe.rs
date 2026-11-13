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

// ===== IP-M1 刀 v2 门禁 P2F 三族回归钉测（2026-11-14，语料 got/expect 逐字
// 录自 batch-ipm1-gate-v2 L1 judge：A=a×b 嵌套/piece-3 双裹、B=壳首 caret+
// 前导裸文本、C=带 attr 壳 mid-caret id 剥离）——修复刀同提交转正常驻。====

// ④ B 族：壳首 caret+前导裸文本（语料 `foo<b>[]bar</b>` dps=div）
// EXP `<div>foo</div><div><b>bar</b></div>`，v2 got `<div>foo<b>bar</b></div><div><br></div>`。
#[test]
fn test_insertparagraph_shell_start_caret_bare_text_m1b2() {
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
        "<html><body><div id=ce4 contenteditable>foo<b>bar</b></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host4 = document.getElementById("ce4");
var bt4 = host4.childNodes[1].firstChild;
var r4 = document.createRange();
r4.setStart(bt4, 0);
r4.setEnd(bt4, 0);
getSelection().removeAllRanges();
getSelection().addRange(r4);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce4\").innerHTML")
            .unwrap()
            .value,
        "<div>foo</div><div><b>bar</b></div>",
        "④ 壳首 caret+前导裸文本：裸文本留左块、壳整体随右块（Chrome 实录）"
    );
}

// ⑤ C 族：带 attr 壳 mid-caret（语料 `<b id=x class=y>foo[]bar</b>` swcss=true dps=div）
// EXP `<div><b id="x" class="y">foo</b></div><div><b class="y">bar</b></div>`（右壳剥 id）。
#[test]
fn test_insertparagraph_attr_shell_mid_caret_id_strip_m1c2() {
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
        "<html><body><div id=ce5 contenteditable><b id=x class=y>foobar</b></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host5 = document.getElementById("ce5");
var bt5 = host5.childNodes[0].firstChild;
var r5 = document.createRange();
r5.setStart(bt5, 3);
r5.setEnd(bt5, 3);
getSelection().removeAllRanges();
getSelection().addRange(r5);
document.execCommand("stylewithcss", false, "true");
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce5\").innerHTML")
            .unwrap()
            .value,
        "<div><b id=\"x\" class=\"y\">foo</b></div><div><b class=\"y\">bar</b></div>",
        "⑤ 带 attr 壳 mid-caret：右壳重开剥 id 保 class（Chrome 实录）"
    );
}

// ⑥ A 族 mid：p 内 a>b 嵌套壳中点（语料 `<p><a href=foo><b>foo[]bar</b></a></p>` dps=div）
// EXP `<p><a href="foo"><b>foo</b></a></p><p><a href="foo"><b>bar</b></a></p>`（双侧全链重开含 a）。
// 断言双面（2026-11-14 桥级定谳）：主拆分支左半裁切走 extract
// _zwIparaDeleteData → parsed 文本 wrapper 写经 SetChildText mutation 落宿
// 主真树（bridge sandbox 只见日志不落 JS 树——t1 诊断 deleteData 后 nv 读
// foo 而宿主序列化仍 foobar 实证；apply_mutations_to_html 一次性重放丢
// piece-3 新建文本 `<b></b>`，与 corpus persistent 应用语义不同面，不作镜
// 像）。故结构面 = 沙箱内右块（cloneSlice 新建文本 + piece-3 克隆直写序列
// 化可见），发射面 = SetChildText 日志（左半裁切 foo，corpus 终态由门禁批
// L1 ip 腿 judge 对账）。
#[test]
fn test_insertparagraph_nested_ab_shell_mid_caret_m1a4() {
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
        "<html><body><div id=ce6 contenteditable><p><a href=foo><b>foobar</b></a></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host6 = document.getElementById("ce6");
var bt6 = host6.childNodes[0].firstChild.firstChild.firstChild;
var r6 = document.createRange();
r6.setStart(bt6, 3);
r6.setEnd(bt6, 3);
getSelection().removeAllRanges();
getSelection().addRange(r6);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute(concat!(
                "document.getElementById(\"ce6\").childNodes[1]",
                " && document.getElementById(\"ce6\").childNodes[1].innerHTML",
            ))
            .unwrap()
            .value,
        "<a href=\"foo\"><b>bar</b></a>",
        "⑥ 结构面：新块壳链重开含 a 层（cloneSlice 新建文本入链，guard 免双裹）"
    );
    let m = mutations.lock().unwrap().clone();
    assert!(
        m.iter().any(|mm| matches!(mm,
            DomMutation::SetChildText { text, .. } if text == "foo")),
        "⑥ 发射面：左半裁切须发 SetChildText(foo)（extract deleteData 落日志面）"
    );
}

// ⑦ A 族 end：p 内 a>b 嵌套壳尾 caret 无后随（语料 `<p><a href=foo><b>foo&nbsp;[]</b></a></p>` dps=div）
// EXP `<p><a href="foo"><b>foo&nbsp;</b></a></p><p><br></p>`（新块裸 br，壳链不重开）。
#[test]
fn test_insertparagraph_nested_ab_shell_end_caret_bare_br_m1a5() {
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
        "<html><body><div id=ce7 contenteditable><p><a href=foo><b>foo&nbsp;</b></a></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host7 = document.getElementById("ce7");
var bt7 = host7.childNodes[0].firstChild.firstChild.firstChild;
var r7 = document.createRange();
r7.setStart(bt7, 4);
r7.setEnd(bt7, 4);
getSelection().removeAllRanges();
getSelection().addRange(r7);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce7\").innerHTML")
            .unwrap()
            .value,
        "<p><a href=\"foo\"><b>foo&nbsp;</b></a></p><p><br></p>",
        "⑦ p 内 a>b 嵌套壳尾 caret（无后随内容）：新块裸 br 不重开壳链"
    );
}

// ⑧ A 族裸壳：a 壳 mid-caret（语料 `<a href=foo>foo[]bar</a>` dps=div）
// EXP `<div><a href="foo">foo</a></div><div><a href="foo">bar</a></div>`
//（a 层双侧恒重开，v2 右内容上抛裸文本形 P2F）。
#[test]
fn test_insertparagraph_bare_a_shell_mid_caret_m1a1() {
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
        "<html><body><div id=ce8 contenteditable><a href=foo>foobar</a></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host8 = document.getElementById("ce8");
var bt8 = host8.childNodes[0].firstChild;
var r8 = document.createRange();
r8.setStart(bt8, 3);
r8.setEnd(bt8, 3);
getSelection().removeAllRanges();
getSelection().addRange(r8);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce8\").innerHTML")
            .unwrap()
            .value,
        "<div><a href=\"foo\">foo</a></div><div><a href=\"foo\">bar</a></div>",
        "⑧ 裸 a 壳 mid-caret：a 层双侧恒重开（内容随侧，href 保留）"
    );
}

// ===== IP-M1 刀 v3.1 壳尾重开判据表钉测（2026-11-14，v3 首版存活判据回归
// 14 键 corpus 三方对照重定谳：纯壳尾 fmt/span 重开 br 入链、a 链 nbsp 尾裸
// br、a 链纯文本尾重开减 a）——got/expect 逐字录自语料实录。====

// ⑨ 壳尾重开（无 a 链）：语料 `<p><b>foo[]</b></p>` dps=div
// EXP `<p><b>foo</b></p><p><b><br></b></p>`（b 壳重开，br 入壳内首）。
#[test]
fn test_insertparagraph_b_shell_end_caret_reopen_m1r1() {
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
        "<html><body><div id=ce9 contenteditable><p><b>foo</b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host9 = document.getElementById("ce9");
var bt9 = host9.childNodes[0].firstChild.firstChild;
var r9 = document.createRange();
r9.setStart(bt9, 3);
r9.setEnd(bt9, 3);
getSelection().removeAllRanges();
getSelection().addRange(r9);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce9\").innerHTML")
            .unwrap()
            .value,
        "<p><b>foo</b></p><p><b><br></b></p>",
        "⑨ 壳尾 caret 无 a 链：fmt 壳重开 br 入链内首（corpus 实录）"
    );
}

// ⑩ a 链壳尾 nbsp（重开禁断）：语料 `<p><b><a href=foo>foo&nbsp;[]</a></b></p>` dps=div
// EXP `<p><b><a href="foo">foo&nbsp;</a></b></p><p><br></p>`（新块裸 br 不重开——
// v3.2 结构判：壳链 walk 未过 a 层，与文本面 nbsp 无关）。
#[test]
fn test_insertparagraph_ba_chain_nbsp_end_caret_bare_m1r2() {
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
        "<html><body><div id=ce10 contenteditable><p><b><a href=foo>foo&nbsp;</a></b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host10 = document.getElementById("ce10");
var bt10 = host10.childNodes[0].firstChild.firstChild.firstChild;
var r10 = document.createRange();
r10.setStart(bt10, 4);
r10.setEnd(bt10, 4);
getSelection().removeAllRanges();
getSelection().addRange(r10);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce10\").innerHTML")
            .unwrap()
            .value,
        "<p><b><a href=\"foo\">foo&nbsp;</a></b></p><p><br></p>",
        "⑩ a 链壳尾 nbsp：不重开壳链（corpus 实录，b 在 a 内层不重开）"
    );
}

// ⑪ a 链壳尾纯文本（重开减 a）：语料 `<p><b><a href=foo>foo[]</a></b></p>` dps=div
// EXP `<p><b><a href="foo">foo</a></b></p><p><b><br></b></p>`（外层 b 重开，a 不重开）。
#[test]
fn test_insertparagraph_ba_chain_plain_end_caret_outer_reopen_m1r3() {
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
        "<html><body><div id=ce11 contenteditable><p><b><a href=foo>foo</a></b></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host11 = document.getElementById("ce11");
var bt11 = host11.childNodes[0].firstChild.firstChild.firstChild;
var r11 = document.createRange();
r11.setStart(bt11, 3);
r11.setEnd(bt11, 3);
getSelection().removeAllRanges();
getSelection().addRange(r11);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce11\").innerHTML")
            .unwrap()
            .value,
        "<p><b><a href=\"foo\">foo</a></b></p><p><b><br></b></p>",
        "⑪ a 链壳尾纯文本：外层 fmt 壳重开减 a（corpus 实录，b 在 a 外层重开）"
    );
}

// ⑫ a 外层壳尾（v3.2 gate P2F 对账钉测）：语料 `<p><a href=foo><b>foo[]</b></a></p>`
// dps=div EXP `<p><a href="foo"><b>foo</b></a></p><p><br></p>`（新块裸 br 不重开——
// b 在 a 内层；v3.1 \xa0 文本判漏此纯文本形，gate v3 P2F 2 键实录）。
#[test]
fn test_insertparagraph_ab_chain_plain_end_caret_bare_m1r4() {
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
        "<html><body><div id=ce12 contenteditable><p><a href=foo><b>foo</b></a></p></div>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var host12 = document.getElementById("ce12");
var bt12 = host12.childNodes[0].firstChild.firstChild.firstChild;
var r12 = document.createRange();
r12.setStart(bt12, 3);
r12.setEnd(bt12, 3);
getSelection().removeAllRanges();
getSelection().addRange(r12);
document.execCommand("defaultparagraphseparator", false, "div");
document.execCommand("insertparagraph", false, "");
"##,
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("document.getElementById(\"ce12\").innerHTML")
            .unwrap()
            .value,
        "<p><a href=\"foo\"><b>foo</b></a></p><p><br></p>",
        "⑫ a 外层壳尾纯文本：裸 br 不重开（corpus 实录，b 在 a 内层）"
    );
}
