// k115 css-mixins @function CSSOM 结构化面钉测（2026-11-25）
// corpus 形蒸馏：WPT css-mixins/functions/at-function-cssom 全页形——
// 此前 @function 走 type-1 保形物化（CSSFunctionRule 接口缺位 32F）。
// 钉面六向：
//   C1 cssText canonical——空体 `{ }` 单空格/returns 保形与 type(*) 剥壳/
//      param type() 展开/双 result 折叠/escape 重编码 `\9 ` TAB 形/
//      `I\ dent` 空格 ident 形＋k114 3P 红线恒同；
//   C2 接口面——name decoded TAB 形/getParameters() 六函数形（type 剥壳、
//      `*` 缺省、defaultValue 键有值才有）/returnType 五形；
//   D1 Descriptors 读面——result/getPropertyValue/--unknown 空/length/item/
//      Symbol.iterator/cssText canonical；
//   D2 Unknown descriptors 丢弃＋cssText setter mutation（白名单重解析＋
//      rule.cssText prelude 拼装重算）；
//   E1 结构——Empty cssRules 0/Single [CSSFunctionDeclarations].style
//      CSSFunctionDescriptors instanceof 三壳；
//   N1 嵌套 @supports 体不抛（replaceSync `}` 切分器粒度）＋k146 条件组物化
//      （cssRules = [CSSSupportsRule, CSSFunctionDeclarations]——k124 held-F
//      锚 1 翻转 2）；
//   N2 k146 树形臂边界——bare run 聚合 last-wins/嵌套 supports 递归/
//      @media @container skip 省略。
// https://drafts.csswg.org/css-mixins-1/#function-rule
const K115_FN_CSSOM_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  function objEq(a, b, m) { if (JSON.stringify(a) !== JSON.stringify(b)) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  stage('C1', function () {
    var cases = [
      ['@function --empty() { }', '@function --empty() { }'],
      ['@function --ret-length() returns <length> { }', '@function --ret-length() returns <length> { }'],
      ['@function --ret-auto() returns type(<length> | auto) { }', '@function --ret-auto() returns type(<length> | auto) { }'],
      ['@function --ret-star() returns type(*) { }', '@function --ret-star() { }'],
      ['@function --param-typed(--x <length>) { }', '@function --param-typed(--x <length>) { }'],
      ['@function --param-type-fn(--x type(<length>)) { }', '@function --param-type-fn(--x <length>) { }'],
      ['@function --param-type-fn-uni(--x type(*)) { }', '@function --param-type-fn-uni(--x) { }'],
      ['@function --param-default(--x: 10px) { }', '@function --param-default(--x: 10px) { }'],
      ['@function --param-multi-mixed(--x: 10px, --y, --z <length>) { }', '@function --param-multi-mixed(--x: 10px, --y, --z <length>) { }'],
      ['@function --body-result() { result: 10px; }', '@function --body-result() { result: 10px; }'],
      ['@function --body-locals() { --x: 1px; --y: 2px; result: 10px; }', '@function --body-locals() { --x: 1px; --y: 2px; result: 10px; }'],
      ['@function --body-result-multi() { result: 10px; result: 20px; }', '@function --body-result-multi() { result: 20px; }'],
      ['@function --identy(--a I\\ dent) returns type(I\\ dent) { }', '@function --identy(--a I\\ dent) returns I\\ dent { }']
    ];
    for (var i = 0; i < cases.length; i++) {
      var ss = new CSSStyleSheet();
      ss.replaceSync(cases[i][0]);
      chk(eq(ss.cssRules.length, 1, 'C1 rules: ' + cases[i][0]));
      if (ss.cssRules.length === 1) {
        chk(eq(ss.cssRules[0].cssText, cases[i][1], 'C1 cssText: ' + cases[i][0]));
        chk(isTrue(ss.cssRules[0] instanceof CSSFunctionRule, 'C1 instanceof: ' + cases[i][0]));
      }
    }
    var ssT = new CSSStyleSheet();
    ssT.replaceSync('@function --escaped-\\9 -tab(--param-\\9 -tab) { --local-\\9 -tab: 1px; }');
    chk(eq(ssT.cssRules[0].cssText,
      '@function --escaped-\\9 -tab(--param-\\9 -tab) { --local-\\9 -tab: 1px; }',
      'C1 escapes TAB 保形'));
  });
  stage('C2', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --f0() {} @function --f1(--x) {} @function --f2(--x <length>) {} @function --f3(--x: 10px) {} @function --f4(--x <length>: 10px) {} @function --f5(--x type(<length> | auto): 10px, --y, --z: red) {}');
    chk(eq(ss.cssRules.length, 6, 'C2 六规则'));
    chk(eq(ss.cssRules[0].name, '--f0', 'C2 name'));
    chk(objEq(ss.cssRules[0].getParameters(), [], 'C2 f0'));
    chk(objEq(ss.cssRules[1].getParameters(), [{ name: '--x', type: '*' }], 'C2 f1'));
    chk(objEq(ss.cssRules[2].getParameters(), [{ name: '--x', type: '<length>' }], 'C2 f2'));
    chk(objEq(ss.cssRules[3].getParameters(), [{ name: '--x', type: '*', defaultValue: '10px' }], 'C2 f3'));
    chk(objEq(ss.cssRules[4].getParameters(), [{ name: '--x', type: '<length>', defaultValue: '10px' }], 'C2 f4'));
    chk(objEq(ss.cssRules[5].getParameters(), [
      { name: '--x', type: '<length> | auto', defaultValue: '10px' },
      { name: '--y', type: '*' },
      { name: '--z', type: '*', defaultValue: 'red' }
    ], 'C2 f5'));
    var sr = new CSSStyleSheet();
    sr.replaceSync('@function --g0() {} @function --g1() returns <length> {} @function --g2() returns <length>+ {} @function --g3() returns type(*) {} @function --g4() returns type(<length> | auto) {}');
    chk(eq(sr.cssRules[0].returnType, '*', 'C2 g0'));
    chk(eq(sr.cssRules[1].returnType, '<length>', 'C2 g1'));
    chk(eq(sr.cssRules[2].returnType, '<length>+', 'C2 g2'));
    chk(eq(sr.cssRules[3].returnType, '*', 'C2 g3'));
    chk(eq(sr.cssRules[4].returnType, '<length> | auto', 'C2 g4'));
    var se = new CSSStyleSheet();
    se.replaceSync('@function --escaped-\\9 -tab(--param-\\9 -tab) { --local-\\9 -tab: 1px; }');
    chk(eq(se.cssRules[0].name, '--escaped-\t-tab', 'C2 escapes name decoded'));
    chk(eq(se.cssRules[0].getParameters()[0].name, '--param-\t-tab', 'C2 escapes param decoded'));
    chk(eq(se.cssRules[0].cssRules[0].style.getPropertyValue('--local-\t-tab'), '1px', 'C2 escapes local 读'));
  });
  stage('D1', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --foo() { --x: 1px; --y: 2px; result: 3px; }');
    var d = ss.cssRules[0].cssRules[0].style;
    chk(isTrue(d instanceof CSSFunctionDescriptors, 'D1 instanceof'));
    chk(eq(d.result, '3px', 'D1 result'));
    chk(eq(d.getPropertyValue('--x'), '1px', 'D1 --x'));
    chk(eq(d.getPropertyValue('--unknown'), '', 'D1 unknown 空'));
    chk(eq(d.length, 3, 'D1 length'));
    chk(eq(d.item(0), '--x', 'D1 item0'));
    chk(eq(d.item(2), 'result', 'D1 item2'));
    chk(objEq(Array.from(d), ['--x', '--y', 'result'], 'D1 iterator'));
    chk(eq(d.cssText, '--x: 1px; --y: 2px; result: 3px;', 'D1 cssText'));
  });
  stage('D2', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --foo() { --x: 1px; syntax: "<length>"; --y: 2px; color: red; unknown: unknown; result: 3px; }');
    var d = ss.cssRules[0].cssRules[0].style;
    chk(eq(d.length, 3, 'D2 unknown 丢弃'));
    chk(eq(d.cssText, '--x: 1px; --y: 2px; result: 3px;', 'D2 canonical'));
    d.cssText = '--x: 1px; color: red; result: 3px; --y: 2px;';
    chk(eq(d.length, 3, 'D2 setter'));
    chk(eq(d.cssText, '--x: 1px; result: 3px; --y: 2px;', 'D2 setter canonical'));
    chk(eq(d.getPropertyValue('--y'), '2px', 'D2 setter --y'));
    chk(eq(ss.cssRules[0].cssText, '@function --foo() { --x: 1px; result: 3px; --y: 2px; }', 'D2 rule.cssText 重算'));
  });
  stage('E1', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --foo() {}');
    chk(eq(ss.cssRules.length, 1, 'E1 empty 1 规则'));
    chk(isTrue(ss.cssRules[0] instanceof CSSFunctionRule, 'E1 empty instanceof'));
    chk(eq(ss.cssRules[0].cssRules.length, 0, 'E1 empty cssRules 0'));
    var ss2 = new CSSStyleSheet();
    ss2.replaceSync('@function --foo() { result: 100px; }');
    chk(isTrue(ss2.cssRules[0].cssRules[0] instanceof CSSFunctionDeclarations, 'E1 declarations'));
    chk(isTrue(ss2.cssRules[0].cssRules[0].style instanceof CSSFunctionDescriptors, 'E1 descriptors'));
  });
  stage('N1', function () {
    var ss = new CSSStyleSheet();
    var threw = false;
    try {
      ss.replaceSync('@function --foo() { @supports (width: 100px) { result: 100px; } result: 200px; }');
    } catch (e) { threw = true; }
    chk(isTrue(threw === false, 'N1 嵌套体不抛'));
    chk(eq(ss.cssRules.length, 1, 'N1 1 规则'));
    chk(isTrue(ss.cssRules[0] instanceof CSSFunctionRule, 'N1 instanceof'));
    // k146 形（css-mixins-1 §conditional-rules CSSOM——at-function-cssom 判据
    // 形）：cssRules = [CSSSupportsRule, CSSFunctionDeclarations]——条件组
    // 物化臂兑现（k124 held-F 锚 1 翻转为 2）。
    chk(eq(ss.cssRules[0].cssRules.length, 2, 'N1 k146 子规则 2'));
    chk(isTrue(ss.cssRules[0].cssRules[0] instanceof CSSSupportsRule, 'N1 supports 臂'));
    chk(eq(ss.cssRules[0].cssRules[0].type, 12, 'N1 type 12'));
    chk(eq(ss.cssRules[0].cssRules[0].conditionText, '(width: 100px)', 'N1 conditionText'));
    chk(isTrue(ss.cssRules[0].cssRules[0].cssRules[0] instanceof CSSFunctionDeclarations, 'N1 内层 declarations'));
    chk(eq(ss.cssRules[0].cssRules[0].cssRules[0].style.result, '100px', 'N1 内层 result'));
    chk(isTrue(ss.cssRules[0].cssRules[1] instanceof CSSFunctionDeclarations, 'N1 bare 臂'));
    chk(eq(ss.cssRules[0].cssRules[1].style.result, '200px', 'N1 bare result'));
  });
  stage('N2', function () {
    // k146 树形臂边界：bare 前置 run 聚合（last-wins 折叠）＋嵌套 supports
    // 递归＋@media/@container skip 省略（FIXME(k146+) 物化面）。
    var ss = new CSSStyleSheet();
    ss.replaceSync('@function --n2a() { --x: 1px; --x: 2px; @supports (display: grid) { --y: 3px; } }');
    var r0 = ss.cssRules[0];
    chk(eq(r0.cssRules.length, 2, 'N2a bare 前置 [decls, supports]'));
    chk(isTrue(r0.cssRules[0] instanceof CSSFunctionDeclarations, 'N2a bare run declarations'));
    chk(eq(r0.cssRules[0].style.getPropertyValue('--x'), '2px', 'N2a run 内 last-wins'));
    chk(isTrue(r0.cssRules[1] instanceof CSSSupportsRule, 'N2a supports 臂'));
    chk(eq(r0.cssRules[1].cssRules[0].style.getPropertyValue('--y'), '3px', 'N2a 内层 local'));
    var ss2 = new CSSStyleSheet();
    ss2.replaceSync('@function --n2b() { @supports (a: b) { @supports (c: d) { result: 1px; } } }');
    var s1 = ss2.cssRules[0].cssRules[0];
    chk(isTrue(s1 instanceof CSSSupportsRule, 'N2b 外层 supports'));
    chk(isTrue(s1.cssRules[0] instanceof CSSSupportsRule, 'N2b 嵌套 supports'));
    chk(eq(s1.cssRules[0].cssRules[0].style.result, '1px', 'N2b 深层 result'));
    var ss3 = new CSSStyleSheet();
    ss3.replaceSync('@function --n2c() { @media screen { result: 1px; } @container (width > 1px) { --z: 2px; } result: 3px; }');
    var rc = ss3.cssRules[0];
    chk(eq(rc.cssRules.length, 1, 'N2c skip 省略 bare 单臂'));
    chk(isTrue(rc.cssRules[0] instanceof CSSFunctionDeclarations, 'N2c declarations'));
    chk(eq(rc.cssRules[0].style.result, '3px', 'N2c result 折叠'));
  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_k115_fn_cssom_pins() {
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
        "<html><head><title>k115pins</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k115.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(K115_FN_CSSOM_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("k115 fn-cssom execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k115 @function CSSOM 结构化面钉测失败：{}", verdict);
}

#[test]
fn test_k115_fn_cssom_fallback_pins() {
    // 回调缺席回落臂：__zw_parse_function_rule 未注册（旧引擎/quickjs 腿形）→
    // @function 通用 type-1 保形物化（k114 形零漂移——非 CSSFunctionRule）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let verdict = match sandbox.execute(
        r#"(function () {
          var ss = new CSSStyleSheet();
          ss.replaceSync('@function --foo(--x <length>) { result: 1px; }');
          if (ss.cssRules.length !== 1) return 'FAIL:1rule';
          if (ss.cssRules[0] instanceof CSSFunctionRule) return 'FAIL:should-be-type1';
          if (ss.cssRules[0].selectorText !== '@function --foo(--x <length>)') return 'FAIL:selectorText';
          if (ss.cssRules[0].cssText !== '@function --foo(--x <length>) { result: 1px; }') return 'FAIL:cssText';
          return 'PASS';
        })()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("k115 fallback execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k115 回调缺席回落钉测失败：{}", verdict);
}
