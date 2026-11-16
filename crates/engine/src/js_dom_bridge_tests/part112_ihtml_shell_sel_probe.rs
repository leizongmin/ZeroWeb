// 编辑命令兼容 ihtml 轴二行内壳刀（2026-11-13）：inserthtml 宿主直子文本门
// （part06.js _zwExecCmdApplyInsertHtml `sc.parentNode === host`）对选区锚在
// 行内壳/rawtext 容器内的静默 bail 修复——双向钉测。
//
// Chrome 实录（e1-ihtml.log r2 post @ffdcd28af 组合态，judge 零漂移背书）：
// ① styled 壳全含选区 = 壳溶解 + 裸载荷落选区起点（foo<span style>[bar]</span>baz
//   + ab<b>c</b>d → fooab<b>c</b>dbaz；跨壳起止 [foo<span>bar]/[bar</span>baz]
//   与双壳 [bar</span><span>baz] 同判——凡壳内容全被选中即整壳溶解）；
// ② rawtext 壳内部分选区 = 壳内拼接保壳（<xmp>f[o]o</xmp> + abc → <xmp>fabco</xmp>，
//   script 同型）；
// ③ pre 内块载荷 = 单块解包后拼接（<pre>1[23]4</pre> + <pre>abc</pre> →
//   <pre>1abc4</pre>；collapsed [12[]34] 与全选 [1234] 同臂）。
// 红 = 守卫 bail 复发（NOOP：零 SetInnerHtml 发射，got=场景原样）；绿 = 壳域臂生效。
// 断言双面：SetInnerHtml mutation 串恰等 + apply_mutations_to_html 镜像终态
// （wpt-runner compare innerHTML 同源 parser）。
#[test]
fn test_inserthtml_styled_shell_dissolve_r3254_m3() {
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
        "<html><body>\
         <div id=ce1 contenteditable>foo<span style=\"color:#aBcDeF\">bar</span>baz</div>\
         <div id=ce2 contenteditable>qux<span style=\"color:#aBcDeF\">bar</span>quz</div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // ① 全含壳选区（[bar]）：壳溶解、载荷落壳位。
    sandbox
        .execute(
            r##"
var host1 = document.getElementById("ce1");
var bt1 = host1.childNodes[1].firstChild;
var r1 = document.createRange();
r1.setStart(bt1, 0);
r1.setEnd(bt1, 3);
getSelection().removeAllRanges();
getSelection().addRange(r1);
document.execCommand("inserthtml", false, "ab<b>c</b>d");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce1")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "fooab<b>c</b>dbaz"),
            "① 全含壳选区须溶壳插载荷（fooab<b>c</b>dbaz），实际: {found:?}"
        );
        let out = apply_mutations_to_html(
            "<html><body><div id=ce1 contenteditable>foo<span style=\"color:#aBcDeF\">bar</span>baz</div></body></html>",
            &m,
        )
        .unwrap();
        assert!(
            out.contains(">fooab&lt;b&gt;") || out.contains("fooab<b>c</b>dbaz"),
            "① 镜像终态须含溶壳插载荷结果，实际: {out}"
        );
    }

    // ② 跨壳起点选区（[foo<span>bar]）：前置文本与壳同删，载荷落选区起点。
    sandbox
        .execute(
            r##"
var host2 = document.getElementById("ce2");
var ft2 = host2.childNodes[0];
var bt2 = host2.childNodes[1].firstChild;
var r2 = document.createRange();
r2.setStart(ft2, 0);
r2.setEnd(bt2, 3);
getSelection().removeAllRanges();
getSelection().addRange(r2);
document.execCommand("inserthtml", false, "ab<b>c</b>d");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce2")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "ab<b>c</b>dquz"),
            "② 跨壳起点选区须前置文本+壳同删（ab<b>c</b>dquz），实际: {found:?}"
        );
    }
}

#[test]
fn test_inserthtml_rawtext_shell_splice_r3254_m3() {
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
        "<html><body>\
         <div id=ce3 contenteditable><xmp>foo</xmp></div>\
         <div id=ce4 contenteditable><script>foo</script>bar</div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // ① xmp 壳内部分选区（f[o]o）：壳内拼接保壳。
    sandbox
        .execute(
            r##"
var xt = document.getElementById("ce3").childNodes[0].firstChild;
var r3 = document.createRange();
r3.setStart(xt, 1);
r3.setEnd(xt, 2);
getSelection().removeAllRanges();
getSelection().addRange(r3);
document.execCommand("inserthtml", false, "abc");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce3")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "<xmp>fabco</xmp>"),
            "① xmp 壳内拼接须保壳（<xmp>fabco</xmp>），实际: {found:?}"
        );
    }

    // ② script 壳内同型（f[o]o + abc → <script>fabco</script>bar）。
    sandbox
        .execute(
            r##"
var st = document.getElementById("ce4").childNodes[0].firstChild;
var r4 = document.createRange();
r4.setStart(st, 1);
r4.setEnd(st, 2);
getSelection().removeAllRanges();
getSelection().addRange(r4);
document.execCommand("inserthtml", false, "abc");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce4")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "<script>fabco</script>bar"),
            "② script 壳内拼接须保壳（<script>fabco</script>bar），实际: {found:?}"
        );
    }
}

#[test]
fn test_inserthtml_pre_block_unwrap_r3254_m3() {
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
        "<html><body><div id=ce5 contenteditable><pre>1234</pre></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // pre 内块载荷（1[23]4 + <pre>abc</pre>）：单块解包后壳内拼接。
    sandbox
        .execute(
            r##"
var pt = document.getElementById("ce5").childNodes[0].firstChild;
var r5 = document.createRange();
r5.setStart(pt, 1);
r5.setEnd(pt, 3);
getSelection().removeAllRanges();
getSelection().addRange(r5);
document.execCommand("inserthtml", false, "<pre>abc</pre>");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce5")))
            .cloned();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "<pre>1abc4</pre>"),
            "pre 内块载荷须解包拼接（<pre>1abc4</pre>），实际: {found:?}"
        );
    }
}

#[test]
fn test_inserthtml_xmp_markup_payload_r3254_m3() {
    // markup 载荷进 rawtext 壳（f[o]o + <b>abc</b> → <xmp>f<b>abc</b>o</xmp>，
    // Chrome 为 DOM 节点插入产物——壳内容序列化含真元素）。本实现走
    // host.innerHTML 重建串，能否复现取决于 parser 对 xmp 内容的 rawtext
    // 处理（Rust 镜像 parser 与 shim 融合视图两侧须同为 markup 保留）。
    // 红定谳后按两侧 parser 事实落地或 #[ignore] 挂账。
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
        "<html><body><div id=ce6 contenteditable><xmp>foo</xmp></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
var xt6 = document.getElementById("ce6").childNodes[0].firstChild;
var r6 = document.createRange();
r6.setStart(xt6, 1);
r6.setEnd(xt6, 2);
getSelection().removeAllRanges();
getSelection().addRange(r6);
document.execCommand("inserthtml", false, "<b>abc</b>");
"##,
        )
        .unwrap();
    {
        let m = mutations.lock().unwrap();
        let found = m
            .iter()
            .rev()
            .find(|mm| matches!(mm, DomMutation::SetInnerHtml { selector, .. } if selector.contains("ce6")))
            .cloned();
        let out = apply_mutations_to_html(
            "<html><body><div id=ce6 contenteditable><xmp>foo</xmp></div></body></html>",
            &m,
        )
        .unwrap();
        assert!(
            matches!(&found, Some(DomMutation::SetInnerHtml { html, .. }) if html == "<xmp>f<b>abc</b>o</xmp>")
                && out.contains("<xmp>f<b>abc</b>o</xmp>"),
            "markup 载荷进 xmp 须壳内元素保留（<xmp>f<b>abc</b>o</xmp>），实际: {found:?} 镜像: {out}"
        );
    }
}
