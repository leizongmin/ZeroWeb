// js_dom_bridge 测试切片 57（R3907）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-56 共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。
// part16 已近 2000 行上限（rule 5），新切片承载 html-semantics M2 契约测试。

// ── R3907：option 元素文本/反射语义（html-semantics M2 切片 7）──
//
// 生产 shim 路径：part03 `_zwOptChildText`/`_zwOptAttrNoNs`（child text content +
// strip-and-collapse + 无 ns 内容属性反射）+ part04 option.text setter（textContent 委托）。
// spec https://html.spec.whatwg.org/multipage/form-elements.html#dom-option-text 。
// 验证：W1 strip/collapse 模型（首尾剥离/内侧折叠/NBSP 保留）；W2 collect text value
//（元素递归/HTML+SVG script 跳过/MathML+null ns script 递归/comment 不计）；W3 label 反射
//（无 ns 存在即采纳含空串/NS 影子不可见/子文本回落）；W4 value 反射同构；W5 text setter
// 替换全部子（旧文本子脱离树）；W6 解析 option（sel 形）value/text 口径；W7 select.value
// 宿主路径不回退。
#[test]
fn test_option_semantics_r3907() {
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
        "<html><body>\
         <form><select id='s'><option value='foo'>bar</option></select></form>\
         <option id='o2'>  txt  </option>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// W1：strip and collapse ASCII whitespace（首尾剥离 + 内侧折叠单空格；NBSP 不动）。
var o1 = document.createElement('option');
o1.textContent = ' \t\n text \f\r ';
globalThis.__w1a = o1.text;
var o1b = document.createElement('option');
o1b.textContent = 'before\t\tafter';
globalThis.__w1b = o1b.text;
var o1c = document.createElement('option');
o1c.textContent = ' x ';
globalThis.__w1c = o1c.text;
// W2：collect text value——元素递归；HTML/SVG script 跳过；MathML/null ns script 递归；comment 不计。
var o2 = document.createElement('option');
o2.appendChild(document.createElement('font')).appendChild(document.createTextNode(' font '));
globalThis.__w2a = o2.text;
var o2b = document.createElement('option');
o2b.appendChild(document.createTextNode(' before '));
o2b.appendChild(document.createElement('script')).appendChild(document.createTextNode(' script '));
o2b.appendChild(document.createTextNode(' after '));
globalThis.__w2b = o2b.text;
var o2c = document.createElement('option');
o2c.appendChild(document.createTextNode(' before '));
o2c.appendChild(document.createElementNS('http://www.w3.org/2000/svg', 'script'))
   .appendChild(document.createTextNode(' script '));
o2c.appendChild(document.createTextNode(' after '));
globalThis.__w2c = o2c.text;
var o2d = document.createElement('option');
o2d.appendChild(document.createTextNode(' before '));
o2d.appendChild(document.createElementNS('http://www.w3.org/1998/Math/MathML', 'script'))
   .appendChild(document.createTextNode(' script '));
o2d.appendChild(document.createTextNode(' after '));
globalThis.__w2d = o2d.text;
var o2e = document.createElement('option');
o2e.appendChild(document.createTextNode(' before '));
o2e.appendChild(document.createElementNS(null, 'script')).appendChild(document.createTextNode(' script '));
o2e.appendChild(document.createTextNode(' after '));
globalThis.__w2e = o2e.text;
var o2f = document.createElement('option');
o2f.appendChild(document.createTextNode('x'));
o2f.appendChild(document.createComment('c'));
globalThis.__w2f = o2f.text;
var o2g = document.createElement('option');
var o2gSpan = o2g.appendChild(document.createElement('span'));
o2gSpan.appendChild(document.createTextNode(' Some '));
o2gSpan.appendChild(document.createElement('script')).appendChild(document.createTextNode(' script '));
o2g.appendChild(document.createTextNode(' Text '));
globalThis.__w2g = o2g.text;
// W3：label 反射——无 ns 属性存在即采纳（空串含内）；NS 影子不可见；缺省回落 child text content。
var o3a = document.createElement('option');
o3a.setAttribute('label', '');
globalThis.__w3a = o3a.label;
var o3b = document.createElement('option');
o3b.setAttribute('label', 'L');
o3b.textContent = 'child';
globalThis.__w3b = o3b.label;
var o3c = document.createElement('option');
o3c.setAttributeNS('http://www.example.com/', 'label', 'L');
globalThis.__w3c = o3c.label;
var o3d = document.createElement('option');
o3d.setAttributeNS('http://www.example.com/', 'label', 'L');
o3d.appendChild(document.createTextNode(' child '));
globalThis.__w3d = o3d.label;
var o3e = document.createElement('option');
o3e.appendChild(document.createTextNode(' child '));
globalThis.__w3e = o3e.label;
// W4：value 反射同构（attr 空串采纳；NS 影子不可见；子文本回落；attr 优先于子文本）。
var o4a = document.createElement('option');
o4a.setAttributeNS('http://www.example.com/', 'value', 'V');
globalThis.__w4a = o4a.value;
var o4b = document.createElement('option');
o4b.appendChild(document.createTextNode(' child '));
globalThis.__w4b = o4b.value;
var o4c = document.createElement('option');
o4c.setAttribute('value', '');
o4c.appendChild(document.createTextNode(' child '));
globalThis.__w4c = o4c.value;
var o4d = document.createElement('option');
o4d.setAttribute('value', 'V');
o4d.appendChild(document.createTextNode(' child '));
globalThis.__w4d = o4d.value;
// W5：text setter 替换全部子——旧文本子脱离树（node identity 更替）；''.text 清子。
var o5 = document.createElement('option');
o5.appendChild(document.createTextNode('bar'));
var o5old = o5.firstChild;
o5.text = 'baz';
globalThis.__w5a = (o5old !== o5.firstChild) + '/' + o5old.nodeValue + '/' + (o5old.parentNode === null);
globalThis.__w5b = o5.text;
o5.text = '';
globalThis.__w5c = String(o5.firstChild === null) + '/' + o5.text;
// W6：解析 option（sel 形）——value 属性优先；无 value 属性回落 collapse 后文本。
var s = document.getElementById('s');
var o6a = s.options[0];
globalThis.__w6a = o6a.value + '/' + o6a.text;
var o6b = document.getElementById('o2');
globalThis.__w6b = o6b.value + '/' + o6b.text;
// W7：select.value 宿主路径不回退。
globalThis.__w7 = s.value;
"##,
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__w1a").unwrap().value,
        "text",
        "W1：首尾 ASCII whitespace 剥离"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1b").unwrap().value,
        "before after",
        "W1：内侧连续空白折叠单空格"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w1c").unwrap().value,
        "\u{a0}x\u{a0}",
        "W1：NBSP 保留不折叠"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2a").unwrap().value,
        "font",
        "W2：元素子递归收集"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2b").unwrap().value,
        "before after",
        "W2：HTML script 文本不计"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2c").unwrap().value,
        "before after",
        "W2：SVG script 文本不计"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2d").unwrap().value,
        "before script after",
        "W2：MathML script 递归计入"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2e").unwrap().value,
        "before script after",
        "W2：null ns script 递归计入"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2f").unwrap().value,
        "x",
        "W2：comment 子不计"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w2g").unwrap().value,
        "Some Text",
        "W2：嵌套层 script 逐层排除（span 内 script 文本不计）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3a").unwrap().value,
        "",
        "W3：label 属性空串存在即采纳（不回落文本）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3b").unwrap().value,
        "L",
        "W3：label 属性值优先于子文本"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3c").unwrap().value,
        "",
        "W3：NS 影子 label 不可见（无子回落空串）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3d").unwrap().value,
        "child",
        "W3：NS 影子 label 下回落 child text content"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w3e").unwrap().value,
        "child",
        "W3：无 label 属性回落 collapse 后子文本"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4a").unwrap().value,
        "",
        "W4：NS 影子 value 不可见"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4b").unwrap().value,
        "child",
        "W4：无 value 属性回落 collapse 后子文本"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4c").unwrap().value,
        "",
        "W4：value 属性空串存在即采纳"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w4d").unwrap().value,
        "V",
        "W4：value 属性优先于子文本"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5a").unwrap().value,
        "true/bar/true",
        "W5：旧文本子 node identity 更替且脱离树"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5b").unwrap().value,
        "baz",
        "W5：text setter 后 .text 返新值"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w5c").unwrap().value,
        "true/",
        "W5：text='' 移除全部子"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w6a").unwrap().value,
        "foo/bar",
        "W6：解析 option value 属性优先 + .text 口径"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w6b").unwrap().value,
        "txt/txt",
        "W6：解析 option 无 value 属性回落 collapse 后文本"
    );
    assert_eq!(
        sandbox.execute("globalThis.__w7").unwrap().value,
        "foo",
        "W7：select.value 宿主路径不回退"
    );
}
