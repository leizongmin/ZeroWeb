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
    /// `CompositionEvent.data` / `InputEvent.data`。
    pub data: Option<String>,
    /// `InputEvent.inputType`。
    pub input_type: Option<String>,
    /// `InputEvent.isComposing`。
    pub is_composing: bool,
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
            let data = d
                .data
                .as_deref()
                .map(|value| format!("'{}'", escape_js_string(value)))
                .unwrap_or_else(|| "null".to_string());
            let input_type = d
                .input_type
                .as_deref()
                .map(|value| format!("'{}'", escape_js_string(value)))
                .unwrap_or_else(|| "null".to_string());
            let is_composing = d.is_composing;
            format!(
                "{{key:{key},code:{code},submitter:{submitter},data:{data},inputType:{input_type},isComposing:{is_composing}}}"
            )
        }
    };
    format!("__zw_dispatch_event('{esc_sel}', '{esc_ty}', {detail_json})")
}

/// 构造「派发过渡事件」的脚本（R3248 transitionend + R3252 transitionrun/transitionstart，CSS Transitions）。
/// 宿主在过渡创建/启动/完成帧（`TransitionClock::drain_just_run` / `drain_just_started` /
/// `drain_just_finished` → pipeline `take_pending_transition_events`）执行：`querySelector(selector)` 取唯一
/// 目标元素（`unique_selector_for_node` 保证唯一；stale 已移除 → null guard），
/// `new TransitionEvent(event_type, {propertyName, elapsedTime, bubbles:true})` 派发。
/// `event_type` = `'transitionrun'`（创建，可能 delay 期）/ `'transitionstart'`（delay 过后活跃）/ `'transitionend'`
/// （完成）；三者 init dict 完全相同（CSS Transitions §transitionrun / §transitionstart / §transitionend），
/// 仅事件名不同。TransitionEvent 构造器在 shim part05:1380 注册。UI 编排回调（fade-out 后删元素）依赖。
pub fn script_dispatch_transition_event(selector: &str, event_type: &str, property: &str, elapsed: f64) -> String {
    let sel = escape_js_string(selector);
    let ty = escape_js_string(event_type);
    let prop = escape_js_string(property);
    // elapsed 为有限非负 f64（transition.duration）；直接内嵌数值（非字符串，免转义）。
    let elapsed_str = if elapsed.is_finite() && elapsed >= 0.0 {
        format!("{elapsed}")
    } else {
        "0".to_string()
    };
    format!(
        "(function(){{var _e=document.querySelector('{sel}');if(_e){{try{{_e.dispatchEvent(new TransitionEvent('{ty}',{{propertyName:'{prop}',elapsedTime:{elapsed_str},bubbles:true}}));}}catch(_x){{}}}}}})();"
    )
}

/// 构造「派发动画事件」的脚本（R3249 animationend + R3250 animationiteration + R3251 animationstart，CSS Animations）。
/// 宿主在动画启动/完成/迭代边界帧（`AnimationClock::drain_just_started` / `drain_just_finished` /
/// `drain_just_iterated` → pipeline `take_pending_animation_events`）执行：`querySelector(selector)` 取唯一
/// 目标元素，`new AnimationEvent(event_type, {animationName, elapsedTime, bubbles:true})` 派发。
/// `event_type` = `'animationstart'`（首次进入活跃间隔，elapsedTime=0）/ `'animationend'`（有限动画完成）/
/// `'animationiteration'`（迭代边界，infinite 循环回调）；三者 init dict 完全相同
/// （CSS Animations §animationstart / §animationend / §animationiteration），仅事件名不同。
/// AnimationEvent 构造器在 shim part05:1383 注册。
pub fn script_dispatch_animation_event(selector: &str, event_type: &str, name: &str, elapsed: f64) -> String {
    let sel = escape_js_string(selector);
    let ty = escape_js_string(event_type);
    let nm = escape_js_string(name);
    let elapsed_str = if elapsed.is_finite() && elapsed >= 0.0 {
        format!("{elapsed}")
    } else {
        "0".to_string()
    };
    format!(
        "(function(){{var _e=document.querySelector('{sel}');if(_e){{try{{_e.dispatchEvent(new AnimationEvent('{ty}',{{animationName:'{nm}',elapsedTime:{elapsed_str},bubbles:true}}));}}catch(_x){{}}}}}})();"
    )
}

/// 构造「用户滚动」注入脚本（R3253，UI Events §scroll via user input）。宿主（renderer `handle_scroll_event`）
/// 在收到 browser IPC `ScrollEventParams { delta_x, delta_y }`（用户滚轮/触摸/键盘滚动）时执行：调内部钩子
/// `__zw_user_scroll(dx, dy)`（part01.js）——更新 `_winScroll`（使 `window.scrollY/scrollX` 跟踪用户滚动）+
/// 派 'scroll' 事件（infinite scroll / lazy load / sticky nav / parallax 的**用户滚动**触发依赖）。
///
/// 走内部钩子而非 `globalThis.scrollBy`：绕过页面可能覆写的 `scrollBy`（real browser 的 scroll 事件由实际
/// 滚动派发，不受页面 JS 影响）。`typeof` 守卫防 shim 未安装（无页面 / JS 未启）时 ReferenceError 中断。
/// delta 有限数值；NaN/负经 `__zw_user_scroll` 内部 `Number(dx)||0` 与 `_zwApplyScroll` 的 `<0` clamp 归一。
pub fn script_user_scroll(delta_x: f64, delta_y: f64) -> String {
    let dx = if delta_x.is_finite() { delta_x } else { 0.0 };
    let dy = if delta_y.is_finite() { delta_y } else { 0.0 };
    format!("if(typeof __zw_user_scroll==='function')__zw_user_scroll({dx},{dy});")
}

/// 构造「视口尺寸变化」注入脚本（R3254，CSSOM View §resizing / UI Events §resize）。宿主（renderer
/// `handle_set_viewport`）在收到 browser IPC `SetViewportParams { width, height }` 时执行：调内部钩子
/// `__zw_user_resize(w, h)`（part01.js）——更新 `innerWidth/innerHeight`（+ outer，headless outer≈inner）
/// 使响应式 JS 读到新尺寸 + 派 'resize' 事件到 window（`window.addEventListener('resize')` / innerWidth
/// watcher / matchMedia 触发依赖）。typeof 守卫防 shim 未安装时 ReferenceError。w/h 有限数值；NaN/负经
/// `__zw_user_resize` 内部归一。
pub fn script_user_resize(width: f64, height: f64) -> String {
    let w = if width.is_finite() { width } else { 0.0 };
    let h = if height.is_finite() { height } else { 0.0 };
    format!("if(typeof __zw_user_resize==='function')__zw_user_resize({w},{h});")
}

/// 构造「经原生绑定派发 DOM 事件」的脚本（P1b host→page native 派发，R3121；event 对象丰富化 R3124）。
/// 宿主在 `native_dom` 开启时于 polyfill 派发（[`script_dispatch_dom_event`]）**之外额外**执行：
/// 经 `__zw_native_query_selector(sel)` 解析目标节点（返 native 元素对象，internal slot 存 NodeId）
/// → 调原生 `dispatchEvent(event)`，触发该节点经 native `addEventListener` 注册的监听器（存于
/// engine `dom_bindings::gc::LISTENERS`，polyfill `__zw_dispatch_event` 不达，闭合 S4 host 驱动半边）。
///
/// **event 对象（R3124）**：不再是 bare `{type}`，而带 `target`/`currentTarget`（= 目标节点 `t`，解锁
/// `e.target`/`e.currentTarget` 高频读——事件委托 / 区域检测 / 框架钩子）+ `bubbles:true`（UI 事件
/// click/input/change/submit/keydown 默认冒泡；native `dispatchEvent` 本身**不冒泡**——R3109 限制，
/// bubbles 字段仅为监听器可读的语义标记，真实冒泡待后续）。闭合 R3121 限制①。
///
/// **typeof 守卫**：`__zw_native_query_selector` 仅 native 绑定安装时定义（WebView 进程内沙箱）；
/// 未安装（生产 worker 沙箱，L2 前）→ 守卫 early-return no-op，**避免 ReferenceError 中断派发**
///（信任边界输入校验：生成的串可安全注入任意沙箱）。无匹配节点（querySelector 返 null）→ 第二层
/// 守卫 no-op。选择器 / 事件类型经 [`escape_js_string`] 安全嵌入。复用既有 native 工厂 + dispatchEvent
/// 绑定，零新 engine 代码。
pub fn script_dispatch_native_event(selector: &str, event_type: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_ty = escape_js_string(event_type);
    format!(
        "(function(){{if(typeof __zw_native_query_selector!=='function')return;\
var t=__zw_native_query_selector('{esc_sel}');\
if(t)t.dispatchEvent({{type:'{esc_ty}',target:t,currentTarget:t,bubbles:true}});}})()"
    )
}

/// 构造「调用 form.reset()」的 shim 脚本（P1a form reset，R3050）。宿主在 reset 按钮被 click 时执行：
/// 解析 form 选择器 → 调 shim `form.reset()`（R3048：dispatch cancelable 'reset' 事件 + 未 preventDefault 则
/// 把控件恢复 defaultValue/defaultChecked/defaultSelected）。复用 R3048 全部 reset 语义（防重复实现）。
/// 选择器经 `escape_js_string` 安全嵌入。form 不存在或 reset 非函数 → no-op（guard 防 throw）。
pub fn script_call_form_reset(form_selector: &str) -> String {
    let esc = escape_js_string(form_selector);
    format!("(function(){{var f=document.querySelector('{esc}');if(f&&typeof f.reset==='function')f.reset();}})()")
}

/// 构造不派发页面事件的 UA 表单重置脚本。
///
/// JavaScript 被禁用时，用户代理仍须恢复表单控件默认状态，但不得调用页面
/// `reset` listener。恢复规则与 shim `form.reset()` 的未取消分支保持一致。
/// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#resetting-a-form
pub fn script_reset_form_controls(form_selector: &str) -> String {
    let esc = escape_js_string(form_selector);
    format!(
        "(function(){{\
var f=document.querySelector('{esc}');if(!f||!f.elements)return;\
var cs=f.elements;\
for(var i=0;i<cs.length;i++){{var c=cs[i],t=c.tagName;\
if(t==='TEXTAREA'||t==='OUTPUT')c.value=c.defaultValue;\
else if(t==='INPUT'){{var k=c.type;\
if(k==='checkbox'||k==='radio')c.checked=c.defaultChecked;\
else if(k!=='submit'&&k!=='reset'&&k!=='button'&&k!=='image'&&k!=='file')c.value=c.defaultValue;\
}}else if(t==='SELECT'){{var os=c.options;for(var j=0;os&&j<os.length;j++)os[j].selected=os[j].defaultSelected;}}\
if(typeof __zw_clear_user_edited==='function')__zw_clear_user_edited(c);\
}}\
}})()"
    )
}

/// 构造 checkbox 用户激活的 checkedness 更新脚本。
///
/// 必须经 IDL `.checked=` setter，而不是宿主直接改内容属性；setter 会捕获
/// dirty checkedness 对应的 `defaultChecked` 基线，供后续 `form.reset()` 恢复。
/// https://html.spec.whatwg.org/multipage/input.html#checkbox-state-(type=checkbox)
pub fn script_toggle_checkbox_checked(selector: &str) -> String {
    let esc = escape_js_string(selector);
    format!("(function(){{var e=document.querySelector('{esc}');if(e)e.checked=!e.checked;}})()")
}

/// 构造 radio 用户激活的 checkedness 更新脚本。
///
/// 同 name 组成员均经 IDL `.checked=` setter 更新，确保每个成员保留各自的
/// defaultChecked 基线；目标最终 checked，其他同组成员 unchecked。
/// https://html.spec.whatwg.org/multipage/input.html#radio-button-state-(type=radio)
pub fn script_select_radio_checked(selector: &str) -> String {
    let esc = escape_js_string(selector);
    format!(
        "(function(){{\
var t=document.querySelector('{esc}');if(!t||t.checked)return;\
var n=t.getAttribute('name'),rs=document.querySelectorAll('input[type=radio]');\
for(var i=0;i<rs.length;i++){{var r=rs[i];if(r!==t&&n!==null&&r.getAttribute('name')===n)r.checked=false;}}\
t.checked=true;\
}})()"
    )
}

/// 构造设置 checkbox/radio checkedness 的宿主脚本，不派发事件。
pub fn script_set_control_checked(selector: &str, checked: bool) -> String {
    let esc = escape_js_string(selector);
    let checked = if checked { "true" } else { "false" };
    format!("(function(){{var e=document.querySelector('{esc}');if(e)e.checked={checked};}})()")
}

/// 构造 option selectedness 更新脚本，不派发事件。
///
/// `clear_others` 用于 select-one，按节点身份清除兄弟，避免重复 value 选错 option。
/// https://html.spec.whatwg.org/multipage/form-elements.html#concept-option-selectedness
pub fn script_set_option_selected(
    option_selector: &str,
    select_selector: &str,
    selected: bool,
    clear_others: bool,
) -> String {
    let option = escape_js_string(option_selector);
    let select = escape_js_string(select_selector);
    let selected = if selected { "true" } else { "false" };
    let clear_others = if clear_others { "true" } else { "false" };
    format!(
        "(function(){{\
var o=document.querySelector('{option}'),s=document.querySelector('{select}');if(!o||!s)return;\
if({clear_others}){{var os=s.options;for(var i=0;i<os.length;i++)os[i].selected={selected}&&os[i]===o;}}\
else o.selected={selected};\
}})()"
    )
}

/// 构造 details/dialog open 状态更新脚本，不派发事件。
pub fn script_set_open(selector: &str, open: bool) -> String {
    let selector = escape_js_string(selector);
    let open = if open { "true" } else { "false" };
    format!("(function(){{var e=document.querySelector('{selector}');if(e)e.open={open};}})()")
}

/// 构造「设置 location.hash」的 shim 脚本（P1a 导航，R3053）。宿主在 `<a href="#...">` 被 click 时执行：
/// 调 shim `location.hash = hash`（R3006：更新 hash + history entry + 派 hashchange 事件 + 触 onhashchange）。
/// headless 无 viewport → 不滚动到锚。hash 经 `escape_js_string` 安全嵌入。
pub fn script_call_set_location_hash(hash: &str) -> String {
    let esc = escape_js_string(hash);
    format!("location.hash='{esc}';")
}

/// 构造「向焦点 input/textarea 注入一个文本字符」的 shim 脚本（P1a form input）。
/// 宿主在 keydown 可打印字符时执行：shim `__zw_text_input(sel, ch)` 把字符 append 到 value
/// （`.value` set 更新缓存 + 记 value 属性 mutation）并派发 'input' 事件。非 input/textarea → no-op。
pub fn script_text_input(selector: &str, key: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_ch = escape_js_string(key);
    format!("__zw_text_input('{esc_sel}', '{esc_ch}')")
}

/// 构造不派发 `input` listener 的 UA 文本插入脚本。
///
/// 仅用于页面 JavaScript 禁用路径；IDL value/selection 状态仍由用户代理更新。
/// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-textarea/input-relevant-value
pub fn script_text_input_without_event(selector: &str, text: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_text = escape_js_string(text);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
if(!e||(e.tagName!=='INPUT'&&e.tagName!=='TEXTAREA'))return;\
var v=String(e.value||''),s=Number(e.selectionStart),n=Number(e.selectionEnd);\
if(!Number.isFinite(s))s=v.length;if(!Number.isFinite(n))n=s;\
s=Math.max(0,Math.min(v.length,s));n=Math.max(s,Math.min(v.length,n));\
var x='{esc_text}';e.value=v.slice(0,s)+x+v.slice(n);\
var c=s+x.length;if(typeof e.setSelectionRange==='function')e.setSelectionRange(c,c);\
}})()"
    )
}

/// 构造「Backspace 删末字符」的 shim 脚本（P1a form input 编辑互补）。宿主在 keydown
/// Backspace 时执行：shim `__zw_text_delete(sel)` 删 value 末字符并派发 'input' 事件。
pub fn script_text_delete(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_text_delete('{esc_sel}')")
}

/// 构造不派发 `input` listener 的 UA Backspace 脚本。
///
/// 选区非空时删除选区；否则删除 caret 前一个 UTF-16 code unit。
/// https://w3c.github.io/input-events/#input-event-order-during-user-initiated-editing
pub fn script_text_delete_without_event(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
if(!e||(e.tagName!=='INPUT'&&e.tagName!=='TEXTAREA'))return;\
var v=String(e.value||''),s=Number(e.selectionStart),n=Number(e.selectionEnd);\
if(!Number.isFinite(s))s=v.length;if(!Number.isFinite(n))n=s;\
s=Math.max(0,Math.min(v.length,s));n=Math.max(s,Math.min(v.length,n));\
if(s===n){{if(s===0)return;s--;}}\
e.value=v.slice(0,s)+v.slice(n);\
if(typeof e.setSelectionRange==='function')e.setSelectionRange(s,s);\
}})()"
    )
}

/// 构造设置文本控件 live value 与 UTF-16 selection 的宿主脚本，不派发事件。
pub fn script_set_text_control_state(
    selector: &str,
    value: &str,
    selection_start: usize,
    selection_end: usize,
) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_value = escape_js_string(value);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
if(!e||(e.tagName!=='INPUT'&&e.tagName!=='TEXTAREA'))return;\
e.value='{esc_value}';\
if(typeof e.setSelectionRange==='function')e.setSelectionRange({selection_start},{selection_end});\
if(typeof __zw_mark_user_edited==='function')__zw_mark_user_edited('{esc_sel}');\
}})()"
    )
}

/// 构造只更新文本控件 UTF-16 selection 的宿主脚本，不修改 live value 或派发事件。
///
/// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#textFieldSelection
pub fn script_set_text_control_selection(selector: &str, selection_start: usize, selection_end: usize) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
if(!e||(e.tagName!=='INPUT'&&e.tagName!=='TEXTAREA'))return;\
if(typeof e.setSelectionRange==='function')e.setSelectionRange({selection_start},{selection_end});\
}})()"
    )
}

/// 读取文本控件当前 value 与 DOM 选区，返回 JSON 数组字符串。
///
/// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#textFieldSelection
pub fn script_text_control_snapshot(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var el=document.querySelector('{esc_sel}');\
if(!el)return '';\
return JSON.stringify([String(el.value||''),Number(el.selectionStart||0),Number(el.selectionEnd||0)]);}})()"
    )
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
/// （[`PAGE_SCRIPT_ERROR_GLOBAL`]）而非污染持久 V8 Isolate；并在执行期设/清 `document.currentScript`
///（HTML §4.11.3.1：classic 脚本执行期间 currentScript 指向自身元素）。
///
/// **背景**：persistent_context 模式跨 execute 复用同一 Isolate。页面脚本抛出的未捕获异常若直达
/// V8，embedder 侧 `TryCatch::reset()` 在当前 rusty_v8（150.2.0）下无法清掉跨 execute 的 pending
/// exception——下一条 execute 的新 TryCatch 会观测到它并返回 "Runtime error: null"，使**页面上任何
/// 抛错的 `<script>` 都会废掉其后所有脚本**，并使 host 的 window.onerror 报告（R2940）失效。
/// 在页面脚本层包 try-catch：throw 被这里捕获→调用方读 sentinel 得 Err→`run_page_scripts` 据此
/// 报 window.onerror，且 Isolate 保持干净。
///
/// **currentScript**：执行前 `__zw_set_current_script(script_index)` 设索引（该脚本在全部 `<script>`
/// 元素中的文档序，与 shim `getElementsByTagName('script')` 对齐），`finally` 块无条件 `__zw_clear_current_script()`
/// 清（即便抛错也清，保证脚本执行期外 currentScript 恒 null）。module 脚本不经本函数（spec：module
/// currentScript 恒 null），调用方仅在 classic 分支调用。`script_index` 由 [`extract_page_scripts_indexed`]
///（zero_engine）提供。
///
/// **作用域**：`code` 内的 `var`/`function` 声明提升到脚本顶层作用域（try 块对它们透明），与未包装
/// 行为一致；顶层 `let`/`const`/`class` 会变为 try 块作用域——classic 内联脚本罕见，module 走
/// `execute_module`。成功时 sentinel 留 `undefined`（非字符串），抛错时设为消息字符串，二者经
/// [`page_script_error_check`] 的 `===undefined` 判别可靠区分（即便 `throw undefined` 也只产生
/// 字符串 "undefined"，不与 undefined 值混淆）。
pub fn script_run_classic_page(code: &str, script_index: usize) -> String {
    format!(
        "globalThis.__zw_set_current_script&&globalThis.__zw_set_current_script({idx});\nglobalThis.{g}=undefined;\ntry{{\n{code}\n}}catch(__zw_e){{globalThis.{g}=(__zw_e&&__zw_e.message)?String(__zw_e.message):String(__zw_e);}}\nfinally{{globalThis.__zw_clear_current_script&&globalThis.__zw_clear_current_script();}}",
        idx = script_index,
        g = PAGE_SCRIPT_ERROR_GLOBAL
    )
}

/// `document.currentScript` 设索引 shim 调用串（R3258）：`__zw_set_current_script(idx)`（typeof 守卫，
/// shim 未安装时 no-op）。供不走 sentinel 包装的 classic 执行路径（webview/reftest 进程内路径）在
/// 脚本体执行前调用。`idx` = 脚本在全部 `<script>` 元素中的文档序（[`extract_page_scripts_indexed`]）。
pub fn script_set_current_script(script_index: usize) -> String {
    format!(
        "if(typeof __zw_set_current_script==='function')__zw_set_current_script({i});",
        i = script_index
    )
}

/// `document.currentScript` 清 shim 调用串（R3258）：`__zw_clear_current_script()`（typeof 守卫）。供
/// classic 执行路径在脚本体执行后调用（与 [`script_set_current_script`] 配对）。建议置于 `finally` 块
/// 保证即便抛错也清。
pub fn script_clear_current_script() -> &'static str {
    "if(typeof __zw_clear_current_script==='function')__zw_clear_current_script();"
}

/// 读取 [`script_run_classic_page`] 写入的 sentinel：返回空串表示成功（无抛错），非空串为错误消息。
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
