// slice43（live 集合成员维护两缺口收口）：
// ①slice32 集合面 parsed 子树移除成员维护缺口（slice42 残余申报·xS-3 降格补证）：
//   `_zwHCCollectSubtree` 对 sel 父仅回落 pending 桶 added（R51c），parsed 后代不入
//   remFlat → live 集合/查询面成员 stale（slice40 补偿扫仅覆盖动态账本 `_zwNADynEls`
//   面）。本钉以常驻形态实证自然红（此前仅过程观察未归档）。
// ②slice42 缺陷轮 I-5：attr 维护面连接性门——detached 元素 setAttribute 经
//   `_zwNAAttrChanged` 混入文档级 NA 集合（join 臂无 `_zwDocContains36` 门）。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
// https://dom.spec.whatwg.org/#concept-collection（HTMLCollection live 语义）

/// slice43 钉共用沙箱装配（s42_sandbox 同款：shim + 快照 + DOM 回调）。
macro_rules! s43_sandbox {
    ($html:expr) => {{
        use std::sync::{Arc, Mutex};
        use zero_script_sandbox::{Sandbox, V8Sandbox};

        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
        let dom_html = Arc::new(Mutex::new($html.to_string()));
        let page_url = Arc::new(Mutex::new("https://zero.test/s43".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①parsed 子树移除：快照解析产出的容器（sel 父）含 NA 集合成员（双 name= 命中 →
// live 集合）与 tag 集合成员；整树移除后集合成员应随树离场（spec：named objects /
// HTMLCollection 均限当下 document tree，https://dom.spec.whatwg.org/#concept-collection-live）。
// 修前 remFlat 展开对 sel 父仅回落 pending 桶（R51c），parsed 后代不入 remFlat：
// held NA 集合/tag 集合成员残留（stale），fresh tag 查询（`_zwDocAllElements` 快照面
// ∪ pending-added，剔除面走 pendingRemoved）同样残留。对照形态（script 建子树，
// handle 展开路径）不回退。
#[test]
fn named_access_collection_parsed_subtree_remove_s43() {
    let mut sandbox = s43_sandbox!(
        "<html><body><div id='w43'><img name='p43' id='a43'><img name='p43' id='b43'></div></body></html>"
    );
    sandbox
        .execute(
            "var na = window.p43;
             var tag = document.getElementsByTagName('img');
             globalThis.__r_na0 = na.length;
             globalThis.__r_tag0 = tag.length;
             document.body.removeChild(document.getElementById('w43'));
             globalThis.__r_gone = String(document.getElementById('w43') === null);
             globalThis.__r_na_after = na.length;
             globalThis.__r_tag_held = tag.length;
             globalThis.__r_tag_fresh = document.getElementsByTagName('img').length;
             var box = document.createElement('div');
             var c1 = document.createElement('img');
             c1.setAttribute('name', 'p43c');
             var c2 = document.createElement('img');
             c2.setAttribute('name', 'p43c');
             box.appendChild(c1);
             box.appendChild(c2);
             document.body.appendChild(box);
             globalThis.__r_ctl0 = window.p43c.length;
             document.body.removeChild(box);
             globalThis.__r_ctl_after = String(typeof window.p43c === 'undefined');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_na0").unwrap().value,
        "2",
        "双 name=p43 安装 live 集合（安装面基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_tag0").unwrap().value,
        "2",
        "tag 集合基线 2（快照面成员）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_gone").unwrap().value,
        "true",
        "parsed 容器已离文档树（sel 父自身入 pendingRemoved，id 查询面即时）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_na_after").unwrap().value,
        "0",
        "held NA 集合随子树移除清空（spec：named objects 限当下 document tree；修前 stale 2）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_tag_held").unwrap().value,
        "0",
        "held tag 集合随子树移除清空（HTMLCollection live；修前 stale 2）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_tag_fresh").unwrap().value,
        "0",
        "fresh tag 查询不含已移除 parsed 后代（pendingRemoved 剔除面；修前 stale 2）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_ctl0").unwrap().value,
        "2",
        "对照形态：script 建双成员动态安装集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_ctl_after").unwrap().value,
        "true",
        "对照形态：handle 子树批删 0 命中回收不回退（remFlat 展开面天然覆盖，slice42 口径）"
    );
}

// ②attr 维护面连接性门：detached 元素 setAttribute('name') 命中既有 NA 集合名，
// spec 口径不得并入（named objects 限 document tree——WPT basics "not reachable"
// 面，与 `_zwNAAttrDynamicSync` 动态注册面 `_zwDocContains36` 门同口径）。修前
// `_zwNAAttrChanged` join 臂无连接性门 → detached 成员混入文档级集合。对照形态：
// in-doc 元素 setAttribute 命中名仍并入（R54/R333 口径收窄不误伤）。
#[test]
fn named_access_attr_join_detached_gate_s43() {
    let mut sandbox = s43_sandbox!(
        "<html><body><img name='q43' id='qa43'><img name='q43' id='qb43'><img id='qc43'></body></html>"
    );
    sandbox
        .execute(
            "globalThis.__r_base = window.q43.length;
             var d = document.createElement('img');
             d.setAttribute('name', 'q43');
             globalThis.__r_detached = window.q43.length;
             document.getElementById('qc43').setAttribute('name', 'q43');
             globalThis.__r_indoc_join = window.q43.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_base").unwrap().value,
        "2",
        "双 name=q43 安装 live 集合（安装面基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_detached").unwrap().value,
        "2",
        "detached 元素 setAttribute 命中名不入文档级集合（spec：named objects 限 document tree；修前混入为 3）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_indoc_join").unwrap().value,
        "3",
        "对照形态：in-doc 元素 attr 命中名仍并入（连接性门不误伤 attr 面 join）"
    );
}
