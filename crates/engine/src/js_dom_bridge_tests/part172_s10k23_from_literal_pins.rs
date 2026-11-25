// s10k23 刀㉓ fromLiteral 静态方法常驻钉测（2026-11-25）
// corpus 形蒸馏：WPT trusted-types/trusted-types-from-literal.tentative 21F 收敛
//（45/0）——三数据接口（TrustedHTML/TrustedScript/TrustedScriptURL）缺静态
// fromLiteral（旧 `"fromLiteral" in type` false + works 面 TypeError；21 throws
// 面缺方法 TypeError 意外绿）。钉面五向＋负面一页：
//   V1 works 基形——instanceof 接口＋toString 精确值（basic/empty/multi-line/
//     emoji cooked 转义形）＋`"fromLiteral" in type` 存在面；
//   V2 归一化域——TrustedHTML parse-then-serialize（`<b>`→`<b></b>`），
//     TrustedScript/TrustedScriptURL 不归一化（原样直通）；
//   V3 throws 保信——非模板形态（string/null/undefined/对象/普通数组/frozen
//     数组）＋表达式形态（substitutions 非空）＋non-literal with param 全
//     TypeError（旧缺方法意外绿面落刀后以真 TypeError 保持）；
//   V4 bound method——bind 后调用（实现不依赖 this）；
//   V5 品牌保真——产物入 enforced sink 不抛（品牌集命中）＋isHTML 判定；
//   N 负面页——无 meta：fromLiteral 构造与强制无关＋throws 形不变。
// https://w3c.github.io/trusted-types/dist/spec/#dom-trustedhtml-fromliteral
const S10K23_FL_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function throwsT(fn, m) {
    try { fn(); fails.push('FAIL:' + m + ' 未抛'); }
    catch (e) { if (!(e instanceof TypeError)) fails.push('FAIL:' + m + ' 抛非 TypeError:' + (e && e.name)); }
  }
  var types = [TrustedHTML, TrustedScript, TrustedScriptURL];
  // V1 works 基形 + 存在面
  for (var ti = 0; ti < types.length; ti++) {
    (function (type) {
      chk(isTrue('fromLiteral' in type, 'V1 in ' + type.name));
      var c1 = type.fromLiteral`abc`;
      chk(isTrue(c1 instanceof type, 'V1 instanceof ' + type.name));
      chk(eq(c1.toString(), 'abc', 'V1 basic ' + type.name));
      chk(eq(type.fromLiteral``.toString(), '', 'V1 empty ' + type.name));
      chk(isTrue(type.fromLiteral`A\nB`.toString().indexOf('\n') >= 0, 'V1 multiline ' + type.name));
      chk(eq(type.fromLiteral`\u{1f4a9}`.toString(), '\u{1f4a9}', 'V1 emoji ' + type.name));
      // V3 throws 保信
      throwsT(function () { type.fromLiteral('abc'); }, 'V3 string ' + type.name);
      throwsT(function () { type.fromLiteral(null); }, 'V3 null ' + type.name);
      throwsT(function () { type.fromLiteral(undefined); }, 'V3 undefined ' + type.name);
      throwsT(function () { type.fromLiteral({}); }, 'V3 object ' + type.name);
      throwsT(function () { type.fromLiteral(['abc']); }, 'V3 array ' + type.name);
      throwsT(function () { type.fromLiteral(Object.freeze(['abc'])); }, 'V3 frozen ' + type.name);
      throwsT(function () { type.fromLiteral`Hello ${2 + 3} world`; }, 'V3 expr ' + type.name);
      throwsT(function () { type.fromLiteral(['abc', 'def'], 'xxx'); }, 'V3 param ' + type.name);
      // V4 bound method
      var tag = type.fromLiteral.bind(type);
      chk(eq(tag`abc`.toString(), 'abc', 'V4 bound ' + type.name));
    })(types[ti]);
  }
  // V2 归一化域（仅 TrustedHTML）
  chk(eq(TrustedHTML.fromLiteral`<b>`.toString(), '<b></b>', 'V2 HTML 归一化'));
  chk(eq(TrustedScript.fromLiteral`<b>`.toString(), '<b>', 'V2 Script 不归一化'));
  chk(eq(TrustedScriptURL.fromLiteral`<b>`.toString(), '<b>', 'V2 ScriptURL 不归一化'));
  // V5 品牌保真——产物入 enforced sink 不抛（品牌集命中）
  var div = document.createElement('div');
  try { div.innerHTML = TrustedHTML.fromLiteral`<i>ok</i>`; }
  catch (e) { fails.push('FAIL:V5 sink 接受 fromLiteral 产物:' + (e && e.message || e)); }
  chk(isTrue(trustedTypes.isHTML(TrustedHTML.fromLiteral`x`), 'V5 isHTML'));
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

// 负面页：无 meta——fromLiteral 构造与强制无关，throws 形不变
const S10K23_FL_NEG_PIN_JS: &str = r##"
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
  chk(isTrue(globalThis.__zwTtEnforced() === false, 'N0 无 meta 页 enforcement false'));
  chk(eq(TrustedHTML.fromLiteral`<b>`.toString(), '<b></b>', 'N1 归一化与强制无关'));
  chk(eq(TrustedScript.fromLiteral`p1`.toString(), 'p1', 'N1 Script works'));
  noThrow(function () { TrustedScriptURL.fromLiteral`https://x.test/`; }, 'N1 ScriptURL works');
  throwsT(function () { TrustedHTML.fromLiteral('plain'); }, 'N2 throws 形不变');
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k23_from_literal_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // from-literal 同构基座：CSP meta 在场（enforced 静态形）
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k23fl</title><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"></head><body><div id=\"k23host\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k23.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K23_FL_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k23 fromLiteral execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k23 fromLiteral 钉测失败：{}", verdict);
}

#[test]
fn test_s10k23_from_literal_negative_pins() {
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
        "<html><head><title>k23flneg</title></head><body><div id=\"k23neg\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k23-neg.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K23_FL_NEG_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k23 fromLiteral-neg execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k23 fromLiteral 负面钉测失败：{}", verdict);
}
