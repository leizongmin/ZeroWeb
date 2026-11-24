// s10k16 激活链 getElementsByClassName 钉测（刀⑯，2026-11-25）
// corpus 形蒸馏：WPT dom/events/Event-dispatch-single-activation-behavior.html
// 132F wholesale 根因（`e.getElementsByClassName is not a function`——R151 fragment
// face 只挂 content fragment，_zwMEl 元素本体缺方法）的常驻回归钉：解析快照
// template.content 克隆元素 GECN 面／token 全含匹配／不含 self／空 token 不抛／
// DOMString null 转换／corpus 同构 click 激活链（form submit／checkbox oninput／
// details toggle）。R516 face＝part03 _zwMEl 装配块。
const S10K16_ACTIVATION_GECN_PINS_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + a + '!=' + b + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  var kids, cbClone, fClone;

  // ① 解析快照 template.content 克隆元素 GECN（根因面）＋子树查询
  stage('1', function () {
  var tpl = document.querySelector('template#k16tpl');
  kids = tpl.content.children;
  chk(eq(kids.length, 4, '① template kids'));
  cbClone = kids[0].cloneNode(true);
  fClone = kids[1].cloneNode(true);
  chk(eq(typeof cbClone.getElementsByClassName, 'function', '① clone has GECN'));
  var wrap1 = document.createElement('div');
  document.body.appendChild(wrap1);
  wrap1.appendChild(fClone);
  var actsInWrap = wrap1.getElementsByClassName('activates');
  chk(eq(actsInWrap.length, 1, '① wrapper subtree query'));
  chk(eq(actsInWrap[0] && actsInWrap[0].tagName, 'FORM', '① hit tag'));
  });
  // ② token 全含匹配（spec dom-element-getelementsbyclassname 步骤——多 token AND）
  stage('2', function () {
  chk(eq(fClone.getElementsByClassName('activates').length, 0, '② subtree excludes self'));
  chk(eq(fClone.getElementsByClassName('activates click').length, 0, '② all-tokens miss'));
  chk(eq(fClone.getElementsByClassName('click container').length, 1, '② all-tokens hit'));
  });
  // ③ 不含 self（含 class 的元素查自身子树 → 空）
  stage('3', function () {
  cbClone.className = 'activates';
  chk(eq(cbClone.getElementsByClassName('activates').length, 0, '③ self excluded'));
  });
  // ④ 空 token 集 → 空数组不抛（part06 R185 同口径）
  stage('4', function () {
  chk(eq(fClone.getElementsByClassName('').length, 0, '④ empty string'));
  chk(eq(fClone.getElementsByClassName('   ').length, 0, '④ blank string'));
  });
  // ⑤ DOMString 转换先行（null→"null" token 双命中面）
  stage('5', function () {
  fClone.getElementsByClassName('click')[0].setAttribute('class', 'null');
  chk(eq(fClone.getElementsByClassName(null).length, 1, '⑤ null→null token'));
  fClone.getElementsByClassName('null')[0].setAttribute('class', 'click container');

  });
  // ⑥ corpus 同构 click 激活链——两个 fresh clone 先后挂同一 live target
  // （corpus pair 循环形），form pair → submit input click → onsubmit；
  // 已挂 MEl clone 重挂 live 父（re-parent）为独立 appendChild 面，挂账不钉。
  stage('6', function () {
  globalThis.__k16acts = [];
  var tgt6 = document.createElement('div');
  document.body.appendChild(tgt6);
  var f6 = kids[1].cloneNode(true);
  var c6 = kids[0].cloneNode(true);
  tgt6.appendChild(f6);
  tgt6.appendChild(c6);
  f6.getElementsByClassName('click')[0].click();
  chk(eq(globalThis.__k16acts.join(','), 'form', '⑥ submit chain'));
  globalThis.__k16acts = [];
  c6.click();
  chk(eq(globalThis.__k16acts.join(','), 'cb', '⑥ checkbox chain'));
  });
  // ⑥b corpus 组合：parent=form clone, target=cb clone 入 parent container——
  stage('6b', function () {
  //   只激活 target（form 不提交，spec single activation）
  globalThis.__k16acts = [];
  var p6 = kids[1].cloneNode(true);
  var t6 = kids[0].cloneNode(true);
  var w6 = document.createElement('div');
  document.body.appendChild(w6);
  w6.appendChild(p6);
  p6.getElementsByClassName('container')[0].appendChild(t6);
  t6.click();
  chk(eq(globalThis.__k16acts.join(','), 'cb', '⑥b single activation'));
  });
  // ⑥c checkbox clone 直挂 → click → oninput（checked 读真）
  stage('6c', function () {
  globalThis.__k16acts = [];
  var c6 = kids[0].cloneNode(true);
  var w6c = document.createElement('div');
  document.body.appendChild(w6c);
  w6c.appendChild(c6);
  c6.click();
  chk(eq(globalThis.__k16acts.join(','), 'cb', '⑥c checkbox chain'));

  });
  // ⑦ details clone → summary click → open 翻转 + ontoggle
  stage('7', function () {
  globalThis.__k16acts = [];
  var d7 = kids[2].cloneNode(true);
  var w7 = document.createElement('div');
  document.body.appendChild(w7);
  w7.appendChild(d7);
  d7.getElementsByClassName('click')[0].click();
  chk(eq(String(d7.open), 'true', '⑦ details open'));
  chk(eq(globalThis.__k16acts.join(','), 'det', '⑦ toggle handler'));

  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k16_activation_gecn_pins() {
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
        "<html><head><title>k16pins</title></head><body><div id=\"test_container\"></div><template id=\"k16tpl\"><input class=\"click activates container\" type=\"checkbox\" oninput=\"globalThis.__k16acts.push(this.checked ? 'cb' : 'cbn')\"><form class=\"activates\" onsubmit=\"globalThis.__k16acts.push('form'); return false\"><input class=\"click container\" type=\"submit\"></form><details class=\"activates\" ontoggle=\"globalThis.__k16acts.push('det')\"><summary class=\"click container\"></summary></details><a href=\"#link\" class=\"click container activates\"></a></template></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/dom/events/pins-k16.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K16_ACTIVATION_GECN_PINS_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k16 execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k16 激活链 GECN 钉测失败：{}", verdict);
}
