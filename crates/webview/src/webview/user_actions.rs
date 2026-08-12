use std::sync::{Arc, Mutex};

#[cfg(feature = "v8")]
use zero_engine::script_dispatch_native_event;
use zero_engine::{
    DomEventDetail, DomMutation, register_dom_callbacks, script_dispatch_dom_event, script_reset_form_controls,
    script_set_control_checked, script_set_text_control_state, script_text_control_snapshot,
};
use zero_page_runtime::{
    ActionNoopReason, ActionTargetState, EventDispatchResult, FormNavigationIntent, HtmlActionRequest, HtmlUserAction,
    InvalidationKind, PageEffect, PageNodeHandle, PageNodeRef, PlannedEvent, PlannedMutation, RadioActionState,
    TextActionState, plan_html_action, resolve_html_action,
};

use super::{WebView, WebViewError, render_result_to_webview};

/// ZeroWebView 内部 user-action 执行结果。
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebViewUserActionResult {
    /// 动作是否修改了页面状态。
    pub changed: bool,
    /// cancelable 事件是否取消了默认动作。
    pub canceled: bool,
    /// 无动作时的显式原因。
    pub noop_reason: Option<ActionNoopReason>,
    /// 需要嵌入宿主消费的 typed effects。
    pub effects: Vec<PageEffect>,
    /// 动作请求的最终失效级别。
    pub invalidation: InvalidationKind,
}

impl WebViewUserActionResult {
    fn noop(reason: ActionNoopReason) -> Self {
        Self {
            changed: false,
            canceled: false,
            noop_reason: Some(reason),
            effects: Vec::new(),
            invalidation: InvalidationKind::None,
        }
    }
}

struct DomScriptResult {
    value: String,
    changed: bool,
}

impl WebView {
    /// 返回当前文档中 selector 对应的 scoped node reference。
    #[doc(hidden)]
    pub fn page_node_ref_for_selector(&self, selector: &str) -> Option<PageNodeRef> {
        let handle = self.page_node_handle_for_selector(selector)?;
        Some(PageNodeRef::new(
            self.navigation_epoch,
            self.document_generation,
            PageNodeHandle::new(handle),
        ))
    }

    /// 返回当前 focus owner。
    #[doc(hidden)]
    pub fn user_action_focus_owner(&self) -> Option<PageNodeRef> {
        self.focus_owner
    }

    /// 通过 shared action core 执行 identity-based HTML user action。
    #[doc(hidden)]
    pub fn dispatch_user_action(
        &mut self,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        self.run_page_scripts()?;
        if !request
            .target
            .is_current(self.navigation_epoch, self.document_generation)
        {
            return Ok(WebViewUserActionResult::noop(ActionNoopReason::StaleTarget));
        }
        let Some(selector) = self.selector_for_page_node_handle(request.target.node().get()) else {
            return Ok(WebViewUserActionResult::noop(ActionNoopReason::MissingTarget));
        };
        let html = self.cached_html.clone();
        if zero_engine::has_attribute(&html, &selector, "disabled") {
            return Ok(WebViewUserActionResult::noop(ActionNoopReason::DisabledTarget));
        }
        let state = match &request.action {
            HtmlUserAction::InsertText { .. } | HtmlUserAction::DeleteBackward => {
                let snapshot = self.execute_dom_script(&script_text_control_snapshot(&selector))?;
                let Some((value, selection_start, selection_end)) =
                    serde_json::from_str::<(String, usize, usize)>(&snapshot.value).ok()
                else {
                    return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                };
                ActionTargetState::Text(TextActionState {
                    value,
                    selection_start,
                    selection_end,
                })
            }
            HtmlUserAction::Activate if zero_engine::is_checkbox(&html, &selector) => ActionTargetState::Checkbox {
                checked: zero_engine::has_attribute(&html, &selector, "checked"),
            },
            HtmlUserAction::Activate if zero_engine::is_radio(&html, &selector) => {
                ActionTargetState::Radio(RadioActionState {
                    checked: zero_engine::has_attribute(&html, &selector, "checked"),
                    previous_checked: zero_engine::checked_radio_group_selector(&html, &selector)
                        .and_then(|previous| self.page_node_ref_for_selector(&previous)),
                })
            }
            HtmlUserAction::MoveFocus { forward } => {
                let next = zero_engine::next_focus_selector(&html, Some(&selector), *forward)
                    .and_then(|next| self.page_node_ref_for_selector(&next));
                ActionTargetState::Focus { next }
            }
            HtmlUserAction::Reset => {
                let Some(form) = zero_engine::enclosing_form_selector(&html, &selector)
                    .and_then(|form| self.page_node_ref_for_selector(&form))
                else {
                    return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                };
                ActionTargetState::Reset { form }
            }
            HtmlUserAction::Submit => {
                let Some(form) = zero_engine::enclosing_form_selector(&html, &selector)
                    .and_then(|form| self.page_node_ref_for_selector(&form))
                else {
                    return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                };
                ActionTargetState::Submit {
                    form,
                    submitter: zero_engine::is_submit_button(&html, &selector).then_some(request.target),
                }
            }
            _ => return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable)),
        };
        let plan = match plan_html_action(&request, self.navigation_epoch, self.document_generation, &state) {
            Ok(plan) => plan,
            Err(reason) => return Ok(WebViewUserActionResult::noop(reason)),
        };
        let is_reset = matches!(request.action, HtmlUserAction::Reset);
        let mut changed = self.apply_planned_mutations(&plan.prepare)?;
        if is_reset {
            changed |= self.execute_dom_script("__zw_begin_host_action_transaction()")?.changed;
        }
        let dispatch = if let Some(event) = plan.cancelable_event.as_ref() {
            self.dispatch_planned_event(event)?
        } else {
            (true, false)
        };
        let outcome = resolve_html_action(
            plan,
            EventDispatchResult {
                default_allowed: dispatch.0,
                html_changed: dispatch.1,
            },
        );
        changed |= outcome.html_changed;
        changed |= self.apply_planned_mutations(&outcome.mutations)?;
        if is_reset {
            changed |= self.execute_dom_script("__zw_end_host_action_transaction()")?.changed;
        }
        for event in &outcome.followup_events {
            changed |= self.dispatch_planned_event(event)?.1;
        }
        let mut effects = Vec::new();
        for effect in outcome.effects {
            match effect {
                PageEffect::Focus(next) => {
                    self.focus_owner = next;
                    effects.push(PageEffect::Focus(next));
                }
                PageEffect::SubmitForm { form, submitter } => {
                    if let Some(intent) = self.form_navigation_intent(form, submitter) {
                        effects.push(PageEffect::Navigate(intent));
                    }
                }
                effect => effects.push(effect),
            }
        }
        Ok(WebViewUserActionResult {
            changed,
            canceled: outcome.canceled,
            noop_reason: None,
            effects,
            invalidation: outcome.invalidation,
        })
    }

    fn dispatch_planned_event(&mut self, event: &PlannedEvent) -> Result<(bool, bool), WebViewError> {
        let Some(selector) = self.selector_for_page_node_handle(event.target.node().get()) else {
            return Ok((false, false));
        };
        let detail = event.input_type.as_ref().map(|input_type| DomEventDetail {
            data: event.data.clone(),
            input_type: Some(input_type.clone()),
            ..Default::default()
        });
        let script = script_dispatch_dom_event(&selector, &event.event_type, detail.as_ref());
        let result = self.execute_dom_script(&script)?;
        #[cfg(feature = "v8")]
        if self.config.native_dom {
            let native = self.execute_dom_script(&script_dispatch_native_event(&selector, &event.event_type))?;
            return Ok((result.value.trim() != "prevented", result.changed || native.changed));
        }
        Ok((result.value.trim() != "prevented", result.changed))
    }

    fn apply_planned_mutations(&mut self, mutations: &[PlannedMutation]) -> Result<bool, WebViewError> {
        let mut changed = false;
        for mutation in mutations {
            let script = match mutation {
                PlannedMutation::SetText {
                    target,
                    value,
                    selection_start,
                    selection_end,
                } => {
                    let Some(selector) = self.selector_for_page_node_handle(target.node().get()) else {
                        continue;
                    };
                    script_set_text_control_state(&selector, value, *selection_start, *selection_end)
                }
                PlannedMutation::SetChecked { target, checked } => {
                    let Some(selector) = self.selector_for_page_node_handle(target.node().get()) else {
                        continue;
                    };
                    script_set_control_checked(&selector, *checked)
                }
                PlannedMutation::ResetForm { form } => {
                    let Some(selector) = self.selector_for_page_node_handle(form.node().get()) else {
                        continue;
                    };
                    script_reset_form_controls(&selector)
                }
            };
            changed |= self.execute_dom_script(&script)?.changed;
        }
        Ok(changed)
    }

    fn form_navigation_intent(
        &self,
        form: PageNodeRef,
        submitter: Option<PageNodeRef>,
    ) -> Option<FormNavigationIntent> {
        let form = self.selector_for_page_node_handle(form.node().get())?;
        let submitter = submitter.and_then(|node| self.selector_for_page_node_handle(node.node().get()));
        let base = self.current_url.as_deref().unwrap_or("about:blank");
        let values = self.form_control_value_overrides();
        if let Some((url, body)) =
            zero_engine::form_post_submission_with_values(&self.cached_html, &form, submitter.as_deref(), base, &values)
        {
            return Some(FormNavigationIntent {
                url,
                method: "POST".to_string(),
                body: Some(body),
            });
        }
        zero_engine::form_get_submission_url_with_values(&self.cached_html, &form, submitter.as_deref(), base, &values)
            .map(|url| FormNavigationIntent {
                url,
                method: "GET".to_string(),
                body: None,
            })
    }

    fn execute_dom_script(&mut self, script: &str) -> Result<DomScriptResult, WebViewError> {
        self.ensure_sandbox()?;
        #[cfg(feature = "v8")]
        self.install_native_dom_bindings();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
        let dom_html = Arc::new(Mutex::new(self.cached_html.clone()));
        let page_url = Arc::new(Mutex::new(self.current_url.clone().unwrap_or_default()));
        {
            let sandbox = self
                .js_sandbox
                .as_mut()
                .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?;
            register_dom_callbacks(&mut **sandbox, &mutations, &dom_html, &page_url, &self.canvas_registry);
        }
        self.ensure_js_shim()?;
        let value = self
            .js_sandbox
            .as_mut()
            .expect("js sandbox")
            .execute(script)
            .map_err(|error| WebViewError::Script(error.to_string()))?
            .value;
        let recorded = mutations.lock().unwrap_or_else(|error| error.into_inner()).clone();
        if recorded.is_empty() {
            #[cfg(feature = "v8")]
            self.sync_render_after_native_dom();
            return Ok(DomScriptResult { value, changed: false });
        }
        let (result, html_snapshot, _) = self
            .pipeline
            .render_with_dom_mutations(&recorded, &self.cached_css)
            .map_err(|error| WebViewError::Script(format!("apply mutations: {error}")))?;
        if let Some(html) = html_snapshot {
            self.cached_html = html;
        }
        self.last_render = Some(render_result_to_webview(&result));
        Ok(DomScriptResult { value, changed: true })
    }
}
