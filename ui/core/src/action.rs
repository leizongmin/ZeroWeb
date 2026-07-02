//! Action / Message 接口（spec IF-002）。
//!
//! 控件只发出 `Action`，由应用层 reducer 更新业务状态并触发失效（spec FR-003 单向数据流）。
//! 未注册 Action 必须返回诊断，不得静默执行。

use crate::binding::Value;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// 稳定的 action 标识（点分命名，如 `browser.go_back`、`form.submit`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActionId(pub CompactString);

impl ActionId {
    pub fn new(name: &str) -> ActionId {
        ActionId(CompactString::new(name))
    }
}

/// 事件处理结果（spec IF-002）。
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    /// 未处理，继续冒泡。
    Ignored,
    /// 已消费，停止冒泡。
    Consumed,
    /// 发出无 payload 的 action。
    Emit(ActionId),
    /// 发出带 payload 的 action。
    EmitWithPayload(ActionId, ActionPayload),
}

/// Action 携带的 payload（受控值类型，禁止任意脚本对象）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionPayload {
    /// 无 payload（与 `Emit` 等价的显式形式）。
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    /// 结构化 payload（用于 command palette query、表单提交等）。
    Value(Value),
}

/// `WidgetSpec` 上声明的 action 绑定（spec IF-005 `ActionBinding`）。
///
/// 把某 UI 触发点（如 click）映射到一个 `ActionId`，可附带静态 payload。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionBinding {
    /// 触发名（约定：`click` / `submit` / `change` / `select` 等）。
    pub trigger: CompactString,
    pub action: ActionId,
    pub payload: Option<ActionPayload>,
}

/// dispatch 返回的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionResult {
    /// 成功派发并被消费。
    Handled,
    /// action 未注册。
    UnknownAction(ActionId),
}

/// Action 注册表与派发器（spec IF-002 `ActionRegistry`）。
///
/// 具体实现由 `ui/runtime` 提供（持有 reducer 闭包表）；此处定义 trait 边界。
pub trait ActionRegistry {
    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_result_emit_and_payload() {
        let id = ActionId::new("app.inc");
        assert_eq!(EventResult::Emit(id.clone()), EventResult::Emit(id.clone()));

        let with_payload = EventResult::EmitWithPayload(id.clone(), ActionPayload::Int(3));
        match with_payload {
            EventResult::EmitWithPayload(a, ActionPayload::Int(n)) => {
                assert_eq!(a, id);
                assert_eq!(n, 3);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn unknown_action_returned_not_silent() {
        struct EmptyRegistry;
        impl ActionRegistry for EmptyRegistry {
            fn dispatch(&mut self, action: &ActionId, _payload: Option<ActionPayload>) -> ActionResult {
                ActionResult::UnknownAction(action.clone())
            }
        }
        let mut r = EmptyRegistry;
        let id = ActionId::new("nope");
        assert_eq!(r.dispatch(&id, None), ActionResult::UnknownAction(id));
    }
}
