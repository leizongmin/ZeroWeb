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

// ── canvas 接口全局急切注册（R49xx，siteopt r2/t3） ──

#[test]
fn test_canvas_ctr_global_eager_registration() {
    // R49xx：CanvasRenderingContext2D 全局构造器此前仅由 _zwMakeCtx2d 首调懒创建，
    // 页面首次 getContext('2d') 前 `typeof CanvasRenderingContext2D` 恒 'undefined'、
    // 直接引用抛 ReferenceError → typeof 门控的 canvas 特性检测误判不支持
    //（html5test canvas.context：`canvas.getContext && typeof CanvasRenderingContext2D
    // != 'undefined' && canvas.getContext('2d') instanceof CanvasRenderingContext2D`）。
    // spec：支持某接口的 realm 上接口对象随全局对象暴露。
    // https://webidl.spec.whatwg.org/#es-interfaces
    // 本测断言（均不预先 getContext）：① typeof = 'function'；② 全局不可枚举覆盖面
    // （引用可用 + prototype 不可写）；③ instanceof 通路（首次 getContext 即链上）；
    // ④ 与懒注册等价性（getContext 前 typeof 与 after 一致）。
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // ① shim 装载后（无任何 getContext 调用）typeof 即为 'function'。
    sandbox
        .execute("globalThis.__typeofBefore = String(typeof CanvasRenderingContext2D);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__typeofBefore").unwrap().value,
        "function",
        "首次 getContext 前 typeof CanvasRenderingContext2D 应为 'function'（急切注册）"
    );

    // ② 引用可用 + prototype 属性不可写/不可删（与懒注册同规格）。
    sandbox
        .execute(
            "globalThis.__nameOk = String(CanvasRenderingContext2D.name);\
             var desc = Object.getOwnPropertyDescriptor(CanvasRenderingContext2D, 'prototype');\
             globalThis.__protoLocked = String(desc && !desc.writable && !desc.configurable);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__nameOk").unwrap().value,
        "CanvasRenderingContext2D",
        "全局引用可用（此前抛 ReferenceError）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__protoLocked").unwrap().value,
        "true",
        "prototype 属性不可写/不可删（spec 接口对象规格）"
    );

    // ③ html5test 判定式全通路：同一表达式在零预热 realm 上应为 true。
    sandbox
        .execute(
            "var c = document.createElement('canvas');\
             globalThis.__html5testStyle = String(!!(c.getContext && typeof CanvasRenderingContext2D != 'undefined' && c.getContext('2d') instanceof CanvasRenderingContext2D));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__html5testStyle").unwrap().value,
        "true",
        "html5test 式 typeof 门控检测应判定支持"
    );

    // ④ 懒注册兜底等价性：急切注册后 _zwMakeCtx2d 不重复定义（幂等）——
    // 捕获 getContext 前的构造器引用，断言 ctx 原型恒等该引用
    //（若懒块守卫被删导致重复定义，instanceof/typeof 仍真，唯引用恒等可抓）。
    sandbox
        .execute(
            "globalThis.__ctorBefore = CanvasRenderingContext2D;\
             var ctx = document.createElement('canvas').getContext('2d');\
             globalThis.__ctxInstanceof = String(ctx instanceof CanvasRenderingContext2D);\
             globalThis.__ctorStable = String(typeof CanvasRenderingContext2D === 'function');\
             globalThis.__ctorIdentity = String(Object.getPrototypeOf(ctx) === CanvasRenderingContext2D.prototype && CanvasRenderingContext2D === globalThis.__ctorBefore);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ctxInstanceof").unwrap().value,
        "true",
        "getContext 返回的 ctx 应 instanceof 急切注册的全局构造器"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ctorStable").unwrap().value,
        "true",
        "getContext 后构造器仍稳定存在（幂等兜底不覆盖）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ctorIdentity").unwrap().value,
        "true",
        "ctx 原型应恒等 getContext 前捕获的构造器引用（无重复定义）"
    );
}

// siteopt slice24：parsed CharacterData 子（初始 HTML 解析出的注释/文本）的 removeChild
// 全链——shim 视图移除 + host `RemoveChildAt` mutation 真移除。
// 判别史：baidu SSR `s-data:` 注释经 san prelude `parentNode.removeChild(n)` 移除失效
// （旧四移除分支只认 handle/sel 身份 → 静默穿透）→ 水合 walker 把注释当当前节点 →
// "Element type not match, expect 1 but 8" → 聊天输入组件 boot 中断 → sugrec 双通道 0。
// 浏览器级最小复现（min-repro）在 base 4f0ef846f RED：A_initialParsed/B_innerHTML
// 双案 removeChild 返回后子节点数不变（RED 依据归档 diag/evidence/slice24/
// boot-hook-bh1-9docErr.json、boot-hook-bh2-clean.json、verdict-base-s24j.json；
// 钉 RED→GREEN 复跑日志 review-te-pin-red-green-repro.log）；
// Chrome 同页全 GREEN。本单测钉同一语义的 shim→host 全链。
// https://dom.spec.whatwg.org/#dom-node-removechild
// https://dom.spec.whatwg.org/#concept-node-pre-remove
#[test]
fn test_parsed_characterdata_remove_child_s24() {
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
        "<html><body><div id=\"host\"><!--pc--><b id=\"b\">x</b></div>\
         <p id=\"p\">hello</p></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var host = document.getElementById('host');\
             var c = host.firstChild;\
             globalThis.__cType = c.nodeType;\
             globalThis.__cData = String(c.data);\
             var ret = host.removeChild(c);\
             globalThis.__retIdentity = ret === c;\
             globalThis.__cParentNull = c.parentNode === null;\
             globalThis.__kidCount = host.childNodes.length;\
             globalThis.__fcIsB = host.firstChild && host.firstChild.id === 'b';\
             globalThis.__noContain = !host.contains(c);\
             globalThis.__ih = host.innerHTML;\
             var again = 'none';\
             try { host.removeChild(c); } catch (e) { again = e.name; }\
             globalThis.__reRemove = again;\
             var p = document.getElementById('p');\
             var t = p.firstChild;\
             globalThis.__tType = t.nodeType;\
             p.removeChild(t);\
             globalThis.__pKids = p.childNodes.length;\
             globalThis.__pFcNull = p.firstChild === null;",
        )
        .unwrap();
    // shim 视图：注释识别 + 移除生效（旧形态：kidCount 仍 2、firstChild 仍是注释）。
    assert_eq!(
        sandbox.execute("String(globalThis.__cType + ':' + globalThis.__cData)").unwrap().value,
        "8:pc",
        "初始解析注释应包装为 nodeType 8 / data 'pc' 的 CharacterData 子"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__retIdentity + ':' + globalThis.__cParentNull)").unwrap().value,
        "true:true",
        "removeChild 应返被移除节点且 parentNode 置空（spec concept-node-pre-remove）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__kidCount + ':' + globalThis.__fcIsB + ':' + globalThis.__noContain)").unwrap().value,
        "1:true:true",
        "移除后融合视图应只剩元素子（childNodes/firstChild/contains 同步）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ih").unwrap().value,
        "<b id=\"b\">x</b>",
        "innerHTML 序列化应不再含注释"
    );
    assert_eq!(
        sandbox.execute("globalThis.__reRemove").unwrap().value,
        "NotFoundError",
        "重复移除：节点已非父的子（融合视图已剔除）→ NotFoundError（spec pre-remove 步骤 1-2，与 R126 校验族一致）"
    );
    // 邻近边界：parsed 文本子移除同语义（融合视图 childNodes/firstChild 同步）。
    // 注：sel 父的 textContent getter 直读 host（apply 窗口内 stale）——与 R125 元素子
    // 移除同款既有限制（innerHTML 有 R380 融合门、textContent 无），本切片不加宽；
    // baidu 场景不受影响（spec dom-node-textcontent：注释不计入 textContent）。
    assert_eq!(
        sandbox.execute("String(globalThis.__tType + ':' + globalThis.__pKids + ':' + globalThis.__pFcNull)").unwrap().value,
        "3:0:true",
        "parsed 文本子移除后融合视图应为空（childNodes/firstChild）"
    );

    // host 落地：队列含 RemoveChildAt（#host 注释 idx 0 + #p 文本 idx 0），apply 后
    // 注释/文本真消失、元素子保留。
    let queue = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let rm_count = queue
        .iter()
        .filter(|m| matches!(m, DomMutation::RemoveChildAt { child_index: 0, .. }))
        .count();
    assert_eq!(rm_count, 2, "两次 CharacterData 移除应各排队一条 RemoveChildAt");
    let out = apply_mutations_to_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()), &queue)
        .unwrap();
    assert!(
        !out.contains("<!--pc-->") && !out.contains("pc"),
        "apply 后 host 文档应真移除注释\n{out}"
    );
    assert!(
        out.contains("id=\"b\""),
        "元素子应保留\n{out}"
    );
    assert!(
        !out.contains("hello"),
        "apply 后 host 文档应真移除文本子\n{out}"
    );
}

// slice24 邻近钉：innerHTML 解析 plain 元素的 getAttributeNode/getAttributeNodeNS
//（spec https://dom.spec.whatwg.org/#dom-element-getattributenode 、
// https://dom-element-getattributenodens）。jQuery Sizzle attrHandle.id 优先调
// `elem.getAttributeNode('id')` 读 nodeValue——旧 plain 工厂缺方法抛 TypeError
//（baidu 首页 hydration 链 reject → sugrec 通道死）。与元素 proxy R122（part04）
// 同语义：Attr 真实例（instanceof Attr、ownerElement 指回）、miss 返 null、value
// 写回经 setAttribute 传播到 attrs 数组。无直接上游 WPT 用例（shim plain 世界为
// ZeroWeb 特有架构，理由同上），补等价本地钉。
#[test]
fn test_plain_parsed_get_attribute_node_s24() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var d = document.getElementById('body') || document.body;\
             d.innerHTML = '<span id=\"sp\" class=\"c1\" xlink:title=\"xt\">t</span>';\
             var sp = d.firstChild;\
             globalThis.__isFn = typeof sp.getAttributeNode === 'function' && typeof sp.getAttributeNodeNS === 'function';\
             var an = sp.getAttributeNode('id');\
             globalThis.__anOk = !!an && an.nodeType === 2 && an.name === 'id' && an.value === 'sp' && an.nodeValue === 'sp';\
             globalThis.__isAttr = !!an && an instanceof Attr;\
             globalThis.__ownerOk = !!an && an.ownerElement === sp;\
             globalThis.__missNull = sp.getAttributeNode('nope') === null && sp.getAttributeNodeNS(null, 'nope') === null;\
             globalThis.__ciOk = sp.getAttributeNode('ID') !== null;\
             an.value = 'sp2';\
             globalThis.__writeBack = sp.getAttribute('id') === 'sp2';\
             var xtn = sp.getAttributeNodeNS('http://www.w3.org/1999/xlink', 'title');\
             globalThis.__nsOk = !!xtn && xtn.value === 'xt' && xtn.localName === 'title';\
             globalThis.__nsMiss = sp.getAttributeNodeNS('http://www.w3.org/1999/xlink', 'other') === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isFn)").unwrap().value,
        "true",
        "plain 解析元素应有 getAttributeNode/getAttributeNodeNS 方法（spec dom-element-getattributenode）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__anOk + ':' + globalThis.__isAttr + ':' + globalThis.__ownerOk)").unwrap().value,
        "true:true:true",
        "getAttributeNode 应返 Attr 真实例（nodeType 2、name/value/nodeValue、ownerElement 指回）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__missNull + ':' + globalThis.__ciOk)").unwrap().value,
        "true:true",
        "miss 返 null；非 NS 变体大小写不敏感（R116 HTML 小写语义）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__writeBack)").unwrap().value,
        "true",
        "Attr.value 写回应经 setAttribute 传播到 attrs 数组（R122 setter 共享路径）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nsOk + ':' + globalThis.__nsMiss)").unwrap().value,
        "true:true",
        "getAttributeNodeNS 按 (ns, local) 定位（xlink prefix→ns 映射，R190 同源）"
    );
}

// slice24 邻近钉：getComputedStyle 的 display UA 默认回落（CSS 层叠第 2 步 UA 声明
// 兜底——https://drafts.csswg.org/css-cascade/#cascading ；UA stylesheet
// https://html.spec.whatwg.org/multipage/rendering.html#the-css-user-agent-style-sheet-and-presentational-hints ）。
// 旧形：host 只覆盖 sel 注册元素，plain 元素（innerHTML 解析产物 / createElement 未
// 落 host）查 display 返 ''——jQuery 1.x css_defaultDisplay 以非空判定跳过 iframe 兜底，
// '' 逼入 iframe 分支（plain 世界 iframe 无同步 contentWindow）抛 TypeError（baidu his
// suggest 初始化链断，sugrec 通道死）。无直接上游 WPT 用例（shim 双世界特有），补等价本地钉。
#[test]
fn test_computed_style_ua_default_display_s24() {
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
        "<html><body><div id=\"hd\" style=\"display:none\">h</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var sp = document.createElement('span');\
             document.body.appendChild(sp);\
             var dv = document.createElement('div');\
             document.body.appendChild(dv);\
             var ce = document.createElement('my-widget');\
             document.body.appendChild(ce);\
             globalThis.__sp = getComputedStyle(sp).display;\
             globalThis.__dv = getComputedStyle(dv).display;\
             globalThis.__ce = getComputedStyle(ce).display;\
             globalThis.__spPv = getComputedStyle(sp).getPropertyValue('display');\
             dv.style.display = 'none';\
             globalThis.__dvNone = getComputedStyle(dv).display;\
             globalThis.__hd = getComputedStyle(document.getElementById('hd')).display;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__sp").unwrap().value,
        "inline",
        "plain 新建 span 的计算 display 应为 UA 默认 inline（旧 '' 逼入 iframe 兜底分支）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dv").unwrap().value,
        "block",
        "plain 新建 div 的计算 display 应为 UA 默认 block"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ce").unwrap().value,
        "inline",
        "未知元素（custom element）缺省 inline（CSS2.1 UA sheet 兜底）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__spPv").unwrap().value,
        "inline",
        "getPropertyValue('display') 同语义"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dvNone").unwrap().value,
        "none",
        "inline style display 优先于 UA 默认（层叠序 inline > UA）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hd").unwrap().value,
        "none",
        "sel 注册元素 host 计算值仍优先（inline style='display:none' → none，host 路径不受回落影响）"
    );
}

// slice24 收尾轮（TE1+F4）：同父连续多删的桶移位修正钉。pin1（上）两案异父且各删
// idx 0——RemoveChildAt 的移位递减（part04 removeChild parsed-CharacterData 第四分支：
// 按桶 removed 中基底位于本子之前的兄弟数递减 host 索引）零覆盖。本钉同父连删两注释子：
// 正向臂先删基底 0 再删基底 2 → 队列索引 [0, 1]（第二删递减 1——apply 批序下基底 0
// 已真移除）；反向臂先删基底 2 再删基底 0 → [2, 0]（removed 基底 2 不位于 0 前，
// 禁递减——防「按 removed 总数无差递减」的过头形态；负向回归即队列出现 -1/错位，
// apply 解析 usize 失败硬错）。队列索引是契约断言：naive 基底直传正向臂得 [0, 2]
//（apply 误删基底 1 元素子）。
// https://dom.spec.whatwg.org/#concept-node-pre-remove
#[test]
fn test_parsed_characterdata_same_parent_multi_remove_shift_s24() {
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
        "<html><body>\
         <div id=\"h1\"><!--f0--><b id=\"a1\">1</b><!--f1--><b id=\"a2\">2</b></div>\
         <div id=\"h2\"><!--r0--><b id=\"b1\">1</b><!--r1--><b id=\"b2\">2</b></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var h1 = document.getElementById('h1');\
             var f0 = h1.firstChild;\
             globalThis.__f0Type = f0.nodeType;\
             globalThis.__f0Data = String(f0.data);\
             var f1 = h1.childNodes[2];\
             globalThis.__f1Type = f1.nodeType;\
             var r1 = h1.removeChild(f0);\
             globalThis.__r1Ok = r1 === f0 && f0.parentNode === null;\
             globalThis.__h1Kids1 = h1.childNodes.length;\
             globalThis.__f1Shift = h1.childNodes[1] === f1;\
             var r2 = h1.removeChild(f1);\
             globalThis.__r2Ok = r2 === f1 && f1.parentNode === null;\
             globalThis.__h1Kids2 = h1.childNodes.length;\
             globalThis.__h1Ih = h1.innerHTML;\
             var h2 = document.getElementById('h2');\
             var r0 = h2.firstChild;\
             var r1c = h2.childNodes[2];\
             var q1 = h2.removeChild(r1c);\
             globalThis.__q1Ok = q1 === r1c && r1c.parentNode === null;\
             globalThis.__h2Kids1 = h2.childNodes.length;\
             var q2 = h2.removeChild(r0);\
             globalThis.__q2Ok = q2 === r0 && r0.parentNode === null;\
             globalThis.__h2Kids2 = h2.childNodes.length;\
             globalThis.__h2Ih = h2.innerHTML;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__f0Type + ':' + globalThis.__f0Data + ':' + globalThis.__f1Type)")
            .unwrap()
            .value,
        "8:f0:8",
        "同父两注释子应识别为 CharacterData（nodeType 8、data 保留）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__r1Ok + ':' + globalThis.__r2Ok)")
            .unwrap()
            .value,
        "true:true",
        "同父连续两次 removeChild 应各返被移除节点且 parentNode 置空"
    );
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__h1Kids1 + ':' + globalThis.__f1Shift + ':' + globalThis.__h1Kids2)"
            )
            .unwrap()
            .value,
        "3:true:2",
        "首次移除后融合视图 3 子且 f1 前移至 idx 1，再次移除后 2 子"
    );
    assert_eq!(
        sandbox.execute("globalThis.__h1Ih").unwrap().value,
        "<b id=\"a1\">1</b><b id=\"a2\">2</b>",
        "正向臂两注释移除后 innerHTML 应只剩两元素子"
    );
    assert_eq!(
        sandbox
            .execute(
                "String(globalThis.__q1Ok + ':' + globalThis.__q2Ok + ':' + globalThis.__h2Kids1 + ':' + globalThis.__h2Kids2)"
            )
            .unwrap()
            .value,
        "true:true:3:2",
        "反向臂（先基底 2 后基底 0）同语义：各返被移除节点、融合视图 3→2"
    );
    assert_eq!(
        sandbox.execute("globalThis.__h2Ih").unwrap().value,
        "<b id=\"b1\">1</b><b id=\"b2\">2</b>",
        "反向臂两注释移除后 innerHTML 应只剩两元素子"
    );

    // host 落地：队列索引契约断言——正向臂 [0,1]（移位递减生效）、反向臂 [2,0]
    //（不位于本子之前的 removed 禁递减）。
    let queue = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let idx_of = |sel: &str| -> Vec<usize> {
        queue
            .iter()
            .filter_map(|m| match m {
                DomMutation::RemoveChildAt { parent_selector, child_index }
                    if parent_selector == sel =>
                {
                    Some(*child_index)
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(
        idx_of("#h1"),
        vec![0, 1],
        "正向臂队列索引应 [0,1]：第二删按基底 0 已移除递减 1（naive 基底直传得 [0,2] 误删元素子）"
    );
    assert_eq!(
        idx_of("#h2"),
        vec![2, 0],
        "反向臂队列索引应 [2,0]：removed 基底 2 不位于 0 前，禁过头递减（无差递减回归即 -1 硬错）"
    );
    let out = apply_mutations_to_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()), &queue)
        .unwrap();
    assert!(
        !out.contains("f0") && !out.contains("f1") && !out.contains("r0") && !out.contains("r1"),
        "apply 后两父的四条注释应真移除\n{out}"
    );
    assert!(
        out.contains("id=\"a1\"")
            && out.contains("id=\"a2\"")
            && out.contains("id=\"b1\"")
            && out.contains("id=\"b2\""),
        "四条元素子应全保留（移位不误删）\n{out}"
    );
}

// slice25 翻转钉（原 slice24 收尾轮 TE2+F1 known-deviation 现状钉，同提交翻转兑现
// 问题池 F1）：getAttributeNodeNS 无前缀属性 ns 语义——spec
// https://dom.spec.whatwg.org/#concept-attribute-namespace 属性 ns 在创建时定死，
// HTML 解析产物无前缀属性 namespace=null：getAttributeNodeNS(HTML-ns, 'id') 应返
// null（slice24 现状误命中返 Attr 的「无前缀属性 ∈ 元素 ns」分支已移除——part03
// _zwMEl 与同工厂 getAttributeNS 的 _zwMNsMatch（entry.ns 缺省 null）语义对齐）。
// (null,'id') 命中面（__hitOk/__hitFields/__hitOwner——slice24 钉的判别命中共相）
// 翻转后保持 GREEN。负控制：① 有前缀属性 xlink:href 命中面不变（R190 prefix→ns
// 映射）且 (null,'href') 不命中；② sel 世界（proxy R122 实例层，ns 显式元数据）
// (null,name) 命中不受本翻转扰动。
#[test]
fn test_plain_parsed_get_attribute_node_ns_no_prefix_null_s24_flip() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var d = document.getElementById('body') || document.body;\
             d.innerHTML = '<span id=\"sp\" class=\"c1\">t</span>';\
             var sp = d.firstChild;\
             var nid = sp.getAttributeNodeNS(null, 'id');\
             globalThis.__hitOk = !!nid && nid instanceof Attr;\
             globalThis.__hitFields = !!nid && nid.name === 'id' && nid.value === 'sp' && nid.localName === 'id' && nid.prefix === null && nid.namespaceURI === null;\
             globalThis.__hitOwner = !!nid && nid.ownerElement === sp;\
             globalThis.__nsHtml = sp.getAttributeNodeNS('http://www.w3.org/1999/xhtml', 'id') === null;\
             sp.setAttribute('xlink:href', 'u');\
             var pf = sp.getAttributeNodeNS('http://www.w3.org/1999/xlink', 'href');\
             globalThis.__pfHit = !!pf && pf.prefix === 'xlink' && pf.namespaceURI === 'http://www.w3.org/1999/xlink' && pf.localName === 'href' && pf.value === 'u';\
             globalThis.__pfNullMiss = sp.getAttributeNodeNS(null, 'href') === null;\
             var px = document.createElement('div');\
             px.setAttribute('foo', 'bar');\
             var sna = px.getAttributeNodeNS(null, 'foo');\
             globalThis.__selHit = !!sna && sna.value === 'bar';",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hitOk + ':' + globalThis.__hitFields + ':' + globalThis.__hitOwner)")
            .unwrap()
            .value,
        "true:true:true",
        "(null,'id') 应命中已存在属性并返 Attr 真实例（name/value/localName、prefix=null、namespaceURI=null——spec concept-attribute-namespace 无前缀属性 ns 为 null）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nsHtml)").unwrap().value,
        "true",
        "getAttributeNodeNS(HTML-ns, 'id') 应返 null（spec concept-attribute-namespace 无前缀属性 ns=null——slice24 known-deviation F1 已翻转）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__pfHit + ':' + globalThis.__pfNullMiss + ':' + globalThis.__selHit)")
            .unwrap()
            .value,
        "true:true:true",
        "负控制：有前缀属性 (xlink-ns,'href') 命中不变且 (null,'href') 不命中（R190 prefix→ns 映射）；sel 世界 proxy (null,'foo') 命中不受翻转扰动"
    );
}

// slice25 翻转钉（原 slice24 收尾轮 TE2+F2 known-deviation 现状钉，同提交翻转兑现
// 问题池 F2）：_zwUaDisplay UA 默认表 li/canvas 回归 HTML 渲染 UA sheet 标准值
//（https://html.spec.whatwg.org/multipage/rendering.html#the-css-user-agent-style-sheet-and-presentational-hints ）：
// li → 'list-item'（`li { display: list-item; }`）；canvas 不在 UA sheet block 集，
// replaced 元素回落 display 初始值 'inline'（part01 _zwUaDisplay：li 专支 + canvas
// 移出 block 表落 inline 兜底）。双臂：proxy createElement（sel 世界，host miss 后
// 落 UA 表）与 parsed innerHTML（plain 世界）同值。负控制：div/ul 仍 block（block
// 集其余值不受翻转扰动）；inline style 优先序与 host 路径由交付钉
// test_computed_style_ua_default_display_s24 覆盖。'list-item'/'inline' 均非
// 'none'/非空串——jQuery css_defaultDisplay 消费面不触发 iframe 兜底（slice24
// 修复回归面不受扰）。
#[test]
fn test_ua_display_li_list_item_canvas_inline_s24_flip() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var li = document.createElement('li');\
             document.body.appendChild(li);\
             var cv = document.createElement('canvas');\
             document.body.appendChild(cv);\
             globalThis.__li = getComputedStyle(li).display;\
             globalThis.__cv = getComputedStyle(cv).display;\
             document.body.innerHTML = '<li></li><canvas></canvas><div></div><ul></ul>';\
             var kids = document.body.childNodes;\
             globalThis.__liP = getComputedStyle(kids[0]).display;\
             globalThis.__cvP = getComputedStyle(kids[1]).display;\
             globalThis.__ctrl = getComputedStyle(kids[2]).display + ':' + getComputedStyle(kids[3]).display;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__li").unwrap().value,
        "list-item",
        "li 计算 display 应为 'list-item'（spec UA sheet li 规则 display:list-item——slice24 known-deviation F2 已翻转）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cv").unwrap().value,
        "inline",
        "canvas 计算 display 应为 'inline'（不在 UA sheet block 集，replaced 元素回落初始值——slice24 known-deviation F2 已翻转）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__liP + ':' + globalThis.__cvP)").unwrap().value,
        "list-item:inline",
        "plain 双臂同值：innerHTML 解析产物 li/canvas 走同一 UA 表（host miss 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ctrl").unwrap().value,
        "block:block",
        "负控制：div/ul 仍 block——block 集其余值不受翻转扰动"
    );
}
