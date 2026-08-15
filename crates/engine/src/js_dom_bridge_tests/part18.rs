// js_dom_bridge 测试模块拆分 part 18（js-dom M4 R51：child→parent 反向链 + pending overlay
// + pre-insert HierarchyRequestError——WPT dom/common.js indexOf 死循环根因修复）。

#[test]
fn r51_handle_child_parent_node_reflects_real_parent() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 反向链：handle 子（createElement 产物）appendChild 后 parentNode 必须返回真实父
    //（旧 fallback 恒猜 body——WPT dom indexOf 的 `while (node != parentNode.childNodes[i])`
    // 在假父快照上越界恒不等 → 死循环根因）。
    sandbox
        .execute(
            "var host = document.getElementById('host');\n\
             var child = document.createElement('p');\n\
             host.appendChild(child);\n\
             globalThis.__r1 = (child.parentNode === host);\n\
             globalThis.__r2 = (child.parentNode && child.parentNode.id);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "true",
        "R51：handle 子 append 后 parentNode === 真实父 proxy（反链命中）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "host",
        "R51：反链父的 sel 正确（#host）"
    );
}

#[test]
fn r51_detached_handle_node_parent_node_is_null() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='x'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51b".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec：detached 节点（createElement 后未 append）parentNode === null。旧 fallback 猜
    // body 是 WPT indexOf 死循环根因的另一形态（假父快照永不含该节点）。
    sandbox
        .execute(
            "var el = document.createElement('div');\n\
             globalThis.__detached = (el.parentNode === null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__detached").unwrap().value,
        "true",
        "R51：未 append 的 handle 节点 parentNode === null（不再猜 body）"
    );
}

#[test]
fn r51_text_view_children_merge_with_appended_children() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='p'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51c".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 融合视图：textContent 建的本地文本子（前）+ appendChild 建的元素子（后）必须在同一
    // childNodes 里可见（旧短路：_zwLocalChildNodes 命中即 return，append 的子不可见）。
    sandbox
        .execute(
            "var p = document.createElement('p');\n\
             p.textContent = 'Äbc';\n\
             document.body.appendChild(p);\n\
             var el2 = document.createElement('span');\n\
             p.appendChild(el2);\n\
             var cn = p.childNodes;\n\
             globalThis.__len = cn.length;\n\
             globalThis.__t0 = cn[0].nodeType;\n\
             globalThis.__has = Array.prototype.indexOf.call(cn, el2);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__len").unwrap().value,
        "2",
        "R51：textContent 视图 + append 子融合（len=2，旧短路只见文本 1）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__t0").unwrap().value,
        "3",
        "R51：融合序文本子在前（nodeType=3）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__has").unwrap().value,
        "1",
        "R51：append 的元素子在 childNodes[1] 可见（indexOf 命中）"
    );
}

#[test]
fn r51_self_append_throws_hierarchy_request_error() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='self'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51d".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec `dom-node-pre-insert`：child 即 parent 自身 → HierarchyRequestError（WPT
    // Range-mutations 非法用例段；旧 shim 不抛真执行 → JS registry 自环 →
    // _zwHCCollectSubtree 无限递归 → RangeError 栈溢出）。
    sandbox
        .execute(
            "var p = document.createElement('p');\n\
             document.body.appendChild(p);\n\
             globalThis.__threw = 'no'; globalThis.__name = '';\n\
             try { p.appendChild(p); } catch (e) {\n\
               globalThis.__threw = 'yes'; globalThis.__name = e.name;\n\
             }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__threw").unwrap().value,
        "yes",
        "R51：appendChild(自身) 必须抛"
    );
    assert_eq!(
        sandbox.execute("globalThis.__name").unwrap().value,
        "HierarchyRequestError",
        "R51：self-append 抛 HierarchyRequestError（spec dom-node-pre-insert）"
    );
}

#[test]
fn r51_ancestor_append_throws_hierarchy_request_error() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='anc'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51e".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec `dom-node-pre-insert`：child 是 parent 的祖先 → HierarchyRequestError（WPT
    // Range-mutations "paras[0].appendChild(testDiv)"——旧 shim 真执行 → host apply mutations
    // 报「DOM 树中出现循环」整批 mutation 丢弃）。
    sandbox
        .execute(
            "var host = document.getElementById('anc');\n\
             var p = document.createElement('p');\n\
             p.textContent = 't';\n\
             host.appendChild(p);\n\
             globalThis.__threw = 'no'; globalThis.__name = '';\n\
             try { p.appendChild(host); } catch (e) {\n\
               globalThis.__threw = 'yes'; globalThis.__name = e.name;\n\
             }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__threw").unwrap().value,
        "yes",
        "R51：appendChild(祖先) 必须抛"
    );
    assert_eq!(
        sandbox.execute("globalThis.__name").unwrap().value,
        "HierarchyRequestError",
        "R51：ancestor-append 抛 HierarchyRequestError（从 parent 上行判定，方向正确）"
    );
}

#[test]
fn r51_moved_node_leaves_old_parent_childnodes() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='a'></div><div id='b'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51f".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec appendChild 移动语义：child 从旧父移到新父后，旧父 childNodes 不得双份残留
    //（R51：JS 侧旧父 overlay 剔除 + registry 剔除）。
    sandbox
        .execute(
            "var a = document.getElementById('a');\n\
             var b = document.getElementById('b');\n\
             var p1 = document.createElement('p'); p1.textContent = 'x';\n\
             a.appendChild(p1);\n\
             b.appendChild(p1);\n\
             var inA = Array.prototype.indexOf.call(a.childNodes, p1);\n\
             var inB = Array.prototype.indexOf.call(b.childNodes, p1);\n\
             globalThis.__res = inA + ',' + inB;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__res").unwrap().value,
        "-1,0",
        "R51：移动后旧父 childNodes 不含（-1）、新父含（0）"
    );
}

#[test]
fn r51c_query_selector_falls_back_to_pending_added_by_id() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='real'>R</div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51c".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // R51c：host 快照未命中（同步 turn 内 append 的节点）→ pending added 按 id 回落。
    // WPT dom/common.js setupRangeTests 每次开头 querySelector('#test') 取旧树 removeChild——
    // 无回落则跳过 remove → 旧 proxy 泄漏 → pending 表 O(n²)（dataChange 超时根因）。
    sandbox
        .execute(
            "var el = document.createElement('div');\n\
             el.id = 'fresh';\n\
             document.body.appendChild(el);\n\
             globalThis.__hit = document.querySelector('#fresh') === el;\n\
             globalThis.__real = document.querySelector('#real') !== null;\n\
             globalThis.__miss = document.querySelector('#nope') === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hit").unwrap().value,
        "true",
        "R51c：同步 turn 内 append 的 #fresh 可被 querySelector 查到（pending 回落）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__real").unwrap().value,
        "true",
        "R51c：host 快照命中的 #real 优先（回落不抢占）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__miss").unwrap().value,
        "true",
        "R51c：不存在的 id 仍返 null"
    );
}

#[test]
fn r51c_remove_after_append_zeroes_out_pending() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='z'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r51c2".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // R51c：add→remove 对冲消零——remove 后同 id 查询不应回落到已移除节点（WPT setup 重建
    // 模式的核心语义：remove 的旧树不得再被 querySelector 捞回）。
    sandbox
        .execute(
            "var el = document.createElement('div');\n\
             el.id = 'temp';\n\
             document.body.appendChild(el);\n\
             document.body.removeChild(el);\n\
             globalThis.__gone = document.querySelector('#temp') === null;\n\
             globalThis.__orphan = el.parentNode === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__gone").unwrap().value,
        "true",
        "R51c：remove 后 pending 对冲——同 id 查询返 null（不复活已移除节点）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__orphan").unwrap().value,
        "true",
        "R51c：remove 后 parentNode 为 null"
    );
}
