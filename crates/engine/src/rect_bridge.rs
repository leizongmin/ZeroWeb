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

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

use crate::hit_test::{HitTestLayoutSnapshot, node_id_to_u64};

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

// ── 共享 layout-rect snapshot（render 写 / JS worker 读，跨线程）──

/// 共享 layout-rect snapshot：`NodeId`(u64) → rect。
///
/// renderer 主循环 render 后从 `HitTestCache` 填充；js_worker 的 RectBridge handler 读它
/// （经 identity → NodeId 解析后查 rect）。`Arc<Mutex<>>` 跨 render 线程 / js_worker 线程共享。
pub type LayoutRectSnapshot = Arc<Mutex<HashMap<u64, Rect4>>>;

/// 新建空 snapshot。
pub fn new_layout_rect_snapshot() -> LayoutRectSnapshot {
    Arc::new(Mutex::new(HashMap::new()))
}

/// 从 `HitTestCache` 的 layout 树遍历填 snapshot——每个有 `node_id` 的节点 → `(x,y,width,height)`。
///
/// 在 render 后调（renderer 主循环）。锁内递归遍历；无 `node_id` 的匿名盒跳过。
pub fn fill_layout_rect_snapshot(root: &HitTestLayoutSnapshot, snapshot: &LayoutRectSnapshot) {
    if let Ok(mut map) = snapshot.lock() {
        map.clear();
        fill_rect_recursive(root, &mut map);
    }
}

fn fill_rect_recursive(node: &HitTestLayoutSnapshot, map: &mut HashMap<u64, Rect4>) {
    if let Some(id) = node.node_id {
        map.insert(node_id_to_u64(id), (node.x, node.y, node.width, node.height));
    }
    for child in &node.children {
        fill_rect_recursive(child, map);
    }
}

/// 构造生产 rect 查询 handler——`identity`(selector) → 解析 `dom_html` → [`find_by_selector`]
/// → `NodeId` → 查 `snapshot`。
///
/// **为何可行（gBCR path C 的地基）**：渲染管线每次 render 都 fresh-`parse_html` 同一 html 字符串
/// （`pipeline_budget.rs:106/197`），js_worker 持有的 `dom_html` 是同一字符串；slotmap fresh-map +
/// 相同插入顺序 → 确定性 `NodeId`（见 `test_node_id_determinism_across_fresh_parses`），故 handler
/// 的 `find_by_selector` 解析出的 `NodeId` 与 snapshot 键一致。
///
/// **Document 缓存（thread-local）**：`zero_dom::Document` 非 `Send`，无法跨调用缓存于
/// `Send + Sync` handler 闭包；改用 [`RECT_DOC_CACHE`]（per-thread）——每个 js_worker 线程独立槽，
/// html 字符串变化才重 parse（同 render 帧多次 gBCR 复用同一 Document，消除循环调用的 parse 陡坡）。
///
/// `identity` = shim 元素 proxy 的 selector（`querySelector`/`getElementById` 返 stable_selector）；
/// handle-identity（`createElement` 节点）`find_by_selector` 不匹配 → `None` → shim 回落零 rect（follow-up）。
pub fn make_dom_html_rect_handler(dom_html: Arc<Mutex<String>>, snapshot: LayoutRectSnapshot) -> RectLookupHandler {
    Arc::new(move |identity: &str| -> Option<Rect4> {
        let html = dom_html.lock().ok()?.clone();
        // html 变化才重 parse；同 html 复用缓存的 Document（消除每 query 一次 parse）。
        let node_id = RECT_DOC_CACHE.with(|cache| {
            {
                let mut c = cache.borrow_mut();
                let stale = c.as_ref().is_none_or(|(h, _)| h != &html);
                if stale {
                    *c = Some((html.clone(), zero_dom::parse_html(&html)));
                }
            } // 释放 borrow_mut
            let c = cache.borrow();
            let (_, doc) = c.as_ref().expect("cache populated above");
            crate::js_dom_bridge::find_by_selector(doc, identity)
        })?;
        let snap = snapshot.lock().ok()?;
        snap.get(&node_id_to_u64(node_id)).copied()
    })
}

// gBCR handler 的 Document 缓存（per-thread）。Document 非 Send，不能用 Arc<Mutex<>> 跨
// Send + Sync 闭包；thread_local 是 per-thread，每个 js_worker 线程独立槽，无 Send 约束。
// 键 = html 字符串；html 变化（render 后 dom_html 更新）触发重 parse。线程退出时随 thread_local 释放。
thread_local! {
    static RECT_DOC_CACHE: RefCell<Option<(String, zero_dom::Document)>> = const { RefCell::new(None) };
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

    /// fill_layout_rect_snapshot：从 HitTestLayoutSnapshot 树填 NodeId→rect；无 node_id 的盒跳过。
    /// 键 = `node_id_to_u64(node_id)`（与 handler 查询同变换），非原始 ffi 整数。
    #[test]
    fn test_fill_layout_rect_snapshot() {
        use crate::hit_test::{HitTestLayoutSnapshot, node_id_from_u64, node_id_to_u64};
        let id1 = node_id_from_u64(1);
        let id2 = node_id_from_u64(2);
        let root = HitTestLayoutSnapshot {
            node_id: Some(id1),
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            children: vec![
                HitTestLayoutSnapshot {
                    node_id: Some(id2),
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                    children: vec![],
                },
                HitTestLayoutSnapshot {
                    // 匿名盒：无 node_id，应跳过
                    node_id: None,
                    x: 5.0,
                    y: 5.0,
                    width: 5.0,
                    height: 5.0,
                    children: vec![],
                },
            ],
        };
        let snap = new_layout_rect_snapshot();
        fill_layout_rect_snapshot(&root, &snap);
        let map = snap.lock().unwrap();
        assert_eq!(map.len(), 2, "应有 2 个有 node_id 的节点（匿名盒跳过）");
        assert_eq!(map.get(&node_id_to_u64(id1)), Some(&(0.0, 0.0, 800.0, 600.0)));
        assert_eq!(map.get(&node_id_to_u64(id2)), Some(&(10.0, 20.0, 100.0, 50.0)));
        assert!(
            map.get(&node_id_to_u64(node_id_from_u64(999))).is_none(),
            "未填的 node_id 应缺席"
        );
    }

    /// gBCR path (C) 的地基：同一 HTML 字符串两次 fresh `parse_html` 对同一 selector 返回相同 NodeId。
    ///
    /// 渲染管线每次 render 都 fresh-parse 同一 html 字符串（`pipeline_budget.rs:106/197`），
    /// js_worker 的 RectBridge handler 也 fresh-parse 它持有的 `dom_html`。两者 NodeId 必须一致，
    /// handler 才能用 `find_by_selector` 解析身份 → 查 NodeId-keyed snapshot。slotmap fresh-map
    /// 配合相同插入顺序 → 确定性 NodeId；本测试守护该不变量（若 dom crate 改动破坏确定性，
    /// gBCR 会静默回落零 rect）。
    #[test]
    fn test_node_id_determinism_across_fresh_parses() {
        use zero_dom::parse_html;
        let html = "<!DOCTYPE html><html><body>\
                    <div id='t' style='width:100px;height:50px'>hi</div>\
                    <span class='c'>x</span>\
                    </body></html>";
        let doc_a = parse_html(html);
        let doc_b = parse_html(html);
        for sel in ["#t", "span.c", "span", "div"] {
            let id_a = crate::js_dom_bridge::find_by_selector(&doc_a, sel)
                .unwrap_or_else(|| panic!("selector {sel} should match in doc_a"));
            let id_b = crate::js_dom_bridge::find_by_selector(&doc_b, sel)
                .unwrap_or_else(|| panic!("selector {sel} should match in doc_b"));
            assert_eq!(
                node_id_to_u64(id_a),
                node_id_to_u64(id_b),
                "NodeId for {sel} must be deterministic across fresh parses (gBCR path-C foundation)"
            );
        }
    }

    /// make_dom_html_rect_handler：identity(selector) → 解析 dom_html → NodeId → snapshot rect。
    /// 验证 handler 闭包逻辑（parse + find_by_selector + 缓存命中复用 + snapshot 查询 + 未命中 None）。
    #[test]
    fn test_make_dom_html_rect_handler_resolves_selector_to_rect() {
        use crate::js_dom_bridge::find_by_selector;
        use zero_dom::parse_html;
        let html = "<!DOCTYPE html><html><body>\
                    <div id='t' style='width:100px;height:50px'>hi</div>\
                    </body></html>";
        // snapshot 键 = 「同一 html fresh-parse」的 NodeId（模拟渲染管线填充）。
        let doc = parse_html(html);
        let id_t = find_by_selector(&doc, "#t").expect("#t");
        let snapshot = new_layout_rect_snapshot();
        snapshot
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_t), (10.0, 20.0, 100.0, 50.0));

        let dom_html = Arc::new(Mutex::new(html.to_string()));
        let handler = make_dom_html_rect_handler(dom_html, snapshot);

        assert_eq!(
            handler("#t"),
            Some((10.0, 20.0, 100.0, 50.0)),
            "selector identity → rect"
        );
        assert_eq!(
            handler("#nonexistent"),
            None,
            "未命中 selector → None（shim 回落零 rect）"
        );
        assert_eq!(handler("__n1"), None, "handle-identity 暂不支持 → None（follow-up）");
    }

    /// thread-local Document 缓存：dom_html 变化时缓存失效重 parse（否则 gBCR 会查旧 Document 返错/漏）。
    /// 验证 html1→html2 切换后 handler 用新 Document 解析（#b 命中、#a 不再存在）。
    #[test]
    fn test_dom_html_rect_handler_cache_invalidates_on_html_change() {
        use crate::js_dom_bridge::find_by_selector;
        use zero_dom::parse_html;
        let snapshot = new_layout_rect_snapshot();
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let handler = make_dom_html_rect_handler(Arc::clone(&dom_html), Arc::clone(&snapshot));

        // html1: <div id='a'>——首查询触发 parse + 缓存。
        let html1 = "<html><body><div id='a'>A</div></body></html>";
        *dom_html.lock().unwrap() = html1.to_string();
        let id_a = find_by_selector(&parse_html(html1), "#a").expect("#a in html1");
        snapshot
            .lock()
            .unwrap()
            .insert(node_id_to_u64(id_a), (1.0, 2.0, 3.0, 4.0));
        assert_eq!(handler("#a"), Some((1.0, 2.0, 3.0, 4.0)));

        // html2: <span id='b'>（结构不同）→ 缓存须失效重 parse。
        let html2 = "<html><body><span id='b'>B</span></body></html>";
        *dom_html.lock().unwrap() = html2.to_string();
        let id_b = find_by_selector(&parse_html(html2), "#b").expect("#b in html2");
        let mut snap = snapshot.lock().unwrap();
        snap.clear();
        snap.insert(node_id_to_u64(id_b), (5.0, 6.0, 7.0, 8.0));
        drop(snap);
        assert_eq!(handler("#b"), Some((5.0, 6.0, 7.0, 8.0)), "html 切换后新 selector 命中");
        // #a 在 html2 不存在 → None（证缓存已切到 html2 的 Document，非沿用 html1）。
        assert_eq!(handler("#a"), None, "旧 selector 在新 html 应不存在（缓存已失效）");
    }

    /// fill_layout_rect_snapshot 二次填充：clear 后重填（render 更新后换新 rect）。
    #[test]
    fn test_fill_layout_rect_snapshot_clears_before_refill() {
        use crate::hit_test::{HitTestLayoutSnapshot, node_id_from_u64, node_id_to_u64};
        let snap = new_layout_rect_snapshot();
        let id1 = node_id_from_u64(1);
        let id2 = node_id_from_u64(2);
        // 首次填：node 1
        fill_layout_rect_snapshot(
            &HitTestLayoutSnapshot {
                node_id: Some(id1),
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
                children: vec![],
            },
            &snap,
        );
        assert_eq!(snap.lock().unwrap().len(), 1);
        // 二次填：node 2（node 1 应被 clear 掉）
        fill_layout_rect_snapshot(
            &HitTestLayoutSnapshot {
                node_id: Some(id2),
                x: 5.0,
                y: 5.0,
                width: 50.0,
                height: 50.0,
                children: vec![],
            },
            &snap,
        );
        let map = snap.lock().unwrap();
        assert_eq!(map.len(), 1, "二次填充前应 clear");
        assert!(map.get(&node_id_to_u64(id1)).is_none(), "旧 node_id 应被 clear");
        assert_eq!(map.get(&node_id_to_u64(id2)), Some(&(5.0, 5.0, 50.0, 50.0)));
    }
}
