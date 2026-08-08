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
        "0",
        "innerHTML addedNodes=空（parse-based 新子不可同步枚举，documented 限制）"
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
