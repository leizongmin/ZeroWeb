// indent F 族 li 尾 br 尾位持行刀钉测（2026-11-18）：块尾 br 前走出块爬升后
// 撞自在块边界被判冗余，wrap 尾清理摘持行 br——extraneous 判定加尾位持行
// 豁免（前邻可见节点是 br 或含块边界＝隐 br 塌高，vendored :840 height
// 测量语义的免布局近似）。三形 li 内选区（元素端点 (li,2)-(li,3)、文本
// (li,0)-(li,1)、整 li (li,0)-(li,3)）收敛同一整 li 包裹形双 br 全留（语料
// indent :802 Chrome 录形）＋两枚控制键：无尾 br 基线形＋前邻可见文本形
// （p 尾单 br 只止行，摘除不塌高——Chrome 摘，v1 门禁 L1 回归回填）。
#[test]
fn test_indent_li_tail_br_pins() {
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
        "<html><body><div id=ce142 contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
globalThis.__mg = [];
function rng(a, ao, b, bo) {
  var r = document.createRange();
  r.setStart(a, ao);
  r.setEnd(b, bo);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
}
function pin(tag, html, setup) {
  var h = document.getElementById("ce142");
  h.innerHTML = html;
  setup();
  document.execCommand("stylewithcss", false, "false");
  var ret = document.execCommand("indent", false, "");
  __mg.push({ tag: tag, ret: ret, html: h.innerHTML });
}
var HTML = '<ul><li>a<br><br></li><li>b</li></ul>';
// 目标键（语料 indent :802）：li1 尾选区 {<br>}（元素端点 (li,2)-(li,3)）
pin("p802-tail-br", HTML, function () {
  var h = document.getElementById("ce142");
  var li = h.childNodes[0].childNodes[0];
  rng(li, 2, li, 3);
});
// 对照 A：选区缩到 li 内文本（(li,0)-(li,1)）
pin("pA-text", HTML, function () {
  var h = document.getElementById("ce142");
  var li = h.childNodes[0].childNodes[0];
  rng(li, 0, li, 1);
});
// 对照 B：整 li 端点 (li,0)-(li,3)
pin("pB-whole-li", HTML, function () {
  var h = document.getElementById("ce142");
  var li = h.childNodes[0].childNodes[0];
  rng(li, 0, li, 3);
});
// 控制键：无尾 br 基线形（(li,0)-(li,1)）
pin("pC-no-br", '<ul><li>a</li><li>b</li></ul>', function () {
  var h = document.getElementById("ce142");
  var li = h.childNodes[0].childNodes[0];
  rng(li, 0, li, 1);
});
// 控制键 D（v1 门禁 L1 回归回填）：p 尾单 br 前邻可见文本＝br 仅止该行，
// 摘除不塌高须摘（语料 indent `<p>foo<br>{}</p><p>extra` 锚形，Chrome 录形
// blockquote 内无尾 br）
pin("pD-prev-text", '<p>foo<br></p><p>extra</p>', function () {
  var h = document.getElementById("ce142");
  var p = h.childNodes[0];
  rng(p, 2, p, 2);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // 三形收敛：整 li 外包裹单壳，li1 双 br 全留（Chrome 录形＝语料 :802 期望）
    let expect_tail = r#"\"html\":\"<ul><ul><li>a<br><br></li></ul><li>b</li></ul>\""#;
    assert!(
        snap_str.contains(r#"\"tag\":\"p802-tail-br\",\"ret\":true,"#) && snap_str.contains(expect_tail),
        "p802 须整 li 包裹且尾 br 双存: {}",
        snap_str
    );
    assert!(
        snap_str.contains(r#"\"tag\":\"pA-text\",\"ret\":true,"#) && snap_str.contains(expect_tail),
        "pA 文本选区须同收敛形: {}",
        snap_str
    );
    assert!(
        snap_str.contains(r#"\"tag\":\"pB-whole-li\",\"ret\":true,"#) && snap_str.contains(expect_tail),
        "pB 整 li 选区须同收敛形: {}",
        snap_str
    );
    // 控制键：无尾 br 形包裹面不变
    assert!(
        snap_str.contains(r#"\"tag\":\"pC-no-br\",\"ret\":true,\"html\":\"<ul><ul><li>a</li></ul><li>b</li></ul>\""#),
        "pC 无尾 br 基线形须恒一: {}",
        snap_str
    );
    // 控制键 D：前邻可见文本形须摘尾 br（br 仅止行，blockquote 内无 br）
    assert!(
        snap_str.contains(r#"\"tag\":\"pD-prev-text\",\"ret\":true,\"html\":\"<blockquote><p>foo</p></blockquote><p>extra</p>\""#),
        "pD 前邻可见文本形须摘尾 br: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("<li>a<br></li>"),
        "尾位持行豁免不得回退摘 br 形: {}",
        snap_str
    );
}
