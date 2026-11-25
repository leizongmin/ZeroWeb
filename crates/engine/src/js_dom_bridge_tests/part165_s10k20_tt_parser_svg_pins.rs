// s10k20 刀⑳ TT parser svg script 变异域钉测（2026-11-25）
// corpus 形蒸馏：WPT trusted-types/script-enforcement-003/004 svg 脚本变异面
//（无 default policy 阻断 / 有 default policy prepare 救援双模型）。刀⑳ R522 面：
// ① FIX-A——detached 变异窗口（splitText/normalize/removeChild/remove/
//   ChildNode before/after/replaceWith/nodeValue 等中点）prepare 的 connected
//   门不再信 `__zw_contains` 裸 dom_html 反链（pending 变异未 drain 窗口内
//   mutation-blind 假真），parentNode null 直接 false——抢置 started 章消失，
//   append 时点 prepare 完整走 k19 trust 快照门（钉面每 face 附 pre-append
//   started 表断言）；
// ② FIX-D——sel 域 parsed Text/Comment 的 ChildNode before/after/replaceWith
//   （part05 _wrapNodeEntry 安装面，此前 TypeError 簇）；
// ③ FIX-E——撤刀⑫ script 容器 `__zwFragNoExec` 硬阻断章，HTML sink
//   （setHTMLUnsafe/insertAdjacentHTML）设源回归 k19 provenance 门：无政策
//   阻断消息面不变，有政策救援可达（7 消息序与语料逐项恒等）。
// 蒸馏纪律＝每 face 独立 svg（sel 命名空间隔离——语料静态池共享 sel，池漂移
// 碰撞是挂账 刀㉑ clone 面研究对象，不在本钉面）；无政策阶段先于 default
// policy 创建（政策在场即翻臂，同 part164 纪律）；trust 快照由 parser 投递
// 盖章（`__zw_stamp_parser_scripts` 模拟宿主投递，保信面兼作盖章机器金丝）。
// exclude：cloneNode/cloneContents face（s95 run-once 表池漂移假阳——挂账
// 刀㉑）；002 html 域 replaceChild/insertAdjacentText（registry entry.nodes
// sync 假说——挂账 刀㉑，solo TT 腿实证后钉）。
// https://w3c.github.io/trusted-types/dist/spec/#enforcement-in-scripts
// https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
const S10K20_TT_PARSER_SVG_PINS_JS: &str = r##"
(function () {
  var SVGNS = 'http://www.w3.org/2000/svg';
  var LOG = 'window.log_message("RUN")';
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stage(name, fn) { try { fn(); } catch (e) { fails.push('STAGE-' + name + ':' + (e && e.message || e)); } }
  var log;
  function resetLog() { log = []; globalThis.__k20log = log; }
  resetLog();
  globalThis.log_message = function (m) { log.push(String(m)); };
  var container = document.getElementById('k20container');
  // 解析投递盖章（模拟宿主 parse 后投递扫描；A1 保信面兼作盖章金丝——未盖章
  // 时 miss 臂阻断，A1 期望 RUN 即失败）
  try { globalThis.__zw_stamp_parser_scripts([]); } catch (e) { fails.push('FAIL:stamp-parse:' + (e && e.message || e)); }
  function takeStatic(svgId) {
    var sv = document.getElementById(svgId);
    var s = sv.querySelector('script');
    s.remove();
    s.removeAttribute('type');
    return s;
  }
  // FIX-A 负面断言：变异中点后、append 前本 face sel 不得已在 started 表
  //（抢置＝detached 窗口 prepare 误跑复发，append 时点 prepare 将被跳过）
  function assertNotStarted(s, tag) {
    var sel = '';
    try { sel = String(s.__zwSelector || ''); } catch (e) {}
    var marked = false;
    try { marked = !!globalThis._zwScriptStarted[sel]; } catch (e) {}
    chk(isTrue(!marked, tag + ' pre-append started 未抢置(' + sel.slice(-30) + ')'));
  }
  var RESCUE = 'SET,APPEND,CREATE_SCRIPT,SVGScriptElement text,RUN_TRUSTED_SCRIPT';
  function rescueFace(tag, svgId, mutate) {
    stage(tag, function () {
      resetLog();
      var s = svgId ? takeStatic(svgId) : document.createElementNS(SVGNS, 'script');
      log.push('SET');
      mutate(s);
      log.push('APPEND');
      assertNotStarted(s, tag);
      container.appendChild(s);
      chk(eq(log.join(','), RESCUE, tag + ' 救援序'));
    });
  }
  function blockFace(tag, svgId, mutate) {
    stage(tag, function () {
      resetLog();
      var s = takeStatic(svgId);
      mutate(s);
      assertNotStarted(s, tag);
      container.appendChild(s);
      chk(eq(log.join(','), '', tag + ' 无政策阻断'));
    });
  }
  // withLogSetAppend：004 语料 face 体自带 SET/APPEND 打点（003 语料无）
  function keepFace(tag, svgId, mutate, expect, withLogSetAppend) {
    stage(tag, function () {
      resetLog();
      var s = takeStatic(svgId);
      if (withLogSetAppend) log.push('SET');
      mutate(s);
      if (withLogSetAppend) log.push('APPEND');
      assertNotStarted(s, tag);
      container.appendChild(s);
      chk(eq(log.join(','), expect, tag + ' 保信执行'));
    });
  }
  // ── Phase A：无 default policy（003 模型）——丢信阻断空消息面＋保信直执行 ──
  trustedTypes.createPolicy('k20passthrough', { createScript: function (v) { return v; } });
  keepFace('A1', 'k20a1', function (s) { s.firstChild.splitText(3); }, 'RUN');
  keepFace('A2', 'k20a2', function (s) { s.firstChild.splitText(3); s.normalize(); }, 'RUN');
  blockFace('A3', 'k20a3', function (s) { s.firstChild.nodeValue = LOG; });
  blockFace('A4', 'k20a4', function (s) { s.firstChild.before(LOG); });
  blockFace('A5', 'k20a5', function (s) { s.firstChild.after(LOG); });
  blockFace('A6', 'k20a6', function (s) { s.firstChild.replaceWith(document.createTextNode(LOG)); });
  blockFace('A7', 'k20a7', function (s) { s.firstChild.splitText(3); s.removeChild(s.firstChild); });
  blockFace('A8', 'k20a8', function (s) { s.firstChild.splitText(3); s.firstChild.remove(); });
  // ── Phase B：default policy 在场（004 模型）——变异失配 prepare 救援 ──
  trustedTypes.createPolicy('default', {
    createHTML: function (v, t, sink) { log.push('CREATE_HTML'); log.push(sink); return v.replace('RUN', 'RUN_TRUSTED_HTML'); },
    createScript: function (v, t, sink) { log.push('CREATE_SCRIPT'); log.push(sink); return v.replace('RUN', 'RUN_TRUSTED_SCRIPT'); },
  });
  // B1 created svg script + textContent 通用写（无盖章）→ prepare 救援
  rescueFace('B1', null, function (s) { s.textContent = LOG; });
  // B2-B7 CharacterData 直写族（nodeValue/data/appendData/insertData/
  // replaceData/deleteData）——detached 变异窗口 FIX-A 面
  rescueFace('B2', 'k20b2', function (s) { s.firstChild.nodeValue = LOG; });
  rescueFace('B3', 'k20b3', function (s) { s.firstChild.data = LOG; });
  rescueFace('B4', 'k20b4', function (s) { s.firstChild.appendData(LOG); });
  rescueFace('B5', 'k20b5', function (s) { s.firstChild.insertData(0, LOG); });
  rescueFace('B6', 'k20b6', function (s) { s.firstChild.replaceData(0, 0, LOG); });
  rescueFace('B7', 'k20b7', function (s) { s.firstChild.deleteData(0, 2); });
  // B8-B10 FIX-D：sel 域 parsed Text 的 ChildNode before/after/replaceWith
  rescueFace('B8', 'k20b8', function (s) { s.firstChild.before(LOG); });
  rescueFace('B9', 'k20b9', function (s) { s.firstChild.after(LOG); });
  rescueFace('B10', 'k20b10', function (s) { s.firstChild.replaceWith(document.createTextNode(LOG)); });
  // B11 CharacterData.remove（;;;LOG + splitText(3) 摘头段）
  rescueFace('B11', 'k20b11', function (s) { s.firstChild.splitText(3); s.firstChild.remove(); });
  // B12-B18 Node/Element 插拔族（appendChild/insertBefore/replaceChild/
  // removeChild/prepend/append/replaceChildren）
  rescueFace('B12', 'k20b12', function (s) { s.appendChild(document.createTextNode(LOG)); });
  rescueFace('B13', 'k20b13', function (s) { s.insertBefore(document.createTextNode(LOG), s.firstChild); });
  rescueFace('B14', 'k20b14', function (s) { s.replaceChild(document.createTextNode(LOG), s.firstChild); });
  rescueFace('B15', 'k20b15', function (s) { s.firstChild.splitText(3); s.removeChild(s.firstChild); });
  rescueFace('B16', 'k20b16', function (s) { s.prepend(document.createTextNode(LOG)); });
  rescueFace('B17', 'k20b17', function (s) { s.append(document.createTextNode(LOG)); });
  rescueFace('B18', 'k20b18', function (s) { s.replaceChildren(document.createTextNode(LOG)); });
  // B19/B20 保信（splitText/normalize）——004 期望 SET,APPEND,RUN
  keepFace('B19', 'k20b19', function (s) { s.firstChild.splitText(3); }, 'SET,APPEND,RUN', true);
  keepFace('B20', 'k20b20', function (s) { s.firstChild.splitText(3); s.normalize(); }, 'SET,APPEND,RUN', true);
  // B21 FIX-E：setHTMLUnsafe HTML sink 设源（created svg script）——7 消息序
  stage('B21', function () {
    resetLog();
    var s = document.createElementNS(SVGNS, 'script');
    log.push('SET');
    s.setHTMLUnsafe(LOG);
    log.push('APPEND');
    container.appendChild(s);
    chk(eq(log.join(','), 'SET,CREATE_HTML,Element setHTMLUnsafe,APPEND,CREATE_SCRIPT,SVGScriptElement text,RUN_TRUSTED_SCRIPT_TRUSTED_HTML', 'B21 setHTMLUnsafe 救援序'));
  });
  // B22 insertAdjacentHTML afterbegin（parser 静态作宿主；slot 补偿融合文本）
  stage('B22', function () {
    resetLog();
    var s = takeStatic('k20b22');
    log.push('SET');
    s.insertAdjacentHTML('afterbegin', LOG);
    log.push('APPEND');
    assertNotStarted(s, 'B22');
    container.appendChild(s);
    chk(eq(log.join(','), 'SET,CREATE_HTML,Element insertAdjacentHTML,APPEND,CREATE_SCRIPT,SVGScriptElement text,RUN_TRUSTED_SCRIPT_TRUSTED_HTML', 'B22 insertAdjacentHTML 救援序'));
  });
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k20_tt_parser_svg_script_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    // 每 face 独立 svg（sel 命名空间隔离，见文件头蒸馏纪律）
    const K20_SVG_FACES: &[(&str, &str)] = &[
        ("k20a1", "window.log_message(\"RUN\");;;"),
        ("k20a2", "window.log_message(\"RUN\");;;"),
        ("k20a3", ";"),
        ("k20a4", ";"),
        ("k20a5", ";"),
        ("k20a6", ";"),
        ("k20a7", ";;;window.log_message(\"RUN\")"),
        ("k20a8", ";;;window.log_message(\"RUN\")"),
        ("k20b2", ";"),
        ("k20b3", ";"),
        ("k20b4", ";"),
        ("k20b5", ";"),
        ("k20b6", ";"),
        ("k20b7", "//window.log_message(\"RUN\")"),
        ("k20b8", ";"),
        ("k20b9", ";"),
        ("k20b10", ";"),
        ("k20b11", ";;;window.log_message(\"RUN\")"),
        ("k20b12", ";"),
        ("k20b13", ";"),
        ("k20b14", ";"),
        ("k20b15", ";;;window.log_message(\"RUN\")"),
        ("k20b16", ";"),
        ("k20b17", ";"),
        ("k20b18", ";"),
        ("k20b19", "window.log_message(\"RUN\");;;"),
        ("k20b20", "window.log_message(\"RUN\");;;"),
        ("k20b22", ";"),
    ];
    let svgs: String = K20_SVG_FACES
        .iter()
        .map(|(id, c)| format!("<svg id=\"{id}\"><script type=\"unknown\">{c}</script></svg>"))
        .collect();
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(format!(
        "<html><head><title>k20pins</title><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"></head><body><div id=\"k20container\"></div>{svgs}</body></html>"
    )));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k20.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K20_TT_PARSER_SVG_PINS_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k20 execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k20 TT parser svg 变异域钉测失败：{}", verdict);
}

