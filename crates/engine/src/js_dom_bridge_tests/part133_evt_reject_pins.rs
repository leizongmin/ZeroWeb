// 编辑命令值拒绝域钉测（2026-11-17 EVT-REJECT 刀）：值拒绝路径按形补派 input
// ——Chrome-160 master event.html 三真差异键（fontSize "15px"/formatBlock
// "quasit"/insertImage ""：Chrome 派 1 input、刀前 0），其余拒绝形保持零事件
// （fontSize ""/quasit＋formatBlock ""：Chrome 同红形，补派即反向 delta）。
// ret 与树形零改动（语料录形 ret [false]＋树不变在册：formatblock.js
// :872/:1117/:1455、insertimage.js :20）；enabled 面（端点出可编辑域）不补派。
// 证据 docs/goal/archive/editing-compat/evidence/2026-11-17-fb-event-census.md；
// wpt.fyi chrome 160.0.8166.0 master 739652a308 report 逐子测定谳。
#[test]
fn test_evt_reject_pins() {
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
        "<html><body><div id=ce133a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// pin133(id, cmd, value, expRet, expCount, expSame, html, rngInBFalse)——
// html 缺省 foo<b>bar</b>baz、选区 (b.firstChild,0)-(b,1)（corpus event.html
// commandTests 同形）；rngInBFalse true 时 html 换 tt[ce=false] 形（enabled 面
// 控制键：端点出可编辑域）。expSame null＝树态不断言（合法值 apply 控制键）。
function pin133(id, cmd, value, expRet, expCount, expSame, html, rngInBFalse) {
  var h = document.getElementById("ce133a");
  h.innerHTML = html || "foo<b>bar</b>baz";
  var r = document.createRange();
  if (rngInBFalse) {
    var bf = h.querySelector("b");
    r.setStart(bf.firstChild, 0);
    r.setEnd(bf.firstChild, 3);
  } else {
    var b = h.querySelector("b");
    r.setStart(b.firstChild, 0);
    r.setEnd(b, 1);
  }
  getSelection().removeAllRanges();
  getSelection().addRange(r);
  var before = h.innerHTML;
  var count = 0;
  h.addEventListener("input", function () { count++; });
  var ret;
  try { ret = document.execCommand(cmd, false, value); } catch (e133) {
    globalThis.__zwP133.push({ id: id, bad: "THROW " + String(e133 && e133.message || e133) });
    return;
  }
  var bad = null;
  if (ret !== expRet) bad = "ret=" + ret + " exp=" + expRet;
  else if (count !== expCount) bad = "count=" + count + " exp=" + expCount;
  else if (expSame !== null && (h.innerHTML === before) !== expSame) {
    bad = "same=" + (h.innerHTML === before) + " exp=" + expSame;
  }
  globalThis.__zwP133.push({ id: id, bad: bad,
    got: "ret=" + ret + " count=" + count + " out=" + h.innerHTML });
}

globalThis.__zwP133 = [];
// 补派形（3 真差异键）：1 input、ret false、树不变。
pin133("evt-rej-fs-px", "fontsize", "15px", false, 1, true);
pin133("evt-rej-fb-quasit", "formatblock", "quasit", false, 1, true);
pin133("evt-rej-img-empty", "insertimage", "", false, 1, true);
// 同红形（Chrome 同红，零事件维持）。
pin133("evt-rej-fs-empty", "fontsize", "", false, 0, true);
pin133("evt-rej-fs-quasit", "fontsize", "quasit", false, 0, true);
pin133("evt-rej-fb-empty", "formatblock", "", false, 0, true);
// enabled 面控制键：端点入 tt[ce=false]，CSS 长度形也不补派（零事件在册）。
pin133("evt-rej-enabled-face", "fontsize", "15px", false, 0, true,
  "foo<b contenteditable=\"false\">bar</b>baz", true);
// 控制键（正常臂恒一）：bold 全包裹选区 toggle 解壳 ret true＋树变＋1 事件。
pin133("evt-ctl-bold", "bold", "", true, 1, false);
// 合法值 fb 控制键：ret true＋1 事件（树态不断言——apply 形语料腿承载）。
pin133("evt-ctl-fb-valid", "formatblock", "<div>", true, 1, null);

var bad133 = [];
for (var pi = 0; pi < globalThis.__zwP133.length; pi++) {
  var c = globalThis.__zwP133[pi];
  if (c.bad) bad133.push(c.id + ": " + c.bad + " | " + c.got);
}
if (bad133.length) throw "EVTRJPIN MISMATCH x" + bad133.length + " | " + bad133.join(" || ");
"##,
        )
        .expect("evt reject pins: all forms must hold");
}
