//! HTML 用户动作的纯规划、提交与回滚核心。

use crate::PageNodeRef;

/// 用户可触发的 HTML 默认动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlUserAction {
    /// 在当前文本选区插入文本。
    InsertText {
        /// 待插入文本。
        text: String,
    },
    /// 删除当前选区或 caret 前一个 Unicode scalar。
    DeleteBackward,
    /// 顺序移动焦点。
    MoveFocus {
        /// `true` 表示向前，`false` 表示反向。
        forward: bool,
    },
    /// 激活目标控件。
    Activate,
    /// 重置目标表单。
    Reset,
    /// 提交目标表单。
    Submit,
}

/// 待规划的 HTML 用户动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlActionRequest {
    /// 稳定目标身份。
    pub target: PageNodeRef,
    /// 用户动作。
    pub action: HtmlUserAction,
    /// 平台 Shift 修饰键状态。
    pub shift: bool,
}

/// 文本控件规划所需的 live 状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextActionState {
    /// live value。
    pub value: String,
    /// UTF-16 选区起点。
    pub selection_start: usize,
    /// UTF-16 选区终点。
    pub selection_end: usize,
    /// 控件是否为 readonly。
    pub read_only: bool,
    /// 用户输入允许的最大 UTF-16 code unit 数；`None` 表示无限制。
    pub max_length: Option<usize>,
}

/// radio 激活规划所需的组状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioActionState {
    /// 目标当前是否已选中。
    pub checked: bool,
    /// 同组激活前的选中节点。
    pub previous_checked: Option<PageNodeRef>,
}

/// option 激活规划所需的 select 状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionActionState {
    /// owning select。
    pub select: PageNodeRef,
    /// 目标 option 当前是否选中。
    pub selected: bool,
    /// owning select 是否为 multiple。
    pub multiple: bool,
    /// 单选 select 激活前选中的 option。
    pub previous_selected: Option<PageNodeRef>,
}

/// summary 激活规划所需的 details 状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryActionState {
    /// owning details。
    pub details: PageNodeRef,
    /// 激活前 open 状态。
    pub open: bool,
}

/// GET/POST 表单导航意图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormNavigationIntent {
    /// 规范化目标 URL。
    pub url: String,
    /// HTTP method。
    pub method: String,
    /// POST body；GET 为 `None`。
    pub body: Option<String>,
}

/// 宿主提供给动作规划器的目标快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTargetState {
    /// 目标无法参与动作，并携带显式原因。
    Unavailable(ActionNoopReason),
    /// 文本控件。
    Text(TextActionState),
    /// checkbox checkedness。
    Checkbox {
        /// 激活前 checkedness。
        checked: bool,
    },
    /// radio 组 checkedness。
    Radio(RadioActionState),
    /// option 与 owning select 状态。
    Option(OptionActionState),
    /// 可导航链接的规范化 intent。
    Navigate {
        /// 未取消时产生的导航。
        intent: FormNavigationIntent,
    },
    /// 同文档 fragment 导航。
    Fragment {
        /// 目标 hash（含 `#`）。
        hash: String,
    },
    /// details 首个 summary 的激活状态。
    Summary(SummaryActionState),
    /// 顺序焦点移动的计算结果。
    Focus {
        /// 下一个 focus owner；`None` 表示清除焦点。
        next: Option<PageNodeRef>,
    },
    /// reset 后应提交的默认状态 mutation。
    Reset {
        /// form owner。
        form: PageNodeRef,
    },
    /// submit 的 form owner 与 submitter。
    Submit {
        /// form owner。
        form: PageNodeRef,
        /// 触发提交的 submitter；隐式提交为 `None`。
        submitter: Option<PageNodeRef>,
    },
    /// 无特定激活语义的普通元素（contenteditable 宿主、通用可点击元素）——激活仅派发
    /// click 事件，无默认动作（js-dom R142：WPT no-focus-events 的 span[contenteditable]
    /// 点击；旧分类对非表单/非链接/非 summary/非 option 一律 NotApplicable 使合成指针
    /// 点击整簇挂）。
    Generic,
}

/// identity-based 状态变更；宿主负责把 node ref 解析到 live DOM。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedMutation {
    /// 设置文本 live value 与 UTF-16 selection。
    SetText {
        /// 目标文本控件。
        target: PageNodeRef,
        /// 新 live value。
        value: String,
        /// 新选区起点。
        selection_start: usize,
        /// 新选区终点。
        selection_end: usize,
    },
    /// 设置 checkbox/radio checkedness。
    SetChecked {
        /// 目标控件。
        target: PageNodeRef,
        /// 新 checkedness。
        checked: bool,
    },
    /// 设置 option selectedness，并可清除同 select 其他 option。
    SetOptionSelected {
        /// 目标 option。
        target: PageNodeRef,
        /// owning select。
        select: PageNodeRef,
        /// 新 selectedness。
        selected: bool,
        /// 是否先清除 owning select 的其他 option。
        clear_others: bool,
    },
    /// 设置 details/dialog open 状态。
    SetOpen {
        /// 目标 details/dialog。
        target: PageNodeRef,
        /// 新 open 状态。
        open: bool,
    },
    /// 恢复 form 内所有 resettable controls 的默认状态。
    ResetForm {
        /// form owner。
        form: PageNodeRef,
    },
}

/// 规划后的 DOM 事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedEvent {
    /// 事件目标。
    pub target: PageNodeRef,
    /// 事件类型。
    pub event_type: String,
    /// 事件是否可取消。
    pub cancelable: bool,
    /// InputEvent inputType。
    pub input_type: Option<String>,
    /// InputEvent data。
    pub data: Option<String>,
}

impl PlannedEvent {
    fn simple(target: PageNodeRef, event_type: &str, cancelable: bool) -> Self {
        Self {
            target,
            event_type: event_type.to_string(),
            cancelable,
            input_type: None,
            data: None,
        }
    }

    fn input(target: PageNodeRef, event_type: &str, cancelable: bool, input_type: &str, data: Option<String>) -> Self {
        Self {
            target,
            event_type: event_type.to_string(),
            cancelable,
            input_type: Some(input_type.to_string()),
            data,
        }
    }
}

/// 宿主副作用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageEffect {
    /// 更新 focus owner。
    Focus(Option<PageNodeRef>),
    /// 执行表单导航。
    Navigate(FormNavigationIntent),
    /// 更新同文档 fragment，不创建新 Document。
    SetFragment {
        /// 目标 hash（含 `#`）。
        hash: String,
    },
    /// 在 submit listener 完成后构造 entry list 并导航。
    SubmitForm {
        /// form owner。
        form: PageNodeRef,
        /// submitter；隐式提交为 `None`。
        submitter: Option<PageNodeRef>,
    },
}

/// 动作产生的 typed frame invalidation。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationKind {
    /// 无需发布帧。
    None,
    /// live state 变化，需要 paint/hit-test/publish。
    Paint,
    /// 导航将替换文档，丢弃当前帧事务。
    Navigation,
}

/// 动作无法规划的显式原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionNoopReason {
    /// 当前 live Document 中不存在该目标。
    MissingTarget,
    /// target navigation epoch 或 document generation 已过期。
    StaleTarget,
    /// 目标或其 HTML owner 状态为 disabled。
    DisabledTarget,
    /// 文本控件为 readonly。
    ReadOnlyTarget,
    /// 文本控件已无 maxlength 容量。
    MaxLengthReached,
    /// 动作不适用于目标状态。
    NotApplicable,
    /// 已选 radio 重复激活。
    AlreadySelected,
    /// caret 在文本起点且选区为空。
    NothingToDelete,
}

/// 可在三宿主重放的动作计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlActionPlan {
    /// 稳定目标。
    pub target: PageNodeRef,
    /// cancelable event **前**派发的不可取消事件（js-dom R144：指针激活序列的
    /// mousedown/mouseup 先于 click——listener 内的 DOM 变更[伪元素移除/节点移动]
    /// 须发生在 click 派发前，click 在新状态上派发）。
    pub pre_events: Vec<PlannedEvent>,
    /// cancelable event 前应用的临时状态。
    pub prepare: Vec<PlannedMutation>,
    /// 宿主应派发的 cancelable event。
    pub cancelable_event: Option<PlannedEvent>,
    /// event 被取消时应用的恢复状态。
    pub rollback: Vec<PlannedMutation>,
    /// event 未取消时应用的最终状态。
    pub commit: Vec<PlannedMutation>,
    /// commit 后派发的不可取消事件。
    pub followup_events: Vec<PlannedEvent>,
    /// commit 后执行的宿主副作用。
    pub effects: Vec<PageEffect>,
    /// 最终帧失效类型。
    pub invalidation: InvalidationKind,
}

/// cancelable event 的宿主派发结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventDispatchResult {
    /// `preventDefault()` 未被调用。
    pub default_allowed: bool,
    /// listener 是否产生额外 DOM 变更。
    pub html_changed: bool,
}

/// prepare 后根据派发结果得到的最终动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlActionOutcome {
    /// 应应用的 rollback 或 commit mutations。
    pub mutations: Vec<PlannedMutation>,
    /// 应派发的 follow-up events。
    pub followup_events: Vec<PlannedEvent>,
    /// 应执行的宿主副作用。
    pub effects: Vec<PageEffect>,
    /// 最终帧失效类型。
    pub invalidation: InvalidationKind,
    /// 事件是否取消了默认动作。
    pub canceled: bool,
    /// listener 是否产生额外 DOM 变更。
    pub html_changed: bool,
}

/// 构造动作计划，并拒绝 stale identity 或动作/状态不匹配。
pub fn plan_html_action(
    request: &HtmlActionRequest,
    current_navigation_epoch: u64,
    current_document_generation: u64,
    state: &ActionTargetState,
) -> Result<HtmlActionPlan, ActionNoopReason> {
    if !request
        .target
        .is_current(current_navigation_epoch, current_document_generation)
    {
        return Err(ActionNoopReason::StaleTarget);
    }
    if let ActionTargetState::Unavailable(reason) = state {
        return Err(*reason);
    }
    match (&request.action, state) {
        (HtmlUserAction::InsertText { text }, ActionTargetState::Text(state)) => {
            plan_text_insert(request.target, state, text)
        }
        (HtmlUserAction::DeleteBackward, ActionTargetState::Text(state)) => plan_text_delete(request.target, state),
        (HtmlUserAction::Activate, ActionTargetState::Checkbox { checked }) => {
            Ok(plan_checkbox(request.target, *checked))
        }
        (HtmlUserAction::Activate, ActionTargetState::Radio(state)) => plan_radio(request.target, state),
        (HtmlUserAction::Activate, ActionTargetState::Option(state)) => plan_option(request.target, state),
        (HtmlUserAction::Activate, ActionTargetState::Navigate { intent }) => {
            Ok(plan_navigation(request.target, intent.clone()))
        }
        (HtmlUserAction::Activate, ActionTargetState::Fragment { hash }) => {
            Ok(plan_fragment(request.target, hash.clone()))
        }
        (HtmlUserAction::Activate, ActionTargetState::Summary(state)) => Ok(plan_summary(request.target, state)),
        (HtmlUserAction::MoveFocus { .. }, ActionTargetState::Focus { next }) => Ok(plan_focus(request.target, *next)),
        // js-dom R142：普通元素激活 = 纯 click 事件（无默认动作——cancelable_event 的
        // default_allowed 不产生 mutation/effect）。R144：pre_events 补指针激活序列
        // mousedown/mouseup（spec UI Events 指针事件序——真实浏览器 click 前有
        // mousedown/mouseup；listener 内 DOM 变更[伪元素移除/节点移入他文档]发生在
        // click 派发前，click 仍照常派发。WPT click-on-absolute-pseudo /
        // focus-event-document-move）。
        (HtmlUserAction::Activate, ActionTargetState::Generic) => Ok(HtmlActionPlan {
            pre_events: vec![
                PlannedEvent::simple(request.target, "mousedown", true),
                PlannedEvent::simple(request.target, "mouseup", true),
            ],
            target: request.target,
            prepare: vec![],
            cancelable_event: Some(PlannedEvent::simple(request.target, "click", true)),
            rollback: vec![],
            commit: vec![],
            followup_events: vec![],
            effects: vec![],
            invalidation: InvalidationKind::None,
        }),
        (HtmlUserAction::Reset, ActionTargetState::Reset { form }) => Ok(plan_reset(request.target, *form)),
        (HtmlUserAction::Submit, ActionTargetState::Submit { form, submitter }) => {
            Ok(plan_submit(request.target, *form, *submitter))
        }
        _ => Err(ActionNoopReason::NotApplicable),
    }
}

/// 将 cancelable event 结果归约为 rollback 或 commit outcome。
pub fn resolve_html_action(plan: HtmlActionPlan, dispatch: EventDispatchResult) -> HtmlActionOutcome {
    if !dispatch.default_allowed {
        return HtmlActionOutcome {
            mutations: plan.rollback,
            followup_events: vec![],
            effects: vec![],
            invalidation: if plan.prepare.is_empty() {
                InvalidationKind::None
            } else {
                plan.invalidation
            },
            canceled: true,
            html_changed: dispatch.html_changed,
        };
    }
    HtmlActionOutcome {
        mutations: plan.commit,
        followup_events: plan.followup_events,
        effects: plan.effects,
        invalidation: plan.invalidation,
        canceled: false,
        html_changed: dispatch.html_changed,
    }
}

fn plan_checkbox(target: PageNodeRef, checked: bool) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![PlannedMutation::SetChecked {
            target,
            checked: !checked,
        }],
        cancelable_event: Some(PlannedEvent::simple(target, "click", true)),
        rollback: vec![PlannedMutation::SetChecked { target, checked }],
        commit: vec![],
        followup_events: checkedness_events(target),
        effects: vec![],
        invalidation: InvalidationKind::Paint,
    }
}

fn plan_radio(target: PageNodeRef, state: &RadioActionState) -> Result<HtmlActionPlan, ActionNoopReason> {
    if state.checked {
        return Err(ActionNoopReason::AlreadySelected);
    }
    let mut prepare = vec![PlannedMutation::SetChecked { target, checked: true }];
    let mut rollback = vec![PlannedMutation::SetChecked { target, checked: false }];
    if let Some(previous) = state.previous_checked {
        prepare.push(PlannedMutation::SetChecked {
            target: previous,
            checked: false,
        });
        rollback.push(PlannedMutation::SetChecked {
            target: previous,
            checked: true,
        });
    }
    Ok(HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare,
        cancelable_event: Some(PlannedEvent::simple(target, "click", true)),
        rollback,
        commit: vec![],
        followup_events: checkedness_events(target),
        effects: vec![],
        invalidation: InvalidationKind::Paint,
    })
}

fn plan_option(target: PageNodeRef, state: &OptionActionState) -> Result<HtmlActionPlan, ActionNoopReason> {
    if !state.multiple && state.selected {
        return Err(ActionNoopReason::AlreadySelected);
    }
    let selected = if state.multiple { !state.selected } else { true };
    let rollback = if state.multiple {
        vec![PlannedMutation::SetOptionSelected {
            target,
            select: state.select,
            selected: state.selected,
            clear_others: false,
        }]
    } else if let Some(previous) = state.previous_selected {
        vec![PlannedMutation::SetOptionSelected {
            target: previous,
            select: state.select,
            selected: true,
            clear_others: true,
        }]
    } else {
        vec![PlannedMutation::SetOptionSelected {
            target,
            select: state.select,
            selected: false,
            clear_others: true,
        }]
    };
    Ok(HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![PlannedMutation::SetOptionSelected {
            target,
            select: state.select,
            selected,
            clear_others: !state.multiple,
        }],
        cancelable_event: Some(PlannedEvent::simple(target, "click", true)),
        rollback,
        commit: vec![],
        followup_events: checkedness_events(state.select),
        effects: vec![],
        invalidation: InvalidationKind::Paint,
    })
}

fn plan_navigation(target: PageNodeRef, intent: FormNavigationIntent) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: Some(PlannedEvent::simple(target, "click", true)),
        rollback: vec![],
        commit: vec![],
        followup_events: vec![],
        effects: vec![PageEffect::Navigate(intent)],
        invalidation: InvalidationKind::Navigation,
    }
}

fn plan_fragment(target: PageNodeRef, hash: String) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: Some(PlannedEvent::simple(target, "click", true)),
        rollback: vec![],
        commit: vec![],
        followup_events: vec![],
        effects: vec![PageEffect::SetFragment { hash }],
        invalidation: InvalidationKind::None,
    }
}

fn plan_summary(target: PageNodeRef, state: &SummaryActionState) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: Some(PlannedEvent::simple(target, "click", true)),
        rollback: vec![],
        commit: vec![PlannedMutation::SetOpen {
            target: state.details,
            open: !state.open,
        }],
        followup_events: vec![PlannedEvent::simple(state.details, "toggle", false)],
        effects: vec![],
        invalidation: InvalidationKind::Paint,
    }
}

fn checkedness_events(target: PageNodeRef) -> Vec<PlannedEvent> {
    vec![
        PlannedEvent::simple(target, "input", false),
        PlannedEvent::simple(target, "change", false),
    ]
}

fn plan_text_insert(
    target: PageNodeRef,
    state: &TextActionState,
    text: &str,
) -> Result<HtmlActionPlan, ActionNoopReason> {
    if state.read_only {
        return Err(ActionNoopReason::ReadOnlyTarget);
    }
    let (start, end) = normalized_selection(state);
    let text = if let Some(max_length) = state.max_length {
        let retained_length = state
            .value
            .encode_utf16()
            .count()
            .saturating_sub(end.saturating_sub(start));
        truncate_to_utf16(text, max_length.saturating_sub(retained_length))
    } else {
        text
    };
    if text.is_empty() {
        return Err(ActionNoopReason::MaxLengthReached);
    }
    let start_byte = byte_index_at_utf16(&state.value, start);
    let end_byte = byte_index_at_utf16(&state.value, end);
    let mut value = state.value.clone();
    value.replace_range(start_byte..end_byte, text);
    let caret = start + text.encode_utf16().count();
    Ok(text_plan(target, value, caret, "insertText", Some(text.to_string())))
}

fn plan_text_delete(target: PageNodeRef, state: &TextActionState) -> Result<HtmlActionPlan, ActionNoopReason> {
    if state.read_only {
        return Err(ActionNoopReason::ReadOnlyTarget);
    }
    let (start, end) = normalized_selection(state);
    if start == end && start == 0 {
        return Err(ActionNoopReason::NothingToDelete);
    }
    let delete_start = if start == end {
        let byte = byte_index_at_utf16(&state.value, start);
        let previous = state.value[..byte]
            .chars()
            .next_back()
            .ok_or(ActionNoopReason::NothingToDelete)?;
        start.saturating_sub(previous.len_utf16())
    } else {
        start
    };
    let mut value = state.value.clone();
    value.replace_range(
        byte_index_at_utf16(&state.value, delete_start)..byte_index_at_utf16(&state.value, end),
        "",
    );
    Ok(text_plan(target, value, delete_start, "deleteContentBackward", None))
}

fn text_plan(
    target: PageNodeRef,
    value: String,
    caret: usize,
    input_type: &str,
    data: Option<String>,
) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: Some(PlannedEvent::input(
            target,
            "beforeinput",
            true,
            input_type,
            data.clone(),
        )),
        rollback: vec![],
        commit: vec![PlannedMutation::SetText {
            target,
            value,
            selection_start: caret,
            selection_end: caret,
        }],
        followup_events: vec![PlannedEvent::input(target, "input", false, input_type, data)],
        effects: vec![],
        invalidation: InvalidationKind::Paint,
    }
}

fn plan_focus(target: PageNodeRef, next: Option<PageNodeRef>) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: None,
        rollback: vec![],
        commit: vec![],
        followup_events: vec![],
        effects: vec![PageEffect::Focus(next)],
        invalidation: InvalidationKind::None,
    }
}

fn plan_reset(target: PageNodeRef, form: PageNodeRef) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: Some(PlannedEvent::simple(form, "reset", true)),
        rollback: vec![],
        commit: vec![PlannedMutation::ResetForm { form }],
        followup_events: vec![],
        effects: vec![],
        invalidation: InvalidationKind::Paint,
    }
}

fn plan_submit(target: PageNodeRef, form: PageNodeRef, submitter: Option<PageNodeRef>) -> HtmlActionPlan {
    HtmlActionPlan {
        pre_events: vec![],
        target,
        prepare: vec![],
        cancelable_event: Some(PlannedEvent::simple(form, "submit", true)),
        rollback: vec![],
        commit: vec![],
        followup_events: vec![],
        effects: vec![PageEffect::SubmitForm { form, submitter }],
        invalidation: InvalidationKind::Navigation,
    }
}

fn normalized_selection(state: &TextActionState) -> (usize, usize) {
    let len = state.value.encode_utf16().count();
    let start = state.selection_start.min(state.selection_end).min(len);
    let end = state.selection_start.max(state.selection_end).min(len);
    (start, end)
}

fn byte_index_at_utf16(value: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (byte, ch) in value.char_indices() {
        if utf16 >= offset {
            return byte;
        }
        utf16 += ch.len_utf16();
    }
    value.len()
}

fn truncate_to_utf16(value: &str, max_length: usize) -> &str {
    let mut utf16 = 0;
    for (byte, ch) in value.char_indices() {
        let next = utf16 + ch.len_utf16();
        if next > max_length {
            return &value[..byte];
        }
        utf16 = next;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PageNodeHandle;

    fn node(handle: u64) -> PageNodeRef {
        PageNodeRef::new(4, 9, PageNodeHandle::new(handle))
    }

    fn request(target: PageNodeRef, action: HtmlUserAction) -> HtmlActionRequest {
        HtmlActionRequest {
            target,
            action,
            shift: false,
        }
    }

    #[test]
    fn checkbox_plan_rolls_back_when_click_is_prevented() {
        let plan = plan_html_action(
            &request(node(1), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Checkbox { checked: false },
        )
        .unwrap();
        assert_eq!(
            plan.prepare,
            [PlannedMutation::SetChecked {
                target: node(1),
                checked: true
            }]
        );

        let outcome = resolve_html_action(
            plan,
            EventDispatchResult {
                default_allowed: false,
                html_changed: false,
            },
        );
        assert!(outcome.canceled);
        assert_eq!(
            outcome.mutations,
            [PlannedMutation::SetChecked {
                target: node(1),
                checked: false
            }]
        );
        assert!(outcome.followup_events.is_empty());
    }

    #[test]
    fn checked_radio_activation_is_explicit_noop() {
        assert_eq!(
            plan_html_action(
                &request(node(1), HtmlUserAction::Activate),
                4,
                9,
                &ActionTargetState::Radio(RadioActionState {
                    checked: true,
                    previous_checked: Some(node(1)),
                }),
            ),
            Err(ActionNoopReason::AlreadySelected)
        );
    }

    #[test]
    fn radio_rollback_restores_previous_group_member() {
        let plan = plan_html_action(
            &request(node(2), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Radio(RadioActionState {
                checked: false,
                previous_checked: Some(node(1)),
            }),
        )
        .unwrap();
        let outcome = resolve_html_action(
            plan,
            EventDispatchResult {
                default_allowed: false,
                html_changed: true,
            },
        );
        assert_eq!(
            outcome.mutations,
            [
                PlannedMutation::SetChecked {
                    target: node(2),
                    checked: false
                },
                PlannedMutation::SetChecked {
                    target: node(1),
                    checked: true
                }
            ]
        );
        assert!(outcome.html_changed);
    }

    #[test]
    fn text_insert_replaces_utf16_selection_and_emits_input() {
        let plan = plan_html_action(
            &request(
                node(1),
                HtmlUserAction::InsertText {
                    text: "中".to_string()
                },
            ),
            4,
            9,
            &ActionTargetState::Text(TextActionState {
                value: "A😀B".to_string(),
                selection_start: 1,
                selection_end: 3,
                read_only: false,
                max_length: None,
            }),
        )
        .unwrap();
        assert_eq!(plan.cancelable_event.as_ref().unwrap().event_type, "beforeinput");
        assert_eq!(
            plan.commit,
            [PlannedMutation::SetText {
                target: node(1),
                value: "A中B".to_string(),
                selection_start: 2,
                selection_end: 2,
            }]
        );
        assert_eq!(plan.followup_events[0].event_type, "input");
        assert!(!plan.followup_events[0].cancelable);
    }

    #[test]
    fn readonly_text_actions_are_explicit_noops() {
        let state = ActionTargetState::Text(TextActionState {
            value: "value".to_string(),
            selection_start: 5,
            selection_end: 5,
            read_only: true,
            max_length: None,
        });
        assert_eq!(
            plan_html_action(
                &request(node(1), HtmlUserAction::InsertText { text: "x".to_string() },),
                4,
                9,
                &state,
            ),
            Err(ActionNoopReason::ReadOnlyTarget)
        );
        assert_eq!(
            plan_html_action(&request(node(1), HtmlUserAction::DeleteBackward), 4, 9, &state),
            Err(ActionNoopReason::ReadOnlyTarget)
        );
    }

    #[test]
    fn maxlength_truncates_insert_at_utf16_boundaries() {
        let plan = plan_html_action(
            &request(
                node(1),
                HtmlUserAction::InsertText {
                    text: "😀B".to_string(),
                },
            ),
            4,
            9,
            &ActionTargetState::Text(TextActionState {
                value: "Axx".to_string(),
                selection_start: 1,
                selection_end: 3,
                read_only: false,
                max_length: Some(3),
            }),
        )
        .unwrap();
        assert_eq!(
            plan.commit,
            [PlannedMutation::SetText {
                target: node(1),
                value: "A😀".to_string(),
                selection_start: 3,
                selection_end: 3,
            }]
        );
        assert_eq!(plan.cancelable_event.unwrap().data.as_deref(), Some("😀"));

        assert_eq!(
            plan_html_action(
                &request(node(1), HtmlUserAction::InsertText { text: "x".to_string() },),
                4,
                9,
                &ActionTargetState::Text(TextActionState {
                    value: "full".to_string(),
                    selection_start: 4,
                    selection_end: 4,
                    read_only: false,
                    max_length: Some(4),
                }),
            ),
            Err(ActionNoopReason::MaxLengthReached)
        );
    }

    #[test]
    fn delete_at_start_and_stale_target_are_explicit() {
        assert_eq!(
            plan_html_action(
                &request(node(1), HtmlUserAction::DeleteBackward),
                4,
                9,
                &ActionTargetState::Text(TextActionState {
                    value: "A".to_string(),
                    selection_start: 0,
                    selection_end: 0,
                    read_only: false,
                    max_length: None,
                }),
            ),
            Err(ActionNoopReason::NothingToDelete)
        );
        assert_eq!(
            plan_html_action(
                &request(node(1), HtmlUserAction::Activate),
                4,
                10,
                &ActionTargetState::Checkbox { checked: false },
            ),
            Err(ActionNoopReason::StaleTarget)
        );
        assert_eq!(
            plan_html_action(
                &request(node(1), HtmlUserAction::Activate),
                4,
                9,
                &ActionTargetState::Unavailable(ActionNoopReason::DisabledTarget),
            ),
            Err(ActionNoopReason::DisabledTarget)
        );
    }

    #[test]
    fn prevented_submit_has_no_navigation_effect() {
        let plan = plan_html_action(
            &request(node(1), HtmlUserAction::Submit),
            4,
            9,
            &ActionTargetState::Submit {
                form: node(2),
                submitter: Some(node(1)),
            },
        )
        .unwrap();
        let canceled = resolve_html_action(
            plan.clone(),
            EventDispatchResult {
                default_allowed: false,
                html_changed: false,
            },
        );
        assert!(canceled.effects.is_empty());
        let allowed = resolve_html_action(
            plan,
            EventDispatchResult {
                default_allowed: true,
                html_changed: false,
            },
        );
        assert_eq!(
            allowed.effects,
            [PageEffect::SubmitForm {
                form: node(2),
                submitter: Some(node(1))
            }]
        );
    }

    #[test]
    fn anchor_navigation_emits_one_effect_only_when_click_is_allowed() {
        let intent = FormNavigationIntent {
            url: "https://zero.test/next".to_string(),
            method: "GET".to_string(),
            body: None,
        };
        let plan = plan_html_action(
            &request(node(1), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Navigate { intent: intent.clone() },
        )
        .unwrap();
        assert_eq!(plan.cancelable_event.as_ref().unwrap().event_type, "click");
        assert_eq!(
            resolve_html_action(
                plan.clone(),
                EventDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                },
            )
            .effects,
            [PageEffect::Navigate(intent)]
        );
        assert!(
            resolve_html_action(
                plan,
                EventDispatchResult {
                    default_allowed: false,
                    html_changed: false,
                },
            )
            .effects
            .is_empty()
        );

        let fragment = plan_html_action(
            &request(node(1), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Fragment {
                hash: "#section".to_string(),
            },
        )
        .unwrap();
        assert_eq!(
            resolve_html_action(
                fragment,
                EventDispatchResult {
                    default_allowed: true,
                    html_changed: false,
                },
            )
            .effects,
            [PageEffect::SetFragment {
                hash: "#section".to_string()
            }]
        );
    }

    #[test]
    fn option_activation_rolls_back_single_and_multiple_selection() {
        let single = plan_html_action(
            &request(node(2), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Option(OptionActionState {
                select: node(10),
                selected: false,
                multiple: false,
                previous_selected: Some(node(1)),
            }),
        )
        .unwrap();
        assert_eq!(
            single.prepare,
            [PlannedMutation::SetOptionSelected {
                target: node(2),
                select: node(10),
                selected: true,
                clear_others: true,
            }]
        );
        assert_eq!(
            resolve_html_action(
                single,
                EventDispatchResult {
                    default_allowed: false,
                    html_changed: false,
                },
            )
            .mutations,
            [PlannedMutation::SetOptionSelected {
                target: node(1),
                select: node(10),
                selected: true,
                clear_others: true,
            }]
        );

        let multiple = plan_html_action(
            &request(node(3), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Option(OptionActionState {
                select: node(11),
                selected: true,
                multiple: true,
                previous_selected: None,
            }),
        )
        .unwrap();
        assert_eq!(
            multiple.prepare,
            [PlannedMutation::SetOptionSelected {
                target: node(3),
                select: node(11),
                selected: false,
                clear_others: false,
            }]
        );
        assert_eq!(multiple.followup_events[0].target, node(11));
    }

    #[test]
    fn generic_activation_dispatches_click_without_default() {
        // js-dom R142：普通元素（contenteditable 宿主等）激活 = 纯 click 事件派发，无
        // 默认动作（mutation/effect 全空；preventDefault 只影响 default_allowed 语义，
        // 不产生 rollback）。
        let plan = plan_html_action(
            &request(node(1), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Generic,
        )
        .unwrap();
        assert!(plan.prepare.is_empty());
        assert!(plan.commit.is_empty());
        assert!(plan.effects.is_empty());
        // R144：指针激活序列——pre_events 的 mousedown/mouseup 先于 click。
        assert_eq!(plan.pre_events.len(), 2);
        assert_eq!(plan.pre_events[0].event_type, "mousedown");
        assert_eq!(plan.pre_events[1].event_type, "mouseup");
        let event = plan.cancelable_event.as_ref().unwrap();
        assert_eq!(event.target, node(1));
        assert_eq!(event.event_type, "click");
        let outcome = resolve_html_action(
            plan,
            EventDispatchResult {
                default_allowed: false,
                html_changed: false,
            },
        );
        assert!(outcome.mutations.is_empty());
        assert!(outcome.effects.is_empty());
    }

    #[test]
    fn summary_toggles_after_uncanceled_click() {
        let plan = plan_html_action(
            &request(node(1), HtmlUserAction::Activate),
            4,
            9,
            &ActionTargetState::Summary(SummaryActionState {
                details: node(2),
                open: false,
            }),
        )
        .unwrap();
        assert!(plan.prepare.is_empty());
        assert_eq!(
            plan.commit,
            [PlannedMutation::SetOpen {
                target: node(2),
                open: true,
            }]
        );
        assert_eq!(plan.followup_events[0].target, node(2));
        assert!(
            resolve_html_action(
                plan,
                EventDispatchResult {
                    default_allowed: false,
                    html_changed: false,
                },
            )
            .mutations
            .is_empty()
        );
    }

    #[test]
    fn reset_dispatches_to_form_and_commits_only_when_allowed() {
        let plan = plan_html_action(
            &request(node(1), HtmlUserAction::Reset),
            4,
            9,
            &ActionTargetState::Reset { form: node(2) },
        )
        .unwrap();
        assert_eq!(plan.cancelable_event.as_ref().unwrap().target, node(2));
        let canceled = resolve_html_action(
            plan.clone(),
            EventDispatchResult {
                default_allowed: false,
                html_changed: false,
            },
        );
        assert!(canceled.mutations.is_empty());
        let allowed = resolve_html_action(
            plan,
            EventDispatchResult {
                default_allowed: true,
                html_changed: false,
            },
        );
        assert_eq!(allowed.mutations, [PlannedMutation::ResetForm { form: node(2) }]);
    }
}
