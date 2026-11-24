// s10k15 变参突变方法基座面钉测（刀⑮，2026-11-25）
// corpus 形蒸馏：WPT trusted-types/Node-multiple-arguments{,-tt-enforced}.html
// 90+90F 双根因修复的常驻回归钉：提取方法对象 apply 接收者落位（mixin `this`
// 语义）／before-after 相对位真插入／replaceWith concept-node-replace 序／
// union (Node or DOMString) 转换（无 nodeType 对象 ToString 铸 Text）／
// CharacterData.prototype.textContent 原型成员（corpus 期望侧 stringify 判定）
// ／doc 域 HRE 校验保留／detached 无父链校验-only 旧语义保留。
const S10K15_VARIADIC_MUTATION_PINS_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + a + '!=' + b + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function stringify(arg) { return 'textContent' in Object.getPrototypeOf(arg) ? arg.textContent : arg.toString(); }
  function mkText(s) { return document.createTextNode(s); }
  function mkEl(t) { return document.createElement(t); }

  // ① 提取方法对象 apply 接收者落位（主根因：旧闭包绑 target）——corpus 同构：
  //   query 产品的方法对象 apply 到 createElement 产物（直挂 body，同 corpus 页挂载）
  var outer1 = mkEl('div'); var inner1 = mkEl('p');
  outer1.appendChild(inner1); document.body.appendChild(outer1);
  var r1 = mkEl('span'); r1.appendChild(mkText('rep'));
  outer1.replaceWith.apply(inner1, [mkText('x'), r1]);
  chk(eq(outer1.textContent, "xrep", '① replaceWith on receiver'));
  // ② before/after 相对位真插入（旧校验-only 桩不插）
  var outer2 = mkEl('div'); var inner2 = mkEl('p');
  outer2.appendChild(inner2); document.body.appendChild(outer2);
  outer2.before.apply(inner2, ['b1', 'b2']);
  outer2.after.apply(inner2, ['a1']);
  chk(eq(outer2.textContent, 'b1b2a1', '② before/after position'));
  // ③ union 转换：无 nodeType 对象（TrustedScript-like）ToString 铸 Text
  var fakeTS = { toString: function () { return "'TS';"; } };
  var outer3 = mkEl('div'); document.body.appendChild(outer3);
  outer3.append.apply(outer3, [fakeTS, mkText('m'), 'str']);
  chk(eq(outer3.textContent, "'TS';mstr", '③ union conversion'));
  // ④ CharacterData.prototype.textContent 原型成员（corpus 期望侧判定面）
  chk(eq(('textContent' in Object.getPrototypeOf(mkText('x'))), true, '④ proto has textContent'));
  chk(eq(stringify(mkText('n1')), 'n1', '④ stringify text node'));
  chk(eq(('textContent' in Object.getPrototypeOf(fakeTS)), false, '④ ts proto no textContent'));
  // ⑤ 原型 textContent 读/写面（缺 own 成员实例经原型取 data）
  var t5 = mkText('d0');
  chk(eq(t5.textContent, 'd0', '⑤ read'));
  t5.textContent = 'd1';
  chk(eq(t5.data, 'd1', '⑤ write-through data'));
  // ⑥ replaceWith spec 序：先 pre-insert 群到 node 前，再 remove（接收者移除）
  var outer6 = mkEl('div'); var mid6 = mkEl('p'); var tail6 = mkText('t8');
  outer6.appendChild(mid6); outer6.appendChild(tail6); document.body.appendChild(outer6);
  mid6.replaceWith('r8');
  chk(eq(outer6.textContent, 'r8t8', '⑥ replaceWith order'));
  // ⑦ detached 无父链：before 校验-only 旧语义保留（不抛不插）
  var det7 = mkEl('div');
  var threw7 = '';
  try { det7.before.call(det7, 'x'); } catch (e) { threw7 = e.name || ''; }
  chk(eq(threw7, '', '⑦ detached before no-throw'));
  return fails.length ? fails.join('; ') : 'PASS';
})();
"##;

#[test]
fn test_s10k15_variadic_mutation_pins() {
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
        "<html><head><title>k15pins</title></head><body><div id=\"k15container\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k15.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    let verdict = sandbox.execute(S10K15_VARIADIC_MUTATION_PINS_JS).unwrap().value;
    assert_eq!(verdict, "PASS", "s10k15 变参突变方法钉测失败：{}", verdict);
}
