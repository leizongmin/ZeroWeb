// s10k28 刀㉗ 常驻钉测（2026-11-26）：TT 违规 sample 形——sink 面
// 「sink 名|40 单元截断」＋createPolicy 名单 block 面（directive
// 'trusted-types'＋originalPolicy=名单政策原文＋sample 名截断无前缀）。
// corpus 形蒸馏：trusted-types-reporting-clipping-of-sample(.tentative) 20F 簇
//（39/40/41 单元边界＋surrogate 对尾单元延至 41＝Firefox 口径）。
// https://w3c.github.io/webappsec-trusted-types/dist/spec/#create-a-trusted-type-violation-record
// https://github.com/w3c/webappsec-csp/issues/704

/// 三 meta 执法页（镜像语料 clipping 页头：名单＋require-trusted-types-for＋
/// connect-src 各单 meta——验证 enforcePolicy/listPolicy 按指令归属记录）。
const S10K28_CLIP_PAGE_HTML: &str = "<html><head>\
  <meta http-equiv=\"Content-Security-Policy\" content=\"trusted-types 'none'\">\
  <meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script'\">\
  <meta http-equiv=\"Content-Security-Policy\" content=\"connect-src 'none'\">\
  <title>k28clip</title></head><body></body></html>";

#[test]
fn test_s10k28_sample_clip_pins() {
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
        Arc::new(Mutex::new(S10K28_CLIP_PAGE_HTML.to_string()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k28-clip.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 第一段：四违规触发（sink 41/40/42 单元＋createPolicy 名单 block）。
    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  try {{ div.innerHTML = 'A'.repeat(41); }} catch (e) {{}}
  try {{ div.innerHTML = 'B'.repeat(40); }} catch (e) {{}}
  try {{ div.innerHTML = '\u{{1D406}}'.repeat(21); }} catch (e) {{}}
  try {{ tt.createPolicy('X'.repeat(45), {{ createHTML: function (x) {{ return x; }} }}); }} catch (e) {{}}
  return 'TRIGGERED';
}})();"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k28 clip trigger execute error: {}", e),
    };
    assert_eq!(verdict, "TRIGGERED", "s10k28 clip 触发段失败：{}", verdict);
    // 驱动到期：四违规 task（setTimeout FIFO 单源）。
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        r#"(function () {
  var v = globalThis.__v27;
  if (v.length < 4) return 'FAIL:count(' + v.length + ')';
  var e0 = 'Element innerHTML|' + 'A'.repeat(40);
  if (v[0].sample !== e0) return 'FAIL:v0.sample(' + v[0].sample + ')';
  if (v[0].dir !== 'require-trusted-types-for') return 'FAIL:v0.dir(' + v[0].dir + ')';
  if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';
  if (v[0].op !== "require-trusted-types-for 'script'") return 'FAIL:v0.op(' + v[0].op + ')';
  var e1 = 'Element innerHTML|' + 'B'.repeat(40);
  if (v[1].sample !== e1) return 'FAIL:v1.sample(' + v[1].sample + ')';
  var e2 = 'Element innerHTML|' + '\u{1D406}'.repeat(20);
  if (v[2].sample !== e2) return 'FAIL:v2.sample(' + v[2].sample + ')';
  if (v[3].dir !== 'trusted-types') return 'FAIL:v3.dir(' + v[3].dir + ')';
  if (v[3].op !== "trusted-types 'none'") return 'FAIL:v3.op(' + v[3].op + ')';
  if (v[3].sample !== 'X'.repeat(40)) return 'FAIL:v3.sample(' + v[3].sample + ')';
  if (v[3].disp !== 'enforce') return 'FAIL:v3.disp(' + v[3].disp + ')';
  return 'PASS';
})()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k28 clip fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k28 sample 截断钉测失败：{}", fields);
}

/// 名单页（允许 DupName 单名）——createPolicy block 两面钉测载体。
const S10K28_POLICY_PAGE_HTML: &str = "<html><head>\
  <meta http-equiv=\"Content-Security-Policy\" content=\"trusted-types DupName\">\
  <meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script'\">\
  <title>k28policy</title></head><body></body></html>";

#[test]
fn test_s10k28_policy_block_pins() {
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
        Arc::new(Mutex::new(S10K28_POLICY_PAGE_HTML.to_string()));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k28-policy.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 第一段：重名 block（DupName 二次创建）＋名单外 block（OtherName）。
    let verdict = match sandbox.execute(&format!(
        r#"{prelude}{body}
  var ok = tt.createPolicy('DupName', {{ createHTML: function (x) {{ return x; }} }});
  if (ok.name !== 'DupName') return 'FAIL:firstName(' + ok.name + ')';
  var d1 = '';
  try {{ tt.createPolicy('DupName', {{ createHTML: function (x) {{ return x; }} }}); }} catch (e) {{ d1 = e && e.constructor && e.constructor.name; }}
  if (d1 !== 'TypeError') return 'FAIL:dupThrow(' + d1 + ')';
  var d2 = '';
  try {{ tt.createPolicy('OtherName', {{ createHTML: function (x) {{ return x; }} }}); }} catch (e) {{ d2 = e && e.constructor && e.constructor.name; }}
  if (d2 !== 'TypeError') return 'FAIL:gateThrow(' + d2 + ')';
  return 'TRIGGERED';
}})();"#, prelude = S10K27_PIN_PRELUDE_JS, body = S10K27_PIN_BODY_JS)) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k28 policy trigger execute error: {}", e),
    };
    assert_eq!(verdict, "TRIGGERED", "s10k28 policy 触发段失败：{}", verdict);
    // 驱动到期：重名 block ＋名单外 block 各一违规（'trusted-types' 指令）。
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    let fields = match sandbox.execute(
        r#"(function () {
  var v = globalThis.__v27;
  if (v.length < 2) return 'FAIL:count(' + v.length + ')';
  if (v[0].dir !== 'trusted-types') return 'FAIL:v0.dir(' + v[0].dir + ')';
  if (v[0].op !== 'trusted-types DupName') return 'FAIL:v0.op(' + v[0].op + ')';
  if (v[0].sample !== 'DupName') return 'FAIL:v0.sample(' + v[0].sample + ')';
  if (v[0].disp !== 'enforce') return 'FAIL:v0.disp(' + v[0].disp + ')';
  if (v[1].dir !== 'trusted-types') return 'FAIL:v1.dir(' + v[1].dir + ')';
  if (v[1].op !== 'trusted-types DupName') return 'FAIL:v1.op(' + v[1].op + ')';
  if (v[1].sample !== 'OtherName') return 'FAIL:v1.sample(' + v[1].sample + ')';
  return 'PASS';
})()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k28 policy fields execute error: {}", e),
    };
    assert_eq!(fields, "PASS", "s10k28 createPolicy block 钉测失败：{}", fields);
}
