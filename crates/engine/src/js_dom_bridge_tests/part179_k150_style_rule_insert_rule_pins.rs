// k150 css-mixins CSSStyleRule.insertRule 裸声明追加钉测（2026-11-28）
// corpus 形蒸馏：WPT css/css-mixins/mixins/mixin-invalidation.tentative.html t3
//『invalidation on adding @apply rule』——style rule 对象此前无 insertRule 方法
//（`is not a function` TypeError）；`@apply --m3;` 形 at-declaration 在 JS 写面
// parse 恒丢弃。face＝_parseDeclarations at-decl 臂＋_zwSerializeDecls at 臂＋
// _zwStyleRuleInsertRule 裸声明追加工厂（hasApply 分流重建＝引擎 wire canonical
// css_wire style_rules_text has_apply 多行/单行同配方，flush 后 stale 重物化恒等）。
// 钉面六向：
//   D1 t3 主臂——insertRule('@apply --m3;') cssText 逐字节判据形＋返回值
//   （尾插 index＝插入前 decl 数）；
//   D2 普通 decl 插入——无 @apply 条目单行形分流＋返回值；
//   D3 错误臂——空文本/块形（嵌套规则容器未建模）SyntaxError；
//   D4 index 头插臂——index 0 插入返回 0＋首行位；
//   D5 at-decl parse 臂——_ruleFromText 构造表 @apply decl 读回（style 面）；
//   D6 恒承臂——插入后 style 写 rebuild at 条目保形不丢。
// https://drafts.csswg.org/css-mixins-1/#cssom
const K150_STYLE_RULE_INSERT_RULE_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  function mkSheet(css) { var ss = new CSSStyleSheet(); ss.replaceSync(css); return ss; }

  stage('D1', function () {
    var ss = mkSheet('@mixin --m3() { @result { color: green; } } #target3 { color: red; }');
    var sr = ss.cssRules[1];
    var ret = sr.insertRule('@apply --m3;');
    chk(eq(ret, 1, 'D1 return index'));
    chk(eq(sr.cssText,
      '#target3 {\n  color: red;\n  @apply --m3;\n}',
      'D1 cssText'));
  });

  stage('D2', function () {
    var ss = mkSheet('#target2 { color: red; }');
    var sr = ss.cssRules[0];
    var ret = sr.insertRule('width: 10px');
    chk(eq(ret, 1, 'D2 return index'));
    chk(eq(sr.cssText, '#target2 { color: red; width: 10px; }', 'D2 cssText single-line'));
  });

  stage('D3', function () {
    var ss = mkSheet('#t3 { color: red; }');
    var sr = ss.cssRules[0];
    function expectErr(code) {
      try { sr.insertRule(code); return 'FAIL:D3-no-throw(' + JSON.stringify(code) + ')'; }
      catch (e) { return (String(e && e.name) === 'SyntaxError') ? null : 'FAIL:D3-not-SyntaxError:' + (e && e.name); }
    }
    chk(eq(expectErr(''), null, 'D3 empty'));
    chk(eq(expectErr('& div { color: red; }'), null, 'D3 block form'));
  });

  stage('D4', function () {
    var ss = mkSheet('@mixin --m4() { @result { color: green; } } #t4 { color: red; }');
    var sr = ss.cssRules[1];
    sr.insertRule('@apply --m4;');
    var ret = sr.insertRule('color: blue', 0);
    chk(eq(ret, 0, 'D4 return index'));
    chk(eq(sr.cssText,
      '#t4 {\n  color: blue;\n  color: red;\n  @apply --m4;\n}',
      'D4 cssText head-insert'));
  });

  stage('D5', function () {
    var ss = mkSheet('.a { @apply --m1; color: blue; }');
    var sr = ss.cssRules[0];
    chk(eq(sr.style.getPropertyValue('@apply'), '--m1', 'D5 getPropertyValue'));
    chk(eq(sr.style.length, 2, 'D5 length'));
    chk(eq(sr.cssText, '.a { @apply --m1; color: blue; }', 'D5 cssText'));
  });

  stage('D6', function () {
    var ss = mkSheet('@mixin --m6() { @result { color: green; } } #t6 { color: red; }');
    var sr = ss.cssRules[1];
    sr.insertRule('@apply --m6;');
    // 恒承：插入后 style 写 rebuild——at 条目经 _zwSerializeDecls at 臂串形
    // 不丢（形变单行属 FIXME 记档面，本臂只钉 @apply 保形）。
    sr.style.setProperty('margin', '1px');
    chk(eq(sr.cssText.indexOf('@apply --m6;') >= 0 ? 'OK' : sr.cssText, 'OK',
      'D6 at-decl survives rebuild'));
  });

  if (fails.length) return 'K150-PINS:' + fails.join(' | ');
  return 'PASS';
})()"##;

#[test]
fn test_k150_style_rule_insert_rule_pins() {
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
        "<html><head><title>k150pins</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k150.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(K150_STYLE_RULE_INSERT_RULE_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("k150 style-rule insertRule execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k150 CSSStyleRule.insertRule 裸声明追加钉测失败：{}", verdict);
}
