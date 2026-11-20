// R13 cE=false wrap 域钉测（2026-11-17 R13 刀）：选区整体含 cE=false 行内内容
// 时 wrap 命令（formatBlock/indent）整面纳入单块容器——Chrome delta 对 vendored
// spec「if it is editable」排除面的收窄。语料对账：formatblock.html 条目 362
// compare 三形＋indent.html swcss=true/false 三连形（143/144/145/146 idx，census
// 归域档 docs/goal/archive/editing-compat/evidence/2026-11-17-r13-cefalse-wrap-census.md）。
// 容器形对账：swcss=true 产 webkit-indent-blockquote 形（三行 accepted 皆收）；
// swcss=false 恒 plain blockquote；fb 恒 p。守卫保留面：部分含非可编辑链节点
// 维持拦截零树变更返 true（既有保护面不变）。
#[test]
fn test_r13_cefalse_wrap_pins() {
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
        "<html><body><div id=ce134a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// pin134(id, cmd, value, swcss, html, useHostRange, expRet, expOut)——html 装
// ce134a；useHostRange true＝选区 (h,0)-(h,childNodes.length)（语料 { ... } 全含
// 形），false＝html 需含 <p>abc</p> 时选区 (p,0)-(p,1) 或 guard 保留形选区
// span 首 text 0-3（由 html 形态分支）。expOut 精确串比 innerHTML。
function pin134(id, cmd, value, swcss, html, rngKind, expRet, expOut) {
  var h = document.getElementById("ce134a");
  h.innerHTML = html;
  var r = document.createRange();
  if (rngKind === "host") {
    r.setStart(h, 0);
    r.setEnd(h, h.childNodes.length);
  } else if (rngKind === "p") {
    var p = h.querySelector("p");
    r.setStart(p, 0);
    r.setEnd(p, 1);
  } else if (rngKind === "innerspan") {
    var sp = h.querySelector("span");
    r.setStart(sp.firstChild, 0);
    r.setEnd(sp.firstChild, 3);
  }
  getSelection().removeAllRanges();
  getSelection().addRange(r);
  if (swcss !== null) {
    document.execCommand("stylewithcss", false, swcss);
  }
  var before = h.innerHTML;
  var ret;
  try { ret = document.execCommand(cmd, false, value); } catch (e134) {
    globalThis.__zwP134.push({ id: id, bad: "THROW " + String(e134 && e134.message || e134),
      got: "" });
    return;
  }
  var after = h.innerHTML;
  var bad = null;
  if (ret !== expRet) bad = "ret=" + ret + " exp=" + expRet;
  else if (expOut !== null && after !== expOut) bad = "out mismatch";
  globalThis.__zwP134.push({ id: id, bad: bad, got: "ret=" + ret + " out=" + after });
}

globalThis.__zwP134 = [];
var WS_SPANS = " <span contenteditable=\"false\">A</span> ; "
  + "<span contenteditable=\"false\">B</span> ; "
  + "<span contenteditable=\"false\">C</span> ";
var WBQ_STYLE = "margin: 0 0 0 40px; border: none; padding: 0px;";
// fb 362（语料 accepted[0] ws 内保留形）：三 span＋ws 整体入新 p，ret true。
pin134("r13-fb-362", "formatblock", "p", null, WS_SPANS, "host", true,
  "<p>" + WS_SPANS + "</p>");
// indent 三连形 swcss=true（语料 idx 145 accepted[2] ws 内保留形）：webkits
// 风格 blockquote 单容器。
pin134("r13-indent-145-T", "indent", "", "true", WS_SPANS, "host", true,
  "<blockquote style=\"" + WBQ_STYLE + "\">" + WS_SPANS + "</blockquote>");
// indent 三连形 swcss=false（语料 idx 146 accepted[0]）：plain blockquote。
pin134("r13-indent-146-F", "indent", "", "false", WS_SPANS, "host", true,
  "<blockquote>" + WS_SPANS + "</blockquote>");
// indent `<p>[abc]</p>` swcss=true（语料 idx 143 accepted[1]）：p 保留入
// webkit 风格 blockquote。
pin134("r13-indent-143-T", "indent", "", "true", "<p>abc</p>", "p", true,
  "<blockquote style=\"" + WBQ_STYLE + "\"><p>abc</p></blockquote>");
// 守卫保留面：部分含非可编辑链节点（span 全含但其 cE=false 宿主含界外 baz）
// 维持拦截零树变更返 true。
pin134("r13-fb-guard-preserve", "formatblock", "p", null,
  "<div contenteditable=\"false\"><span>bar</span>baz</div>", "innerspan",
  true, null);

var bad134 = [];
for (var pi = 0; pi < globalThis.__zwP134.length; pi++) {
  var c = globalThis.__zwP134[pi];
  if (c.bad) bad134.push(c.id + ": " + c.bad + " | " + c.got);
}
if (bad134.length) throw "R13PIN MISMATCH x" + bad134.length + " | " + bad134.join(" || ");
"##,
        )
        .expect("r13 cE=false wrap pins: all forms must hold");
}
