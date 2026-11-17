// indent 邻居合并去重刀钉测（2026-11-18 A 族刀）：_zwJusWrapViaHtml 邻居
// 复用臂 prefix/suffix 复本缺陷修复——被复用邻居壳嵌入展开形后原 outerHTML
// 仍残留在 prefix/suffix＝壳内容复本两次（语料 indent.html A 族三键 got 双
// bar/foo 实证）。三形：次邻复用（idx37 形）、前邻复用（idx40 形）、前邻复
// 用＋次邻 post-rebuild 合并（idx41 形，ViaHtml 补 DOM 臂 :1180 同款次邻臂）。
// Chrome-160 录形均单块 br 分段合并；post-rebuild 臂使两侧邻居全并。
#[test]
fn test_indent_merge_dedup_pins() {
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
        "<html><body><div id=ce137a contenteditable></div></body></html>".to_string(),
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
  var h = document.getElementById("ce137a");
  h.innerHTML = html;
  setup();
  document.execCommand("stylewithcss", false, "false");
  var ret = document.execCommand("indent", false, "");
  __mg.push({ tag: tag, ret: ret, html: h.innerHTML });
}
// idx37 次邻复用：foo 包裹并入次邻裸 blockquote（复本去重＋suffix 排除邻居）
pin("next-reuse", "foo<blockquote>bar</blockquote><p>extra</p>", function () {
  var k = document.getElementById("ce137a").childNodes;
  rng(k[0], 0, k[0], 3);
});
// idx40 前邻复用：baz 包裹并入前邻裸 blockquote（复本去重＋prefix 排除邻居）
pin("prev-reuse", "foo<blockquote>bar</blockquote>baz<p>extra</p>", function () {
  var k = document.getElementById("ce137a").childNodes;
  rng(k[2], 0, k[2], 3);
});
// idx41 前邻复用＋次邻合并：两侧邻居全并单块三段（post-rebuild 次邻臂）
pin("prev-next-merge", "<blockquote>foo</blockquote>bar<blockquote>baz</blockquote><p>extra</p>", function () {
  var k = document.getElementById("ce137a").childNodes;
  rng(k[1], 0, k[1], 3);
});
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // idx37：次邻复用单块合并（复本 bar 不得残留第二块）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"next-reuse\",\"ret\":true,\"html\":\"<blockquote>foo<br>bar</blockquote><p>extra</p>\""#
        ),
        "next-reuse 须产单块 <blockquote>foo<br>bar</blockquote>: {}",
        snap_str
    );
    assert!(
        !snap_str.contains("<blockquote>foo<br>bar</blockquote><blockquote>bar</blockquote>"),
        "次邻复用后不得残留邻居内容复本块: {}",
        snap_str
    );
    // idx40：前邻复用单块合并（复本 bar 不得残留第二块）
    assert!(
        snap_str.contains(
            r#"\"tag\":\"prev-reuse\",\"ret\":true,\"html\":\"foo<blockquote>bar<br>baz</blockquote><p>extra</p>\""#
        ),
        "prev-reuse 须产 foo<blockquote>bar<br>baz</blockquote>: {}",
        snap_str
    );
    // idx41：两侧邻居全并单块三段 br 分隔
    assert!(
        snap_str.contains(
            r#"\"tag\":\"prev-next-merge\",\"ret\":true,\"html\":\"<blockquote>foo<br>bar<br>baz</blockquote><p>extra</p>\""#
        ),
        "prev-next-merge 须产单块三段 foo<br>bar<br>baz: {}",
        snap_str
    );
}
