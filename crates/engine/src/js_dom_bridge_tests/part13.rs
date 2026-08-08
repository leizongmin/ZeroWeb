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
