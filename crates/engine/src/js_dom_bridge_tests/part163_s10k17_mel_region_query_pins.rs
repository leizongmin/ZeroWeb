// s10k17 激活链 MEl 区域查询并查钉测（刀⑰，2026-11-25）
// corpus 形蒸馏：WPT dom/events/Event-dispatch-single-activation-behavior.html
// 残 121F 根因（R517——`_zwMEl` 产物内部 mutation 纯 JS 私有树无 host wire，
// createElement 作用域 GECN/qsa 的 host 查询与 pending 桶均不可见 MEl 子树内
// 元素 → stamp/href 相位漏嵌套 target → classList 缺 test{i} → filter 全弃
// got=[]；R518——A/AREA href IDL getter 相对串原样回落，对 hashchange
// e.newURL 绝对形失配）的常驻回归钉：stamp 相位 parent+target 双命中与本体
// identity／filter 相位 produced==expected 同本体／qsa href 相位嵌套 aMEl 命中
// ＋getter 绝对＋attr 原串／MEl click 同步片段导航／无 MEl handle 域零扰动／
// 深链 container 双命中＋无双计／href 基选择与 R130 空串口径。
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
        "<html><head><title>k17pins</title></head><body><div id=\"test_container\"></div><template id=\"k17tpl\"><input class=\"click activates container\" type=\"checkbox\" oninput=\"globalThis.__k17acts.push(this.checked ? this : 'cbn')\"><form class=\"activates\" onsubmit=\"globalThis.__k17acts.push(this); return false\"><input class=\"click container\" type=\"submit\"></form><details class=\"activates\" ontoggle=\"globalThis.__k17acts.push(this)\"><summary class=\"click container\"></summary></details><a href=\"#link\" class=\"click container activates\"></a></template></body></html>".to_string(),
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
