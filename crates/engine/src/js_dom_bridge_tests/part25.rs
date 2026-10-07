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

// t2-pb1 T3（首轮测试有效性审查缺口，2026-10-02）：`__zwHostDynamicScripts` bootstrap
// 旗标钉（与上测 `__zwHostOwnsDynamicScripts` 是两个旗标——本旗标由 renderer js_worker
// bootstrap 与 ResetDocumentState 重建后置位，js_worker.rs 两处 execute；语义同为 R387b
// shim 页面 fetch 分支让位，renderer 宿主通路单执行者归属）。两段：未置位负控制（fetch
// 通道被调）；置位后通道关闭。renderer 侧置位时序（bootstrap + 复位重臂）由 js_worker
// 测试 host_dynamic_scripts_flag_set_at_bootstrap_and_rearmed_after_reset 黑盒覆盖。
#[test]
fn r387b2_host_dynamic_scripts_bootstrap_flag_skips_shim_fetch() {
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
    // fetch stub：同上测（计数 stub 激活 R387b 分支）。
    sandbox.execute(
        "globalThis.__fetchCalls = 0;\
         globalThis.fetch = function (u) {\
           globalThis.__fetchCalls = (globalThis.__fetchCalls | 0) + 1;\
           return Promise.reject(new Error('stub-net'));\
         };",
    )
    .unwrap();
    // 阶段 1：旗标未置位（browser 单进程路径）→ R387b 原行为。
    sandbox.execute(
        "var s = globalThis.document.createElement('script');\
         s.src = '/dyn-bs-a.js';\
         globalThis.document.getElementById('host').appendChild(s);",
    )
    .unwrap();
    let calls = sandbox.execute("String(globalThis.__fetchCalls)").unwrap().value;
    assert_eq!(calls, "1", "旗标未置位：R387b 页面 fetch 通道保持原行为（负控制）");
    // 阶段 2：置位（renderer bootstrap 等价动作）→ 通道关闭。
    sandbox.execute(
        "globalThis.__zwHostDynamicScripts = true;\
         var s2 = globalThis.document.createElement('script');\
         s2.src = '/dyn-bs-b.js';\
         globalThis.document.getElementById('host').appendChild(s2);",
    )
    .unwrap();
    let calls2 = sandbox.execute("String(globalThis.__fetchCalls)").unwrap().value;
    assert_eq!(calls2, "1", "旗标置位：R387b shim 页面 fetch 分支让位（renderer 宿主通路单执行者归属）");
}

// slice18 评审收尾 S1：loadmatrix 最小判别案（CI 常驻 fixture）端到端钉——以入驻
// fixture 页（`tests/fixtures/loadmatrix-zw-001.html`，v1 同源基本型 + v9 真跨域）
// 的**整页脚本**驱动，双相位判别「单执行者归属宿主」契约：
// 相位 1（负控制，无 `__zwHostOwnsDynamicScripts`）：shim 页面 fetch 通道保持
//   原行为——两个动态 src 均经通道取回（channel==2；修前缺陷形态：同源与宿主
//   tick 双执行、跨域无 ACAO 误派 error）。
// 相位 2（renderer SetDomSnapshot 契约置位）：通道整体关闭（channel==0），宿主
//   apply（drain→apply→快照换新，真管线同序）+ R145 identity 桥后派发
//   （`__zw_dispatch_script_event`，绝对 URL）送达 onload 恰一次——单执行
//   语义钉：__zwALoad==1、__zwXLoad==1、__zwErr==0。修前 base（85479210a，
//   不识标志）相位 2 显红：通道照走 channel==2（双案皆取回）且 stub reject
//   双误派 __zwErr==2（跨域误 error 语义）。
// 与 r387b 的差别：r387b 用合成 createElement 序列钉通道门控本身；本钉以入驻
// fixture 全页驱动（解析→页面脚本→动态追加→宿主派发→页面计数），锁定「页面级
// 可观测语义」（计数==1）而非仅通道调用数。
// https://html.spec.whatwg.org/multipage/scripting.html#fetch-a-classic-script
#[test]
fn r387c_loadmatrix_fixture_single_execution_end_to_end() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    const FIXTURE: &str = include_str!("../../tests/fixtures/loadmatrix-zw-001.html");
    let page_url_value = "https://zw.test/loadmatrix-zw-001.html".to_string();
    // 相位执行器：独立 sandbox 各相位（标志置位是文档域一次性契约，不回收）。
    fn run_phase(host_owns: bool, fixture: &str, page_url: &str) -> (String, String) {
        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(fixture.to_string()));
        let page_url_cell: Arc<Mutex<String>> = Arc::new(Mutex::new(page_url.to_string()));
        let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
            std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(
            &mut sandbox,
            &mutations,
            &dom_html,
            &page_url_cell,
            &canvas_registry,
            None,
        );
        // fetch stub：计数（含 URL 签名）+ reject（宿主是唯一执行者，stub 不供源码）。
        sandbox
            .execute(
                "globalThis.__zwFetchCalls = 0;\
                 globalThis.__zwFetchUrls = '';\
                 globalThis.fetch = function (u) {\
                   globalThis.__zwFetchCalls = (globalThis.__zwFetchCalls | 0) + 1;\
                   globalThis.__zwFetchUrls += String(u) + ';';\
                   return Promise.reject(new Error('stub-net'));\
                 };",
            )
            .unwrap();
        if host_owns {
            sandbox
                .execute("globalThis.__zwHostOwnsDynamicScripts = true;")
                .unwrap();
        }
        // 整页驱动：按文档序执行页面脚本（engine 页面脚本提取真路径）。
        for script in crate::extract_page_scripts(fixture) {
            match script {
                crate::PageScript::Inline(code) => {
                    sandbox.execute(&code).unwrap();
                }
                _ => panic!("fixture 只含内联页面脚本"),
            }
        }
        let calls = sandbox.execute("String(globalThis.__zwFetchCalls)").unwrap().value;
        let urls = sandbox.execute("String(globalThis.__zwFetchUrls)").unwrap().value;
        (calls, urls)
    }
    // 相位 1：无宿主置位 → 通道原行为（两个动态 src 均经页面 fetch 通道）。
    let (calls1, urls1) = run_phase(false, FIXTURE, &page_url_value);
    assert_eq!(
        calls1, "2",
        "无宿主置位：同源与跨域动态 src 均经 shim 页面 fetch 通道（负控制）"
    );
    assert!(
        urls1.contains("/s18-dyn-a.js") && urls1.contains("cdn.zw.test/s18-dyn-x.js"),
        "两案 src 均入通道（同源 + 跨域）"
    );
    // 相位 2：renderer SetDomSnapshot 契约置位 → 通道关闭 + 宿主派发单执行。
    let mut sandbox2 = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(FIXTURE.to_string()));
    let page_url_cell: Arc<Mutex<String>> = Arc::new(Mutex::new(page_url_value.clone()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox2,
        &mutations,
        &dom_html,
        &page_url_cell,
        &canvas_registry,
        None,
    );
    sandbox2
        .execute(
            "globalThis.__zwFetchCalls = 0;\
             globalThis.fetch = function () {\
               globalThis.__zwFetchCalls = (globalThis.__zwFetchCalls | 0) + 1;\
               return Promise.reject(new Error('stub-net'));\
             };\
             globalThis.__zwHostOwnsDynamicScripts = true;",
        )
        .unwrap();
    for script in crate::extract_page_scripts(FIXTURE) {
        match script {
            crate::PageScript::Inline(code) => {
                sandbox2.execute(&code).unwrap();
            }
            _ => panic!("fixture 只含内联页面脚本"),
        }
    }
    let calls2 = sandbox2
        .execute("String(globalThis.__zwFetchCalls)")
        .unwrap()
        .value;
    assert_eq!(
        calls2, "0",
        "宿主所有权置位：shim cors 语义 fetch 通道整体跳过（同源/跨域一律不经页面 fetch）"
    );
    // 宿主 apply（真管线同序：drain → apply → 快照换新；runtime.rs tick_dynamic_scripts
    // 派发前置 set_dom_snapshot 同构）——动态 script 元素落快照后派发才可达。
    let batch: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    let (applied_html, handle_selectors) =
        crate::apply_mutations_to_html_with_handles(&dom_html.lock().unwrap(), &batch)
            .expect("宿主 apply：动态脚本追加落快照");
    *dom_html.lock().unwrap() = applied_html;
    // R145 identity 桥（renderer register_identity_bridge_callback 同款）：sel→handle
    // 反查——宿主 sel 派发落到页面在 createElement handle proxy 上注册的 onload/onerror。
    let sel_to_handle: std::collections::HashMap<String, String> = handle_selectors
        .into_iter()
        .map(|(h, s)| (s, h))
        .collect();
    sandbox2.register_callback(
        "__zw_handle_for_selector",
        Box::new(move |args: &[String]| -> String {
            let sel = args.first().map(String::as_str).unwrap_or("");
            sel_to_handle.get(sel).cloned().unwrap_or_default()
        }),
    );
    // 宿主派发（tick_dynamic_scripts 同入口，绝对 URL）：单执行语义钉。
    sandbox2
        .execute(
            "globalThis.__zw_dispatch_script_event('https://zw.test/s18-dyn-a.js', 'load');\
             globalThis.__zw_dispatch_script_event('https://cdn.zw.test/s18-dyn-x.js', 'load');",
        )
        .unwrap();
    let verdict = sandbox2
        .execute(
            "JSON.stringify({\
               aLoad: (globalThis.__zwALoad || 0),\
               xLoad: (globalThis.__zwXLoad || 0),\
               err: (globalThis.__zwErr || 0)\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        verdict,
        r#"{"aLoad":1,"xLoad":1,"err":0}"#,
        "单执行语义：两案 onload 恰一次、页面计数恰为 1、零误派 error"
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

// R5010（mm-regression 根修，2026-10-07）：insertAdjacentHTML → apply → gEBI 经 R100
// identity 反查返回 handle-only proxy 后，其子读（childNodes/firstChild/textContent）
// 必须经 `_r100SelOfHandle` 锚回 sel 查 host，不得恒空。生产 face：s30-multimatch
// 集成驱动读 verdict textContent=''（apply 已落、pending 表已被 `__zw_apply_generation_
// bump` 清、querySelector 命中后 `_zwQueryWrapIdentity` 以 `__zw_handle_for_selector`
// 反查包成 handle proxy——`_childNodeList(null, handle)` 旧直接返 []）。
// https://dom.spec.whatwg.org/#dom-node-textcontent
// https://dom.spec.whatwg.org/#dom-parentnode-queryselector
#[test]
fn r5010_r100_handle_proxy_child_reads_anchor_sel_after_iadj_html_apply() {
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
    // ① 页面同 turn：insertAdjacentHTML 解析产物（plain、无 handle/sel）入 pending 表。
    sandbox
        .execute(
            "globalThis.document.body.insertAdjacentHTML('beforeend',\
             \x20 '<div id=\"s30-verdict\">S30-MULTIMATCH: PASS</div>');",
        )
        .unwrap();
    // ② 宿主 apply（renderer drain 同入口）：insertAdjacentHTML 落快照（verdict 进
    //    host 文档——读侧数据源；桥侧 iadj apply 本身不产 handle，生产 identity 桥的
    //    handle→sel merge 另有源头，读路径修复不依赖其来源）。
    let batch: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch.is_empty(), "insertAdjacentHTML 必须产出 mutation 记录");
    let applied = crate::js_dom_bridge::apply_mutations_to_html_with_handles(
        &dom_html.lock().unwrap_or_else(|e| e.into_inner()),
        &batch,
    )
    .expect("宿主 apply：insertAdjacentHTML 落快照");
    *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = applied.0;
    // ③ apply 代际 purge（生产 K3-C bump 同入口）：清 pending 表 → gEBI 回落查询链。
    sandbox.execute("globalThis.__zw_apply_generation_bump();").unwrap();
    // ④ identity 桥 stub（r387c 同款注册，镜像生产 `__zw_handle_for_selector`
    //    selector→handle 反查）：查询命中后 `_zwQueryWrapIdentity` 反查命中 →
    //    返 handle-only proxy（生产 FAIL 形态：verdict 被 `__n{n}` handle 包装）。
    sandbox.register_callback(
        "__zw_handle_for_selector",
        Box::new(|args: &[String]| -> String {
            if args.first().map(String::as_str) == Some("#s30-verdict") {
                "__n226".to_string()
            } else {
                String::new()
            }
        }),
    );
    // ⑤ 修复面：handle-only proxy 的子读经 `_r100SelOfHandle` 锚回 sel 查 host。
    //    （JS 串内不用 `//` 行注释——Rust `\<newline>` 行继续使整串成单行。）
    sandbox
        .execute(
            "var v = globalThis.document.getElementById('s30-verdict');\
             globalThis.__r5010 = v ? JSON.stringify({\
               handle: v.__zwHandle || null,\
               kids: v.childNodes.length,\
               text: v.textContent,\
               first: v.firstChild ? v.firstChild.nodeValue : null\
             }) : 'null';",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r5010").unwrap().value;
    assert!(
        out.contains("\"handle\":\"__n226\""),
        "R5010：gEBI 须经 R100 反查返回 handle-only proxy（生产 FAIL 形态前提），got: {out}"
    );
    assert!(
        out.contains("\"kids\":1")
            && out.contains("\"text\":\"S30-MULTIMATCH: PASS\"")
            && out.contains("\"first\":\"S30-MULTIMATCH: PASS\""),
        "R5010：handle-only proxy 子读须锚回 sel 读到 apply 后真实子树（childNodes/firstChild/textContent），got: {out}"
    );
}
