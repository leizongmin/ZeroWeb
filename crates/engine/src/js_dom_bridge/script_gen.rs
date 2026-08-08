//! V8 shim 调用串 / 页面脚本包装生成（R3001 从父文件拆出，控制主文件行数）。
//!
//! 纯字符串构造——无 `Document` / `parse_html` / `DomMutation` 依赖。宿主在各 hook 点
//! （事件派发、错误报告、资源 load/error、字体 settle、页面脚本 try-catch 包装）生成
//! 对应 `__zw_*` 调用串，经 `Sandbox::execute` 执行。经 `pub use script_gen::*` 重导出，
//! 调用方（callbacks / engine / 集成层）仍以父模块路径访问，零调用点改动。

/// 键盘等 DOM 事件的附加字段（传给 JS `KeyboardEvent`）。
#[derive(Debug, Clone, Default)]
pub struct DomEventDetail {
    /// `KeyboardEvent.key`
    pub key: Option<String>,
    /// `KeyboardEvent.code`
    pub code: Option<String>,
    /// `SubmitEvent.submitter`——触发 submit 的按钮唯一选择器（R2984）。click submit button → 该按钮；
    /// Enter 隐式提交 → None（spec：表单默认提交按钮或 null）。
    pub submitter: Option<String>,
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// 生成在 V8 中派发 DOM 事件的脚本片段。
pub fn script_dispatch_dom_event(selector: &str, event_type: &str, detail: Option<&DomEventDetail>) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_ty = escape_js_string(event_type);
    let detail_json = match detail {
        None => "null".to_string(),
        Some(d) => {
            let key = d
                .key
                .as_deref()
                .map(|k| format!("'{}'", escape_js_string(k)))
                .unwrap_or_else(|| "null".to_string());
            let code = d
                .code
                .as_deref()
                .map(|c| format!("'{}'", escape_js_string(c)))
                .unwrap_or_else(|| "null".to_string());
            let submitter = d
                .submitter
                .as_deref()
                .map(|s| format!("'{}'", escape_js_string(s)))
                .unwrap_or_else(|| "null".to_string());
            format!("{{key:{key},code:{code},submitter:{submitter}}}")
        }
    };
    format!("__zw_dispatch_event('{esc_sel}', '{esc_ty}', {detail_json})")
}

/// 构造「调用 form.reset()」的 shim 脚本（P1a form reset，R3050）。宿主在 reset 按钮被 click 时执行：
/// 解析 form 选择器 → 调 shim `form.reset()`（R3048：dispatch cancelable 'reset' 事件 + 未 preventDefault 则
/// 把控件恢复 defaultValue/defaultChecked/defaultSelected）。复用 R3048 全部 reset 语义（防重复实现）。
/// 选择器经 `escape_js_string` 安全嵌入。form 不存在或 reset 非函数 → no-op（guard 防 throw）。
pub fn script_call_form_reset(form_selector: &str) -> String {
    let esc = escape_js_string(form_selector);
    format!("(function(){{var f=document.querySelector('{esc}');if(f&&typeof f.reset==='function')f.reset();}})()")
}

/// 构造「向焦点 input/textarea 注入一个文本字符」的 shim 脚本（P1a form input）。
/// 宿主在 keydown 可打印字符时执行：shim `__zw_text_input(sel, ch)` 把字符 append 到 value
/// （`.value` set 更新缓存 + 记 value 属性 mutation）并派发 'input' 事件。非 input/textarea → no-op。
pub fn script_text_input(selector: &str, key: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_ch = escape_js_string(key);
    format!("__zw_text_input('{esc_sel}', '{esc_ch}')")
}

/// 构造「Backspace 删末字符」的 shim 脚本（P1a form input 编辑互补）。宿主在 keydown
/// Backspace 时执行：shim `__zw_text_delete(sel)` 删 value 末字符并派发 'input' 事件。
pub fn script_text_delete(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_text_delete('{esc_sel}')")
}

/// 构造「报告未捕获脚本错误」的 shim 脚本（R2940 onerror host 集成）。宿主在页面 `<script>` 执行
/// 出错（ScriptError）时执行：shim `__zw_report_error(msg, src, line, col)` 调 legacy window.onerror
///（5-arg 签名）+ 派发 ErrorEvent 'error' 到 window（addEventListener('error') listener），使 Sentry /
/// analytics / GA 等错误上报库的 hook 触发。`message` 取首行（V8 stack trace 多行，window.onerror 的
/// message 为单行）；`source` = 页面 URL；lineno/colno 当前 best-effort 传 0（V8 错误未暴露结构化行列）。
/// https://html.spec.whatwg.org/#runtime-script-errors
pub fn script_report_error(message: &str, source: &str, lineno: u32, colno: u32) -> String {
    let first_line = message.lines().next().unwrap_or("").trim();
    let esc_msg = escape_js_string(first_line);
    let esc_src = escape_js_string(source);
    format!("__zw_report_error('{esc_msg}', '{esc_src}', {lineno}, {colno})")
}

/// 构造「派发 img 元素级 load/error 事件」的 shim 脚本（R2943）。宿主在 img fetch 完成（成功 → "load"，
/// 失败 → "error"）时执行：shim `__zw_dispatch_img_event(absUrl, type)` 按 src 绝对 URL 匹配 `<img>` 元素
/// proxy，用其自身 selector 派发 load/error（保证 listener key 匹配，img.onload/onerror 触发）。`abs_url`
/// = 资源绝对 URL（与 shim 经 `__zw_parse_url` 解析 img.src 的绝对形式比较）；`ty` = "load" / "error"。
pub fn script_dispatch_img_event(abs_url: &str, ty: &str) -> String {
    let esc_url = escape_js_string(abs_url);
    let esc_ty = escape_js_string(ty);
    format!("__zw_dispatch_img_event('{esc_url}', '{esc_ty}')")
}

/// 构造「派发 `<link rel=stylesheet>` 元素级 load/error 事件」的 shim 脚本（R2944）。宿主在样式表 fetch
/// 完成（成功 → "load" / 失败 → "error"）时执行：shim `__zw_dispatch_link_event(absHref, type)` 按 href 绝对
/// URL 匹配 `<link>` 元素 proxy 并用其自身 selector 派发（link.onload/onerror 触发）。
pub fn script_dispatch_link_event(abs_href: &str, ty: &str) -> String {
    let esc_url = escape_js_string(abs_href);
    let esc_ty = escape_js_string(ty);
    format!("__zw_dispatch_link_event('{esc_url}', '{esc_ty}')")
}

/// 构造「派发外部 `<script src>` 元素级 load/error 事件」的 shim 脚本（R2944）。宿主在外部脚本 fetch 完成
///（成功+执行 → "load" / fetch 失败 → "error"）时执行：shim `__zw_dispatch_script_event(absSrc, type)` 按
/// src 绝对 URL 匹配 `<script>` 元素 proxy 并用其自身 selector 派发（script.onload/onerror 触发）。
pub fn script_dispatch_script_event(abs_src: &str, ty: &str) -> String {
    let esc_url = escape_js_string(abs_src);
    let esc_ty = escape_js_string(ty);
    format!("__zw_dispatch_script_event('{esc_url}', '{esc_ty}')")
}

/// 顶层 try-catch 包装捕获的页面脚本错误所写入的 sentinel 全局名。包装器在成功时将其留为
/// `undefined`，抛错时设为错误消息字符串。调用方经 [`page_script_error_check`] 读取。
pub const PAGE_SCRIPT_ERROR_GLOBAL: &str = "__zw_pgerr__";

/// 将 classic 页面 `<script>` 体包进顶层 try-catch，使未捕获的 throw 被捕获进 sentinel 全局
/// （[`PAGE_SCRIPT_ERROR_GLOBAL`]）而非污染持久 V8 Isolate。
///
/// **背景**：persistent_context 模式跨 execute 复用同一 Isolate。页面脚本抛出的未捕获异常若直达
/// V8，embedder 侧 `TryCatch::reset()` 在当前 rusty_v8（150.2.0）下无法清掉跨 execute 的 pending
/// exception——下一条 execute 的新 TryCatch 会观测到它并返回 "Runtime error: null"，使**页面上任何
/// 抛错的 `<script>` 都会废掉其后所有脚本**，并使 host 的 window.onerror 报告（R2940）失效。
/// 在页面脚本层包 try-catch：throw 被这里捕获→调用方读 sentinel 得 Err→`run_page_scripts` 据此
/// 报 window.onerror，且 Isolate 保持干净。
///
/// **作用域**：`code` 内的 `var`/`function` 声明提升到脚本顶层作用域（try 块对它们透明），与未包装
/// 行为一致；顶层 `let`/`const`/`class` 会变为 try 块作用域——classic 内联脚本罕见，module 走
/// `execute_module`。成功时 sentinel 留 `undefined`（非字符串），抛错时设为消息字符串，二者经
/// [`page_script_error_check`] 的 `===undefined` 判别可靠区分（即便 `throw undefined` 也只产生
/// 字符串 "undefined"，不与 undefined 值混淆）。
pub fn script_wrap_page_caught(code: &str) -> String {
    format!(
        "globalThis.{g}=undefined;\ntry{{\n{code}\n}}catch(__zw_e){{globalThis.{g}=(__zw_e&&__zw_e.message)?String(__zw_e.message):String(__zw_e);}}",
        g = PAGE_SCRIPT_ERROR_GLOBAL
    )
}

/// 读取 [`script_wrap_page_caught`] 写入的 sentinel：返回空串表示成功（无抛错），非空串为错误消息。
/// 调用方据此把抛错 surface 为 `Err`。作为独立 execute（包装器执行后 Isolate 干净，本次读取可靠）。
pub fn page_script_error_check() -> String {
    format!(
        "(globalThis.{g}===undefined)?'':globalThis.{g}",
        g = PAGE_SCRIPT_ERROR_GLOBAL
    )
}

/// 调用 shim 的 `<body on*>` → `window.on*` 反射（R2946）。宿主在派发页面生命周期事件（load 等）前执行，
/// 覆盖**无 `<script>` 页面**（其不经 `__zw_begin_script`，故反射不会随脚本执行触发）。有脚本页已在
/// `__zw_begin_script` 内反射过，此处幂等 no-op（按 page URL 去重）。返 shim 调用串。
pub fn script_reflect_body_handlers() -> &'static str {
    "globalThis.__zw_reflect_body_handlers&&globalThis.__zw_reflect_body_handlers();"
}

/// 构造「字体 settle」shim 调用串（R2947）。宿主在 `finish_page_load`（页面脚本阶段收尾）调用：
/// `had_loaded`/`had_error` 据本轮 drain 的 `AsyncPageLoad.take_font_events()` 推导。shim `__zw_font_settle`
/// 派发 FontFaceSet 'loadingdone'（有成功）/ 'loadingerror'（有失败）+ resolve `document.fonts.ready`
/// Promise（settle 语义，不论成败；无 @font-face 页面 had_loaded=had_error=false → 仅 resolve ready，不派事件）。
pub fn script_font_settle(had_loaded: bool, had_error: bool) -> String {
    format!(
        "globalThis.__zw_font_settle&&globalThis.__zw_font_settle({},{});",
        if had_loaded { "true" } else { "false" },
        if had_error { "true" } else { "false" }
    )
}

/// 构造「反映 @font-face 字体为 FontFace」shim 调用串（R2950）。宿主在 `finish_page_load` 对每个
/// font_event 调用：shim `__zw_add_fontface(family, status)` 构造 FontFace(family) + 设 status + add 进
/// document.fonts（按 family 去重）。使 FontFaceSet 含文档 @font-face 字体（补全 set 语义）。`status` =
/// "loaded" / "error"。`family` 经 [`escape_js_string`] 转义防注入。
pub fn script_add_fontface(family: &str, status: &str) -> String {
    format!(
        "globalThis.__zw_add_fontface&&globalThis.__zw_add_fontface('{}','{}');",
        escape_js_string(family),
        escape_js_string(status)
    )
}
