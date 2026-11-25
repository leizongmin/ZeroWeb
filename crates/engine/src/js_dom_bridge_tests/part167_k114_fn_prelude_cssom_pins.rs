// k114 css-mixins @function 定义文法钉测（2026-11-25）
// corpus 形蒸馏：WPT css-mixins/functions/at-function-parsing 全页形
// `new CSSStyleSheet().replaceSync(prelude + " {}")` → cssRules.length——
// 此前 shim replaceSync `}` 切分器空体丢弃使 valid 形恒 0 规则（48F）、
// invalid 形 vacuous 0 规则（38P）。钉面五向：
//   V1 valid 空体保留——prelude 文法门合法 → 1 规则物化；
//   V2 valid 形抽样（type() 联合/default 类型核验/# 乘数/returns 复合形）；
//   I1 invalid 形门拒——0 规则且 replaceSync 无异常语义（空体丢弃形恒同）；
//   T1 cssText 保形——with-body 形通用 type-1 路径今日 3P 红线零漂移；
//   R1 insertRule 写面——非法抛 SyntaxError DOMException，合法物化；
//   N1 负面——非 @function 空体丢弃粒度不变＋`@function-bar` 异名不误捕。
// https://drafts.csswg.org/css-mixins-1/#function-rule
const K114_FN_PRELUDE_CSSOM_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  function rulesOf(ss) { return ss.cssRules.length; }
  stage('V1', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --foo() {}');
    chk(eq(rulesOf(ss), 1, 'V1 空体 valid 保留 1 规则'));
  });
  stage('V2', function () {
    var forms = [
      '@function --foo(--x type(<length> | auto))',
      '@function --foo(--x <length>: 10px)',
      '@function --foo(--x <length>#)',
      '@function --foo(--x) returns type(<length> | auto)',
      '@function --foo(--x:1px, --y, --z:2px)',
    ];
    for (var i = 0; i < forms.length; i++) {
      var ss = new CSSStyleSheet();
      ss.replaceSync(forms[i] + ' {}');
      chk(eq(rulesOf(ss), 1, 'V2 valid: ' + forms[i]));
    }
  });
  stage('I1', function () {
    var forms = [
      '@function --foo(,)',
      '@function --foo(--x <dino>)',
      '@function --foo(--x <length>: 10deg)',
      '@function --foo(--x <transform-list>#)',
      '@function --foo(--x: 10px !important)',
      '@function --foo(--x) returns *',
    ];
    for (var i = 0; i < forms.length; i++) {
      var ss = new CSSStyleSheet();
      var threw = false;
      try { ss.replaceSync(forms[i] + ' {}'); } catch (e) { threw = true; }
      chk(isTrue(threw === false, 'I1 replaceSync 不抛: ' + forms[i]));
      chk(eq(rulesOf(ss), 0, 'I1 invalid 0 规则: ' + forms[i]));
    }
  });
  stage('T1', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --foo(--x <length>) { result: 1px; }');
    chk(eq(rulesOf(ss), 1, 'T1 with-body 物化'));
    chk(eq(ss.cssRules[0].cssText, '@function --foo(--x <length>) { result: 1px; }', 'T1 cssText 保形'));
  });
  stage('R1', function () {
    var ss = new CSSStyleSheet();
    var threw = null;
    try { ss.insertRule('@function --foo(,) { result: 1; }', 0); } catch (e) { threw = e; }
    chk(isTrue(threw != null && threw.name === 'SyntaxError', 'R1 非法抛 SyntaxError'));
    var idx = ss.insertRule('@function --foo(--x <length>: 10px) { result: var(--x); }', 0);
    chk(eq(idx, 0, 'R1 合法插返 0'));
    chk(eq(rulesOf(ss), 1, 'R1 合法物化 1 规则'));
  });
  stage('N1', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('div {}');
    chk(eq(rulesOf(ss), 0, 'N1 非 @function 空体丢弃粒度不变'));
    var ss2 = new CSSStyleSheet();
    ss2.replaceSync('@function-bar --x {}');
    chk(eq(rulesOf(ss2), 0, 'N1 @function-bar 异名不误捕'));
  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_k114_fn_prelude_cssom_pins() {
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
        "<html><head><title>k114pins</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k114.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(K114_FN_PRELUDE_CSSOM_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("k114 fn-prelude cssom execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k114 @function CSSOM 钉测失败：{}", verdict);
}
