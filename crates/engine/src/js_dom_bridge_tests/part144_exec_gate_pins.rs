// exec2/exec3 刀钉测（2026-11-19）：① exec3 unknown-command 门——表外命令
// （junk/heading/increaseFontSize）入口即返 false 不抛零事件零 DOM（Chrome
// 录音 exec-command-never-throw-exceptions.tentative ret===false 断言 +
// without-editable 语料 expectedResult:false 行）；② exec2 递归重入门——
// input listener 内嵌套 execCommand 返 false 零插入（recursive-exec-command-
// calls.tentative：外层 ret=true textContent="0"、嵌套 ret=false、计数恒 1）；
// ③ 深度清零控制键——外层完成后新调用不受死锁（textContent 翻新形）；④
// 已知命令控制键——bold 在宿主直子文本选区照常 ret=true 派 1 input。
#[test]
fn test_exec_gate_pins() {
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
        "<html><body><div id=ce144 contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
globalThis.__mg = [];
var h = document.getElementById("ce144");
function caretAt(node, off) {
  var r = document.createRange();
  r.setStart(node, off);
  r.setEnd(node, off);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
}
// ① exec3 面：表外命令零效果（html 不变＋零 input 事件＋ret false）
function pinUnknown(tag, cmd) {
  h.innerHTML = "abc";
  var evs = 0;
  var li = function () { evs++; };
  h.addEventListener("input", li);
  var ret;
  var threw = "no";
  try { ret = document.execCommand(cmd, false, "foo"); }
  catch (e) { threw = String(e && e.message || e); }
  h.removeEventListener("input", li);
  __mg.push({ tag: tag, ret: ret, threw: threw, evs: evs, html: h.innerHTML });
}
pinUnknown("u-junk", "unknown-command");
pinUnknown("u-heading", "heading");
pinUnknown("u-incfs", "increaseFontSize");
// ② exec2 面：input listener 内嵌套调用（语料 recursive 形——外层插 "0"，
// listener 对前 9 次 input 调嵌套 insertText，嵌套须 ret=false 零效果）
__mg.push({ tag: "rec", rec: (function () {
  var m = [];
  try {
  m.push("s1");
  h.innerHTML = "<br>";
  m.push("s2");
  var counter = 0;
  var nested = [];
  var li = function () {
    if (++counter < 10) {
      nested.push(document.execCommand("insertText", false, ", " + counter));
    }
  };
  h.addEventListener("input", li);
  caretAt(h, 0);
  m.push("s3");
  var outer = document.execCommand("insertText", false, "0");
  m.push("s4");
  var out = { outer: outer, counter: counter, nested: nested, text: h.textContent };
  // ③ 清零控制键：移除 listener 后重置，新调用须不受死锁（嵌套数组不再被
  // 后续事件污染——rec listener 只覆盖 outer 窗）
  h.removeEventListener("input", li);
  h.innerHTML = "<br>";
  caretAt(h, 0);
  m.push("s5");
  out.outer2 = document.execCommand("insertText", false, "1");
  out.text2 = h.textContent;
  out.m = m;
  return out;
  } catch (e) { return { err: String(e && e.stack || e), m: m }; }
})() });
// ④ 已知命令控制键：bold 照常应用
(function () {
  h.innerHTML = "abc";
  var evs = 0;
  var li = function () { evs++; };
  h.addEventListener("input", li);
  var t = h.childNodes[0];
  var r = document.createRange();
  r.setStart(t, 1);
  r.setEnd(t, 2);
  getSelection().removeAllRanges();
  getSelection().addRange(r);
  var ret = document.execCommand("bold", false, "");
  h.removeEventListener("input", li);
  __mg.push({ tag: "bold", ret: ret });
  __mg.push({ tag: "bold-shape", html: h.innerHTML, evs: evs });
})();
"##,
        )
        .unwrap();

    let snap = sandbox.execute("JSON.stringify(globalThis.__mg)").unwrap();
    let snap_str = format!("{:?}", snap.value);
    // ① 表外命令：ret false 不抛零事件零 DOM（recording：without-editable
    // heading/increaseFontSize expectedResult:false；junk never-throw ret false）
    assert!(
        snap_str.contains(r#"\"tag\":\"u-junk\",\"ret\":false,\"threw\":\"no\",\"evs\":0,\"html\":\"abc\""#),
        "u-junk 须 ret=false 零抛零事件零 DOM: {}",
        snap_str
    );
    assert!(
        snap_str.contains(r#"\"tag\":\"u-heading\",\"ret\":false,"#)
            && snap_str.contains(r#"\"tag\":\"u-incfs\",\"ret\":false,"#),
        "heading/increaseFontSize 须 ret=false: {}",
        snap_str
    );
    // ② exec2：外层 true、嵌套拒一次（嵌套拒绝不派事件→计数恒 1）、text "0"
    assert!(
        snap_str.contains(r#"\"rec\":{\"outer\":true,\"counter\":1,\"nested\":[false],\"text\":\"0\""#),
        "recursion 形须 outer=true counter=1 nested=[false] text=0: {}",
        snap_str
    );
    // ③ 清零控制键：重置后 insertText 照常生效
    assert!(
        snap_str.contains(r#"\"outer2\":true,\"text2\":\"1\""#),
        "深度须清零（outer2=true text2=1）: {}",
        snap_str
    );
    // ④ bold 控制键：已知命令不受门影响
    assert!(
        snap_str.contains(r#"\"tag\":\"bold\",\"ret\":true"#)
            && snap_str.contains(r#"\"tag\":\"bold-shape\",\"html\":\"a<b>b</b>c\",\"evs\":1"#),
        "bold 须 ret=true b 包裹形派 1 input: {}",
        snap_str
    );
}
