// renderer 信任边界 ②C'（2026-11-01）：`__zw*` 特权 mutator counter-plugin 常驻回归。
// 覆盖面：① 非测试模式删除面（4 枚 mutator 页面不可见 + reader 钩子保留 + 幂等）；
// ② shim 闭包化非回归（permissions.request() headless 自动授予、execCommand copy 桥）；
// ③ fullscreen 激活门删除后不可旁路；④ 测试模式不变量（无 plugin 时钩子原样——runner
// 契约）。counter-plugin 由宿主在各 shim 装载点于页面脚本前执行，测试直接复刻该装配序。

// 非测试模式装配序复刻：shim → counter-plugin（幂等执行两次，context 重建重定义后
// 二次删除同款语义）。
fn execute_shim_with_counter_plugin(sandbox: &mut zero_script_sandbox::V8Sandbox) {
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.execute(crate::counter_plugin_script()).unwrap();
    sandbox.execute(crate::counter_plugin_script()).unwrap();
}

#[test]
fn counter_plugin_deletes_privileged_mutators_2c_prime() {
    // 删除面：4 枚特权 mutator 页面不可见；reader 钩子保留（permissions.query 与
    // fullscreen 激活门实现自身消费，非 mutator）；navigator 基础面不回归。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    execute_shim_with_counter_plugin(&mut sandbox);
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__mutators = ['__zwUserActivate','__zwClipboardStoreWrite',\
               '__zwDeleteAllCookies','__zwSetPermission']\
               .map(function(n){ return typeof globalThis[n]; }).join(',');\
             globalThis.__readers = [\
               typeof globalThis.__zwPermissionState,\
               typeof globalThis.__zwNotifyPermissionChange].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__mutators").unwrap().value,
        "undefined,undefined,undefined,undefined",
        "4 枚特权 mutator 非测试模式页面不可见（typeof undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__readers").unwrap().value,
        "function,function",
        "内部 reader 钩子保留（query/fullscreen 门消费）"
    );
    assert_eq!(
        sandbox.execute("typeof navigator.permissions.query").unwrap().value,
        "function",
        "permissions.query 正常路径不回归"
    );
}

#[test]
fn counter_plugin_permissions_request_auto_grant_survives_closure_2c_prime() {
    // ②C' 核心非回归：request() headless 自动授予（'prompt'→granted）改走闭包别名
    // （part02 _zwPermSetState），counter-plugin 删除 __zwSetPermission 全局后语义不变。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    execute_shim_with_counter_plugin(&mut sandbox);

    sandbox
        .execute(
            "globalThis.__req = 'pending';\
             navigator.permissions.request({ name: 'fullscreen' }).then(function(s){\
               globalThis.__req = s.state;\
               navigator.permissions.query({ name: 'fullscreen' }).then(function(q){\
                 globalThis.__qr = q.state;\
               });\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__req").unwrap().value,
        "granted",
        "request() 自动授予经闭包别名落注册表（非测试模式不回归）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__qr").unwrap().value,
        "granted",
        "query 读到 request() 授予的 granted（同一注册表）"
    );
}

#[test]
fn counter_plugin_fullscreen_activation_gate_not_bypassable_2c_prime() {
    // 激活门：非测试模式 __zwUserActivate 已删除，页面无瞬态激活入口 → 无激活无权限
    // 走 error 路径（与删除前同断言面——但删除后页面无从旁路）。权限路 __zwSetPermission
    // 同删（②C' 全收口）；navigator.permissions.request({name:'fullscreen'}) 自动授予
    // 为 headless 权限 API 既有语义（非钩子旁路，另行锚定不回归）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    execute_shim_with_counter_plugin(&mut sandbox);
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='d'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__gate = [];\
             document.addEventListener('fullscreenerror', function(){ globalThis.__gate.push('error'); });\
             document.getElementById('d').requestFullscreen().then(\
               function(){ globalThis.__gate.push('resolved'); },\
               function(e){ globalThis.__gate.push((e instanceof TypeError) ? 'TypeError' : 'other'); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__gate.join(',')").unwrap().value,
        "TypeError,error",
        "非测试模式无激活入口 → TypeError 拒绝 + fullscreenerror（不可旁路）"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "激活门拒绝路径不设 fullscreenElement"
    );
}

#[test]
fn counter_plugin_exec_command_copy_bridge_survives_closure_2c_prime() {
    // ②C' part06 非回归：execCommand('copy') defaultPrevented 桥改走闭包别名
    // （part02 _zwClipStoreWrite），counter-plugin 删除 __zwClipboardStoreWrite 全局后
    // copy→readText 往返不回归（复制按钮高频面）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    execute_shim_with_counter_plugin(&mut sandbox);
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__rt = 'X';\
             document.addEventListener('copy', function(ev){\
               ev.preventDefault();\
               ev.clipboardData.setData('text/plain', 'ZWPAYLOAD');\
             });\
             document.execCommand('copy');\
             navigator.clipboard.readText().then(function(t){ globalThis.__rt = t; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__rt").unwrap().value,
        "ZWPAYLOAD",
        "execCommand copy 桥经闭包别名落 store（删除全局后不回归）"
    );
}

#[test]
fn shim_without_counter_plugin_keeps_hooks_test_mode_invariance() {
    // 测试模式不变量：仅装载 shim（宿主在 ZW_RENDERER_AUTOMATION=1 时跳过 plugin，
    // runner 装配同款）→ 4 枚钩子原样，testdriver stub 依赖面零变化。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
        .execute(
            "globalThis.__kept = ['__zwUserActivate','__zwClipboardStoreWrite',\
               '__zwDeleteAllCookies','__zwSetPermission']\
               .map(function(n){ return typeof globalThis[n] === 'function'; }).every(Boolean);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__kept)").unwrap().value,
        "true",
        "测试模式（无 plugin）钩子原样保留（runner 契约）"
    );
}
