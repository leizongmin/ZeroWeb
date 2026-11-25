// k118 css-mixins `<dashed-function>` 调用实参文法写面钉测（2026-11-25）
// corpus 形蒸馏：WPT css/css-mixins/functions/{dashed-function-parsing(49 形),
// dashed-function-named-arg.tentative(15 形)} 失败簇 34 形（此前调用实参
// 文法零验证——坏词元/空实参/块非独占/空块体/保留形全收）。
// 钉面五向：
//   C1 元素 style setProp 矩阵——合法形收（回读原值：基础/嵌套调用/逗号块/
//      非首保留形对照）＋坏词元/空实参/块非独占/空块体/跨空白保留形拒；
//   C2 cssText 写面——坏形声明丢弃＋同串后续合法声明保留（bad declaration
//      同语义）＋custom property 豁免（--x: --func(!) 保留）；
//   C3 样式表通道——replaceSync 规则 style 读面（css-parser consume_
//      declaration 门单源→wire 缺声明）＋合法形存储；
//   C4 零漂移——var(--func) 引用/无调用值/无参空调用照常（引用不虚警）；
//   C5 !important 尾——坏形带尾拒、合法形带尾收（hub 剥尾再验语义）。
// Rust 侧文法权威钉面：css-parser tests/dashed_fn_validate.rs（全 64 形矩阵）
// ＋style-system t54 模块 k118 单测。验证单源 dashed_function_value_valid，
// 三通道共享（element hub / stylesheet gate / host validate pre-arm）。
// https://drafts.csswg.org/css-mixins-1/#function-rule
// https://github.com/w3c/csswg-drafts/issues/11749
const K118_DASHED_FN_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  function fresh() { var el = document.createElement('div'); document.body.appendChild(el); return el; }

  stage('C1', function () {
    // 合法形收（回读原值）。
    var e1 = fresh();
    e1.style.top = '--func(auto, 100px)';
    chk(eq(e1.style.top, '--func(auto, 100px)', 'C1 valid store'));
    var e2 = fresh();
    e2.style.top = '--func(--bar(), --baz(--fez()))';
    chk(eq(e2.style.top, '--func(--bar(), --baz(--fez()))', 'C1 nested store'));
    var e3 = fresh();
    e3.style.top = '--func({1},{2})';
    chk(eq(e3.style.top, '--func({1},{2})', 'C1 comma-block store'));
    var e4 = fresh();
    e4.style.top = '--func(50px --myident:)';
    chk(eq(e4.style.top, '--func(50px --myident:)', 'C1 reserved-nonfirst store'));
    // 坏词元拒（! ; } ]）。
    var bads = ['--func(!)', '--func(;)', '--func(})', '--func(])'];
    for (var i = 0; i < bads.length; i++) {
      var eb = fresh();
      eb.style.top = bads[i];
      chk(eq(eb.style.top, '', 'C1 reject bad-token ' + bads[i]));
    }
    // 空实参拒。
    var e5 = fresh();
    e5.style.top = '--func(asdf,)';
    chk(eq(e5.style.top, '', 'C1 reject trailing-comma'));
    var e6 = fresh();
    e6.style.top = '--func(a,,b)';
    chk(eq(e6.style.top, '', 'C1 reject empty-mid'));
    var e7 = fresh();
    e7.style.top = '--func(,)';
    chk(eq(e7.style.top, '', 'C1 reject leading-comma'));
    // 块非独占 / 空块体拒。
    var e8 = fresh();
    e8.style.top = '--func(123 {})';
    chk(eq(e8.style.top, '', 'C1 reject block-not-sole'));
    var e9 = fresh();
    e9.style.top = '--func({})';
    chk(eq(e9.style.top, '', 'C1 reject empty-block'));
    var e10 = fresh();
    e10.style.top = '--func(1, { })';
    chk(eq(e10.style.top, '', 'C1 reject empty-block-2nd'));
    // 保留形拒（跨空白）。
    var e11 = fresh();
    e11.style.top = '--func( --myident : )';
    chk(eq(e11.style.top, '', 'C1 reject reserved'));
    var e12 = fresh();
    e12.style.top = '--func(10px, --myident : )';
    chk(eq(e12.style.top, '', 'C1 reject reserved-2nd-arg'));
    // 嵌套坏实参递归拒。
    var e13 = fresh();
    e13.style.top = '--func(--bar(!))';
    chk(eq(e13.style.top, '', 'C1 reject nested-bad'));
  });

  stage('C2', function () {
    var el = fresh();
    el.style.cssText = 'top: --func(!); color: red;';
    chk(eq(el.style.getPropertyValue('top'), '', 'C2 top dropped'));
    chk(eq(el.style.getPropertyValue('color'), 'red', 'C2 color kept'));
    var el2 = fresh();
    el2.style.cssText = '--x: --func(!); top: 1px;';
    chk(eq(el2.style.getPropertyValue('--x'), '--func(!)', 'C2 custom prop exempt'));
    chk(eq(el2.style.getPropertyValue('top'), '1px', 'C2 top kept'));
  });

  stage('C3', function () {
    var ss = new CSSStyleSheet();
    ss.replaceSync('div { top: --func(!); color: red; } #ok { top: --func(auto); }');
    chk(eq(ss.cssRules.length, 2, 'C3 rules'));
    chk(eq(ss.cssRules[0].style.getPropertyValue('top'), '', 'C3 gate top dropped'));
    chk(eq(ss.cssRules[0].style.getPropertyValue('color'), 'red', 'C3 color kept'));
    chk(eq(ss.cssRules[1].style.getPropertyValue('top'), '--func(auto)', 'C3 valid stored'));
  });

  stage('C4', function () {
    var el = fresh();
    el.style.top = 'var(--func)';
    chk(eq(el.style.top, 'var(--func)', 'C4 var-ref untouched'));
    var el2 = fresh();
    el2.style.top = '--func()';
    chk(eq(el2.style.top, '--func()', 'C4 zero-arg store'));
    var el3 = fresh();
    el3.style.top = 'auto';
    chk(eq(el3.style.top, 'auto', 'C4 plain untouched'));
    var el4 = fresh();
    el4.style.top = '--func(--)';
    chk(eq(el4.style.top, '--func(--)', 'C4 bare-dashed-ident store'));
  });

  stage('C5', function () {
    var el = fresh();
    el.style.top = '--func(!) !important';
    chk(eq(el.style.top, '', 'C5 important-bad rejected'));
    var el2 = fresh();
    el2.style.top = '--func(auto) !important';
    // !important 存 priority 位，属性 getter 回读不回显尾巴（T53
    // _styleValueAndPriority 拆分语义）——回读非空即收。
    chk(eq(el2.style.top, '--func(auto)', 'C5 important-good store'));
  });

  return fails.length ? fails.join('|') : 'PASS';
})()"##;

#[test]
fn test_k118_dashed_fn_write_face_pins() {
    // 写面端到端：全 shim（generate_js_dom_shim）＋宿主
    // __zw_validate_decl_value 回调（register_dom_callbacks 注册→
    // zero_style_system::validate_decl_value DFN-VALID pre-arm 单源）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k118pins</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/css-mixins/pins-k118.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(K118_DASHED_FN_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("k118 dashed-fn execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "k118 <dashed-function> 写面钉测失败：{}", verdict);
}
