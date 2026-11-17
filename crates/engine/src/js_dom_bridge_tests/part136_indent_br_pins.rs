// indent br 边界刀钉测（2026-11-17 刀）：indent 新建块边界相邻 br 消费——
// 仅当边界 br 是唯一行界（连续 br run=1）。尾侧 2 录形（idx28/29：sublist
// 尾单 br 摘除并删）、首侧 2 录形（idx30 直形／idx18 span 行内壳尾链形）；
// run=4 守卫（idx31 四枚全留）＋caret 前 br 双守卫（idx15/17 现绿恒一：
// br 留块内不消费）。list 臂早退＋outdent 复用不传旗标＝不触发。
// 证据 docs/goal/editing-compat/evidence/2026-11-17-indent-census.md。
#[test]
fn test_indent_br_pins() {
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
        "<html><body><div id=ce136a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
globalThis.__ind = [];
function rng(a, ao, b, bo) {
  var r = document.createRange();
  r.setStart(a, ao);
  r.setEnd(b, bo);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
}
function pin(tag, html, setup) {
  var h = document.getElementById("ce136a");
  h.innerHTML = html;
  setup();
  document.execCommand("stylewithcss", false, "false");
  var ret = document.execCommand("indent", false, "");
  __ind.push({ tag: tag, ret: ret, html: h.innerHTML });
}
// 尾侧 idx28：foo[bar]<br>baz → 尾 br 摘除并删
pin("end-single", "foobar<br>baz<p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[0], 3, k[0], 6);
});
// 尾侧 idx29：foo[bar]<br><br><br><br>baz → 仅首 br 消费余三枚保留
pin("end-multi", "foobar<br><br><br><br>baz<p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[0], 3, k[0], 6);
});
// 首侧 idx30：foobar<br>[ba]z → 包裹前单 br 删
pin("start-bare", "foobar<br>baz<p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[2], 0, k[2], 2);
});
// 首侧 idx18：<span>foo<br>{}</span>bar → span 尾 br（行内壳尾链）删
pin("start-shell", "<span>foo<br></span>bar<p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[0], 2, k[0], 2);
});
// run=4 守卫 idx31：foobar<br><br><br><br>[ba]z → 四枚全留不消费
pin("run4-guard", "foobar<br><br><br><br>baz<p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[5], 0, k[5], 2);
});
// caret 前 p 守卫 idx15（现绿恒一）：br 留块内
pin("caret-p-guard", "<p><br>foo</p><p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[0], 0, k[0], 0);
});
// caret 前 span 守卫 idx17（现绿恒一）：br 留块内
pin("caret-span-guard", "<span><br>foo</span>bar<p>extra</p>", function () {
  var k = document.getElementById("ce136a").childNodes;
  rng(k[0], 0, k[0], 0);
});

"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__ind)").unwrap();
    println!("INDENT BR PINS: {:?}", snap);
    let snap_str = format!("{:?}", snap);
    // 7 形全 ret=true（语料 ret 键 [true]）
    assert_eq!(
        snap_str.matches(r#"\"ret\":true"#).count(),
        7,
        "indent 7 形 ret 须均 true: {}",
        snap_str
    );
    // 尾侧 idx28：尾 br 删、baz 留块外
    assert!(
        snap_str.contains(
            r#"\"tag\":\"end-single\",\"ret\":true,\"html\":\"<blockquote>foobar</blockquote>baz<p>extra</p>\""#
        ),
        "end-single 须产 <blockquote>foobar</blockquote>baz: {}",
        snap_str
    );
    // 尾侧 idx29：仅首 br 消费，余三枚随 baz 保留块外
    assert!(
        snap_str.contains(
            r#"\"tag\":\"end-multi\",\"ret\":true,\"html\":\"<blockquote>foobar</blockquote><br><br><br>baz<p>extra</p>\""#
        ),
        "end-multi 须产 <blockquote>foobar</blockquote><br><br><br>baz: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("<blockquote>foobar<br>"),
        "尾侧两形不得残留包裹尾 br: {}",
        snap_str
    );
    // 首侧 idx30：包裹前单 br 删
    assert!(
        snap_str.contains(
            r#"\"tag\":\"start-bare\",\"ret\":true,\"html\":\"foobar<blockquote>baz</blockquote><p>extra</p>\""#
        ),
        "start-bare 须产 foobar<blockquote>baz</blockquote>: {}",
        snap_str
    );
    // 首侧 idx18：span 尾 br 删（壳留 <span>foo</span>）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"start-shell\",\"ret\":true,\"html\":\"<span>foo</span><blockquote>bar</blockquote><p>extra</p>\""#
        ),
        "start-shell 须产 <span>foo</span><blockquote>bar</blockquote>: {}",
        snap_str
    );
    // run=4 守卫 idx31：四枚 br 全留（唯一行界判据不误伤多 br 空行分隔形）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"run4-guard\",\"ret\":true,\"html\":\"foobar<br><br><br><br><blockquote>baz</blockquote><p>extra</p>\""#
        ),
        "run4-guard 须保全四枚 br: {}",
        snap_str
    );
    // caret 前 p 守卫 idx15（现绿恒一）：br 留块内、foo 出块落新 p
    assert!(
        snap_str.contains(
            r#"\"tag\":\"caret-p-guard\",\"ret\":true,\"html\":\"<blockquote><br></blockquote><p>foo</p><p>extra</p>\""#
        ),
        "caret-p-guard br 须留块内 foo 出块: {}",
        snap_str
    );
    // caret 前 span 守卫 idx17（现绿恒一）：br 留块内、foo 留 span 块外
    assert!(
        snap_str.contains(
            r#"\"tag\":\"caret-span-guard\",\"ret\":true,\"html\":\"<blockquote><br></blockquote><span>foo</span>bar<p>extra</p>\""#
        ),
        "caret-span-guard br 须留块内 foo 留 span: {}",
        snap_str
    );
}
