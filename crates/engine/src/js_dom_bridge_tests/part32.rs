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
             globalThis.__nsLitMiss = xtn === null;\
             globalThis.__nsNullFull = (function () { var a = sp.getAttributeNodeNS(null, 'xlink:title'); return !!a && a.value === 'xt' && a.localName === 'xlink:title' && a.prefix === null && a.namespaceURI === null; })();\
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
    // slice26 翻转（原 D5 现状守卫——Chrome/154 oracle 裁决翻转，详见
    // test_factory_attr_ns_gate_s26_flip）：HTML 解析字面 'xlink:title' 属性 ns=null、
    // localName=整串——(xlink-ns,'title') 双 null；(null,'xlink:title') 按整串命中且
    // Attr 字段 ns=null；他 local 查询仍 miss。
    assert_eq!(
        sandbox.execute("String(globalThis.__nsLitMiss + ':' + globalThis.__nsNullFull + ':' + globalThis.__nsMiss)").unwrap().value,
        "true:true:true",
        "getAttributeNodeNS 字面前缀名按 concept-attribute-namespace 字面对（ns=null、localName=整串；slice26 已翻转 R190 映射近似）"
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
             globalThis.__pfXlinkMiss = pf === null;\
             globalThis.__pfNullFull = (function () { var a = sp.getAttributeNodeNS(null, 'xlink:href'); return !!a && a.value === 'u' && a.localName === 'xlink:href' && a.prefix === null && a.namespaceURI === null; })();\
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
    // slice26 翻转（原 D5 现状守卫——Chrome/154 oracle 裁决翻转，详见
    // test_factory_attr_ns_gate_s26_flip）：字面 'xlink:href' 属性 ns=null、
    // localName=整串——(xlink-ns,'href') 双 null；(null,'xlink:href') 按整串命中；
    // (null,'href') 不命中（无短名属性）；proxy (null,'foo') 命中不受扰动。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__pfXlinkMiss + ':' + globalThis.__pfNullFull + ':' + globalThis.__pfNullMiss + ':' + globalThis.__selHit)")
            .unwrap()
            .value,
        "true:true:true:true",
        "getAttributeNodeNS 字面前缀名四面（xlink-ns miss、null+整串 hit、null+短名 miss、sel 控制不变；slice26 已翻转 R190 映射近似）"
    );
}

// slice25 翻转钉（原 slice24 收尾轮 TE2+F2 known-deviation 现状钉，同提交翻转兑现
// 问题池 F2）：_zwUaDisplay UA 默认表 li/canvas 回归 HTML 渲染 UA sheet 标准值
//（https://html.spec.whatwg.org/multipage/rendering.html#the-css-user-agent-style-sheet-and-presentational-hints ）：
// li → 'list-item'（`li { display: list-item; }`）；canvas 不在 UA sheet block 集，
// replaced 元素回落 display 初始值 'inline'（part01 _zwUaDisplay：li 专支 + canvas
// 移出 block 表落 inline 兜底）。双臂：proxy createElement（sel 世界，host miss 后
// 落 UA 表）与 parsed innerHTML（plain 世界）同值。负控制：div/ul 仍 block（block
// 集其余值不受翻转扰动）；inline style 优先序与 host miss 回落序由交付钉
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

// slice26 翻转钉（slice25 D 族 D1 残缺口）：_zwUaDisplay UA 默认表 hidden 组 + block
// 补全——HTML 渲染 UA sheet（https://html.spec.whatwg.org/multipage/rendering.html#the-css-user-agent-style-sheet-and-presentational-hints
// 15.3.1 hidden 列表 `…, head, …, script, style, template, title { display: none; }`；
// 15.3.3 flow content 列表 `address, blockquote, center, …, legend, … { display: block; }`）。
// Chrome/154 oracle（diag/evidence/slice26/chrome-oracle.json）：script/head/style/title
// → 'none'、center/legend → 'block'。host 面（style-system ua_default_display）六值已
// 对齐（script/style/title/head→None L101、center/legend→Block L72/74）——D4 双面义务
// 无分叉，本翻转仅 JS 回落面。消费面：jQuery css_defaultDisplay 以非 'none'/非空判定
// 跳过 iframe 兜底——none 组翻 none 后 .show() 入 iframe 分支为 Chrome 同款行为；活体
// 可见轴由 baidu 首页锚 + sugrec 链（s24o-final TAG=fix26）回归钉住。无直接上游 WPT
// 用例（shim 双世界 getComputedStyle 回落为 ZeroWeb 特有架构），补等价本地钉。
#[test]
fn test_ua_display_none_group_center_legend_s26_flip() {
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
        "<html><head id=\"s26hd\"><title id=\"s26ti\">t</title>\
         <script id=\"s26sc\"></script><style id=\"s26sty\"></style></head>\
         <body><center id=\"s26ce\"></center><legend id=\"s26lg\"></legend></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var tags = ['script','head','style','title','center','legend'];\
             globalThis.__proxyD = tags.map(function (t) {\
               var el = document.createElement(t);\
               document.body.appendChild(el);\
               return getComputedStyle(el).display;\
             }).join(',');\
             var hids = ['s26sc','s26hd','s26sty','s26ti','s26ce','s26lg'];\
             globalThis.__hostD = hids.map(function (i2) {\
               return getComputedStyle(document.getElementById(i2)).display;\
             }).join(',');\
             document.body.innerHTML = '<script></script><style></style><title></title><center></center><legend></legend><div></div><span></span>';\
             var kids = document.body.childNodes;\
             globalThis.__plainD = [];\
             for (var i = 0; i < 7; i++) globalThis.__plainD.push(getComputedStyle(kids[i]).display);\
             globalThis.__plainD = globalThis.__plainD.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__proxyD").unwrap().value,
        "none,none,none,none,block,block",
        "proxy createElement 臂（JS 回落面采样——createElement 产物无 sel，读 _zwUaDisplay 表）：script/head/style/title → 'none'（UA sheet 15.3.1 hidden 列表）、center/legend → 'block'（15.3.3 flow content）——D1 残缺口已翻转"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hostD").unwrap().value,
        "none,none,none,none,block,block",
        "host 面 in-DOM 采样臂（I5 收尾第二面）：初始快照六元素（head 族 script/head/style/title + body 族 center/legend）经 getElementById → __zw_get_computed_style → style-system ua_default_display——slice25 D4 教训采样面维度：本臂读 host 面（快照注册元素），proxy/plain 两臂读 JS 回落面；双面同六值直钉 D4 无分叉义务（任一面翻转须同提交双面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__plainD").unwrap().value,
        "none,none,none,block,block,block,inline",
        "plain innerHTML 臂同值（JS 回落面采样，同 proxy 臂——innerHTML 不产 <head>，解析器 body 上下文丢弃，head 由 proxy 臂覆盖）；负控制 div/span 不受扰动"
    );
}

// slice26 翻转钉（slice25 D 族 D2 修复 + D5 前缀面 proxy 臂）：代理世界 NS 读族限定名
// 兜底跨 ns 守卫——spec https://dom.spec.whatwg.org/#dom-element-getattributenodens
// namespace 与 localName 双匹配；https://dom.spec.whatwg.org/#concept-attribute-namespace
// 属性 ns 创建时定死。Chrome/154 oracle（diag/evidence/slice26/chrome-oracle.json）：
// setAttribute('foo') 后 (xhtml-ns,'foo') → null；字面 'xlink:href'（setAttribute 产物，
// local=整串、ns=null）查 (xlink-ns,'href') → null；setAttributeNS(xlink,'xlink:href')
// 显式 ns 属性 (xlink-ns,'href') 命中面保持；(null,name) 无前缀命中面保持；(null,字面
// 限定名) × 显式 ns 属性反向 miss 臂（__metaCrossNull，收尾轮增补，与 detached 世界同形）。
#[test]
fn test_proxy_ns_qname_fallback_cross_ns_s26_flip() {
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
            "var el = document.createElement('div');\
             document.body.appendChild(el);\
             el.setAttribute('foo', 'bar');\
             var hit = el.getAttributeNodeNS(null, 'foo');\
             globalThis.__nullHit = !!hit && hit instanceof Attr && hit.value === 'bar' && hit.localName === 'foo' && hit.namespaceURI === null;\
             globalThis.__gaNsNull = el.getAttributeNS(null, 'foo') === 'bar';\
             globalThis.__gaNsXhtml = el.getAttributeNS('http://www.w3.org/1999/xhtml', 'foo') === null;\
             globalThis.__ganXhtml = el.getAttributeNodeNS('http://www.w3.org/1999/xhtml', 'foo') === null;\
             globalThis.__haNsXhtml = el.hasAttributeNS('http://www.w3.org/1999/xhtml', 'foo') === false;\
             el.setAttribute('xlink:href', 'u');\
             globalThis.__litXlinkMiss = el.getAttributeNodeNS('http://www.w3.org/1999/xlink', 'href') === null;\
             globalThis.__litXlinkGaMiss = el.getAttributeNS('http://www.w3.org/1999/xlink', 'href') === null;\
             globalThis.__litNullFull = (function () { var a = el.getAttributeNodeNS(null, 'xlink:href'); return !!a && a.value === 'u' && a.localName === 'xlink:href'; })();\
             el.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', 'u2');\
             var xh = el.getAttributeNodeNS('http://www.w3.org/1999/xlink', 'href');\
             globalThis.__metaHit = !!xh && xh.value === 'u2' && xh.prefix === 'xlink' && xh.localName === 'href' && xh.namespaceURI === 'http://www.w3.org/1999/xlink';\
             globalThis.__metaGaHit = el.getAttributeNS('http://www.w3.org/1999/xlink', 'href') === 'u2';\
             var el2 = document.createElement('div');\
             document.body.appendChild(el2);\
             el2.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', 'u2');\
             globalThis.__metaCrossNull = el2.getAttributeNodeNS(null, 'xlink:href') === null && el2.getAttributeNS(null, 'xlink:href') === null;\
             el.removeAttributeNS('http://www.w3.org/1999/xhtml', 'foo');\
             globalThis.__rmCrossNsKept = el.getAttribute('foo') === 'bar';\
             el.removeAttributeNS(null, 'foo');\
             globalThis.__rmNullNsRemoved = el.getAttribute('foo') === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nullHit + ':' + globalThis.__gaNsNull)").unwrap().value,
        "true:true",
        "(null,'foo') 无前缀命中面保持：gANNS 返 Attr 真实例（localName='foo'、ns=null）、gANS 命中"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__gaNsXhtml + ':' + globalThis.__ganXhtml + ':' + globalThis.__haNsXhtml)").unwrap().value,
        "true:true:true",
        "跨 ns miss 面（D2 翻转）：setAttribute('foo') 属性 ns=null，(xhtml-ns,'foo') 三读族（gANS/gANNS/hasAttributeNS）均不得命中——Chrome oracle 同面"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__litXlinkMiss + ':' + globalThis.__litXlinkGaMiss + ':' + globalThis.__litNullFull)").unwrap().value,
        "true:true:true",
        "字面前缀名（setAttribute('xlink:href')，local=整串、ns=null）：(xlink-ns,'href') 双读族 miss（D5 proxy 臂翻转）；(null,'xlink:href') 按整串 local 命中——Chrome oracle 同面"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__metaHit + ':' + globalThis.__metaGaHit)").unwrap().value,
        "true:true",
        "显式 ns 属性命中面保持：setAttributeNS(xlink,'xlink:href') 后 (xlink-ns,'href') gANNS 返全字段 Attr、gANS 命中（slice25 D5 守卫的正命中面不受守卫扰动）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__metaCrossNull)").unwrap().value,
        "true",
        "反向跨 ns miss（缺陷轮 S2/TE-S1 同根收尾臂）：setAttributeNS(xlink-ns,'xlink:href') 显式 ns 属性不得被 (null,'xlink:href') 字面限定名误命中——detached 世界 __metaCrossNull 同构面；判别力：_s26QNameEffNs 的 metaMap 支（_attrNSMeta 显式 ns 印记）退化回落前缀映射时，HTML-ns 元素上 effNs=null === 查询 null，host 扁平 'xlink:href'（'u2'）经字面路径误命中，本臂转红。单属性面（fresh 元素 el2）——同 qname 双属性并存在本世界受 R122 扁平存储结构限制（imported-tests.txt 账），混合序列面不在本臂"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rmCrossNsKept + ':' + globalThis.__rmNullNsRemoved)").unwrap().value,
        "true:true",
        "removeAttributeNS 跨 ns 守卫：(xhtml-ns,'foo') 删不掉 ns=null 的 'foo'；(null,'foo') 正常删除（spec removeattributens 缺失即 no-op）"
    );
}

// slice26 翻转钉（slice25 D 族 D5 前缀面裁决翻转）：plain 工厂世界属性 NS 字面化——
// spec https://dom.spec.whatwg.org/#concept-attribute-namespace 属性 ns 创建时定死。
// Chrome/154 oracle（diag/evidence/slice26/chrome-oracle.json）：HTML span 解析
// 'xlink:title' → name/localName 均 'xlink:title'、prefix/ns null，
// getAttributeNodeNS(xlink-ns,'title') 双 null、(null,'xlink:title') 命中；
// setAttribute('xlink:href')（任何元素）同理（setAttribute 不调前缀）。撤销无印记条目的
// prefix→ns 映射后：匹配按字面整名（loc=整串、ans=null），NS 印记（entry.ns——R190
// _r190FixNs 克隆权威）优先。slice25 两处 D5 现状守卫钉（__nsOk/__pfHit）随本翻转撤销。
// 注：本世界 innerHTML 解析产物经快照扁平化——`xlink:href` 到达工厂时已是 'href'（上游
// 前缀丢失，登记观察），故 adjust-foreign-attributes 的外来正命面（Chrome 解析 SVG use
// → ns=xlink 命中）在本世界不可表达；proxy 世界 R190 derive 近似保留（D5 登记残余）。
#[test]
fn test_factory_attr_ns_gate_s26_flip() {
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
             d.innerHTML = '<span id=\"sp\" xlink:title=\"xt\">t</span>';\
             var sp = d.firstChild;\
             globalThis.__xlinkMiss = sp.getAttributeNodeNS('http://www.w3.org/1999/xlink', 'title') === null;\
             globalThis.__nullFullHit = (function () { var a = sp.getAttributeNodeNS(null, 'xlink:title'); return !!a && a.value === 'xt' && a.name === 'xlink:title' && a.localName === 'xlink:title' && a.prefix === null && a.namespaceURI === null; })();\
             globalThis.__gaNullFull = sp.getAttributeNS(null, 'xlink:title') === 'xt';\
             globalThis.__gaNullTailMiss = sp.getAttributeNS(null, 'title') === null;\
             globalThis.__gaXlinkMiss = sp.getAttributeNS('http://www.w3.org/1999/xlink', 'title') === null;\
             globalThis.__ctrlNullId = (function () { var a = sp.getAttributeNodeNS(null, 'id'); return !!a && a.value === 'sp'; })();\
             globalThis.__ctrlXhtmlId = sp.getAttributeNodeNS('http://www.w3.org/1999/xhtml', 'id') === null;\
             globalThis.__ctrlPlainLit = (function () { var a = sp.getAttributeNode('xlink:title'); return !!a && a.value === 'xt'; })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__xlinkMiss + ':' + globalThis.__nullFullHit)").unwrap().value,
        "true:true",
        "HTML span 解析 'xlink:title'：getAttributeNodeNS(xlink-ns,'title') 双 null（slice25 D5 现状守卫 __nsOk 已翻转）；(null,'xlink:title') 按整串 local 命中且 Attr 字段全 null ns——Chrome oracle 同面"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__gaNullFull + ':' + globalThis.__gaNullTailMiss + ':' + globalThis.__gaXlinkMiss)").unwrap().value,
        "true:true:true",
        "同工厂 getAttributeNS 门控一致化：(null,'xlink:title') 命中、(null,'title') 冒号尾误命中撤销、(xlink-ns,'title') miss"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ctrlNullId + ':' + globalThis.__ctrlXhtmlId)").unwrap().value,
        "true:true",
        "负控制：slice25 F1 翻转面不回退——(null,'id') 命中、(xhtml-ns,'id') null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ctrlPlainLit)").unwrap().value,
        "true",
        "负控制：非 NS 读 getAttributeNode('xlink:title') 字面命中不受门控扰动（R116 字面族）"
    );
}

// slice26 翻转钉（slice25 D 族 D3 NS-miss 兜底守卫）：detached body（createHTMLDocument
// R132 覆写版）getAttributeNS/getAttributeNodeNS 的 NS 元数据 miss 不再回落 plain 限定名
// 直查——spec https://dom.spec.whatwg.org/#dom-element-getattributenodens 双匹配；无前缀
// 属性 ns 恒 null（concept-attribute-namespace）。Chrome/154 oracle（diag/evidence/
// slice26/chrome-oracle.json）：setAttribute('foo') 后 (xhtml-ns,'foo') → null、(null,'foo')
// → 命中；setAttributeNS(xhtml-ns,'baz') 后 (null,'baz') → null、(xhtml-ns,'baz') → 命中。
// part20 detached body R132 显式 NS 元数据路径不受影响（元数据命中面正控制）。
#[test]
fn test_detached_body_ns_fallback_s26_flip() {
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
            "var b = document.implementation.createHTMLDocument('t').body;\
             b.setAttribute('foo', 'bar');\
             globalThis.__gaNull = b.getAttributeNS(null, 'foo') === 'bar';\
             globalThis.__gaXhtml = b.getAttributeNS('http://www.w3.org/1999/xhtml', 'foo') === null;\
             var fn0 = b.getAttributeNodeNS(null, 'foo');\
             globalThis.__ganNull = !!fn0 && fn0.value === 'bar';\
             globalThis.__ganXhtml = b.getAttributeNodeNS('http://www.w3.org/1999/xhtml', 'foo') === null;\
             b.setAttributeNS('http://www.w3.org/1999/xhtml', 'baz', 'q');\
             globalThis.__metaGa = b.getAttributeNS('http://www.w3.org/1999/xhtml', 'baz') === 'q';\
             globalThis.__metaGan = b.getAttributeNodeNS('http://www.w3.org/1999/xhtml', 'baz') !== null;\
             globalThis.__metaCrossNull = b.getAttributeNS(null, 'baz') === null && b.getAttributeNodeNS(null, 'baz') === null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__gaNull + ':' + globalThis.__ganNull)").unwrap().value,
        "true:true",
        "(null,'foo') 命中面保持：无前缀属性（setAttribute 产物）ns=null，gANS 命中、gANNS 返 Attr"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__gaXhtml + ':' + globalThis.__ganXhtml)").unwrap().value,
        "true:true",
        "跨 ns miss 面（D3 翻转）：NS 元数据 miss 后不回落 plain 直查——(xhtml-ns,'foo') 双读族 null（Chrome oracle 同面）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__metaGa + ':' + globalThis.__metaGan)").unwrap().value,
        "true:true",
        "正控制：setAttributeNS(xhtml-ns,'baz') 显式 NS 元数据路径命中不变（part20 detached body 面不受翻转扰动）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__metaCrossNull)").unwrap().value,
        "true",
        "反向跨 ns miss（D3 翻转）：显式 ns 属性 (xhtml-ns,'baz') 不得被 (null,'baz') 误命中"
    );
}

// siteopt slice P-B3（bilibili hydration 诊断线）：parsed CharacterData 子（初始 HTML
// 解析出的文本/注释视图）的 **Node 可变方法面**——原型链 Text/Comment.prototype →
// CharacterData → Node 接通后方法调用形态可达，且对非 Element/Document/Fragment 父
// 抛 HierarchyRequestError（Chrome oracle 同面）。判别史：bilibili 视频页 Vue hydration
// 崩溃栈 `recv.appendChild is not a function`（recv = _wrapNodeEntry 文本视图，siteopt
// r17 实锤 keys 吻合）——站点在 Chrome 可用 ⇒ 真实路径 elm 应为元素 ⇒ 分歧在视图原型
// 链缺失（方法面 miss）而非 append 语义本身。修前纯对象视图原型是 Object.prototype，
// Node.prototype 上 R117 族（insertBefore/removeChild/replaceChild）不可达。
// appendChild 经 Node.prototype 本义 `insertBefore(node, null)`（spec dom-node-append-child）。
// https://dom.spec.whatwg.org/#dom-node-append-child
// https://dom.spec.whatwg.org/#concept-node-pre-insert
#[test]
fn test_parsed_characterdata_node_mutable_methods_pb3() {
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
        "<html><body><p id=\"p\">hello</p><!--anchor--><ul id=\"list\"><li>a</li></ul></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var p = document.getElementById('p');\
             var t = p.firstChild;\
             globalThis.__tType = t.nodeType;\
             globalThis.__tIsText = t instanceof Text;\
             var threw = 'none';\
             try { t.appendChild(document.createElement('x')); } catch (e) { threw = e.name; }\
             globalThis.__tAppend = threw;\
             var threw2 = 'none';\
             try { t.insertBefore(document.createElement('x'), null); } catch (e) { threw2 = e.name; }\
             globalThis.__tInsert = threw2;\
             var bodyComment = null;\
             var kids = document.body.childNodes;\
             for (var i = 0; i < kids.length; i++) { if (kids[i].nodeType === 8) { bodyComment = kids[i]; break; } }\
             globalThis.__cFound = !!bodyComment;\
             globalThis.__cIsComment = !!bodyComment && bodyComment instanceof Comment;\
             var threw3 = 'none';\
             try { bodyComment.appendChild(document.createTextNode('y')); } catch (e) { threw3 = e.name; }\
             globalThis.__cAppend = threw3;\
             var ul = document.getElementById('list');\
             var li = document.createElement('li');\
             li.textContent = 'b';\
             var appended = 'none';\
             try { ul.appendChild(li); appended = ul.childNodes.length; } catch (e) { appended = 'ERR:' + e.name; }\
             globalThis.__elAppend = String(appended);\
             globalThis.__elLast = String(ul.lastChild.textContent);\
             var li2 = ul.firstChild;\
             globalThis.__liIsEl = li2 instanceof Element;\
             var threw4 = 'none';\
             try { li2.appendChild(document.createTextNode('ok')); } catch (e) { threw4 = 'ERR:' + e.name; }\
             globalThis.__liAppend = threw4;\
             globalThis.__liText = String(li2.textContent);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__tType)").unwrap().value, "3", "parsed 文本子 nodeType 3 不变");
    assert_eq!(
        sandbox.execute("String(globalThis.__tIsText)").unwrap().value,
        "true",
        "parsed 文本视图 instanceof Text（原型链 Text.prototype 接通）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tAppend)").unwrap().value,
        "HierarchyRequestError",
        "parsed 文本 appendChild → HierarchyRequestError（Chrome oracle：CharacterData 无子面，pre-insert 父类型校验）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tInsert)").unwrap().value,
        "HierarchyRequestError",
        "parsed 文本 insertBefore 同面（appendChild 的语义本体）"
    );
    assert_eq!(sandbox.execute("String(globalThis.__cFound)").unwrap().value, "true", "body 注释子可达");
    assert_eq!(
        sandbox.execute("String(globalThis.__cIsComment)").unwrap().value,
        "true",
        "parsed 注释视图 instanceof Comment（原型链 Comment.prototype 接通）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cAppend)").unwrap().value,
        "HierarchyRequestError",
        "parsed 注释 appendChild 同面（CharacterData 族一致）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__elAppend").unwrap().value,
        "2",
        "正控制：元素 proxy appendChild 正常（#list 2 子，R117 own 分派不回归）"
    );
    assert_eq!(sandbox.execute("globalThis.__elLast").unwrap().value, "b", "元素 append 的子内容正确");
    assert_eq!(
        sandbox.execute("String(globalThis.__liIsEl)").unwrap().value,
        "true",
        "parsed 元素子 instanceof Element（元素 proxy 面不受原型接通扰动）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__liAppend").unwrap().value,
        "none",
        "正控制：parsed 元素子 appendChild 文本无异常（pre-insert 对元素父放行）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__liText").unwrap().value,
        "aok",
        "正控制：append 的文本落树（li 文本内容 a+ok）"
    );
}

#[test]
fn test_insert_before_fallback_no_prototype_appendchild_loop_pb3() {
    // R341 还账（PR #76 审查 D1 / PR #78）：insertBefore 尾部 appendChild 兜底不得
    // 委托 R341 安装的 Node.prototype.appendChild——其 spec 本义即
    // `insertBefore(node, null)`，「无 own insertBefore + 无 __zwHandle」接收者
    // （R219 ①形态：Object.create(HTMLHtmlElement.prototype)，原型链 appendChild
    // 即被安装的原型版）会互调成环至 RangeError 被 catch 吞——终态 no-op 同旧态，
    // 但一次满栈空转纯浪费。钉法：newNode.nodeType 用 getter 计数——每次
    // insertBefore 入口校验读一次，环状递归放大到千次级；断环后个位数。
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r#"
var hits = 0;
var n2 = {};
Object.defineProperty(n2, 'nodeType', { get: function () { hits++; return 1; } });
var plain = Object.create(globalThis.HTMLHtmlElement.prototype);
plain.nodeType = 1; plain.nodeName = 'HTML'; plain.tagName = 'HTML';
globalThis.__premise = plain.appendChild === globalThis.Node.prototype.appendChild;
// PR #78 双审查加固（测试角色 Low-1/2）：前提钉防「钉空转」——appendChild 双侧
// 皆缺失时 undefined === undefined 恒真（空洞通过）；路由前提钉死「真走兜底分支」
// （own insertBefore / __zwHandle 任一存在都会改道，守卫不再被执行）。
globalThis.__premiseFn = typeof plain.appendChild === 'function';
globalThis.__routeNoOwnIb = !Object.prototype.hasOwnProperty.call(plain, 'insertBefore');
globalThis.__routeNoHandle = plain.__zwHandle === undefined;
var r1 = 'none';
try { r1 = plain.insertBefore(n2, null) === n2 ? 'returned' : 'other'; } catch (e1) { r1 = 'ERR:' + e1.name; }
globalThis.__r = r1;
globalThis.__hits = hits;
"#,
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__premise)").unwrap().value,
        "true",
        "前提：接收者 appendChild 解析到 R341 安装的原型版（否则本钉空转无意义）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__premiseFn)").unwrap().value,
        "true",
        "前提加固（审查 Low-1）：appendChild 是函数——封死「双侧缺失 undefined===undefined」空洞通过"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__routeNoOwnIb)").unwrap().value,
        "true",
        "路由前提（审查 Low-2）：无 own insertBefore——真走兜底分支"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__routeNoHandle)").unwrap().value,
        "true",
        "路由前提（审查 Low-2）：无 __zwHandle——非代理，不走直调分支"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r)").unwrap().value,
        "returned",
        "终态不变：无实现接收者 insertBefore 静默返回入参（R219 ①同面）"
    );
    let hits: i64 = sandbox
        .execute("String(globalThis.__hits)")
        .unwrap()
        .value
        .trim()
        .parse()
        .unwrap_or(-1);
    assert!(
        hits > 0 && hits < 20,
        "断环：nodeType 读取个位数（成环时入口校验 × 递归深度 = 数千次），实测 {hits}"
    );
    // 审查 Info-3：终态 no-op 不向 host 发任何变更记录（断环与成环终态一致的本证）。
    assert!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).is_empty(),
        "无实现接收者兜底跳过不产生 DOM mutation 记录"
    );
}

// SVG className IDL（spec SVG2 svg-types `InterfaceSVGAnimatedString` + `SVGElement::className`
// [SameObject] readonly）：SVG ns 元素 className 返 SVGAnimatedString（baseVal/animVal live
// reflect class 属性），HTML 元素维持 string 反射（spec dom-classname）。SVG 元素 className
// 赋值 readonly → no-op（class 不动）。判别史：bilibili 视频页树 diff（P-B3.2 r33，
// 2026-10-06）Chrome 侧 `String(svgEl.className)` 为 "[object SVGAnimatedString]"、ZeroWeb
// 侧 plain string——解析树 svg/path 与 createElementNS 产物同面修复。
// https://svgwg.org/svg2-draft/types.html#InterfaceSVGAnimatedString
// https://html.spec.whatwg.org/multipage/dom.html#dom-classname
#[test]
fn test_svg_classname_animated_string_pb3() {
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
        "<html><body><svg id=\"s1\" class=\"icon a\"><path id=\"pth\" class=\"p1\"/><foreignObject id=\"fo\"><div id=\"fod\" class=\"fd\"></div></foreignObject></svg><div id=\"d1\" class=\"box\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var s1 = document.getElementById('s1');\
             var pth = document.getElementById('pth');\
             var d1 = document.getElementById('d1');\
             globalThis.__svIsAnimated = s1.className instanceof SVGAnimatedString;\
             globalThis.__svSameObject = s1.className === s1.className;\
             globalThis.__svBase = String(s1.className.baseVal);\
             globalThis.__svAnim = String(s1.className.animVal);\
             globalThis.__svStr = String(s1.className);\
             globalThis.__pthIsAnimated = pth.className instanceof SVGAnimatedString;\
             globalThis.__pthBase = String(pth.className.baseVal);\
             globalThis.__fodIsString = typeof document.getElementById('fod').className === 'string';\
             globalThis.__htmlIsString = typeof d1.className === 'string';\
             globalThis.__htmlVal = String(d1.className);\
             var ctor = new SVGAnimatedString();\
             globalThis.__ctorBase = String(ctor.baseVal);\
             globalThis.__ctorTag = String(Object.prototype.toString.call(new SVGAnimatedString()));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__svIsAnimated)").unwrap().value,
        "true",
        "解析树 svg 元素 className instanceof SVGAnimatedString"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svSameObject)").unwrap().value,
        "true",
        "[SameObject]：el.className === el.className（spec SVG2 SameObject 扩展属性）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svBase)").unwrap().value,
        "icon a",
        "baseVal reflect class 属性初值"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svAnim)").unwrap().value,
        "icon a",
        "animVal 同 baseVal（SVG className 无动画分离面）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svStr)").unwrap().value,
        "[object SVGAnimatedString]",
        "String(className) 形态对齐 Chrome 树 diff 对照面"
    );
    assert_eq!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).len(),
        0,
        "纯读取轮不产生 DOM mutation 记录"
    );
    // createElementNS 面（轮 2）：SVG ns 产物同面（CreateElementNS 本身产生记录，既有行为）。
    sandbox
        .execute("var s2 = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); globalThis.__nsIsAnimated = s2.className instanceof SVGAnimatedString; globalThis.__nsBase = String(s2.className.baseVal); globalThis.__nsSame = s2.className === s2.className; globalThis.__svIso = s1.className === s2.className;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nsIsAnimated)").unwrap().value,
        "true",
        "createElementNS(svg) 产物同面"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nsBase)").unwrap().value,
        "",
        "无 class 属性 → baseVal 空串"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nsSame)").unwrap().value,
        "true",
        "ns 产物同面 [SameObject]"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svIso)").unwrap().value,
        "false",
        "per-element 身份：不同元素缓存对象互异（sel 与 handle 两路径产物隔离）"
    );
    assert_eq!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).len(),
        1,
        "createElementNS 产生 1 条记录（既有行为，建立基线）"
    );
    // live 面（轮 3）：setAttribute 落 class 属性，baseVal 即时反映。
    sandbox
        .execute("s1.setAttribute('class', 'x y'); globalThis.__svLive = String(s1.className.baseVal);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__svLive)").unwrap().value,
        "x y",
        "baseVal live：setAttribute('class') 后即时反映"
    );
    assert_eq!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).len(),
        2,
        "setAttribute('class') 再产生 1 条 mutation 记录（累计 2）"
    );
    // SVG no-op 赋值（轮 4）：readonly [SameObject]，class 属性与 mutation 记录都不动。
    sandbox
        .execute("s1.className = 'w'; globalThis.__svgSetNoop = String(s1.getAttribute('class')); globalThis.__svgSetNoopBase = String(s1.className.baseVal);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__pthIsAnimated)").unwrap().value,
        "true",
        "svg 嵌套 path（解析树）同面（ns 探测覆盖 foreign content 后代）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pthBase)").unwrap().value,
        "p1",
        "path baseVal reflect 自身 class"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fodIsString)").unwrap().value,
        "true",
        "foreignObject 内 HTML 后代（integration point → XHTML ns）className 仍为 string"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlIsString)").unwrap().value,
        "true",
        "HTML 元素 className 仍为 string（dom-classname 反射不回归）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlVal)").unwrap().value,
        "box",
        "HTML className 值不变"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svgSetNoop)").unwrap().value,
        "x y",
        "SVG className 赋值 no-op：class 属性不动（readonly [SameObject]）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__svgSetNoopBase)").unwrap().value,
        "x y",
        "SVG className 赋值后 baseVal 不被覆盖"
    );
    assert_eq!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).len(),
        2,
        "SVG className 赋值 no-op 不产生新 mutation（readonly [SameObject]，对照 setAttribute 基线）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ctorBase)").unwrap().value,
        "",
        "new SVGAnimatedString() 构造值面：baseVal 空串"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ctorTag)").unwrap().value,
        "[object SVGAnimatedString]",
        "构造实例 Symbol.toStringTag 面"
    );
    // HTML 反射写正控制（轮 5）：d1.className = 'z' 落 class 属性 + 产生 mutation
    //（R174/R122 行为不回归；与 SVG no-op 零新增对照）。
    sandbox
        .execute("d1.className = 'z'; globalThis.__htmlSet = String(d1.getAttribute('class'));")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlSet)").unwrap().value,
        "z",
        "HTML className 赋值反射写 class 属性（R174/R122 面不回归）"
    );
    assert_eq!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).len(),
        3,
        "HTML className 赋值产生 mutation 记录（累计 3，对照 SVG no-op 零新增）"
    );
    // ns 产物 live 面（轮 6，审查加固）：handle 型 setAttribute 后 baseVal 即时反映。
    sandbox
        .execute("s2.setAttribute('class', 'q'); globalThis.__nsLive = String(s2.className.baseVal);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nsLive)").unwrap().value,
        "q",
        "ns 产物 baseVal live：setAttribute('class') 后即时反映（handle 型写路径）"
    );
    assert_eq!(
        mutations.lock().unwrap_or_else(|e| e.into_inner()).len(),
        4,
        "ns 产物 setAttribute 产生 mutation 记录（累计 4）"
    );
}
