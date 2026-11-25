// s10k17 激活链 MEl 区域查询并查钉测（刀⑰，2026-11-25）
// corpus 形蒸馏：WPT dom/events/Event-dispatch-single-activation-behavior.html
// 残 121F 根因（R517——`_zwMEl` 产物内部 mutation 纯 JS 私有树无 host wire，
// createElement 作用域 GECN/qsa 的 host 查询与 pending 桶均不可见 MEl 子树内
// 元素 → stamp/href 相位漏嵌套 target → classList 缺 test{i} → filter 全弃
// got=[]；R518——A/AREA href IDL getter 相对串原样回落，对 hashchange
// e.newURL 绝对形失配）的常驻回归钉：stamp 相位 parent+target 双命中与本体
// identity／filter 相位 produced==expected 同本体／qsa href 相位嵌套 aMEl 命中
// ＋getter 绝对＋attr 原串／MEl click 同步片段导航／无 MEl handle 域零扰动／
// 深链 container 双命中＋无双计／href 基选择与 R130 空串口径／⑧ R517b（k17b）
// TEMPLATE 子元素不下钻——嵌套 template 只在装配后可见（createTestTree 同构
// 装配链端到端；v1 电池 36F 回归钉）／⑨ R519（刀⑱）MEl dispatchEvent inline
// on* **属性** handler 消费——submit/reset/toggle 属性形激活＋target-only
//（祖先站不重入）＋click/input/change 三特判型单执行（R152/R155 拥有面）。
// R517 face＝part04 handle 域 GECN/qsa 静态结果并查；R518 face＝part03 _zwMEl
// 装配块 R130 升级。
const S10K17_MEL_REGION_QUERY_PINS_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + a + '!=' + b + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  var kids, parent, target, aT;
  function filterTest17(acts) {
    var f = [];
    for (var i = 0; i < acts.length; i++) {
      var x = acts[i];
      if (x && x.classList && x.classList.contains('test17')) f.push(x);
    }
    return f;
  }

  // ① R517 stamp 相位——createElement 作用域 GECN 并入 MEl 区域匹配集
  stage('1', function () {
  var tpl = document.querySelector('template#k17tpl');
  kids = tpl.content.children;
  chk(eq(kids.length, 4, '① template kids'));
  var test = document.createElement('div');
  document.getElementById('test_container').appendChild(test);
  parent = kids[1].cloneNode(true);
  test.appendChild(parent);
  var slot = parent.getElementsByClassName('container')[0];
  chk(isTrue(slot && slot.__zwIsMEl === true, '① slot MEl'));
  target = kids[0].cloneNode(true);
  slot.appendChild(target);
  var stamps = test.getElementsByClassName('activates');
  chk(eq(stamps.length, 2, '① stamps len（parent host 面 + target MEl 区域并查）'));
  chk(isTrue(stamps[0] === parent, '① stamps[0] identity'));
  chk(isTrue(stamps[1] === target, '① stamps[1] identity（嵌套 MEl 子树）'));
  for (var i = 0; i < stamps.length; i++) stamps[i].classList.add('test17');
  chk(isTrue(target.classList.contains('test17'), '① target stamped'));
  });
  // ② corpus filter 相位——click → activations → classList filter 命中 target 本体
  stage('2', function () {
  globalThis.__k17acts = [];
  target.click();
  chk(eq(globalThis.__k17acts.length, 1, '② raw acts'));
  var f = filterTest17(globalThis.__k17acts);
  chk(eq(f.length, 1, '② filtered len'));
  chk(isTrue(f[0] === target, '② filtered identity（produced==expected 同本体）'));
  });
  // ③ R517 qsa href 相位——嵌套 aMEl 被并查；R518 getter 绝对 URL、attr 原串
  stage('3', function () {
  var test2 = document.createElement('div');
  document.getElementById('test_container').appendChild(test2);
  var parent2 = kids[2].cloneNode(true);
  test2.appendChild(parent2);
  var slot2 = parent2.getElementsByClassName('container')[0];
  aT = kids[3].cloneNode(true);
  slot2.appendChild(aT);
  var qa = test2.querySelectorAll('a, area');
  var hit = -1;
  for (var q = 0; q < qa.length; q++) if (qa[q] === aT) { hit = q; break; }
  chk(isTrue(hit >= 0, '③ qsa 命中嵌套 aMEl'));
  for (var h = 0; h < qa.length; h++) qa[h].href = '#test17_link';
  var abs = String(aT.href);
  chk(isTrue(abs.indexOf('/pins-k17.html#test17_link') >= 0, '③ getter 绝对'));
  chk(eq(abs.slice(-12), '#test17_link', '③ 尾形'));
  chk(eq(aT.getAttribute('href'), '#test17_link', '③ attr 原串'));
  });
  // ④ R518 伴生面——MEl a click → 同步片段导航（hashchange 事件 deferred，
  // runner 侧既有 hashchange 语料已证，钉同步位置面）
  stage('4', function () {
  var hrefBefore = String(globalThis.location.href).split('#')[0];
  aT.click();
  chk(eq(String(globalThis.location.href), hrefBefore + '#test17_link', '④ 同步导航'));
  });
  // ⑤ 面回归守卫——无 MEl 的 handle 域 GECN/qsa 零扰动（并查 no-op）
  stage('5', function () {
  var t5 = document.createElement('div');
  document.getElementById('test_container').appendChild(t5);
  var d5 = document.createElement('div');
  d5.className = 'zzq';
  t5.appendChild(d5);
  var g5 = t5.getElementsByClassName('zzq');
  chk(eq(g5.length, 1, '⑤ GECN len'));
  chk(isTrue(g5[0] === d5, '⑤ GECN identity'));
  var q5 = t5.querySelectorAll('div.zzq');
  chk(eq(q5.length, 1, '⑤ qsa len'));
  chk(isTrue(q5[0] === d5, '⑤ qsa identity'));
  });
  // ⑥ 深链 + 去重——container 深查 [slot,target] 双命中；activates 无双计
  stage('6', function () {
  var test6 = document.createElement('div');
  document.getElementById('test_container').appendChild(test6);
  var p6 = kids[1].cloneNode(true);
  test6.appendChild(p6);
  var s6 = p6.getElementsByClassName('container')[0];
  var t6 = kids[0].cloneNode(true);
  s6.appendChild(t6);
  var gc = test6.getElementsByClassName('container');
  chk(eq(gc.length, 2, '⑥ container len（slot+target 嵌套链）'));
  chk(isTrue(gc[0] === s6 && gc[1] === t6, '⑥ container identity'));
  var ga = test6.getElementsByClassName('activates');
  var pCnt = 0, tCnt = 0;
  for (var a6 = 0; a6 < ga.length; a6++) { if (ga[a6] === p6) pCnt++; if (ga[a6] === t6) tCnt++; }
  chk(eq(pCnt, 1, '⑥ parent 无双计'));
  chk(eq(tCnt, 1, '⑥ target 单计'));
  });
  // ⑦ R518 基选择口径——getter base = 页面 URL（__zw_parse_url 主链路）＋
  // R130 既有契约（特殊 query percent-encode／空 attr → ''）
  stage('7', function () {
  var a7 = kids[3].cloneNode(true);
  var w7 = document.createElement('div');
  document.getElementById('test_container').appendChild(w7);
  w7.appendChild(a7);
  a7.setAttribute('href', '/rel');
  chk(eq(String(a7.href), 'https://wpt.test/rel', '⑦ 根相对解析'));
  a7.setAttribute('href', 'http://example.org/?ä');
  chk(eq(String(a7.href), 'http://example.org/?%C3%A4', '⑦ R130 特殊 query 编码'));
  a7.setAttribute('href', '');
  chk(eq(String(a7.href), '', '⑦ 空 attr → 空（R130 旧口径保持）'));
  });
  // ⑧ R517b（k17b）——TEMPLATE 子元素不下钻：嵌套 template 只在装配后可见
  //（WPT shadow-dom createTestTree 同构；v1 电池 36F 回归钉：fusion walk 经
  // MEl template childNodes 下钻 content → 嵌套 template 误入顶层查询 →
  // attachShadow 二次装配 → ids 表污染 → slots/composed-path 断言全断）
  stage('8', function () {
  var clone8 = document.getElementById('test_nested17').cloneNode(true);
  var q8 = clone8.querySelectorAll('template');
  chk(eq(q8.length, 1, '⑧ 顶层 template 数（嵌套不外漏）'));
  chk(isTrue(q8[0] && q8[0].id === 'tplA17', '⑧ 顶层命中本体'));
  function attach8(t8) {
    var p8 = t8.parentNode;
    p8.removeChild(t8);
    var sr8 = p8.attachShadow({ mode: t8.getAttribute('data-mode') });
    if (t8.id) sr8.id = t8.id;
    sr8.appendChild(document.importNode(t8.content, true));
    return sr8;
  }
  var ids8 = {};
  (function walk8(root8) {
    if (root8.id) ids8[root8.id] = root8;
    var qi8 = root8.querySelectorAll('[id]');
    for (var i8 = 0; i8 < qi8.length; i8++) ids8[qi8[i8].id] = qi8[i8];
    var qt8 = root8.querySelectorAll('template');
    for (var t8 = 0; t8 < qt8.length; t8++) walk8(attach8(qt8[t8]));
  })(clone8);
  chk(isTrue(!!ids8.s1_17 && !!ids8.s2_17 && !!ids8.c1_17, '⑧ ids 链齐'));
  chk(eq(ids8.s1_17.assignedNodes().length, 1, '⑧ s1 assigned'));
  chk(isTrue(ids8.s1_17.assignedNodes()[0] === ids8.c1_17, '⑧ s1→c1'));
  chk(eq(ids8.s2_17.assignedNodes().length, 1, '⑧ s2 assigned'));
  chk(isTrue(ids8.s2_17.assignedNodes()[0] === ids8.s1_17, '⑧ s2→s1'));
  });
  // ⑨ R519（刀⑱）——MEl dispatchEvent inline on* 属性 handler 消费（s186 属性
  // 回落；WPT Event-dispatch-single-activation 残 66F 钉：form submit/reset 与
  // details toggle 属性形激活、target-only、三特判型单执行）。引用纪律＝语料
  // 同构：child clone 经 getContainer(parent) 插入（corpus 唯一装配形态），
  // child 保持 MEl 域 identity；clone root 直挂 handle div 会走宿主重建世界
  //（probe 实证），identity 比对不可用。
  stage('9', function () {
  var tpl18 = document.querySelector('template#k18tpl');
  var kids18 = tpl18.content.children;
  chk(eq(kids18.length, 4, '⑨ template18 kids'));
  function gecnInc9(e, cls) {
    var ls = [];
    try { if (e.classList.contains(cls)) ls.push(e); } catch (_g9) {}
    var sub = e.getElementsByClassName(cls);
    for (var i9 = 0; i9 < sub.length; i9++) ls.push(sub[i9]);
    return ls;
  }
  // 装配帮手：parent clone → test div；child clone → parent container（语料形）。
  // 返回 child clone 本体（MEl 域 identity 面）。
  function mount9(parentShape, childShape) {
    var t9m = document.createElement('div');
    document.getElementById('test_container').appendChild(t9m);
    var p9m = parentShape.cloneNode(true);
    t9m.appendChild(p9m);
    var c9m = childShape.cloneNode(true);
    gecnInc9(p9m, 'container')[0].appendChild(c9m);
    return c9m;
  }
  // ⑨-a submit input click → form onsubmit 属性激活（k17tpl kids[1] 复用）
  globalThis.__k17acts = [];
  var f9 = mount9(kids[1], kids[1]);
  f9.classList.add('test9');
  f9.getElementsByClassName('click')[0].click();
  chk(eq(globalThis.__k17acts.length, 1, '⑨-a submit acts len'));
  chk(isTrue(globalThis.__k17acts[0] === f9, '⑨-a form identity'));
  // ⑨-b target-only——child form 嵌套 parent form，父 onsubmit 属性不触发
  //（submit bubble 祖先站无 on* 消费——R519 只在 target 站）
  globalThis.__k17acts = [];
  var cf9 = mount9(kids[1], kids[1]);
  cf9.getElementsByClassName('click')[0].click();
  chk(eq(globalThis.__k17acts.length, 1, '⑨-b 仅 child form 激活'));
  chk(isTrue(globalThis.__k17acts[0] === cf9, '⑨-b child identity'));
  // ⑨-c reset input click → form onreset 属性激活
  globalThis.__k18acts = [];
  var rf9 = mount9(kids[1], kids18[0]);
  rf9.getElementsByClassName('click')[0].click();
  chk(eq(globalThis.__k18acts.length, 1, '⑨-c reset acts len'));
  chk(isTrue(globalThis.__k18acts[0] === rf9, '⑨-c form identity'));
  // ⑨-d summary click → details ontoggle 属性激活 + open 翻转
  globalThis.__k18acts = [];
  var d9 = mount9(kids[1], kids18[2]);
  d9.getElementsByClassName('click')[0].click();
  chk(eq(globalThis.__k18acts.length, 1, '⑨-d toggle acts len'));
  chk(isTrue(globalThis.__k18acts[0] === d9, '⑨-d details identity'));
  chk(isTrue(d9.hasAttribute('open'), '⑨-d open 翻转'));
  // ⑨-e 三特判型单执行——onclick/oninput/onchange 属性形 click 恰序一次
  //（R152/R155 特判面拥有，R519 不重入）
  globalThis.__k18acts = [];
  var cb9 = mount9(kids[1], kids18[3]);
  cb9.click();
  chk(eq(globalThis.__k18acts.join(','), 'oc,in,chg', '⑨-e 三型恰序单执行'));
  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k17_mel_region_query_pins() {
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
        "<html><head><title>k17pins</title></head><body><div id=\"test_container\"></div><template id=\"k17tpl\"><input class=\"click activates container\" type=\"checkbox\" oninput=\"globalThis.__k17acts.push(this.checked ? this : 'cbn')\"><form class=\"activates\" onsubmit=\"globalThis.__k17acts.push(this); return false\"><input class=\"click container\" type=\"submit\"></form><details class=\"activates\" ontoggle=\"globalThis.__k17acts.push(this)\"><summary class=\"click container\"></summary></details><a href=\"#link\" class=\"click container activates\"></a></template><template id=\"k18tpl\"><form class=\"activates\" onreset=\"globalThis.__k18acts.push(this)\"><input class=\"click container\" type=\"reset\"></form><form class=\"activates\" onsubmit=\"globalThis.__k18acts.push(this); return false\"><button class=\"click container\" type=\"submit\"></button></form><details class=\"activates\" ontoggle=\"globalThis.__k18acts.push(this)\"><summary class=\"click container\"></summary></details><input class=\"click\" type=\"checkbox\" onclick=\"globalThis.__k18acts.push('oc')\" oninput=\"globalThis.__k18acts.push('in')\" onchange=\"globalThis.__k18acts.push('chg')\"></template><div id=\"test_nested17\"><div id=\"host17\"><template id=\"tplA17\" data-mode=\"open\"><div id=\"host2_17\"><template id=\"tplB17\" data-mode=\"open\"><slot id=\"s2_17\" name=\"slot2\"></slot></template><slot id=\"s1_17\" name=\"slot1\" slot=\"slot2\"></slot></div></template><div id=\"c1_17\" slot=\"slot1\"></div></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/dom/events/pins-k17.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K17_MEL_REGION_QUERY_PINS_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k17 execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k17 MEl 区域查询并查钉测失败：{}", verdict);
}
