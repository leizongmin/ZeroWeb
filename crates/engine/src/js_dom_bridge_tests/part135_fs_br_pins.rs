// fs A 族钉测（2026-11-17 刀）：fontsize 载体臂不包占位 br——Chrome-160 录形
// `<p>[foo<p><br><p>bar]` 双变体中段恒裸 `<p><br></p>`（data/fontsize.js:38/
// :43，swcss T/F 同），qI-after=true（录形 indeterm-after=true：br 继承缺省
// "3" vs 载体 "4" → indeterminate，读侧刀④ _zwValIsFormattableQI 按未包裹
// br 计值）。同选区 forecolor 等十命令录形均裹 br ＝命令特有 delta，forecolor
// 反向钉防通用化（span color 载体含 br）。证据
// docs/goal/archive/editing-compat/evidence/2026-11-17-fs-census.md。
#[test]
fn test_fs_br_pins() {
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
        "<html><body><div id=ce135a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
globalThis.__fs = [];
function pin(tag, cmd, value, sw) {
  var h = document.getElementById("ce135a");
  h.innerHTML = "<p>foo</p><p><br></p><p>bar</p>";
  var ps = h.querySelectorAll("p");
  var r = document.createRange();
  r.setStart(ps[0].firstChild, 0);
  r.setEnd(ps[2].firstChild, 3);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
  document.execCommand("stylewithcss", false, sw);
  var ret = document.execCommand(cmd, false, value);
  var sel = getSelection();
  var sr = sel.rangeCount ? sel.getRangeAt(0) : null;
  var caret = "norange";
  if (sr) {
    var d = function (n, o) {
      return (n.nodeType === 3 ? "#" + String(n.data) + "#" : "<" + n.nodeName + ">") + "@" + o;
    };
    caret = d(sr.startContainer, sr.startOffset) + ".." + d(sr.endContainer, sr.endOffset);
  }
  __fs.push({
    tag: tag,
    ret: ret,
    html: h.innerHTML,
    caret: caret,
    qV: document.queryCommandValue(cmd === 'fontsize' ? 'fontsize' : cmd),
    qI: document.queryCommandIndeterm(cmd)
  });
}
// A 族主体：fontsize 双变体，br 裸形 + qI-after=true（录形字节）
pin("fs-T", "fontsize", "4", "true");
pin("fs-F", "fontsize", "4", "false");
// 反向钉：forecolor 同选区仍裹 br（data/forecolor.js:38 span color 载体含 br）
pin("fc-T", "forecolor", "#0000FF", "true");
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__fs)").unwrap();
    println!("FS PINS: {:?}", snap);
    let snap_str = format!("{:?}", snap);
    // fs-T/fs-F 主体：占位 br 裸形（裸 br 专属段 <p><br></p> 恰出现 2 次）
    assert_eq!(
        snap_str.matches("<p><br></p>").count(),
        2,
        "fs 双变体须各含裸 br 段 <p><br></p>: {}",
        snap_str
    );
    // 载体裹 br 形在 fontsize 双变体缺席（刀前形态：font/large 载体内含 <br>）
    assert!(
        !snap_str.contains(r#"size=\\\"4\\\"><br>"#) && !snap_str.contains(r#"large\\\"><br>"#),
        "fontsize 载体不得裹 br: {}",
        snap_str
    );
    // qI-after=true ×2、qV-after="4" ×2（录形 indeterm-after=true/value-after="4"）
    assert_eq!(
        snap_str.matches(r#"\"qI\":true"#).count(),
        2,
        "fs 双变体 qI-after 须均 true: {}",
        snap_str
    );
    assert_eq!(
        snap_str.matches(r#"\"qV\":\"4\""#).count(),
        2,
        "fs 双变体 qV-after 须均 \"4\": {}",
        snap_str
    );
    // 选区恢复（端点随文本缘，br 裸形不动端点；三 pin 同断言面）
    assert_eq!(
        snap_str.matches("#foo#@0..#bar#@3").count(),
        3,
        "选区端点须保持 foo@0..bar@3: {}",
        snap_str
    );
    // 反向钉：forecolor T 形 br 仍裹进 span color 载体（录形 data/forecolor.js:38）
    assert!(
        snap_str.contains("<br></span></p>")
            && snap_str.contains(r#"color:rgb(0, 0, 255)\\\"><br>"#),
        "forecolor 反向钉不符（br 须仍裹载体）: {}",
        snap_str
    );
}
