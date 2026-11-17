// indent D 族跨空白并邻刀钉测（2026-11-18）：collapsed whitespace 前向走查
// 补全（spec 双向走查后半）→ 块界夹缝 ws 判折叠→invisible→wrap 拉环收 ws→
// simple indentation element 复用臂跨 ws 并邻（spec wrap 注记：非贴邻兄弟合
// criteria 且仅 invisible 介入时跨过它）。四形：D1 前邻 bq+ws+p 选块（语料
// :724）、D2 选块+ws+后邻 bq（:730）、D3 双侧 bq 夹选块单壳三段（:736）、
// A 族文本成员 br 垫界保持守卫（:290，joinBr inline 判据防退）。
#[test]
fn test_indent_ws_merge_pins() {
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
// D1（语料 :724）：bq(foo) ws p2[bar] → p2 连 ws 并入前邻 bq，无独立双壳
pin("d1-prev-merge",
    '<blockquote><p>foo</p></blockquote> <p>bar</p>', function () {
  var h = document.getElementById("ce139");
  var t = h.childNodes[2].childNodes[0];
  rng(t, 0, t, 3);
});
// D2（语料 :730）：p1[foo] ws bq(bar) → p1 连 ws 并入后邻 bq
pin("d2-next-merge",
    '<p>foo</p> <blockquote><p>bar</p></blockquote>', function () {
  var h = document.getElementById("ce139");
  var t = h.childNodes[0].childNodes[0];
  rng(t, 0, t, 3);
});
// D3（语料 :736）：双侧 bq 夹 ws+p2+ws → 单壳三段（composition 级远邻并入）
pin("d3-both-merge",
    '<blockquote><p>foo</p></blockquote> <p>bar</p> <blockquote><p>baz</p></blockquote>',
    function () {
  var h = document.getElementById("ce139");
  var t = h.childNodes[2].childNodes[0];
  rng(t, 0, t, 3);
});
// A 族守卫（语料 :290）：宿主直子文本成员并邻 bq，br 垫界保持（joinBr
// inline 判据两条件恒真），防 D 刀换判据后文本形丢行界
pin("a-text-br-guard",
    '[foo]<blockquote>bar</blockquote><p>extra</p>', function () {
  var h = document.getElementById("ce139");
  var t = h.childNodes[0];
  rng(t, 0, t, 3);
});
// C 族 ws 形守卫（语料 :762，smoke 翻绿实证）：内层 ol 外 ws 贴邻 li 选块，
// baz 并入内层 ol 且 ws 保留（`<li>bar</li> <li>baz</li>`）＝list 域拉环
// 翻转的语料权威形
pin("c-ws-list-merge",
    '<ol><li>foo<ol><li>bar</li></ol></li> <li>[baz]</ol>', function () {
  var h = document.getElementById("ce139");
  var ol = h.childNodes[0];
  var t = ol.childNodes[2].childNodes[0];
  rng(t, 0, t, 3);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // D1：单壳并邻形（exp 首形逐字），无双壳残留
    assert!(
        snap_str.contains(
            r#"\"tag\":\"d1-prev-merge\",\"ret\":true,\"html\":\"<blockquote><p>foo</p> <p>bar</p></blockquote>\""#
        ),
        "d1 须产单壳并邻形: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("</blockquote> <blockquote>"),
        "d1/d3 不得残留独立双壳: {}",
        snap_str
    );
    // D2：对称单壳并邻形
    assert!(
        snap_str.contains(
            r#"\"tag\":\"d2-next-merge\",\"ret\":true,\"html\":\"<blockquote><p>foo</p> <p>bar</p></blockquote>\""#
        ),
        "d2 须产单壳并邻形: {}",
        snap_str
    );
    // D3：单壳三段形
    assert!(
        snap_str.contains(
            r#"\"tag\":\"d3-both-merge\",\"ret\":true,\"html\":\"<blockquote><p>foo</p> <p>bar</p> <p>baz</p></blockquote>\""#
        ),
        "d3 须产单壳三段形: {}",
        snap_str
    );
    // A 守卫：文本成员并邻 br 垫界保持
    assert!(
        snap_str.contains(
            r#"\"tag\":\"a-text-br-guard\",\"ret\":true,\"html\":\"<blockquote>[foo]<br>bar</blockquote><p>extra</p>\""#
        ),
        "a-text 须保持 br 垫界: {}",
        snap_str
    );
    // C-ws 守卫：baz 并入内层 ol 且 ws 保留
    assert!(
        snap_str.contains(
            r#"\"tag\":\"c-ws-list-merge\",\"ret\":true,\"html\":\"<ol><li>foo</li><ol><li>bar</li> <li>[baz]</li></ol></ol>\""#
        ),
        "c-ws 须产 baz 入内层 ol 且 ws 保留形: {}",
        snap_str
    );
}
