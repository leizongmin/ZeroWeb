// s10k20b 刀⑳补丁② TT CSP 政策闩锁钉测（2026-11-25）
// corpus 形蒸馏：WPT trusted-types/script-enforcement-014/015「meta tag
// removed 后 script 仍 untrusted」——meta http-equiv 交付的政策在处理时收集
// 进文档政策清单，移除 meta 元素不回收（CSP3 #meta-element）。刀⑳ 指纹重扫
// 只覆盖动态注入方向（012/013），移除方向误关 enforcement＝P2F（v3 电池
// NEW=2 定谳）；k19 首调缓存是 lib 装载期 createPolicy（__zwTtCspState 扫描）
// 恰好质 poll 为 ON 的偶然绿。钉面三向＋负面一测：
//   L1 latch 直读——createPolicy 质 poll 后移除 meta，__zwTtEnforced() 恒 true；
//   L2 014 HTML 形——trusted .text 设源后再 plain textNode 变异（fused 失配），
//     无 default policy → 阻断空消息；
//   L3 015 SVG 形——parser 投递 ";" script 摘型复用＋plain 变异 → 阻断；
//   L4 保信方向——trusted 源 fused 恒等 → 照常执行（防过度阻断回归）；
//   N1 负面页——无 meta 任何扫描 never latch，untrusted 直执行。
// 蒸馏纪律＝独立沙箱两页（带 meta 页 / 无 meta 页），互不污染闩锁闭包。
// https://w3c.github.io/trusted-types/dist/spec/#enforcement-in-scripts
// https://www.w3.org/TR/CSP3/#meta-element
const S10K20B_TT_CSP_LATCH_PIN_JS: &str = r##"
(function () {
  var LOG = 'window.log_message("RUN")';
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  globalThis.__msgs = [];
  globalThis.log_message = function (m) { globalThis.__msgs.push(String(m)); };
  // 解析投递盖章（L3 静态 ";" script 保信面兼作盖章金丝）
  try { globalThis.__zw_stamp_parser_scripts([]); } catch (e) { fails.push('FAIL:stamp-parse:' + (e && e.message || e)); }
  // corpus passthroughpolicy.js 装载形：createPolicy 即 __zwTtCspState 扫描——
  // meta 在场时 enforced 首扫＝ON，闩锁在此置位（014/015 翻绿的机制前提）
  stage('PRIME', function () {
    globalThis.__pp = trustedTypes.createPolicy('passthroughpolicy', { createScript: function (v) { return v; } });
    chk(isTrue(globalThis.__zwTtEnforced() === true, 'PRIME 后 enforcement true'));
    globalThis.document.getElementById('latchMeta').remove();
    chk(isTrue(globalThis.__zwTtEnforced() === true, '移除 meta 后闩锁保持 true'));
    globalThis.__msgs.length = 0;
  });
  // L2（014 HTML 形）：trusted 设源＋plain 变异失配 → 无政策阻断
  stage('L2', function () {
    var s = document.createElement('script');
    s.text = globalThis.__pp.createScript(';');
    s.appendChild(document.createTextNode(LOG));
    document.getElementById('latchcontainer').appendChild(s);
    chk(eq(globalThis.__msgs.join(','), '', 'L2 移除 meta 后 untrusted 阻断'));
    globalThis.__msgs.length = 0;
  });
  // L3（015 SVG 形）：parser 投递 ";" script 摘型复用＋plain 变异 → 阻断
  stage('L3', function () {
    var script = null;
    var all = document.querySelectorAll("svg script[type='unknown']");
    for (var i = 0; i < all.length; i++) {
      if (String(all[i].textContent) === ';') { script = all[i]; break; }
    }
    chk(isTrue(script != null, 'L3 静态 ";" script 在场'));
    script.remove();
    script.removeAttribute('type');
    script.appendChild(document.createTextNode(LOG));
    document.getElementById('latchcontainer').appendChild(script);
    chk(eq(globalThis.__msgs.join(','), '', 'L3 移除 meta 后 SVG untrusted 阻断'));
    globalThis.__msgs.length = 0;
  });
  // L4 保信方向：trusted 源 fused 恒等 → 照常执行（闩锁不阻断可信面）
  stage('L4', function () {
    var s = document.createElement('script');
    s.text = globalThis.__pp.createScript(LOG);
    document.getElementById('latchcontainer').appendChild(s);
    chk(eq(globalThis.__msgs.join(','), 'RUN', 'L4 trusted 源保信执行'));
    globalThis.__msgs.length = 0;
  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

// 负面页：无 meta——闩锁不得置位（防过度阻断方向回归）
const S10K20B_TT_CSP_LATCH_NEG_PIN_JS: &str = r##"
(function () {
  var LOG = 'window.log_message("RUN")';
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  globalThis.__msgs = [];
  globalThis.log_message = function (m) { globalThis.__msgs.push(String(m)); };
  try {
    chk(isTrue(globalThis.__zwTtEnforced() === false, '无 meta 页 enforcement false'));
    var s = document.createElement('script');
    s.appendChild(document.createTextNode(LOG));
    document.getElementById('latchneg').appendChild(s);
    chk(eq(globalThis.__msgs.join(','), 'RUN', 'N1 无政策页 untrusted 直执行'));
  } catch (e) { fails.push('STAGE-N1:' + (e && e.message || e)); }
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k20b_tt_csp_latch_remove_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 014/015 起始形态：CSP meta 在场（head）＋静态 svg ";" script＋svg 容器
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k20blatch</title><meta id=\"latchMeta\" http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"></head><body><div id=\"latchdiv\"></div><svg id=\"latchsvg\"><script type=\"unknown\">;</script></svg><svg id=\"latchcontainer\"></svg></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k20b.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K20B_TT_CSP_LATCH_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k20b csp-latch execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k20b CSP 闩锁钉测失败：{}", verdict);
}

#[test]
fn test_s10k20b_tt_csp_latch_negative_pins() {
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
        "<html><head><title>k20bneg</title></head><body><div id=\"latchneg\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k20b-neg.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K20B_TT_CSP_LATCH_NEG_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k20b csp-latch-neg execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k20b CSP 闩锁负面钉测失败：{}", verdict);
}
