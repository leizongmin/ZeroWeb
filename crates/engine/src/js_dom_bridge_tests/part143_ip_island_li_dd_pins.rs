// ip I4 族 CE-island host 本体 li/dd 拆分钉测（2026-11-18）：caret 在 host
// 本体 li/dd（最近 CE≠false = 自身）直子文本中点——Chrome 录音内容拆 div
// 块内嵌 li（dps=div 显式，语料 insertparagraph.js :1996/:2008 `<li
// contenteditable><div>ab</div><div>cd</div></li>` 形）而非同标签兄弟拆分。
// 控制键：h1/dt host 本体退化行断 ab<br>cd（语料 :1992/:2002 绿形）、span
// 行内宿主 ab<br>cd（:1972 绿形）、ul 宿主普通列表同标签兄弟拆分（:1438
// 绿形 `<li>foo</li><li><br></li></ul>`）、dps 未设 island 面产线缺省 'p'
//（本仓双面制语义锚，自定回归基线非语料录音）。
#[test]
fn test_ip_island_li_dd_pins() {
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
        "<html><body><div id=ce143></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
globalThis.__mg = [];
function rngText(t, o) {
  var r = document.createRange();
  r.setStart(t, o);
  r.setEnd(t, o);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
}
function rngEnd(el) {
  var t = el.lastChild;
  var r = document.createRange();
  r.setStart(t, t.nodeValue.length);
  r.setEnd(t, t.nodeValue.length);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
}
function pin(tag, html, setup) {
  var h = document.getElementById("ce143");
  h.innerHTML = html;
  setup();
  var ret = document.execCommand("insertparagraph", false, "");
  __mg.push({ tag: tag, ret: ret, html: h.innerHTML });
}
// 语料 :1996 前置（dps 未设面）+ :1438（ul 宿主，dps 未设）先行：
// __zwDefaultParagraphSeparator 全局随 pin 泄漏，未设面排最前。
pin("p-unset-li", '<ol contenteditable=false><li contenteditable>abcd</li></ol>', function () {
  var t = document.getElementById("ce143").childNodes[0].childNodes[0].childNodes[0];
  rngText(t, 2);
});
pin("p-ul-host", '<ul contenteditable><li>foo</li></ul>', function () {
  var li = document.getElementById("ce143").childNodes[0].childNodes[0];
  rngEnd(li);
});
document.execCommand("defaultparagraphseparator", false, "div");
// 目标键（语料 :1996）：li host 本体中点 caret → div 双块内嵌
pin("p-li-i4", '<ol contenteditable=false><li contenteditable>abcd</li></ol>', function () {
  var t = document.getElementById("ce143").childNodes[0].childNodes[0].childNodes[0];
  rngText(t, 2);
});
// 目标键（语料 :2008）：dd host 本体同形
pin("p-dd-i4", '<dl contenteditable=false><dd contenteditable>abcd</dd></dl>', function () {
  var t = document.getElementById("ce143").childNodes[0].childNodes[0].childNodes[0];
  rngText(t, 2);
});
// 控制键（语料 :1992 绿形）：h1 host 本体退化行断
pin("p-h1-ctrl", '<div contenteditable=false><h1 contenteditable>abcd</h1></div>', function () {
  var t = document.getElementById("ce143").childNodes[0].childNodes[0].childNodes[0];
  rngText(t, 2);
});
// 控制键（语料 :2002 绿形）：dt host 本体退化行断
pin("p-dt-ctrl", '<dl contenteditable=false><dt contenteditable>abcd</dt></dl>', function () {
  var t = document.getElementById("ce143").childNodes[0].childNodes[0].childNodes[0];
  rngText(t, 2);
});
// 控制键（语料 :1972 绿形）：span 行内宿主退化行断（block=li≠host 不入门）
pin("p-span-ctrl", '<ol contenteditable=false><li><span contenteditable>abcd</span></li></ol>', function () {
  var t = document.getElementById("ce143").childNodes[0].childNodes[0].childNodes[0].childNodes[0];
  rngText(t, 2);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // 目标键：div 双块内嵌（Chrome 录形＝语料 :1996/:2008 exp 首形）
    assert!(
        snap_str.contains(r#"\"tag\":\"p-li-i4\",\"ret\":true,\"html\":\"<ol contenteditable=\\\"false\\\"><li contenteditable=\\\"\\\"><div>ab</div><div>cd</div></li></ol>\""#),
        "p-li 须 div 双块内嵌: {}",
        snap_str
    );
    assert!(
        snap_str.contains(r#"\"tag\":\"p-dd-i4\",\"ret\":true,\"html\":\"<dl contenteditable=\\\"false\\\"><dd contenteditable=\\\"\\\"><div>ab</div><div>cd</div></dd></dl>\""#),
        "p-dd 须 div 双块内嵌: {}",
        snap_str
    );
    // dps 未设面：产线缺省 'p' 双块（双面制语义锚）
    assert!(
        snap_str.contains("<p>ab</p><p>cd</p>"),
        "p-unset-li 缺省 'p' 双块: {}",
        snap_str
    );
    // 控制键：h1/dt/span 退化行断形恒一
    assert!(
        snap_str.contains(r#"<h1 contenteditable=\\\"\\\">ab<br>cd</h1>"#),
        "p-h1 须退化行断: {}",
        snap_str
    );
    assert!(
        snap_str.contains(r#"<dt contenteditable=\\\"\\\">ab<br>cd</dt>"#),
        "p-dt 须退化行断: {}",
        snap_str
    );
    assert!(
        snap_str.contains(r#"<span contenteditable=\\\"\\\">ab<br>cd</span>"#),
        "p-span 须退化行断: {}",
        snap_str
    );
    // 控制键：ul 宿主普通列表同标签兄弟拆分恒一（语料 :1438 形）
    assert!(
        snap_str.contains(r#"<ul contenteditable=\\\"\\\"><li>foo</li><li><br></li></ul>"#),
        "p-ul-host 须同标签兄弟拆分: {}",
        snap_str
    );
}
