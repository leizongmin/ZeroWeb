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

#[test]
fn test_k147_fn_wire_pins() {
    // wire 单源通道＋flushToOwner 重建保序（k147：function-invalidation
    // 『Prepending a rule』靶面——此前 Rule::Function 恒不入 wire，`<style>` 表
    // client cache 恒缺 @function 条目 → 任意 insertRule/deleteRule 后
    // flushToOwner 按 cache 重建元素文本**销毁原始规则**）。engine 侧
    // DomMutation::SetText 断言重建文本保双规则文档序（同层后扫胜＝原规则
    // 在后仍胜＝Prepending 后 width 恒 10px）。
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
        "<html><head><style id=s1>@function --f() { result: 10px; } #target { width: --f(); }</style></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k147-wire.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(
        r#"(function () {
          var fails = [];
          function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
          function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
          function chk(r) { if (r) fails.push(r); }
          var ss = document.getElementById('s1').sheet;
          if (!ss) return 'FAIL:no-sheet';
          chk(eq(ss.cssRules.length, 2, 'wire rules'));
          if (ss.cssRules.length === 2) {
            chk(isTrue(ss.cssRules[0] instanceof CSSFunctionRule, 'wire [0] brand'));
            chk(eq(ss.cssRules[0].name, '--f', 'wire name'));
            chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 10px; }', 'wire cssText'));
            chk(eq(ss.cssRules[1].selectorText, '#target', 'wire [1] selector'));
          }
          // 写面（k147 核心行为）：prepend 后 cache 保双规则文档序。
          ss.insertRule('@function --f() { result: 20px; }', 0);
          chk(eq(ss.cssRules.length, 3, 'after insert 3'));
          chk(eq(ss.cssRules[0].cssRules[0].style.result, '20px', 'inserted first'));
          chk(eq(ss.cssRules[1].cssRules[0].style.result, '10px', 'original second'));
          return fails.length ? fails.join('|') : 'PASS';
        })()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("k147 fn-wire execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k147 fn wire 通道钉测失败：{}", verdict);
    // engine 侧：flushToOwner SetText 重建文本保双规则文档序（原始规则在后
    // ——销毁形回归即此断言红）。
    let mts = mutations.lock().unwrap_or_else(|e| e.into_inner());
    let set_text = mts.iter().rev().find_map(|m| match m {
        DomMutation::SetText { text, .. } => Some(text.clone()),
        _ => None,
    });
    let text = set_text.expect("k147 flush SetText 缺席");
    let i20 = text
        .find("@function --f() { result: 20px; }")
        .expect("k147 重建文本缺新规则");
    let i10 = text
        .find("@function --f() { result: 10px; }")
        .expect("k147 重建文本缺原规则（销毁形回归）");
    assert!(
        i20 < i10,
        "k147 重建文本序错：新规则须在前（{} < {}）",
        i20,
        i10
    );
}

#[test]
fn test_k147_fn_wire_pending_overlay_pins() {
    // wire 读面 pending 叠加（k147 第二重根因）：setText（drain 前不入快照）后
    // 冷缓存 cssRules 读——快照路径拿旧文本（判据页 cleanup replaceChild 后新
    // sheet 冷缓存重读 wire 拿旧元素双规则 → flush 混入陈旧规则 → 同名 @function
    // 后扫胜裁决翻转）。叠加后读面须见 pending 文本；未触碰的同页其他表零串扰。
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
        "<html><head><style id=s1>@function --f() { result: 10px; }</style><style id=s2>@function --g() { result: 30px; }</style></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k147-wire-overlay.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(
        r#"(function () {
          var fails = [];
          function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
          function chk(r) { if (r) fails.push(r); }
          // setText（同批 SetText pending，快照仍旧文本）后首次 sheet 访问＝冷缓存 wire 读。
          var el = document.getElementById('s1');
          el.textContent = '@function --f() { result: 20px; }';
          var rs = el.sheet.cssRules;
          chk(eq(rs.length, 1, 'overlay rules'));
          if (rs.length === 1) {
            chk(eq(rs[0].cssText, '@function --f() { result: 20px; }', 'overlay cssText'));
          }
          // 零串扰：未触碰的 s2 表读面恒等（同批非空队列不误改他表答案）。
          var rs2 = document.getElementById('s2').sheet.cssRules;
          chk(eq(rs2.length, 1, 'untouched rules'));
          if (rs2.length === 1) {
            chk(eq(rs2[0].cssText, '@function --g() { result: 30px; }', 'untouched cssText'));
          }
          return fails.length ? fails.join('|') : 'PASS';
        })()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("k147 wire overlay execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k147 wire pending 叠加钉测失败：{}", verdict);
    // engine 侧：同批 SetText 确实 pending（快照未 drain——叠加通道的输入前提）。
    let mts = mutations.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        mts.iter()
            .any(|m| matches!(m, DomMutation::SetText { text, .. } if text.contains("result: 20px;"))),
        "k147 overlay 前提缺 pending SetText"
    );
}

#[test]
fn test_k147_fn_wire_stale_rebuild_pins() {
    // 暖缓存失效重建钉（k147 第三重根因）：id 全局绑定按换代快照（NPO backing
    // drain 换代刷新），cleanup replaceChild 后 bare id 仍绑旧元素 wrapper → 同一
    // sheet 对象暖缓存短路（`if (rulesCache) return`）→ flushToOwner 按混入陈旧
    // 规则的缓存重建文本（function-invalidation『Prepending a rule』got 20px 直证
    // ——沙盒无级联，以「pending SetText 后暖缓存读面」锁形：失效谓词弃缓存 →
    // 重建走 pending 叠加 wire → 新文本）。无关 selector pending 不改写本表文本。
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
        "<html><head><style id=s1>@function --f() { result: 10px; }</style></head><body><p id=p1>a</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k147-wire-stale.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(
        r#"(function () {
          var fails = [];
          function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
          function chk(r) { if (r) fails.push(r); }
          var ss = document.getElementById('s1').sheet;
          chk(eq(ss.cssRules.length, 1, 'initial'));
          chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 10px; }', 'initial cssText'));
          // 暖缓存 + pending SetText 改写本表文本 → 谓词 '1' → 弃缓存重建（叠加 wire）。
          chk(eq(__zw_style_rules_stale('#s1'), '0', 'predicate before'));
          document.getElementById('s1').textContent = '@function --f() { result: 20px; }';
          chk(eq(__zw_style_rules_stale('#s1'), '1', 'predicate after settext'));
          chk(eq(ss.cssRules.length, 1, 'stale rebuild'));
          chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 20px; }', 'stale rebuild cssText'));
          // 无关 selector pending 追加（沙盒无 drain——s1 的 SetText 仍 pending，
          // 谓词按「本 selector pending apply ≠ 快照」持续 '1' 属正确口径）；读面
          // 经重建走叠加 wire，内容恒随 pending 真值（不因他表变更漂移）。
          document.getElementById('p1').textContent = 'b';
          chk(eq(__zw_style_rules_stale('#s1'), '1', 'predicate pending persistent'));
          chk(eq(ss.cssRules.length, 1, 'no-crosstalk'));
          chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 20px; }', 'no-crosstalk cssText'));
          return fails.length ? fails.join('|') : 'PASS';
        })()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("k147 stale rebuild execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k147 暖缓存失效重建钉测失败：{}", verdict);
}

#[test]
fn test_k147_fn_wire_stale_structural_pins() {
    // 结构臂钉（k147 第三重根因补全）：cleanup replaceChild 同 id 同文本换元形态
    //（diag6/probe5 实证——候选文本恒等时文本差分谓词 '0'，sheet 暖缓存跨换代
    // 短路 → deleteRule 按陈旧缓存删错条目，gCS 恒旧值）。失效谓词加结构臂：
    // pending 结构类变异在场即 '1'（宁多重建不漏换代），重建读面走 pending 叠加
    // wire 恒取当下真值——换代但文本不变，规则内容不漂。
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
        "<html><head><style id=s1>@function --f() { result: 10px; }</style></head><body><p id=p1>a</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k147-wire-stale-structural.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(
        r#"(function () {
          var fails = [];
          function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
          function chk(r) { if (r) fails.push(r); }
          var ss = document.getElementById('s1').sheet;
          chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 10px; }', 'initial'));
          chk(eq(__zw_style_rules_stale('#s1'), '0', 'predicate clean queue'));
          // cleanup 同文本换元 pending 态（判据页 cleanup 形态：createElement +
          // setAttribute(id) + textContent(原文) + 位置线 replaceChild）。
          var ne = document.createElement('style');
          ne.setAttribute('id', 's1');
          ne.textContent = '@function --f() { result: 10px; }';
          __zw_insert_child_at('head', '0', ne.__zwHandle);
          __zw_remove_child_at('head', '1');
          // 文本恒等（叠加后 overlay 文本 == 快照文本）→ 文本臂 '0'；结构臂在场
          // → 谓词 '1' → 暖缓存弃置重建。
          chk(eq(__zw_style_rules_stale('#s1'), '1', 'structural same-text stale'));
          chk(eq(ss.cssRules.length, 1, 'structural rebuild'));
          chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 10px; }', 'structural rebuild cssText'));
          // 换元候选文本改写（20px）→ 双臂并真，读面随叠加真值。
          ne.textContent = '@function --f() { result: 20px; }';
          chk(eq(__zw_style_rules_stale('#s1'), '1', 'structural + text stale'));
          chk(eq(ss.cssRules[0].cssText, '@function --f() { result: 20px; }', 'structural + text rebuild cssText'));
          return fails.length ? fails.join('|') : 'PASS';
        })()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("k147 stale structural execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k147 结构臂失效重建钉测失败：{}", verdict);
}
