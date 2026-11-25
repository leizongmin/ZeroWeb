// s10k27 刀㉖ Report-Only CSP monitoring 常驻钉测（2026-11-26）
// corpus 形蒸馏：WPT trusted-types RO/enforced 七页全绿形（default-policy-report-only
// 19P / default-policy 19P / empty-*-report-only 4P / require-RO 4P /
// no-require-RO 9P / trusted-types-report-only 5P——script-url 面＝script 取归
// 车道旗 wire arg12 script-src 族分流修复后解）。corpus 页 gitignored（按 clone
// 取数），本文件＝committed 常驻回归义务载体。刀㉗（s10k28）将 sink 面
// violation sample 更新为「sink 名|40 单元截断值」形（spec TT §violation
// record），本文件相应断言随之改带前缀。
//
// 钉面（sink 管道全链真生效——host DOM 携 CSP meta，镜像语料 harness 语境；
// 违规派发为 task（setTimeout 0，CSP3 §report-violation task 语义）→ 记录式
// host stub ＋ `__zw_fire` 手动驱动到期，listener 收集后二次 execute 断言）：
//   RO 面（无 default policy）——plain 入 sink：原样透传＋不抛＋report 违规
//   （disposition 'report'，sample＝原串，originalPolicy＝RO meta 原文，
//   effectiveDirective/violatedDirective＝require-trusted-types-for）；
//   TT 实例直通无违规。script.src IDL 属性面同管道。
//   RO 面（有 default policy）——返串变形无违规；返 undefined→原串＋report
//   违规；回调抛错原样传播（无包装）＋无违规。
//   enforced 面（有 default policy）——plain 过管道变形；缺对应 create 回调→
//   TypeError＋enforce 违规；返 null→TypeError＋sink 保持原值＋enforce 违规；
//   回调抛错原样传播＋无违规；TT 实例直通。
//   part12 直调包装语义不受 __zwRawCbs 寄存影响——undefined→TypeError
//   （'The createHTML callback returned undefined.'）、null→''。
//   WebSocket CSP 面——enforce connect-src 'none' 检查先于宿主桥缺失臂：
//   违规照达（effectiveDirective 'connect-src'，blockedURI＝解析后 href）＋
//   连接异步 fail（error/close 后 CLOSED）。
// https://w3c.github.io/webappsec-trusted-types/dist/spec/#process-value-with-default-policy
// https://w3c.github.io/webappsec-csp/#report-violation

/// 记录式 host 定时器 stub——`__zw_setTimeout` 只登记 id，`__zw_fire` 手动驱动
/// 到期（等价 TimerBridge resolve 时序；part01.rs 同款）。
const S10K27_TIMER_STUB_JS: &str = "globalThis.__zw_pending = {};\
  globalThis.__zw_armed = [];\
  globalThis.__zw_setTimeout = function (id, delay) { globalThis.__zw_armed.push(id); };\
  globalThis.__zw_fire = function () {\
    var armed = globalThis.__zw_armed; globalThis.__zw_armed = [];\
    for (var i = 0; i < armed.length; i++) { globalThis.__zwResolveCallback(armed[i], ''); } };";

/// RO/enforced 双面共用的违规收集器＋断言工具 IIFE 前缀。
const S10K27_PIN_PRELUDE_JS: &str = r#"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  var tt = globalThis.trustedTypes;
  if (!tt || typeof tt.createPolicy !== 'function') return 'FAIL:no-tt-factory';
  globalThis.__v27 = [];
  document.addEventListener('securitypolicyviolation', function (e) {
    globalThis.__v27.push({ dir: e.effectiveDirective, disp: e.disposition,
      sample: e.sample, op: e.originalPolicy, blocked: e.blockedURI });
  });
"#;

/// 钉测共用收尾（body 元素供 innerHTML 面）。
const S10K27_PIN_BODY_JS: &str = "
  var div = document.createElement('div');
  document.body.appendChild(div);
  var sc = document.createElement('script');
  document.body.appendChild(sc);
";

#[test]
fn test_s10k27_ro_no_default_policy_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(S10K27_TIMER_STUB_JS).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><meta http-equiv=\"Content-Security-Policy-Report-Only\" content=\"require-trusted-types-for 'script';\"><title>k27ro</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k27-ro.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var p1 = true;
  try {{ div.innerHTML = 'abc'; }} catch (e) {{ p1 = false; fails.push('P1 threw ' + e); }}
  chk(eq(p1, true, 'P1 no-throw'));
  chk(eq(div.innerHTML, 'abc', 'P1 raw kept'));
  var p2 = true;
  try {{ sc.src = 'support/namespaces.js#abc'; }} catch (e) {{ p2 = false; fails.push('P2 threw ' + e); }}
  chk(eq(p2, true, 'P2 no-throw'));
  chk(eq(sc.getAttribute('src'), 'support/namespaces.js#abc', 'P2 attr raw kept'));
  var pol = tt.createPolicy('k27a', {{ createHTML: function (x) {{ return '(' + x + ')'; }} }});
  try {{ div.innerHTML = pol.createHTML('xyz'); }} catch (e) {{ fails.push('P3 threw ' + e); }}
  chk(eq(div.innerHTML, '(xyz)', 'P3 tt passthrough'));
  return fails.length ? fails.join(';') : 'PASS';
}})();
"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 ro-no-default execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k27 RO 无 default policy 钉测失败：{}", verdict);
    // 违规派发 task 化——驱动到期后断言字段与序（P1 innerHTML → P2 script.src；
    // TT 直通面无违规）。
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        "(function () { var v = globalThis.__v27;\
          if (v.length !== 2) return 'FAIL:count(' + v.length + ')';\
          if (v[0].dir !== 'require-trusted-types-for') return 'FAIL:v0.dir(' + v[0].dir + ')';\
          if (v[0].disp !== 'report') return 'FAIL:v0.disp(' + v[0].disp + ')';\
          if (v[0].sample !== 'Element innerHTML|abc') return 'FAIL:v0.sample(' + v[0].sample + ')';\
          if (String(v[0].op).indexOf(\"require-trusted-types-for 'script'\") < 0) return 'FAIL:v0.op(' + v[0].op + ')';\
          if (v[0].blocked !== '') return 'FAIL:v0.blocked(' + v[0].blocked + ')';\
          if (v[1].sample !== 'HTMLScriptElement src|support/namespaces.js#abc') return 'FAIL:v1.sample(' + v[1].sample + ')';\
          if (v[1].disp !== 'report') return 'FAIL:v1.disp(' + v[1].disp + ')';\
          return 'PASS'; })()",
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 ro-no-default fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k27 RO 违规字段钉测失败：{}", fields);
}

#[test]
fn test_s10k27_ro_default_policy_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(S10K27_TIMER_STUB_JS).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><meta http-equiv=\"Content-Security-Policy-Report-Only\" content=\"require-trusted-types-for 'script';\"><title>k27rod</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k27-rod.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  tt.createPolicy('default', {{ createHTML: function (x) {{
    if (x === 'r2') return undefined;
    if (x === 'r3') throw new Error('boom');
    return '[' + x + ']';
  }} }});
  var p1 = true;
  try {{ div.innerHTML = 'q'; }} catch (e) {{ p1 = false; fails.push('P1 threw ' + e); }}
  chk(eq(p1, true, 'P1 no-throw'));
  chk(eq(div.innerHTML, '[q]', 'P1 transform applied'));
  var p2 = true;
  try {{ div.innerHTML = 'r2'; }} catch (e) {{ p2 = false; fails.push('P2 threw ' + e); }}
  chk(eq(p2, true, 'P2 undefined no-throw'));
  chk(eq(div.innerHTML, 'r2', 'P2 undefined raw kept'));
  var p3msg = '';
  try {{ div.innerHTML = 'r3'; }} catch (e) {{ p3msg = String(e && e.message); }}
  chk(eq(p3msg, 'boom', 'P3 throw identity propagated'));
  chk(eq(div.innerHTML, 'r2', 'P3 throw sink keeps prior'));
  return fails.length ? fails.join(';') : 'PASS';
}})();
"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 ro-default execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k27 RO 有 default policy 钉测失败：{}", verdict);
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        "(function () { var v = globalThis.__v27;\
          if (v.length !== 1) return 'FAIL:count(' + v.length + ')';\
          if (v[0].disp !== 'report') return 'FAIL:v0.disp(' + v[0].disp + ')';\
          if (v[0].sample !== 'Element innerHTML|r2') return 'FAIL:v0.sample(' + v[0].sample + ')';\
          return 'PASS'; })()",
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 ro-default fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k27 RO default 违规钉测失败：{}", fields);
}

#[test]
fn test_s10k27_enforced_default_policy_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(S10K27_TIMER_STUB_JS).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"><title>k27enf</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k27-enf.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  tt.createPolicy('default', {{ createHTML: function (x) {{
    if (x === 'n1') return null;
    if (x === 'n2') throw new Error('enf-boom');
    return '[' + x + ']';
  }} }});
  var p1 = true;
  try {{ div.innerHTML = 'q'; }} catch (e) {{ p1 = false; fails.push('P1 threw ' + e); }}
  chk(eq(p1, true, 'P1 no-throw'));
  chk(eq(div.innerHTML, '[q]', 'P1 transform applied'));
  var p2threw = '';
  try {{ sc.src = 'plain-url'; }} catch (e) {{ p2threw = e && e.constructor && e.constructor.name; }}
  chk(eq(p2threw, 'TypeError', 'P2 missing createScriptURL TypeError'));
  chk(eq(sc.getAttribute('src'), null, 'P2 sink unchanged'));
  var p3threw = '';
  try {{ div.innerHTML = 'n1'; }} catch (e) {{ p3threw = e && e.constructor && e.constructor.name; }}
  chk(eq(p3threw, 'TypeError', 'P3 null return TypeError'));
  chk(eq(div.innerHTML, '[q]', 'P3 sink keeps prior'));
  var p4msg = '';
  try {{ div.innerHTML = 'n2'; }} catch (e) {{ p4msg = String(e && e.message); }}
  chk(eq(p4msg, 'enf-boom', 'P4 throw identity propagated'));
  var pol = tt.createPolicy('k27b', {{ createHTML: function (x) {{ return '<i>' + x + '</i>'; }} }});
  try {{ div.innerHTML = pol.createHTML('t'); }} catch (e) {{ fails.push('P5 threw ' + e); }}
  chk(eq(div.innerHTML, '<i>t</i>', 'P5 tt passthrough'));
  return fails.length ? fails.join(';') : 'PASS';
}})();
"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 enforced execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k27 enforced 钉测失败：{}", verdict);
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        "(function () { var v = globalThis.__v27;\
          if (v.length !== 2) return 'FAIL:count(' + v.length + ')';\
          if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';\
          if (v[0].sample !== 'HTMLScriptElement src|plain-url') return 'FAIL:v0.sample(' + v[0].sample + ')';\
          if (v[1].disp !== 'enforce') return 'FAIL:v1.disp(' + v[1].disp + ')';\
          if (v[1].sample !== 'Element innerHTML|n1') return 'FAIL:v1.sample(' + v[1].sample + ')';\
          return 'PASS'; })()",
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 enforced fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k27 enforced 违规钉测失败：{}", fields);
}

#[test]
fn test_s10k27_policy_direct_call_wrapper_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(S10K27_TIMER_STUB_JS).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"><title>k27dc</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k27-dc.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var pol = tt.createPolicy('k27c', {{ createHTML: function (x) {{ return String(x); }} }});
  var i1 = '';
  try {{ i1 = String(pol.createHTML(undefined)); }} catch (e) {{ i1 = 'threw:' + e; }}
  chk(eq(i1, 'undefined', 'I undefined input stringified'));
  var u1 = '';
  try {{ pol.createHTML('x'); }} catch (e) {{ u1 = String(e && e.message); }}
  var polU = tt.createPolicy('k27cU', {{ createHTML: function () {{ return undefined; }} }});
  try {{ polU.createHTML('x'); }} catch (e) {{ u1 = String(e && e.message); }}
  chk(eq(u1, 'The createHTML callback returned undefined.', 'U undefined return TypeError'));
  var n1 = '';
  var polN = tt.createPolicy('k27cN', {{ createHTML: function () {{ return null; }} }});
  try {{ n1 = String(polN.createHTML('x')); }} catch (e) {{ n1 = 'threw:' + e; }}
  chk(eq(n1, '', 'N null return empty string'));
  return fails.length ? fails.join(';') : 'PASS';
}})();
"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 direct-call execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k27 直调包装钉测失败：{}", verdict);
}

#[test]
fn test_s10k27_ws_connect_src_pin() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(S10K27_TIMER_STUB_JS).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"connect-src 'none'\"><title>k27ws</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k27-ws.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var ws = null;
  try {{ ws = new WebSocket('ws://wpt.test/x'); }} catch (e) {{ fails.push('ctor threw ' + e); }}
  if (!ws) return fails.length ? fails.join(';') : 'PASS';
  return fails.length ? fails.join(';') : 'PASS';
}})();
"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 ws execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k27 WS 构造钉测失败：{}", verdict);
    // 驱动到期：connect-src 违规 task ＋ 连接异步 fail（error/close）task。
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        "(function () { var v = globalThis.__v27;\
          if (v.length < 1) return 'FAIL:count(' + v.length + ')';\
          if (v[0].dir !== 'connect-src') return 'FAIL:v0.dir(' + v[0].dir + ')';\
          if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';\
          if (String(v[0].op).indexOf(\"connect-src 'none'\") < 0) return 'FAIL:v0.op(' + v[0].op + ')';\
          if (String(v[0].blocked).indexOf('wpt.test') < 0) return 'FAIL:v0.blocked(' + v[0].blocked + ')';\
          return 'PASS'; })()",
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k27 ws fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k27 WS 违规钉测失败：{}", fields);
}
