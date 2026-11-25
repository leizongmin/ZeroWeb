// s10k29 刀㉙ eval 门常驻钉测（2026-11-26）：SHIMW universal eval 门
//（part14 §5.5，刀㉙返修二改惰性装置——非 enforced/非 CSP 页不装保页面直接
// eval 原生语义；语料经 part03 script prepare 装置点，钉测直驱沙箱显式
// __zwMaybeInstallEvalGate 触发）三 face ＋ Function 编译门违规派发。
// corpus 形蒸馏：eval 族 14 页 33F（no-csp/permissive/csp-no-tt/
// csp-tt-no-default-policy/csp-tt-default-policy/eval-reporting 簇/
// tt-block-eval/reporting-for-Window-eval·function-constructor）。
//   ① native 透传 face——非 String 非 TrustedScript 实参原样返回
//     （ECMA-262 PerformEval；eval(42)===42、eval({}) 全等、null/undefined 同值）；
//   ② TrustedScript face——isScript 判定 → ToString 转换（WebIDL wrapper 边界，
//     native eval 不做——probe1 P3 实证）→ 免 TT face（default policy 不被调用）
//     → 仍受 CSP face；
//   ③ String face——TT enforced 时 default policy createScript 四 block
//     （缺回调/抛错/返 undefined·null/mutate）→ violation(sink 'eval'，
//     sample＝原输入) + EvalError → CSP face（宿主 __zwCspEvalBlockedQuery
//     决策回调 '1' → script-src 违规（无 'eval|' 前缀、blockedURI 'eval'、
//     originalPolicy＝script-src 政策原文）+ EvalError）→ native；
//   ④ Function 编译门四 block 点违规派发（旧版只抛 EvalError 不派发——
//     reporting-for-Window-function-constructor 四 face 根因），sample＝装配
//     文本去 'function anonymous' 前缀（语料 `Function|(\n) {\n…\n}` 形）。
// 面三的宿主回调在 engine 钉测沙箱以 JS 桩替代（webview 侧为原生回调，
// s253 钉测同款复刻形）。
// https://w3c.github.io/webappsec-trusted-types/dist/spec/#eval-dom
// https://w3c.github.io/webappsec-csp/#ensure-csp-does-not-block-string-compilation

/// TT 执法页（无 default policy 面＋Function 编译门派发面共用）。
const S10K29_TT_PAGE_HTML: &str = "<html><head>\
  <meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script'\">\
  <title>k29eval</title></head><body></body></html>";

/// TT 执法＋default policy 行为开关页（identity/null/mutate 三态翻转变量）。
const S10K29_TT_DEFAULT_PAGE_HTML: &str = "<html><head>\
  <meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script'\">\
  <title>k29evaldef</title></head><body></body></html>";

/// CSP-only 页（script-src 无 unsafe-eval、无 TT——CSP face 面）。
const S10K29_CSP_PAGE_HTML: &str = "<html><head>\
  <meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'nonce-pin'\">\
  <title>k29evalcsp</title></head><body></body></html>";

/// 宿主 eval 决策回调 JS 桩（webview `__zwCspEvalBlockedQuery` 原生回调的
/// 钉测复刻形——eval-blocked 页恒 '1'，无派发副作用）。
const S10K29_CSP_QUERY_STUB_JS: &str =
  "globalThis.__zwCspEvalBlockedQuery = function () { return '1'; };";

#[test]
fn test_s10k29_eval_gate_tt_pins() {
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
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new(S10K29_TT_PAGE_HTML.to_string()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k29-tt.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var r = [];
  // 钉测直驱沙箱不走 part03 script prepare 装置点——显式触发惰性装置
  //（meta 判定同语料路径）。
  globalThis.__zwMaybeInstallEvalGate && globalThis.__zwMaybeInstallEvalGate();
  if (typeof globalThis.eval !== 'function') return 'FAIL:evalType(' + typeof globalThis.eval + ')';
  var e1 = '';
  try {{ eval('globalThis.__zzK29A = 1;'); }} catch (ex) {{ e1 = ex && ex.constructor && ex.constructor.name; }}
  if (e1 !== 'EvalError') return 'FAIL:p1.throw(' + e1 + ')';
  if (globalThis.__zzK29A !== undefined) return 'FAIL:p1.exec';
  var obj = {{ k: 7 }};
  var v3;
  try {{ v3 = eval(obj); }} catch (ex) {{ return 'FAIL:p3.throw(' + ex + ')'; }}
  if (v3 !== obj) return 'FAIL:p3.identity';
  if (eval(null) !== null) return 'FAIL:p3.null';
  if (eval(undefined) !== undefined) return 'FAIL:p3.undefined';
  var pol = tt.createPolicy('k29a', {{ createScript: function (x) {{ return x; }} }});
  var e4 = '';
  try {{ eval(pol.createScript('globalThis.__zzK29B = 2;')); }} catch (ex) {{ e4 = ex && ex.constructor && ex.constructor.name; }}
  if (e4 !== '') return 'FAIL:p4.throw(' + e4 + ')';
  if (globalThis.__zzK29B !== 2) return 'FAIL:p4.exec(' + globalThis.__zzK29B + ')';
  var v4s = '';
  var e5 = '';
  try {{ eval('x29 = x29'); }} catch (ex) {{ e5 = ex && ex.constructor && ex.constructor.name; }}
  if (e5 !== 'EvalError') return 'FAIL:p5.throw(' + e5 + ')';
  return 'TRIGGERED';
}})();"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 tt trigger execute error: {}", e),
    };
    assert_eq!(verdict, "TRIGGERED", "s10k29 eval TT 触发段失败：{}", verdict);
    // 驱动到期：P1（plain string）与 P5（未声明名读取抛 EvalError 前同面）
    // 各一违规——P3/P4 透传/TS 免 face 无违规。
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        r#"(function () {
  var v = globalThis.__v27;
  if (v.length !== 2) return 'FAIL:count(' + v.length + ')';
  if (v[0].dir !== 'require-trusted-types-for') return 'FAIL:v0.dir(' + v[0].dir + ')';
  if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';
  if (v[0].sample !== 'eval|globalThis.__zzK29A = 1;') return 'FAIL:v0.sample(' + v[0].sample + ')';
  if (String(v[0].op).indexOf("require-trusted-types-for 'script'") < 0) return 'FAIL:v0.op(' + v[0].op + ')';
  if (v[0].blocked !== 'trusted-types-sink') return 'FAIL:v0.blocked(' + v[0].blocked + ')';
  if (v[1].sample !== 'eval|x29 = x29') return 'FAIL:v1.sample(' + v[1].sample + ')';
  return 'PASS';
})()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 tt fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k29 eval TT 钉测失败：{}", fields);
}

#[test]
fn test_s10k29_eval_gate_default_policy_pins() {
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
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new(S10K29_TT_DEFAULT_PAGE_HTML.to_string()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k29-def.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  // 钉测直驱沙箱不走 part03 script prepare 装置点——显式触发惰性装置。
  globalThis.__zwMaybeInstallEvalGate && globalThis.__zwMaybeInstallEvalGate();
  globalThis.__zzK29Mode = 'identity';
  globalThis.__zzK29Calls = [];
  tt.createPolicy('default', {{ createScript: function (s, type, sink) {{
    globalThis.__zzK29Calls.push([String(s), String(type), String(sink)]);
    if (globalThis.__zzK29Mode === 'identity') return s;
    if (globalThis.__zzK29Mode === 'null') return null;
    return s + '!';
  }} }});
  var r = [];
  if (eval('29 + 13') !== 42) return 'FAIL:p1.value';
  if (__zzK29Calls.length !== 1) return 'FAIL:p1.calls(' + __zzK29Calls.length + ')';
  if (__zzK29Calls[0][0] !== '29 + 13' || __zzK29Calls[0][1] !== 'TrustedScript' || __zzK29Calls[0][2] !== 'eval')
    return 'FAIL:p1.args(' + JSON.stringify(__zzK29Calls[0]) + ')';
  var c2 = __zzK29Calls.length;
  if (eval(42) !== 42) return 'FAIL:p2.value';
  if (eval(null) !== null) return 'FAIL:p2.null';
  if (__zzK29Calls.length !== c2) return 'FAIL:p2.called';
  var pol = tt.createPolicy('k29b', {{ createScript: function (x) {{ return x; }} }});
  var c3 = __zzK29Calls.length;
  if (eval(pol.createScript('globalThis.__zzK29C = 3;')) !== 3) return 'FAIL:p3.value';
  if (__zzK29Calls.length !== c3) return 'FAIL:p3.called';
  globalThis.__zzK29Mode = 'mutate';
  var e4 = '';
  try {{ eval('zzMut'); }} catch (ex) {{ e4 = ex && ex.constructor && ex.constructor.name; }}
  if (e4 !== 'EvalError') return 'FAIL:p4.throw(' + e4 + ')';
  globalThis.__zzK29Mode = 'null';
  var e5 = '';
  try {{ eval('zzNull'); }} catch (ex) {{ e5 = ex && ex.constructor && ex.constructor.name; }}
  if (e5 !== 'EvalError') return 'FAIL:p5.throw(' + e5 + ')';
  return 'TRIGGERED';
}})();"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 def trigger execute error: {}", e),
    };
    assert_eq!(verdict, "TRIGGERED", "s10k29 eval default-policy 触发段失败：{}", verdict);
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        r#"(function () {
  var v = globalThis.__v27;
  if (v.length !== 2) return 'FAIL:count(' + v.length + ')';
  if (v[0].sample !== 'eval|zzMut') return 'FAIL:v0.sample(' + v[0].sample + ')';
  if (v[0].dir !== 'require-trusted-types-for') return 'FAIL:v0.dir(' + v[0].dir + ')';
  if (v[1].sample !== 'eval|zzNull') return 'FAIL:v1.sample(' + v[1].sample + ')';
  return 'PASS';
})()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 def fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k29 eval default-policy 钉测失败：{}", fields);
}

#[test]
fn test_s10k29_eval_gate_csp_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(S10K27_TIMER_STUB_JS).unwrap();
    sandbox.execute(S10K29_CSP_QUERY_STUB_JS).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new(S10K29_CSP_PAGE_HTML.to_string()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k29-csp.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var e1 = '';
  // 钉测直驱沙箱不走 part03 script prepare 装置点——显式触发惰性装置
  //（meta script-src 'nonce-pin' 判定）。
  globalThis.__zwMaybeInstallEvalGate && globalThis.__zwMaybeInstallEvalGate();
  try {{ eval('1 + 1'); }} catch (ex) {{ e1 = ex && ex.constructor && ex.constructor.name; }}
  if (e1 !== 'EvalError') return 'FAIL:p1.throw(' + e1 + ')';
  var pol = tt.createPolicy('k29c', {{ createScript: function (x) {{ return x; }} }});
  var e2 = '';
  try {{ eval(pol.createScript('2 + 2')); }} catch (ex) {{ e2 = ex && ex.constructor && ex.constructor.name; }}
  if (e2 !== 'EvalError') return 'FAIL:p2.throw(' + e2 + ')';
  // p3：宿主决策回调删除后 meta 兜底面仍 block（script-src 'nonce-pin' 无
  // 'unsafe-eval'——刀㉙返修 meta 兜底面，runner 语料无宿主回调同判）。
  delete globalThis.__zwCspEvalBlockedQuery;
  var e3 = '';
  try {{ eval('3 + 3'); }} catch (ex) {{ e3 = ex && ex.constructor && ex.constructor.name; }}
  if (e3 !== 'EvalError') return 'FAIL:p3.metaBlock(' + e3 + ')';
  var f = new Function('return 5 + 5');
  if (f() !== 10) return 'FAIL:p4.value';
  return 'TRIGGERED';
}})();"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 csp trigger execute error: {}", e),
    };
    assert_eq!(verdict, "TRIGGERED", "s10k29 eval CSP 触发段失败：{}", verdict);
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        r#"(function () {
  var v = globalThis.__v27;
  if (v.length !== 3) return 'FAIL:count(' + v.length + ')';
  if (v[0].dir !== 'script-src') return 'FAIL:v0.dir(' + v[0].dir + ')';
  if (v[0].sample !== '1 + 1') return 'FAIL:v0.sample(' + v[0].sample + ')';
  if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';
  if (v[0].blocked !== 'eval') return 'FAIL:v0.blocked(' + v[0].blocked + ')';
  if (v[0].op !== "script-src 'nonce-pin'") return 'FAIL:v0.op(' + v[0].op + ')';
  if (v[1].dir !== 'script-src') return 'FAIL:v1.dir(' + v[1].dir + ')';
  if (v[1].sample !== '2 + 2') return 'FAIL:v1.sample(' + v[1].sample + ')';
  // v2＝meta 兜底面（宿主回调删除后）——same shape，originalPolicy 仍 meta 原文。
  if (v[2].dir !== 'script-src') return 'FAIL:v2.dir(' + v[2].dir + ')';
  if (v[2].sample !== '3 + 3') return 'FAIL:v2.sample(' + v[2].sample + ')';
  if (v[2].blocked !== 'eval') return 'FAIL:v2.blocked(' + v[2].blocked + ')';
  if (v[2].op !== "script-src 'nonce-pin'") return 'FAIL:v2.op(' + v[2].op + ')';
  return 'PASS';
})()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 csp fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k29 eval CSP 钉测失败：{}", fields);
}

#[test]
fn test_s10k29_function_gate_violation_pins() {
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
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new(S10K29_TT_PAGE_HTML.to_string()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k29-fn.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var e1 = '';
  try {{ new Function('zzFn'); }} catch (ex) {{ e1 = ex && ex.constructor && ex.constructor.name; }}
  if (e1 !== 'EvalError') return 'FAIL:p1.throw(' + e1 + ')';
  return 'TRIGGERED';
}})();"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 fn trigger execute error: {}", e),
    };
    assert_eq!(verdict, "TRIGGERED", "s10k29 Function 门触发段失败：{}", verdict);
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        r#"(function () {
  var v = globalThis.__v27;
  if (v.length !== 1) return 'FAIL:count(' + v.length + ')';
  if (v[0].dir !== 'require-trusted-types-for') return 'FAIL:v0.dir(' + v[0].dir + ')';
  if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';
  if (v[0].sample !== 'Function|(\n) {\nzzFn\n}') return 'FAIL:v0.sample(' + JSON.stringify(v[0].sample) + ')';
  if (String(v[0].op).indexOf("require-trusted-types-for 'script'") < 0) return 'FAIL:v0.op(' + v[0].op + ')';
  if (v[0].blocked !== 'trusted-types-sink') return 'FAIL:v0.blocked(' + v[0].blocked + ')';
  return 'PASS';
})()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k29 fn fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k29 Function 门违规派发钉测失败：{}", fields);
}
