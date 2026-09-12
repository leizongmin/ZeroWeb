//! Live renderer automation operations shared by WebDriver and WPT testdriver.

use zero_protocol::message::{
    AutomationElementRef, AutomationError, AutomationErrorCode, AutomationKey, AutomationOperation, AutomationRequest,
    AutomationResponse, AutomationResult, AutomationStateQuery, AutomationValue, IpcMessageKind, KeyboardEventParams,
    KeyboardEventType,
};

use super::{PageScriptContext, RendererRuntime};

impl RendererRuntime {
    pub(super) fn handle_automation_request(
        &mut self,
        request_id: u64,
        request: AutomationRequest,
    ) -> Result<(), String> {
        let result = self.execute_automation_request(request);
        self.send_regular_with_id(
            request_id,
            IpcMessageKind::AutomationResponse(AutomationResponse {
                navigation_epoch: self.navigation_epoch,
                document_generation: self.document_generation,
                result,
            }),
        )
    }

    fn execute_automation_request(&mut self, request: AutomationRequest) -> Result<AutomationResult, AutomationError> {
        let font_loader = self.font_loader.duplicate();
        let font_id = self.font_id;
        super::text_metrics::with_measure_ctx_opt(&font_loader, font_id, || {
            self.execute_automation_operation(request.operation)
        })
    }

    fn execute_automation_operation(
        &mut self,
        operation: AutomationOperation,
    ) -> Result<AutomationResult, AutomationError> {
        match operation {
            AutomationOperation::FindElement { using: _, value } => {
                if value.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "element locator must not be empty",
                    ));
                }
                let handle = self
                    .webview
                    .as_ref()
                    .and_then(|webview| webview.page_node_handle_for_selector(&value))
                    .ok_or_else(|| automation_error(AutomationErrorCode::NoSuchElement, "element not found"))?;
                Ok(AutomationResult::Element(Some(self.automation_element_ref(handle))))
            }
            AutomationOperation::FindElements { using: _, value } => {
                if value.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "element locator must not be empty",
                    ));
                }
                // https://w3c.github.io/webdriver/#find-elements — 无匹配返回空列表而非错误。
                let handles = self
                    .webview
                    .as_ref()
                    .map(|webview| webview.page_node_handles_for_selector(&value))
                    .unwrap_or_default();
                let references = handles
                    .into_iter()
                    .map(|handle| self.automation_element_ref(handle))
                    .collect();
                Ok(AutomationResult::Elements(references))
            }
            AutomationOperation::ElementClick { element } => {
                let selector = self.selector_for_automation_element(element)?;
                self.automation_click(&selector).map_err(internal_error)?;
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::SendKeys { element, keys } => {
                let selector = self.selector_for_automation_element(element)?;
                if self.interaction.focus_owner() != Some(selector.as_str()) {
                    self.blur_focused().map_err(internal_error)?;
                    self.focus_target(&selector).map_err(internal_error)?;
                }
                for key in keys {
                    self.automation_send_key(key).map_err(internal_error)?;
                }
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::ElementState { element, query } => {
                // https://w3c.github.io/webdriver/#element-state — 状态读经唯一选择器在
                // 页面脚本上下文求值，与渲染管线同一 live document。
                let selector = self.selector_for_automation_element(element)?;
                let script = element_state_script(&selector, &query);
                self.execute_state_script(script)
            }
            AutomationOperation::ElementClear { element } => {
                // https://w3c.github.io/webdriver/#element-clear — 可编辑元素置空 value，
                // 可勾选元素保持勾选语义不变（仅清文本类）。
                let selector = self.selector_for_automation_element(element)?;
                if self.interaction.focus_owner() != Some(selector.as_str()) {
                    self.blur_focused().map_err(internal_error)?;
                    self.focus_target(&selector).map_err(internal_error)?;
                }
                let script = format!(
                    "(function(){{var el=document.querySelector({selector:?});\
                     if(!el)return JSON.stringify({{ok:false}});\
                     var tag=el.tagName.toLowerCase();\
                     if(tag==='textarea'||(tag==='input'&&el.type!=='checkbox'&&el.type!=='radio'&&el.type!=='file')){{\
                     el.value='';el.dispatchEvent(new Event('input',{{bubbles:true}}));}}\
                     return JSON.stringify({{ok:true}});}})()"
                );
                let value = self.run_page_context_script(&script)?;
                let parsed = automation_value_from_script(&value);
                let AutomationValue::Object(entries) = parsed else {
                    return Err(internal_error("unexpected clear result".into()));
                };
                let ok = entries
                    .iter()
                    .any(|(k, v)| k == "ok" && *v == AutomationValue::Bool(true));
                if !ok {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "element is not clearable",
                    ));
                }
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::GetActiveElement => {
                let element = self
                    .interaction
                    .focus_owner()
                    .and_then(|selector| {
                        self.webview
                            .as_ref()
                            .and_then(|webview| webview.page_node_handle_for_selector(selector))
                    })
                    .map(|handle| self.automation_element_ref(handle));
                Ok(AutomationResult::Element(element))
            }
            AutomationOperation::ExecuteScript { script, arguments } => {
                if script.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "script must not be empty",
                    ));
                }
                let arguments =
                    serde_json::Value::Array(arguments.iter().map(automation_value_to_json).collect::<Vec<_>>());
                let source = format!(
                    "(function(){{var __zw_value=(function(){{{script}\n}}).apply(null,{arguments});\
                     return JSON.stringify({{defined:typeof __zw_value!=='undefined',value:__zw_value}});}})()"
                );
                let value = self.run_page_context_script(&source)?;
                Ok(AutomationResult::Value(automation_value_from_script(&value)))
            }
            AutomationOperation::EvaluateRetaining {
                script,
                group,
                return_by_value,
            } => {
                if script.is_empty() {
                    return Err(automation_error(
                        AutomationErrorCode::InvalidArgument,
                        "script must not be empty",
                    ));
                }
                let group = group.unwrap_or_else(|| DEFAULT_OBJECT_GROUP.to_string());
                // 对象结果保留进注册表（句柄随文档换代经 JS context 重建自然失效）。
                let source = evaluate_retaining_script(&script, &group, return_by_value);
                match self.run_handle_operation(&source)? {
                    HandleOutcome::Pending => Err(internal_error("evaluate cannot be awaited".into())),
                    HandleOutcome::Result(result) => Ok(result),
                }
            }
            AutomationOperation::CallFunctionOnHandle {
                handle,
                function_declaration,
                arguments,
                return_by_value,
                await_promise,
                group,
            } => {
                let group = group.unwrap_or_else(|| DEFAULT_OBJECT_GROUP.to_string());
                let arguments = serde_json::Value::Array(arguments.iter().map(automation_value_to_json).collect());
                let source = call_function_on_handle_script(
                    handle,
                    &function_declaration,
                    &arguments.to_string(),
                    return_by_value,
                    await_promise,
                    &group,
                );
                match self.run_handle_operation(&source)? {
                    HandleOutcome::Pending => self.await_pending_operation(&group, return_by_value),
                    HandleOutcome::Result(result) => Ok(result),
                }
            }
            AutomationOperation::ReleaseHandle { handle } => {
                self.run_handle_operation(&release_handle_script(handle))?;
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::ReleaseObjectGroup { group } => {
                self.run_handle_operation(&release_object_group_script(&group))?;
                Ok(AutomationResult::Empty)
            }
            AutomationOperation::Unsupported { name } => Err(automation_error(
                AutomationErrorCode::UnsupportedOperation,
                format!("unsupported automation operation: {name}"),
            )),
        }
    }

    fn automation_element_ref(&self, node_handle: u64) -> AutomationElementRef {
        AutomationElementRef {
            navigation_epoch: self.navigation_epoch,
            document_generation: self.document_generation,
            node_handle,
        }
    }

    /// 在页面脚本上下文执行 `source`，返回脚本 stdout（JSON 包络字符串）并同步 DOM 变更。
    fn run_page_context_script(&mut self, source: &str) -> Result<String, AutomationError> {
        let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
        let (value, changed) = {
            let mut context = PageScriptContext {
                html: &mut self.cached_html,
                url: &current_url,
                js_worker: &self.js_worker,
                webview: self.webview.as_mut(),
            };
            super::page_scripts::execute_automation_script(&mut context, source)
                .map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?
        };
        self.sync_focus_from_js();
        self.sync_cached_html_from_webview();
        // S11：脚本执行产生的 console 输出先于 AutomationResponse 转发（headless 在
        // 自动化往返中消费并入同一命令的事件排空——晚了要等下一条命令才可见）。
        self.tick_console_log_drain();
        if changed {
            self.publish_webview(None, true).map_err(internal_error)?;
        }
        Ok(value)
    }

    /// 句柄操作的公共尾：执行生成脚本、解包络（含 pending/句柄/错误信号）。
    fn run_handle_operation(&mut self, source: &str) -> Result<HandleOutcome, AutomationError> {
        let value = self.run_page_context_script(source)?;
        parse_handle_operation_envelope(&value)
    }

    /// `awaitPromise` 落定循环：`__zwAutomationAwait` 由首个 execute 投递，此后每轮
    /// 泵宿主 timer 回调 + 读 done 哨兵（execute 边界 drain microtask，见
    /// script-sandbox `perform_microtask_checkpoint` / QuickJS job queue drain），
    /// 直到落定或有界超时。落定值留在注册表脚本侧读取（保对象本体，不走 JSON 往返）。
    fn await_pending_operation(
        &mut self,
        group: &str,
        return_by_value: bool,
    ) -> Result<AutomationResult, AutomationError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(AWAIT_PROMISE_TIMEOUT_SECS);
        loop {
            {
                let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
                let mut context = PageScriptContext {
                    html: &mut self.cached_html,
                    url: &current_url,
                    js_worker: &self.js_worker,
                    webview: self.webview.as_mut(),
                };
                // await 期间 timer 回调的 DOM 变更照常提交到活 DOM。
                super::page_scripts::drain_pending_dom_mutations(&mut context);
            }
            let state = self
                .js_worker
                .execute_script_direct(AWAIT_POLL_SCRIPT)
                .map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?;
            let settled = serde_json::from_str::<serde_json::Value>(&state)
                .ok()
                .and_then(|value| value.get("done").and_then(serde_json::Value::as_bool));
            if settled == Some(true) {
                break;
            }
            if std::time::Instant::now() >= deadline {
                return Err(automation_error(AutomationErrorCode::Timeout, "awaitPromise timed out"));
            }
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        let source = pending_settled_script(group, return_by_value);
        match self.run_handle_operation(&source)? {
            HandleOutcome::Pending => Err(internal_error(
                "await sentinel reported done but tail read pending".into(),
            )),
            HandleOutcome::Result(result) => Ok(result),
        }
    }

    /// 元素状态查询的公共尾：执行生成的脚本并解 JSON 包络。
    fn execute_state_script(&mut self, script: String) -> Result<AutomationResult, AutomationError> {
        let value = self.run_page_context_script(&script)?;
        Ok(AutomationResult::Value(automation_value_from_script(&value)))
    }

    fn selector_for_automation_element(&self, element: AutomationElementRef) -> Result<String, AutomationError> {
        if element.navigation_epoch != self.navigation_epoch || element.document_generation != self.document_generation
        {
            return Err(automation_error(
                AutomationErrorCode::StaleElementReference,
                "element belongs to an old document",
            ));
        }
        self.webview
            .as_ref()
            .and_then(|webview| webview.selector_for_page_node_handle(element.node_handle))
            .ok_or_else(|| {
                automation_error(
                    AutomationErrorCode::StaleElementReference,
                    "element no longer exists in the live document",
                )
            })
    }

    fn automation_click(&mut self, selector: &str) -> Result<(), String> {
        // https://w3c.github.io/webdriver/#element-click
        let (click, checked_handled) = self.dispatch_checked_click(selector.to_string())?;
        if self.interaction.focus_owner() != Some(selector) {
            self.blur_focused()?;
            self.focus_target(selector)?;
        }
        if click.default_allowed && !checked_handled && !self.activate_form_control_at(selector)? {
            self.activate_label_at(selector)?;
        }
        Ok(())
    }

    fn automation_send_key(&mut self, key: AutomationKey) -> Result<(), String> {
        // https://w3c.github.io/webdriver/#element-send-keys
        match key {
            AutomationKey::Text(text) => {
                for character in text.chars() {
                    let value = character.to_string();
                    self.automation_key_event(&value, "Unidentified", false, KeyboardEventType::Down)?;
                    self.automation_key_event(&value, "Unidentified", false, KeyboardEventType::Up)?;
                }
            }
            AutomationKey::Tab => {
                self.automation_key_event("Tab", "Tab", false, KeyboardEventType::Down)?;
                self.automation_key_event("Tab", "Tab", false, KeyboardEventType::Up)?;
            }
            AutomationKey::ShiftTab => {
                self.automation_key_event("Tab", "Tab", true, KeyboardEventType::Down)?;
                self.automation_key_event("Tab", "Tab", true, KeyboardEventType::Up)?;
            }
            AutomationKey::Backspace => {
                self.automation_key_event("Backspace", "Backspace", false, KeyboardEventType::Down)?;
                self.automation_key_event("Backspace", "Backspace", false, KeyboardEventType::Up)?;
            }
            AutomationKey::Enter => {
                self.automation_key_event("Enter", "Enter", false, KeyboardEventType::Down)?;
                self.automation_key_event("Enter", "Enter", false, KeyboardEventType::Up)?;
            }
        }
        Ok(())
    }

    fn automation_key_event(
        &mut self,
        key: &str,
        code: &str,
        shift: bool,
        event_type: KeyboardEventType,
    ) -> Result<(), String> {
        self.handle_keyboard_event(KeyboardEventParams {
            key: key.to_string(),
            code: code.to_string(),
            ctrl: false,
            shift,
            alt: false,
            meta: false,
            event_type,
        })
    }
}

/// 生成元素状态查询脚本：按唯一选择器定位元素，按 query 项读取状态，返 JSON 包络。
///
/// 选择器以 JS 字符串字面量（`{:?}`）内插，脚本环境与渲染管线同一 live document。
/// 找不到元素 → `{"ok":false}`（上游层把引用过期/缺失分开报）。
fn element_state_script(selector: &str, query: &AutomationStateQuery) -> String {
    let find = format!("var el=document.querySelector({selector:?});if(!el)return JSON.stringify({{ok:false}});");
    let body = match query {
        // W3C Get Element Text：渲染文本近似 = textContent（可见性过滤未实现，FIXME）。
        AutomationStateQuery::Text => {
            format!("{find}return JSON.stringify({{ok:true,value:el.textContent==null?null:String(el.textContent)}});")
        }
        // W3C Get Element Rect：shim `getBoundingClientRect` 真值（RectBridge 注册时）。
        AutomationStateQuery::Rect => format!(
            "{find}var r=el.getBoundingClientRect();\
             return JSON.stringify({{ok:true,value:{{x:r.x,y:r.y,width:r.width,height:r.height}}}});"
        ),
        // W3C Is Element Enabled：非表单元素恒 true（无 disabled 语义）。
        AutomationStateQuery::Enabled => format!(
            "{find}var disabled=false;\
             if(el.disabled===true)disabled=true;\
             else if(typeof el.hasAttribute==='function'&&el.hasAttribute('disabled'))disabled=true;\
             return JSON.stringify({{ok:true,value:!disabled}});"
        ),
        // W3C Is Element Selected：option 的 selected / checkbox·radio 的 checkedness。
        AutomationStateQuery::Selected => format!(
            "{find}var selected=false;\
             if(el.tagName.toLowerCase()==='option')selected=!!el.selected;\
             else if(el.type==='checkbox'||el.type==='radio')selected=!!el.checked;\
             return JSON.stringify({{ok:true,value:selected}});"
        ),
        // W3C Get Element Attribute：内容属性值（不存在 → null）。
        AutomationStateQuery::Attribute(name) => format!(
            "{find}var v=el.getAttribute({name:?});\
             return JSON.stringify({{ok:true,value:v===null?null:String(v)}});"
        ),
        // W3C Get Element Property：DOM 属性直读（undefined → null）。
        AutomationStateQuery::Property(name) => format!(
            "{find}var v=el[{name:?}];\
             return JSON.stringify({{ok:true,value:v===undefined?null:v}});"
        ),
        // W3C Get Element CSS Value：计算样式（shim getComputedStyle → host 真值）。
        AutomationStateQuery::CssValue(name) => format!(
            "{find}var cs=getComputedStyle(el);var v=cs.getPropertyValue({name:?});\
             return JSON.stringify({{ok:true,value:String(v)}});"
        ),
    };
    format!("(function(){{{body}}})()")
}

fn automation_error(code: AutomationErrorCode, message: impl Into<String>) -> AutomationError {
    AutomationError {
        code,
        message: message.into(),
    }
}

fn internal_error(message: String) -> AutomationError {
    automation_error(AutomationErrorCode::Internal, message)
}

fn automation_value_to_json(value: &AutomationValue) -> serde_json::Value {
    match value {
        AutomationValue::Null => serde_json::Value::Null,
        AutomationValue::Bool(value) => serde_json::Value::Bool(*value),
        AutomationValue::Number(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        AutomationValue::String(value) => serde_json::Value::String(value.clone()),
        AutomationValue::Array(values) => {
            serde_json::Value::Array(values.iter().map(automation_value_to_json).collect())
        }
        AutomationValue::Object(entries) => serde_json::Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.clone(), automation_value_to_json(value)))
                .collect(),
        ),
        // 句柄参数以标记对象下传，页面脚本在实参列表顶层还原为保留对象
        //（CDP `Runtime.callFunctionOn` 的 `arguments[].objectId` 只出现在实参顶层）。
        AutomationValue::Handle(handle) => serde_json::json!({ "__zwHandleRef": handle.id }),
    }
}

/// 未显式指定 `objectGroup` 时句柄落入的默认组。
const DEFAULT_OBJECT_GROUP: &str = "zw-automation";
/// `awaitPromise` 落定循环的有界超时（秒）——headless 侧自动化 IPC 超时为 10s，留余量。
const AWAIT_PROMISE_TIMEOUT_SECS: u64 = 8;

/// 落定哨兵：只读 done 标志（落定值本体留在 `__zwAutomationAwait`，由尾部脚本按
/// returnByValue 语义取用）。
const AWAIT_POLL_SCRIPT: &str =
    "JSON.stringify(globalThis.__zwAutomationAwait?{done:globalThis.__zwAutomationAwait.done}:{done:null})";

/// 句柄注册表 bootstrap 片段——每个操作脚本内联一份，幂等（首次执行时初始化）。
/// 注册表活在页面 JS context 全局对象上：导航换代 context 销毁重建
///（`reset_document_state` → `sandbox.reset_context`）→ 全部句柄自然失效，
/// 与 CDP「execution context destroyed」语义一致。
const HANDLE_REGISTRY_JS: &str = r#"
var __zw = globalThis.__zwAutomationHandles = globalThis.__zwAutomationHandles || (function () {
  var r = { next: 1, byId: Object.create(null), groups: Object.create(null) };
  r.retain = function (v, group) {
    if (r.next > 65536) throw new Error('automation handle registry full');
    var id = r.next;
    r.next = r.next + 1;
    r.byId[id] = v;
    (r.groups[group] = r.groups[group] || Object.create(null))[id] = true;
    return id;
  };
  r.tail = function (v, group, byValue) {
    var t = typeof v;
    if (byValue || v === null || v === undefined || t === 'boolean' || t === 'number' || t === 'string') {
      if (v === undefined) return JSON.stringify({ defined: false });
      return JSON.stringify({ defined: true, value: v });
    }
    // node 性随句柄上报：shim DOM 节点具 nodeType（CDP subtype:"node" 判定面）。
    var isNode = v && typeof v === 'object' && typeof v.nodeType === 'number' && v.nodeType >= 1 && v.nodeType <= 12;
    return JSON.stringify({ defined: true, handle: r.retain(v, group), node: !!isNode });
  };
  return r;
})();"#;

/// [`AutomationOperation::EvaluateRetaining`] 脚本：执行表达式并按
/// returnByValue 语义处理结果（false：对象 → 注册表句柄；true：对象按值深序列化）。
/// 表达式语义（Chromium `Runtime.evaluate` 单表达式形态）；末尾分号剥除——
/// PW 安装源为 IIFE 自调用（`...();`），直拼 `(...;)` 不编译。
fn evaluate_retaining_script(script: &str, group: &str, return_by_value: bool) -> String {
    let expression = script.trim_end().strip_suffix(';').unwrap_or_else(|| script.trim_end());
    let by_value = if return_by_value { "true" } else { "false" };
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}\nvar __zwV = ({expression});\nreturn __zw.tail(__zwV, {group:?}, {by_value});}})()"
    )
}

/// [`AutomationOperation::CallFunctionOnHandle`] 脚本：句柄对象为 `this` 调用函数，
/// 顶层句柄实参还原为保留对象；`awaitPromise` 时 thenable 结果转落定哨兵。
#[allow(clippy::too_many_arguments)]
fn call_function_on_handle_script(
    handle: u64,
    function_declaration: &str,
    arguments_json: &str,
    return_by_value: bool,
    await_promise: bool,
    group: &str,
) -> String {
    let await_open = if await_promise { "true" } else { "false" };
    let by_value = if return_by_value { "true" } else { "false" };
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
var __zwT = __zw.byId[{handle}];
if (!__zwT) return JSON.stringify({{defined:true,zwMiss:true}});
var __zwF = ({function_declaration});
var __zwA = {arguments_json};
for (var __zwI = 0; __zwI < __zwA.length; __zwI++) {{
  var __zwM = __zwA[__zwI];
  // 仅对象形态的标记（{{__zwHandleRef:n}}）还原为保留对象；裸 falsy 实参
  // （false/0/''）不得进入查找——`falsy && x` 会被误判为标记（实测根因）。
  if (__zwM && typeof __zwM === 'object' && __zwM.__zwHandleRef !== undefined) {{
    __zwA[__zwI] = __zw.byId[__zwM.__zwHandleRef];
    if (!__zwA[__zwI]) return JSON.stringify({{defined:true,zwMiss:true,dbgWanted:__zwM.__zwHandleRef}});
  }}
}}
var __zwV;
try {{ __zwV = __zwF.apply(__zwT, __zwA); }}
catch (__zwE) {{ return JSON.stringify({{defined:true,zwThrow:String(__zwE && __zwE.message || __zwE)}}); }}
if ({await_open} && __zwV && typeof __zwV.then === 'function') {{
  globalThis.__zwAutomationAwait = {{ done: false }};
  Promise.resolve(__zwV).then(
    function (v) {{ globalThis.__zwAutomationAwait = {{ done: true, ok: true, value: v }}; }},
    function (e) {{ globalThis.__zwAutomationAwait = {{ done: true, ok: false, error: String(e && e.message || e) }}; }}
  );
  return JSON.stringify({{ defined: true, pending: true }});
}}
return __zw.tail(__zwV, {group:?}, {by_value});}})()"
    )
}

/// `awaitPromise` 落定后的收尾脚本：从 `__zwAutomationAwait` 取落定值本体并套用
/// returnByValue 语义（保对象本体，不走 JSON 往返）。
fn pending_settled_script(group: &str, return_by_value: bool) -> String {
    let by_value = if return_by_value { "true" } else { "false" };
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
var __zwA = globalThis.__zwAutomationAwait;
globalThis.__zwAutomationAwait = undefined;
if (!__zwA || !__zwA.done) return JSON.stringify({{defined:true,pending:true}});
if (!__zwA.ok) return JSON.stringify({{defined:true,zwThrow:String(__zwA.error)}});
return __zw.tail(__zwA.value, {group:?}, {by_value});}})()"
    )
}

/// [`AutomationOperation::ReleaseHandle`] 脚本：从全部组与注册表移除句柄。
fn release_handle_script(handle: u64) -> String {
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
for (var __zwK in __zw.groups) delete __zw.groups[__zwK][{handle}];
delete __zw.byId[{handle}];
return JSON.stringify({{defined:true,value:true}});}})()"
    )
}

/// [`AutomationOperation::ReleaseObjectGroup`] 脚本：整组移除注册表句柄。
fn release_object_group_script(group: &str) -> String {
    format!(
        "(function(){{{HANDLE_REGISTRY_JS}
var __zwIds = __zw.groups[{group:?}];
if (__zwIds) for (var __zwId in __zwIds) delete __zw.byId[__zwId];
delete __zw.groups[{group:?}];
return JSON.stringify({{defined:true,value:true}});}})()"
    )
}

/// 句柄操作包络的解析结果：终值或 `awaitPromise` 落定哨兵。
enum HandleOutcome {
    Result(AutomationResult),
    Pending,
}

/// 解包络：`zwThrow`/`zwMiss` → 脚本错误；`handle` → 句柄引用；`pending` → 落定哨兵；
/// 其余走既有包络解析（`{defined,value}`）。
fn parse_handle_operation_envelope(value: &str) -> Result<HandleOutcome, AutomationError> {
    let envelope = match serde_json::from_str::<serde_json::Value>(value) {
        Ok(serde_json::Value::Object(envelope)) if envelope.contains_key("defined") => envelope,
        _ => {
            return Ok(HandleOutcome::Result(AutomationResult::Value(
                automation_value_from_script(value),
            )));
        }
    };
    if envelope.get("zwThrow").and_then(serde_json::Value::as_str).is_some() {
        let message = envelope
            .get("zwThrow")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        return Err(automation_error(
            AutomationErrorCode::JavascriptError,
            message.to_string(),
        ));
    }
    if envelope.get("zwMiss").and_then(serde_json::Value::as_bool) == Some(true) {
        return Err(automation_error(
            AutomationErrorCode::JavascriptError,
            "object handle is unknown or has been released".to_string(),
        ));
    }
    if envelope.get("pending").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(HandleOutcome::Pending);
    }
    if let Some(handle) = envelope.get("handle").and_then(serde_json::Value::as_u64) {
        let node = envelope
            .get("node")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        return Ok(HandleOutcome::Result(AutomationResult::Value(AutomationValue::Handle(
            zero_protocol::message::AutomationHandleRef { id: handle, node },
        ))));
    }
    Ok(HandleOutcome::Result(AutomationResult::Value(
        automation_value_from_script(value),
    )))
}

fn automation_value_from_script(value: &str) -> AutomationValue {
    if value == "undefined" || value == "null" {
        return AutomationValue::Null;
    }
    match serde_json::from_str::<serde_json::Value>(value) {
        Ok(serde_json::Value::Object(mut envelope)) if envelope.contains_key("defined") => {
            if envelope.get("defined").and_then(serde_json::Value::as_bool) != Some(true) {
                AutomationValue::Null
            } else {
                automation_value_from_json(envelope.remove("value").unwrap_or(serde_json::Value::Null))
            }
        }
        Ok(value) => automation_value_from_json(value),
        Err(_) => AutomationValue::String(value.to_string()),
    }
}

fn automation_value_from_json(value: serde_json::Value) -> AutomationValue {
    match value {
        serde_json::Value::Null => AutomationValue::Null,
        serde_json::Value::Bool(value) => AutomationValue::Bool(value),
        serde_json::Value::Number(value) => AutomationValue::Number(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => AutomationValue::String(value),
        serde_json::Value::Array(values) => {
            AutomationValue::Array(values.into_iter().map(automation_value_from_json).collect())
        }
        serde_json::Value::Object(entries) => AutomationValue::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, automation_value_from_json(value)))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use zero_protocol::message::FramePublishMode;

    use super::*;

    fn runtime() -> RendererRuntime {
        let html = "<html><body><input id=\"name\"><input id=\"check\" type=\"checkbox\"></body></html>";
        let url = "https://zero.test/automation";
        let (_tx, rx) = mpsc::channel();
        let mut runtime = RendererRuntime::with_io(301, FramePublishMode::Legacy, Box::new(std::io::sink()), rx);
        runtime.compositor_publish = None;
        runtime.stub_network = true;
        runtime.current_url = Some(url.into());
        runtime.cached_html = html.into();
        runtime.navigation_epoch = 9;
        runtime.document_generation = 1;
        runtime.webview.as_mut().unwrap().prepare_document_state(url);
        runtime.webview.as_mut().unwrap().load_html(html, None);
        {
            let mut context = PageScriptContext {
                html: &mut runtime.cached_html,
                url,
                js_worker: &runtime.js_worker,
                webview: runtime.webview.as_mut(),
            };
            super::super::page_scripts::run_page_scripts(&mut context, true, |_url| {
                Err::<String, String>("no fetch".into())
            });
        }
        runtime
    }

    fn find(runtime: &mut RendererRuntime, selector: &str) -> AutomationElementRef {
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::FindElement {
                    using: zero_protocol::message::AutomationLocatorStrategy::CssSelector,
                    value: selector.into(),
                },
            })
            .expect("find element");
        let AutomationResult::Element(Some(element)) = result else {
            panic!("expected element");
        };
        element
    }

    #[test]
    fn live_automation_updates_form_and_rejects_stale_reference() {
        let mut runtime = runtime();
        let name = find(&mut runtime, "#name");
        let check = find(&mut runtime, "#check");

        runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::SendKeys {
                    element: name,
                    keys: vec![AutomationKey::Text("Aé".into())],
                },
            })
            .expect("send unicode keys");
        assert_eq!(
            runtime
                .execute_automation_request(AutomationRequest {
                    operation: AutomationOperation::ExecuteScript {
                        script: "return document.getElementById('name').value;".into(),
                        arguments: Vec::new(),
                    },
                })
                .expect("read live input"),
            AutomationResult::Value(AutomationValue::String("Aé".into()))
        );

        runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::ElementClick { element: check },
            })
            .expect("click checkbox");
        assert_eq!(
            runtime
                .execute_automation_request(AutomationRequest {
                    operation: AutomationOperation::ExecuteScript {
                        script: "return document.getElementById('check').checked;".into(),
                        arguments: Vec::new(),
                    },
                })
                .expect("read live checkedness"),
            AutomationResult::Value(AutomationValue::Bool(true))
        );
        assert_eq!(
            runtime
                .execute_automation_request(AutomationRequest {
                    operation: AutomationOperation::GetActiveElement,
                })
                .expect("active element"),
            AutomationResult::Element(Some(check))
        );

        let stale = AutomationElementRef {
            document_generation: check.document_generation.saturating_sub(1),
            ..check
        };
        let error = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::ElementClick { element: stale },
            })
            .expect_err("stale click must fail");
        assert_eq!(error.code, AutomationErrorCode::StaleElementReference);
    }

    #[test]
    fn find_elements_returns_all_matches_in_document_order() {
        let mut runtime = runtime();
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::FindElements {
                    using: zero_protocol::message::AutomationLocatorStrategy::CssSelector,
                    value: "input".into(),
                },
            })
            .expect("find elements");
        let AutomationResult::Elements(references) = result else {
            panic!("expected elements list");
        };
        assert_eq!(references.len(), 2, "两个 input 都应命中");

        // 空匹配返回空列表（W3C：非错误）。
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::FindElements {
                    using: zero_protocol::message::AutomationLocatorStrategy::CssSelector,
                    value: "#missing".into(),
                },
            })
            .expect("find elements no match");
        let AutomationResult::Elements(references) = result else {
            panic!("expected elements list");
        };
        assert!(references.is_empty(), "无匹配应返回空列表");
    }

    #[test]
    fn unsupported_automation_operation_is_explicit() {
        let mut runtime = runtime();
        let error = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::Unsupported {
                    name: "test_driver.set_permission".into(),
                },
            })
            .expect_err("unsupported operation");
        assert_eq!(error.code, AutomationErrorCode::UnsupportedOperation);
    }

    // ── objectId 句柄桥（CDP Runtime.evaluate returnByValue:false 语义）──

    /// 保留对象返回句柄引用、原始类型按值返回。
    #[test]
    fn evaluate_retaining_splits_objects_and_primitives() {
        let mut runtime = runtime();
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::EvaluateRetaining {
                    script: "1 + 2".into(),
                    group: None,
                    return_by_value: false,
                },
            })
            .expect("retain primitive");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(3.0)));

        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::EvaluateRetaining {
                    script: "({a: 1, b: 21})".into(),
                    group: None,
                    return_by_value: false,
                },
            })
            .expect("retain object");
        let AutomationResult::Value(AutomationValue::Handle(_)) = result else {
            panic!("object result must be a handle, got {result:?}");
        };
    }

    /// 句柄对象作 `this` 调用函数；句柄实参还原为保留对象。
    #[test]
    fn call_function_on_handle_resolves_this_and_handle_arguments() {
        let mut runtime = runtime();
        let target = retain_object(&mut runtime, "({v: 20})");
        let argument = retain_object(&mut runtime, "({v: 1})");
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: target.id,
                    function_declaration: "(function (o) { return this.v + o.v; })".into(),
                    arguments: vec![AutomationValue::Handle(argument)],
                    return_by_value: true,
                    await_promise: false,
                    group: None,
                },
            })
            .expect("call on handle");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(21.0)));
    }

    /// returnByValue:false 的对象结果保留为新句柄（Playwright 句柄链）。
    #[test]
    fn call_function_on_handle_retains_object_results() {
        let mut runtime = runtime();
        let target = retain_object(&mut runtime, "({v: 5})");
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: target.id,
                    function_declaration: "(function () { return { nested: this.v * 2 }; })".into(),
                    arguments: vec![],
                    return_by_value: false,
                    await_promise: false,
                    group: None,
                },
            })
            .expect("retain call result");
        let AutomationResult::Value(AutomationValue::Handle(nested)) = result else {
            panic!("object result must be a handle, got {result:?}");
        };
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: nested.id,
                    function_declaration: "(function () { return this.nested + 1; })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: false,
                    group: None,
                },
            })
            .expect("read nested via handle");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(11.0)));
    }

    /// `awaitPromise`：已落定 Promise 与 setTimeout 异步落定都等待后返回。
    #[test]
    fn call_function_on_handle_awaits_promises() {
        let mut runtime = runtime();
        let target = retain_object(&mut runtime, "({v: 1})");
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: target.id,
                    function_declaration: "(function () { return Promise.resolve(7); })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: true,
                    group: None,
                },
            })
            .expect("await settled promise");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(7.0)));

        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: target.id,
                    function_declaration: "(function () { var self = this; return new Promise(function (r) { setTimeout(function () { r(self.v + 3); }, 30); }); })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: true,
                    group: None,
                },
            })
            .expect("await timer promise");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(4.0)));
    }

    /// releaseHandle 后句柄引用失效（CDP releaseObject 配对语义）。
    #[test]
    fn release_handle_invalidates_reference() {
        let mut runtime = runtime();
        let handle = retain_object(&mut runtime, "({v: 1})");
        runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::ReleaseHandle { handle: handle.id },
            })
            .expect("release handle");
        let error = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: handle.id,
                    function_declaration: "(function () { return this.v; })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: false,
                    group: None,
                },
            })
            .expect_err("released handle must fail");
        assert_eq!(error.code, AutomationErrorCode::JavascriptError);
    }

    /// releaseObjectGroup 整组失效，组外句柄不受影响。
    #[test]
    fn release_object_group_scopes_to_group() {
        let mut runtime = runtime();
        let grouped = retain_object_in_group(&mut runtime, "({v: 1})", "gtest");
        let untouched = retain_object_in_group(&mut runtime, "({v: 2})", "other");
        runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::ReleaseObjectGroup { group: "gtest".into() },
            })
            .expect("release group");
        let error = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: grouped.id,
                    function_declaration: "(function () { return this.v; })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: false,
                    group: None,
                },
            })
            .expect_err("grouped handle must be released");
        assert_eq!(error.code, AutomationErrorCode::JavascriptError);
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::CallFunctionOnHandle {
                    handle: untouched.id,
                    function_declaration: "(function () { return this.v; })".into(),
                    arguments: vec![],
                    return_by_value: true,
                    await_promise: false,
                    group: None,
                },
            })
            .expect("other group survives");
        assert_eq!(result, AutomationResult::Value(AutomationValue::Number(2.0)));
    }

    fn retain_object(runtime: &mut RendererRuntime, script: &str) -> zero_protocol::message::AutomationHandleRef {
        retain_object_in_group(runtime, script, "zw-automation")
    }

    fn retain_object_in_group(
        runtime: &mut RendererRuntime,
        script: &str,
        group: &str,
    ) -> zero_protocol::message::AutomationHandleRef {
        let result = runtime
            .execute_automation_request(AutomationRequest {
                operation: AutomationOperation::EvaluateRetaining {
                    script: script.into(),
                    group: Some(group.into()),
                    return_by_value: false,
                },
            })
            .expect("retain object");
        let AutomationResult::Value(AutomationValue::Handle(handle)) = result else {
            panic!("expected handle, got {result:?}");
        };
        handle
    }
}
