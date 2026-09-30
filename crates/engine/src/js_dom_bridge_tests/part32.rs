// js_dom_bridge 测试切片 32（t7 / js-dom P15 修复：innerHTML 解析 plain 视图上的
// appendChild 不落 host）。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与 part01-31 共享模块作用域。新切片先例：part16/part31。
//
// 根因（设计卡 .acceptance/site-optimizer/html5test-20260929-r1/evidence/t7-design-card.md）：
// `el.innerHTML = html` 的本地解析视图（firstChild 链）是 `_zwMEl` plain 代理（无
// `__zwSelector`/`__zwHandle`），对这种 plain 父 appendChild(handleChild) 只改 JS 世界
// childNodes + R180 plainParent 反链，不向 host 发任何 mutation → live Document 永久缺
// 该子树：getElementById/querySelectorAll/渲染全部不可见（html5test.co ResultsTable
// 建表形态，hideChildren 的 getElementById null 崩掉 showResults）。
// 修复双件：① plain appendChild 发「锚祖先 + child-index 路径」host mutation
// （AppendChildByHandlePath / AppendChildBySelPath）；② `_zwMutationInDoc` 爬 plainParent
// 链，使同 turn getElementById 的 findPendingId 兜底可见（spec
// https://dom.spec.whatwg.org/#concept-node-pre-insert ：插入后节点必须在树中）。
// 无直接上游 WPT 用例（shim 双世界架构为 ZeroWeb 特有；WPT dom 层用例走原生
// dom_bindings 路径不经 shim plain 世界），按仓库流程补等价本地测试并记原因。

// ── ① Rust apply 层：路径寻址 append 变体 ──

#[test]
fn test_apply_append_child_by_sel_path_t7() {
    // SetInnerHtml 解析出的子（host child 0）作路径父，append handle 子落 host。
    let html = "<html><body><div id=\"host\"></div></body></html>";
    let mutations = vec![
        DomMutation::SetInnerHtml {
            selector: "#host".into(),
            html: "<div></div>".into(),
        },
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "div".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "cat-t7".into(),
        },
        DomMutation::AppendChildBySelPath {
            parent_selector: "#host".into(),
            path: vec![0],
            child_handle: "__n1".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    // row 落在解析出的 plain div 内（锚 #host → path[0] → parsed div → append）。
    assert!(
        out.contains("<div><div id=\"cat-t7\">"),
        "cat 应嵌在 #host 的解析子 div 内\n{out}"
    );
}

#[test]
fn test_apply_append_child_by_handle_path_t7() {
    // 锚为 create 句柄（html5test.co 实况形态：left=createElement 句柄，
    // container=left.innerHTML 解析子，cat append 到 container）。
    let html = "<html><body><div id=\"host\"></div></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__anc".into(),
            tag: "div".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#host".into(),
            child_handle: "__anc".into(),
        },
        DomMutation::SetInnerHtmlOnHandle {
            handle: "__anc".into(),
            html: "<div></div>".into(),
        },
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "div".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__n1".into(),
            name: "id".into(),
            value: "row-t7".into(),
        },
        DomMutation::AppendChildByHandlePath {
            parent_handle: "__anc".into(),
            path: vec![0],
            child_handle: "__n1".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(
        out.contains("<div><div id=\"row-t7\">"),
        "row 应嵌在 __anc 的解析子 div 内\n{out}"
    );
}

#[test]
fn test_apply_append_child_by_path_lenient_miss_t7() {
    // 锚/路径解析失败 → lenient no-op（R125 口径：host 与 JS 解析视图分歧不中止整批；
    // 子 handle 悬垂仍硬错）。
    let html = "<html><body><div id=\"host\"></div></body></html>";
    // ① path 越界（#host 只有 0 个子）。
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__n1".into(),
            tag: "div".into(),
        },
        DomMutation::AppendChildBySelPath {
            parent_selector: "#host".into(),
            path: vec![5],
            child_handle: "__n1".into(),
        },
    ];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert_eq!(
        out, "<html><head></head><body><div id=\"host\"></div></body></html>",
        "越界路径应 lenient 跳过（树不变）\n{out}"
    );
    // ② selector 锚 miss。
    let mutations = vec![DomMutation::AppendChildBySelPath {
        parent_selector: "#nope".into(),
        path: vec![0],
        child_handle: "__n1".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert_eq!(
        out, "<html><head></head><body><div id=\"host\"></div></body></html>",
        "锚 miss 应 lenient 跳过（树不变）\n{out}"
    );
    // ③ 子 handle 悬垂 → 硬错（与 AppendChildByHandle 同口径）。
    let mutations = vec![DomMutation::AppendChildBySelPath {
        parent_selector: "#host".into(),
        path: vec![],
        child_handle: "__ghost".into(),
    }];
    assert!(
        apply_mutations_to_html(html, &mutations).is_err(),
        "子 handle 悬垂应硬错"
    );
}

// ── ② shim e2e：P15 崩溃形态（同 turn gEBI + 落 host） ──

#[test]
fn test_shim_plain_parent_remove_host_sync_t7() {
    // 对偶面：plain 父 append（落 host）→ removeChild（须同步摘 host）。修复前 append
    // 不落 host 无此问题；只修 append 不修 remove 会留幽灵节点（gEBI/渲染仍可见）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"results\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var host = document.getElementById('results');\
             host.innerHTML = '<div></div>';\
             var container = host.firstChild;\
             var row = document.createElement('div');\
             row.id = 'row-rm';\
             container.appendChild(row);\
             globalThis.__midAfterAppend = document.getElementById('row-rm');\
             container.removeChild(row);\
             globalThis.__midAfterRemove = document.getElementById('row-rm');\
             globalThis.__removedParentNull = row.parentNode === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__midAfterAppend !== null)").unwrap().value,
        "true",
        "append 后同 turn gEBI 应命中"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__midAfterRemove === null)").unwrap().value,
        "true",
        "remove 后同 turn gEBI 应为 null（幽灵节点回归位）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__removedParentNull)").unwrap().value,
        "true",
        "remove 后 parentNode 应置空"
    );

    // 落 host：apply 后 #results 子树不含 row-rm（append 与 remove 在 host 对冲）。
    let queue = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()), &queue)
        .unwrap();
    assert!(
        !out.contains("row-rm"),
        "append+remove 应在 host 完全对冲，不留幽灵节点\n{out}"
    );
}

#[test]
fn test_shim_plain_parent_append_host_visible_t7() {
    // 端到端复现 html5test.co ResultsTable 形态：
    // host.innerHTML 解析子（plain 视图）上 appendChild 两个 handle 子——
    // id 先设（category 形态，走 _zwPendingAddedById）与 id 后设（row 形态，
    // 走 findPendingId 实时 .id 扫描）。修复前：同 turn getElementById 恒 null。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"results\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var host = document.getElementById('results');\
             host.innerHTML = '<div></div>';\
             var container = host.firstChild;\
             /* 形态 A：id 先于 append（createCategories） */\
             var cat = document.createElement('div');\
             cat.id = 'cat-p15';\
             container.appendChild(cat);\
             /* 形态 B：id 后于 append（createItems） */\
             var row = document.createElement('div');\
             container.appendChild(row);\
             row.id = 'row-p15';\
             /* 同 turn 读取（hideChildren 崩溃时点） */\
             globalThis.__midCat = document.getElementById('cat-p15');\
             globalThis.__midRow = document.getElementById('row-p15');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__midCat !== null && globalThis.__midCat === cat)").unwrap().value,
        "true",
        "同 turn gEBI 应命中 id 先设形态且为同一 proxy（修复前 null）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__midRow !== null && globalThis.__midRow === row)").unwrap().value,
        "true",
        "同 turn gEBI 应命中 id 后设形态（findPendingId 路径，修复前 null）"
    );

    // 落 host：mutation 队列 apply 到初始 html → 解析子 div 内可见两子（渲染/gEBI host 权威层）。
    let queue = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()), &queue)
        .unwrap();
    let icat = out.find("<div id=\"cat-p15\">");
    let irow = out.find("<div id=\"row-p15\">");
    assert!(icat.is_some() && irow.is_some(), "两子都应落 host\n{out}");
    // 都嵌在 #host 的解析子 div 内：位于 `<div id="results">` 之后的第一个裸 `<div>` 之内。
    let ihost = out.find("<div id=\"results\">").unwrap();
    assert!(
        icat.unwrap() > ihost && irow.unwrap() > ihost,
        "两子应位于 #results 子树内\n{out}"
    );
    assert!(
        out.contains("<div><div id=\"cat-p15\">") && out.contains("<div id=\"cat-p15\"></div><div id=\"row-p15\">"),
        "两子应嵌在解析子 div 内且保序\n{out}"
    );
}

// ── ③ 返修轮回归（defect D1/D2，review/t7-defect-r1.json） ──

#[test]
fn test_shim_innerhtml_leading_ws_path_align_t7() {
    // D1：innerHTML 前导空白 markup 下 host trim 解析 vs 本地视图原文解析错位——
    // 修复前 child-index 整体 +1：walk 错落下一兄弟（a 内 append 落进 b）或越界
    // lenient 静默丢（末元素形态）。修复后视图用 trim 后串构建，与 host 逐子对齐。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"results\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var host = document.getElementById('results');\
             host.innerHTML = '\\n<div class=\"a\"></div>\\n<div class=\"b\"></div>';\
             var b = host.childNodes[host.childNodes.length - 1];\
             var cat = document.createElement('div');\
             cat.id = 'ws-cat';\
             b.appendChild(cat);\
             globalThis.__wsGEBI = document.getElementById('ws-cat') !== null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__wsGEBI)").unwrap().value,
        "true",
        "前导空白 markup 下同 turn gEBI 应命中（findPendingId 兜底）"
    );

    // 落 host：ws-cat 必须落在 .b 内。修复前视图多计前导文本子 → index 整体 +1，
    // walk 越界 lenient 静默丢（ws-cat 不出现在 host）。
    let queue = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()), &queue)
        .unwrap();
    let b_seg = out
        .split("<div class=\"b\">")
        .nth(1)
        .unwrap_or("")
        .split("</div>")
        .next()
        .unwrap_or("");
    assert!(
        b_seg.contains("ws-cat"),
        "ws-cat 应落在 .b 内（前导空白不产生索引错位/静默丢弃）\n{out}"
    );
}

#[test]
fn test_shim_plain_mid_remove_no_handle_ghost_t7() {
    // D2：plain 容器内部 t7 落 host 的 handle 后代（深度 ≥2），plain 中间节点摘除时
    // 必须同步摘 host——修复前对冲只认直接 handle 子（无 handle 早退不递归），深层
    // 后代残留 host → apply 后 gEBI 幽灵命中（id_map 未清）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"results\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var host = document.getElementById('results');\
             host.innerHTML = '<div class=\"list\"><div class=\"item\"></div></div>';\
             var list = host.firstChild;\
             var item = list.firstChild;\
             var row = document.createElement('div');\
             row.id = 'row-deep';\
             item.appendChild(row);\
             globalThis.__deepBefore = document.getElementById('row-deep') !== null;\
             list.removeChild(item);\
             globalThis.__deepAfter = document.getElementById('row-deep') === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__deepBefore)").unwrap().value,
        "true",
        "plain 中间节点内 append 后同 turn gEBI 应命中"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__deepAfter)").unwrap().value,
        "true",
        "plain 中间节点摘除后同 turn gEBI 应为 null（handle 后代已标记移除）"
    );

    // 落 host：apply 后序列化不含 row-deep（深层 handle 随 plain 摘除同步出 host）。
    let queue = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()), &queue)
        .unwrap();
    assert!(
        !out.contains("row-deep"),
        "plain 中间节点摘除须把 t7 挂载的 handle 后代同步摘出 host\n{out}"
    );
}
