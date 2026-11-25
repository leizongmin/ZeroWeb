// k117 css-mixins @mixin CSSOM 结构化面钉测（2026-11-25）
// corpus 形蒸馏：WPT css-mixins/mixins/{mixin-cssom,mixin-parsing,
// mixin-invalidation} 全页形——此前 @mixin 随 @-rule defer 恒不入 cssRules
//（CSSMixinRule 接口缺位 cssom 8F＋parsing 17F＋layers 1F）。
// 钉面六向：
//   C1 cssText canonical——空参剥 ()/@result 平级行形/嵌套块单行/
//      `@contents;` 分号行/参数 type() 剥壳/无参数列表保形；
//   C2 接口/结构——CSSMixinRule/CSSResultRule instanceof/name/paramsText/
//      invalidation 前置树 cssRules[0].cssRules.length==2（裸声明聚合 +
//      嵌套块两 style child）/style 读面（--foo/color）；
//   C3 parsing test_child——@layer 顶层三形 invalid（mixin.cssRules 0）/
//      @result 内三形 valid（mixin.cssRules 1）/六种嵌套块 valid；
//   C4 @layer 丢弃深形——statement/block/nested style/body-level @media
//      四形 sibling 保留＋cssText 无 @layer（skip_extent extent 消耗）；
//   C5 写面门——@apply top-level insertRule SyntaxError＋@mixin 空体保留
//     （cssRules=[] 计数）＋非法 @mixin insertRule SyntaxError；
//   C6 replaceSync 深度感知切分——嵌套块段整段（@mixin 与尾随 #foo 规则
//      双段完整，旧 split('}') 碎段形不再）。
// https://drafts.csswg.org/css-mixins-1/#cssom
const K117_MIXIN_CSSOM_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  function mkSheet(css) { var ss = new CSSStyleSheet(); ss.replaceSync(css); return ss; }

  stage('C1', function () {
    // cssom 测 1：空参列表剥 ()＋@result 平级行＋嵌套块单行。
    var s1 = mkSheet('@mixin --m1() { @result { color: green; & { --foo: bar; } } }');
    chk(eq(s1.cssRules.length, 1, 'C1 rules'));
    chk(eq(s1.cssRules[0].cssText,
      '@mixin --m1 {\n  @result {\n  color: green;\n  & { --foo: bar; }\n}\n}',
      'C1 m1 cssText'));
    // cssom 测 3：@contents 分号行。
    var s3 = mkSheet('@mixin --m2() { @result { @contents } }');
    chk(eq(s3.cssRules[0].cssText,
      '@mixin --m2 {\n  @result {\n  @contents;\n}\n}',
      'C1 contents cssText'));
    // cssom 测 6：参数 type() 剥壳＋default 保形＋无 default 保位。
    var s6 = mkSheet('@mixin --m3(--arg type(<length>): 1em, --other-arg) { @result { margin-left: var(--arg); } }');
    chk(eq(s6.cssRules[0].cssText,
      '@mixin --m3(--arg <length>: 1em, --other-arg) {\n  @result {\n  margin-left: var(--arg);\n}\n}',
      'C1 params cssText'));
    // cssom 测 7：无参数列表保形（不加 ()）。
    var s7 = mkSheet('@mixin --m4 { @result { color: green; } }');
    chk(eq(s7.cssRules[0].cssText,
      '@mixin --m4 {\n  @result {\n  color: green;\n}\n}',
      'C1 no-params cssText'));
  });

  stage('C2', function () {
    // invalidation 测 1 前置形（red 源）：接口/name/树计数/style 读面。
    var ss = mkSheet('@mixin --m1() { @result { color: red; & { --foo: bar; } } }');
    var mx = ss.cssRules[0];
    chk(isTrue(mx instanceof CSSMixinRule, 'C2 mixin instanceof'));
    chk(eq(mx.name, '--m1', 'C2 name'));
    chk(eq(mx.paramsText, '', 'C2 paramsText 空'));
    chk(eq(mx.cssRules.length, 1, 'C2 mixin.cssRules 1'));
    chk(isTrue(mx.cssRules[0] instanceof CSSResultRule, 'C2 result instanceof'));
    var res = mx.cssRules[0];
    chk(eq(res.cssRules.length, 2, 'C2 result.cssRules 2'));
    chk(eq(res.cssRules[0].selectorText, '&', 'C2 agg selector'));
    chk(eq(res.cssRules[0].style.color, 'red', 'C2 agg style color'));
    chk(eq(res.cssRules[1].selectorText, '&', 'C2 nested selector'));
    chk(eq(res.cssRules[1].style.getPropertyValue('--foo'), 'bar', 'C2 nested style --foo'));
    // 参数 paramsText canonical。
    var sp = mkSheet('@mixin --m3(--arg type(<length>): 1em, --other-arg) { @result { } }');
    chk(eq(sp.cssRules[0].paramsText, '--arg <length>: 1em, --other-arg', 'C2 paramsText canonical'));
  });

  stage('C3', function () {
    function child(css, expect, m) {
      var sheet = mkSheet(css);
      chk(eq(sheet.cssRules.length, 1, 'C3 present: ' + m));
      if (sheet.cssRules.length === 1) {
        chk(eq(sheet.cssRules[0].cssRules.length, expect, 'C3 child count: ' + m));
      }
    }
    // invalid：@layer 顶层三形 → child count 0（extent 丢弃后无 child）。
    child('@mixin --m() { @layer bar; }', 0, 'layer stmt');
    child('@mixin --m() { @layer bar {} }', 0, 'layer block');
    child('@mixin --m() { @layer {} }', 0, 'layer anon');
    // valid：@layer 在 @result 内（@result 保留）。
    child('@mixin --m() { @result { @layer bar; } }', 1, 'layer stmt in result');
    child('@mixin --m() { @result { @layer bar {} } }', 1, 'layer block in result');
    child('@mixin --m() { @result { @layer {} } }', 1, 'layer anon in result');
    // valid：六种嵌套块。
    child('@mixin --m() { @result { div {} } }', 1, 'style rule');
    child('@mixin --m() { @result { > div {} } }', 1, 'style rule (relative)');
    child('@mixin --m() { @result { @media (width) {} } }', 1, '@media');
    child('@mixin --m() { @result { @supports (width:0) {} } }', 1, '@supports');
    child('@mixin --m() { @result { @container (width) {} } }', 1, '@container');
    child('@mixin --m() { @result { @starting-style {} } }', 1, '@starting-style');
    child('@mixin --m() { @result { @scope (.foo) {} } }', 1, '@scope');
    child('@mixin --m() { @result { @scope {} } }', 1, '@scope (implicit)');
  });

  stage('C4', function () {
    function dropped(css, m) {
      var sheet = mkSheet(css);
      chk(eq(sheet.cssRules.length, 1, 'C4 present: ' + m));
      if (sheet.cssRules.length === 1) {
        var t = sheet.cssRules[0].cssText;
        chk(isTrue(t.indexOf('color: green') >= 0, 'C4 sibling retained: ' + m + ' :: ' + t));
        chk(isTrue(!/@layer/i.test(t), 'C4 layer dropped: ' + m + ' :: ' + t));
      }
    }
    dropped('@mixin --m() { @result { @layer bar; div { color: green } } }', 'result stmt');
    dropped('@mixin --m() { @result { @layer bar {} div { color: green } } }', 'result block');
    dropped('@mixin --m() { @result { div { @layer bar; color: green } } }', 'nested style');
    dropped('@mixin --m() { @media (width) { @layer bar; @result { div { color: green } } } }', 'body @media');
  });

  stage('C5', function () {
    // @apply top-level 门：insertRule 恒 SyntaxError。
    var s5 = new CSSStyleSheet();
    var threw = false;
    try { s5.insertRule('@apply --m1();'); } catch (e) { threw = e && e.name === 'SyntaxError'; }
    chk(isTrue(threw, 'C5 apply top-level SyntaxError'));
    // @mixin 空体保留（k117 门——cssRules=[] 计数）。
    var se = mkSheet('@mixin --m { }');
    chk(eq(se.cssRules.length, 1, 'C5 empty mixin retained'));
    chk(eq(se.cssRules[0].cssRules.length, 0, 'C5 empty mixin cssRules'));
    // 非法 @mixin：缺名 → insertRule SyntaxError。
    var sb = new CSSStyleSheet();
    var threw2 = false;
    try { sb.insertRule('@mixin { color: green; }'); } catch (e) { threw2 = e && e.name === 'SyntaxError'; }
    chk(isTrue(threw2, 'C5 invalid mixin SyntaxError'));
  });

  stage('C6', function () {
    // 深度感知切分：@mixin 嵌套段与尾随规则双段完整（旧 split('}') 碎段形
    // 由此消失——@mixin 段不被尾部 } 提前切断）。
    var sm = mkSheet('@mixin --m1() { @result { color: green; } } #foo { color: red; }');
    chk(eq(sm.cssRules.length, 2, 'C6 two rules'));
    chk(isTrue(sm.cssRules[0] instanceof CSSMixinRule, 'C6 [0] mixin'));
    chk(eq(sm.cssRules[1].selectorText, '#foo', 'C6 [1] selector'));
    chk(eq(sm.cssRules[1].style.color, 'red', 'C6 [1] style'));
  });

  return fails.length ? fails.join('|') : 'PASS';
})()"##;

#[test]
fn test_k117_mixin_cssom_pins() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k117pins</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k117.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(K117_MIXIN_CSSOM_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("k117 mixin-cssom execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k117 @mixin CSSOM 结构化面钉测失败：{}", verdict);
}

#[test]
fn test_k117_mixin_wire_pins() {
    // wire 单源通道：style_rules_text @mixin 臂（css_wire k117_wire_tests 同域
    // shim 侧复核——wire 条目 `@mixin\x1e{canonical}` 经 shim 分派物化
    // CSSMixinRule，cssText 树形与 host 回调单源恒同）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><style id=s1>@mixin --m1() { @result { color: green; & { --foo: bar; } } } #foo { @apply --m1; }</style><style id=s2>@mixin --m3(--arg type(<length>): 1em, --other-arg) { @result { margin-left: var(--arg); } }</style></head><body><p id=out></p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k117-wire.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(
        r#"(function () {
          var fails = [];
          var ss = document.getElementById('s1').sheet;
          if (!ss) return 'FAIL:no-sheet';
          if (ss.cssRules.length !== 2) return 'FAIL:rules=' + ss.cssRules.length;
          if (!(ss.cssRules[0] instanceof CSSMixinRule)) return 'FAIL:[0]-brand';
          if (ss.cssRules[0].name !== '--m1') return 'FAIL:name';
          if (ss.cssRules[0].cssText !== '@mixin --m1 {\n  @result {\n  color: green;\n  & { --foo: bar; }\n}\n}') return 'FAIL:m1-cssText=' + JSON.stringify(ss.cssRules[0].cssText);
          if (ss.cssRules[1].cssText !== '#foo {\n  @apply --m1;\n}') return 'FAIL:apply-cssText=' + JSON.stringify(ss.cssRules[1].cssText);
          var ss2 = document.getElementById('s2').sheet;
          if (!ss2) return 'FAIL:no-sheet2';
          if (ss2.cssRules.length !== 1) return 'FAIL:rules2=' + ss2.cssRules.length;
          if (ss2.cssRules[0].paramsText !== '--arg <length>: 1em, --other-arg') return 'FAIL:params2=' + JSON.stringify(ss2.cssRules[0].paramsText);
          if (ss2.cssRules[0].cssText !== '@mixin --m3(--arg <length>: 1em, --other-arg) {\n  @result {\n  margin-left: var(--arg);\n}\n}') return 'FAIL:m3-cssText=' + JSON.stringify(ss2.cssRules[0].cssText);
          return fails.length ? fails.join('|') : 'PASS';
        })()"#,
    ) {
        Ok(v) => v.value,
        Err(e) => panic!("k117 wire execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k117 wire 通道钉测失败：{}", verdict);
}
