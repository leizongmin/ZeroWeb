// s10k25 刀㉕ parsed-doc write/writeln 常驻钉测（2026-11-26）
// corpus 形蒸馏：WPT trusted-types/Document-write-appending-line-feed
// 6F→0F——`DOMParser.parseFromString` 产物文档缺 write/writeln（6F 全形态
// 'doc.write/writeln is not a function'）。产物是独立 inert 文档：不走主文档
// 写流（part06 buffer/Base 重放全局单例），写入物化进本产物 body；物化语义＝
// **每次调用隐式 open**（spec write steps：write-writable 要求 active parser
// 未中止——parsed 文档解析完即无 active parser，恒 not write-writable → 每次
// write 先 open 清空再写；Chrome 实证：parse 管道产物 body 初始 "[]"，face1
// write 后精确 "(1)(2)(3)(4)"＝初始内容被清而非叠加）。TT 管道镜像主文档
// write 口径：逐参 String 拼接（Trusted 关联数据串直拼），含 plain 分量→
// 拼接串整体过**一次** sink 管道，全 Trusted→直通；writeln '\n' 管道后置。
// 钉面（host DOM 带 CSP meta＝真 enforced harness——sink 管道全链真生效；
// `_zwTtSinkValue` 内部闭包局部 `_zwTtEnforced` 引用，测内翻转 globalThis
// 属性不可达，故不走翻转法；非 enforced 直入臂由 debug 单页与无 CSP 页
// 语料覆盖，不入钉面）：
//   P1 方法面——parsed doc write/writeln 存在；
//   P2 Trusted-only 直通（"(1)(2)(3)(4)"）；
//   P3 plain-only 过 default policy 一次（"[1234]"）；
//   P4 混合参整体一次管道（"[(1)(2)3(4)]"）；
//   P5 writeln 尾 \n 管道后置（"[1234]\n"）；
//   P6 隐式 open 替换＝核心钉面——parse 管道产物初始 "[]" 被首 write 清掉，
//      连续 write 各自替换（非累计）。
// https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#dom-document-write
const S10K25_PDW_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  var doc = null, doc2 = null;
  var tt = globalThis.trustedTypes;
  if (!tt || typeof tt.createPolicy !== 'function') return 'FAIL:no-tt-factory';
  // default policy 名字面 'default'（管道按名查找；其他名不命中）。
  var defPolicy = tt.createPolicy('default', { createHTML: function (x) { return '[' + x + ']'; } });
  var custPolicy = tt.createPolicy('k25custom', { createHTML: function (x) { return '(' + x + ')'; } });
  doc = (new DOMParser()).parseFromString('<body></body>', 'text/html');
  // P1 方法面
  chk(eq(typeof doc.write, 'function', 'P1 write typeof'));
  chk(eq(typeof doc.writeln, 'function', 'P1 writeln typeof'));
  // P2 Trusted-only 直通
  doc.write(custPolicy.createHTML('1'), custPolicy.createHTML('2'), custPolicy.createHTML('3'), custPolicy.createHTML('4'));
  chk(eq(doc.body.innerHTML, '(1)(2)(3)(4)', 'P2 trusted-only'));
  // P3 plain-only 过 default policy 一次
  doc.write('1', '2', '3', '4');
  chk(eq(doc.body.innerHTML, '[1234]', 'P3 plain default-once'));
  // P4 混合参整体一次管道
  doc.write(custPolicy.createHTML('1'), custPolicy.createHTML('2'), '3', custPolicy.createHTML('4'));
  chk(eq(doc.body.innerHTML, '[(1)(2)3(4)]', 'P4 mixed'));
  // P5 writeln 尾 \n 管道后置
  doc.writeln('1', '2', '3', '4');
  chk(eq(doc.body.innerHTML, '[1234]\n', 'P5 writeln newline'));
  // P6 隐式 open 替换＝核心钉面：parse 管道产物初始 "[]"（sink 管道常驻钉），
  // 首 write 清掉；连续 write 各自替换非累计（plain 参经 default policy 包裹）。
  doc2 = (new DOMParser()).parseFromString('<body></body>', 'text/html');
  chk(eq(doc2.body.innerHTML, '[]', 'P6 initial pipe artifact'));
  doc2.write('<b>first</b>');
  chk(eq(doc2.body.innerHTML, '[<b>first</b>]', 'P6 write1'));
  doc2.write('<i>second</i>');
  chk(eq(doc2.body.innerHTML, '[<i>second</i>]', 'P6 open-replace'));
  return fails.length ? fails.join(';') : 'PASS';
})();
"##;

#[test]
fn test_s10k25_parsed_doc_write_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // host DOM 带 CSP meta＝真 enforced harness（sink 管道全链真生效，镜像语料
    // 页 require-trusted-types-for 语境）；body 置空避免占位块元素干扰。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"><title>k25pdw</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k25-pdw.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K25_PDW_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k25 parsed-doc write execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k25 parsed-doc write/writeln 钉测失败：{}", verdict);
}
