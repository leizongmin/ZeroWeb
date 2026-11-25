// s10k19 TT script 身份门钉测（刀⑲，2026-11-25）
// corpus 形蒸馏：WPT trusted-types/script-enforcement-001/002（enforcement
// 无/有 default policy 双页）的 trust/drop/rescue 全矩阵。刀⑲ R521 面：
// 声明 sink（HTMLScriptElement.prototype text/textContent/innerText accessor）
// setter 期管道 + 快照盖章；Node.prototype.textContent / HTMLElement.prototype
// innerText 通用 descriptor 直调 bypass（不盖章 → prepare 时身份救援）；
// splitText 保信（fused 恒等）；nodeValue 变异失配阻断；cloneNode 丢信
//（clone 无快照键）；default policy prepare 时救援（sink 'HTMLScriptElement
// text'/'SVGScriptElement text' 分型、返回串替换执行体）+ 声明 sink setter
// 期管道消息序。引用纪律＝钉页 meta CSP 激活 enforcement（_zwTtEnforced meta
// 扫描面）；无政策阶段先于 default policy 创建（政策在场即翻臂）。
const S10K19_TT_SCRIPT_PROVENANCE_PINS_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  function throwsTf(fn, m) {
    try { fn(); } catch (e) { return eq(e && e.name, 'TypeError', m); }
    return 'FAIL:' + m + '(no-throw)';
  }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  var log;
  function resetLog() { log = []; globalThis.__k19log = log; }
  resetLog();
  var container = document.getElementById('k19container');
  function runLog(src) {
    resetLog();
    var s = document.createElement('script');
    return s;
  }
  // ① 声明 sink 保信——script.text = TS(...) → 快照盖章 → prepare 等值 → 执行
  stage('1', function () {
    var ts = trustedTypes.createPolicy('pins19passthrough', { createScript: function (v) { return v; } });
    var s = runLog();
    s.text = ts.createScript('globalThis.__k19log.push("RUN19A")');
    container.appendChild(s);
    chk(eq(log.join(','), 'RUN19A', '① 声明 sink 执行'));
  });
  // ② Node.prototype.textContent descriptor 直调 bypass——不盖章 → prepare 阻断
  //（无政策）＋ getter 回读原文（通用读臂）
  stage('2', function () {
    var d = Object.getOwnPropertyDescriptor(globalThis.Node.prototype, 'textContent');
    chk(isTrue(!!d && typeof d.set === 'function', '② descriptor 在位'));
    var s = runLog();
    d.set.call(s, 'globalThis.__k19log.push("RUN19B")');
    chk(eq(s.textContent, 'globalThis.__k19log.push("RUN19B")', '② 回读原文'));
    container.appendChild(s);
    chk(eq(log.join(','), '', '② bypass 阻断（无政策）'));
  });
  // ③ HTMLElement.prototype.innerText descriptor 直调 bypass——同②判据面
  stage('3', function () {
    var d = Object.getOwnPropertyDescriptor(globalThis.HTMLElement.prototype, 'innerText');
    chk(isTrue(!!d && typeof d.set === 'function', '③ descriptor 在位'));
    var s = runLog();
    d.set.call(s, 'globalThis.__k19log.push("RUN19C")');
    chk(eq(s.innerText, 'globalThis.__k19log.push("RUN19C")', '③ 回读原文'));
    container.appendChild(s);
    chk(eq(log.join(','), '', '③ bypass 阻断'));
  });
  // ④ innerHTML createHTML 落树——无政策 prepare 阻断（TrustedHTML sink 管道
  // 在位：plain 串 TypeError 于解析前）
  stage('4', function () {
    chk(throwsTf(function () { document.createElement('script').innerHTML = 'x'; }, '④ plain TypeError'));
    var ph = trustedTypes.createPolicy('pins19html', { createHTML: function (v) { return v; } });
    var s = runLog();
    s.innerHTML = ph.createHTML('globalThis.__k19log.push("RUN19D")');
    container.appendChild(s);
    chk(eq(log.join(','), '', '④ createHTML 落树阻断'));
  });
  // ⑤ splitText 保信——fused 恒等 → 快照等值 → 执行
  stage('5', function () {
    var ts5 = trustedTypes.createPolicy('pins19p5', { createScript: function (v) { return v; } });
    var s = runLog();
    s.text = ts5.createScript('globalThis.__k19log.push("RUN19E");;;');
    s.firstChild.splitText(3);
    container.appendChild(s);
    chk(eq(log.join(','), 'RUN19E', '⑤ splitText 保信'));
  });
  // ⑥ nodeValue 变异失配——快照失配 → 阻断
  stage('6', function () {
    var ts6 = trustedTypes.createPolicy('pins19p6', { createScript: function (v) { return v; } });
    var s = runLog();
    s.text = ts6.createScript(';');
    s.firstChild.nodeValue = 'globalThis.__k19log.push("RUN19F")';
    container.appendChild(s);
    chk(eq(log.join(','), '', '⑥ 变异阻断'));
  });
  // ⑦ cloneNode 丢信——clone 无快照键 → 阻断（快照不得随 sel/handle 泄漏到克隆）
  stage('7', function () {
    var ts7 = trustedTypes.createPolicy('pins19p7', { createScript: function (v) { return v; } });
    var s = runLog();
    s.text = ts7.createScript('globalThis.__k19log.push("RUN19G")');
    var c = s.cloneNode(true);
    container.appendChild(c);
    chk(eq(log.join(','), '', '⑦ clone 阻断'));
  });
  // ⑧ default policy prepare 时救援——appendChild(createTextNode) bypass 形：
  // 消息序 SET/APPEND/CREATE_SCRIPT/'HTMLScriptElement text'/RUNX（返回串替换
  // 执行体——'RUN' → 'RUNX' 由政策改写，树文本不变）
  stage('8', function () {
    trustedTypes.createPolicy('default', {
      createScript: function (v, t, sink) {
        log.push('CREATE_SCRIPT'); log.push(sink);
        return v.replace('RUN', 'RUNX');
      },
    });
    var d = Object.getOwnPropertyDescriptor(globalThis.Node.prototype, 'textContent');
    var s = document.createElement('script');
    log.push('SET');
    d.set.call(s, 'globalThis.__k19log.push("RUN19H")');
    log.push('APPEND');
    container.appendChild(s);
    chk(eq(log.join(','), 'SET,APPEND,CREATE_SCRIPT,HTMLScriptElement text,RUNX19H', '⑧ prepare 救援序'));
  });
  // ⑨ SVG script 救援——sink 'SVGScriptElement text'（接口名分型；SVG textContent
  // 非声明 sink，generic 写不盖章）
  stage('9', function () {
    resetLog();
    var s = document.createElementNS('http://www.w3.org/2000/svg', 'script');
    log.push('SET');
    s.textContent = 'globalThis.__k19log.push("RUN19I")';
    log.push('APPEND');
    container.appendChild(s);
    chk(eq(log.join(','), 'SET,APPEND,CREATE_SCRIPT,SVGScriptElement text,RUNX19I', '⑨ SVG 救援序'));
  });
  // ⑩ 声明 sink setter 期管道（default policy 在场）——消息序 SET/CREATE_SCRIPT/
  // sink/APPEND/RUNX（变换值落树 + 快照＝变换后 fused → prepare 等值直执行）
  stage('10', function () {
    resetLog();
    var s = document.createElement('script');
    log.push('SET');
    s.text = 'globalThis.__k19log.push("RUN19J")';
    log.push('APPEND');
    container.appendChild(s);
    chk(eq(log.join(','), 'SET,CREATE_SCRIPT,HTMLScriptElement text,APPEND,RUNX19J', '⑩ setter 期管道序'));
  });
  // ⑪ CharacterData 变异族 before/after——本地视图节点缺法回归钉（s10k19
  // 补 before/after/splitText/remove/replaceWith 后 fused ≠ 快照 → default
  // policy 救援序；001 同族判据面为无政策阻断）
  stage('11', function () {
    resetLog();
    var s1 = document.createElement('script');
    log.push('SET');
    s1.text = trustedTypes.createPolicy('pins19p11', { createScript: function (v) { return v; } }).createScript(';');
    s1.firstChild.before(document.createTextNode('globalThis.__k19log.push("RUN19K");'));
    log.push('APPEND');
    container.appendChild(s1);
    var s2 = document.createElement('script');
    log.push('SET2');
    s2.text = trustedTypes.createPolicy('pins19p11b', { createScript: function (v) { return v; } }).createScript(';');
    s2.firstChild.after(document.createTextNode('globalThis.__k19log.push("RUN19L");'));
    log.push('APPEND2');
    container.appendChild(s2);
    chk(eq(log.join(','), 'SET,APPEND,CREATE_SCRIPT,HTMLScriptElement text,RUNX19K,SET2,APPEND2,CREATE_SCRIPT,HTMLScriptElement text,RUNX19L', '⑪ before/after 变异救援'));
  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k19_tt_script_provenance_pins() {
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
        "<html><head><title>k19pins</title><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"></head><body><div id=\"k19container\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k19.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K19_TT_SCRIPT_PROVENANCE_PINS_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k19 execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k19 TT script 身份门钉测失败：{}", verdict);
}
