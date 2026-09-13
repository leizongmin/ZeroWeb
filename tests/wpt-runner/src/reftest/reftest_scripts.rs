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

/// 返回 (JS 后最终 HTML, 页面最终焦点 selector)。焦点 selector 取 FocusChanged 记录的
/// **最后一条**（spec 焦点转移语义：最后一次 focus/blur 胜出），供渲染管线
/// `set_focused_selector` 注入 `:focus`/`:focus-within` 样式判定（R4241——serialize→re-parse
/// 边界不携带焦点，须显式跨接）。
pub(super) fn apply_scripted_dom_mutations(
    html: &str,
    base_dir: Option<&Path>,
    wpt_root: Option<&Path>,
    canvas_registry: &std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>>,
) -> (String, Option<String>) {
    let scripts = extract_page_scripts(html);
    let onload_handlers = extract_onload_handlers(html);
    if scripts.is_empty() && onload_handlers.is_empty() {
        return (html.to_string(), None);
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
        Err(_) => return (html.to_string(), None),
    };
    #[cfg(feature = "quickjs")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        match zero_script_sandbox::QuickJSSandbox::with_config(config) {
            Ok(s) => Box::new(s),
            Err(_) => return (html.to_string(), None),
        };

    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("about:blank")));
    register_dom_callbacks(&mut *sandbox, &mutations, &dom_html, &page_url, canvas_registry, None);

    if let Err(e) = sandbox.execute(generate_js_dom_shim()) {
        eprintln!("  [reftest JS] DOM shim init warning: {e}");
        return (html.to_string(), None);
    }
    // reftest harness 自有更完整的 <body>/<frameset>/<html> onload 处理（下方直接执行 handler 体 + 派发
    // 'load'）；禁用 R2946 body→window 反射以避免双 fire（重复 mutation 致 apply_mutations_to_html 失败）。
    let _ = sandbox.execute("globalThis.__zw_no_body_reflect = true;");

    // 按文档序执行每个脚本。外链脚本从 base_dir 读取本地文件（reftest 离线运行）。
    // R4321：每个脚本执行后读 shim `document.write` 写缓冲（见
    // `take_document_write_buffer`），非空则记 (脚本全序号, 缓冲) 供末尾合成
    // SetOuterHtml 替换——WPT ref 页「解析期 write() 无 close」用法（计数器 ref 页
    // system-symbolic / system-additive 等）此前缓冲永不落地 → ref 页缺失 write 行。
    let scripts_indexed = zero_engine::extract_page_scripts_indexed(html);
    let mut doc_write_hits: Vec<(usize, String)> = Vec::new();
    for (exec_pos, script) in scripts.iter().enumerate() {
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
        // R57（M3）：module 脚本按**块作用域**包裹——真 module 每个文件独立作用域，
        // 经典脚本同全局作用域会让多个 module 脚本的顶层 `const canvas = ...`
        // 重声明互撞（canvas-grid 用例 10 格 canvas 各一个 module 脚本，第 2 格起
        // 全 SyntaxError 中止 → 格子全空白，2d.gradient.colorInterpolationMethod
        // oracle A/B 10.7%）。块包裹近似 module 隔离（import/export 仍不支持——
        // best-effort，parse 失败按既有 warning 流程）。
        let is_module = matches!(script, PageScript::InlineModule(_) | PageScript::ExternalModule(_));
        let full = if is_module {
            format!("__zw_begin_script && __zw_begin_script();\n{{\n{code}\n}}")
        } else {
            format!("__zw_begin_script && __zw_begin_script();\n{code}")
        };
        if let Err(e) = sandbox.execute(&full) {
            eprintln!("  [reftest JS] Script execution warning: {e}");
        }
        // 本脚本的全序号：extract_page_scripts 与 extract_page_scripts_indexed 对同一
        // html 的过滤序一致，按执行位次对位取全序号（含非 JS type 的 script）。
        if let (Some(&ord), Some(buf)) = (
            scripts_indexed.get(exec_pos).map(|(_, i)| i),
            take_document_write_buffer(&mut *sandbox),
        ) {
            doc_write_hits.push((ord, buf));
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
    // R4321：先应用 recorded mutations（既有行为，作用于原始 html），再两段式落地
    // document.write 缓冲（合成 SetOuterHtml 作用于上一步产物）。两段分离保证合成
    // 选择器失配等 Err 只回落 write 兜底（= 旧行为），不殃及 recorded mutations。
    let focus_selector = recorded
        .iter()
        .filter_map(|m| match m {
            DomMutation::FocusChanged { selector } => Some(selector.clone()),
            _ => None,
        })
        .next_back()
        .flatten();
    let mut current = html.to_string();
    if !recorded.is_empty() {
        match apply_mutations_to_html(html, &recorded) {
            Ok(new_html) => {
                if std::env::var("REFTEST_DEBUG_HTML").is_ok() {
                    let sample = new_html.chars().take(2000).collect::<String>();
                    eprintln!("  [reftest JS] mutated html (first 2000): {sample}");
                }
                current = new_html;
            }
            Err(e) => {
                eprintln!("  [reftest JS] apply mutations warning: {e}");
            }
        }
    }
    if !doc_write_hits.is_empty() {
        current = apply_document_write_flushes(current, &doc_write_hits);
    }
    (current, focus_selector)
}

/// R4321：读 shim `document.write` 写缓冲（`js_dom_shim/part06.js`：write() 只缓冲、
/// close() 才应用——S16 简化语义）。非空字符串 = 有未 close 的 write → 返回并清空；
/// null/undefined（未 write 或 close 已消费）→ None。
fn take_document_write_buffer(sandbox: &mut dyn zero_script_sandbox::Sandbox) -> Option<String> {
    let result = sandbox
        .execute_json("(typeof __zwDocWriteBuffer === 'string') ? __zwDocWriteBuffer : null")
        .ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&result.value).ok()?;
    match parsed {
        serde_json::Value::String(s) if !s.is_empty() => {
            let _ = sandbox.execute("globalThis.__zwDocWriteBuffer = '';");
            Some(s)
        }
        _ => None,
    }
}

/// R4321：把 (脚本全序号, 缓冲) 落地为 `SetOuterHtml`（脚本元素整体替换为缓冲内容，
/// ≈ 解析流「write 插入于脚本自身位置」语义）。倒序应用——移除后置脚本不影响前置
/// 脚本的结构选择器（nth 路径只数前驱兄弟）。单个脚本落地失败仅告警跳过，不阻断
/// 其余（harness 兜底，数据完整性不受影响）。
///
/// 实体单次解码：解析流语义下缓冲中的字符引用（`&#x2685;` 等——script 源文本不解码，
/// write 进解析流才解码）恰好解码一次。SetOuterHtml 的片段路径（含 `<`）经
/// html5ever 解析自带解码；**纯文本快路径**（无 `<` → create_text_node）不解码——
/// 此处对无 `<` 缓冲先行解码（经同一 html5ever），保证两路均为单次解码。
fn apply_document_write_flushes(html: String, hits: &[(usize, String)]) -> String {
    let doc = zero_dom::parse_html(&html);
    let script_ids = doc.get_elements_by_tag_name("script");
    let mut mutations = Vec::new();
    for (ord, buf) in hits.iter().rev() {
        let Some(&script_id) = script_ids.get(*ord) else {
            eprintln!("  [reftest JS] document.write flush: script ord {ord} not found, skipped");
            continue;
        };
        let Some(selector) = zero_engine::unique_selector_for_node(&doc, script_id) else {
            eprintln!("  [reftest JS] document.write flush: no unique selector for script ord {ord}, skipped");
            continue;
        };
        let decoded = if buf.contains('<') {
            buf.clone()
        } else {
            let frag = zero_dom::parse_html_fragment(buf, "http://www.w3.org/1999/xhtml", "body");
            frag.text_content(frag.root()).unwrap_or_else(|| buf.clone())
        };
        mutations.push(DomMutation::SetOuterHtml {
            selector,
            html: decoded,
        });
    }
    if mutations.is_empty() {
        return html;
    }
    match apply_mutations_to_html(&html, &mutations) {
        Ok(new_html) => new_html,
        Err(e) => {
            eprintln!("  [reftest JS] document.write flush warning: {e}");
            html
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
