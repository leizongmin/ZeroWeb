//! App 生命周期与入口 trait（spec IF-006 `UiApp`）。
//!
//! 宿主应用实现 `UiApp`，提供根声明、action dispatch；`PlatformRuntime` 驱动事件循环。

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult};
use zero_ui_core::theme::SemanticTokens;
use zero_ui_core::widget::WidgetSpec;

/// 宿主应用 trait（spec IF-006）。
pub trait UiApp {
    /// 当前根声明树（每次 rebuild 后由 runtime reconcile）。
    fn root_spec(&self) -> WidgetSpec;

    /// 派发 action 到应用 reducer（spec FR-003 单向数据流）。
    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult;

    /// 当前主题的 semantic token（P1-6 主题单源）。
    ///
    /// 返回 `Some` 时由 runtime 在 `begin` / 每次 pump 帧注入 `WidgetHost::set_tokens`，
    /// 让控件 paint 经 `PaintCtx.tokens` 消费，无需各自存 theme 字段。
    /// 返回 `None`（默认）则保持 host 当前 tokens 不变，向后兼容无主题概念的 example。
    fn theme_tokens(&self) -> Option<SemanticTokens> {
        None
    }
}
