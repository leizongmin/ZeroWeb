// s10k24 刀㉔ write 清空重置常驻钉测（2026-11-25）
// corpus 形蒸馏：WPT trusted-types/block-string-assignment-to-Document-write
// 21 面（10F→0F）——Document.write 写流是「Base 重放＋累计 buffer 全量重插」
// 模型（R5001 隐式 open 流），body 空清只移除已物化子、旧 Base/buffer 留存使
// 下次 write 全量重放复活已清内容（语料每 face `innerText=''` 开场＋write 期待
// 独立内容；got 累积翻倍实证）。钉面：
//   P1 写流物化基形——write 后 body.innerText 反映写入内容；
//   P2 body 空清生效——innerText='' 清空（融合视图同步）；
//   P3 清空后写独立内容＝核心钉面——重置后 write 不复活已清内容；
//   P4 多参拼接——write(a, b) 逐参直拼物化；
//   P5 非 body 清空不重置流——重放累计语义保留（触发面收窄到 body 本体）。
// https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#document-write-steps
const S10K24_WCR_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  var body = document.body;
  // P1 写流物化基形
  document.write('<b>abc</b>');
  chk(eq(body.innerText, 'abc', 'P1 write materialize'));
  // P2 body 空清生效
  body.innerText = '';
  chk(eq(body.innerText, '', 'P2 clear effective'));
  // P3 清空后写独立内容（无复活）＝刀㉔核心钉面
  document.write('<i>def</i>');
  chk(eq(body.innerText, 'def', 'P3 no resurrect'));
  // P4 多参拼接
  body.innerText = '';
  document.write('<b>x</b>', '<b>y</b>');
  chk(eq(body.innerText, 'xy', 'P4 multi-arg join'));
  // P5 非 body 清空不重置流（重放累计语义保留）——物化产物 <b> 子树清空，
  // 下次 write 重放复活（仅 body 本体清空重置流；触发面收窄对照）。
  body.innerText = '';
  document.write('<b>keep</b>');
  var bEl = body.querySelector('b');
  bEl.textContent = '';
  document.write('more');
  chk(eq(body.innerText, 'keepmore', 'P5 accumulation preserved'));
  return fails.length ? fails.join(';') : 'PASS';
})();
"##;

#[test]
fn test_s10k24_write_clear_reset_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 无 CSP meta：write 非强制页口径（plain string 直入 buffer；TT 管道面由
    // 语料页 block-string-assignment-to-Document-write 电池 TT 腿常驻覆盖）。
    // body 置空：占位块元素会在 innerText 产块界换行（Chrome 同型），干扰 P1/P5
    // 精确等值（P0 探针实证空 div 初始 innerText "\n"）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k24wcr</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k24-wcr.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K24_WCR_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k24 write-clear-reset execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k24 write 清空重置钉测失败：{}", verdict);
}
