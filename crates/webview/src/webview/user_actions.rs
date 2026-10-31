use std::borrow::Cow;
use std::sync::{Arc, Mutex};

#[cfg(feature = "v8")]
use zero_engine::script_dispatch_native_event;
use zero_engine::{
    DomEventDetail, DomMutation, register_dom_callbacks, script_call_set_location_hash, script_contenteditable_delete,
    script_contenteditable_enter, script_contenteditable_forward_delete, script_contenteditable_insert,
    script_contenteditable_insert_paragraph, script_contenteditable_probe, script_dispatch_dom_event,
    script_pointer_move, script_reset_form_controls, script_set_control_checked, script_set_open,
    script_set_option_selected, script_set_text_control_state, script_text_control_snapshot, script_text_delete,
    script_text_delete_without_event, script_text_input, script_text_input_without_event,
};
use zero_page_runtime::{
    ActionNoopReason, ActionTargetState, EventDispatchResult, FormNavigationIntent, HtmlActionRequest, HtmlUserAction,
    InvalidationKind, JsExecutor, OptionActionState, PageEffect, PageNodeHandle, PageNodeRef, PlannedEvent,
    PlannedMutation, RadioActionState, SummaryActionState, TextActionState, plan_html_action, resolve_html_action,
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

    /// 更新内部 executor 的文本控件 selection，不派发页面事件。
    #[doc(hidden)]
    pub fn set_text_control_selection(
        &mut self,
        target: PageNodeRef,
        selection_start: usize,
        selection_end: usize,
    ) -> Result<(), WebViewError> {
        self.set_text_control_selection_impl(None, target, selection_start, selection_end)
    }

    /// 更新外部 executor 的文本控件 selection，不派发页面事件。
    #[doc(hidden)]
    pub fn set_external_text_control_selection(
        &mut self,
        executor: &dyn JsExecutor,
        target: PageNodeRef,
        selection_start: usize,
        selection_end: usize,
    ) -> Result<(), WebViewError> {
        self.set_text_control_selection_impl(Some(executor), target, selection_start, selection_end)
    }

    fn set_text_control_selection_impl(
        &mut self,
        executor: Option<&dyn JsExecutor>,
        target: PageNodeRef,
        selection_start: usize,
        selection_end: usize,
    ) -> Result<(), WebViewError> {
        if !target.is_current(self.navigation_epoch, self.document_generation) {
            return Ok(());
        }
        let Some(selector) = self.selector_for_page_node_handle(target.node().get()) else {
            return Ok(());
        };
        self.execute_dom_script(
            executor,
            &zero_engine::script_set_text_control_selection(&selector, selection_start, selection_end),
        )?;
        Ok(())
    }

    /// 通过 shared action core 执行 identity-based HTML user action。
    #[doc(hidden)]
    pub fn dispatch_user_action(
        &mut self,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        self.dispatch_user_action_impl(None, true, true, request)
    }

    /// 通过内部 executor 执行 action，并显式控制页面 JavaScript 事件派发。
    #[doc(hidden)]
    pub fn dispatch_user_action_with_javascript(
        &mut self,
        javascript_enabled: bool,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        self.dispatch_user_action_impl(None, javascript_enabled, true, request)
    }

    /// 在页面脚本已初始化的同一 sandbox 中执行自动化动作。
    #[doc(hidden)]
    pub fn dispatch_loaded_page_user_action(
        &mut self,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        self.dispatch_user_action_impl(None, true, false, request)
    }

    /// 通过外部页面 JS executor 执行 shared HTML user action。
    #[doc(hidden)]
    pub fn dispatch_external_user_action(
        &mut self,
        executor: &dyn JsExecutor,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        self.dispatch_user_action_impl(Some(executor), true, false, request)
    }

    /// 通过外部 executor 执行 action，并显式控制页面 JavaScript 事件派发。
    #[doc(hidden)]
    pub fn dispatch_external_user_action_with_javascript(
        &mut self,
        executor: &dyn JsExecutor,
        javascript_enabled: bool,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        self.dispatch_user_action_impl(Some(executor), javascript_enabled, false, request)
    }

    fn dispatch_user_action_impl(
        &mut self,
        executor: Option<&dyn JsExecutor>,
        javascript_enabled: bool,
        initialize_page_scripts: bool,
        request: HtmlActionRequest,
    ) -> Result<WebViewUserActionResult, WebViewError> {
        if initialize_page_scripts && executor.is_none() && javascript_enabled {
            self.run_page_scripts()?;
        }
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
        if zero_engine::is_effectively_disabled(&html, &selector) {
            return Ok(WebViewUserActionResult::noop(ActionNoopReason::DisabledTarget));
        }
        // R4008（html-semantics M2 切片 19）：label 本体激活的宿主转发探测。spec
        // the-label-element activation behavior——click 落 label 自身（或非交互后代）
        // → 派发完成后激活 labeled control。shim 不承接宿主 generic Event 的转发
        //（renderer 真实输入路径由 Rust 默认动作层 activate_label_at 负责，shim 再
        // 转发会双激活），plan 路径（testdriver）的转发在本层：label 自身照常走
        // Generic plan 派发 click（页面 label 监听器先收到事件），未取消时对 control
        // 递归 Activate（合成 click 激活步骤）。解析器过滤 labelable（label 非
        // labelable）——递归不可能回到 label，无自引用环。
        // https://html.spec.whatwg.org/multipage/forms.html#the-label-element
        let label_control_selector = if matches!(request.action, HtmlUserAction::Activate)
            && zero_engine::query_tag_from_html(&html, &selector).eq_ignore_ascii_case("LABEL")
        {
            zero_engine::associated_label_control_selector(&html, &selector)
        } else {
            None
        };
        let state = match &request.action {
            // 尾簇 31：ForwardDelete 同走 text-control 探测，但 **CE 宿主不分流**——
            // `__zw_ce_delete` 是 Backward 语义（caret 前退），ForwardDelete 复用会删错
            // 字符（delete.html CE div 案 'a' 被误删）；CE ForwardDelete（range 后向
            // 删除）挂账，此处落 text-control 快照对 contenteditable 返 noop
            //（NotApplicable）。
            HtmlUserAction::InsertText { .. } | HtmlUserAction::DeleteBackward | HtmlUserAction::ForwardDelete => {
                // R3254-M2 切片 2（editing goal）：焦点元素为 contenteditable 宿主时走
                // CE 键入/删除管线（shim __zw_ce_insert/delete——Selection caret 处
                // deleteContents/insertNode 真实 DOM 变更 + beforeinput/input 事件序），
                // 不落 TextActionState（那是 input/textarea value 模型）。text control
                // 仍走既有快照+plan 管线（零变化）。
                let ce_probe = self.execute_dom_script(executor, &script_contenteditable_probe(&selector))?;
                if ce_probe.value.trim() == "1" {
                    let script = match &request.action {
                        // R3254-M2 切片 3：CE 宿主内 "\n"（textarea/worker Enter 路由复用
                        // InsertText 通道）→ __zw_ce_enter（<br> 插入）而非字面插入。
                        HtmlUserAction::InsertText { text } if text == "\n" => script_contenteditable_enter(&selector),
                        HtmlUserAction::InsertText { text } => script_contenteditable_insert(&selector, text),
                        // 尾簇 32：CE ForwardDelete 接通（`__zw_ce_forward_delete`——
                        // deleteContentForward 语义；取代尾簇 31 的 noop 挂账）。
                        HtmlUserAction::ForwardDelete => script_contenteditable_forward_delete(&selector),
                        _ => script_contenteditable_delete(&selector),
                    };
                    let result = self.execute_dom_script(executor, &script)?;
                    return Ok(WebViewUserActionResult {
                        changed: result.changed,
                        canceled: false,
                        noop_reason: None,
                        effects: Vec::new(),
                        invalidation: if result.changed {
                            InvalidationKind::Paint
                        } else {
                            InvalidationKind::None
                        },
                    });
                }
                // R3254-K4（keyboard-default-actions goal M2，2026-09-07）：空格对
                // button 类控件 = 激活（click 合成）而非文本插入。InsertText 通道落到
                // button-ish 目标（BUTTON / input type=button|submit|reset）时递归为
                // Activate 请求（复用既有激活管线全流程——状态构建/click 派发/disabled
                // 守卫/effects）。Enter 已由 Submit 臂覆盖（enclosing form / submit
                // button→click→submit 语义）；此处闭合空格。
                // R4009（html-semantics M2 切片 20）：SUMMARY 可聚焦（focus.rs
                // FOCUSABLE_TAGS 既有）且空格为其激活键（spec the-summary-element
                // activation behaviour）——递归 Activate 落既有 plan_summary 管线。
                // details-keyboard-activation 语料 summary.focus()+send_keys(space)
                // 断言面（open 翻转且不滚动）。
                if let HtmlUserAction::InsertText { text } = &request.action
                    && text == " "
                {
                    let tag = zero_engine::query_tag_from_html(&html, &selector);
                    let input_type = zero_engine::query_attr_from_html(&html, &selector, "type");
                    let buttonish = tag.eq_ignore_ascii_case("BUTTON")
                        || tag.eq_ignore_ascii_case("SUMMARY")
                        || (tag.eq_ignore_ascii_case("INPUT")
                            && matches!(input_type.to_ascii_lowercase().as_str(), "button" | "submit" | "reset"));
                    if buttonish {
                        return self.dispatch_user_action_impl(
                            executor,
                            javascript_enabled,
                            false,
                            HtmlActionRequest {
                                target: request.target,
                                action: HtmlUserAction::Activate,
                                shift: request.shift,
                            },
                        );
                    }
                }
                // t2-pd2a：文本控件写入走**原子脚本**（`__zw_text_input`/`__zw_text_delete`
                // ——活状态读 value+selection、改值、beforeinput→textInput→input 事件序
                // 一体，单次优先通道往返）。旧「快照读→宿主计划→内嵌全值 SetText」三段
                // 管线在击键重叠时按陈旧快照整值覆盖写入（lost-update）——页面负载下 2s
                // 有界等待超时使重叠成为常态（DDG 首页实测 "rust"→"rsut"、24 字符落 2
                // 字符；迟到脚本按旧计划整值覆盖新状态）。快照读+计划仅承载 noop 契约
                //（ReadOnlyTarget/MaxLengthReached/NothingToDelete 原因面），不再产生
                // 写计划；readonly/maxlength/截断守卫同点下沉 shim（活读取）。 canceled
                // 经脚本 "prevented" 返回值传递（beforeinput 取消语义）。ForwardDelete
                // 留计划管线（无原子脚本面；非连续键入路径，挂账池）。
                let snapshot = self.execute_dom_script(executor, &script_text_control_snapshot(&selector))?;
                let Some((value, selection_start, selection_end)) =
                    serde_json::from_str::<(String, usize, usize)>(&snapshot.value).ok()
                else {
                    return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                };
                let state = TextActionState {
                    value,
                    selection_start,
                    selection_end,
                    read_only: zero_engine::has_attribute(&html, &selector, "readonly"),
                    max_length: zero_engine::query_attr_from_html(&html, &selector, "maxlength")
                        .parse()
                        .ok(),
                };
                // 计划仅作 noop 判定（Err = 旧契约原因面）；Ok 不取其写计划。
                let plan_only_reason = plan_html_action(
                    &request,
                    self.navigation_epoch,
                    self.document_generation,
                    &ActionTargetState::Text(state.clone()),
                )
                .err();
                let atomic_script = match (&request.action, plan_only_reason) {
                    (_, Some(reason)) => return Ok(WebViewUserActionResult::noop(reason)),
                    (HtmlUserAction::InsertText { text }, _) if !text.is_empty() => {
                        // JS 禁用（UA 默认动作语义）：无事件变体——插入不派
                        // beforeinput/input（同计划管线 javascript_enabled=false 臂，
                        // 页面监听器不参与取消）；JS 启用走完整事件序。
                        if javascript_enabled {
                            Some(script_text_input(&selector, text))
                        } else {
                            Some(script_text_input_without_event(&selector, text))
                        }
                    }
                    (HtmlUserAction::DeleteBackward, _) => {
                        if javascript_enabled {
                            Some(script_text_delete(&selector))
                        } else {
                            Some(script_text_delete_without_event(&selector))
                        }
                    }
                    _ => None,
                };
                if let Some(script) = atomic_script {
                    let result = self.execute_dom_script(executor, &script)?;
                    let canceled = result.value.trim() == "prevented";
                    return Ok(WebViewUserActionResult {
                        changed: result.changed,
                        canceled,
                        noop_reason: None,
                        effects: Vec::new(),
                        invalidation: if result.changed {
                            InvalidationKind::Paint
                        } else {
                            InvalidationKind::None
                        },
                    });
                }
                ActionTargetState::Text(state)
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
            HtmlUserAction::Activate => {
                if let Some(hash) = zero_engine::anchor_hash_target(&html, &selector) {
                    ActionTargetState::Fragment { hash }
                } else if let Some(url) = zero_engine::anchor_click_target(
                    &html,
                    &selector,
                    self.current_url.as_deref().unwrap_or("about:blank"),
                ) {
                    ActionTargetState::Navigate {
                        intent: FormNavigationIntent {
                            url,
                            method: "GET".to_string(),
                            body: None,
                        },
                    }
                } else if let Some(summary) = zero_engine::summary_activation_snapshot(&html, &selector) {
                    let Some(details) = self.page_node_ref_for_selector(&summary.details_selector) else {
                        return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                    };
                    ActionTargetState::Summary(SummaryActionState {
                        details,
                        open: summary.open,
                    })
                } else if let Some(option) = zero_engine::option_activation_snapshot(&html, &selector) {
                    let Some(select) = self.page_node_ref_for_selector(&option.select_selector) else {
                        return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                    };
                    ActionTargetState::Option(OptionActionState {
                        select,
                        selected: option.selected,
                        multiple: option.multiple,
                        previous_selected: option
                            .previous_selected_selector
                            .and_then(|previous| self.page_node_ref_for_selector(&previous)),
                    })
                } else {
                    // js-dom R142：非表单/非链接/非 summary/非 option 的普通元素（contenteditable
                    // 宿主等）——激活仍须派发 click 事件（真实浏览器对任意元素点击都有 click）；
                    // 旧 NotApplicable 使合成指针点击整簇哑火（WPT no-focus-events）。
                    ActionTargetState::Generic
                }
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
                // R3254-K1（keyboard goal M1）：Enter 默认动作分发序——**编辑宿主优先
                // 消费**（CE 宿主 → insertParagraph 语义就地拆分/退化，editing-compat
                // insertparagraph 簇 2026-10-30 前为 __zw_ce_enter 换行；宿主在表单内
                // 也不提交）；未消费才走表单隐式提交管线（enclosing form → submit
                // 事件+导航意图）。FIXME: submit 通道无修饰载荷，Shift+Enter 未区分
                //（runner Actions 路径同汇于此；inline 宿主退化后输出一致，无语料分压）。
                let ce_probe = self.execute_dom_script(executor, &script_contenteditable_probe(&selector))?;
                if ce_probe.value.trim() == "1" {
                    let result =
                        self.execute_dom_script(executor, &script_contenteditable_insert_paragraph(&selector))?;
                    return Ok(WebViewUserActionResult {
                        changed: result.changed,
                        canceled: false,
                        noop_reason: None,
                        effects: Vec::new(),
                        invalidation: if result.changed {
                            InvalidationKind::Paint
                        } else {
                            InvalidationKind::None
                        },
                    });
                }
                let Some(form_sel) = zero_engine::enclosing_form_selector(&html, &selector) else {
                    return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                };
                let Some(form) = self.page_node_ref_for_selector(&form_sel) else {
                    return Ok(WebViewUserActionResult::noop(ActionNoopReason::NotApplicable));
                };
                // R3254-K3 切片 B（keyboard goal，2026-09-07）：submitter 判定——**点击
                // submit 按钮**（selector 自身是按钮）→ submitter = 按钮；**Enter 隐式提交**
                // → spec「default button = 表单 tree order 首个 submit button」：
                //   ① default button 存在但 disabled → 提交无动作（WPT implicit-submission
                //     "disabled submit button"——unreached_func 守卫）；
                //   ② 存在且 enabled → submitter = default button；
                //   ③ 不存在 → 直接提交，submitter = None。
                //   https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#implicit-submission
                let submitter_sel = if zero_engine::is_submit_button(&html, &selector) {
                    Some(selector.clone())
                } else {
                    zero_engine::default_submit_button_selector(&html, &form_sel)
                };
                if let Some(button_sel) = &submitter_sel {
                    // spec dom-form-submit：default button disabled（祖先 fieldset disabled
                    // 的整体面未实现——记录）→ 隐式提交无动作。disabled 探针经脚本（live
                    // 反射含 host apply 前的最新状态）。
                    let probe = zero_engine::script_control_disabled_probe(button_sel);
                    let disabled = self.execute_dom_script(executor, &probe)?.value.trim() == "1";
                    if disabled && !zero_engine::is_submit_button(&html, &selector) {
                        // spec：disabled default button → 隐式提交无动作。返**成功 + 零
                        // effects**（非 noop——runner send_keys 对 noop(NotApplicable) 视为
                        // 错误抛出；WPT implicit-submission "disabled submit button" 的
                        // unreached_func 守卫期望 send_keys 正常 resolve）。
                        return Ok(WebViewUserActionResult {
                            changed: false,
                            canceled: false,
                            noop_reason: None,
                            effects: Vec::new(),
                            invalidation: InvalidationKind::None,
                        });
                    }
                }
                let submitter = submitter_sel.and_then(|sel| self.page_node_ref_for_selector(&sel));
                ActionTargetState::Submit { form, submitter }
            }
        };
        let mut plan = match plan_html_action(&request, self.navigation_epoch, self.document_generation, &state) {
            Ok(plan) => plan,
            Err(reason) => return Ok(WebViewUserActionResult::noop(reason)),
        };
        // uievents-compat M2 片 1（2026-10-03）：激活/悬停迁移前置跨界序。宿主激活
        // = 指针点击——spec Pointer Events §11/UI Events §5.3：over/out/enter/leave
        // 边界事件先于 pointerdown/mousedown 序列派发。悬停目标与上次不同 → 调
        // `__zw_pointer_move`（跨界序 + move 对，坐标近似 0,0——合成激活无真命中点）；
        // 相同 → 免重派（真实浏览器同元素连击不重发边界序）。InsertText/keydown 等
        // 非指针动作不迁移悬停。
        let is_pointer_action = matches!(request.action, HtmlUserAction::Activate);
        let target_selector = selector.clone();
        if javascript_enabled && is_pointer_action && self.pointer_over.as_deref() != Some(target_selector.as_str()) {
            self.execute_dom_script(executor, &script_pointer_move(&target_selector, 0.0, 0.0, 0, 0))?;
            self.pointer_over = Some(target_selector.clone());
        }
        // 同目标连击（dblclick 判定，UI Events §3.3——detail 按连击计数；目标变化
        // 即重置。headless 无时间窗：同目标连击近似，跨动作空白期不去抖）。
        let mut dblclick = false;
        if is_pointer_action {
            if self.click_streak_target.as_deref() == Some(target_selector.as_str()) {
                self.click_streak_count += 1;
            } else {
                self.click_streak_target = Some(target_selector.clone());
                self.click_streak_count = 1;
            }
            if self.click_streak_count >= 2 {
                dblclick = true;
            }
        }
        if dblclick {
            // 第二次及以后的同目标激活 → click 之后补派 dblclick（UI Events
            // §5.2.2 事件序 click → dblclick；detail=2 连击计数）。
            plan.followup_events
                .push(zero_page_runtime::PlannedEvent::dblclick(plan.target));
        }
        // click 的 UIEvent.detail = 连击计数（spec UI Events §3.3——首次点击 1、
        // 连击递增；WPT dblclick_event_mouse 断言 click detail 1 / dblclick detail 2）。
        if is_pointer_action
            && let Some(click_event) = plan.cancelable_event.as_mut()
            && click_event.event_type == "click"
        {
            click_event.detail = Some(self.click_streak_count);
        }
        let is_reset = matches!(request.action, HtmlUserAction::Reset);
        let mut changed = self.apply_planned_mutations(executor, &plan.prepare)?;
        if is_reset {
            changed |= self
                .execute_dom_script(executor, "__zw_begin_host_action_transaction()")?
                .changed;
        }
        let dispatch = if javascript_enabled {
            // js-dom R144：指针激活序列的 pre_events（mousedown/mouseup）先于 cancelable
            // click 派发（listener 内的 DOM 变更——伪元素移除/节点移动——发生在 click 前，
            // click 在新状态上照常派发）。pre_event 的 canceled 分支忽略（mousedown 取消
            // 不抑制后续 click——简化：不可取消路径的 html_changed 仍并入）。
            for event in &plan.pre_events {
                changed |= self.dispatch_planned_event(executor, event)?.1;
            }
            if let Some(event) = plan.cancelable_event.as_ref() {
                self.dispatch_planned_event(executor, event)?
            } else {
                (true, false)
            }
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
        changed |= self.apply_planned_mutations(executor, &outcome.mutations)?;
        if is_reset {
            changed |= self
                .execute_dom_script(executor, "__zw_end_host_action_transaction()")?
                .changed;
        }
        if javascript_enabled {
            for event in &outcome.followup_events {
                changed |= self.dispatch_planned_event(executor, event)?.1;
            }
        }
        let mut effects = Vec::new();
        // navigation-compat S4I：锚激活线程（testdriver click 的 host 管线此前绕过 JS 锚分支
        // ——navigate 事件的 sourceElement/downloadRequest 恒缺）。在 hash/锚导航 effect 执行前
        // 把触发元素 + download 属性线程进页面（`__zwNavSourceElement`/`__zwNavDownloadRequest`
        // ——shim hash-setter 读取），执行后清除。
        let anchor_activated = zero_engine::anchor_activation_matches(&html, &selector);
        let anchor_thread_scripts = if anchor_activated {
            let dl = zero_engine::anchor_download_request(&html, &selector)
                .map(|value| serde_json::to_string(&value).unwrap_or_else(|_| "\"\"".into()))
                .unwrap_or_else(|| "null".into());
            let thread_script = format!(
                "(function(){{var el=document.querySelector({sel});try{{globalThis.__zwNavSourceElement=el||null;}}catch(_e){{}}try{{var d={dl};if(d===null&&el&&el.getAttribute){{var v=el.getAttribute('download');if(v!==null&&v!==undefined)d=String(v);}}globalThis.__zwNavDownloadRequest=d;}}catch(_e2){{}}}})();",
                sel = serde_json::to_string(&selector).unwrap_or_else(|_| "null".into()),
                dl = dl,
            );
            Some((
                thread_script,
                "try{globalThis.__zwNavSourceElement=null;}catch(_e){}try{globalThis.__zwNavDownloadRequest=null;}catch(_e){}".to_string(),
            ))
        } else {
            None
        };
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
                PageEffect::Navigate(intent) => {
                    // M2-S4I：锚点击的 host 导航先过页面导航面（navigate 事件 + intercept/
                    // download/cancel——`__zwNavAnchorNavigate` 返回状态决定 host 是否继续；
                    // 'host' = 页面未消化（跨文档默认）→ 既有 host 导航。表单提交不经此
                    //（anchor_activated 门——SubmitForm 产生的 Navigate 不受影响）。
                    let anchor_url = zero_engine::anchor_click_target(
                        &html,
                        &selector,
                        self.current_url.as_deref().unwrap_or("about:blank"),
                    );
                    if javascript_enabled && anchor_activated && anchor_url.as_deref() == Some(intent.url.as_str()) {
                        let dl = zero_engine::anchor_download_request(&html, &selector)
                            .map(|value| serde_json::to_string(&value).unwrap_or_else(|_| "\"\"".into()))
                            .unwrap_or_else(|| "null".into());
                        let call = format!(
                            "(function(){{var se=null;try{{se=document.querySelector({sel});}}catch(_e){{}}var r='host';try{{r=globalThis.__zwNavAnchorNavigate({url},se,{dl});}}catch(_e2){{r='host';}}try{{globalThis.__zwNavSourceElement=null;}}catch(_e3){{}}try{{globalThis.__zwNavDownloadRequest=null;}}catch(_e4){{}}return String(r);}})();",
                            sel = serde_json::to_string(&selector).unwrap_or_else(|_| "null".into()),
                            url = serde_json::to_string(&intent.url).unwrap_or_else(|_| "\"\"".into()),
                            dl = dl,
                        );
                        // 统一走 execute_dom_script（内部沙箱/外部 executor 两态；DomScriptResult.value
                        // = 脚本完成值——'canceled'/'intercepted'/'download'/'host'）。
                        match self.execute_dom_script(executor, &call) {
                            Ok(result) => {
                                let status = result.value.trim().trim_matches('"').trim_matches('"').to_string();
                                if status != "host" {
                                    // 页面已消化（canceled/intercepted/download）——跳过
                                    // host 导航；同文档提交态同步 host URL。
                                    if status == "intercepted" {
                                        self.current_url = Some(intent.url.clone());
                                    }
                                    continue;
                                }
                            }
                            Err(_) => {} // 脚本失败 → 既有 host 导航兜底
                        }
                    }
                    effects.push(PageEffect::Navigate(intent));
                }
                PageEffect::SetFragment { hash } => {
                    if let (true, Some((thread_script, cleanup))) = (javascript_enabled, anchor_thread_scripts.as_ref())
                    {
                        let _ = self.execute_dom_script(executor, thread_script)?;
                        changed |= self
                            .execute_dom_script(executor, &script_call_set_location_hash(&hash))?
                            .changed;
                        let _ = self.execute_dom_script(executor, cleanup)?;
                        continue;
                    }
                    changed |= self
                        .execute_dom_script(executor, &script_call_set_location_hash(&hash))?
                        .changed;
                    if let Some(current) = self.current_url.as_deref()
                        && let Ok(mut url) = url::Url::parse(current)
                    {
                        url.set_fragment(Some(hash.trim_start_matches('#')));
                        self.current_url = Some(url.to_string());
                    }
                }
            }
        }
        // R4008：label 激活行为在派发完成后执行（click 未取消 = 默认动作允许）。
        // 递归走 control 的完整激活管线（状态构建/click 派发/disabled 守卫/effects，
        // 同空格→button 递归先例）；控件 disabled → 递归 DisabledTarget noop 即
        // spec 语义（label 激活不越过 disabled 控件），noop_reason 不上抛。
        if !outcome.canceled
            && let Some(control_selector) = label_control_selector.as_ref()
            && let Some(control_target) = self.page_node_ref_for_selector(control_selector)
        {
            let control_result = self.dispatch_user_action_impl(
                executor,
                javascript_enabled,
                false,
                HtmlActionRequest {
                    target: control_target,
                    action: HtmlUserAction::Activate,
                    shift: request.shift,
                },
            )?;
            changed |= control_result.changed;
            effects.extend(control_result.effects);
        }
        Ok(WebViewUserActionResult {
            changed,
            canceled: outcome.canceled,
            noop_reason: None,
            effects,
            invalidation: outcome.invalidation,
        })
    }

    fn dispatch_planned_event(
        &mut self,
        executor: Option<&dyn JsExecutor>,
        event: &PlannedEvent,
    ) -> Result<(bool, bool), WebViewError> {
        let Some(selector) = self.selector_for_page_node_handle(event.target.node().get()) else {
            return Ok((false, false));
        };
        // R3254-K3 切片 B：submitter 解析进 detail（SubmitEvent.submitter——R2984 通道）。
        let submitter_sel = event
            .submitter
            .as_ref()
            .and_then(|submitter| self.selector_for_page_node_handle(submitter.node().get()));
        let detail = if event.input_type.is_some() || submitter_sel.is_some() || event.detail.is_some() {
            Some(DomEventDetail {
                data: event.data.clone(),
                input_type: event.input_type.clone(),
                submitter: submitter_sel,
                // uievents-compat M2 片 1：UIEvent.detail（click 连击计数/dblclick=2）
                // 透传——shim 泛型分支 own-property 注入。
                detail: event.detail,
                ..Default::default()
            })
        } else {
            None
        };
        let script = script_dispatch_dom_event(&selector, &event.event_type, detail.as_ref());
        let result = self.execute_dom_script(executor, &script)?;
        #[cfg(feature = "v8")]
        if executor.is_none() {
            let native =
                self.execute_dom_script(executor, &script_dispatch_native_event(&selector, &event.event_type))?;
            return Ok((result.value.trim() != "prevented", result.changed || native.changed));
        }
        Ok((result.value.trim() != "prevented", result.changed))
    }

    fn apply_planned_mutations(
        &mut self,
        executor: Option<&dyn JsExecutor>,
        mutations: &[PlannedMutation],
    ) -> Result<bool, WebViewError> {
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
                PlannedMutation::SetOptionSelected {
                    target,
                    select,
                    selected,
                    clear_others,
                } => {
                    let Some(option_selector) = self.selector_for_page_node_handle(target.node().get()) else {
                        continue;
                    };
                    let Some(select_selector) = self.selector_for_page_node_handle(select.node().get()) else {
                        continue;
                    };
                    script_set_option_selected(&option_selector, &select_selector, *selected, *clear_others)
                }
                PlannedMutation::SetOpen { target, open } => {
                    let Some(selector) = self.selector_for_page_node_handle(target.node().get()) else {
                        continue;
                    };
                    script_set_open(&selector, *open)
                }
                PlannedMutation::ResetForm { form } => {
                    let Some(selector) = self.selector_for_page_node_handle(form.node().get()) else {
                        continue;
                    };
                    script_reset_form_controls(&selector)
                }
            };
            changed |= self.execute_dom_script(executor, &script)?.changed;
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

    fn execute_dom_script(
        &mut self,
        executor: Option<&dyn JsExecutor>,
        script: &str,
    ) -> Result<DomScriptResult, WebViewError> {
        // 刀 A′（html-semantics M2 s143）：script 执行边界强制 flush apply-pending。
        // R379/pa2b 的 apply 换代通知（apply_pending_shared_mutations/apply_mutations_subset）
        // 走 execute_script 常规队列，页面回调积压时滞留队尾；用户动作脚本（优先通道或
        // 沙箱直执）可反超 → shim 侧 overlay 补偿状态（槽位/解析标记）在 host apply 落地
        // 后仍未清算的窗口被用户动作读取穿透（探针实证：BUMP 在 apply 后多次到位才清掉
        // IFRAME/FORM 槽位补偿，窗口内融合视图含已应用前缀的 overlay 副本）。此处把幂等
        // bump（微秒级、标记表空即 no-op）拼进本脚本头部，保证用户动作脚本读取前补偿
        // 状态已清算。前提守卫 = 共享队列无未应用尾（applied_mutations 游标对齐）——恰为
        // bump 清除「host apply 后节点已进快照」的成立条件；有未应用尾则跳过（维持既有
        // apply 后通知语义，不提前清导致解析补偿节点丢失）。
        // 注：WPT implicit-submission「Submit event with a submit button」的根因是查询
        // 视图已应用前缀重放双计（桥侧 MUT_APPLIED_CURSOR 修复），非本窗口——本刀收口
        // 的是同族 shim overlay 滞留面，不承担该键。
        let pending_tail_len = {
            let guard = self.shared_mutations.lock().unwrap_or_else(|e| e.into_inner());
            guard[self.applied_mutations.min(guard.len())..].len()
        };
        let script = if pending_tail_len == 0 {
            Cow::Owned(format!(
                "if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();{}",
                script
            ))
        } else {
            Cow::Borrowed(script)
        };
        if let Some(executor) = executor {
            // t2-pb1 fix#13：用户动作（键入/删除/提交等）走优先通道 + 有界等待（快照与
            // 脚本成对同通道 FIFO）。普通通道同步往返在页面回调洪水下逐个等 28-30s 臂，
            // 主循环被输入派发占死时导航 IPC 饿死。超时脚本留队列照常执行（值晚至），
            // 调用方按 Err 降级（见 apply_text_edit 等的错误分支）。
            // s143：executor 队列每批清空（下方 837-838），[0..count) 全为本脚本
            // pending → 重放起点 0；发布 0 同时清同线程可能残留的共享队列游标陈值
            //（陈值 > 0 会跳过本批前缀——见桥侧 MUT_APPLIED_CURSOR）。
            zero_engine::js_dom_bridge::publish_mut_applied_cursor(0);
            executor.set_dom_snapshot_priority(&self.cached_html, self.current_url.as_deref().unwrap_or("about:blank"));
            let mutations = executor.mutations();
            mutations.lock().unwrap_or_else(|error| error.into_inner()).clear();
            // R-baidu3：drain 代际递增——查询视图增量链只在无 drain 的批内成立。
            zero_engine::js_dom_bridge::bump_mut_drain_gen();
            let value = executor
                .execute_script_priority_bounded(&script, zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT)
                .map_err(WebViewError::Script)?;
            let recorded = mutations.lock().unwrap_or_else(|error| error.into_inner()).clone();
            let result = self.apply_dom_script_result(value, recorded)?;
            executor.set_dom_snapshot(&self.cached_html, self.current_url.as_deref().unwrap_or("about:blank"));
            // R379/pa2b 换代通知对偶（editing join dup 修复，2026-10-30）：用户动作
            // 私有批 apply 落地后须向 shim 下发换代通知——其余 apply 路径
            // （apply_mutations_subset / apply_pending_shared_mutations）尾部同契约，
            // 本路径此前只有脚本头部 pre-bump（s143 刀 A′），apply 后桶补偿仍持
            // fresh stamp（stamp === gen），动作 resolve 后的立即读命中「host 快照
            // 已含子 + overlay 副本」双计（WPT editing join 语料 sendBackspaceKey →
            // assert innerHTML 的 span 逐测试递增实证）。
            if result.changed {
                let _ = executor.execute_script_priority_bounded(
                    "if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();",
                    zero_page_runtime::USER_ACTION_SCRIPT_TIMEOUT,
                );
            }
            return Ok(result);
        }
        self.ensure_sandbox()?;
        #[cfg(feature = "v8")]
        self.install_native_dom_bindings();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
        let dom_html = Arc::new(Mutex::new(self.cached_html.clone()));
        // s168c：登记本站 Arc 为桥侧同步目标（见 WebView::sync_dom_html_bridge）。
        self.dom_html_bridge = Some(Arc::clone(&dom_html));
        let page_url = Arc::new(Mutex::new(self.current_url.clone().unwrap_or_default()));
        // s143：私有队列 [0..count) 全为本脚本 pending → 查询视图重放起点 0
        //（见桥侧 MUT_APPLIED_CURSOR）。
        zero_engine::js_dom_bridge::publish_mut_applied_cursor(0);
        {
            let sandbox = self
                .js_sandbox
                .as_mut()
                .ok_or_else(|| WebViewError::Script("no js sandbox".to_string()))?;
            register_dom_callbacks(
                &mut **sandbox,
                &mutations,
                &dom_html,
                &page_url,
                &self.canvas_registry,
                Some(&self.layout_rect_snapshot),
            );
        }
        self.ensure_js_shim()?;
        let value = self
            .js_sandbox
            .as_mut()
            .expect("js sandbox")
            .execute(&script)
            .map_err(|error| WebViewError::Script(error.to_string()))?
            .value;
        let recorded = mutations.lock().unwrap_or_else(|error| error.into_inner()).clone();
        let result = self.apply_dom_script_result(value, recorded.clone())?;
        // R4400（html-semantics M2 切片 24，2026-10-22）：plan 通道私有批**回灌共享队列
        // 全史**。共享 append-only 队列的服务对象是读回调的 latest-wins 全史扫描
        //（`sel_attr_override` 等——基座钉定在加载时 html，队列即「加载以来全部写史」）；
        // 本函数私有队列的批 apply 后即弃，共享队列历史对这批写**失明**——此后任意
        // 页面脚本 execute 的同元素读会命中更早的 stale 覆盖（WPT
        // dialog-toggle-source「open with button, close with light dismiss」实证：
        // plan click → invoker showModal 的 SetAttr(open) 落私有批，共享队列仍以
        // 前序用例的 RemoveAttr(open) 为 latest → R4200 toggle 队列 fire 时
        // newState 误算 closed）。回灌 + 游标同步推进（已应用，泵不得重放）。
        // https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element
        if !recorded.is_empty() {
            {
                let mut shared = self.shared_mutations.lock().unwrap_or_else(|error| error.into_inner());
                shared.extend(recorded.iter().cloned());
            }
            self.applied_mutations += recorded.len();
        }
        *dom_html.lock().unwrap_or_else(|error| error.into_inner()) = self.cached_html.clone();
        // R358/R3243：快照就地刷新 → bump 宿主视图缓存代际（同 count 查询防 stale 视图）。
        zero_engine::js_dom_bridge::bump_dom_view_gen();
        // R348：**恢复共享 mutation 队列绑定**——上面的 register_dom_callbacks 用本函数
        // 自建的 fresh `mutations` Arc 覆盖了沙箱级 `__zw_*` 回调的捕获（同名单重注册
        // 后者胜），此后页面脚本的所有 DOM 写入都进了这个私有队列，而
        // `apply_pending_shared_mutations`（pump/run_page_scripts 的统一 apply 入口）
        // 只消费 `shared_mutations` → 用户动作之后的页面元素**静默消失**（WPT
        // Event-dispatch-on-disabled-elements「Real clicks」input 元素 20 帧不落 doc 的
        // 根因）。用户动作脚本自身的变化已由 apply_dom_script_result 同步应用，此处把
        // 回调重绑回共享队列 + 刷新快照，恢复 pump 消费语义（镜像 run_page_scripts 的
        // 注册形态）。
        {
            let sandbox = self.js_sandbox.as_mut().expect("js sandbox");
            let shared = self.shared_mutations.clone();
            let dom_html_shared = Arc::new(std::sync::Mutex::new(self.cached_html.clone()));
            // s168c：R348 重绑后的 Arc 是最终注册（同名单后者胜）——同步目标随之切换。
            self.dom_html_bridge = Some(Arc::clone(&dom_html_shared));
            let page_url = Arc::new(std::sync::Mutex::new(self.current_url.clone().unwrap_or_default()));
            // s143：共享队列全史注册 → 发布已应用前缀作查询视图重放起点（R4400 回灌
            // 已含在本游标内——见桥侧 MUT_APPLIED_CURSOR；无此发布则异步续延查询把
            // 已应用进基座的 InsertAdjacentHtml 再重放一次，WPT implicit-submission
            // 「Submit event with a submit button」实证双计）。
            zero_engine::js_dom_bridge::publish_mut_applied_cursor(self.applied_mutations);
            register_dom_callbacks(
                &mut **sandbox,
                &shared,
                &dom_html_shared,
                &page_url,
                &self.canvas_registry,
                Some(&self.layout_rect_snapshot),
            );
            zero_engine::js_dom_bridge::publish_live_query_doc(self.pipeline.cached_doc_shared());
        }
        // 换代通知对偶（sandbox 直执分支，契约同上方 executor 分支注记）——回灌/
        // 重注册完成后下发，bump 清算窗口覆盖本批 apply 的全部桶补偿。
        if result.changed {
            let _ = self.js_sandbox.as_mut().expect("js sandbox").execute(
                "if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();",
            );
        }
        Ok(result)
    }

    fn apply_dom_script_result(
        &mut self,
        value: String,
        recorded: Vec<DomMutation>,
    ) -> Result<DomScriptResult, WebViewError> {
        if recorded.is_empty() {
            #[cfg(feature = "v8")]
            self.sync_render_after_native_dom();
            return Ok(DomScriptResult { value, changed: false });
        }
        let (result, html_snapshot, handle_selectors) = self
            .pipeline
            .render_with_dom_mutations(&recorded, &self.cached_css)
            .map_err(|error| WebViewError::Script(format!("apply mutations: {error}")))?;
        if let Some(html) = html_snapshot {
            self.cached_html = html;
        }
        // R379/pa2b（editing-compat D2 簇补账）：第 4 应用路径（execute_dom_script /
        // 用户动作脚本尾批）此前 apply+换基座后**漏发 shim 换代通知**——base 已含
        // apply 结果而 `_zwApplyGeneration` 未 bump，R380/R381 桶级代际门的前提
        // （「base 含 apply 结果 ⇒ gen 已换代、gate 关」）在本路径被破坏：本脚本
        // 尾批入桶的条目（如 CE join 壳 span）stamp === gen 判「同 turn」继续并入
        // 融合视图，而 base 重读已含同内容 → 双计（WPT editing join 语料探针实证
        // div1 双 span）。与其余 3 条应用路径（webview.rs 两处 + apply_pending_shared_
        // mutations）对齐，apply 后统一换代。失败静默（钩子缺失 = 旧 shim 版本）。
        // https://dom.spec.whatwg.org/#concept-node-tree
        let _ = self.execute_script(
            "if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();",
        );
        // slice33 缺陷轮 S-1：第 4 应用路径（execute_dom_script）补 R100 失效面清理——
        // 结构性 Remove/RemoveHandle 后清 selector_handle_map（sel→handle 残账，与其余
        // 3 条应用路径同契约）。
        self.evict_removed_identities(&recorded, &handle_selectors);
        if let Some(result) = &result {
            self.last_render = Some(render_result_to_webview(result));
        }
        Ok(DomScriptResult { value, changed: true })
    }
}
