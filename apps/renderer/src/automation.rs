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
        if changed {
            self.publish_webview(None, true).map_err(internal_error)?;
        }
        Ok(value)
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
    }
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
}
