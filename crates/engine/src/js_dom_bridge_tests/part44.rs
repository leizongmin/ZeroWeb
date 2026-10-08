// t8i（site-compat bilibili-20261002-r1）回归钉：handle 形态 template.content 缺
// cloneNode + 身份不恒等。根因：part04 get trap TEMPLATE content 分支的视图构造里
// cloneNode 被 `if (!handle)` 门只补给 sel 形态，handle 形态（createElement('template')，
// Vue legacy insertStaticContent 创建路径）视图为普通字面对象——原型链
// Object.prototype，「走 Node.prototype 泛型」不可达（基线对照实证既有面，
// evidence/t8g-merged-smoke/r1/clone-probe-*.json）。

/// cloneNode 钉：handle 形态 content.cloneNode(true) 可用且语义正确——返回
/// DocumentFragment、deep 克隆子、克隆体与源独立。修复前 typeof content.cloneNode
/// 为 'undefined'（execute 抛 TypeError），红→绿判别。
#[test]
fn test_t8i_handle_template_content_clone_node() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    sandbox
        .execute(
            "var t = document.createElement('template');\
             t.innerHTML = '<div data-v-a>a</div><p data-v-b>b</p>';\
             var c = t.content;\
             globalThis.__t8iCloneNodeType = typeof c.cloneNode;\
             var cl = c.cloneNode(true);\
             globalThis.__t8iCloneIsFragment = cl.nodeType === 11;\
             globalThis.__t8iCloneChildTags = (cl.childNodes ? Array.prototype.map.call(cl.childNodes, function (n) { return n.tagName || ''; }) : []).join(',');\
             if (cl.firstChild) { var extra = document.createElement('span'); cl.appendChild(extra); }\
             globalThis.__t8iSrcUnchanged = c.childNodes.length === 2;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__t8iCloneNodeType")
            .unwrap()
            .value,
        "function",
        "handle 形态 content.cloneNode 应为函数（修复前 undefined，Vue legacy 爆发点）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__t8iCloneIsFragment)")
            .unwrap()
            .value,
        "true",
        "cloneNode(true) 应返回 DocumentFragment（nodeType 11）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__t8iCloneChildTags")
            .unwrap()
            .value,
        "DIV,P",
        "deep 克隆应复制全部元素子（Vue insertStaticContent 的消费形态）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__t8iSrcUnchanged)")
            .unwrap()
            .value,
        "true",
        "克隆体变异不得影响源 content 子树"
    );
}

/// 身份恒等钉：`t.content === t.content`（spec the-template-element：content 返回
/// 模板 contents 的同一 DocumentFragment）。修复前每读新建视图、恒等必假。
#[test]
fn test_t8i_template_content_identity() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    sandbox
        .execute(
            "var t = document.createElement('template');\
             t.innerHTML = '<div>x</div>';\
             globalThis.__t8iId1 = t.content === t.content;\
             globalThis.__t8iLenBefore = t.content.childNodes.length;\
             t.content.appendChild(document.createElement('b'));\
             globalThis.__t8iId2 = t.content === t.content;\
             globalThis.__t8iLenAfter = t.content.childNodes.length;\
             globalThis.__t8iKidSeen = __t8iLenBefore === 1 && __t8iLenAfter === 2;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t8iId1)").unwrap().value,
        "true",
        "t.content === t.content 应恒等（spec the-template-element）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t8iId2)").unwrap().value,
        "true",
        "content 子树变异后身份仍恒等（活读包装）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__t8iKidSeen)")
            .unwrap()
            .value,
        "true",
        "缓存视图的 childNodes 仍为活读（innerHTML 1 子 + appendChild 后 2 子，经同一册可见）"
    );
}
