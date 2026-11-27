// k149 css-mixins @mixin body CSSOM decl 写父链重算钉测（2026-11-28）
// corpus 形蒸馏：WPT css-mixins/mixins/mixin-invalidation.tentative.html t1/t2
//『invalidation of @mixin from same / different stylesheet』——嵌套规则 decl 写
//（style.color='green'）后 @result/@mixin 父层 cssText 无重算（物化静态文本）
// → 写面丢 green。face＝part06 父链 recompute 闭包（_zwMixinRuleLines 行集重组，
// host scan_block/render_mixin_css_text canonical 镜像）＋_makeRuleStyle
// `_zwDecls` 声明列表引用（合成 & child 逐 decl 行动态派生）。
// 钉面五向：
//   D1 t1 显式块臂——嵌套 & child 写后三层 cssText 逐字节（判据页期望形）；
//   D2 t2 合成 & 臂——@result 裸 decl 写（合成 child，_zwDecls 派生行）；
//   D3 物化↔重组恒等——写后写回原值 cssText 复原物化形（重组无漂移）；
//   D4 重组头 params 剥/保形——有参 `@mixin --m(--a) {` 头恒等＋全串往返；
//   D5 显式非 & selector 臂——div 嵌套块写后单行形。
// https://drafts.csswg.org/css-mixins-1/#cssom
const K149_MIXIN_INVALIDATION_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  function mkSheet(css) { var ss = new CSSStyleSheet(); ss.replaceSync(css); return ss; }

  stage('D1', function () {
    var ss = mkSheet('@mixin --m1() { @result { color: red; & { --foo: bar; } } }');
    var mx = ss.cssRules[0];
    var res = mx.cssRules[0];
    // 物化前态（C1 形恒承）。
    chk(eq(mx.cssText,
      '@mixin --m1 {\n  @result {\n  color: red;\n  & { --foo: bar; }\n}\n}',
      'D1 pre cssText'));
    // t1 写面：显式嵌套块 child。
    res.cssRules[1].style.color = 'green';
    chk(eq(res.cssRules[1].cssText, '& { --foo: bar; color: green; }',
      'D1 sr cssText'));
    chk(eq(res.cssText,
      '  @result {\n  color: red;\n  & { --foo: bar; color: green; }\n}',
      'D1 rr cssText'));
    chk(eq(mx.cssText,
      '@mixin --m1 {\n  @result {\n  color: red;\n  & { --foo: bar; color: green; }\n}\n}',
      'D1 mx cssText'));
  });

  stage('D2', function () {
    var ss = mkSheet('@mixin --m2() { @result { color: red; } }');
    var mx = ss.cssRules[0];
    var res = mx.cssRules[0];
    chk(eq(res.cssRules.length, 1, 'D2 agg child'));
    chk(eq(res.cssRules[0].style.color, 'red', 'D2 pre'));
    // t2 写面：合成 & child（bare decl 载体，行集走 style._zwDecls 派生）。
    res.cssRules[0].style.color = 'green';
    chk(eq(mx.cssText,
      '@mixin --m2 {\n  @result {\n  color: green;\n}\n}',
      'D2 mx cssText'));
  });

  stage('D3', function () {
    var ss = mkSheet('@mixin --m1() { @result { color: red; & { --foo: bar; } } }');
    var mx = ss.cssRules[0];
    var pre = mx.cssText;
    var res = mx.cssRules[0];
    // 写＋removeProperty：decls 复原后重组形必须复原物化形（行集配方与
    // canonical 无漂移；CSSOM 语义 setProp 覆写不删声明，removeProperty 才删）。
    res.cssRules[1].style.setProperty('color', 'green');
    chk(eq(mx.cssText.indexOf('color: green') >= 0 ? mx.cssText : '',
      '@mixin --m1 {\n  @result {\n  color: red;\n  & { --foo: bar; color: green; }\n}\n}',
      'D3 written form'));
    res.cssRules[1].style.removeProperty('color');
    chk(eq(mx.cssText, pre, 'D3 remove round-trip identity'));
    // 合成 child 覆写 round-trip（声明保留、值复原——removeProperty 空块是
    // 正确 CSSOM 语义，物化形不可复原）。
    var ss2 = mkSheet('@mixin --m2() { @result { color: red; } }');
    var pre2 = ss2.cssRules[0].cssText;
    var agg = ss2.cssRules[0].cssRules[0].cssRules[0];
    agg.style.setProperty('color', 'green');
    chk(eq(ss2.cssRules[0].cssText,
      '@mixin --m2 {\n  @result {\n  color: green;\n}\n}',
      'D3 synthetic written'));
    agg.style.setProperty('color', 'red');
    chk(eq(ss2.cssRules[0].cssText, pre2, 'D3 synthetic identity'));
  });

  stage('D4', function () {
    // 有参：type() 剥壳 canonical 保位，重组头与物化头恒等。
    var ss = mkSheet('@mixin --m3(--arg type(<length>): 1em) { @result { color: red; & { --x: 1px; } } }');
    var mx = ss.cssRules[0];
    chk(eq(mx.paramsText, '--arg <length>: 1em', 'D4 paramsText'));
    var pre = mx.cssText;
    chk(eq(pre.slice(0, pre.indexOf('{')),
      '@mixin --m3(--arg <length>: 1em) ', 'D4 head'));
    mx.cssRules[0].cssRules[1].style.setProperty('--x', '2px');
    chk(eq(mx.cssText.slice(0, mx.cssText.indexOf('{')),
      '@mixin --m3(--arg <length>: 1em) ', 'D4 head after write'));
    // 写面后再写回：全串复原。
    mx.cssRules[0].cssRules[1].style.setProperty('--x', '1px');
    chk(eq(mx.cssText, pre, 'D4 round-trip'));
  });

  stage('D5', function () {
    var ss = mkSheet('@mixin --m5 { @result { color: red; div { color: blue; } } }');
    var mx = ss.cssRules[0];
    chk(eq(mx.cssText,
      '@mixin --m5 {\n  @result {\n  color: red;\n  div { color: blue; }\n}\n}',
      'D5 pre cssText'));
    mx.cssRules[0].cssRules[1].style.color = 'green';
    chk(eq(mx.cssText,
      '@mixin --m5 {\n  @result {\n  color: red;\n  div { color: green; }\n}\n}',
      'D5 mx cssText'));
  });

  if (fails.length) return 'K149-PINS:' + fails.join(' | ');
  return 'PASS';
})()"##;

#[test]
fn test_k149_mixin_invalidation_parent_recompute_pins() {
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
        "<html><head><title>k149pins</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k149.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(K149_MIXIN_INVALIDATION_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("k149 mixin-invalidation execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k149 @mixin body decl 写父链重算钉测失败：{}", verdict);
}
