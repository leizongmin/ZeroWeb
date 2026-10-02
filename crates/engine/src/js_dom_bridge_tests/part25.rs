// R380（js-dom M4，pending-apply RFC pa3 前置）：part25——pa1 §3.3/§4 探针实证的
// registry 缺口修复（part24 超 2900 行后的新切片段；CLAUDE.md §5 文件大小控制）：
// ① sel 域融合 innerHTML——pending 桶非空时从 `_childNodeList` 融合视图序列化，
//   替代 host 快照旧树（R377 Fail 实际形态：innerHTML 读 apply 滞后旧树）；
// ② innerHTML setter 克隆路径纯文本内容补 text registry 子（R151 只填 markup 形态，
//   纯文本源码落 else 清空分支 → script 克隆的 `_handleChildren[scriptH]` 恒空，
//   R377 插入期脚本钩子源码收集失败 no-op）。
// 注：fragment 展开的「registry 文本子随迁」实验（把 fragment 顶层子数组搬给首个
// 克隆子）已**证伪移除**——fragment registry 只存顶层子，子元素后代经 `_zwMEl
// appendChild` 自记账；顶层数组搬给首子造出自环（`a.children=[a,b]`）→
// `_ceApplyConn`/`_zwHCCollectSubtree` DFS 死循环（part02
// test_fragment_flatten_all_insertion_paths_e2e 复现，100% CPU 挂起）。本测试同时
// 锁定该回归不复发（replaceWith(fragment) 后 connected 传播终止）。

#[test]
fn r387_dynamic_script_append_executes() {
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R387：动态 classic 脚本插入期执行——createElement('script') + textContent= + appendChild
    // 入文档即同步跑（spec prepare-the-script-element；SPA 加载器/分析 SDK 标准路径）。
    // run-once：重复 append 不重跑（`_zwRanScripts` 标记，R377 同源语义）。
    sandbox.execute(
        "try {\
         var s = globalThis.document.createElement('script');\
         s.textContent = \"globalThis.__r387ran = 'yes';\";\
         globalThis.document.getElementById('host').appendChild(s);\
         var first = String(globalThis.__r387ran);\
         globalThis.__r387ran = 'second';\
         globalThis.document.getElementById('host').appendChild(s);\
         globalThis.__r387a = first + ':' + String(globalThis.__r387ran);\
         } catch (err) { globalThis.__r387a = 'ERR:' + err.message; }",
    ).unwrap();
    let out = sandbox.execute("globalThis.__r387a").unwrap().value;
    assert_eq!(
        out, "yes:second",
        "R387：动态 script appendChild 同步执行 + run-once 不重跑"
    );
}

// slice18（site-compat baidu 建议链 /sugrec，R-baidu8 接管收尾）：`__zwHostOwnsDynamicScripts`
// 置位（renderer js_worker SetDomSnapshot 语义）→ 动态 src 脚本**整体**跳过 shim 页面 fetch
// 通道，单点交宿主 PendingDynamicScripts（no-cors IPC 取回，tick_dynamic_scripts）执行。
// spec：classic script 取回是 no-cors 资源取回（request mode "no-cors"），页面 fetch() 是
// cors 语义——无 ACAO 跨域 CDN 脚本在此通道恒败误派 error；且宿主 tick 通道并存，同源
// 脚本双通道双执行（AMD define 双注册同族破坏）。两段：未置位原行为（fetch 被调）；置位
// 后同源/跨域均零调用（红：无本门控时同源也会调 → 2；绿：恒 1）。
// https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
#[test]
fn r387b_host_owns_dynamic_scripts_skips_shim_fetch() {
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zw.test/x.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // fetch stub：裸 sandbox 无 fetch，注入计数 stub 激活 R387b 分支（rejection → 元素 error
    // 的微任务面非本钉断言对象——宿主派发语义由 renderer R-baidu8 runtime 钉覆盖）。
    sandbox.execute(
        "globalThis.__fetchCalls = 0;\
         globalThis.fetch = function (u) {\
           globalThis.__fetchCalls = (globalThis.__fetchCalls | 0) + 1;\
           return Promise.reject(new Error('stub-net'));\
         };",
    )
    .unwrap();
    // 阶段 1：标志未置位 → R387b 原行为（同源 src，fetch 通道被调一次）。
    sandbox.execute(
        "var s = globalThis.document.createElement('script');\
         s.src = '/dyn-a.js';\
         globalThis.document.getElementById('host').appendChild(s);",
    )
    .unwrap();
    let calls = sandbox.execute("String(globalThis.__fetchCalls)").unwrap().value;
    assert_eq!(calls, "1", "标志未置位：R387b 页面 fetch 通道保持原行为（负控制）");
    // 阶段 2：置位（renderer SetDomSnapshot 等价动作）→ 通道整体关闭（同源/跨域一律
    // 不再经页面 fetch——单执行者归属宿主 no-cors 取回）。
    sandbox.execute(
        "globalThis.__zwHostOwnsDynamicScripts = true;\
         var s2 = globalThis.document.createElement('script');\
         s2.src = '/dyn-b.js';\
         globalThis.document.getElementById('host').appendChild(s2);",
    )
    .unwrap();
    let calls2 = sandbox.execute("String(globalThis.__fetchCalls)").unwrap().value;
    assert_eq!(
        calls2, "1",
        "标志置位：shim cors 语义 fetch 通道整体跳过（跨域不误派 error、同源不与宿主 tick 双执行）"
    );
}

// slice18（site-compat baidu 建议链 /sugrec）：宿主 selector 派发链守卫——engine harness
//（无 `__zw_handle_for_selector` 宿主回调注册）下，动态 script 元素的 onload IDL 监听
//（handle key）经 `__zw_dispatch_script_event`（sel 主路径 + R145 桥回落 sel 键）必须送达。
// green-on-base 回归守卫（活体缺口在 renderer 侧 handle 表跨文档陈旧——
// renderer_js_worker_reset_document_state_clears_handle_selector_map 钉承载该修复）。
// https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
#[test]
fn r2944_host_dispatch_reaches_handle_keyed_onload_without_reverse_map() {
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://zw.test/x.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 页面路径：createElement 产物 handle proxy 上挂 onload IDL（DOM 标准注册路径）
    // → appendChild 入 pending mutations。裸 sandbox 无 fetch 且标志未置位 →
    // shim R387b 页面 fetch 通道不激活（宿主派发为唯一事件源，本钉对象）。
    sandbox.execute(
        "globalThis.__xh = undefined;\
         var s = globalThis.document.createElement('script');\
         s.src = '/dyn-x.js';\
         s.onload = function () { globalThis.__xh = 'LOADED'; };\
         globalThis.document.getElementById('host').appendChild(s);",
    )
    .unwrap();
    // 宿主按绝对 URL 派发（tick_dynamic_scripts 同入口）。
    sandbox
        .execute("globalThis.__zw_dispatch_script_event('http://zw.test/dyn-x.js', 'load');")
        .unwrap();
    let out = sandbox.execute("String(globalThis.__xh)").unwrap().value;
    assert_eq!(
        out, "LOADED",
        "宿主 selector 派发必须经 shim handle 代理扫描送达 onload IDL 监听（R145 反查表缺失场景）"
    );
}

#[test]
fn r380_fused_innerhtml_and_text_registry_children() {
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
        "<html><body><div id='container'><div id='target'></div><b></b></div><template><span>New </span><script>document.querySelector('b').remove();</script><span>content</span></template></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // ① 克隆路径：innerHTML setter 纯文本分支补 text 子（R380 ②）→ 克隆 script 有
    //    registry 源码（R377 钩子/序列化的读取面）。
    // ② replaceWith fragment 展开后 target parentNode 同步 null（R379 M6 标记语义）。
    // ③ sel 域融合 innerHTML：同 turn 内 script.remove()（WPT 用例序——remove 先于
    //    host apply）→ container.innerHTML 从融合 childNodes 序列化（含两 span、
    //    不含旧 target / script / b）。与 WPT 用例断言同构。
    //    （JS 串内不用 `//` 行注释——Rust `\<newline>` 行继续使整串成单行，注释会吞代码。）
    sandbox
        .execute(
            "try {\
             var log = [];\
             var target = globalThis.document.getElementById('target');\
             var tpl = globalThis.document.querySelector('template');\
             var frag = tpl.content.cloneNode(true);\
             var sc = frag.querySelector('script');\
             log.push('clone:' + (sc && sc.textContent && sc.textContent.indexOf('querySelector') >= 0 ? 'src' : 'empty'));\
             target.replaceWith(frag);\
             log.push('pw:' + (target.parentNode === null ? 'null' : 'non-null'));\
             var container = globalThis.document.getElementById('container');\
             container.querySelector('script').remove();\
             var ih = container.innerHTML;\
             log.push('fused:' + (ih === '<span>New </span><span>content</span>' ? 'exact' : ('no:' + ih.slice(0, 80))));\
             globalThis.__r380a = log.join('|');\
             } catch (err) { globalThis.__r380a = 'ERR:' + err.message; }",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r380a").unwrap().value;
    assert_eq!(
        out, "clone:src|pw:null|fused:exact",
        "R380：克隆 script 经纯文本 registry 分支有源码 + replaceWith 同步标记 + sel 域 innerHTML 融合序列化与 WPT 期望串全等"
    );
}

#[test]
fn r388_iframe_history_navigation_preserves_session_history() {
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
        "<html><body><iframe src='../resources/blank.html?name=isHistoryNavigation'></iframe></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/service-workers/cache-storage/serviceworker/cache-keys-attributes-for-service-worker.https.html"
            .to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetches_for_callback = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            let reload = args.get(7).cloned().unwrap_or_default();
            let history = args.get(8).cloned().unwrap_or_default();
            fetches_for_callback
                .lock()
                .unwrap()
                .push(format!("{url}|reload={reload}|history={history}"));
            "__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1f<!doctype html><body>loaded</body>"
                .to_string()
        }),
    );
    // https://html.spec.whatwg.org/multipage/nav-history-apis.html#traverse-the-history-by-a-delta
    // Cross-document iframe `src` navigation creates a replacement Window; history.go(-1)
    // still traverses the iframe element's session history and marks the fetch as a history navigation.
    sandbox
        .execute(
            "try {\
             var frame = document.querySelector('iframe');\
             var first = frame.contentWindow;\
             frame.src = '../resources/blank.html?ignore';\
             var second = frame.contentWindow;\
             second.history.go(-1);\
             var third = frame.contentWindow;\
             globalThis.__r388a = JSON.stringify({\
               same12: first === second,\
               same23: second === third,\
               href: third.location.href,\
               historyLength: third.history.length\
             });\
             } catch (err) { globalThis.__r388a = 'ERR:' + err.message; }",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__r388a").unwrap().value,
        r#"{"same12":false,"same23":false,"href":"https://wpt.test/service-workers/cache-storage/resources/blank.html?name=isHistoryNavigation","historyLength":1}"#,
        "R388：iframe history.go(-1) 回到前一跨文档 entry，且新 Window 继承修剪后的 session history"
    );
    assert_eq!(
        fetches.lock().unwrap().as_slice(),
        &[
            "https://wpt.test/service-workers/cache-storage/resources/blank.html?name=isHistoryNavigation|reload=|history="
                .to_string(),
            "https://wpt.test/service-workers/cache-storage/resources/blank.html?ignore|reload=|history="
                .to_string(),
            "https://wpt.test/service-workers/cache-storage/resources/blank.html?name=isHistoryNavigation|reload=|history=1"
                .to_string(),
        ],
        "R388：history traversal fetch 必须携带 isHistoryNavigation 标记"
    );
}
