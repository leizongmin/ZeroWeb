//! App 生命周期与入口 trait（spec IF-006 `UiApp`）。
//!
//! 宿主应用实现 `UiApp`，提供根声明、action dispatch；`PlatformRuntime` 驱动事件循环。

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult};
use zero_ui_core::widget::WidgetSpec;

/// 宿主应用 trait（spec IF-006）。
pub trait UiApp {
    /// 当前根声明树（每次 rebuild 后由 runtime reconcile）。
    fn root_spec(&self) -> WidgetSpec;

    /// 派发 action 到应用 reducer（spec FR-003 单向数据流）。
    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult;
}
