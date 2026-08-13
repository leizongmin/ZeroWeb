//! Live renderer automation operations shared by WebDriver and WPT testdriver.

use zero_protocol::message::{
    AutomationElementRef, AutomationError, AutomationErrorCode, AutomationKey, AutomationOperation, AutomationRequest,
    AutomationResponse, AutomationResult, AutomationValue, IpcMessageKind, KeyboardEventParams, KeyboardEventType,
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
                let source = format!("(function(){{return (function(){{{script}\n}}).apply(null,{arguments});}})()");
                let current_url = self.current_url.as_deref().unwrap_or("about:blank").to_string();
                let (value, changed) = {
                    let mut context = PageScriptContext {
                        html: &mut self.cached_html,
                        url: &current_url,
                        js_worker: &self.js_worker,
                        webview: self.webview.as_mut(),
                    };
                    super::page_scripts::execute_automation_script(&mut context, &source)
                        .map_err(|message| automation_error(AutomationErrorCode::JavascriptError, message))?
                };
                self.sync_focus_from_js();
                self.sync_cached_html_from_webview();
                if changed {
                    self.publish_webview(None, true).map_err(internal_error)?;
                }
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
