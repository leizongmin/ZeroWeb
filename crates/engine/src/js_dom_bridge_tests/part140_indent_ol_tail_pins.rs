// indent C 族 ol 域尾区刀钉测（2026-11-18）：双刀面——①items normalize 尾区
// 门（选区两端整体落在 item 最后一个 ol/ul 直接子之后的尾区→跳过 normalize，
// spec else 臂不拆 li）；②li 单项锚定收口（选区两端同锚最近 li → 块扩展收口
// 该 li 列表父界，spec 尾爬升不越列表宿主）。四形：目标键整包（语料 indent
// `<ol><li>foo<ol><li>bar</ol>[baz]</ol>`）、前导文本 normalize 照跑合并形
// （`<ol><li>[foo]<ol><li>bar</ol>baz</ol>`）、跨 li 选区 bq 整包形
// （`<ol><li>[foo<ol><li>bar</ol><li>baz]</ol>`）、内层 li 同锚新 ol 嵌套形
// （`<ol><li>foo<ol><li>[bar]</ol>baz</ol>`）。
#[test]
fn test_indent_ol_tail_pins() {
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
function findText(root, data, out) {
  (function walk(n) {
    if (out.node) return;
    if (n.nodeType === 3 && n.data === data) { out.node = n; return; }
    for (var c = n.firstChild; c; c = c.nextSibling) walk(c);
  })(root);
}
// 目标键（语料尾区形）：baz 选区整体在 inner ol 之后 → li#1 整包新嵌套 ol
pin("t-tail-baz",
    '<ol><li>foo<ol><li>bar</ol>baz</ol>', function () {
  var h = document.getElementById("ce139");
  var o = { node: null };
  findText(h, "baz", o);
  rng(o.node, 0, o.node, 3);
});
// 前导文本形（同树）：foo 选区在 inner ol 之前 → normalize 照跑（拆分+合并形）
pin("t-lead-foo-inner",
    '<ol><li>foo<ol><li>bar</ol>baz</ol>', function () {
  var h = document.getElementById("ce139");
  var t = h.childNodes[0].childNodes[0].childNodes[0];
  rng(t, 0, t, 3);
});
// 跨 li 形（语料 `<ol><li>[foo<ol><li>bar]</ol><li>baz</ol>`）：start/end 异 li
// → 不收口，多子列表形嵌套包（不 bq）
pin("a-cross-inner",
    '<ol><li>foo<ol><li>bar</ol><li>baz</ol>', function () {
  var h = document.getElementById("ce139");
  var o = { node: null };
  findText(h, "bar", o);
  rng(h.childNodes[0].childNodes[0].childNodes[0], 0, o.node, 3);
});
// 内层 li 同锚形（语料 `<ol><li>foo<ol><li>[bar]</ol>baz</ol>`）：normalize
// hoist 后同锚收口，新 ol 包 li#2 于 inner ol 内
pin("t-inner-bar",
    '<ol><li>foo<ol><li>bar</ol>baz</ol>', function () {
  var h = document.getElementById("ce139");
  var o = { node: null };
  findText(h, "bar", o);
  rng(o.node, 0, o.node, 3);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // 目标键：li#1 整包新嵌套 ol（baz 并首 li 尾），无 bq 整包
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t-tail-baz\",\"ret\":true,\"html\":\"<ol><ol><li>foo<ol><li>bar</li></ol>baz</li></ol></ol>\""#
        ),
        "t-tail 须产 li#1 整包新嵌套 ol 形: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("<blockquote><ol>"),
        "t-tail/a-cross 不得产 bq 整包形: {}",
        snap_str
    );
    // 前导形：normalize 照跑合并形（foo+bar 单 ol 并邻）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t-lead-foo-inner\",\"ret\":true,\"html\":\"<ol><ol><li>foo</li><li>bar</li></ol><li>baz</li></ol>\""#
        ),
        "t-lead 须产 normalize 照跑合并形: {}",
        snap_str
    );
    // 跨 li 形：嵌套包不 bq
    assert!(
        snap_str.contains(
            r#"\"tag\":\"a-cross-inner\",\"ret\":true,\"html\":\"<ol><ol><li>foo</li><ol><li>bar</li></ol></ol><li>baz</li></ol>\""#
        ),
        "a-cross 须产嵌套包形（不 bq）: {}",
        snap_str
    );
    // 内层形：新 ol 嵌 inner ol 内
    assert!(
        snap_str.contains(
            r#"\"tag\":\"t-inner-bar\",\"ret\":true,\"html\":\"<ol><li>foo</li><ol><ol><li>bar</li></ol></ol><li>baz</li></ol>\""#
        ),
        "t-inner 须产 inner 内新 ol 形: {}",
        snap_str
    );
}
