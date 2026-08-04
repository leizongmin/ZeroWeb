//! P1a layout-geometry 反馈 bridge——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 持 [`RectLookupHandler`]（元素身份 → 布局 rect）；`register` 在 sandbox 注
//! `__zw_getBoundingClientRect(identity)` 同步回调。JS shim 的 `getBoundingClientRect()`
//! 传元素身份（handle `__n{n}` 或 selector），本回调锁内克隆 handler Option 后 inline 查询，
//! 返回 `"x,y,w,h"`（无 handler / 未命中 → 空串 → shim 回落零 rect，零回归）。
//!
//! 与 `FetchBridge`/`TimerBridge` 同 handler-cell 模式，但**同步**——rect 查询无网络/wait，
//! `register_callback` 契约 `&[String] -> String` 直接返，不需 `AsyncResolver`/子线程。
//! 元素身份 → NodeId 的解析（compound key）封装在 wiring 侧注入的 handler 闭包内，
//! 故 RectBridge 本身不依赖 DOM/layout 细节，保持通用。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

/// 布局 rect（x, y, w, h），序列化为 `"x,y,w,h"` 供 JS 解析。
pub type Rect4 = (f32, f32, f32, f32);

/// 元素身份 → 布局 rect 查询闭包。
///
/// 身份 = shim 元素 proxy 的 compound key（handle `__n{n}` 或 selector）。handler 由 wiring 侧
/// 注入，内部解析身份 → `NodeId` → 查 layout-rect snapshot。返回 `None` 表示未命中（回落零 rect）。
pub type RectLookupHandler = Arc<dyn Fn(&str) -> Option<Rect4> + Send + Sync>;

/// 锁内克隆 handler Option 后调用（`register` 回调与 `lookup` 共用，避免逻辑重复）。
fn invoke_handler(handler_cell: &Mutex<Option<RectLookupHandler>>, identity: &str) -> Option<Rect4> {
    let handler_opt: Option<RectLookupHandler> = handler_cell.lock().ok().and_then(|c| c.as_ref().cloned());
    handler_opt.and_then(|h| h(identity))
}

/// P1a layout-geometry 反馈 bridge——`getBoundingClientRect` 真实化（unlock IntersectionObserver/
/// ResizeObserver 的共同基建）。
pub struct RectBridge {
    handler_cell: Arc<Mutex<Option<RectLookupHandler>>>,
}

impl RectBridge {
    /// 构造——handler 延后由 [`Self::set_handler`] 注入（chicken-and-egg：worker spawn 时
    /// layout-rect snapshot 未就绪）。
    pub fn new() -> Self {
        Self {
            handler_cell: Arc::new(Mutex::new(None)),
        }
    }

    /// 注入生产 rect 查询 handler（wiring 侧在 layout-rect snapshot 就绪后调）。
    /// 多次调用：后注入者覆盖前者。
    pub fn set_handler(&self, handler: RectLookupHandler) {
        if let Ok(mut cell) = self.handler_cell.lock() {
            *cell = Some(handler);
        }
    }

    /// 查询元素身份的布局 rect（handler 未注入或未命中 → `None`）。可单测（不依赖 sandbox）。
    pub fn lookup(&self, identity: &str) -> Option<Rect4> {
        invoke_handler(&self.handler_cell, identity)
    }

    /// 注册 `__zw_getBoundingClientRect(identity)` 同步回调——shim 的 `getBoundingClientRect` 调此。
    /// 返回 `"x,y,w,h"`；handler 未注入或未命中 → 空串（shim 回落零 rect，零回归）。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let handler_cell = Arc::clone(&self.handler_cell);
        sandbox.register_callback(
            "__zw_getBoundingClientRect",
            Box::new(move |args: &[String]| -> String {
                let identity = args.first().map(String::as_str).unwrap_or("");
                match invoke_handler(&handler_cell, identity) {
                    Some((x, y, w, h)) => format!("{x},{y},{w},{h}"),
                    None => String::new(),
                }
            }),
        );
    }
}

impl Default for RectBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// handler 未注入 → lookup 永远 None（shim 回落零 rect，零回归）。
    #[test]
    fn test_no_handler_returns_none() {
        let bridge = RectBridge::new();
        assert_eq!(bridge.lookup("div"), None);
        assert_eq!(bridge.lookup(""), None);
    }

    /// 注入 handler 后命中身份返 rect、未命中返 None。
    #[test]
    fn test_handler_hit_and_miss() {
        let bridge = RectBridge::new();
        bridge.set_handler(Arc::new(|id: &str| {
            if id == "__n1" {
                Some((10.0, 20.0, 100.0, 50.0))
            } else {
                None
            }
        }));
        assert_eq!(bridge.lookup("__n1"), Some((10.0, 20.0, 100.0, 50.0)));
        assert_eq!(bridge.lookup("__n2"), None); // 未命中
    }

    /// 多次 set_handler：后注入者覆盖前者（layout 更新后换 handler）。
    #[test]
    fn test_set_handler_overrides() {
        let bridge = RectBridge::new();
        bridge.set_handler(Arc::new(|_| Some((1.0, 2.0, 3.0, 4.0))));
        assert_eq!(bridge.lookup("any"), Some((1.0, 2.0, 3.0, 4.0)));
        bridge.set_handler(Arc::new(|_| Some((9.0, 8.0, 7.0, 6.0))));
        assert_eq!(bridge.lookup("any"), Some((9.0, 8.0, 7.0, 6.0)));
    }

    /// handler 可读身份字符串内容（模拟 compound key：handle vs selector 分支）。
    #[test]
    fn test_handler_reads_identity() {
        let bridge = RectBridge::new();
        bridge.set_handler(Arc::new(|id: &str| {
            if let Some(n) = id.strip_prefix("__n") {
                let v: f32 = n.parse().unwrap_or(0.0);
                Some((0.0, 0.0, v, v))
            } else if id.starts_with("div") {
                Some((5.0, 5.0, 200.0, 100.0))
            } else {
                None
            }
        }));
        assert_eq!(bridge.lookup("__n42"), Some((0.0, 0.0, 42.0, 42.0)));
        assert_eq!(bridge.lookup("div.main"), Some((5.0, 5.0, 200.0, 100.0)));
        assert_eq!(bridge.lookup("span"), None);
    }
}
