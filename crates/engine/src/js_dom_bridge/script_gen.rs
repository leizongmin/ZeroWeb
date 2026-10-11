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
    /// `KeyboardEvent.shiftKey`。
    pub shift_key: bool,
    /// `KeyboardEvent.ctrlKey`。
    pub ctrl_key: bool,
    /// `KeyboardEvent.altKey`。
    pub alt_key: bool,
    /// `KeyboardEvent.metaKey`。
    pub meta_key: bool,
    /// `SubmitEvent.submitter`——触发 submit 的按钮唯一选择器（R2984）。click submit button → 该按钮；
    /// Enter 隐式提交 → None（spec：表单默认提交按钮或 null）。
    pub submitter: Option<String>,
    /// MouseEvent.clientX——视口 CSS 坐标（CDP `Input.dispatchMouseEvent` → DOM 指针事件；
    /// Playwright hit-target 拦截器经 `event.clientX/Y` 复核命中点，缺省 undefined 会被判
    /// 未命中 → "html intercepts pointer events"）。
    pub client_x: Option<f32>,
    /// MouseEvent.clientY——同上。
    pub client_y: Option<f32>,
    /// MouseEvent.button——按下的指针键（uievents-compat M2 片 1；-1 = move 无变化）。
    pub button: Option<i16>,
    /// MouseEvent.buttons——按下键位掩码（down=1、up/move=0）。
    pub buttons: Option<u16>,
    /// UIEvent.detail——点击计数（dblclick=2）。
    pub detail: Option<u32>,
    /// MouseEvent.relatedTarget——over/out/enter/leave 对侧元素唯一选择器。
    pub related_target: Option<String>,
    /// Event.bubbles——缺省由 shim 分支按事件类型决定（enter/leave false）；显式覆盖。
    pub bubbles: Option<bool>,
    /// PointerEvent.pointerType——缺省 'mouse'。
    pub pointer_type: Option<String>,
    /// PointerEvent.pointerId——uievents-compat 尾簇 11：None = 缺省（UA 指针 1）；
    /// Some(-1) = 非指针生成（click() API / Enter 激活，PE spec）。
    pub pointer_id: Option<i32>,
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
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
            let shift_key = d.shift_key;
            let ctrl_key = d.ctrl_key;
            let alt_key = d.alt_key;
            let meta_key = d.meta_key;
            let client_x = d.client_x.map(|v| format!("{v}")).unwrap_or_else(|| "null".to_string());
            let client_y = d.client_y.map(|v| format!("{v}")).unwrap_or_else(|| "null".to_string());
            // uievents-compat M2 片 1：指针字段透传（null = 缺省由 shim 分支按事件类型定）。
            let button = d.button.map(|v| format!("{v}")).unwrap_or_else(|| "null".to_string());
            let buttons = d.buttons.map(|v| format!("{v}")).unwrap_or_else(|| "null".to_string());
            let detail_count = d.detail.map(|v| format!("{v}")).unwrap_or_else(|| "null".to_string());
            let related_target = d
                .related_target
                .as_deref()
                .map(|s| format!("'{}'", escape_js_string(s)))
                .unwrap_or_else(|| "null".to_string());
            let bubbles = d.bubbles.map(|v| format!("{v}")).unwrap_or_else(|| "null".to_string());
            let pointer_type = d
                .pointer_type
                .as_deref()
                .map(|s| format!("'{}'", escape_js_string(s)))
                .unwrap_or_else(|| "null".to_string());
            // uievents-compat 尾簇 11：pointerId 透传（null = shim 分支缺省——UA 指针 1）。
            let pointer_id = d
                .pointer_id
                .map(|v| format!("{v}"))
                .unwrap_or_else(|| "null".to_string());
            format!(
                "{{key:{key},code:{code},submitter:{submitter},data:{data},inputType:{input_type},isComposing:{is_composing},shiftKey:{shift_key},ctrlKey:{ctrl_key},altKey:{alt_key},metaKey:{meta_key},clientX:{client_x},clientY:{client_y},button:{button},buttons:{buttons},detail:{detail_count},relatedTarget:{related_target},bubbles:{bubbles},pointerType:{pointer_type},pointerId:{pointer_id}}}"
            )
        }
    };
    format!("__zw_dispatch_event('{esc_sel}', '{esc_ty}', {detail_json})")
}

/// 构造「设置 document.readyState 状态宿」的脚本（t8m，spec `dom-document-readystate`）。
/// 写 shim 全局 `__zwReadyState`——页面可见 shim document 的 readyState getter（part06.js）
/// 读该全局（`dom_bindings/document.rs` native 模板不读它，按 run_script 模型保持固定值），
/// 未注入/非字符串/非规范三态值缺省 "complete"。宿主在页面脚本阶段起点提交
///（state="loading"），与首条页面脚本同执行通道 FIFO 保序（renderer 走
/// `execute_script_direct_priority` 同步执行，保证先于阶段脚本）。
pub fn script_set_ready_state(state: &str) -> String {
    let st = escape_js_string(state);
    format!("try{{globalThis.__zwReadyState='{st}';}}catch(_e){{}}")
}

/// 构造「readyState 过渡 + document readystatechange 派发」的原子单提交脚本（t8m，
/// spec HTML §the end：readystatechange 于 DOMContentLoaded 后（"interactive"）与
/// window load 前（"complete"）各派发一次，fires at the Document）。
/// 赋值与派发在同一脚本串内顺序完成，保证 handler 内读到的 readyState 与过渡值一致、
/// 事件恰一次；readystatechange 不冒泡不可取消（shim `__zw_dispatch_event` 的
/// readystatechange 分支硬编码 bubbles:false/cancelable:false）。
pub fn script_transition_ready_state(state: &str) -> String {
    format!(
        "{};{}",
        script_set_ready_state(state),
        script_dispatch_dom_event("html", "readystatechange", None)
    )
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
        "(function(){{var _e=document.querySelector('{sel}');if(_e){{try{{if(typeof _zwQueryWrapIdentity==='function')_e=_zwQueryWrapIdentity('{sel}');_e.dispatchEvent(new TransitionEvent('{ty}',{{propertyName:'{prop}',elapsedTime:{elapsed_str},bubbles:true}}));}}catch(_x){{}}\
         try{{if(typeof __zw_dispatch_prefixed_alias==='function')__zw_dispatch_prefixed_alias(_e,'{ty}','{prop}',{elapsed_str});}}catch(_x2){{}}}}}})();"
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
        "(function(){{var _e=document.querySelector('{sel}');if(_e){{try{{if(typeof _zwQueryWrapIdentity==='function')_e=_zwQueryWrapIdentity('{sel}');_e.dispatchEvent(new AnimationEvent('{ty}',{{animationName:'{nm}',elapsedTime:{elapsed_str},bubbles:true}}));}}catch(_x){{}}\
         try{{if(typeof __zw_dispatch_prefixed_alias==='function')__zw_dispatch_prefixed_alias(_e,'{ty}','{nm}',{elapsed_str});}}catch(_x2){{}}}}}})();"
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

/// 生成「合成指针悬停迁移」脚本（uievents-compat M2 片 1，2026-10-03）。宿主在
/// 指针悬停目标变化时执行：调 shim 钩子 `__zw_pointer_move(sel, x, y)`（part06.js）
/// ——跨界时派发 over/out/enter/leave 边界序（relatedTarget 对侧；enter/leave 不冒泡），
/// 随后在新目标上派 pointermove/mousemove 对。宿主侧悬停态自持（webview `pointer_over`）。
pub fn script_pointer_move(selector: &str, client_x: f32, client_y: f32) -> String {
    let sel = escape_js_string(selector);
    format!("if(typeof __zw_pointer_move==='function')__zw_pointer_move('{sel}',{client_x},{client_y});")
}

/// 生成「mutation 驱动悬停重结算」脚本（uievents-compat 尾簇 8）。runner 在探测环
/// 发现悬停失效（`__zw_ptr_hover_dirty()`）并以 fresh gBCR 命中测试后调用：调 shim
/// `__zw_mut_hover_settle(sel, x, y)`（part06.js）——命中与现悬停不同 → 补跨界序
/// （双层 pointer/compat mouse；只派边界序不派 move 对——spec 的 mutation 触发
/// 重算不合成 move）。
pub fn script_mut_hover_settle(selector: &str, client_x: f32, client_y: f32) -> String {
    let sel = escape_js_string(selector);
    format!("if(typeof __zw_mut_hover_settle==='function')__zw_mut_hover_settle('{sel}',{client_x},{client_y});")
}

/// 生成「Actions down 步序列」脚本（uievents-compat M3，2026-10-03）。runner 把
/// Actions 链逐步重放为宿主命令——down 步调 shim
/// `__zw_pointer_down_sequence(sel, x, y, pointerType, button)`（part06.js）：悬停迁移
/// 跨界序 → pointerdown →（未取消时）mousedown → [contextmenu（右键）]；页内
/// listener 在 pointerdown 里 setPointerCapture 后，后续 move/up 步按捕获路由。
pub fn script_pointer_down_sequence(
    selector: &str,
    client_x: f32,
    client_y: f32,
    pointer_type: &str,
    button: i16,
) -> String {
    let sel = escape_js_string(selector);
    let pty = escape_js_string(pointer_type);
    format!(
        "if(typeof __zw_pointer_down_sequence==='function')__zw_pointer_down_sequence('{sel}',{client_x},{client_y},'{pty}',{button});"
    )
}

/// 生成「Actions up 步序列」脚本（uievents-compat M3）。up 步调 shim
/// `__zw_pointer_up_sequence(upSel, downSel, x, y, pointerType, button, chain)`（part06.js）：
/// pointerup →（未取消时）mouseup → click/auxclick 组合（同目标连击 dblclick；
/// 跨目标 click@最近公共祖先；非主键 auxclick）。downSel = down 步的命中元素选择器
/// （click 组合的 down 侧落点；空串回落 shim 内记录值）。chain = up 落点祖先选择器
/// 链（'|' 分隔，近祖优先——touch 抬起悬停拆除的 leave 锚回退）。
pub fn script_pointer_up_sequence(
    up_selector: &str,
    down_selector: &str,
    client_x: f32,
    client_y: f32,
    pointer_type: &str,
    button: i16,
    ancestor_chain: &str,
) -> String {
    let up = escape_js_string(up_selector);
    let down = escape_js_string(down_selector);
    let pty = escape_js_string(pointer_type);
    let chain = escape_js_string(ancestor_chain);
    format!(
        "if(typeof __zw_pointer_up_sequence==='function')__zw_pointer_up_sequence('{up}','{down}',{client_x},{client_y},'{pty}',{button},'{chain}');"
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

/// 构造「宿主焦点治理」脚本（slice22 focus governance）：宿主驱动焦点迁移（mousedown 默认
/// 动作 / Tab 焦点导航）调 shim 内部钩子 `__zw_host_focus(sel)`（获焦相位：状态同步、派发
/// focus 与 focusin）或 `__zw_host_blur(sel)`（失焦相位：清状态、派发 focusout 与 blur），
/// 一次执行完成「页面可见焦点状态同步（`document.activeElement` 读的 shim `_activeElKey`）
/// 与该相位焦点事件派发」。此前宿主路径只派事件不更状态 → `document.activeElement` 停留在
/// 页面 JS 最后 `focus()` 的元素（「焦点状态报告」与「事件落点」分离）。
///
/// 规范锚：HTML §6.5.2 focusing steps——焦点迁移先更 focused area 再派焦点事件族
/// <https://html.spec.whatwg.org/multipage/interaction.html#focusing-steps>；focus 是 mousedown
/// 的默认动作（UI Events §5.2.2 <https://w3c.github.io/uievents/#focus-event-focus>）。
/// `focus` 为 true 获焦相位、false 失焦相位；事件流与旧 `script_dispatch_dom_event` 逐字节
/// 同通道（shim `__zw_dispatch_event` 同一 UA 通道）。
pub fn script_host_focus(selector: &str, focus: bool) -> String {
    let esc_sel = escape_js_string(selector);
    let hook = if focus { "__zw_host_focus" } else { "__zw_host_blur" };
    format!("if(typeof {hook}==='function'){hook}('{esc_sel}');")
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

/// 构造「提交资源元素最终状态」的 shim 脚本。`outcome` 为 `loaded` / `available` /
/// `error`；图片成功时携带固有像素尺寸。shim 负责更新元素 IDL 状态并按元素类型派发规范事件。
/// https://html.spec.whatwg.org/multipage/embedded-content.html#the-img-element
/// https://html.spec.whatwg.org/multipage/media.html#media-elements
pub fn script_commit_resource_element_state(
    tag: &str,
    abs_url: &str,
    outcome: &str,
    natural_width: u32,
    natural_height: u32,
    media_duration_ms: Option<u64>,
) -> String {
    let esc_tag = escape_js_string(tag);
    let esc_url = escape_js_string(abs_url);
    let esc_outcome = escape_js_string(outcome);
    // media-playback M2a：video 容器时长真值（毫秒；None → 'null'——shim 回落
    // headless 近似 duration，测试面零回归）。
    let duration = match media_duration_ms {
        Some(ms) => ms.to_string(),
        None => "null".to_string(),
    };
    format!(
        "__zw_commit_resource_element_state('{esc_tag}', '{esc_url}', '{esc_outcome}', {natural_width}, {natural_height}, {duration})"
    )
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

/// 构造「派发 `securitypolicyviolation` 事件」的 shim 脚本（security-hardening M2-s1，
/// spec CSP3 §report-the-violation——被 CSP 阻止的脚本在其执行点派发违规事件）。
///
/// 调 shim 全局 `__zw_dispatch_securitypolicyviolation`（part06.js：`_makeEvent` 事件 +
/// 字段自有属性 + document 站 `_dispatchWithBubble`，AT_TARGET tgt='doc' + bubble 上行
/// window——document/window watcher 双达；native 构造实例过不了 shim 站内检查，实测
/// R2-s1 探针，故事件形态走 shim 侧）。lineNumber/columnNumber 本切片未定位
///（FIXME M2-s2：extract 侧携带源位置），置 0。字段经 [`escape_js_string`] 安全嵌入。
pub fn script_dispatch_securitypolicyviolation(
    script_ordinal: usize,
    document_uri: &str,
    effective_directive: &str,
    original_policy: &str,
    blocked_uri: &str,
    line: u32,
    column: u32,
) -> String {
    let esc_doc_uri = escape_js_string(document_uri);
    let esc_directive = escape_js_string(effective_directive);
    let esc_policy = escape_js_string(original_policy);
    let esc_blocked = escape_js_string(blocked_uri);
    format!(
        "(function(){{try{{\
if(typeof __zw_dispatch_securitypolicyviolation!=='function')return;\
__zw_dispatch_securitypolicyviolation({{\
scriptOrdinal:{script_ordinal},\
documentURI:'{esc_doc_uri}',\
effectiveDirective:'{esc_directive}',\
originalPolicy:'{esc_policy}',\
blockedURI:'{esc_blocked}',\
lineNumber:{line},columnNumber:{column}}});\
}}catch(_eSpv){{}}}})()"
    )
}

/// 顶层 try-catch 包装捕获的页面脚本错误所写入的 sentinel 全局名。包装器在成功时将其留为
/// `undefined`，抛错时设为错误消息字符串。调用方经 [`page_script_error_check`] 读取。
pub const PAGE_SCRIPT_ERROR_GLOBAL: &str = "__zw_pgerr__";

/// `var` 声明的 accessor 转导出片段（R201）：get/set 双向转发 eval 绑定——后续脚本
/// 读 `globalThis.NAME` 得当前值，写落回 eval 绑定（WPT dom/common.js 的跨脚本再赋值
/// 流）。仅 strict eval 使用：非 strict 的间接 eval 里 var 本已泄漏为全局数据属性，
/// accessor 重定义会换成 getter，getter 内 `return NAME` 解析到全局属性 = accessor
/// 自身 → 无限递归（lit e2e template_content_fragment_view 回归实证）。
fn var_accessor_export(name: &str) -> String {
    format!(
        "try{{Object.defineProperty(globalThis,'{name}',{{configurable:true,get:function(){{return {name};}},set:function(_zw_v){{{name}=_zw_v;}}}});}}catch(_zw_ex){{}}"
    )
}

/// classic 脚本源首指令序言的 'use strict' 判定（Directive Prologue，
/// https://tc39.es/ecma262/#sec-directive-prologues-early-errors）：前导空白、
/// `//` 行注释与 `/* */` 块注释（含跨行）不产生语句；序言内出现 'use strict'
/// （单/双引号，其后为空或 `;` 起始）即 strict；其他字符串字面量指令（如
/// `"use x";`）之后序言继续，遇非指令语句终止。
///
/// bing-t2（site-optimizer 2026-10-10）：旧「首非空行恰为指令」漏掉前导注释形态
/// ——bing 全站内联脚本 `//<![CDATA[\n"use strict";` 起头，V8 按规范进入 strict
/// eval（顶层 var 困独立变量环境）而导出机制不启动 → `_w/_d/_ge` 全局蒸发、下游
/// 全站 ReferenceError 级联（离线 fixture 复现；真浏览器对照 Chrome `_w`=object）。
fn script_directive_is_strict(code: &str) -> bool {
    let mut in_block_comment = false;
    for line in code.lines() {
        let mut rest = line.trim();
        if in_block_comment {
            match rest.find("*/") {
                Some(end) => {
                    in_block_comment = false;
                    rest = rest[end + 2..].trim_start();
                    if rest.is_empty() {
                        continue;
                    }
                }
                None => continue,
            }
        }
        // 剥离行首交替注释前缀（`/*a*/ /*b*/ "use strict";`）；行注释覆盖至行尾。
        loop {
            if rest.starts_with("//") {
                rest = "";
                break;
            }
            if let Some(stripped) = rest.strip_prefix("/*") {
                match stripped.find("*/") {
                    Some(end) => {
                        rest = stripped[end + 2..].trim_start();
                    }
                    None => {
                        in_block_comment = true;
                        rest = "";
                        break;
                    }
                }
            } else {
                break;
            }
        }
        if rest.is_empty() {
            continue;
        }
        // 候选指令语句：引号起始的字符串字面量；非字符串语句 = 序言终止。
        let quote = match rest.as_bytes().first() {
            Some(b'\'') => '\'',
            Some(b'"') => '"',
            _ => return false,
        };
        let after = &rest[1..];
        let Some(close) = after.find(quote) else {
            return false; // 未闭合（跨行字符串非指令）
        };
        let directive = &after[..close];
        let tail = after[close + 1..].trim_start();
        let directive_like = tail.is_empty() || tail.starts_with(';');
        if directive == "use strict" {
            return directive_like;
        }
        // 字符串后接非 `;` 语法（拼接/调用）不是指令——序言终止。
        if !directive_like {
            return false;
        }
    }
    false
}

/// bing-t2（site-optimizer 2026-10-10）：classic 脚本顶层声明导出的**单遍字符
/// 状态机扫描**，取代旧行首零缩进锚定——minified 脚本全部顶层声明在单行中部
/// （bing #6 的 `};;var _w=window,_d=document,...` @col 26588），行锚扫描全漏。
///
/// 状态：括号深度 `(`/`[`/`{`、单/双引号字符串、模板字面量（`${}` 花括号计数）、
/// 行/块注释、正则字面量（除法/正则歧义按前一有效字符 + 关键字启发式——误判
/// 最坏漏一个导出或经 try 包裹静默 no-op，不改变脚本自身执行语义）。语句边界 =
/// 深度 0 且前一有效字符为起点/`;`/`}`/`)`（`}` 覆盖 if/for/try 块尾，`)` 覆盖
/// `if(x)var y=1;` 无大括号臂）。
///
/// 导出语义逐项沿用既有回执：var 仅 strict 时 accessor 转发（R201 非 strict 数据
/// 属性自递归教训）；const 值快照；let accessor 转发（M2-S2 跨脚本递减可见性）；
/// function/async function/class 恒值导出（R147/R3254/WAB2-M1 的 WPT 依据）；
/// 每名 try 包裹（lit bundle IIFE 内部同名消亡不炸整个后缀）；同名去重。
/// 已知接受缺口（保守方向=漏导出）：`for(var i=..)` 头内声明（深度>0）、
/// `label: var`、顶层块内 var（`{ var x }`——真实浏览器 var 穿块全局，旧行锚
/// 会导出，状态机按深度判定不导出）与解构声明符（`var {x} = lib`——弃链，
/// 见 Name 相 D1 注记）不导出；const/let 多声明符只发布首名；`)` 后 `/` 按正则
/// 处理（`(a)/2` 形态漏导出，正则内容当代码的风险反向更重——D4 权衡）。
fn scan_top_level_decl_exports(code: &str, is_strict: bool) -> String {
    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '$'
    }
    /// 正则字面量可出现的关键字前缀（`return /x/` vs `a / b`）。
    const REGEX_KEYWORDS: [&str; 13] = [
        "return",
        "typeof",
        "instanceof",
        "in",
        "of",
        "new",
        "delete",
        "void",
        "throw",
        "case",
        "do",
        "yield",
        "await",
    ];
    fn skip_ws_comments(src: &[char], i: &mut usize) {
        let n = src.len();
        loop {
            while *i < n && src[*i].is_whitespace() {
                *i += 1;
            }
            if *i + 1 < n && src[*i] == '/' && src[*i + 1] == '/' {
                while *i < n && src[*i] != '\n' {
                    *i += 1;
                }
                continue;
            }
            if *i + 1 < n && src[*i] == '/' && src[*i + 1] == '*' {
                *i += 2;
                while *i + 1 < n && !(src[*i] == '*' && src[*i + 1] == '/') {
                    *i += 1;
                }
                *i = (*i + 2).min(n);
                continue;
            }
            break;
        }
    }
    /// var 链解析的子阶段：Name=期待声明符名；End=初始化器/表达式（期待深度 0 的
    /// `,` 推进或 `;`/EOF 终结）。
    enum VarPhase {
        Name,
        End,
    }
    fn push_export(out: &mut Vec<String>, seen: &mut std::collections::HashSet<String>, name: &str, accessor: bool) {
        // D3（缺陷首轮）：导出名须是合法 JS 标识符（首字符非数字）——误读的正则/
        // 模式内容可能产生 `2` 之类伪名，值形式 `globalThis.2=2` 是语法错误，会
        // 击杀整个 eval 源（脚本级回归）。
        let mut chars = name.chars();
        let legal = match chars.next() {
            Some(first) => first.is_ascii_alphabetic() || first == '_' || first == '$',
            None => false,
        } && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$');
        if !legal || !seen.insert(name.to_string()) {
            return;
        }
        if accessor {
            out.push(var_accessor_export(name));
        } else {
            out.push(format!("try{{globalThis.{name}={name};}}catch(_zw_ex){{}}"));
        }
    }

    let src: Vec<char> = code.chars().collect();
    let n = src.len();
    let mut out: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0usize;
    let mut depth: i32 = 0;
    let mut prev_sig: char = '\0'; // 前一有效字符（注释/空白透明）
    let mut last_ident = String::new();
    // ASI：自上一有效 token 以来是否出现过换行（含块注释内换行）——声明关键字
    // 不可能延续表达式，操作数结尾（'x'/`)`/`]`）+ 换行即语句边界（WPT
    // prefixed-animation-event-tests.js 形态：`'use strict'` 无分号 + 注释块 +
    // 换行后的缩进 `function name(`——行锚旧扫描按行首可命中，语句边界判定须补）。
    let mut saw_newline = false;
    // var 链模式：Some((base, phase))——base 为链起始深度，声明符按深度 0（相对 base）
    // 的 `,` 分隔，`;` 消费终结，其他有效字符按 ASI 终结（不消费）。
    let mut var_phase: Option<(i32, VarPhase)> = None;

    while i < n {
        let c = src[i];
        match c {
            '/' if i + 1 < n && src[i + 1] == '/' => {
                while i < n && src[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < n && src[i + 1] == '*' => {
                i += 2;
                let start = i;
                while i + 1 < n && !(src[i] == '*' && src[i + 1] == '/') {
                    i += 1;
                }
                if src[start..i].contains(&'\n') {
                    saw_newline = true;
                }
                i = (i + 2).min(n);
            }
            '\'' | '"' => {
                let q = c;
                i += 1;
                while i < n {
                    if src[i] == '\\' {
                        i += 2;
                        continue;
                    }
                    if src[i] == q || src[i] == '\n' {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                prev_sig = 'x';
                last_ident.clear();
                saw_newline = false;
            }
            '`' => {
                // 模板字面量整体消费到匹配反引号；${} 花括号计数。串内嵌套模板/
                // ${} 内含花括号字符串的极端组合误读方向 = 漏导出（保守），不伪导出。
                i += 1;
                let mut braces = 0i32;
                while i < n {
                    let t = src[i];
                    if t == '\\' {
                        i += 2;
                        continue;
                    }
                    if t == '`' && braces == 0 {
                        i += 1;
                        break;
                    }
                    if t == '$' && i + 1 < n && src[i + 1] == '{' {
                        braces += 1;
                        i += 2;
                        continue;
                    }
                    if t == '{' {
                        braces += 1;
                    } else if t == '}' {
                        braces -= 1;
                    }
                    i += 1;
                }
                prev_sig = 'x';
                last_ident.clear();
                saw_newline = false;
            }
            '/' => {
                // 除法 vs 正则：正则出现于表达式位置（前一有效字符非操作数结尾）或
                // 关键字后。`)`（if 头尾）按正则处理——误判方向 = 多跳一段代码漏导出
                // （保守），避免把正则内容当代码产生伪 accessor。
                let regex_pos = matches!(
                    prev_sig,
                    '\0' | ';'
                        | '('
                        | ')'
                        | '{'
                        | '}'
                        | ','
                        | '='
                        | ':'
                        | '['
                        | '!'
                        | '&'
                        | '|'
                        | '?'
                        | '+'
                        | '-'
                        | '*'
                        | '%'
                        | '<'
                        | '>'
                        | '~'
                        | '^'
                ) || REGEX_KEYWORDS.contains(&last_ident.as_str());
                if regex_pos {
                    i += 1;
                    let mut in_class = false;
                    while i < n {
                        let r = src[i];
                        if r == '\\' {
                            i += 2;
                            continue;
                        }
                        if r == '\n' {
                            break; // 正则不跨行——保守退出
                        }
                        if r == '[' {
                            in_class = true;
                        } else if r == ']' {
                            in_class = false;
                        } else if r == '/' && !in_class {
                            i += 1;
                            while i < n && src[i].is_ascii_alphabetic() {
                                i += 1; // flags
                            }
                            break;
                        }
                        i += 1;
                    }
                    prev_sig = 'x';
                    last_ident.clear();
                    saw_newline = false;
                } else {
                    i += 1;
                    prev_sig = '/';
                    last_ident.clear();
                    saw_newline = false;
                }
            }
            '(' | '[' | '{' => {
                // D1（缺陷首轮）：Name 相遇 `{`/`[` = 解构声明符模式起点
                // （`var { x } = lib` / `var [a] = arr`——合法 var 声明符不可能是
                // 括号开头，成员表达式 `var a[0]` 亦非法）——弃链（模式绑定名不
                // 导出，保守漏导出；否则模式后 `}` `=` 的 RHS 首标识符会命中 Name
                // 相被伪导出为 accessor——实测毒化 globalThis 自身致全页读取栈溢出）。
                if let Some((base, phase)) = var_phase.as_ref()
                    && matches!(*phase, VarPhase::Name)
                    && depth == *base
                {
                    var_phase = None;
                }
                depth += 1;
                prev_sig = c;
                last_ident.clear();
                saw_newline = false;
                i += 1;
            }
            ')' | ']' | '}' => {
                depth = (depth - 1).max(0);
                prev_sig = c;
                last_ident.clear();
                saw_newline = false;
                i += 1;
            }
            _ if c.is_whitespace() => {
                if c == '\n' {
                    saw_newline = true;
                }
                i += 1;
            }
            _ if is_ident_char(c) => {
                let start = i;
                while i < n && is_ident_char(src[i]) {
                    i += 1;
                }
                let word: String = src[start..i].iter().collect();
                // var 链模式内不匹配新声明（初始化器内文本按普通 token 消费）。
                let chain_base = var_phase.as_ref().map(|(b, _)| *b);
                if let Some(base) = chain_base {
                    if depth == base {
                        if matches!(var_phase, Some((_, VarPhase::Name))) {
                            // R201 门控：var 导出仅 strict（非 strict 间接 eval 的 var
                            // 本泄漏全局数据属性，后缀冗余且形态风险见 R201 注记）。
                            if is_strict {
                                push_export(&mut out, &mut seen, &word, true);
                            }
                            skip_ws_comments(&src, &mut i);
                            // `=`（无 or 有初始化都进入 End——初始化器内标识符按
                            // 表达式 token 消费，链仅在深度 0 的 `,`/`;`/EOF 推进或
                            // 终结；在此 ASI 终结会吃掉 `var a=globalThis,b=1`
                            // 的后续声明符——bing #6 链实证）。
                            // D2（缺陷首轮）：消费 `=` 后 prev_sig 须记 `=`——正则
                            // 字面量可出现于表达式位置（`var re = /a,b/`），记 'x'
                            // 会判除法、正则内容当代码、内部逗号触发伪声明符。
                            if i < n && src[i] == '=' {
                                i += 1;
                                var_phase = Some((base, VarPhase::End));
                                prev_sig = '=';
                                last_ident.clear();
                                continue;
                            }
                            var_phase = Some((base, VarPhase::End));
                            prev_sig = 'x';
                            last_ident.clear();
                            continue;
                        }
                        // End 阶段 ASI：换行后遇**声明关键字**（不可能延续表达式）→
                        // var 语句终结，当前关键字落入下方新语句匹配（`var a = 1
                        // \nasync function f(){}`——WPT 无分号形态）；其他标识符按
                        // 表达式 token 消费不终结链。
                        if saw_newline
                            && matches!(word.as_str(), "var" | "function" | "async" | "class" | "const" | "let")
                        {
                            var_phase = None; // 贯通至下方语句匹配（不 continue）
                        } else {
                            prev_sig = 'x';
                            last_ident = word;
                            saw_newline = false;
                            continue;
                        }
                    } else {
                        prev_sig = 'x';
                        last_ident.clear();
                        continue;
                    }
                }
                let at_stmt_start = depth == 0
                    && (matches!(prev_sig, '\0' | ';' | '}' | ')')
                        || (saw_newline && matches!(prev_sig, 'x' | ')' | ']' | '}')));
                saw_newline = false;
                if at_stmt_start {
                    match word.as_str() {
                        "var" => {
                            var_phase = Some((depth, VarPhase::Name));
                            prev_sig = 'x';
                            last_ident.clear();
                            continue;
                        }
                        "function" | "async" => {
                            // `function NAME(` / `function* NAME(` / `async function NAME(`
                            let mut j = i;
                            if word == "async" {
                                skip_ws_comments(&src, &mut j);
                                let s = j;
                                while j < n && is_ident_char(src[j]) {
                                    j += 1;
                                }
                                if src[s..j].iter().collect::<String>() != "function" {
                                    prev_sig = 'x';
                                    last_ident = word;
                                    continue;
                                }
                            }
                            skip_ws_comments(&src, &mut j);
                            if j < n && src[j] == '*' {
                                j += 1;
                                skip_ws_comments(&src, &mut j);
                            }
                            let s = j;
                            while j < n && is_ident_char(src[j]) {
                                j += 1;
                            }
                            let name: String = src[s..j].iter().collect();
                            skip_ws_comments(&src, &mut j);
                            if !name.is_empty() && j < n && src[j] == '(' {
                                push_export(&mut out, &mut seen, &name, false);
                            }
                            prev_sig = 'x';
                            last_ident.clear();
                            continue;
                        }
                        "class" => {
                            let mut j = i;
                            skip_ws_comments(&src, &mut j);
                            let s = j;
                            while j < n && is_ident_char(src[j]) {
                                j += 1;
                            }
                            let name: String = src[s..j].iter().collect();
                            skip_ws_comments(&src, &mut j);
                            let valid = (j < n && src[j] == '{')
                                || src[j..].iter().take(7).collect::<String>().starts_with("extends");
                            if !name.is_empty() && valid {
                                push_export(&mut out, &mut seen, &name, false);
                            }
                            prev_sig = 'x';
                            last_ident.clear();
                            continue;
                        }
                        "const" | "let" => {
                            let mut j = i;
                            skip_ws_comments(&src, &mut j);
                            let s = j;
                            while j < n && is_ident_char(src[j]) {
                                j += 1;
                            }
                            let name: String = src[s..j].iter().collect();
                            // 声明符后（允许空白）须 `=` 或行尾（伪匹配 `letName` 排除）。
                            let mut k = j;
                            skip_ws_comments(&src, &mut k);
                            if !name.is_empty() && (k >= n || src[k] == '=') {
                                push_export(&mut out, &mut seen, &name, word == "let");
                            }
                            prev_sig = 'x';
                            last_ident.clear();
                            continue;
                        }
                        _ => {}
                    }
                }
                prev_sig = 'x';
                last_ident = word;
            }
            _ => {
                // `,` / `;` / 运算符等：var 链深度 0 边界处理，其余记有效字符
                if let Some((base, phase)) = var_phase.as_mut()
                    && depth == *base
                    && (c == ',' || c == ';')
                {
                    if c == ',' {
                        *phase = VarPhase::Name;
                    } else {
                        var_phase = None; // `;` 消费，链终结
                    }
                    i += 1;
                    prev_sig = c;
                    last_ident.clear();
                    continue;
                }
                prev_sig = c;
                last_ident.clear();
                saw_newline = false;
                i += 1;
            }
        }
    }
    out.join("")
}

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
pub fn script_run_classic_page(code: &str, script_index: usize, source_url: Option<&str>) -> String {
    let code_literal = format!("'{}'", escape_js_string(code));
    // R147（js-dom M4）：顶层函数声明的**全局发布**。间接 eval `(0,eval)` 中源内
    // 'use strict'/"use strict" 指令使 eval 建独立变量环境——顶层 `function` 声明
    // **不落 globalThis**（真实浏览器 classic 脚本即便 strict 也创建全局绑定，spec
    // Script Is A Global Code；WPT 外链测试库如 prefixed-animation-event-tests.js
    // 的 `function runAnimationEventTests` 跨 `<script>` 不可见 → "is not defined"）。
    // 修复：扫描行首 `function NAME(` 形态（启发式——行首锚定，字符串/注释内换行后
    // 的伪匹配最坏多发布一个无害 globalThis 赋值），eval 源后拼接
    // `;globalThis.NAME=NAME;`（strict 局部声明经此导出；non-strict 本已全局，恒等）。
    //
    // R198（js-dom M4）：顶层 `const NAME =` / `let NAME =` 同款全局发布。strict eval
    // 的块级声明同样困在独立变量环境（WPT dom/nodes/support/NodeList-static-length-
    // tampered.js 顶层 `const indexOfNodeList = new Function(...)` 跨 `<script>` 不可见
    // → 后续脚本报 "indexOfNodeList is not defined"）。const/let 只读不重赋——
    // `globalThis.NAME=NAME` 读取局部绑定写入全局属性，与 function 导出对称。
    //
    // **每名独立 try 包裹（lit bundle 回归教训）**：minified bundle 的 IIFE 内部代码
    // 也是零缩进行首（`var ns = (function() {` 换行后 `const t=globalThis,...`）——
    // 行首锚定无法区分真顶层与 IIFE 作用域。后缀在 eval 顶层执行时，IIFE 作用域的
    // 名字已消亡 → 裸 `globalThis.t=t` 抛 ReferenceError **中止整个 eval**（lit e2e
    // 六测试 NO-REPORT/EXEC-ERR 实证）。每名 `try{...}catch(_){}` 包裹：作用域外的
    // 名字静默跳过，顶层名正常导出。R147 的 function 导出同样补包裹（lit bundle 无
    // 行首 function 故未暴露，防御同款形态）。
    // R201：**strict 判定**——var accessor 导出仅在 strict eval 有意义。非 strict 的
    //间接 eval 里 var 本就泄漏到全局（globalThis.NAME 数据属性）——accessor 重定义
    //会把数据属性换成 getter，getter 内 `return NAME` 解析到全局属性 = accessor 自身
    //→ 无限递归（lit e2e template_content_fragment_view 回归实证：非 strict 页面脚本
    //`var log = []` 后 accessor 自递归 Maximum call stack）。
    // bing-t2（site-optimizer 2026-10-10）：判定按 Directive Prologue 规则实现
    //（[`script_directive_is_strict`]）——旧「首非空行恰为指令」漏掉前导注释形态
    //（bing 全站 `//<![CDATA[` 起头），V8 已进 strict eval 而导出不启动。
    let is_strict = script_directive_is_strict(code);
    // bing-t2（site-optimizer 2026-10-10）：顶层声明导出改**单遍状态机扫描**
    //（[`scan_top_level_decl_exports`]）——minified 脚本的全部顶层声明在单行中部
    //（bing #6 `};;var _w=window,_d=document,...` @col 26588），旧行首零缩进锚定全漏。
    // 导出语义（var 仅 strict accessor/const 快照/let accessor/function·class 恒值、
    // 每名 try 包裹、R147/R198/R201/M2-S2/R3254 各自的 WPT/lit 依据）见该函数注记。
    let exports = scan_top_level_decl_exports(code, is_strict);
    // R147：eval 源拼接形态 `(0,eval)('<源>'+';globalThis.x=x;')`——后缀是**带引号的
    // 字符串字面量**（与源同串相接），在 eval 的同一变量环境内执行（strict 局部声明
    // 可见），且不改 'use strict' 必须为源首语句的语义（拼接发生在两侧而非插入）。
    // t2-pb3 诊断可观测性：`source_url` 给定时在 eval 源**真末尾**（导出后缀之后）追加
    // `//# sourceURL=<url>`——V8 仅在源末行取 sourceURL 为脚本名，注释落在导出后缀
    // 之前会被 R201 accessor 后缀（strict 顶层 var）整行拼接污染甚至弃用（V8 实测回
    // `<anonymous>`；缺陷角色 N1 定向闭环）。直接执行路径（无 wrapper）由
    // page_scripts::append_source_url 覆盖。
    let mut tail = exports;
    if let Some(url) = source_url {
        if !tail.is_empty() {
            tail.push('\n');
        }
        tail.push_str("//# sourceURL=");
        tail.extend(url.chars().filter(|c| !c.is_control()));
    }
    let export_suffix = if tail.is_empty() {
        String::new()
    } else {
        // bing-t2（site-optimizer 2026-10-10）：后缀分隔用**换行**而非 `;`——源码以
        // 行注释结尾时（bing 全站 `//]]>`），`;` 后缀与注释同行被整体吞掉，导出后缀
        // 静默失效（离线 fixture 实证：后缀已生成但 defineProperty 从未执行）。
        // 换行不改指令序言（directive 仍为源首语句）与 eval 同变量环境语义。
        format!("+'{}'", escape_js_string(&format!("\n{tail}")))
    };
    format!(
        // security-hardening M2-s6：`(0,globalThis.__zwRealEval||eval)`——eval 门禁
        // per-script 包装（webview 侧）会把 globalThis.eval 换成检查包装，本机制自身的
        // 页面脚本执行必须走**原生 eval**（包装安装片段先捕获 __zwRealEval；无门禁时
        // 回落 eval，语义同为间接 eval 全局作用域）。
        "globalThis.__zw_set_current_script&&globalThis.__zw_set_current_script({idx});\nglobalThis.{g}=undefined;\ntry{{(0,globalThis.__zwRealEval||eval)({code_literal}{export_suffix});}}catch(__zw_e){{globalThis.{g}=(__zw_e&&__zw_e.message)?String(__zw_e.message):String(__zw_e);}}\nfinally{{globalThis.__zw_clear_current_script&&globalThis.__zw_clear_current_script();}}",
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

/// 构造「contenteditable 宿主键入」的宿主脚本（R3254-M2 切片 2，editing goal）。
///
/// 宿主在 keydown 可打印字符且焦点元素为 contenteditable 宿主时执行：shim
/// `__zw_ce_insert(sel, text)` 按 Selection caret 做 deleteContents+insertNode DOM
/// 变更（经 mutation-emitting range 面回传宿主 rerender）并派发 beforeinput/input。
/// 非 contenteditable 宿主 → shim 侧 no-op。
pub fn script_contenteditable_insert(selector: &str, text: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_text = escape_js_string(text);
    format!("__zw_ce_insert('{esc_sel}', '{esc_text}')")
}

/// 构造「contenteditable 宿主 Backspace」的宿主脚本（R3254-M2 切片 2）。
///
/// shim `__zw_ce_delete(sel)`：选区非空删选区；collapsed 在文本节点内回退一个
/// UTF-16 单元（代理对安全）删除并派发 beforeinput/input。
pub fn script_contenteditable_delete(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_ce_delete('{esc_sel}')")
}

/// 构造「contenteditable 宿主 ForwardDelete」的宿主脚本（uievents-compat 尾簇 32）。
///
/// shim `__zw_ce_forward_delete(sel)`：选区非空删选区；collapsed 删 caret 后一个
/// UTF-16 单元（deleteContentForward 语义）并派发 beforeinput/input。
/// 构造「wheel 源 scroll」的宿主脚本（uievents-compat 尾簇 35）。
///
/// shim `__zw_wheel_scroll(sel, px, py, dx, dy)`：WheelEvent 派发（delta 透传；
/// target = sel 命中元素）。
pub fn script_wheel_scroll(selector: &str, px: f32, py: f32, dx: f32, dy: f32) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_wheel_scroll('{esc_sel}',{px},{py},{dx},{dy});")
}

/// 构造「contenteditable 宿主 ForwardDelete」的宿主脚本（uievents-compat 尾簇 32）。
///
/// shim `__zw_ce_forward_delete(sel)`：选区非空删选区；collapsed 删 caret 后一个
/// UTF-16 单元（deleteContentForward 语义）并派发 beforeinput/input。
pub fn script_contenteditable_forward_delete(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_ce_forward_delete('{esc_sel}')")
}

/// 构造「探测元素是否 contenteditable 宿主」的宿主脚本（R3254-M2 切片 2）。
///
/// 返 '1'（是宿主——自身或祖先 contenteditable=true）或 ''（否）。InsertText/
/// DeleteBackward 动作解析时先探测：text control 走既有 TextActionState 管线，
/// contenteditable 宿主走 `script_contenteditable_insert/delete`。
pub fn script_contenteditable_probe(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
return (e && typeof __zw_is_ce_host === 'function' && __zw_is_ce_host(e)) ? '1' : '';}})()"
    )
}

/// 构造「探测元素是否有 enclosing form」的宿主脚本（uievents-compat 尾簇 11）。
/// 返 '1'（target 在 form 内）或 ''（formless——Enter on formless buttonish 走激活
/// 点击而非隐式提交 noop）。
pub fn script_enclosing_form_probe(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
return (e && e.closest && e.closest('form')) ? '1' : '';}})()"
    )
}

/// 构造「探测元素是否 button 类控件」的宿主脚本（R3254-K4 切片 2，keyboard
/// default-actions goal）。
///
/// 返 '1'（BUTTON 或 input type=button|submit|reset——空格激活目标）或 ''（否）。
/// runner send_keys 空格时序：button-ish 目标在 **keyup** 触发激活（UI Events/Chromium
/// 语义——Enter 在 keydown、Space 在 keyup），keydown 只派事件不激活。
pub fn script_buttonish_probe(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
if (!e) return '';\
var t = String(e.tagName).toUpperCase();\
if (t === 'BUTTON') return '1';\
if (t === 'INPUT' && ['button','submit','reset'].indexOf(String(e.getAttribute('type') || '').toLowerCase()) >= 0) return '1';\
return '';}})()"
    )
}

/// 构造「表单控件 disabled 状态探针」脚本（R3254-K3 切片 B，keyboard default-actions
/// goal——隐式提交的 default button disabled 判定）。命中元素反射 `disabled` IDL 属性
///（INPUT/BUTTON/SELECT/TEXTAREA 等表单控件；经 shim proxy 的 IDL 反射，含 live 状态）→
/// '1'/''。元素不存在 → ''（调用方按 enabled 处理——submit 路径已先解析 form/button）。
pub fn script_control_disabled_probe(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!(
        "(function(){{var e=document.querySelector('{esc_sel}');\
return (e && e.disabled) ? '1' : '';}})()"
    )
}

/// 构造「select type-ahead 键入跳转」的宿主脚本（R3254-K5 切片 3，keyboard
/// default-actions goal M3）。
///
/// shim `__zw_select_type_ahead(sel, char)`：焦点 SELECT（closed——headless 无展开态）
/// 上的可打印字符默认动作。多字符缓冲（500ms idle 清空——Chromium type-ahead 窗近似）：
/// 缓冲累计后取首个 text 以缓冲为前缀（大小写不敏感）的 enabled option 选中；value/
/// selectedIndex 变化派 input（bubbles）+ change（bubbles）事件——JS 可观察验收面。
/// 目标非 SELECT 或无可跳 option → 返 ''（runner 落回既有 InsertText 路径）。
pub fn script_select_type_ahead(selector: &str, character: char) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_char = escape_js_string(&character.to_string());
    format!("__zw_select_type_ahead && __zw_select_type_ahead('{esc_sel}', '{esc_char}')")
}

/// 构造「contenteditable 宿主 Enter 换行」的宿主脚本（R3254-M2 切片 3，editing goal）。
///
/// shim `__zw_ce_enter(sel)`：caret 处插 `<br>`（insertLineBreak 语义——insertParagraph
/// 块级拆分 defer 记录），经 innerHTML setter → SetInnerHtml mutation 流转宿主，
/// 派发 beforeinput/input 事件序。仅宿主直子文本节点内 caret 应用。
pub fn script_contenteditable_enter(selector: &str) -> String {
    let esc_sel = escape_js_string(selector);
    format!("__zw_ce_enter('{esc_sel}')")
}

/// 构造「Esc 默认动作——dialog cancel/close」的宿主脚本（R3254-K3，keyboard
/// default-actions goal M2 切片 2）。
///
/// shim `__zw_esc_dialog_cancel()`：模态 dialog 优先派 cancelable 'cancel'，未被
/// preventDefault → close（reason=cancel）+ 'close' 事件。无 open dialog → no-op。
pub fn script_esc_dialog_cancel() -> String {
    "__zw_esc_dialog_cancel && __zw_esc_dialog_cancel()".to_string()
}

/// 构造「select 键盘导航」的宿主脚本（R3254-K5，keyboard-default-actions goal M3）。
///
/// shim `__zw_select_key_action(sel, key)`：焦点在 SELECT 上时 ArrowDown/ArrowUp/
/// Home/End 移动选中项（跳过 disabled、不回绕——Chromium closed select 语义近似），
/// 选中变化派 input（bubbles、不可取消）+ change（bubbles）事件——headless 无真
/// 下拉 UI，selectedIndex/value/事件序为 JS 可观察验收面。
pub fn script_select_key_action(selector: &str, key: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_key = escape_js_string(key);
    format!("__zw_select_key_action('{esc_sel}', '{esc_key}')")
}

/// 构造「滚动键默认动作」的宿主脚本（R3254-KP5，keyboard-page-scrolling goal M2 切片 2）。
///
/// shim `__zw_scroll_key_default(sel, key)`：keydown 滚动键未被取消时的滚动默认动作——
/// 幅度映射与 browser `app_input.scroll_delta_for_key`（R3254-M9）同源：Space/PageDown
/// = +0.85×视口高、PageUp = −0.85×视口高、ArrowDown/Up = ±40、Home/End = 顶/底。
/// 目标元素经 R3047 `scrollTop`/`scrollLeft` setter 落 `_scrollOffsets` 并同步派 'scroll'
/// 事件（headless 无真滚动管线，JS 可观察面 = scrollTop 值 + scroll 事件）。目标非
/// Element（null/undefined）时回落 window.scrollTo。
pub fn script_scroll_key_default(selector: &str, key: &str) -> String {
    let esc_sel = escape_js_string(selector);
    let esc_key = escape_js_string(key);
    format!("__zw_scroll_key_default && __zw_scroll_key_default('{esc_sel}', '{esc_key}')")
}
