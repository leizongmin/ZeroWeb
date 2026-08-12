//! Reftest 脚本辅助 —— 执行 reftest 页面中会修改 DOM 的 JS（harness JS vein）。
//!
//! 很多 WPT reftest 依赖页面脚本动态设置条件（生成元素、改 class、等待事件等）。
//! 这些函数在渲染前 best-effort 执行内联/外链脚本与 `<body onload>` handler，把
//! V8 sandbox 记录到的 `DomMutation` 应用回 HTML，使最终截图反映脚本执行后的状态。

use std::path::Path;

use zero_engine::pipeline::PageScript;
use zero_engine::{
    DomMutation, apply_mutations_to_html, extract_page_scripts, generate_js_dom_shim, register_dom_callbacks,
};

pub(super) fn apply_scripted_dom_mutations(
    html: &str,
    base_dir: Option<&Path>,
    wpt_root: Option<&Path>,
    canvas_registry: &std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>>,
) -> String {
    let scripts = extract_page_scripts(html);
    let onload_handlers = extract_onload_handlers(html);
    if scripts.is_empty() && onload_handlers.is_empty() {
        return html.to_string();
    }

    use std::sync::Arc;
    use std::sync::Mutex;
    use zero_script_sandbox::SandboxConfig;

    let config = SandboxConfig {
        // DOM-mutating reftest 脚本通常很短；与既有 reftest JS 超时一致。
        timeout_ms: 5000,
        persistent_context: true,
        ..Default::default()
    };
    #[cfg(feature = "v8")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> = match zero_script_sandbox::V8Sandbox::with_config(config) {
        Ok(s) => Box::new(s),
        Err(_) => return html.to_string(),
    };
    #[cfg(feature = "quickjs")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        match zero_script_sandbox::QuickJSSandbox::with_config(config) {
            Ok(s) => Box::new(s),
            Err(_) => return html.to_string(),
        };

    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("about:blank")));
    register_dom_callbacks(&mut *sandbox, &mutations, &dom_html, &page_url, canvas_registry);

    if let Err(e) = sandbox.execute(generate_js_dom_shim()) {
        eprintln!("  [reftest JS] DOM shim init warning: {e}");
        return html.to_string();
    }
    // reftest harness 自有更完整的 <body>/<frameset>/<html> onload 处理（下方直接执行 handler 体 + 派发
    // 'load'）；禁用 R2946 body→window 反射以避免双 fire（重复 mutation 致 apply_mutations_to_html 失败）。
    let _ = sandbox.execute("globalThis.__zw_no_body_reflect = true;");

    // 按文档序执行每个脚本。外链脚本从 base_dir 读取本地文件（reftest 离线运行）。
    for script in &scripts {
        let code: Option<String> = match script {
            PageScript::Inline(c) | PageScript::InlineModule(c) => Some(c.clone()),
            PageScript::External(src) | PageScript::ExternalModule(src) => {
                match fetch_external_script(src, base_dir, wpt_root) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        eprintln!("  [reftest JS] external script {src}: {e}");
                        None
                    }
                }
            }
        };
        let Some(code) = code else { continue };
        if code.trim().is_empty() {
            continue;
        }
        // module 语义需要编译管线；reftest 视角下按经典脚本 best-effort 执行。
        let full = format!("__zw_begin_script && __zw_begin_script();\n{code}");
        if let Err(e) = sandbox.execute(&full) {
            eprintln!("  [reftest JS] Script execution warning: {e}");
        }
    }

    // 派发 load 事件：(a) 直接执行 `<body onload>` 属性 handler 体；
    // shim 的 setTimeout 经 microtask 立即跑（V8 execute 返回前排空 microtask）。
    for handler in onload_handlers.iter().filter(|h| !h.trim().is_empty()) {
        let full = format!("__zw_begin_script && __zw_begin_script();\n{handler}");
        if let Err(e) = sandbox.execute(&full) {
            eprintln!("  [reftest JS] onload handler warning: {e}");
        }
    }
    // (b) 派发 window 'load' 事件，触发 `addEventListener('load', …)` 监听器（best-effort）。
    let _ = sandbox.execute(
        "if (typeof __zw_dispatch_event === 'function') { try { __zw_dispatch_event('html','load',null); } catch(_e){} }",
    );

    let recorded = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if std::env::var("REFTEST_DEBUG").is_ok() {
        eprintln!("  [reftest JS] recorded {} mutation(s): {:?}", recorded.len(), recorded);
    }
    if std::env::var("REFTEST_DEBUG_HTML").is_ok() {
        let sample = html.chars().take(800).collect::<String>();
        eprintln!("  [reftest JS] original html (first 800): {sample}");
    }
    if recorded.is_empty() {
        return html.to_string();
    }
    match apply_mutations_to_html(html, &recorded) {
        Ok(new_html) => {
            if std::env::var("REFTEST_DEBUG_HTML").is_ok() {
                let sample = new_html.chars().take(2000).collect::<String>();
                eprintln!("  [reftest JS] mutated html (first 2000): {sample}");
            }
            new_html
        }
        Err(e) => {
            eprintln!("  [reftest JS] apply mutations warning: {e}");
            html.to_string()
        }
    }
}

/// 提取 `<body>`/`<frameset>`/`<html>` 上 `onload` 属性的 handler 体（JS 源码）。
///
/// 这些属性在 `load` 事件触发时由浏览器编译为函数体执行；reftest 直接把属性值当
/// JS 源码运行（与 browser 侧 shim 语义一致：shim 的 setTimeout 会立即经 microtask 跑）。
pub(super) fn extract_onload_handlers(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut out = Vec::new();
    for tag in ["body", "frameset", "html"] {
        for id in doc.get_elements_by_tag_name(tag) {
            if let Some(h) = doc.get_attribute(id, "onload")
                && !h.trim().is_empty()
            {
                out.push(h);
            }
        }
    }
    out
}

/// 解析并读取外链脚本（reftest 离线运行，失败返回 `Err` 由调用方跳过、不阻塞）。
///
/// WPT URL 语义（R546/R551 谱系补齐，2026-08-07）：
/// - 以 `/` 开头的 src（如 `/common/reftest-wait.js`）是**套件根相对 URL**，
///   相对 wpt_root（wpt-data 根）解析——不能按文件系统绝对路径处理；
/// - src 可能带 query（如 `foo.js?x=1`，少见），剥离后再加载。
pub(super) fn fetch_external_script(
    src: &str,
    base_dir: Option<&Path>,
    wpt_root: Option<&Path>,
) -> Result<String, String> {
    let src = src.split('?').next().unwrap_or(src);
    let resolved = if src.starts_with('/') {
        match wpt_root {
            Some(root) => root.join(src.trim_start_matches('/')),
            None => {
                return Err(format!(
                    "absolute WPT path {src} but wpt_root not configured (set ReftestConfig::wpt_root)"
                ));
            }
        }
    } else if let Some(base) = base_dir {
        base.join(src)
    } else {
        std::path::Path::new(src).to_path_buf()
    };
    std::fs::read_to_string(&resolved).map_err(|e| format!("{}: {e}", resolved.display()))
}
