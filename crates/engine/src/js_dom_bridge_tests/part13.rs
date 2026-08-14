// js_dom_bridge 测试模块拆分 part 13（R3028+，控制单文件 <2000 行，include! 入 js_dom_bridge_tests.rs）。
// 承接 part12 溢出：MutationObserver characterDataOldValue（R3028）+ innerHTML childList emission（R3029）。

/// 判断 `apply_mutations_to_html` 输出中某属性是否 present。序列化器（dom::serializer）对每个属性恒输出
/// ` name="value"`（空值 → `name=""`），故属性 present ⟺ 串含 ` name="`。比裸 `.contains("muted")` 更健壮
/// （避免 "muted"/"loop" 等在元素/文本他处的子串误判）。供 R3040 布尔 reflected 属性 apply 验证用。
fn bool_attr_present(html: &str, attr: &str) -> bool {
    html.contains(&format!(" {}=\"", attr))
}

#[test]
fn test_character_data_old_value_and_text_lw_r3028() {
    // R3028：characterDataOldValue（MO observe options 最后一档）+ sel-based 文本读 latest-wins。
    // ① characterDataOldValue:true → rec.oldValue = mutate 前旧文本（连续变更各取前值）；
    // ② 未请求 → oldValue 恒 null（spec）；③ textContent getter latest-wins 闭合 `textContent=` 后 stale 快照；
    // ④ option.text/option.label sel-based 同步 latest-wins（= textContent）。闭合 sel_text_override latent gap。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='a'>init</div><select id='s'><option id='o'>opt-init</option></select></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① characterDataOldValue：首次变更 oldValue=初值 'init'；二次变更 oldValue=前值 'first'。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             var mo = new MutationObserver(function(){});\
             mo.observe(a, { characterData: true, childList: true, subtree: true, characterDataOldValue: true });\
             a.textContent = 'first';\
             globalThis.__r1 = mo.takeRecords();\
             a.textContent = 'second';\
             globalThis.__r2 = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].type").unwrap().value,
        "childList",
        "R49：textContent= 发 childList（spec 替换子树；characterData 仅文本节点编辑发）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].addedNodes[0].data").unwrap().value,
        "first",
        "childList addedNodes[0]=新文本节点（data='first'）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2[0].addedNodes[0].data").unwrap().value,
        "second",
        "R49：二次 textContent= addedNodes[0].data='second'（同值 no-op 不发——异值替换可见）"
    );

    // ② 未请求 characterDataOldValue → oldValue 恒 null（spec），即使旧值存在。
    sandbox
        .execute(
            "mo.disconnect();\
             var mo2 = new MutationObserver(function(){});\
             mo2.observe(a, { characterData: true, subtree: true });\
             a.textContent = 'third';\
             globalThis.__r3 = mo2.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r3.length)").unwrap().value,
        "0",
        "R49：characterData-only（无 oldValue 请求）observer 对 textContent= 收 0 记录（childList 不投递）"
    );

    // ③ textContent getter latest-wins（R3028 stale 快照修复）：`textContent=` 后立即读反映新值。
    // 旧实现 getter 走纯快照 `__zw_get_text`，render apply 前快照仍为初值 → 返 stale 旧值。
    sandbox
        .execute(
            "a.textContent = 'after';\
             globalThis.__tc = a.textContent;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tc").unwrap().value,
        "after",
        "textContent getter latest-wins：textContent= 后读反映新值（闭合 stale 快照）"
    );

    // ④ option.text / option.label sel-based latest-wins（= textContent）。
    sandbox
        .execute(
            "var o = document.getElementById('o');\
             o.textContent = 'new-opt';\
             globalThis.__ot = o.text;\
             globalThis.__ol = o.label;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ot").unwrap().value,
        "new-opt",
        "option.text sel-based latest-wins（= textContent）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ol").unwrap().value,
        "new-opt",
        "option.label sel-based latest-wins 回落 text"
    );

    // ⑤ host 侧 sel_text_override 单元断言：同 selector 最近 SetText 胜出，无命中 None。
    // 不假设 selector 字符串格式——从记录的 text='after' 反推其 selector（仅 #a 写过 'after'）。
    let list = mutations.lock().unwrap();
    let a_sel = list
        .iter()
        .rev()
        .find_map(|m| match m {
            DomMutation::SetText { selector, text } if text == "after" => Some(selector.clone()),
            _ => None,
        })
        .expect("存在 text='after' 的 SetText 记录");
    assert_eq!(sel_text_override(&list, &a_sel), Some("after".to_string()));
    assert_eq!(sel_text_override(&list, "#nonexistent"), None);
    drop(list);
}

#[test]
fn test_inner_html_child_list_emission_r3029() {
    // R3029：innerHTML setter childList emission（闭合 R3028 已知限制④）。element.innerHTML 整体替换子树，
    // 旧不 emit childList 记录——observe(el,{childList:true}); el.innerHTML='...'; takeRecords() 收 0 记录，
    // 框架/库「innerHTML 触发重渲染」失效。本切片补 emit：type=childList，removedNodes=替换前旧子（snapshot 读，
    // _childNodeList 对 handle-only 无 sel 返 []），addedNodes 留空（parse-based 新子 snapshot apply 前不可同步
    // 枚举，documented 限制，同 outerHTML/insertAdjacentHTML）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='c'><span id='old1'>a</span><span id='old2'>b</span></div><div id='e'></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① observe(c,{childList:true}) + c.innerHTML → 1 childList 记录，removedNodes=2 旧 span，addedNodes=[]。
    sandbox
        .execute(
            "var c = document.getElementById('c');\
             var mo = new MutationObserver(function(){});\
             mo.observe(c, { childList: true });\
             c.innerHTML = '<b>new</b><i>x</i>';\
             globalThis.__r1 = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r1.length)").unwrap().value,
        "1",
        "innerHTML：c.innerHTML 变更 → 1 childList 记录"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].type").unwrap().value,
        "childList",
        "innerHTML 记录 type=childList"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].target.id").unwrap().value,
        "c",
        "innerHTML childList 记录 target=c"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r1[0].removedNodes.length)").unwrap().value,
        "2",
        "innerHTML removedNodes=替换前 2 旧子 span"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].removedNodes[0].tagName").unwrap().value,
        "SPAN",
        "innerHTML removedNodes[0]=旧 span 元素（tagName=SPAN）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r1[0].addedNodes.length)").unwrap().value,
        "2",
        "innerHTML addedNodes=2（R3031 parse-based 回填：B + I 顶层节点）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].addedNodes[0].tagName").unwrap().value,
        "B",
        "innerHTML addedNodes[0]=新 B 元素（introspection 可读）"
    );

    // ② 未观测 childList → innerHTML 不产记录（仅 attributes 观测被 childList 过滤掉）。
    sandbox
        .execute(
            "mo.disconnect();\
             var mo2 = new MutationObserver(function(){});\
             mo2.observe(c, { attributes: true });\
             c.innerHTML = '<u>again</u>';\
             globalThis.__r2 = mo2.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r2.length)").unwrap().value,
        "0",
        "未观测 childList → innerHTML 不产记录（仅 attributes 观测）"
    );

    // ③ innerHTML into empty element → removedNodes=[]（无旧子），仍 1 childList 记录。
    sandbox
        .execute(
            "var e = document.getElementById('e');\
             var mo3 = new MutationObserver(function(){});\
             mo3.observe(e, { childList: true });\
             e.innerHTML = '<p>fresh</p>';\
             globalThis.__r3 = mo3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r3.length)").unwrap().value,
        "1",
        "innerHTML into empty：仍 1 childList 记录"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r3[0].removedNodes.length)").unwrap().value,
        "0",
        "innerHTML into empty：removedNodes=0（无旧子）"
    );

    // ④ subtree：observe(body,{childList,subtree}) + c.innerHTML → 后代 childList 冒泡到 body observer。
    sandbox
        .execute(
            "var body = document.body;\
             var mo4 = new MutationObserver(function(){});\
             mo4.observe(body, { childList: true, subtree: true });\
             c.innerHTML = '<s>sub</s>';\
             globalThis.__r4 = mo4.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r4.length)").unwrap().value,
        "1",
        "subtree：后代 c.innerHTML 冒泡到 body observer → 1 记录"
    );
}

#[test]
fn form_entry_list_covers_basic_control_family() {
    // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#constructing-form-data-set
    let html = "<html><body>\
        <form id='main' action='/submit'>\
          <input name='text' value='A'>\
          <input type='checkbox' name='check' value='yes' checked>\
          <input type='checkbox' name='unchecked' value='no'>\
          <input type='radio' name='radio' value='one'>\
          <input type='radio' name='radio' value='two' checked>\
          <select name='pick'><option value='a'>A</option><option value='b' selected>B</option></select>\
          <textarea name='note'>hello</textarea>\
          <input name='foreign' value='skip' form='other'>\
          <button id='go' name='go' value='send'>Go</button>\
        </form>\
        <input name='external' value='outside' form='main'>\
        <form id='other'></form>\
        </body></html>";
    assert_eq!(
        form_get_submission_url(html, "#main", Some("#go"), "https://zero.test/form"),
        Some(
            "https://zero.test/submit?text=A&check=yes&radio=two&pick=b&note=hello&go=send&external=outside"
                .to_string()
        )
    );
}

#[test]
fn test_get_computed_style_dynamic_inline_r3030() {
    // R3030：getComputedStyle 动态样式正确性。旧实现读 stale HTML 快照——`el.style.X=...` push
    // SetStyle 后、render apply 前，快照仍是旧 style → gCS 返旧值。本切片把 inline style mutation
    // 子集顺序 apply 到 parsed doc 后再 cascade，latest-wins 语义同 render。
    // ① stale-fix：`style.color='red'` 后 gCS().color=red 计算值（旧返快照旧值 ''）；
    // ② keyword：`style.display='none'` 后 gCS().display='none'（旧返 UA 默认 block）；
    // ③ latest-wins：连续 color='red'→'blue' 取末值 blue；
    // ④ cssText 整体替换（SetAttr style）：`cssText='color:green'` 后 color=green；
    // ⑤ removeProperty（RemoveStyle）：移除后回落非 inline 值（不再反映已移除声明）；
    // ⑥ getPropertyValue(kebab) 与 camelCase 读一致；⑦ 多 selector 各自独立。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // #b 带 inline color:blue（既有 inline，验证 override 与既有 style 合并而非覆盖）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='a'>x</div><div id='b' style='color: blue'>y</div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① stale-fix + ② keyword：set color + display 后 gCS 反映新计算值。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             a.style.color = 'red';\
             a.style.display = 'none';\
             globalThis.__c = getComputedStyle(a).color;\
             globalThis.__d = getComputedStyle(a).display;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "rgb(255, 0, 0)",
        "stale-fix：el.style.color='red' 后 gCS().color=red 计算值（旧返 stale 旧值）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d)").unwrap().value,
        "none",
        "keyword：el.style.display='none' 后 gCS().display='none'（旧返 UA 默认 block）"
    );

    // ③ latest-wins：连续 color red→blue 取末值。
    sandbox
        .execute(
            "a.style.color = 'blue';\
             globalThis.__lw = getComputedStyle(a).color;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__lw)").unwrap().value,
        "rgb(0, 0, 255)",
        "latest-wins：连续 color red→blue 取末值 blue"
    );

    // ④ cssText 整体替换（SetAttr style 路径）：color 变 green。
    sandbox
        .execute(
            "a.style.cssText = 'color: green';\
             globalThis.__ct = getComputedStyle(a).color;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ct)").unwrap().value,
        "rgb(0, 128, 0)",
        "cssText 整体替换（SetAttr style）：color=green"
    );

    // ⑤ removeProperty（RemoveStyle）：移除 color 后回落非 inline 值（非 green）。
    sandbox
        .execute(
            "a.style.removeProperty('color');\
             globalThis.__rm = getComputedStyle(a).color;",
        )
        .unwrap();
    assert_ne!(
        sandbox.execute("String(globalThis.__rm)").unwrap().value,
        "rgb(0, 128, 0)",
        "removeProperty 后 color 不再反映已移除的 green 声明"
    );

    // ⑥ getPropertyValue(kebab) 与 camelCase 读一致（同 selector 命中缓存，仍 latest-wins）。
    sandbox
        .execute(
            "a.style.color = 'red';\
             globalThis.__pv = getComputedStyle(a).getPropertyValue('color');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pv)").unwrap().value,
        "rgb(255, 0, 0)",
        "getPropertyValue(kebab) 与 camelCase 读一致"
    );

    // ⑦ 多 selector 各自独立：#b 既有 inline color:blue，set display:none 不影响 #b 的 color。
    sandbox
        .execute(
            "var b = document.getElementById('b');\
             b.style.display = 'none';\
             globalThis.__bc = getComputedStyle(b).color;\
             globalThis.__bd = getComputedStyle(b).display;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__bc)").unwrap().value,
        "rgb(0, 0, 255)",
        "多 selector 独立：#b 既有 inline color:blue 保持（override 与既有 style 合并）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bd)").unwrap().value,
        "none",
        "多 selector 独立：#b set display:none 反映"
    );
}

#[test]
fn test_mo_parse_based_added_nodes_r3031() {
    // R3031：parse-based childList node-lists。innerHTML/outerHTML/insertAdjacentHTML 整体替换/插入时，
    // 新子经 host fragment 解析生成，shim 旧无同步枚举 → childList 记录 addedNodes 恒 []。本切片复用
    // _zwMBuildBodyTree（host __zw_parse_html_child_nodes 二次 parse）建 _zwMEl 代理树回填 addedNodes，
    // 统一三处。代理支持 nodeType/tagName/getAttribute/hasAttribute 等 introspection。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='c'><span>old</span></div><div id='d'><p id='e'>e</p></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① innerHTML setter：addedNodes 回填解析片段顶层节点（2：SPAN.x + B），removedNodes=旧子（1 span）。
    sandbox
        .execute(
            "var c = document.getElementById('c');\
             var mo = new MutationObserver(function(){});\
             mo.observe(c, { childList: true });\
             c.innerHTML = '<span class=\"x\">a</span><b>b</b>';\
             globalThis.__r = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r[0].addedNodes.length)").unwrap().value,
        "2",
        "innerHTML setter：addedNodes 回填 2 顶层节点（旧恒 0）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r[0].addedNodes[0].tagName").unwrap().value,
        "SPAN",
        "innerHTML addedNodes[0].tagName=SPAN（introspection 可读）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r[0].addedNodes[0].getAttribute('class')").unwrap().value,
        "x",
        "innerHTML addedNodes[0].getAttribute('class')=x（属性可读）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r[0].addedNodes[1].tagName").unwrap().value,
        "B",
        "innerHTML addedNodes[1].tagName=B"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r[0].removedNodes.length)").unwrap().value,
        "1",
        "innerHTML removedNodes=1 旧子（R3029 既保持）"
    );

    // ② insertAdjacentHTML（beforeend）：addedNodes 回填 1 顶层节点（I）。
    sandbox
        .execute(
            "mo.disconnect();\
             var mo2 = new MutationObserver(function(){});\
             mo2.observe(c, { childList: true });\
             c.insertAdjacentHTML('beforeend', '<i>y</i>');\
             globalThis.__r2 = mo2.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r2[0].addedNodes.length)").unwrap().value,
        "1",
        "insertAdjacentHTML：addedNodes 回填 1 节点（旧恒 0）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2[0].addedNodes[0].tagName").unwrap().value,
        "I",
        "insertAdjacentHTML addedNodes[0].tagName=I"
    );

    // ③ outerHTML setter：addedNodes 回填解析片段顶层节点（target=元素 sel pragmatic 近似）。
    sandbox
        .execute(
            "var e = document.getElementById('e');\
             var mo3 = new MutationObserver(function(){});\
             mo3.observe(e, { childList: true });\
             e.outerHTML = '<section class=\"r\">new</section>';\
             globalThis.__r3 = mo3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r3[0].addedNodes.length)").unwrap().value,
        "1",
        "outerHTML setter：addedNodes 回填 1 节点（旧恒 0）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r3[0].addedNodes[0].tagName").unwrap().value,
        "SECTION",
        "outerHTML addedNodes[0].tagName=SECTION"
    );

    // ④ 空片段 / 纯文本片段：不抛错，addedNodes 反映实际顶层节点数（纯文本→0 element，文本节点不计 element
    //    但 childList spec addedNodes 含文本节点；此处验证不抛 + 数量一致）。
    sandbox
        .execute(
            "mo3.disconnect();\
             var d = document.getElementById('d');\
             var mo4 = new MutationObserver(function(){});\
             mo4.observe(d, { childList: true });\
             d.innerHTML = 'plain text only';\
             globalThis.__r4 = mo4.takeRecords();",
        )
        .unwrap();
    // 纯文本片段：顶层 1 文本节点（addedNodes 含文本节点）。
    assert_eq!(
        sandbox.execute("String(globalThis.__r4[0].addedNodes.length)").unwrap().value,
        "1",
        "纯文本 innerHTML：addedNodes 回填 1 文本节点"
    );
}

#[test]
fn test_classlist_domtokenlist_full_r3032() {
    // R3032：classList 完整 DOMTokenList。旧实现仅 add/remove/toggle/contains，缺 toggle(force)/replace/
    // item/length/indexed/forEach/toString(value)/variadic/Symbol.iterator。本切片补全。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d' class='a b'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① length + indexed 访问 + item。
    sandbox
        .execute(
            "var cl = document.getElementById('d').classList;\
             globalThis.__len = cl.length;\
             globalThis.__i0 = cl[0]; globalThis.__i1 = cl[1];\
             globalThis.__item0 = cl.item(0); globalThis.__item5 = cl.item(5);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "2", "classList.length=2");
    assert_eq!(sandbox.execute("globalThis.__i0").unwrap().value, "a", "classList[0]=a（indexed 访问）");
    assert_eq!(sandbox.execute("globalThis.__i1").unwrap().value, "b", "classList[1]=b");
    assert_eq!(sandbox.execute("globalThis.__item0").unwrap().value, "a", "classList.item(0)=a");
    assert_eq!(
        sandbox.execute("String(globalThis.__item5)").unwrap().value,
        "null",
        "classList.item(越界)=null"
    );

    // ② forEach + for...of（Symbol.iterator）迭代。
    sandbox
        .execute(
            "globalThis.__fe = []; cl.forEach(function(t){ globalThis.__fe.push(t); });\
             globalThis.__fo = []; for (var x of cl) globalThis.__fo.push(x);\
             globalThis.__feJ = String(globalThis.__fe); globalThis.__foJ = String(globalThis.__fo);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__feJ").unwrap().value, "a,b", "forEach 迭代 a,b");
    assert_eq!(sandbox.execute("globalThis.__foJ").unwrap().value, "a,b", "for...of 迭代 a,b");

    // ③ toggle(token, force)：force=false 移除（返 false）、force=true 加回（返 true）。
    sandbox
        .execute(
            "globalThis.__tf = cl.toggle('a', false);\
             globalThis.__tt = cl.toggle('a', true);\
             globalThis.__ca = cl.contains('a');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__tf").unwrap().value, "false", "toggle('a',false) 移除返 false");
    assert_eq!(sandbox.execute("globalThis.__tt").unwrap().value, "true", "toggle('a',true) 加回返 true");
    assert_eq!(sandbox.execute("globalThis.__ca").unwrap().value, "true", "contains('a') 反映 force=true 后存在");

    // ④ replace：③ 后 class 为 'b a'（toggle false 移 a、true 末加 a）；replace('a','c')=true → 'b c'，
    //    replace('z','x')=false（不存在）。
    sandbox
        .execute(
            "globalThis.__ra = cl.replace('a', 'c');\
             globalThis.__val1 = cl.value;\
             globalThis.__rz = cl.replace('z', 'x');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ra").unwrap().value, "true", "replace('a','c')=true");
    assert_eq!(sandbox.execute("globalThis.__val1").unwrap().value, "b c", "replace('a','c') 后 value='b c'（a 在 ③ 末位）");
    assert_eq!(sandbox.execute("globalThis.__rz").unwrap().value, "false", "replace 不存在 token=false");

    // ⑤ variadic add/remove。
    sandbox
        .execute(
            "cl.add('x', 'y', 'z');\
             globalThis.__vlen = cl.length;\
             cl.remove('x', 'y');\
             globalThis.__vlen2 = cl.length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__vlen)").unwrap().value, "5", "variadic add 3 → length 5 (b c x y z)");
    assert_eq!(sandbox.execute("String(globalThis.__vlen2)").unwrap().value, "3", "variadic remove 2 → length 3");

    // ⑥ toString/value 反映当前 class 串。
    sandbox
        .execute("globalThis.__ts = String(cl); globalThis.__val2 = cl.value;", )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ts").unwrap().value, "b c z", "toString()='b c z'");
    assert_eq!(sandbox.execute("globalThis.__val2").unwrap().value, "b c z", "value='b c z'");

    // ⑦ spec token 校验：空串/含空白 token → 抛（不静默）。
    sandbox
        .execute(
            "globalThis.__e1 = false; globalThis.__e2 = false;\
             try { cl.add(''); } catch (_e) { globalThis.__e1 = true; }\
             try { cl.add('has space'); } catch (_e) { globalThis.__e2 = true; }",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e1)").unwrap().value, "true", "add('') 抛（spec 空串 token）");
    assert_eq!(sandbox.execute("String(globalThis.__e2)").unwrap().value, "true", "add('has space') 抛（spec 含空白 token）");
}

#[test]
fn test_htmlcollection_nodelist_item_nameditem_r3033() {
    // R3033：HTMLCollection/NodeList 集合 API。getElementsBy*/querySelectorAll/getElementsByName 旧返纯数组，
    // 缺 spec 的 .item(i)（全部缺）+ HTMLCollection 的 .namedItem(name)（getElementsBy* 缺）。本切片补全。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='c'><p id='p1' name='para'>a</p><p id='p2'>b</p><span class='x'>s</span></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① getElementsByTagName('p') → HTMLCollection：length + item + indexed + namedItem(id) + namedItem(name)。
    sandbox
        .execute(
            "var ps = document.getElementsByTagName('p');\
             globalThis.__len = ps.length;\
             globalThis.__item0 = ps.item(0).id;\
             globalThis.__item1 = ps.item(1).id;\
             globalThis.__item5 = String(ps.item(5));\
             globalThis.__idx0 = ps[0].id;\
             globalThis.__ni_id = ps.namedItem('p1').id;\
             globalThis.__ni_name = ps.namedItem('para').id;\
             globalThis.__ni_none = String(ps.namedItem('nope'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "2", "getElementsByTagName length=2");
    assert_eq!(sandbox.execute("globalThis.__item0").unwrap().value, "p1", "item(0).id=p1");
    assert_eq!(sandbox.execute("globalThis.__item1").unwrap().value, "p2", "item(1).id=p2");
    assert_eq!(sandbox.execute("globalThis.__item5").unwrap().value, "null", "item(越界)=null");
    assert_eq!(sandbox.execute("globalThis.__idx0").unwrap().value, "p1", "indexed [0].id=p1（数组访问仍工作）");
    assert_eq!(sandbox.execute("globalThis.__ni_id").unwrap().value, "p1", "namedItem('p1') 按 id 匹配");
    assert_eq!(sandbox.execute("globalThis.__ni_name").unwrap().value, "p1", "namedItem('para') 按 name 匹配");
    assert_eq!(sandbox.execute("globalThis.__ni_none").unwrap().value, "null", "namedItem(不匹配)=null");

    // ② R50：HTMLCollection 为 spec legacy platform object——**无** forEach/values/entries
    //（WPT HTMLCollection-iterator 断言不存在；旧 Array 承载泄漏），for-of（@@iterator）替代
    //+ for...in 仅 indexed/named（0,1——item/namedItem 在原型不可枚举）。
    sandbox
        .execute(
            "globalThis.__fe = [];\
             for (var _e of ps) globalThis.__fe.push(_e.id);\
             globalThis.__feJ = String(globalThis.__fe);\
             globalThis.__hasFE = ('forEach' in ps);\
             var keys = []; for (var k in ps) keys.push(k); globalThis.__inKeys = String(keys);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__feJ").unwrap().value, "p1,p2", "for-of 迭代产出（p1,p2）");
    assert_eq!(
        sandbox.execute("String(globalThis.__hasFE)").unwrap().value,
        "false",
        "HTMLCollection 无 forEach（R50 spec 语义，旧 Array 泄漏已移除）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__inKeys").unwrap().value,
        "0,1",
        "for...in 仅 0,1（item/namedItem/toString/constructor 非 enumerable 不泄漏；length 在 Proxy target 之外）"
    );

    // ③ getElementsByClassName('x') → HTMLCollection：item + namedItem。
    sandbox
        .execute(
            "var xs = document.getElementsByClassName('x');\
             globalThis.__xlen = xs.length;\
             globalThis.__xtag = xs.item(0).tagName;\
             globalThis.__xhas = (typeof xs.namedItem === 'function');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__xlen)").unwrap().value, "1", "getElementsByClassName length=1");
    assert_eq!(sandbox.execute("globalThis.__xtag").unwrap().value, "SPAN", "getElementsByClassName item(0).tagName=SPAN");
    assert_eq!(sandbox.execute("String(globalThis.__xhas)").unwrap().value, "true", "HTMLCollection 有 namedItem 方法");

    // ④ querySelectorAll('p') → NodeList：有 item，无 namedItem（spec NodeList 无 namedItem）。
    sandbox
        .execute(
            "var qs = document.querySelectorAll('p');\
             globalThis.__qitem = qs.item(0).id;\
             globalThis.__qhasItem = (typeof qs.item === 'function');\
             globalThis.__qhasNamed = (typeof qs.namedItem === 'function');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__qitem").unwrap().value, "p1", "querySelectorAll item(0).id=p1");
    assert_eq!(sandbox.execute("String(globalThis.__qhasItem)").unwrap().value, "true", "NodeList 有 item 方法");
    assert_eq!(sandbox.execute("String(globalThis.__qhasNamed)").unwrap().value, "false", "NodeList 无 namedItem（spec）");

    // ⑤ getElementsByName('para') → NodeList：item。
    sandbox
        .execute(
            "var ns = document.getElementsByName('para');\
             globalThis.__nlen = ns.length;\
             globalThis.__nitem = ns.item(0).id;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__nlen)").unwrap().value, "1", "getElementsByName length=1");
    assert_eq!(sandbox.execute("globalThis.__nitem").unwrap().value, "p1", "getElementsByName item(0).id=p1");

    // ⑥ 元素子树作用域：getElementById('c').getElementsByTagName('p') → HTMLCollection length=2 + item。
    sandbox
        .execute(
            "var cps = document.getElementById('c').getElementsByTagName('p');\
             globalThis.__clen = cps.length;\
             globalThis.__citem = cps.item(0).id;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__clen)").unwrap().value, "2", "元素子树 getElementsByTagName length=2");
    assert_eq!(sandbox.execute("globalThis.__citem").unwrap().value, "p1", "元素子树 item(0).id=p1");
}

#[test]
fn test_text_node_data_setter_persist_char_r3034() {
    // R3034：text/comment 节点 .data/.nodeValue IDL setter。旧实现落入 set trap 末尾 generic fallthrough
    // → 误设 'data' 内容属性 + attributes MO 记录（类型错），且文本内容未持久化（读回返旧值）。本切片修。
    // ① persistence：t.data='bye' → 读回 'bye'（旧 bug 读回 'hi'）；② nodeValue setter 同样持久化；
    // ③ characterData emission：observe(t,{characterData}) + t.data= → 1 characterData 记录（旧发 attributes）；
    // ④ comment 节点 data setter 持久化。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='c'>old</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① ② data/nodeValue setter 持久化（旧 bug：读回返旧值）。
    sandbox
        .execute(
            "var c = document.getElementById('c');\
             var t = document.createTextNode('hi');\
             c.appendChild(t);\
             t.data = 'bye';\
             globalThis.__d1 = t.data;\
             globalThis.__nv = t.nodeValue;\
             t.nodeValue = 'again';\
             globalThis.__d2 = t.data;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__d1").unwrap().value, "bye", "t.data='bye' 持久化（旧 bug 读回 'hi'）");
    assert_eq!(sandbox.execute("globalThis.__nv").unwrap().value, "bye", "nodeValue 反映 data 设置");
    assert_eq!(sandbox.execute("globalThis.__d2").unwrap().value, "again", "t.nodeValue='again' 持久化（与 data 等价）");

    // ③ characterData emission：observe(t,{characterData}) + t.data= → 1 characterData 记录（旧发 attributes）。
    sandbox
        .execute(
            "var mo = new MutationObserver(function(){});\
             mo.observe(t, { characterData: true });\
             t.data = 'final';\
             globalThis.__r = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r.length)").unwrap().value,
        "1",
        "observe(t,{{characterData}}) + t.data= → 1 记录"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r[0].type").unwrap().value,
        "characterData",
        "记录 type=characterData（旧 bug 发 attributes）"
    );

    // ④ comment 节点 data setter 持久化（同 text 节点路径）。
    sandbox
        .execute(
            "var cm = document.createComment('c');\
             c.appendChild(cm);\
             cm.data = 'changed';\
             globalThis.__cm = cm.data;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cm").unwrap().value,
        "changed",
        "comment.data='changed' 持久化（同 text 节点路径）"
    );

    // ⑤ data setter 不再误发 attributes 记录（旧 generic fallthrough 发 attributes）。
    sandbox
        .execute(
            "var mo2 = new MutationObserver(function(){});\
             mo2.observe(t, { attributes: true, characterData: true });\
             t.data = 'noattr';\
             globalThis.__r2 = mo2.takeRecords();",
        )
        .unwrap();
    // 只应有 characterData 记录，无 attributes 记录（旧 bug：'data' 被当属性发 attributes）。
    let r2_types = sandbox
        .execute(
            "var ts = []; for (var i = 0; i < globalThis.__r2.length; i++) ts.push(globalThis.__r2[i].type); String(ts);",
        )
        .unwrap()
        .value;
    assert!(
        !r2_types.contains("attributes"),
        "data setter 不发 attributes 记录（旧 bug 发 attributes，got {r2_types:?}）"
    );
    assert!(
        r2_types.contains("characterData"),
        "data setter 发 characterData 记录（got {r2_types:?}）"
    );
}

#[test]
fn test_document_title_setter_writeback_r3035() {
    // R3035：document.title setter 写回 host <title>。旧 setter 仅更新 in-JS 缓存，不写回（R2815 限制①）
    // → render 不反映新 title、apply_mutations 后 <title> 文本不变。本切片经 __zw_set_text('title', v) 写回。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>Old Title</title></head><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① getter 读 <title> 文本（空白折叠）。
    sandbox.execute("globalThis.__t0 = document.title;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t0").unwrap().value,
        "Old Title",
        "document.title getter 读 <title> 文本"
    );

    // ② setter 写缓存 + 写回 host；apply_mutations 后 <title> 文本更新。
    sandbox
        .execute("document.title = 'New Title'; globalThis.__t1 = document.title;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t1").unwrap().value,
        "New Title",
        "setter 后 getter 读回新值（缓存）"
    );
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        out.contains("<title>New Title</title>"),
        "setter 写回 host <title>（apply_mutations 后含 <title>New Title</title>）\n{out}"
    );

    // ③ 无 <title> 时 setter no-op（不 panic、不创 <title>）。
    let mut sandbox2 = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    let m2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let h2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><p>no title</p></body></html>".to_string()));
    let pu2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox2, &m2, &h2, &pu2, &canvas_registry);
    sandbox2.execute("document.title = 'X'; globalThis.__nt = document.title;").unwrap();
    assert_eq!(
        sandbox2.execute("globalThis.__nt").unwrap().value,
        "X",
        "无 <title> 时 setter 仍更新缓存（getter 读回）"
    );
}

#[test]
fn test_element_sheet_cssstylesheet_r3036() {
    // R3036：element.sheet CSSStyleSheet 入口。<style>/<link rel=stylesheet> 的 .sheet 应返 CSSStyleSheet
    //（CSS-in-JS 库 + 样式表操作经 .sheet.cssRules/insertRule 读改规则）。当前 get trap 对 'sheet' 返 undefined。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head>\
         <style id='s'>body { color: red; }</style>\
         <link id='ls' rel='stylesheet' href='x.css'>\
         <link id='li' rel='icon' href='i.png'>\
         </head><body><div id='d'></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① <style>.sheet → CSSStyleSheet：cssRules 读规则（selectorText='body'）。
    sandbox
        .execute(
            "var sheet = document.getElementById('s').sheet;\
             globalThis.__hasSheet = (sheet != null);\
             globalThis.__rules = sheet ? sheet.cssRules.length : -1;\
             globalThis.__sel = sheet && sheet.cssRules[0] ? sheet.cssRules[0].selectorText : '';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hasSheet)").unwrap().value,
        "true",
        "<style>.sheet 返 CSSStyleSheet（非 null/undefined）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rules)").unwrap().value,
        "1",
        "<style>.sheet.cssRules.length=1（反映 style 内容）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__sel").unwrap().value,
        "body",
        "<style>.sheet.cssRules[0].selectorText='body'"
    );

    // ② <link rel=stylesheet>.sheet → CSSStyleSheet（非 null）。
    sandbox
        .execute("globalThis.__ls = (document.getElementById('ls').sheet != null);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ls)").unwrap().value,
        "true",
        "<link rel=stylesheet>.sheet 返 CSSStyleSheet"
    );

    // ③ <link rel=icon>.sheet → null（非 stylesheet）。
    sandbox
        .execute("globalThis.__li = String(document.getElementById('li').sheet);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__li").unwrap().value,
        "null",
        "<link rel=icon>.sheet=null（非 stylesheet）"
    );

    // ④ 非 style/link 元素 .sheet → undefined（generic Element 无 .sheet）。
    sandbox
        .execute("globalThis.__ds = String(document.getElementById('d').sheet);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ds").unwrap().value,
        "undefined",
        "<div>.sheet=undefined（非 style/link，fall through）"
    );

    // ⑤ .sheet.insertRule 写回 + cssRules 反映。
    sandbox
        .execute(
            "sheet.insertRule('p { margin: 0; }', 1);\
             globalThis.__rules2 = sheet.cssRules.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rules2)").unwrap().value,
        "2",
        ".sheet.insertRule 后 cssRules.length=2"
    );
}

#[test]
fn test_reflected_string_attr_reads_r3037() {
    // R3037：reflected string 内容属性读。type/name/placeholder/min/max/step/pattern/alt/src/rel/target 等
    // 旧 get trap 未拦 → 读返 undefined（写正常）。表单校验库读 input.min/max/pattern/type、analytics 读
    // src/name 等失效。本切片补 get trap reflected string 查表读。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <input id='i' type='text' value='v0' placeholder='ph' min='3' max='10' step='1' pattern='[0-9]+' name='nm'>\
         <a id='a' href='http://x/' rel='noopener' target='_blank'>l</a>\
         <img id='im' alt='alttext' src='s.png' loading='lazy'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① input reflected string 读（旧恒 undefined）。
    sandbox
        .execute(
            "var i = document.getElementById('i');\
             globalThis.__type = i.type;\
             globalThis.__ph = i.placeholder;\
             globalThis.__min = i.min;\
             globalThis.__max = i.max;\
             globalThis.__step = i.step;\
             globalThis.__pat = i.pattern;\
             globalThis.__name = i.name;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__type").unwrap().value, "text", "input.type=text");
    assert_eq!(sandbox.execute("globalThis.__ph").unwrap().value, "ph", "input.placeholder=ph");
    assert_eq!(sandbox.execute("globalThis.__min").unwrap().value, "3", "input.min=3");
    assert_eq!(sandbox.execute("globalThis.__max").unwrap().value, "10", "input.max=10");
    assert_eq!(sandbox.execute("globalThis.__step").unwrap().value, "1", "input.step=1");
    assert_eq!(sandbox.execute("globalThis.__pat").unwrap().value, "[0-9]+", "input.pattern=[0-9]+");
    assert_eq!(sandbox.execute("globalThis.__name").unwrap().value, "nm", "input.name=nm");

    // ② img reflected string 读（alt/src/loading）。
    sandbox
        .execute(
            "var im = document.getElementById('im');\
             globalThis.__alt = im.alt;\
             globalThis.__src = im.src;\
             globalThis.__load = im.loading;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__alt").unwrap().value, "alttext", "img.alt=alttext");
    assert_eq!(sandbox.execute("globalThis.__src").unwrap().value, "s.png", "img.src=s.png（raw 属性值）");
    assert_eq!(sandbox.execute("globalThis.__load").unwrap().value, "lazy", "img.loading=lazy");

    // ③ a reflected string 读（rel/target）。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             globalThis.__rel = a.rel;\
             globalThis.__tgt = a.target;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__rel").unwrap().value, "noopener", "a.rel=noopener");
    assert_eq!(sandbox.execute("globalThis.__tgt").unwrap().value, "_blank", "a.target=_blank");

    // ④ 缺省属性返 ''（spec reflected string 缺省空串，非 undefined）。
    sandbox.execute("globalThis.__accept = document.getElementById('i').accept;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__accept").unwrap().value, "", "input.accept 缺省=''（spec 空串）");

    // ⑤ set-then-get round-trip：IDL setter 写 → 读反映（set trap fallthrough 写属性 + get latest-wins 读）。
    sandbox
        .execute(
            "i.placeholder = 'newph';\
             globalThis.__rpg = i.placeholder;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__rpg").unwrap().value,
        "newph",
        "input.placeholder='newph' 后读回 'newph'（set→get round-trip）"
    );

    // ⑥ setAttribute→IDL get：setAttribute('min','5') → input.min='5'。
    sandbox
        .execute(
            "i.setAttribute('min', '5');\
             globalThis.__ming = i.min;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ming").unwrap().value,
        "5",
        "setAttribute('min','5') 后 input.min='5'（attr↔IDL 一致）"
    );

    // ⑦ camelCase 映射：input.formMethod / a.crossOrigin 反射 formmethod / crossorigin（缺省 ''）。
    sandbox.execute("globalThis.__fm = document.getElementById('i').formMethod;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fm").unwrap().value,
        "",
        "input.formMethod 缺省=''（camelCase→attr 映射，无 formmethod 属性）"
    );
}

#[test]
fn test_reflected_uint_bool_reads_r3038() {
    // R3038：reflected 数值型 + 布尔型属性读（R3037 follow-up）。colSpan/rowSpan/maxLength/minLength（number）、
    // required/readOnly/multiple（boolean）旧读返 undefined。本切片补 number/boolean 语义读。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <input id='i' maxlength='5' minlength='2' required>\
         <table><tr><td id='t' colspan='3' rowspan='4'>c</td><td id='t2'>nc</td></tr></table>\
         <textarea id='ta' readonly></textarea>\
         <select id='s' multiple><option>a</option></select>\
         <input id='i2' type='text'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 数值型 reflected 读（number 语义，旧恒 undefined）。
    sandbox
        .execute(
            "globalThis.__ml = document.getElementById('i').maxLength;\
             globalThis.__mlt = document.getElementById('i').minLength;\
             globalThis.__cs = document.getElementById('t').colSpan;\
             globalThis.__rs = document.getElementById('t').rowSpan;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ml").unwrap().value, "5", "input.maxLength=5（number）");
    assert_eq!(sandbox.execute("globalThis.__mlt").unwrap().value, "2", "input.minLength=2（number）");
    assert_eq!(sandbox.execute("globalThis.__cs").unwrap().value, "3", "td.colSpan=3（number）");
    assert_eq!(sandbox.execute("globalThis.__rs").unwrap().value, "4", "td.rowSpan=4（number）");

    // ② 数值型缺省 default：maxLength/minLength 缺省 -1，colSpan/rowSpan 缺省 1。
    sandbox
        .execute(
            "globalThis.__ml2 = document.getElementById('i2').maxLength;\
             globalThis.__mlt2 = document.getElementById('i2').minLength;\
             globalThis.__cs2 = String(document.getElementById('t2').colSpan);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ml2").unwrap().value, "-1", "maxLength 缺省=-1（spec 不限制）");
    assert_eq!(sandbox.execute("globalThis.__mlt2").unwrap().value, "-1", "minLength 缺省=-1");
    assert_eq!(sandbox.execute("globalThis.__cs2").unwrap().value, "1", "td.colSpan 缺省=1（spec default）");

    // ③ 布尔型 reflected 读（presence-based boolean，旧恒 undefined）。
    sandbox
        .execute(
            "globalThis.__req = String(document.getElementById('i').required);\
             globalThis.__ro = String(document.getElementById('ta').readOnly);\
             globalThis.__mu = String(document.getElementById('s').multiple);\
             globalThis.__req2 = String(document.getElementById('i2').required);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__req").unwrap().value, "true", "input.required=true（presence）");
    assert_eq!(sandbox.execute("globalThis.__ro").unwrap().value, "true", "textarea.readOnly=true（presence）");
    assert_eq!(sandbox.execute("globalThis.__mu").unwrap().value, "true", "select.multiple=true（presence）");
    assert_eq!(sandbox.execute("globalThis.__req2").unwrap().value, "false", "input.required 缺省=false");

    // ④ setAttribute → 数值型 get 一致 + setAttribute 移除 → 布尔 false。
    sandbox
        .execute(
            "document.getElementById('i2').setAttribute('maxlength', '8');\
             globalThis.__ml3 = document.getElementById('i2').maxLength;\
             document.getElementById('i').removeAttribute('required');\
             globalThis.__req3 = String(document.getElementById('i').required);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ml3").unwrap().value, "8", "setAttribute('maxlength','8') → maxLength=8");
    assert_eq!(
        sandbox.execute("globalThis.__req3").unwrap().value,
        "false",
        "removeAttribute('required') → required=false（presence 消失）"
    );
}

#[test]
fn test_boolean_reflected_set_false_r3039() {
    // R3039：布尔 reflected set-false bug。required/readOnly/multiple `=false` 旧经 generic fallthrough 写
    // attr="false"（present）→ 读返 true（应 false）。本切片 set trap 专用分支 falsy→removeAttribute，闭合往返。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <input id='i' required>\
         <textarea id='ta'></textarea>\
         <select id='s'><option>a</option></select>\
         <input id='i2' type='text'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① required=false 真移除（旧 set-false bug：写 attr="false" 仍 present → 读 true）。
    sandbox
        .execute(
            "globalThis.__r0 = String(document.getElementById('i').required);\
             document.getElementById('i').required = false;\
             globalThis.__r1 = String(document.getElementById('i').required);\
             globalThis.__r1Has = String(document.getElementById('i').hasAttribute('required'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__r0").unwrap().value, "true", "初始 required=true");
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "false",
        "required=false 后读 false（旧 set-false bug 读 true）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1Has").unwrap().value,
        "false",
        "required=false 后 hasAttribute=false（真移除，非 attr='false' 残留）"
    );

    // ② required=true 设回（presence），apply_mutations 后属性反映。
    sandbox
        .execute("document.getElementById('i').required = true; globalThis.__r2 = String(document.getElementById('i').required);")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__r2").unwrap().value, "true", "required=true 后读 true");
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        out.contains("required="),
        "required=true apply 后属性 present\n{out}"
    );

    // ③ readOnly/multiple falsy→remove（apply_mutations 验证属性移除）。
    sandbox
        .execute(
            "document.getElementById('ta').readOnly = true;\
             document.getElementById('ta').readOnly = false;\
             document.getElementById('s').multiple = true;\
             document.getElementById('s').multiple = false;",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(
        !out2.contains("readonly") && !out2.contains("multiple"),
        "readOnly/multiple false 后属性移除（apply_mutations 不含 readonly/multiple）\n{out2}"
    );

    // ④ 多次 toggle（true→false→true）往返正确（latest-wins remove/set 序列）。
    sandbox
        .execute(
            "var i2 = document.getElementById('i2');\
             i2.required = true; i2.required = false; i2.required = true;\
             globalThis.__toggle = String(i2.required);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__toggle").unwrap().value,
        "true",
        "required true→false→true 末态 true（latest-wins 往返）"
    );
}

#[test]
fn test_reflected_bool_attrs_expanded_r3040() {
    // R3040：扩 _REFLECTED_BOOL 覆盖更多纯布尔 reflected 属性（R3038 读 + R3039 set-false 修复机制的统一延伸）。
    // 覆盖 noValidate/async/defer/nomodule/autoplay/controls/loop/muted/playsInline/reversed/isMap/itemScope——
    // 旧读恒 undefined（get trap 未拦）、set `=false` 经 generic fallthrough 写 attr="false"（set-false bug）。
    // 本切片补 presence 读（boolean）+ falsy→removeAttribute（set），闭合 set→get 全往返。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <form id='f' novalidate><input></form>\
         <script id='sc' async defer></script>\
         <video id='v' autoplay controls muted loop playsinline></video>\
         <ol id='ol' reversed><li>a</li></ol>\
         <img id='img' ismap>\
         <div id='d' itemscope></div>\
         <script id='sc2'></script>\
         <video id='v2'></video>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① presence 读（boolean true，旧恒 undefined）——camelCase 映射（noValidate/playsInline/isMap/itemScope）。
    sandbox
        .execute(
            "globalThis.__nv = String(document.getElementById('f').noValidate);\
             globalThis.__as = String(document.getElementById('sc').async);\
             globalThis.__df = String(document.getElementById('sc').defer);\
             globalThis.__ap = String(document.getElementById('v').autoplay);\
             globalThis.__ct = String(document.getElementById('v').controls);\
             globalThis.__lo = String(document.getElementById('v').loop);\
             globalThis.__mu = String(document.getElementById('v').muted);\
             globalThis.__pi = String(document.getElementById('v').playsInline);\
             globalThis.__rv = String(document.getElementById('ol').reversed);\
             globalThis.__im = String(document.getElementById('img').isMap);\
             globalThis.__is = String(document.getElementById('d').itemScope);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__nv").unwrap().value, "true", "form.noValidate=true（presence）");
    assert_eq!(sandbox.execute("globalThis.__as").unwrap().value, "true", "script.async=true");
    assert_eq!(sandbox.execute("globalThis.__df").unwrap().value, "true", "script.defer=true");
    assert_eq!(sandbox.execute("globalThis.__ap").unwrap().value, "true", "video.autoplay=true");
    assert_eq!(sandbox.execute("globalThis.__ct").unwrap().value, "true", "video.controls=true");
    assert_eq!(sandbox.execute("globalThis.__lo").unwrap().value, "true", "video.loop=true");
    assert_eq!(sandbox.execute("globalThis.__mu").unwrap().value, "true", "video.muted=true");
    assert_eq!(sandbox.execute("globalThis.__pi").unwrap().value, "true", "video.playsInline=true（playsinline 映射）");
    assert_eq!(sandbox.execute("globalThis.__rv").unwrap().value, "true", "ol.reversed=true");
    assert_eq!(sandbox.execute("globalThis.__im").unwrap().value, "true", "img.isMap=true（ismap 映射）");
    assert_eq!(sandbox.execute("globalThis.__is").unwrap().value, "true", "div.itemScope=true（itemscope 映射）");

    // ② 缺省读（boolean false，旧恒 undefined）。
    sandbox
        .execute(
            "globalThis.__nv2 = String(document.getElementById('f').noValidate);\
             globalThis.__as2 = String(document.getElementById('sc2').async);\
             globalThis.__as2d = String(document.getElementById('sc2').defer);\
             globalThis.__nm2 = String(document.getElementById('sc2').nomodule);\
             globalThis.__ap2 = String(document.getElementById('v2').autoplay);\
             globalThis.__mu2 = String(document.getElementById('v2').muted);\
             globalThis.__im2 = String(document.getElementById('img').itemScope);",
        )
        .unwrap();
    // 注：form#f 已有 novalidate（① 读 true），此断言验同一 form 缺省分支用 sc2/v2 等无属性元素。
    assert_eq!(sandbox.execute("globalThis.__nv2").unwrap().value, "true", "form.noValidate 仍 true（已设属性）");
    assert_eq!(sandbox.execute("globalThis.__as2").unwrap().value, "false", "script.async 缺省=false");
    assert_eq!(sandbox.execute("globalThis.__as2d").unwrap().value, "false", "script.defer 缺省=false");
    assert_eq!(sandbox.execute("globalThis.__nm2").unwrap().value, "false", "script.nomodule 缺省=false");
    assert_eq!(sandbox.execute("globalThis.__ap2").unwrap().value, "false", "video.autoplay 缺省=false");
    assert_eq!(sandbox.execute("globalThis.__mu2").unwrap().value, "false", "video.muted 缺省=false");
    assert_eq!(sandbox.execute("globalThis.__im2").unwrap().value, "false", "img.itemScope 缺省=false（非微数据元素）");

    // ③ set-false bug 修复：`el.async=false` 真移除（旧 generic fallthrough 写 async="false" 仍 present → 读 true）。
    sandbox
        .execute(
            "var sc = document.getElementById('sc');\
             sc.async = false;\
             globalThis.__asF = String(sc.async);\
             globalThis.__asFHas = String(sc.hasAttribute('async'));\
             sc.defer = false; sc.nomodule = true;\
             globalThis.__dfF = String(sc.defer);\
             globalThis.__nmT = String(sc.nomodule);\
             globalThis.__nmTHas = String(sc.hasAttribute('nomodule'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__asF").unwrap().value, "false", "script.async=false 后读 false（旧 set-false bug 读 true）");
    assert_eq!(sandbox.execute("globalThis.__asFHas").unwrap().value, "false", "async=false 后 hasAttribute=false（真移除）");
    assert_eq!(sandbox.execute("globalThis.__dfF").unwrap().value, "false", "script.defer=false 后读 false");
    assert_eq!(sandbox.execute("globalThis.__nmT").unwrap().value, "true", "script.nomodule=true 后读 true（presence）");
    assert_eq!(sandbox.execute("globalThis.__nmTHas").unwrap().value, "true", "nomodule=true 后 hasAttribute=true");

    // ④ apply_mutations 验证 set-false 真移除 + set-true 写 presence（media + form + list + img + microdata）。
    sandbox
        .execute(
            "document.getElementById('v').muted = false;\
             document.getElementById('v').controls = false;\
             document.getElementById('ol').reversed = false;\
             document.getElementById('img').isMap = false;\
             document.getElementById('d').itemScope = false;\
             document.getElementById('f').noValidate = false;\
             document.getElementById('v2').loop = true;",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    // v（原 autoplay controls muted loop playsinline）：muted/controls=false 移除，autoplay/loop/playsinline 保留。
    assert!(!bool_attr_present(&out, "muted"), "video.muted=false 后属性移除\n{out}");
    assert!(!bool_attr_present(&out, "controls"), "video.controls=false 后属性移除\n{out}");
    assert!(bool_attr_present(&out, "autoplay"), "video.autoplay 仍 present（未设 false）\n{out}");
    assert!(bool_attr_present(&out, "loop"), "video.loop 仍 present（未设 false）\n{out}");
    // ol.reversed / img.ismap / div.itemscope / form.novalidate 移除；v2.loop 新增 present。
    assert!(!bool_attr_present(&out, "reversed"), "ol.reversed=false 后属性移除\n{out}");
    assert!(!bool_attr_present(&out, "ismap"), "img.isMap=false 后属性移除\n{out}");
    assert!(!bool_attr_present(&out, "itemscope"), "div.itemScope=false 后属性移除\n{out}");
    assert!(!bool_attr_present(&out, "novalidate"), "form.noValidate=false 后属性移除\n{out}");

    // ⑤ true→false→true toggle 末态 true（latest-wins set 序列，全属性统一）。
    sandbox
        .execute(
            "var v2 = document.getElementById('v2');\
             v2.muted = true; v2.muted = false; v2.muted = true;\
             globalThis.__tgMu = String(v2.muted);\
             var img2 = document.getElementById('img');\
             img2.isMap = false; img2.isMap = true;\
             globalThis.__tgIm = String(img2.isMap);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__tgMu").unwrap().value, "true", "muted true→false→true 末态 true");
    assert_eq!(sandbox.execute("globalThis.__tgIm").unwrap().value, "true", "isMap false→true 末态 true");
}

#[test]
fn test_reflected_uint_cols_rows_start_r3041() {
    // R3041：数值型 reflected cols/rows/start（R3038 follow-up——colSpan/rowSpan/maxLength/minLength 已在 R3038）。
    // textarea.cols（default 20）/textarea.rows（default 2）/ol.start（default 1）旧读恒 undefined。本切片补 number 语义读
    //（扩 _REFLECTED_UINT 表，set 走既有 generic fallthrough 写属性串，读 parseInt 往返——同 maxLength 模式）。
    // 另验 TABLE.rows 行集合（R2843）不受影响（更早分支返集合，textarea.rows 才命中 _REFLECTED_UINT）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <textarea id='ta' cols='40' rows='8'>txt</textarea>\
         <textarea id='ta2'></textarea>\
         <ol id='ol' start='5'><li>a</li><li>b</li></ol>\
         <ol id='ol2'><li>x</li></ol>\
         <table id='tbl'><tbody><tr><td>c</td></tr><tr><td>d</td></tr></tbody></table>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 数值型 reflected 读（number，旧恒 undefined）。
    sandbox
        .execute(
            "globalThis.__cols = document.getElementById('ta').cols;\
             globalThis.__rows = document.getElementById('ta').rows;\
             globalThis.__start = document.getElementById('ol').start;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cols").unwrap().value, "40", "textarea.cols=40（number）");
    assert_eq!(sandbox.execute("globalThis.__rows").unwrap().value, "8", "textarea.rows=8（number）");
    assert_eq!(sandbox.execute("globalThis.__start").unwrap().value, "5", "ol.start=5（number）");

    // ② 缺省 default：cols=20 / rows=2 / start=1（spec default）。
    sandbox
        .execute(
            "globalThis.__cols2 = document.getElementById('ta2').cols;\
             globalThis.__rows2 = document.getElementById('ta2').rows;\
             globalThis.__start2 = document.getElementById('ol2').start;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cols2").unwrap().value, "20", "textarea.cols 缺省=20（spec default）");
    assert_eq!(sandbox.execute("globalThis.__rows2").unwrap().value, "2", "textarea.rows 缺省=2（spec default）");
    assert_eq!(sandbox.execute("globalThis.__start2").unwrap().value, "1", "ol.start 缺省=1（spec default）");

    // ③ set→get round-trip（IDL setter 经 generic fallthrough 写属性 → 读 parseInt 反映）。
    sandbox
        .execute(
            "var ta = document.getElementById('ta');\
             ta.cols = 60; ta.rows = 4;\
             var ol = document.getElementById('ol');\
             ol.start = 10;\
             globalThis.__cols3 = ta.cols;\
             globalThis.__rows3 = ta.rows;\
             globalThis.__start3 = ol.start;\
             globalThis.__cols3Has = ta.getAttribute('cols');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cols3").unwrap().value, "60", "ta.cols=60 后读 60（round-trip）");
    assert_eq!(sandbox.execute("globalThis.__rows3").unwrap().value, "4", "ta.rows=4 后读 4（round-trip）");
    assert_eq!(sandbox.execute("globalThis.__start3").unwrap().value, "10", "ol.start=10 后读 10（round-trip）");
    assert_eq!(sandbox.execute("globalThis.__cols3Has").unwrap().value, "60", "ta.cols=60 写入 cols 内容属性");

    // ④ TABLE.rows 仍返行集合（R2843，不受 _REFLECTED_UINT 扩展影响）——2 行（tr），非 number。
    sandbox
        .execute("globalThis.__tblRows = document.getElementById('tbl').rows.length;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__tblRows").unwrap().value, "2", "TABLE.rows 仍为行集合（length=2，非 numeric default 2）");
}

#[test]
fn test_expando_non_primitive_properties_r3042() {
    // R3042：expando 非原始属性 set/get 修复。set trap generic fallthrough 旧对**非原始值**（function/object/null）
    // 写垃圾内容属性（`__zw_set_attr(sel, p, '[object Object]')`）且 get 读不回（undefined）。real browser：expando
    // 存于 JS 对象非内容属性。本切片改存 per-element expando map（get 读回）。仅非原始值——string/number/boolean
    // 保持 generic fallthrough 写属性行为（零回归：真 reflected attr setter 永不收非原始值）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'>x</div><button id='b'>btn</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① function expando：`el.handler = fn` 存 expando，读回为同一 function 可调用（旧读 undefined）。
    sandbox
        .execute(
            "var b = document.getElementById('b');\
             b.handler = function(x) { return x * 2; };\
             globalThis.__hType = typeof b.handler;\
             globalThis.__hCall = b.handler(21);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hType").unwrap().value, "function", "b.handler typeof=function（旧 undefined）");
    assert_eq!(sandbox.execute("globalThis.__hCall").unwrap().value, "42", "b.handler(21)=42（可调用）");

    // ② object expando：`el._data = {a:1,b:2}` 存 expando，读回可访问字段（jQuery/analytics 高频用法）。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             d._data = { a: 1, b: 'two', nested: { v: 99 } };\
             globalThis.__dType = typeof d._data;\
             globalThis.__da = d._data.a;\
             globalThis.__db = d._data.b;\
             globalThis.__dn = d._data.nested.v;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__dType").unwrap().value, "object", "d._data typeof=object（旧 undefined）");
    assert_eq!(sandbox.execute("globalThis.__da").unwrap().value, "1", "d._data.a=1");
    assert_eq!(sandbox.execute("globalThis.__db").unwrap().value, "two", "d._data.b='two'");
    assert_eq!(sandbox.execute("globalThis.__dn").unwrap().value, "99", "d._data.nested.v=99（深访问）");

    // ③ array + null expando：array 存 expando 读回 length；null 存 expando 读回 null。
    sandbox
        .execute(
            "d.items = [10, 20, 30];\
             d.flag = null;\
             globalThis.__arrLen = d.items.length;\
             globalThis.__arr1 = d.items[1];\
             globalThis.__flag = String(d.flag);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__arrLen").unwrap().value, "3", "d.items.length=3（array expando）");
    assert_eq!(sandbox.execute("globalThis.__arr1").unwrap().value, "20", "d.items[1]=20");
    assert_eq!(sandbox.execute("globalThis.__flag").unwrap().value, "null", "d.flag=null（null expando，旧写 attr flag='null'）");

    // ④ expando 非内容属性——不发 attributes MO 记录，且 apply_mutations 不写垃圾属性。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var mo = new MutationObserver(function(){});\
             mo.observe(d, { attributes: true });\
             d._data = { refreshed: true };\
             d.handler2 = function(){};\
             globalThis.__moCount = mo.takeRecords().length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__moCount").unwrap().value, "0", "expando set 不发 attributes MO（非内容属性）");
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(!out.contains("_data"), "apply 不写 _data 垃圾属性\n{out}");
    assert!(!out.contains("handler"), "apply 不写 handler 垃圾属性\n{out}");

    // ⑤ R3069 行为更新：reflected attr setter 仍写属性（role 有显式 get 分支 → set 写 role 属性，回归守卫）；
    //   非 reflected 原始值（customStr/customNum）现在入 expando 不写内容属性（real browser 语义，闭合 R3042 限制①），
    //   且 property 读回真值（get trap 读 _expando round-trip）。
    sandbox
        .execute(
            "var d2 = document.getElementById('d');\
             d2.role = 'button';\
             d2.customStr = 'hello';\
             d2.customNum = 7;\
             globalThis.__roleAttr = d2.getAttribute('role');\
             globalThis.__customAttr = String(d2.getAttribute('customStr') === null || d2.getAttribute('customStr') === '');\
             globalThis.__numAttr = String(d2.getAttribute('customNum') === null || d2.getAttribute('customNum') === '');\
             globalThis.__customRead = d2.customStr;\
             globalThis.__numRead = d2.customNum;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__roleAttr").unwrap().value, "button", "role='button' 仍写属性（reflected setter 不受 R3069 影响）");
    assert_eq!(sandbox.execute("globalThis.__customAttr").unwrap().value, "true", "customStr='hello' 入 expando 不写内容属性（R3069，旧写 'hello' 属性）");
    assert_eq!(sandbox.execute("globalThis.__numAttr").unwrap().value, "true", "customNum=7 入 expando 不写内容属性（R3069，旧写 '7' 属性）");
    assert_eq!(sandbox.execute("globalThis.__customRead").unwrap().value, "hello", "customStr property 读回 'hello'（expando round-trip）");
    assert_eq!(sandbox.execute("globalThis.__numRead").unwrap().value, "7", "customNum property 读回 7（expando round-trip 保类型）");

    // ⑥ 多元素独立 expando（per-element 隔离）+ reflected 布尔 setter 不受影响。
    sandbox
        .execute(
            "b._own = { id: 'B' }; d._own = { id: 'D' };\
             globalThis.__bOwn = b._own.id;\
             globalThis.__dOwn = d._own.id;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__bOwn").unwrap().value, "B", "b._own.id='B'（per-element 隔离）");
    assert_eq!(sandbox.execute("globalThis.__dOwn").unwrap().value, "D", "d._own.id='D'（per-element 隔离，不串）");
}

#[test]
fn test_expando_primitive_properties_r3069() {
    // R3069：expando 原始值 set/get 修复（闭合 R3042 限制①）。string/number/boolean 非 reflected 属性旧经 set
    // generic fallthrough 写内容属性，但 get trap 无分支读 → 读返 undefined（`el.flag='x'; el.flag` → undefined，
    // correctness bug）。real browser：自定义原始属性存 JS 对象非内容属性。本切片改：非 reflected 原始值存 _expando
    //（get trap 已读 _expando），保类型；reflected 原始属性（type/name/colSpan/size 等）仍写属性（get 读属性 round-trip）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d' title='orig'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① string 原始 expando：`el.customFlag = 'on'` → 读回 'on'（旧读 undefined）。不写内容属性。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             d.customFlag = 'on';\
             globalThis.__strRead = d.customFlag;\
             globalThis.__strNotWritten = String(d.getAttribute('customFlag') !== 'on');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__strRead").unwrap().value, "on", "string expando 读回 'on'（旧 undefined）");
    assert_eq!(sandbox.execute("globalThis.__strNotWritten").unwrap().value, "true", "string expando 不写内容属性（getAttribute != 'on'）");

    // ② number 原始 expando：`el.count = 42` → 读回 42（number，保类型，非 '42'）。
    sandbox.execute("d.count = 42; globalThis.__numRead = d.count; globalThis.__numType = typeof d.count;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__numRead").unwrap().value, "42", "number expando 读回 42");
    assert_eq!(sandbox.execute("globalThis.__numType").unwrap().value, "number", "number expando 保类型 number（非 string）");

    // ③ boolean 原始 expando：`el.enabled = true` → 读回 true（boolean，保类型）。
    sandbox.execute("d.enabled = true; globalThis.__boolRead = d.enabled; globalThis.__boolType = typeof d.enabled;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__boolRead").unwrap().value, "true", "boolean expando 读回 true");
    assert_eq!(sandbox.execute("globalThis.__boolType").unwrap().value, "boolean", "boolean expando 保类型 boolean");

    // ④ reflected string 属性仍写属性（regression 守卫）：`el.title='new'` → 读 'new'（经 reflected getter 读属性）。
    sandbox.execute("d.title = 'new'; globalThis.__refRead = d.title;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__refRead").unwrap().value, "new", "reflected title 仍写属性 + 读回（regression 守卫）");

    // ⑤ reflected 原始属性不污染 expando：title 写属性（非 expando），customFlag 仍在 expando（互不影响）。
    sandbox.execute("globalThis.__mixed = d.title + '/' + d.customFlag;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__mixed").unwrap().value, "new/on", "reflected(title)+expando(customFlag) 共存互不影响");

    // ⑥ reflected numeric（colSpan 经 td）仍写属性 round-trip。
    let mut s2 = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() }).unwrap();
    s2.execute(generate_js_dom_shim()).unwrap();
    let m2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let h2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><table><tr><td id='td'></td></tr></table></body></html>".to_string()));
    let u2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut s2, &m2, &h2, &u2, &canvas_registry);
    s2.execute("var td = document.getElementById('td'); td.colSpan = 3; td.myProp = 'exp'; globalThis.__cs = td.colSpan; globalThis.__mp = td.myProp;").unwrap();
    assert_eq!(s2.execute("globalThis.__cs").unwrap().value, "3", "reflected numeric colSpan=3 仍写属性 + 读回（regression 守卫）");
    assert_eq!(s2.execute("globalThis.__mp").unwrap().value, "exp", "同元素 colSpan(reflected) + myProp(expando) 共存");
}

#[test]
fn test_expando_enumeration_r3046() {
    // R3046：expando 枚举表面（R3042 follow-up，闭合已知限制④）。主元素 proxy 加 has/ownKeys/
    // getOwnPropertyDescriptor 三 trap 暴露 expando 为 enumerable own 属性——`Object.keys(el)` / `for...in` /
    // `'foo' in el` / `Object.assign` 含 expando（real browser 语义）。旧 default（target {} 空）→ 全不含。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'>x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 设 expando 前：Object.keys(el) 不含 expando（无 expando → []）。
    sandbox
        .execute("globalThis.__keysBefore = JSON.stringify(Object.keys(document.getElementById('d')));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__keysBefore").unwrap().value,
        "[]",
        "设 expando 前 Object.keys(el)=[]"
    );

    // ② 设 expando（object/function/array + number）后：Object.keys(el) 含 expando 键。
    //    R3069：原始值（number/string/boolean）亦入 _expando map（闭合 R3042 限制①），故 numProp=42 现可枚举
    //    （旧 R3042 限制① 下 number 走 attr fallthrough 不入 expando、不可枚举——R3069 后与 real browser 一致）。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             d._data = { a: 1 };\
             d.handler = function(){};\
             d.tags = ['x','y'];\
             d.numProp = 42;\
             globalThis.__keysAfter = JSON.stringify(Object.keys(d).sort());",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__keysAfter").unwrap().value,
        "[\"_data\",\"handler\",\"numProp\",\"tags\"]",
        "Object.keys(el) 含全部 expando 键（含 numProp=42，R3069 原始值亦入 expando）"
    );

    // ③ `'foo' in el` 对 expando（含原始值 numProp）返 true；未设 返 false。
    sandbox
        .execute(
            "globalThis.__inData = String('_data' in d);\
             globalThis.__inNum = String('numProp' in d);\
             globalThis.__inNope = String('notSet' in d);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__inData").unwrap().value, "true", "'_data' in el = true（非原始 expando）");
    assert_eq!(sandbox.execute("globalThis.__inNum").unwrap().value, "true", "'numProp' in el = true（number 亦入 expando，R3069）");
    assert_eq!(sandbox.execute("globalThis.__inNope").unwrap().value, "false", "'notSet' in el = false（未设）");

    // ④ getOwnPropertyDescriptor 返 enumerable own 描述符（非原始 expando）。
    sandbox
        .execute(
            "var desc = Object.getOwnPropertyDescriptor(d, '_data');\
             globalThis.__descDataA = String(desc.value.a);\
             globalThis.__descEnum = String(desc.enumerable);\
             globalThis.__descConf = String(desc.configurable);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__descDataA").unwrap().value, "1", "descriptor.value.a=1（_data 对象保真）");
    assert_eq!(sandbox.execute("globalThis.__descEnum").unwrap().value, "true", "descriptor.enumerable=true");
    assert_eq!(sandbox.execute("globalThis.__descConf").unwrap().value, "true", "descriptor.configurable=true");

    // ⑤ for...in 迭代含 expando 键（含原始值 numProp，R3069）。
    sandbox
        .execute(
            "var collected = [];\
             for (var k in d) collected.push(k);\
             globalThis.__forIn = JSON.stringify(collected.sort());",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__forIn").unwrap().value,
        "[\"_data\",\"handler\",\"numProp\",\"tags\"]",
        "for...in 含全部 expando 键（含 numProp）"
    );

    // ⑥ Object.assign({}, el) 复制非原始 expando（值保持类型）。
    sandbox
        .execute(
            "var copy = Object.assign({}, d);\
             globalThis.__copyDataA = String(copy._data.a);\
             globalThis.__copyHandler = typeof copy.handler;\
             globalThis.__copyTagsLen = String(copy.tags.length);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__copyDataA").unwrap().value, "1", "Object.assign 复制 _data.a=1");
    assert_eq!(sandbox.execute("globalThis.__copyHandler").unwrap().value, "function", "Object.assign 复制 handler=function");
    assert_eq!(sandbox.execute("globalThis.__copyTagsLen").unwrap().value, "2", "Object.assign 复制 tags.length=2（array 保真）");

    // ⑦ 回归守卫：reflected 属性（id）经 get trap 读但非 own → Object.keys 不含、'id' in el 仍 false（pre-existing）。
    sandbox
        .execute(
            "globalThis.__idGet = document.getElementById('d').id;\
             globalThis.__idIn = String('id' in document.getElementById('d'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__idGet").unwrap().value, "d", "el.id 经 get trap 读='d'（reflected 仍工作）");
    assert_eq!(sandbox.execute("globalThis.__idIn").unwrap().value, "false", "'id' in el=false（reflected 非 own，pre-existing 不变）");
}

#[test]
fn test_reflected_size_element_aware_r3043() {
    // R3043：`.size` element-aware reflected 数值读。input.size default 20 / select.size default 0（spec 两元素 default 不同，
    // `_REFLECTED_UINT` 表无 element-awareness 故专用 tag-gate 分支）。旧读恒 undefined。set 走 generic fallthrough 写 size 属性。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <input id='i1' size='10'>\
         <input id='i2'>\
         <select id='s1' size='3'><option>a</option><option>b</option></select>\
         <select id='s2'><option>x</option></select>\
         <div id='d'>x</div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① size 读（number，旧恒 undefined）——input=10 / select=3。
    sandbox
        .execute(
            "globalThis.__i1 = document.getElementById('i1').size;\
             globalThis.__s1 = document.getElementById('s1').size;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__i1").unwrap().value, "10", "input.size=10（number）");
    assert_eq!(sandbox.execute("globalThis.__s1").unwrap().value, "3", "select.size=3（number）");

    // ② 缺省 default（element-aware）：input=20 / select=0。
    sandbox
        .execute(
            "globalThis.__i2 = document.getElementById('i2').size;\
             globalThis.__s2 = String(document.getElementById('s2').size);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__i2").unwrap().value, "20", "input.size 缺省=20（spec default）");
    assert_eq!(sandbox.execute("globalThis.__s2").unwrap().value, "0", "select.size 缺省=0（spec default，区别 input 20）");

    // ③ set→get round-trip（generic fallthrough 写 size 属性 → 读 parseInt 反映）。
    sandbox
        .execute(
            "document.getElementById('i2').size = 8;\
             document.getElementById('s2').size = 5;\
             globalThis.__i3 = document.getElementById('i2').size;\
             globalThis.__s3 = document.getElementById('s2').size;\
             globalThis.__i3Attr = document.getElementById('i2').getAttribute('size');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__i3").unwrap().value, "8", "input.size=8 后读 8（round-trip）");
    assert_eq!(sandbox.execute("globalThis.__s3").unwrap().value, "5", "select.size=5 后读 5（round-trip）");
    assert_eq!(sandbox.execute("globalThis.__i3Attr").unwrap().value, "8", "input.size=8 写入 size 内容属性");

    // ④ 非 INPUT/SELECT 元素 .size → undefined（real browser 无 .size；不误返 number default）。
    sandbox.execute("globalThis.__dSize = String(document.getElementById('d').size);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__dSize").unwrap().value, "undefined", "div.size=undefined（非 input/select fall through）");
}

#[test]
fn test_form_get_submission_url_r3054() {
    // R3054：form_get_submission_url 解析 <form> GET 提交目标 URL（闭合 click 默认动作族：reset/anchor/hash →
    // form-submit 导航）。成功控件收集 + urlencoded query 替换 action URL query 段 + 按 base 解析 action。
    let base = "https://example.com/search";

    // ① 基础 GET：text + hidden → action?name=value（action 绝对 URL）。
    let html = "<html><body>\
        <form id='f' action='https://example.com/search'>\
          <input name='q' value='rust'>\
          <input type='hidden' name='lang' value='en'>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html, "#f", None, base),
        Some("https://example.com/search?q=rust&lang=en".to_string()),
        "基础 GET：q=rust & lang=en"
    );

    // ② action 缺省 → 提交到当前文档 URL（base_url）。
    let html2 = "<html><body><form id='f'><input name='p' value='1'></form></body></html>";
    assert_eq!(
        form_get_submission_url(html2, "#f", None, base),
        Some("https://example.com/search?p=1".to_string()),
        "action 缺省 → 提交到 base_url"
    );

    // ③ action 相对 → 按 base 解析为绝对。
    let html3 = "<html><body><form id='f' action='/results'><input name='q' value='hi'></form></body></html>";
    assert_eq!(
        form_get_submission_url(html3, "#f", None, base),
        Some("https://example.com/results?q=hi".to_string()),
        "相对 action /results → 绝对 https://example.com/results?q=hi"
    );

    // ④ checkbox/radio：仅 checked 入；值=value 属性或 "on"。
    let html4 = "<html><body><form id='f' action='/s'>\
        <input type='checkbox' name='c1' value='yes' checked>\
        <input type='checkbox' name='c2' value='no'>\
        <input type='radio' name='r' checked>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html4, "#f", None, base),
        Some("https://example.com/s?c1=yes&r=on".to_string()),
        "checkbox checked 入 c1=yes；unchecked c2 跳过；radio 无 value → on"
    );

    // ⑤ select：selected option 值（value 属性优先）；无 selected → 首 option（spec 默认选中 quirk）。
    let html5 = "<html><body><form id='f' action='/s'>\
        <select name='size'>\
          <option value='sm'>Small</option>\
          <option value='lg' selected>Large</option>\
        </select>\
        <select name='def'>\
          <option>One</option>\
          <option>Two</option>\
        </select>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html5, "#f", None, base),
        Some("https://example.com/s?size=lg&def=One".to_string()),
        "select：size=lg（selected）；def=One（无 selected → 首项，无 value 属性 → 文本）"
    );

    // ⑥ textarea：值 = 子树文本内容；跳过无 name / disabled。
    let html6 = "<html><body><form id='f' action='/s'>\
        <textarea name='msg'>hello world</textarea>\
        <input name='skip' disabled value='x'>\
        <input value='noname'>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html6, "#f", None, base),
        Some("https://example.com/s?msg=hello+world".to_string()),
        "textarea=文本（form-urlencoded 空格→+）；disabled skip 跳过；无 name noname 跳过"
    );

    // ⑦ submitter：click submit 按钮其 name=value 入（type=submit）；非激活 submit/reset/button 跳过。
    // submitter_sel 须为 query_all_in_subtree 同款选择器——id'd 元素两端均产 `#id`（stable_selector_for_node
    // / selector_from_element_hit 一致），故 id'd submit 按钮可靠匹配。
    let html7 = "<html><body><form id='f' action='/s'>\
        <input name='q' value='x'>\
        <button id='go' type='submit' name='go' value='search'>Go</button>\
        <input type='submit' name='other' value='o'>\
        </form></body></html>";
    assert_eq!(
        form_get_submission_url(html7, "#f", Some("#go"), base),
        Some("https://example.com/s?q=x&go=search".to_string()),
        "submitter #go name=go=search 入；非激活 submit other 跳过"
    );
    // 无 submitter → 无提交按钮值（仅普通控件）。
    assert_eq!(
        form_get_submission_url(html7, "#f", None, base),
        Some("https://example.com/s?q=x".to_string()),
        "无 submitter → 仅 q=x（submit 按钮均不参与）"
    );

    // ⑧ method=POST → None（headless POST 导航 defer）；method=GET（显式）→ Some。
    let html8p = "<html><body><form id='f' method='post' action='/s'><input name='q' value='x'></form></body></html>";
    assert_eq!(
        form_get_submission_url(html8p, "#f", None, base),
        None,
        "method=POST → None（headless POST defer）"
    );
    let html8g = "<html><body><form id='f' method='GET' action='/s'><input name='q' value='x'></form></body></html>";
    assert_eq!(
        form_get_submission_url(html8g, "#f", None, base),
        Some("https://example.com/s?q=x".to_string()),
        "method=GET（显式大写）→ Some"
    );

    // ⑨ 现有 query 段被替换（spec：form 数据集替换 action URL query）。
    let html9 = "<html><body><form id='f' action='/s?old=1#frag'><input name='q' value='new'></form></body></html>";
    assert_eq!(
        form_get_submission_url(html9, "#f", None, base),
        Some("https://example.com/s?q=new#frag".to_string()),
        "action 旧 query old=1 被替换为 q=new；fragment 保留"
    );

    // ⑩ 特殊字符 urlencoded（spec application/x-www-form-urlencoded）。
    let html10 = "<html><body><form id='f' action='/s'><input name='q' value='a b&c=d'></form></body></html>";
    let url10 = form_get_submission_url(html10, "#f", None, base).unwrap();
    assert!(
        url10.contains("q=a+b%26c%3Dd") || url10.contains("q=a%20b%26c%3Dd"),
        "特殊字符 urlencoded（空格 + / %20，& → %26，= → %3D）：{url10}"
    );

    // ⑪ 非 form 元素 / 无控件 → None / action 无 query。
    assert_eq!(
        form_get_submission_url("<html><body><div id='f'><input name='q' value='x'></div></body></html>", "#f", None, base),
        None,
        "非 <form> 元素 → None"
    );
    assert_eq!(
        form_get_submission_url("<html><body><form id='f' action='/s'></form></body></html>", "#f", None, base),
        Some("https://example.com/s".to_string()),
        "无控件 GET → action 无 query（仅路径）"
    );
}

#[test]
fn test_htmlcollection_proxy_semantics_r50() {
    // R50：HTMLCollection 从 Array 承载升级为 Proxy 承载（spec legacy platform object，
    // https://dom.spec.whatwg.org/#interface-htmlcollection + WebIDL legacy platform objects）。
    // 覆盖 WPT dom/collections 五用例的语义面：own 枚举（无 length/item/namedItem）、无
    // values/entries/forEach、indexed/named 拒绝 set/defineProperty/delete、canonical 索引
    // 边界（负数/2^31/2^32 走 named）、illegal invocation（作 prototype）、live overlay
    // （同步 appendChild 后 c[0] 可见）。
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
        "<html><body><i id='foo'></i><b id='bar' name='baz'></b></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① own 枚举：[indices…, names…]——无 length/item/namedItem（spec supported property
    //    names；WPT supported-property-names 首断言）。
    sandbox
        .execute(
            "var c = document.getElementsByTagName('i');\n\
             globalThis.__own = Object.getOwnPropertyNames(c).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__own").unwrap().value,
        "0,foo",
        "own keys = ['0','foo']（无 length/item/namedItem）"
    );

    // ② 无 values/entries/forEach（spec HTMLCollection 非 iterable interface member；WPT
    //    HTMLCollection-iterator）+ @@iterator 存在 + for-of 可迭代。
    sandbox
        .execute(
            "globalThis.__no_iter = ('values' in c) + ',' + ('entries' in c) + ',' + ('forEach' in c);\n\
             globalThis.__has_it = typeof Symbol !== 'undefined' && typeof c[Symbol.iterator] === 'function';\n\
             var ids = [];\n\
             for (var el of c) ids.push(el.getAttribute('id'));\n\
             globalThis.__forof = ids.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__no_iter").unwrap().value,
        "false,false,false",
        "values/entries/forEach 不存在（非 iterable 接口成员）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__has_it)").unwrap().value,
        "true",
        "@@iterator 存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__forof").unwrap().value,
        "foo",
        "for-of 迭代产出元素"
    );

    // ③ indexed 拒绝 set/defineProperty/delete（loose no-op；strict/define 抛 TypeError——
    //    getOwnPropertyDescriptor 报 non-writable non-configurable 语义由 trap 表达）。
    sandbox
        .execute(
            "var before = c[0];\n\
             c[0] = 'x';\n\
             globalThis.__set_kept = (c[0] === before);\n\
             globalThis.__def_threw = 'no';\n\
             try { Object.defineProperty(c, 0, { value: 5 }); } catch (e) { globalThis.__def_threw = 'yes'; }\n\
             delete c[0];\n\
             globalThis.__del_kept = (c[0] === before);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__set_kept)").unwrap().value,
        "true",
        "indexed set 不覆盖元素（loose no-op）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__def_threw").unwrap().value,
        "yes",
        "defineProperty 已有 indexed 抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__del_kept)").unwrap().value,
        "true",
        "delete indexed no-op（元素保留）"
    );

    // ④ named 拒绝 set（WPT own-props "Setting non-array index while named property exists"）。
    sandbox
        .execute(
            "var b = document.getElementsByTagName('b');\n\
             var el = b.namedItem('bar');\n\
             b['bar'] = 'x';\n\
             globalThis.__named_kept = (b['bar'] === el);\n\
             globalThis.__named_strict = 'no';\n\
             try { b['baz'] = 'x'; } catch (e) { globalThis.__named_strict = 'yes'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__named_kept)").unwrap().value,
        "true",
        "named set 不覆盖元素（loose no-op）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__named_strict").unwrap().value,
        "no",
        "loose named set 静默 no-op（不抛）"
    );

    // ⑤ canonical 索引边界：item() ToUint32（4294967296 → 0）；'-2' 落 named getter
    //    （WPT supported-property-indices）。
    sandbox
        .execute(
            "var d = document.createElement('i'); d.id = '-2'; document.body.appendChild(d);\n\
             var neg = document.getElementsByTagName('i');\n\
             globalThis.__neg_named = String(neg['-2'] === d);\n\
             globalThis.__wrap_item = String(neg.item(4294967296) === neg[0]);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__neg_named").unwrap().value,
        "true",
        "'-2'（非 canonical 索引）落 named getter 命中新元素"
    );
    assert_eq!(
        sandbox.execute("globalThis.__wrap_item").unwrap().value,
        "true",
        "item(2^32) ToUint32 → 0（命中首元素）"
    );

    // ⑥ live overlay：同步脚本 appendChild 后 c[0] 立即可见（WPT own-props "Setting array
    //    index while indexed property doesn't exist (loose)"：append 后 c[0]===element）。
    sandbox
        .execute(
            "var q = document.getElementsByTagName('q');\n\
             globalThis.__q_empty = q.length;\n\
             q[0] = 'foo';\n\
             globalThis.__q_still_undef = String(q[0] === undefined);\n\
             var el = document.createElement('q');\n\
             document.body.appendChild(el);\n\
             globalThis.__q_live = String(q[0] === el);\n\
             globalThis.__q_len = q.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__q_empty)").unwrap().value,
        "0",
        "初始空集合"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__q_still_undef)").unwrap().value,
        "true",
        "越界 indexed set 被拒（仍 undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__q_live").unwrap().value,
        "true",
        "同步 appendChild 后 c[0] === element（live overlay）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__q_len)").unwrap().value,
        "1",
        "live overlay 后 length=1"
    );

    // ⑦ illegal invocation：collection 作 prototype，base object 读 .length 抛 TypeError
    //    （WPT HTMLCollection-as-prototype）。
    sandbox
        .execute(
            "var obj = Object.create(document.getElementsByTagName('i'));\n\
             globalThis.__proto_len_threw = 'no';\n\
             try { obj.length; } catch (e) { globalThis.__proto_len_threw = 'yes'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__proto_len_threw").unwrap().value,
        "yes",
        "Object.create(collection).length 抛 illegal invocation TypeError"
    );

    // ⑧ expando：可写可删，own 枚举含 expando（WPT "with expando object"）。
    sandbox
        .execute(
            "var e2 = document.getElementsByTagName('b');\n\
             e2.someProperty = 'v';\n\
             globalThis.__expando = e2.someProperty;\n\
             globalThis.__expando_own = Object.getOwnPropertyNames(e2).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__expando").unwrap().value,
        "v",
        "expando 可写可读"
    );
    assert_eq!(
        sandbox.execute("globalThis.__expando_own").unwrap().value,
        "0,bar,baz,someProperty",
        "own keys = [indices, names, expando]"
    );
}
