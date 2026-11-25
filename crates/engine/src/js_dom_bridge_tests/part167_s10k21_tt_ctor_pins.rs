// s10k21 刀㉑ Trusted 接口原型 constructor 钉测（2026-11-25）
// corpus 形蒸馏：WPT trusted-types/HTMLElement-generic 残 20F——Trusted
// 实例 `x.constructor.name` 须为接口名（WebIDL interface prototype object
// 强制自有 writable/non-enum/configurable constructor＝接口对象）。缺位时落
// Object.prototype 链读出 "Object"，测试以 `value.constructor.name !=
// trustedTypes.getPropertyType(elem, attr)` 计 expect_exception，把「sink 品牌集
// 正确接受匹配型 Trusted 值（不抛）」的正确行为误判成 fail（innerHTML/srcdoc/
// text/innerText/textContent × Trusted × connected 域；script.src 域 sink 类型
// 感知拒错型 Trusted 值不受影响＝保信蒸馏面）。钉面四向＋负面一页：
//   V1 constructor 恒等——TrustedHTML/TrustedScript/TrustedScriptURL/工厂；
//   V2 匹配型 Trusted 值 sink 接受不抛（−20 face 形：enforced 静态 meta 页）；
//   V3 plain 串 enforced sink 抛 TypeError（既有绿面保信——品牌集不因
//     constructor 补位误放行字符串）；
//   V4 类型感知保信——TrustedScript 值入 TrustedScriptURL sink（script.src）
//     仍拒（防过度接受回归）；
//   N1 无 meta 负面页——enforcement off plain 赋值不抛＋constructor 恒等与
//     enforcement 无关（独立沙箱两页互不污染闩锁闭包）。
// https://webidl.spec.whatwg.org/#interface-prototype-object
// https://w3c.github.io/trusted-types/dist/spec/#enforcement-in-scripts
const S10K21_TT_CTOR_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function throwsT(fn, m) {
    try { fn(); fails.push('FAIL:' + m + ' 未抛'); }
    catch (e) { if (!(e instanceof TypeError)) fails.push('FAIL:' + m + ' 抛非 TypeError:' + (e && e.name)); }
  }
  function noThrow(fn, m) { try { fn(); } catch (e) { fails.push('FAIL:' + m + ' 不应抛:' + (e && e.message || e)); } }
  var pp = trustedTypes.createPolicy('testpolicy', {
    createScript: function (s) { return s; },
    createHTML: function (s) { return s; },
    createScriptURL: function (s) { return s; }
  });
  // V1 constructor 恒等（刀面本体）
  chk(isTrue(pp.createHTML('<b>x</b>').constructor.name === 'TrustedHTML', 'V1 ctor TrustedHTML'));
  chk(isTrue(pp.createScript('2+2').constructor.name === 'TrustedScript', 'V1 ctor TrustedScript'));
  chk(isTrue(pp.createScriptURL('https://x.test/').constructor.name === 'TrustedScriptURL', 'V1 ctor TrustedScriptURL'));
  chk(isTrue(trustedTypes.constructor.name === 'TrustedTypePolicyFactory', 'V1 ctor 工厂'));
  chk(isTrue(pp.constructor.name === 'TrustedTypePolicy', 'V1 ctor 政策'));
  // V2 匹配型 Trusted 值接受不抛（enforced 页）
  var th = pp.createHTML('<b>hello</b>');
  var div = document.createElement('div');
  var sc = document.createElement('script');
  var fr = document.createElement('iframe');
  noThrow(function () { div.innerHTML = th; }, 'V2 innerHTML trusted');
  noThrow(function () { fr.srcdoc = th; }, 'V2 srcdoc trusted');
  noThrow(function () { sc.text = pp.createScript('2+2'); }, 'V2 text trusted');
  noThrow(function () { sc.innerText = pp.createScript('2+3'); }, 'V2 innerText trusted');
  noThrow(function () { sc.textContent = pp.createScript('2+4'); }, 'V2 textContent trusted');
  // V3 plain 串 enforced sink 抛 TypeError（既有绿面保信）
  throwsT(function () { div.innerHTML = '<b>x</b>'; }, 'V3 innerHTML plain');
  throwsT(function () { fr.srcdoc = '<p>x</p>'; }, 'V3 srcdoc plain');
  throwsT(function () { sc.text = 'p1'; }, 'V3 text plain');
  throwsT(function () { sc.innerText = 'p2'; }, 'V3 innerText plain');
  throwsT(function () { sc.textContent = 'p3'; }, 'V3 textContent plain');
  // V4 类型感知保信：TrustedScript 值入 TrustedScriptURL sink 仍拒
  throwsT(function () { sc.src = pp.createScript('https://x.test/'); }, 'V4 src 拒 TrustedScript');
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

// 负面页：无 meta——enforcement off，plain 赋值不抛；constructor 恒等与强制无关
const S10K21_TT_CTOR_NEG_PIN_JS: &str = r##"
(function () {
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function noThrow(fn, m) { try { fn(); } catch (e) { fails.push('FAIL:' + m + ' 不应抛:' + (e && e.message || e)); } }
  var pp = trustedTypes.createPolicy('negpolicy', {
    createScript: function (s) { return s; },
    createHTML: function (s) { return s; }
  });
  chk(isTrue(pp.createHTML('<b>y</b>').constructor.name === 'TrustedHTML', 'N1 ctor 恒等不受强制影响'));
  chk(isTrue(globalThis.__zwTtEnforced() === false, 'N1 无 meta 页 enforcement false'));
  var div = document.createElement('div');
  var sc = document.createElement('script');
  noThrow(function () { div.innerHTML = '<b>x</b>'; }, 'N1 innerHTML plain');
  noThrow(function () { sc.text = 'p1'; }, 'N1 text plain');
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k21_tt_ctor_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // HTMLElement-generic phase 2/3 同构基座：CSP meta 在场（enforced 静态形）
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k21ctor</title><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"></head><body><div id=\"k21host\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k21.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K21_TT_CTOR_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k21 ctor execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k21 Trusted ctor 钉测失败：{}", verdict);
}

#[test]
fn test_s10k21_tt_ctor_negative_pins() {
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
        "<html><head><title>k21ctorneg</title></head><body><div id=\"k21neg\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k21-neg.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K21_TT_CTOR_NEG_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k21 ctor-neg execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k21 Trusted ctor 负面钉测失败：{}", verdict);
}
