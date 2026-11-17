// indent 容器整体升级刀钉测（2026-11-18 B 族刀）：blockExtend do-while 两侧
// 同断于块容器内侧边界（首尾子皆可见块）时 contained 收容器子代＝新壳嵌
// 容器内（语料 indent B 族 4 键＋E 族 div 键 got 实证）。升级判据＝容器全
// 部子节点入选区 contained 集 → 容器整体入含域（新壳包外，Chrome 录形
// "outermost possible location"）；部分覆盖（子代残留在列外）不升级＝嵌内
// 保持（indent.html 现绿键）。四形：styled bq 整壳（L514）、div 整壳（E1
// L89）、webkit bq 整壳（L719）、styled bq 部分覆盖负断言（条目 1 现绿形）。
#[test]
fn test_indent_container_hoist_pins() {
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
        "<html><body><div id=ce138a contenteditable></div></body></html>".to_string(),
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
  var h = document.getElementById("ce138a");
  h.innerHTML = html;
  setup();
  document.execCommand("stylewithcss", false, "false");
  var ret = document.execCommand("indent", false, "");
  __mg.push({ tag: tag, ret: ret, html: h.innerHTML });
}
// L514 整壳升级：styled bq 双 p 全选 → 新裸 bq 包 C 外
pin("styled-whole",
    '<blockquote style="margin-right: 0" dir="ltr"><p>foobar</p><p>baz</p></blockquote><p>extra</p>',
    function () {
  var bq = document.getElementById("ce138a").childNodes[0];
  rng(bq.childNodes[0].childNodes[0], 3, bq.childNodes[1].childNodes[0], 1);
});
// E1 (L89) 整壳升级：div 双 p（dir=rtl）全选 → 新裸 bq 包 div 外
pin("div-whole",
    '<div><p>Foo barbaz</p><p dir="rtl">Foo barbaz</p></div><p>Extra</p>',
    function () {
  var dv = document.getElementById("ce138a").childNodes[0];
  rng(dv.childNodes[0].childNodes[0], 3, dv.childNodes[1].childNodes[0], 3);
});
// L719 整壳升级：webkit-indent-blockquote 双 p 全选 → 新裸 bq 包壳外
pin("webkit-whole",
    '<blockquote class="webkit-indent-blockquote" style="margin: 0 0 0 40px; border: none; padding: 0px"><p>foobar</p><p>baz</p></blockquote><p>extra</p>',
    function () {
  var bq = document.getElementById("ce138a").childNodes[0];
  rng(bq.childNodes[0].childNodes[0], 3, bq.childNodes[1].childNodes[0], 1);
});
// 部分覆盖负断言（现绿形）：styled bq 内只选 p1、p2 残留 → 不升级＝嵌内
pin("partial-cover",
    '<blockquote style="margin-right: 0" dir="ltr"><p>foobar</p><p>baz</p></blockquote><p>extra</p>',
    function () {
  var bq = document.getElementById("ce138a").childNodes[0];
  var t = bq.childNodes[0].childNodes[0];
  rng(t, 3, t, 6);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // styled-whole：新裸 bq 包 styled bq 外（属性空格为语料归一化面）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"styled-whole\",\"ret\":true,\"html\":\"<blockquote><blockquote style=\\\"margin-right: 0\\\" dir=\\\"ltr\\\"><p>foobar</p><p>baz</p></blockquote></blockquote><p>extra</p>\""#
        ),
        "styled-whole 须产整壳外包裹: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("<blockquote style=\\\"margin-right: 0\\\" dir=\\\"ltr\\\"><blockquote><p>foobar</p><p>baz</p></blockquote></blockquote>"),
        "styled-whole 不得残留壳内嵌套形: {}",
        snap_str
    );
    // div-whole：新裸 bq 包 div 外
    assert!(
        snap_str.contains(
            r#"\"tag\":\"div-whole\",\"ret\":true,\"html\":\"<blockquote><div><p>Foo barbaz</p><p dir=\\\"rtl\\\">Foo barbaz</p></div></blockquote><p>Extra</p>\""#
        ),
        "div-whole 须产整壳外包裹: {}",
        snap_str
    );
    // webkit-whole：新裸 bq 包 webkit 壳外
    assert!(
        snap_str.contains(
            r#"\"tag\":\"webkit-whole\",\"ret\":true,\"html\":\"<blockquote><blockquote class=\\\"webkit-indent-blockquote\\\""#
        ),
        "webkit-whole 须产整壳外包裹: {}",
        snap_str
    );
    // partial-cover：嵌内保持（升级负例）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"partial-cover\",\"ret\":true,\"html\":\"<blockquote style=\\\"margin-right: 0\\\" dir=\\\"ltr\\\"><blockquote><p>foobar</p></blockquote><p>baz</p></blockquote><p>extra</p>\""#
        ),
        "partial-cover 须保持壳内嵌套（不升级）: {}",
        snap_str
    );
}
