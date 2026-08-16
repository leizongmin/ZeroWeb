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
           _exc(function(){ document.createElementNS('http://example.com/', 'f:o:o'); }, 'NamespaceError'),\n\
         ].join(',');",
    ).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r80a").unwrap().value,
        "SPAN,span,true,HTML:SPAN,span,html,SPAN,SPAN,span,false,true,NamespaceError,InvalidCharacterError,InvalidCharacterError,NamespaceError,NamespaceError,NamespaceError",
        "R80：HTML ns createElementNS tagName 大写 / prefix·localName 原值 / 非 HTML ns 非 HTMLElement / validate-and-extract 全规则"
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
