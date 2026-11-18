// indent E-表 tbody 全覆盖整表升级刀钉测（2026-11-18）：行组端点形选区
// （data-start/end 语义 (tbody,0)-(tbody,1)）块扩展止于 (table,0)-(table,1)，
// contained 顶层 tbody 被 wrap 过滤器滤除退化为 cell 文本成员——升级臂按
// 「行组整体含于 newRange 且该表全部行组均覆盖」判据把成员升级为整表
// （Chrome 录形＝整表外包裹单壳，"outermost possible location"——tr/td 壳
// 不可落 tbody 内容模型）。四形：目标键整表升级（语料 indent :140）、表级
// 端点形对照（:147，既有直包面）、tr 部分覆盖负断言（:134，逐 td 壳）、
// cell 文本负断言（:122，单 cell 壳）。
#[test]
fn test_indent_table_tbody_pins() {
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
        "<html><body><div id=ce139 contenteditable></div></body></html>".to_string(),
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
  var h = document.getElementById("ce139");
  h.innerHTML = html;
  setup();
  document.execCommand("stylewithcss", false, "false");
  var ret = document.execCommand("indent", false, "");
  __mg.push({ tag: tag, ret: ret, html: h.innerHTML });
}
var HTML = '<table><tbody><tr><td>foo<td>bar<td>baz</table><p>extra';
// 目标键（语料 :140 tbody 端点形）：行组全覆盖 → 整表外包裹单壳
pin("t140-tbody", HTML, function () {
  var h = document.getElementById("ce139");
  var tb = h.childNodes[0].childNodes[0];
  rng(tb, 0, tb, 1);
});
// 对照（语料 :147 table 端点形）：table 直在 contained，既有整包面不变
pin("t147-table", HTML, function () {
  var h = document.getElementById("ce139");
  var t = h.childNodes[0];
  rng(t, 0, t, 1);
});
// 负断言（语料 :134 tr 部分覆盖形）：行组未全含 → 逐 td 壳维持
pin("t134-tr-partial", HTML, function () {
  var h = document.getElementById("ce139");
  var tr = h.childNodes[0].childNodes[0].childNodes[0];
  rng(tr, 0, tr, 2);
});
// 负断言（语料 :122 cell 文本形）：单 cell 文本 → 单 cell 壳维持
pin("t122-cell-text", HTML, function () {
  var h = document.getElementById("ce139");
  var td = h.childNodes[0].childNodes[0].childNodes[0].childNodes[1];
  var t = td.childNodes[0];
  rng(t, 1, t, 2);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // 目标键：整表外包裹单壳（升级后与 :147 同态）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t140-tbody\",\"ret\":true,\"html\":\"<blockquote><table><tbody><tr><td>foo</td><td>bar</td><td>baz</td></tr></tbody></table></blockquote><p>extra</p>\""#
        ),
        "t140 须产整表外包裹单壳形: {}",
        snap_str
    );
    // 对照：table 端点直包形不变
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t147-table\",\"ret\":true,\"html\":\"<blockquote><table><tbody><tr><td>foo</td><td>bar</td><td>baz</td></tr></tbody></table></blockquote><p>extra</p>\""#
        ),
        "t147 须维持 table 端点直包形: {}",
        snap_str
    );
    // 负断言：tr 部分覆盖逐 td 壳、无整表包裹
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t134-tr-partial\",\"ret\":true,\"html\":\"<table><tbody><tr><td><blockquote>foo</blockquote></td><td><blockquote>bar</blockquote></td><td>baz</td></tr></tbody></table><p>extra</p>\""#
        ),
        "t134 须维持逐 td 壳形: {}",
        snap_str
    );
    // 负断言：cell 文本单 cell 壳
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t122-cell-text\",\"ret\":true,\"html\":\"<table><tbody><tr><td>foo</td><td><blockquote>bar</blockquote></td><td>baz</td></tr></tbody></table><p>extra</p>\""#
        ),
        "t122 须维持单 cell 壳形: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("<blockquote><table><tbody><tr><td><blockquote>"),
        "部分覆盖形不得触发整表升级: {}",
        snap_str
    );
}
