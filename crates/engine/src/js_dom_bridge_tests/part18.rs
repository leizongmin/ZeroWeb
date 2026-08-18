// js_dom_bridge 测试模块拆分 part 18（js-dom M4 R51：child→parent 反向链 + pending overlay
// + pre-insert HierarchyRequestError——WPT dom/common.js indexOf 死循环根因修复）。

#[test]
fn html5test_inner_html_is_synchronously_observable_on_created_element() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("https://zero.test/html5test".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#dom-innerhtml
    // html5test performs this probe before queued host mutations are applied.
    sandbox
        .execute(
            "var element = document.createElement('div');\n\
             element.innerHTML = '<form id=\"form\"></form><input form=\"form\">';\n\
             document.body.appendChild(element);\n\
             globalThis.__html5testSyncInnerHtml = element.childNodes.length === 2 &&\n\
               element.lastChild.form === element.firstChild &&\n\
               document.getElementById('form') === element.firstChild;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__html5testSyncInnerHtml").unwrap().value,
        "true",
        "created elements expose parsed innerHTML children and their synchronous form owner"
    );
}

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

// js-dom M4 R54：live collection 并入点过滤——detached/foreign 容器子树不进文档级集合
//（R53 根因：getElementsByTagName('p') 的 els 每 setup 净 +2 泄漏 → 失效循环 O(els) 级联变慢）。
// 判定从**挂载点**出发（mutSel → __zw_contains('html', sel)；mutHandle → _zwNodeParent
// 逐跳上行），不从子节点上行（R53 两版教训：pending 树 sel 链断在未 apply 的容器上）。
#[test]
fn r54_detached_container_subtree_not_in_document_collection() {
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
        "<html><body><div id='a'></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r54".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec：getElementsByTagName 只返主文档节点。append 到 detached 容器（未挂主文档）的
    // 元素不得出现在文档级集合里；同批 append 到主文档的元素正常进。
    sandbox
        .execute(
            "var doc = document;\n\
             var detachedDiv = doc.createElement('div');\n\
             var p1 = doc.createElement('p');\n\
             detachedDiv.appendChild(p1);\n\
             var p2 = doc.createElement('p');\n\
             doc.body.appendChild(p2);\n\
             var ps = doc.getElementsByTagName('p');\n\
             globalThis.__len = ps.length;\n\
             globalThis.__hasDetached = Array.prototype.indexOf.call(ps, p1);\n\
             globalThis.__hasInDoc = Array.prototype.indexOf.call(ps, p2);\n\
             globalThis.__hasInDoc2 = Array.prototype.indexOf.call(ps, p2);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__len").unwrap().value,
        "1",
        "R54：集合 = [p2]（主文档 append），detached 容器的 p 不进文档级集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasDetached").unwrap().value,
        "-1",
        "R54：detached 子 p1 不在集合（构建期并入过滤）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasInDoc").unwrap().value,
        "0",
        "R54：主文档 append 的 p2 正常入集（index 0）"
    );
}

// R54 场景 2：集合先建（含快照基线），后续 append 到 detached 容器——失效循环 add 分支
// 同样过滤（挂载点判定 false → 不并入）。R53 泄漏的精确模式。
#[test]
fn r54_invalidate_add_branch_filters_detached_growth() {
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
        "<html><body><p id='base'>one</p></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r54b".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 集合先建（els=[#base] 快照基线），再向 detached 容器 append 多轮 <p>——集合长度
    // 必须保持 1（旧实现每 append 并入 → els 净增长 → O(els) 失效级联）。
    sandbox
        .execute(
            "var doc = document;\n\
             var ps = doc.getElementsByTagName('p');\n\
             var holder = doc.createElement('div');\n\
             for (var i = 0; i < 5; i++) {\n\
               var p = doc.createElement('p');\n\
               holder.appendChild(p);\n\
             }\n\
             globalThis.__lenBefore = ps.length;\n\
             doc.body.appendChild(holder);\n\
             globalThis.__lenAfterMount = ps.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__lenBefore").unwrap().value,
        "1",
        "R54：detached append 期间集合长度不变（els 不泄漏增长）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenAfterMount").unwrap().value,
        "6",
        "R54：holder 挂入主文档后子树 <p> 全部可见（1 基线 + 5 新增）"
    );
}

// R54 场景 3：构建期 pending 并入过滤——pending 表里的 detached 容器子树节点在
// _zwMakeCollection 构建时不并入（_zwNodeParent 反链挂载点判定）。
#[test]
fn r54_build_time_pending_merge_skips_detached() {
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
        "<html><body><section id='s'></section></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r54c".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 先 append 到 detached 容器（入 pending 表），再取集合——pending 并入须按挂载点过滤。
    // 对照：同 tag 的主文档 append 正常并入。
    sandbox
        .execute(
            "var doc = document;\n\
             var foreign = doc.createElement('div');\n\
             var q1 = doc.createElement('q');\n\
             foreign.appendChild(q1);\n\
             var q2 = doc.createElement('q');\n\
             doc.getElementById('s').appendChild(q2);\n\
             var qs = doc.getElementsByTagName('q');\n\
             globalThis.__qlen = qs.length;\n\
             globalThis.__qHasForeign = Array.prototype.indexOf.call(qs, q1);\n\
             globalThis.__qHasInDoc = Array.prototype.indexOf.call(qs, q2);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__qlen").unwrap().value,
        "1",
        "R54：构建期 pending 并入过滤——detached 容器的 <q> 不进集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__qHasForeign").unwrap().value,
        "-1",
        "R54：foreign 子 q1 不在集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__qHasInDoc").unwrap().value,
        "0",
        "R54：主文档 #s 下的 q2 正常入集"
    );
}

// js-dom M4 R55：childNodes 基底缓存（按 sel 键）——同 turn 重复读消 host 往返 + 文本节点
// identity 稳定（旧行为每次 _wrapNodeEntry 重包装 → childNodes[i] !== 上次读的同位置节点，
// indexOf identity 循环依赖此相等）。失效：register_dom_callbacks 重注册（dom_html 换代）经
// globalThis._zwChildBaseInvalidateAll 全量失效。
#[test]
fn r55_childnodes_base_cache_identity_and_freshness() {
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
        "<html><body><p id='t'>hello</p></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r55".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 缓存命中：两次读同 sel childNodes，文本子 identity 稳定（缓存数组复用基底）。
    // ② 缓存内文本节点可编辑（R48 方法仍可用——缓存不冻结行为）。
    // ③ 本回合 append 的 pending 子经 overlay 可见（基底缓存不吞 pending）。
    sandbox
        .execute(
            "var p = document.getElementById('t');\n\
             var c1 = p.childNodes;\n\
             var c2 = p.childNodes;\n\
             globalThis.__ident = (c1[0] === c2[0]);\n\
             globalThis.__len0 = c1.length;\n\
             var span = document.createElement('span');\n\
             p.appendChild(span);\n\
             var c3 = p.childNodes;\n\
             globalThis.__lenAfter = c3.length;\n\
             globalThis.__hasSpan = Array.prototype.indexOf.call(c3, span);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ident").unwrap().value,
        "true",
        "R55：两次读 childNodes 的文本子 identity 稳定（基底缓存）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__len0").unwrap().value,
        "1",
        "R55：快照基底 len=1（文本子）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenAfter").unwrap().value,
        "2",
        "R55：本回合 append 经 overlay 可见（基底缓存不吞 pending）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasSpan").unwrap().value,
        "1",
        "R55：append 的 span 在 childNodes[1]（overlay 融合）"
    );
}

// R55 场景 2：重注册（dom_html 换代）→ 基底缓存全量失效，读到新快照基底。
#[test]
fn r55_reregister_invalidates_base_cache() {
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
        "<html><body><p id='t'>old</p></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r55b".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var p = document.getElementById('t');\n\
             globalThis.__firstRead = p.childNodes[0].nodeValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__firstRead").unwrap().value,
        "old",
        "R55：首读基底 = 'old'"
    );

    // 模拟 dispatch_event 重注册：dom_html 换代为新快照（mutation 已 flush 的最新 HTML）。
    let dom_html2 = Arc::new(Mutex::new(
        "<html><body><p id='t'>new</p></body></html>".to_string(),
    ));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html2, &page_url, &canvas_registry);
    sandbox
        .execute(
            "var p2 = document.getElementById('t');\n\
             globalThis.__secondRead = p2.childNodes[0].nodeValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__secondRead").unwrap().value,
        "new",
        "R55：重注册后基底缓存失效，读到新快照 'new'（旧缓存会返回 'old'）"
    );
}

// R56c（M8/DC-8）：fill(fillRule) 透传端到端——evenodd/缺省 nonzero 两形式。
// driving: 2d.path.fill.winding.evenodd.1（ctx.fill("evenodd")）/ .2（fill(path,"evenodd")）
// / 2d.path.fill.winding.add（缺省 nonzero 嵌套同向叠加）。
#[test]
fn test_fill_rule_passthrough_r56c() {
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
        "<html><body><canvas id='cv' width='100' height='50'></canvas></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r56c".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // evenodd.1 场景：绿底 + 同一矩形两遍 + fill("evenodd") → 中心保持绿（不填红）。
    sandbox.execute(
        "var cv = document.getElementById('cv');\
         var ctx = cv.getContext('2d');\
         ctx.fillStyle = '#0f0';\
         ctx.fillRect(0, 0, 100, 50);\
         ctx.beginPath();\
         ctx.rect(0, 0, 100, 50);\
         ctx.rect(0, 0, 100, 50);\
         ctx.fillStyle = '#f00';\
         ctx.fill('evenodd');\
         globalThis.__ee1 = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ee1").unwrap().value,
        "0,255,0,255",
        "fill('evenodd') 双矩形中心不填（保持绿底）——evenodd.1"
    );

    // evenodd.2 场景：Path2D 形式 fill(path, 'evenodd') 同语义。
    sandbox.execute(
        "ctx.fillStyle = '#0f0';\
         ctx.fillRect(0, 0, 100, 50);\
         var path = new Path2D();\
         path.rect(0, 0, 100, 50);\
         path.rect(0, 0, 100, 50);\
         path.closePath();\
         ctx.fillStyle = '#f00';\
         ctx.fill(path, 'evenodd');\
         globalThis.__ee2 = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ee2").unwrap().value,
        "0,255,0,255",
        "fill(path, 'evenodd') 双矩形中心不填——evenodd.2"
    );

    // winding.add 场景：缺省 nonzero 下嵌套同向矩形绕组 2 ≠ 0 → 全填（绿）。
    sandbox.execute(
        "ctx.fillStyle = '#f00';\
         ctx.fillRect(0, 0, 100, 50);\
         ctx.beginPath();\
         ctx.moveTo(-10, -10);\
         ctx.lineTo(110, -10);\
         ctx.lineTo(110, 60);\
         ctx.lineTo(-10, 60);\
         ctx.lineTo(-10, -10);\
         ctx.lineTo(0, 0);\
         ctx.lineTo(100, 0);\
         ctx.lineTo(100, 50);\
         ctx.lineTo(0, 50);\
         ctx.fillStyle = '#0f0';\
         ctx.fill();\
         globalThis.__add = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__add").unwrap().value,
        "0,255,0,255",
        "缺省 nonzero：嵌套同向绕组 2 仍填——winding.add"
    );

    // overlap 场景：半透明绿两重叠矩形一次 fill——重叠区单层 alpha 127（不叠加成 64）。
    sandbox.execute(
        "ctx.clearRect(0, 0, 100, 50);\
         ctx.fillStyle = '#000';\
         ctx.fillRect(0, 0, 100, 50);\
         ctx.fillStyle = 'rgba(0, 255, 0, 0.5)';\
         ctx.beginPath();\
         ctx.rect(0, 0, 100, 50);\
         ctx.closePath();\
         ctx.rect(10, 10, 80, 30);\
         ctx.fill();\
         globalThis.__ov = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    // WPT _assertPixelApprox 容差 1：0.5×255=127.5 的舍入取 127 或 128 都在容差内。
    let ov = sandbox.execute("globalThis.__ov").unwrap().value;
    let parts: Vec<i32> = ov.split(',').map(|v| v.parse().unwrap()).collect();
    assert!(
        parts == vec![0, 127, 0, 255] || parts == vec![0, 128, 0, 255],
        "同次 fill 重叠区单层 alpha（~127 非 64）——fill.overlap，实际 {ov}"
    );
}

// R56e（M8/DC-8）：ensuresubpath 族 + clip 相交/空 + ellipse 负半径 e2e。
// driving: 2d.path.lineTo/arcTo/bezierCurveTo/quadraticCurveTo.ensuresubpath.1/2
// + 2d.path.clip.empty/intersect + 2d.path.ellipse.basics。
#[test]
fn test_ensuresubpath_clip_ellipse_r56e() {
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
        "<html><body><canvas id='cv' width='100' height='50'></canvas></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r56e".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① lineTo 无子路径 = moveTo（stroke 不画，画布保持底色）。
    sandbox.execute(
        "var cv = document.getElementById('cv');\
         var ctx = cv.getContext('2d');\
         ctx.fillStyle = '#0f0'; ctx.fillRect(0, 0, 100, 50);\
         ctx.strokeStyle = '#f00'; ctx.lineWidth = 50;\
         ctx.beginPath(); ctx.lineTo(100, 50); ctx.stroke();\
         globalThis.__lt = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__lt").unwrap().value,
        "0,255,0,255",
        "lineTo no-subpath: canvas stays green (moveTo semantics)"
    );

    // ② quadratic 无子路径：第一控制点为起点，退化直线照画（绿线覆盖）。
    sandbox.execute(
        "ctx.fillStyle = '#f00'; ctx.fillRect(0, 0, 100, 50);\
         ctx.strokeStyle = '#0f0'; ctx.lineWidth = 50;\
         ctx.beginPath(); ctx.quadraticCurveTo(0, 25, 100, 25); ctx.stroke();\
         globalThis.__qc = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__qc").unwrap().value,
        "0,255,0,255",
        "quadratic no-subpath draws from first control point"
    );

    // ③ clip 空 + 相邻相交：全裁。
    sandbox.execute(
        "ctx.fillStyle = '#0f0'; ctx.fillRect(0, 0, 100, 50);\
         ctx.beginPath(); ctx.clip();\
         ctx.fillStyle = '#f00'; ctx.fillRect(0, 0, 100, 50);\
         globalThis.__ce = ctx.getImageData(50, 25, 1, 1).data.join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ce").unwrap().value,
        "0,255,0,255",
        "empty clip culls everything"
    );

    // ④ ellipse 负半径 → IndexSizeError；-0 与 0 合法。
    sandbox.execute(
        "var errs = [];\
         try { ctx.ellipse(10, 10, -2, 5, 0, 0, 1, false); } catch(e){ errs.push(e.name); }\
         try { ctx.ellipse(10, 10, 0, -1.5, 0, 0, 1, false); } catch(e){ errs.push(e.name); }\
         var okNegZero = true;\
         try { ctx.ellipse(10, 10, -0, 5, 0, 0, 1, false); } catch(e){ okNegZero = false; }\
         globalThis.__el = errs.join(',') + ';' + String(okNegZero);",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__el").unwrap().value,
        "IndexSizeError,IndexSizeError;true",
        "ellipse negative radii throw, -0 accepted"
    );
}

// js-dom M4 R79：Node.contains / compareDocumentPosition 全节点形态 + document 进链 +
// handle 元素兄弟导航 + 跨树 DISCONNECTED 方向位（WPT Node-contains/Node-compareDocumentPosition
// 2446F→0 双 100% 的 driving 单测）。

fn r79_sandbox() -> (zero_script_sandbox::V8Sandbox, std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>>) {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::V8Sandbox;
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><div id='host'><p id='a'>A</p></div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r79".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    (sandbox, mutations)
}

#[test]
fn r79_contains_self_descendant_and_document_chain() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.querySelector('#a');\n\
         var text = p.firstChild;\n\
         var created = document.createElement('span');\n\
         p.appendChild(created);\n\
         globalThis.__r79a = [\n\
           host.contains(host),\n\
           host.contains(p),\n\
           host.contains(text),\n\
           host.contains(created),\n\
           p.contains(host),\n\
           host.contains(null),\n\
           document.contains(p),\n\
           document.contains(created),\n\
           html_contains(),\n\
         ].join(',');\n\
         function html_contains() { return document.documentElement.contains(p); }",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r79a").unwrap().value,
        "true,true,true,true,false,false,true,true,true",
        "contains: self/descendant/text/pending-null/document 链（html.parentNode=doc R79）"
    );
}

#[test]
fn r79_compare_document_position_bitmask_family() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.querySelector('#a');\n\
         var text = p.firstChild;\n\
         var created = document.createElement('span');\n\
         p.appendChild(created);\n\
         var foreign = document.implementation.createHTMLDocument('');\n\
         var fp = foreign.createElement('p');\n\
         foreign.body.appendChild(fp);\n\
         var P = 2, F = 4, C = 8, CB = 16;\n\
         globalThis.__r79b = [\n\
           p.compareDocumentPosition(p),\n\
           p.compareDocumentPosition(text) - CB - F,\n\
           text.compareDocumentPosition(p) - C - P,\n\
           host.compareDocumentPosition(p) - CB - F,\n\
           document.compareDocumentPosition(p) - CB - F,\n\
           (function(){ var r = p.compareDocumentPosition(fp); return r - 1 - 32 === P || r - 1 - 32 === F; })(),\n\
           (function(){ var r1 = p.compareDocumentPosition(fp), r2 = fp.compareDocumentPosition(p); return (r1 & 2) !== (r2 & 2); })(),\n\
         ].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r79b").unwrap().value,
        "0,0,0,0,0,true,true",
        "compareDocumentPosition: same=0 / 祖先=CONTAINS|PRECEDING / 后代=CONTAINED_BY|FOLLOWING / 跨树带方向位且反对称"
    );
}

#[test]
fn r79_handle_element_sibling_navigation() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p1 = document.createElement('p'); p1.textContent = 'one';\n\
         var p2 = document.createElement('p'); p2.textContent = 'two';\n\
         var p3 = document.createElement('p'); p3.textContent = 'three';\n\
         host.appendChild(p1); host.appendChild(p2); host.appendChild(p3);\n\
         globalThis.__r79c = [\n\
           p2.previousSibling === p1,\n\
           p2.nextSibling === p3,\n\
           p1.previousSibling && p1.previousSibling.id,\n\
           p3.nextSibling === null,\n\
           p2.previousSibling.textContent,\n\
           p1.hasChildNodes(),\n\
           p1.firstChild.nodeValue,\n\
         ].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r79c").unwrap().value,
        "true,true,a,true,one,true,one",
        "R79：handle 元素（pending 节点）previousSibling/nextSibling 经父 childNodes 融合视图派生 + hasChildNodes 与 firstChild 视图一致"
    );
}

// js-dom M4 R80：createElementNS HTML ns 大写 tagName + validate-and-extract + 接口按 localName
// + Node 常量经原型链 + Element.nodeValue=null（WPT Document-createElementNS 0P→187P 的 driving 单测；
// 剩余 409F 全为 iframe contentDocument 深结构既存）。

#[test]
fn r80_create_element_ns_html_uppercase_and_validation() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var HTMLNS = 'http://www.w3.org/1999/xhtml';\n\
         var e1 = document.createElementNS(HTMLNS, 'span');\n\
         var e2 = document.createElementNS(HTMLNS, 'html:span');\n\
         var e3 = document.createElementNS(HTMLNS, 'SPAN');\n\
         var e4 = document.createElementNS('test', 'span');\n\
         var _exc = function(f, name) { try { f(); return 'no-throw'; } catch (e) { return e.name === name ? name : ('wrong:' + e.name); } };\n\
         globalThis.__r80a = [\n\
           e1.tagName, e1.localName, e1.prefix === null,\n\
           e2.tagName, e2.localName, e2.prefix,\n\
           e3.tagName, e3.localName,\n\
           e4.tagName, e4 instanceof HTMLElement, e4 instanceof Element,\n\
           _exc(function(){ document.createElementNS(null, 'f:oo'); }, 'NamespaceError'),\n\
           _exc(function(){ document.createElementNS(null, ':foo'); }, 'InvalidCharacterError'),\n\
           _exc(function(){ document.createElementNS(null, 'foo:'); }, 'InvalidCharacterError'),\n\
           _exc(function(){ document.createElementNS('http://example.com/', 'xml:foo'); }, 'NamespaceError'),\n\
           _exc(function(){ document.createElementNS('http://example.com/', 'xmlns:foo'); }, 'NamespaceError'),\n\
           _exc(function(){ document.createElementNS('http://example.com/', 'f:o:o'); }, 'no-throw'),\n\
         ].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r80a").unwrap().value,
        "SPAN,span,true,HTML:SPAN,span,html,SPAN,SPAN,span,false,true,NamespaceError,InvalidCharacterError,InvalidCharacterError,NamespaceError,NamespaceError,no-throw",
        "R80/R81：HTML ns createElementNS tagName 大写 / prefix·localName 原值 / 非 HTML ns 非 HTMLElement / validate-and-extract（f:o:o 有 ns 合法——R81 spec 纠正）"
    );
}

#[test]
fn r80_node_constants_and_node_value() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var e = document.createElement('div');\n\
         var ns = document.createElementNS('http://www.w3.org/1999/xhtml', 'span');\n\
         globalThis.__r80b = [\n\
           e.nodeType === e.ELEMENT_NODE,\n\
           ns.nodeType === ns.ELEMENT_NODE,\n\
           ns instanceof HTMLSpanElement,\n\
           ns.nodeValue === null,\n\
           Node.DOCUMENT_POSITION_FOLLOWING,\n\
           e.TEXT_NODE,\n\
         ].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r80b").unwrap().value,
        "true,true,true,true,4,3",
        "R80：Node 常量经实例原型链可见 + createElementNS HTML ns 接口按 localName + Element.nodeValue=null"
    );
}

// js-dom M4 R81：Document-createElement-namespace 簇 + Node-textContent 簇（contentType/ns 派生、
// Document textContent 恒 null、PI textContent=data、textContent 融合视图读 + 替换全部子语义）。

#[test]
fn r81_create_element_namespace_by_document_type() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var HTMLNS = 'http://www.w3.org/1999/xhtml';\n\
         var htmlDoc = document.implementation.createHTMLDocument('t');\n\
         var xhtmlDoc = document.implementation.createDocument(HTMLNS, 'html', null);\n\
         var xmlDoc = document.implementation.createDocument(null, 'root', null);\n\
         var svgDoc = document.implementation.createDocument('http://www.w3.org/2000/svg', 'svg', null);\n\
         var newDoc = new Document();\n\
         var parsed = new DOMParser().parseFromString('<root/>', 'text/xml');\n\
         var parsedHtml = new DOMParser().parseFromString('<html><body>x</body></html>', 'text/html');\n\
         globalThis.__r81a = [\n\
           htmlDoc.contentType, htmlDoc.createElement('p').namespaceURI,\n\
           xhtmlDoc.contentType, xhtmlDoc.createElement('p').namespaceURI,\n\
           xmlDoc.contentType, xmlDoc.createElement('p').namespaceURI === null,\n\
           svgDoc.contentType, svgDoc.createElement('p').namespaceURI === null,\n\
           newDoc.contentType, newDoc.createElement('p').namespaceURI === null,\n\
           parsed.contentType, parsed.createElement('p').namespaceURI === null,\n\
           parsedHtml.contentType, parsedHtml.createElement('p').namespaceURI,\n\
         ].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r81a").unwrap().value,
        "text/html,http://www.w3.org/1999/xhtml,application/xhtml+xml,http://www.w3.org/1999/xhtml,application/xml,true,image/svg+xml,true,application/xml,true,text/xml,true,text/html,http://www.w3.org/1999/xhtml",
        "R81：createElement ns 由文档类型派生（HTML/XHTML doc → HTML ns；XML/SVG/MathML/new Document/DOMParser-xml → null）"
    );
}

#[test]
fn r81_document_text_content_null_and_pi_text_content() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var pi = document.createProcessingInstruction('tgt', 'dat');\n\
         var host = document.querySelector('#host');\n\
         var oldTc = host.textContent;\n\
         document.textContent = 'x';\n\
         globalThis.__r81b = [\n\
           document.nodeValue === null,\n\
           document.textContent === null,\n\
           pi.textContent,\n\
           typeof Text, typeof Comment, typeof CharacterData,\n\
           oldTc,\n\
         ].join(',');\n\
         pi.textContent = 'newdat';\n\
         globalThis.__r81b2 = pi.textContent + ',' + pi.data;",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r81b").unwrap().value,
        "true,true,dat,function,function,function,A",
        "R81：Document nodeValue/textContent 恒 null + setter no-op；PI textContent=data；CharacterData 族构造器存在"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r81b2").unwrap().value,
        "newdat,newdat",
        "R81：PI textContent= 写 data（spec CharacterData 同源）"
    );
}

#[test]
fn r81_text_content_fused_read_and_replace_children() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var p = document.querySelector('#a');\n\
         var t1 = p.textContent;\n\
         var host = document.querySelector('#host');\n\
         var sub = document.createElement('span');\n\
         sub.textContent = 'DEF';\n\
         host.appendChild(sub);\n\
         var t2 = host.textContent;\n\
         var empty = document.createElement('div');\n\
         empty.appendChild(document.createTextNode(''));\n\
         empty.textContent = null;\n\
         var eKids = empty.childNodes.length;\n\
         var eFirst = empty.firstChild === null ? 'null' : String(empty.firstChild);\n\
         host.textContent = null;\n\
         var hKids = host.childNodes.length;\n\
         var subParent = sub.parentNode === null ? 'null' : String(sub.parentNode);\n\
         var literal = document.createElement('div');\n\
         literal.textContent = '<b>xyz</b>';\n\
         var litKids = literal.childNodes.length;\n\
         var litData = literal.firstChild ? literal.firstChild.data : 'null';\n\
         var litText = literal.textContent;\n\
         globalThis.__r81c = [t1, t2, eKids, eFirst, hKids, subParent, litKids, litData, litText].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r81c").unwrap().value,
        "A,ADEF,0,null,0,null,1,<b>xyz</b>,<b>xyz</b>",
        "R81：textContent 融合 childNodes 拼接读（pending 子可见）+ setter 替换全部子（空文本子也清、子 parentNode 脱钩）+ 不解析 markup"
    );
}

#[test]
fn r82_whattoshow_unsigned_and_pointer_semantics() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var p = document.querySelector('#a');\n\
         var it1 = document.createNodeIterator(p, 0xFFFFFFFF, null);\n\
         var it2 = document.createNodeIterator(p, NodeFilter.SHOW_ELEMENT, null);\n\
         var n1 = it1.nextNode();\n\
         var pb1 = it1.pointerBeforeReferenceNode;\n\
         var n2 = it2.nextNode();\n\
         var pb2 = it2.pointerBeforeReferenceNode;\n\
         var it3 = document.createNodeIterator(p, NodeFilter.SHOW_COMMENT, null);\n\
         var n3 = it3.nextNode();\n\
         var pb3 = it3.pointerBeforeReferenceNode;\n\
         globalThis.__r82a = [it1.whatToShow, n1 ? n1.nodeName : 'null', pb1, it2.whatToShow, n2 ? n2.nodeName : 'null', pb2, n3 === null ? 'null' : n3, pb3].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r82a").unwrap().value,
        "4294967295,P,false,1,P,false,null,true",
        "R82：whatToShow 无符号（0xFFFFFFFF=4294967295 非 -1）+ nextNode 命中后 pointer=false / 耗尽（null）保持 true"
    );
}

// js-dom M4 R83：walker maskFor 全 nodeType 位掩码 + TreeWalker/NodeIterator fresh 起点区分
// + previousNode 结构序逆向 + WebIDL optional undefined/null 语义 + handle 元素 before/after
// + handle 元素 innerHTML 融合序列化（WPT ChildNode-before/after、TreeWalker-acceptNode-filter
// "this value and node argument"、NodeIterator undefined/null 形态的 driving 单测）。

#[test]
fn r83_walker_mask_fresh_start_and_optional_semantics() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var root = document.createElement('div');\n\
         root.id = 'root';\n\
         var a1 = document.createElement('div'); a1.id = 'A1';\n\
         var b1 = document.createElement('div'); b1.id = 'B1';\n\
         root.appendChild(a1); a1.appendChild(b1);\n\
         // TreeWalker fresh（currentNode=root）首个 nextNode 越过 root——filter 收 A1。\n\
         var wIds = []; var wArgIds = [];\n\
         var w = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT, {\n\
           acceptNode: function(node) { wArgIds.push(node.id); return NodeFilter.FILTER_ACCEPT; }\n\
         });\n\
         var wn; while ((wn = w.nextNode())) wIds.push(wn.id);\n\
         // NodeIterator：迭代集合含 root（DIV 首位）。\n\
         var it = document.createNodeIterator(root, NodeFilter.SHOW_ELEMENT, null);\n\
         var iIds = []; var inn; while ((inn = it.nextNode())) iIds.push(inn.id || inn.tagName);\n\
         // optional undefined/null 语义。\n\
         var wU = document.createTreeWalker(root, undefined, undefined);\n\
         var wN = document.createTreeWalker(root, null, null);\n\
         globalThis.__r83a = [wIds.join('>'), wArgIds[0], iIds.join('>'), wU.whatToShow, wN.whatToShow].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r83a").unwrap().value,
        "A1>B1|A1|root>A1>B1|4294967295|0",
        "R83：TreeWalker fresh 首步越过 root（filter 收 A1）；NodeIterator 含 root；undefined→SHOW_ALL、null→0"
    );
}

#[test]
fn r83_handle_element_before_after_and_innerhtml() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var parent = document.createElement('div');\n\
         var child = document.createElement('x');\n\
         parent.appendChild(child);\n\
         child.before();\n\
         globalThis.__r83b1 = parent.innerHTML;\n\
         var p2 = document.createElement('div');\n\
         var c2 = document.createElement('x');\n\
         p2.appendChild(c2);\n\
         c2.before('text');\n\
         globalThis.__r83b2 = p2.innerHTML;\n\
         var p3 = document.createElement('div');\n\
         var c3 = document.createElement('x');\n\
         p3.appendChild(c3);\n\
         c3.after('A', 'B');\n\
         globalThis.__r83b3 = p3.innerHTML;",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r83b1").unwrap().value,
        "<x></x>",
        "R83：before() 无参 no-op + handle 父 innerHTML 反映子"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r83b2").unwrap().value,
        "text<x></x>",
        "R83：before('text') 插前兄弟文本"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r83b3").unwrap().value,
        "<x></x>AB",
        "R83：after('A','B') 插后兄弟文本（保参数序）"
    );
}

// js-dom M4 R84：sibling/CDATA 兄弟导航断链修复 + NodeIterator detach/重入守卫 +
// TreeWalker filter 返回值归一（false→REJECT 剪枝）/root 不 filter/currentNode 重定位
// effPos 区分（WPT dom/traversal NodeIterator/TreeWalker 整簇 driving 单测）。

#[test]
fn r84_sibling_text_node_navigation_chain() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.querySelector('#a');\n\
         // p 的前一兄弟：host 内 <p id=a> 前无节点（首个子）。\n\
         var prev = p.previousSibling;\n\
         // 建带文本的容器：h<div>text</div> 尾插注释 → 各节点兄弟链完整。\n\
         var c = document.createElement('div');\n\
         c.appendChild(document.createTextNode('T1'));\n\
         c.appendChild(document.createComment('C1'));\n\
         var t1 = c.firstChild;\n\
         var t1Next = t1.nextSibling;\n\
         var t1NextPrev = t1Next ? t1Next.previousSibling : null;\n\
         globalThis.__r84a = [prev === null ? 'null' : prev.nodeName,\n\
             t1.nodeName, t1.parentNode ? t1.parentNode.nodeName : 'null',\n\
             t1Next ? t1Next.nodeName : 'null',\n\
             t1NextPrev === t1 ? 'same' : 'diff'].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r84a").unwrap().value,
        "null|#text|DIV|#comment|same",
        "R84：首个子 previousSibling=null；text 节点 parentNode 指父 + 兄弟链双向可达（R3018 同款语义）"
    );
}

#[test]
fn r84_cdata_parent_link_and_iterator_detach_reentrancy() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.createElement('p');\n\
         var xmlDocument = new Document();\n\
         var cd = xmlDocument.createCDATASection('xyz');\n\
         p.appendChild(cd);\n\
         var cdParent = cd.parentNode ? cd.parentNode.nodeName : 'null';\n\
         var cdNext = cd.nextSibling;\n\
         // detach() 恒 no-op（spec：历史方法）。\n\
         var it = document.createNodeIterator(host, 0xFFFFFFFF, null);\n\
         var det = it.detach();\n\
         var detOk = det === undefined;\n\
         // filter 重入抛 InvalidStateError。\n\
         var reentered = 'no';\n\
         var it2 = document.createNodeIterator(host, NodeFilter.SHOW_ALL, function(node) {\n\
           if (reentered === 'no') { reentered = 'try'; try { it2.nextNode(); } catch (e) { reentered = e.name; } }\n\
           return NodeFilter.FILTER_ACCEPT;\n\
         });\n\
         it2.nextNode(); it2.nextNode();\n\
         globalThis.__r84b = [cdParent, cdNext === null ? 'null' : cdNext.nodeName, detOk, reentered].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r84b").unwrap().value,
        "P|null|true|InvalidStateError",
        "R84：CDATA append 后 parentNode=P + 无兄弟 null；detach no-op 返 undefined；filter 重入抛 InvalidStateError"
    );
}

#[test]
fn r84_treewalker_filter_false_rejects_and_root_unfiltered() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         // filter 返 false（→0，归一 REJECT）：root 的 firstChild 深入受 REJECT 剪枝。\n\
         var w1 = document.createTreeWalker(host, NodeFilter.SHOW_ALL, function() { return false; });\n\
         var fc1 = w1.firstChild();\n\
         var after1 = w1.currentNode === host ? 'host' : (w1.currentNode ? w1.currentNode.nodeName : 'null');\n\
         // root 不 filter：filter 只拒非 # 节点时 firstChild 深入到首个 # 文本。\n\
         var para = document.querySelector('#a');\n\
         var w2 = document.createTreeWalker(para, NodeFilter.SHOW_ALL, function(n) { return n.nodeName[0] === '#'; });\n\
         var fc2 = w2.firstChild();\n\
         // 重定位到被滤节点（show ELEMENT 但 currentNode 指向文本）→ nextSibling 走结构序。\n\
         var w3 = document.createTreeWalker(host, NodeFilter.SHOW_ELEMENT, null);\n\
         var txt = document.querySelector('#a').firstChild;\n\
         w3.currentNode = txt;\n\
         var ns3raw = w3.nextSibling();\n\
         var ns3 = ns3raw ? String(ns3raw.nodeName) : 'null';\n\
         globalThis.__r84c = [fc1 === null ? 'null' : fc1.nodeName, after1,\n\
             fc2 ? fc2.nodeName : 'null', ns3].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r84c").unwrap().value,
        "null|host|#text|null",
        "R84：filter 返 false 按 REJECT 剪枝（firstChild null 且 currentNode 不动）；root 不被 filter（#filter 可达子树文本）；重定位被滤节点 effPos=-1 走结构序"
    );
}

// js-dom M4 R85：TreeWalker 层级方法导航式重写 + previousNode 规范镜像 +
// html.previousSibling=doctype（WPT TreeWalker-basic/traversal-skip/reject/skip-most
// + TreeWalker.html previousSibling document 簇的 driving 单测）。

#[test]
fn r85_treewalker_navigational_hierarchy_and_prevnode() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var root = document.createElement('div');\n\
         var a = document.createElement('div'); a.id = 'a';\n\
         root.appendChild(a);\n\
         var b = document.createTextNode('b'); a.appendChild(b);\n\
         var c = document.createElement('div'); c.id = 'c'; a.appendChild(c);\n\
         var d = document.createElement('div'); d.id = 'd'; c.appendChild(d);\n\
         var e = document.createTextNode('e'); d.appendChild(e);\n\
         var j = document.createComment('j'); c.appendChild(j);\n\
         // WPT TreeWalker-basic: walker root = a div, not the outer container.\n\
         var w = document.createTreeWalker(a);\n\
         var pn0 = w.parentNode();\n\
         var fc = w.firstChild();\n\
         var ns = w.nextSibling();\n\
         var lc = w.lastChild();\n\
         var ps = w.previousSibling();\n\
         var nn = w.nextNode();\n\
         var pn = w.parentNode();\n\
         var prevN = w.previousNode();\n\
         var ns2 = w.nextSibling();\n\
         function s(n) { return n === null ? 'null' : (n.nodeType + ':' + (n.id || n.nodeValue)); }\n\
         globalThis.__r85a = [s(pn0), s(fc), s(ns), s(lc), s(ps), s(nn), s(pn), s(prevN), s(ns2)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r85a").unwrap().value,
        "null|3:b|1:c|8:j|1:d|3:e|1:d|1:c|null",
        "R85：TreeWalker-basic Walk over nodes 全序列（导航式层级方法 + previousNode 规范镜像）"
    );
}

#[test]
fn r85_treewalker_reject_prunes_prevnode_and_sibling_climb() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var root = document.createElement('div'); root.id = 'root';\n\
         var a1 = document.createElement('div'); a1.id = 'A1';\n\
         var b1 = document.createElement('div'); b1.id = 'B1';\n\
         var c1 = document.createElement('div'); c1.id = 'C1';\n\
         var b2 = document.createElement('div'); b2.id = 'B2';\n\
         var b3 = document.createElement('div'); b3.id = 'B3';\n\
         root.appendChild(a1); a1.appendChild(b1); b1.appendChild(c1);\n\
         a1.appendChild(b2); a1.appendChild(b3);\n\
         var rejB1 = { acceptNode: function(n) { return n.id === 'B1' ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT; } };\n\
         var w = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT, rejB1);\n\
         w.currentNode = b3;\n\
         var p1 = w.previousNode();\n\
         var p2 = w.previousNode();\n\
         // skip-most：SKIP 的父站不止单步爬升（B1→B3 期望）。\n\
         var root2 = document.createElement('div');\n\
         var x1 = document.createElement('div'); x1.id = 'X1';\n\
         var y1 = document.createElement('div'); y1.id = 'Y1'; y1.className = 'keep';\n\
         var y2 = document.createElement('div'); y2.id = 'Y2';\n\
         var y2t = document.createTextNode('matters');\n\
         var y3 = document.createElement('div'); y3.id = 'Y3'; y3.className = 'keep';\n\
         root2.appendChild(x1); x1.appendChild(y1); x1.appendChild(y2); y2.appendChild(y2t); x1.appendChild(y3);\n\
         var keep = { acceptNode: function(n) { return n.className === 'keep' ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP; } };\n\
         var w2 = document.createTreeWalker(root2, NodeFilter.SHOW_ELEMENT, keep);\n\
         w2.currentNode = root2;\n\
         var f1 = w2.firstChild();\n\
         var f1next = f1 ? w2.nextSibling() : null;\n\
         function s(n) { return n === null || n === undefined ? 'null' : (n.id || n.nodeValue || n.nodeName); }\n\
         globalThis.__r85b = [s(p1), s(p2), s(f1), s(f1next)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r85b").unwrap().value,
        "B2|A1|Y1|Y3",
        "R85：REJECT 剪 previousNode 子树（B3→B2→A1 跳过 B1/C1）；skip-most nextSibling 父站仅 ACCEPT 止（Y1→Y3）"
    );
}

#[test]
fn r85_html_sibling_doctype() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var htmlEl = document.documentElement;\n\
         var prev = htmlEl.previousSibling;\n\
         var prevType = prev ? prev.nodeType : 'null';\n\
         var next = htmlEl.nextSibling;\n\
         var dt = document.doctype;\n\
         var dtNext = dt && dt.nextSibling ? dt.nextSibling.nodeName : 'null';\n\
         globalThis.__r85c = [String(prevType), next === null ? 'null' : next.nodeName, dtNext].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r85c").unwrap().value,
        "10|null|HTML",
        "R85：html.previousSibling=doctype(10)、nextSibling=null；doctype.nextSibling=HTML（真浏览器语义，oracle expected 一致性前提）"
    );
}

// js-dom M4 R86：detached 子树保留其子 + NodeIterator 移除 retarget（WPT
// NodeIterator-removal 簇 driving 单测——remove 后 firstChild 可读 + 引用重定位）。

#[test]
fn r86_detached_subtree_children_and_iterator_retarget() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.createElement('p');\n\
         host.appendChild(p);\n\
         var t = document.createTextNode('inner');\n\
         p.appendChild(t);\n\
         var iter = document.createNodeIterator(p);\n\
         iter.nextNode(); iter.nextNode(); // root + text，ref=text、before=false\n\
         var refBefore = iter.referenceNode.nodeName;\n\
         var refIsText = iter.referenceNode === t;\n\
         // remove t（root=p 的子，ref===removed）：retarget 前驱 = t 的父 p（spec\n\
         // nodeiterator-remove：指针后置 → reference = removed 的树序前驱）。\n\
         p.removeChild(t);\n\
         var fcAfter = p.firstChild ? p.firstChild.nodeName : 'null';\n\
         var refAfter = iter.referenceNode ? iter.referenceNode.nodeName : 'null';\n\
         var refIsP = iter.referenceNode === p;\n\
         globalThis.__r86a = [refBefore, String(refIsText), fcAfter, refAfter, String(refIsP)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r86a").unwrap().value,
        "#text|true|null|P|true",
        "R86：remove 子 text 后 p.firstChild=null + 迭代器 ref 从被移除 text retarget 到前驱（父 p）"
    );
}

#[test]
fn r86_reappend_clears_removed_marker() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var c1 = document.createElement('span'); c1.id = 'k1';\n\
         var c2 = document.createElement('span'); c2.id = 'k2';\n\
         var c3 = document.createElement('span'); c3.id = 'k3';\n\
         host.appendChild(c1); host.appendChild(c2); host.appendChild(c3);\n\
         function count() {\n\
           var it = document.createNodeIterator(host);\n\
           var cnt = 0; var n; while ((n = it.nextNode())) cnt++;\n\
           return cnt;\n\
         }\n\
         var cnt1 = count();\n\
         host.removeChild(c2);\n\
         var cnt2 = count();\n\
         host.appendChild(c2);\n\
         var cnt3 = count();\n\
         globalThis.__r86b = [String(cnt1), String(cnt2), String(cnt3)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r86b").unwrap().value,
        "6|5|6",
        "R86：移除后迭代器跳过该节点（6→5），re-append 清标记恢复（→6）"
    );
}

// js-dom M4 R87：文本子 remove/restore 二次周期（WPT NodeIterator-removal 恢复段——
// 元素 remove/restore 后其 text 子再 remove 仍须 retarget；旧 guard 查注册表恒 miss 静默 no-op）。

#[test]
fn r87_text_child_second_removal_cycle_retargets() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.createElement('p');\n\
         host.appendChild(p);\n\
         p.textContent = 'inner';\n\
         // 第一周期：remove/restore 元素 p（子树注销——textContent 注册文本随之消失，\n\
         // 恢复后子视图来自物化缓存）。\n\
         var op = p.parentNode, os = p.nextSibling;\n\
         op.removeChild(p);\n\
         op.insertBefore(p, os);\n\
         // 第二周期：remove p 的 text 子（无注册条目——须走物化缓存路径）。\n\
         var t = p.firstChild;\n\
         var iter = document.createNodeIterator(p);\n\
         iter.nextNode(); iter.nextNode(); // ref=text、before=false\n\
         var refIsText = iter.referenceNode === t;\n\
         p.removeChild(t);\n\
         var fcAfter = p.firstChild ? p.firstChild.nodeName : 'null';\n\
         var refIsP = iter.referenceNode === p;\n\
         globalThis.__r87a = [String(refIsText), fcAfter, String(refIsP)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r87a").unwrap().value,
        "true|null|true",
        "R87：restore 后二次 remove text 子仍 retarget（ref→父 p）且父视图剔除 removed"
    );
}

#[test]
fn r87_previous_node_before_false_returns_ref_via_filter() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var p = document.createElement('p');\n\
         host.appendChild(p);\n\
         var t = document.createTextNode('inner');\n\
         p.appendChild(t);\n\
         var iter = document.createNodeIterator(p);\n\
         iter.nextNode(); iter.nextNode(); // ref=text、before=false\n\
         // spec previousNode：pointer-before=false → 翻 before=true、返当前 ref（过 filter）。\n\
         var back = iter.previousNode();\n\
         var backIsText = back === t;\n\
         var beforeNow = iter.pointerBeforeReferenceNode;\n\
         // before=true 后再 previousNode → 树序前驱（text 前是 root p → 返 p；再前 null）。\n\
         var back2 = iter.previousNode();\n\
         var back2IsP = back2 === p;\n\
         var back3 = iter.previousNode();\n\
         globalThis.__r87b = [String(backIsText), String(beforeNow), String(back2IsP), String(back3 === null)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r87b").unwrap().value,
        "true|true|true|true",
        "R87：previousNode 的 before=false 半边返当前 ref 并翻指针；再前返 root、耗尽 null"
    );
}

// js-dom M4 R88：filter 执行中移除节点——pre-remove 步骤对 in-flight 遍历位置生效
//（WPT NodeIterator-removal-during-filtering：返回被 filter 节点 + reference retarget）。

#[test]
fn r88_filter_removes_candidate_retargets_reference() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var root = document.createElement('div');\n\
         var a = document.createElement('a-el');\n\
         var b = document.createElement('b-el');\n\
         var c = document.createElement('c-el');\n\
         host.appendChild(root);\n\
         root.appendChild(a); root.appendChild(b); root.appendChild(c);\n\
         var it = document.createNodeIterator(root, 0x1, {\n\
         \x20 acceptNode: function (node) { if (node === b) b.remove(); return 1; }\n\
         });\n\
         it.nextNode(); it.nextNode(); // root, a\n\
         var returned = it.nextNode();   // filter(b) 内 remove(b)\n\
         var returnedIsB = returned === b;\n\
         var refIsA = it.referenceNode === a;\n\
         var nextIsC = it.nextNode() === c;\n\
         globalThis.__r88a = [String(returnedIsB), String(refIsA), String(nextIsC)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r88a").unwrap().value,
        "true|true|true",
        "R88：filter 内移除候选——返回被 filter 节点 b、reference retarget 到 a、遍历继续到 c"
    );
}

#[test]
fn r88_filter_removes_ancestor_in_flight_previous() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var root = document.createElement('div');\n\
         var a = document.createElement('a-el');\n\
         var a1 = document.createElement('a1-el');\n\
         var b = document.createElement('b-el');\n\
         var b1 = document.createElement('b1-el');\n\
         a.appendChild(a1); b.appendChild(b1);\n\
         host.appendChild(root); root.appendChild(a); root.appendChild(b);\n\
         var armed = false;\n\
         var it = document.createNodeIterator(root, 0x1, {\n\
         \x20 acceptNode: function (node) { if (armed && node === b1) b.remove(); return 1; }\n\
         });\n\
         for (var i = 0; i < 5; i++) it.nextNode(); // ref=b1、before=false\n\
         armed = true;\n\
         var returned = it.previousNode(); // filter(b1) 内 remove(b)——in-flight=b1 在 b 子树\n\
         var returnedIsB1 = returned === b1;\n\
         var refIsA1 = it.referenceNode === a1;\n\
         var beforeNow = it.pointerBeforeReferenceNode;\n\
         globalThis.__r88b = [String(returnedIsB1), String(refIsA1), String(beforeNow)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r88b").unwrap().value,
        "true|true|false",
        "R88：previousNode 的 filter 内移除祖先——返回 b1、reference retarget a1、指针翻 false"
    );
}

// js-dom M4 R89：TreeWalker currentNode setter 纯赋值（不跑 filter）+ previousNode
// ACCEPT 有子先入子树尾（filtered 序前驱——WPT previousNodeLastChildReject /
// TreeWalker "Recursive filters need to throw"）。

#[test]
fn r89_setter_does_not_run_filter_recursive_throws_on_traverse() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var depth = 0;\n\
         var walker;\n\
         var setterThrew = false;\n\
         walker = document.createTreeWalker(document, 0xFFFFFFFF, function () {\n\
         \x20 if (depth === 0) { depth++; try { walker.firstChild(); } catch (e) { globalThis.__inner = e.name; } }\n\
         \x20 return 1;\n\
         });\n\
         try { walker.currentNode = document.body; } catch (e) { setterThrew = true; }\n\
         var innerName = globalThis.__inner || 'none';\n\
         // setter 后首个遍历方法：filter 重入（depth 已 1 不重入——真浏览器 filter 从未跑过；\n\
         // 本实现物化窗口在首遍历方法内，重入检测经 active flag 生效）。\n\
         var depthAfterSet = depth;\n\
         globalThis.__r89a = [String(setterThrew), innerName, String(depthAfterSet)].join('|');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r89a").unwrap().value,
        "false|none|0",
        "R89：currentNode setter 纯赋值——不跑 filter（depth 0）、不抛异常"
    );
}

#[test]
fn r89_previous_node_accept_with_children_digs_last() {
    let (mut sandbox, _mutations) = r79_sandbox();
    sandbox.execute(
        "var host = document.querySelector('#host');\n\
         var root = document.createElement('div');\n\
         host.appendChild(root);\n\
         var a1 = document.createElement('div'); a1.id = 'A1';\n\
         var b1 = document.createElement('div'); b1.id = 'B1';\n\
         var b2 = document.createElement('div'); b2.id = 'B2';\n\
         var c1 = document.createElement('div'); c1.id = 'C1';\n\
         var c2 = document.createElement('div'); c2.id = 'C2';\n\
         root.appendChild(a1); a1.appendChild(b1); a1.appendChild(b2);\n\
         b1.appendChild(c1); b1.appendChild(c2);\n\
         var walker = document.createTreeWalker(root, 1, function (n) { return n.id === 'C2' ? 2 : 1; });\n\
         walker.firstChild();                 // A1\n\
         walker.nextNode();                   // B1\n\
         walker.nextNode();                   // C1\n\
         walker.nextNode();                   // B2（C2 被拒跳过）\n\
         var pv = walker.previousNode();      // 期望 C1（B1 ACCEPT 有子 → 先入子树尾；C2 拒 → C1）\n\
         globalThis.__r89b = (pv && pv.id) + '|' + walker.currentNode.id;",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r89b").unwrap().value,
        "C1|C1",
        "R89：previousNode 的 ACCEPT 有子先 dig 子树尾（filtered 序前驱），childless 才返"
    );
}

#[test]
fn r79_parent_element_of_document_element_is_null_but_parent_node_is_document() {
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
        "<html><body><div id='host'>text</div></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/r79-parentelement".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ZRG-2026-08-17（zeroweb-regression-guard）：R79 把 html 的 parentNode 设为 document 后，
    // parentNode/parentElement 共用 _parentNodeFor 使 documentElement.parentElement 错误返回
    // document——parity 采集器沿 parentElement 上行到 html 后走进 document，node.tagName 为
    // undefined → toLowerCase 崩溃。spec：parentElement 只返元素父（html 的父是 Document 非元素
    // → null）；parentNode 保持 document（R79 contains/compareDocumentPosition 依赖）。
    sandbox
        .execute(
            "var de = document.documentElement;\n\
             globalThis.__r1 = (de.parentElement === null);\n\
             globalThis.__r2 = (de.parentNode === document);\n\
             globalThis.__r3 = (document.body.parentElement === de);\n\
             var chain = [];\n\
             for (var node = document.body; node; node = node.parentElement) chain.push(node.tagName);\n\
             globalThis.__r4 = chain.join('>');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "true",
        "documentElement.parentElement 必须为 null（spec dom-node-parentelement，非元素父）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "true",
        "documentElement.parentNode 保持 document（R79 contains/compareDocumentPosition 前提）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r3").unwrap().value,
        "true",
        "body.parentElement === documentElement（正常元素父链不受影响）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r4").unwrap().value,
        "BODY>HTML",
        "parentElement 链从 body 上行止于 html（不进 document），selectorFor 类遍历不再崩溃"
    );
}


// js-dom M3 R96：`_REFLECTED_UINT[prop]` 裸下标查表把 Object.prototype 继承名
//（hasOwnProperty/valueOf/toLocaleString/isPrototypeOf/propertyIsEnumerable）当命中
//（truthy 函数）→ `parseInt(entry.a=undefined)`=NaN → `return entry.d=undefined` 提前
// 返回，R93 原型链回落不可达——CE 升级元素上 `el.hasOwnProperty`/`el.valueOf` 等读
// undefined（lit ReactiveElement 的 hasOwnProperty 探测、Object.prototype 方法以元素为
// receiver 的调用全部中断）。修复：own-property 判定。本测试断言修复后六名全可达
//（typeof function + 调用语义正确 + expando 读写不受扰）。
#[test]
fn test_object_prototype_methods_reachable_on_element_r96() {
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            r#"
class MyEl extends HTMLElement {
  constructor() { super(); this._mark = 'ctor'; }
  bump() { return 42; }
}
customElements.define('my-el', MyEl);
var el = document.createElement('my-el');
// 六个 Object.prototype 继承名 + valueOf 调用语义（返元素自身非 undefined→字符串化走 toString 分支）。
globalThis.__r1 = typeof el.hasOwnProperty;
globalThis.__r2 = typeof el.propertyIsEnumerable;
globalThis.__r3 = typeof el.valueOf;
globalThis.__r4 = typeof el.toLocaleString;
globalThis.__r5 = typeof el.isPrototypeOf;
globalThis.__r6 = String(el.hasOwnProperty('_mark'));
globalThis.__r7 = String(el.hasOwnProperty('nope'));
// expando（ctor 内赋值）读写不受扰。
globalThis.__r8 = el._mark;
el.enableUpdating = function() {};
globalThis.__r9 = typeof el.enableUpdating;
// reflected-uint 真命中不受扰（colSpan 缺省 1，spec default）。
var td = document.createElement('td');
globalThis.__r10 = String(td.colSpan);
"#,
        )
        .unwrap();
    let out = sandbox
        .execute(
            r#"['__r1','__r2','__r3','__r4','__r5','__r6','__r7','__r8','__r9','__r10']
.map(function(n){ return String(globalThis[n]); }).join('|')"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "function|function|function|function|function|true|false|ctor|function|1",
        "R96：Object.prototype 继承名经 get trap 原型链回落可达（own-property 查表修复），\
        hasOwnProperty 调用语义正确，expando 与 reflected-uint 真命中不受扰"
    );
}

// ── js-dom M1 L2 R102：查询回调 live Document 读路径 ──
//
// 行为契约（live 读正确性——性能收益是 opportunistic，行为等价是 land 门禁）：
// 1. publish live 后查询读 live：对 live doc 的直改（不经 mutation 队列——模拟 native
//    绑定直改路径）查询可见；
// 2. 未 publish（None）查询回落快照路径（引擎测试/reftest 等直接调用方零行为变化）；
// 3. pending InsertAdjacentHtml 时（live_ok=false）查询走 pending 应用视图（R57 语义：
//    同批 insertAdjacentHTML 后 querySelector 可见——live 读不得提前绕过）。
#[test]
fn test_live_query_doc_read_path_r102() {
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
        "<html><body><div id=\"host\"><p id=\"snap-only\">snap</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 基线：未 publish live——快照路径查询可见快照元素。
    let out = sandbox
        .execute("String(document.querySelector('#snap-only') !== null)")
        .unwrap().value;
    assert_eq!(out, "true", "R102 基线：快照路径查询可见");

    // 1. publish 一个 live doc（内容 = 快照 + live 独有元素）——live 读应看到 live 独有。
    let live = zero_dom::parse_html(
        "<html><body><div id=\"host\"><p id=\"snap-only\">snap</p><span id=\"live-only\">live</span></div></body></html>",
    );
    crate::js_dom_bridge::publish_live_query_doc(Some(std::rc::Rc::new(std::cell::RefCell::new(live))));
    let out = sandbox
        .execute(
            "String(document.querySelector('#live-only') !== null) + '|' \
             + String(document.querySelector('#snap-only') !== null)",
        )
        .unwrap().value;
    assert_eq!(
        out, "true|true",
        "R102 live 读：live 独有元素可见 + 快照元素仍在（live 是超集）"
    );

    // 2. publish None——回落快照路径：live 独有元素不再可见。
    crate::js_dom_bridge::publish_live_query_doc(None);
    let out = sandbox
        .execute("String(document.querySelector('#live-only') !== null)")
        .unwrap().value;
    assert_eq!(out, "false", "R102 回落：None 后快照路径，live 独有不可见");

    // 3. pending InsertAdjacentHtml——R57 语义保持：同批 insertAdjacentHTML 后查询可见
    //    （走 pending 应用视图而非 live——live 未含该未应用 mutation）。
    crate::js_dom_bridge::publish_live_query_doc(None); // 干净 live 状态
    sandbox
        .execute(
            "var h = document.getElementById('host');\
             h.insertAdjacentHTML('beforeend', '<b id=\"adj\">adj</b>');\
             globalThis.__adjVisible = String(document.querySelector('#adj') !== null);",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__adjVisible").unwrap().value;
    assert_eq!(
        out, "true",
        "R102 pending 语义：同批 insertAdjacentHTML 后查询可见（R57 回归守卫）"
    );

    crate::js_dom_bridge::publish_live_query_doc(None);
}

// ── js-dom M4 R105：passive-by-default（spec HTML default-passive-value）──
//
// window/document/documentElement/body 四类 target 的 touchstart/touchmove/wheel/
// mousewheel listener 未显式 passive 时默认 passive（preventDefault no-op）；
// {passive:false} 显式关闭；非 passive-by-default 元素（div）与非默认事件（touchend）
// 默认非 passive。WPT dom/events/passive-by-default.html 驱动（100P/0F）。
#[test]
fn test_passive_by_default_r105() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            r#"(function(){
  var results = [];
  function probe(targetName, type, opts) {
    var t = targetName === 'window' ? window : (targetName === 'document' ? document
      : (targetName === 'html' ? document.documentElement : (targetName === 'body' ? document.body
      : document.getElementById('d'))));
    var prevented = null;
    var h = function (e) { e.preventDefault(); prevented = e.defaultPrevented; };
    t.addEventListener(type, h, opts);
    t.dispatchEvent(new Event(type, { cancelable: true }));
    t.removeEventListener(type, h, opts);
    results.push(targetName + ':' + type + ':' + (opts === undefined ? 'omit' : JSON.stringify(opts)) + '=' + String(prevented));
  }
  probe('window', 'touchstart');           // 默认 passive → false
  probe('window', 'touchstart', { passive: false }); // 显式非 → true
  probe('window', 'wheel');                 // 默认 passive → false
  probe('document', 'touchmove');           // 默认 passive → false
  probe('html', 'mousewheel');              // documentElement 默认 passive → false
  probe('body', 'touchstart');              // body 默认 passive → false
  probe('div', 'touchstart');               // 非 pd target → true
  probe('window', 'touchend');              // 非默认类型 → true
  probe('window', 'touchstart', { passive: true });  // 显式 passive → false
  probe('window', 'touchstart', { passive: undefined }); // undefined = 未指定 → false
  return results.join('|');
})()"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "window:touchstart:omit=false|window:touchstart:{\"passive\":false}=true|window:wheel:omit=false\
         |document:touchmove:omit=false|html:mousewheel:omit=false|body:touchstart:omit=false\
         |div:touchstart:omit=true|window:touchend:omit=true|window:touchstart:{\"passive\":true}=false\
         |window:touchstart:{}=false",
        "R105 passive-by-default 四类 target × 显式/缺省矩阵"
    );
}

// ── js-dom M4 R106：dispatchEvent 入口语义（spec dom-eventtarget-dispatchevent）──
//
// ① event 非 Event（null）→ TypeError；② createEvent 未 initEvent（initialized flag
// 未设）→ InvalidStateError；③ 派发中重入（dispatch flag）→ InvalidStateError；
// ④ listener 抛错不传播（后续 listener 仍跑、dispatchEvent 返 true）。
// WPT dom/events/EventTarget-dispatchEvent.html 驱动（24F→1F，剩 1F 为 handle 树
// 祖先派发深结构记档）。
#[test]
fn test_dispatch_event_entry_semantics_r106() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            r#"(function(){
  var results = [];
  var catchOf = function (f) {
    try { f(); return 'no-throw'; } catch (e) { return e && e.name ? e.name : String(e); }
  };
  // ① dispatchEvent(null) → TypeError（document 与元素两入口）。
  results.push('null-doc:' + catchOf(function () { document.dispatchEvent(null); }));
  results.push('null-el:' + catchOf(function () { document.getElementById('d').dispatchEvent(null); }));
  // ② createEvent 未 initEvent → InvalidStateError；initEvent 后正常（返 true）。
  var ev = document.createEvent('Event');
  results.push('uninit:' + catchOf(function () { document.dispatchEvent(ev); }));
  ev.initEvent('x', false, false);
  results.push('inited:' + String(document.dispatchEvent(ev)));
  // ③ 派发中重入 → InvalidStateError（listener 内再 dispatch 同一 event）。
  var ev2 = document.createEvent('Event');
  ev2.initEvent('y', false, false);
  var reentry = 'not-run';
  document.addEventListener('y', function () {
    reentry = catchOf(function () { document.dispatchEvent(ev2); });
  });
  document.dispatchEvent(ev2);
  results.push('reentry:' + reentry);
  // ④ listener 抛错不传播（第二个 listener 仍跑 + 返 true）。
  var called = [];
  var d = document.getElementById('d');
  d.addEventListener('z', function () { called.push('first'); throw new Error('boom'); });
  d.addEventListener('z', function () { called.push('second'); });
  var ret = 'err-prop';
  try { ret = String(d.dispatchEvent(new Event('z'))); } catch (e) { ret = 'threw:' + e.name; }
  results.push('listener-err:' + ret + ':' + called.join(','));
  return results.join('|');
})()"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "null-doc:TypeError|null-el:TypeError|uninit:InvalidStateError|inited:true\
         |reentry:InvalidStateError|listener-err:true:first,second",
        "R106 dispatchEvent 入口四语义（TypeError/未初始化/重入/异常不传播）"
    );
}
