//! 宿主指针锁定镜像与锁定态输入改道（pointerlock-compat M3 s3，RFC §6.4/§6.5）。
//!
//! 双馈管线（方案 A）的 webview 侧：shim 是语义权威（grant/deny/事件派发在
//! js_dom_shim——M3 s1 落地 `_firePointerLockEvent`），本模块只维护**锁镜像**
//! （host 桥 `__zw_pointer_lock_state_changed` 通知写 target/unadjusted）并在
//! 锁定态把宿主指针输入**改道**到锁目标——不 hit test、不合成跨界序（Pointer
//! Lock spec「mouse events are retargeted to the pointer lock target」，锁定态
//! 光标不可达其他元素，boundary 序无意义）。位移 delta 在本模块算（`last_pos`
//! 前后差，首 move = 0）；非锁定态输入仍走 `__zw_pointer_move` 合成管线
//! （uievents-compat，s2 位移流），两管线由调用方按 [`WebView::pointer_lock_target`]
//! 分流，防双派。
//!
//! 生命周期：grant/option 重锁/exit/移除解锁经 §6.5 host 桥写镜像（五处 shim
//! 调用点）；导航随导航重置点（webview.rs 四连点，与 drag_input 同点）清零并
//! 挂报告；桌面 Esc 释放走同一通知通道（s4）。

use std::sync::Arc;

use super::{WebView, WebViewError};
use zero_engine::DomEventDetail;

/// 宿主指针锁定镜像态——shim 语义权威的 webview 侧投影（RFC §6.4）。
#[derive(Debug, Default)]
pub struct PointerLockState {
    /// 锁目标元素唯一选择器（None = 未锁定）。
    target: Option<String>,
    /// 上一指针位置（锁定后首个 move 前为 None——首 move delta = 0，§8.2 边界注记）。
    last_pos: Option<(f32, f32)>,
    /// `requestPointerLock({unadjustedMovement})` 有效布尔（s1 shim 语义组镜像）。
    unadjusted: bool,
    /// 待上报的锁定布尔跃迁（None = 无变化；Some = 上报浏览器 TabManager 的
    /// locked 新值，s4 转发门/Esc 分支判据——RFC §6.5 PointerLockChanged）。
    pending_report: Option<bool>,
}

impl PointerLockState {
    /// 新建未锁定态。
    pub fn new() -> Self {
        Self::default()
    }

    /// 是否锁定（宿主仲裁面：runner 路由判据 + 桌面转发门）。
    pub fn is_locked(&self) -> bool {
        self.target.is_some()
    }

    /// 锁目标唯一选择器（未锁定 → None）。
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// unadjustedMovement 有效布尔（镜像读面，诊断/转发用）。
    pub fn unadjusted(&self) -> bool {
        self.unadjusted
    }

    /// 授锁（grant/option 重锁——同选择器重授按 shim 语义已去重到跃迁通知）。
    /// 锁定即重置位移基准（首 move delta = 0）。
    pub fn set_locked(&mut self, selector: String, unadjusted: bool) {
        self.target = Some(selector);
        self.unadjusted = unadjusted;
        self.last_pos = None;
    }

    /// 解锁（exit/移除/导航——镜像清零；锁定→未锁定跃迁挂报告）。
    pub fn set_unlocked(&mut self) {
        if self.target.take().is_some() {
            self.pending_report = Some(false);
        }
        self.unadjusted = false;
        self.last_pos = None;
    }

    /// 取待上报跃迁（渲染循环排空——每次取走后槽位清零）。
    pub fn take_report(&mut self) -> Option<bool> {
        self.pending_report.take()
    }

    /// 推进锁定态位移：返回 `(dx, dy)`（i64 截断——spec movement 为 long）并更新
    /// 基准；未锁定 → None（调用方不改道）。
    pub fn advance_move(&mut self, x: f32, y: f32) -> Option<(i64, i64)> {
        self.target.as_ref()?;
        let delta = match self.last_pos {
            // 首 move：只落基准，delta = 0（§8.2 风险注记「首 move delta=0」）。
            None => (0, 0),
            Some((last_x, last_y)) => ((x - last_x) as i64, (y - last_y) as i64),
        };
        self.last_pos = Some((x, y));
        Some(delta)
    }
}

/// 指针键相位（锁定态改道面——`apply_locked_pointer_button` 输入）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockedButtonPhase {
    /// 按下 → 改道派 mousedown。
    Down,
    /// 抬起 → 改道派 mouseup。
    Up,
}

/// §6.5 载荷落镜像（回调体抽出——可单测）：`{locked, selector, handle,
/// unadjustedMovement}` JSON。仅锁定布尔**跃迁**挂报告（deny 冗余通知/已锁重授
/// 静默）；grant 缺 selector 视为坏载荷（锁定无目标 = 镜像不可用）。
fn apply_lock_payload(state: &mut PointerLockState, payload: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        tracing::warn!("pointer lock state payload not JSON: {payload:?}");
        return;
    };
    let Some(locked) = value.get("locked").and_then(serde_json::Value::as_bool) else {
        tracing::warn!("pointer lock state payload missing locked flag");
        return;
    };
    if locked {
        let Some(selector) = value.get("selector").and_then(serde_json::Value::as_str) else {
            tracing::warn!("pointer lock grant payload missing selector");
            return;
        };
        let unadjusted = value
            .get("unadjustedMovement")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !state.is_locked() {
            state.pending_report = Some(true);
        }
        state.set_locked(selector.to_string(), unadjusted);
    } else if state.is_locked() {
        state.set_unlocked();
    }
}

impl WebView {
    /// 锁目标唯一选择器查询面（RFC §6.4）：runner 路由判据 + 桌面仲裁（经 IPC
    /// 回传，s4）。未锁定 → None。
    pub fn pointer_lock_target(&self) -> Option<String> {
        self.pointer_lock
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .target()
            .map(str::to_string)
    }

    /// 锁定态位移单入口（RFC §6.4）：未锁定 no-op；锁定则 delta 由镜像态算、
    /// 直接向锁目标派 mousemove（不 hit test、不合成跨界序），坐标/位移经
    /// s2 位移流 detail 透传（shim mouse 分支消费面）。
    pub fn apply_locked_pointer_move(&mut self, x: f32, y: f32) -> Result<bool, WebViewError> {
        let (target, delta) = {
            let mut state = self.pointer_lock.lock().unwrap_or_else(|e| e.into_inner());
            match state.advance_move(x, y) {
                Some(delta) => (state.target().unwrap_or_default().to_string(), delta),
                None => return Ok(false),
            }
        };
        let detail = DomEventDetail {
            client_x: Some(x),
            client_y: Some(y),
            movement_x: delta.0,
            movement_y: delta.1,
            ..Default::default()
        };
        self.dispatch_dom_event_inner(&target, "mousemove", Some(&detail))?;
        Ok(true)
    }

    /// 锁定态指针键改道（RFC §6.4 `apply_locked_pointer_button`）：Down/Up 各派
    /// mousedown/mouseup 到锁目标。click 合成不在本入口（shim `__zw_pointer_up_sequence`
    /// 组合逻辑针对合成管线；锁定态 click 语料未钉——FIXME: C 面实跑若暴露
    /// locked click 断言面再按 Pointer Lock spec §retarget 补组合）。
    pub fn apply_locked_pointer_button(
        &mut self,
        phase: LockedButtonPhase,
        x: f32,
        y: f32,
    ) -> Result<bool, WebViewError> {
        let target = match self.pointer_lock_target() {
            Some(target) => target,
            None => return Ok(false),
        };
        let event_type = match phase {
            LockedButtonPhase::Down => "mousedown",
            LockedButtonPhase::Up => "mouseup",
        };
        let detail = DomEventDetail {
            client_x: Some(x),
            client_y: Some(y),
            ..Default::default()
        };
        self.dispatch_dom_event_inner(&target, event_type, Some(&detail))?;
        Ok(true)
    }

    /// 取锁定跃迁报告（渲染循环排空——s4 PointerLockChanged IPC 的数据源；
    /// runner 直驱路径无 IPC，读 [`WebView::pointer_lock_target`] 即足）。
    pub fn take_pointer_lock_report(&mut self) -> Option<bool> {
        self.pointer_lock
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take_report()
    }

    /// 注册 §6.5 host 桥 `__zw_pointer_lock_state_changed`——shim grant/deny/exit/
    /// 移除解锁五处通知（payload JSON `{locked, selector, handle, unadjustedMovement}`，
    /// `typeof` 守卫未注册静默跳过）写镜像。与 identity bridge 同注册形（静态函数
    /// 先取 Arc 再借 sandbox——三武装点并列注册，浏览器与 runner 路径无分叉）。
    pub(crate) fn register_pointer_lock_state_callback(
        lock_mirror: &std::sync::Arc<std::sync::Mutex<PointerLockState>>,
        sandbox: &mut dyn zero_script_sandbox::Sandbox,
    ) {
        let shared = Arc::clone(lock_mirror);
        sandbox.register_callback(
            "__zw_pointer_lock_state_changed",
            Box::new(move |args: &[String]| {
                apply_lock_payload(
                    &mut shared.lock().unwrap_or_else(|e| e.into_inner()),
                    args.first().map(String::as_str).unwrap_or(""),
                );
                String::new()
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 未锁定 advance_move 返回 None（调用方不改道）；锁定后首 move delta=0、
    /// 后续 move 为前后差（i64 截断）。
    #[test]
    fn advance_move_quiets_when_unlocked_then_deltas() {
        let mut state = PointerLockState::new();
        assert!(!state.is_locked());
        assert_eq!(state.advance_move(10.0, 5.0), None);
        state.set_locked("div#t".into(), false);
        assert!(state.is_locked());
        assert_eq!(state.target(), Some("div#t"));
        assert_eq!(state.advance_move(10.0, 5.0), Some((0, 0)));
        assert_eq!(state.advance_move(13.0, 1.0), Some((3, -4)));
    }

    /// 授锁重置位移基准（重授后首 move delta=0，option 重锁面）。
    #[test]
    fn relock_resets_last_pos() {
        let mut state = PointerLockState::new();
        state.set_locked("div#t".into(), true);
        assert!(state.unadjusted());
        assert_eq!(state.advance_move(5.0, 5.0), Some((0, 0)));
        state.set_locked("div#t".into(), false);
        assert!(!state.unadjusted());
        assert_eq!(state.advance_move(9.0, 9.0), Some((0, 0)));
    }

    /// 解锁清镜像并挂 false 跃迁报告；未锁定重复解锁不挂（deny 通知静默）；
    /// take_report 取走即清。
    #[test]
    fn unlock_reports_transition_once() {
        let mut state = PointerLockState::new();
        state.set_unlocked();
        assert_eq!(state.take_report(), None);
        state.set_locked("div#t".into(), false);
        state.set_unlocked();
        assert!(!state.is_locked());
        assert_eq!(state.take_report(), Some(false));
        assert_eq!(state.take_report(), None);
    }

    /// grant 跃迁报告由注册侧写（locked false→true → Some(true)）；已锁定重授不重复。
    #[test]
    fn grant_reports_transition_once() {
        let mut state = PointerLockState::new();
        state.pending_report = Some(true);
        assert_eq!(state.take_report(), Some(true));
        // 已锁定重授：set_locked 不动报告槽（注册侧仅跃迁时写）。
        state.set_locked("div#t".into(), true);
        assert_eq!(state.take_report(), None);
    }

    /// §6.5 载荷解析：grant 写镜像+挂 true 报告；exit 挂 false 报告；deny 冗余
    /// 通知（未锁定 locked:false）静默；坏载荷（非 JSON/缺 locked/grant 缺
    /// selector）不动镜像。
    #[test]
    fn lock_payload_applies_transitions_only() {
        let mut state = PointerLockState::new();
        apply_lock_payload(
            &mut state,
            r#"{"locked":true,"selector":"div#t","handle":"h1","unadjustedMovement":true}"#,
        );
        assert!(state.is_locked());
        assert!(state.unadjusted());
        assert_eq!(state.take_report(), Some(true));
        // 已锁重授（unadjusted 翻转）→ 镜像更新、报告不重复。
        apply_lock_payload(
            &mut state,
            r#"{"locked":true,"selector":"div#t","handle":"h1","unadjustedMovement":false}"#,
        );
        assert!(!state.unadjusted());
        assert_eq!(state.take_report(), None);
        // exit → false 报告。
        apply_lock_payload(
            &mut state,
            r#"{"locked":false,"selector":null,"handle":null,"unadjustedMovement":false}"#,
        );
        assert!(!state.is_locked());
        assert_eq!(state.take_report(), Some(false));
        // deny 冗余（未锁定 locked:false）与坏载荷静默。
        apply_lock_payload(
            &mut state,
            r#"{"locked":false,"selector":null,"handle":null,"unadjustedMovement":false}"#,
        );
        apply_lock_payload(&mut state, "not-json");
        apply_lock_payload(&mut state, r#"{"selector":"div#x"}"#);
        apply_lock_payload(&mut state, r#"{"locked":true,"unadjustedMovement":false}"#);
        assert!(!state.is_locked());
        assert_eq!(state.take_report(), None);
    }

    /// e2e（runner/浏览器同路径）：页面 `requestPointerLock`（激活注入后）→ §6.5
    /// host 桥通知 → webview 镜像可查 + 跃迁报告；exit → 镜像清 + false 报告。
    /// 桥回调注册在 execute_script_with_dom 武装点（register_pointer_lock_state_callback
    /// 与 identity bridge 并列）。锁定态 `apply_locked_pointer_move` 改道派 mousemove
    /// （movement 由镜像 delta 算——页面监听收到）。
    #[test]
    fn lock_mirror_e2e_via_host_bridge() {
        use crate::WebViewConfig;
        let mut webview = WebView::new(WebViewConfig::default());
        webview.prepare_document_state("https://zero.test/lock");
        webview.load_html(r#"<html><body><div id="t"></div></body></html>"#, None);
        webview
            .execute_script_with_dom(
                "globalThis.__pl = [];\
                 document.addEventListener('mousemove', function (e) {\
                   globalThis.__pl.push(e.movementX + ',' + e.movementY + ',' + e.screenX);\
                 });",
            )
            .map_err(|error| format!("listener failed: {error}"))
            .unwrap();
        let target_sel = webview
            .execute_script_with_dom("document.getElementById('t').__zwSelector")
            .expect("sel");
        assert!(!target_sel.is_empty() && target_sel != "null", "锁目标 sel 可解析");
        // 未锁定 → 查询 None、改道 no-op、无报告。
        assert_eq!(webview.pointer_lock_target(), None);
        assert!(!webview.apply_locked_pointer_move(10.0, 10.0).unwrap());
        assert_eq!(webview.take_pointer_lock_report(), None);
        // 激活 + grant：镜像经 host 桥通知置位（grant 同步执行——微任务前可见）。
        webview
            .execute_script_with_dom("__zwUserActivate(); document.getElementById('t').requestPointerLock();")
            .map_err(|error| format!("grant failed: {error}"))
            .unwrap();
        assert_eq!(webview.pointer_lock_target().as_deref(), Some(target_sel.as_str()));
        assert_eq!(webview.take_pointer_lock_report(), Some(true));
        // 锁定态改道：mousemove 落锁目标（document 监听冒泡收到），movement = 镜像
        // delta（首 move 0、次 move 前后差），screenX=clientX（s2 合成面）。
        assert!(webview.apply_locked_pointer_move(10.0, 10.0).unwrap());
        assert!(webview.apply_locked_pointer_move(13.0, 12.0).unwrap());
        assert_eq!(
            webview
                .execute_script_with_dom("globalThis.__pl.join('|')")
                .expect("log"),
            "0,0,10|3,2,13",
            "锁定改道 mousemove 位移流 + screenX=clientX"
        );
        // exit：镜像清 + false 报告；改道复归 no-op。
        webview
            .execute_script_with_dom("document.exitPointerLock();")
            .map_err(|error| format!("exit failed: {error}"))
            .unwrap();
        assert_eq!(webview.pointer_lock_target(), None);
        assert_eq!(webview.take_pointer_lock_report(), Some(false));
        assert!(!webview.apply_locked_pointer_move(20.0, 20.0).unwrap());
    }
}
