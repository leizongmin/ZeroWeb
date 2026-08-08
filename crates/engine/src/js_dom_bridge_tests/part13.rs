// js_dom_bridge 测试模块拆分 part 13（R3028+，控制单文件 <2000 行，include! 入 js_dom_bridge_tests.rs）。
// 承接 part12 溢出：MutationObserver characterDataOldValue（R3028）+ innerHTML childList emission（R3029）。

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① characterDataOldValue：首次变更 oldValue=初值 'init'；二次变更 oldValue=前值 'first'。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             var mo = new MutationObserver(function(){});\
             mo.observe(a, { characterData: true, subtree: true, characterDataOldValue: true });\
             a.textContent = 'first';\
             globalThis.__r1 = mo.takeRecords();\
             a.textContent = 'second';\
             globalThis.__r2 = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].type").unwrap().value,
        "characterData",
        "characterDataOldValue：记录 type=characterData"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r1[0].oldValue").unwrap().value,
        "init",
        "characterDataOldValue：首次变更 oldValue=初值 'init'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2[0].oldValue").unwrap().value,
        "first",
        "characterDataOldValue：二次变更 oldValue=前值 'first'（latest-wins 反映同批前序 set）"
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
        sandbox.execute("String(globalThis.__r3[0].oldValue)").unwrap().value,
        "null",
        "未请求 characterDataOldValue → oldValue=null（即使有旧值）"
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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

    // ② forEach 仍工作（数组原生，未被 item/namedItem 破坏）+ for...in 不泄漏 item/namedItem（非 enumerable）。
    sandbox
        .execute(
            "globalThis.__fe = []; ps.forEach(function(el){ globalThis.__fe.push(el.id); });\
             globalThis.__feJ = String(globalThis.__fe);\
             var keys = []; for (var k in ps) keys.push(k); globalThis.__inKeys = String(keys);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__feJ").unwrap().value, "p1,p2", "forEach 迭代仍工作（p1,p2）");
    assert_eq!(
        sandbox.execute("globalThis.__inKeys").unwrap().value,
        "0,1",
        "for...in 仅 0,1（item/namedItem 非 enumerable 不泄漏）"
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox2, &m2, &h2, &pu2);
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
