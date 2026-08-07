//! P1a `document.elementFromPoint(x, y)` bridge——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 持共享 [`ElementFromPointCache`]（`Arc<Mutex<Option<Arc<HitTestCache>>>>`）；[`ElementFromPointBridge::register`]
//! 在 sandbox 注 `__zw_elementFromPoint(x, y)` 同步回调。JS shim 的 `document.elementFromPoint` 调此，
//! 传视口 CSS 逻辑像素坐标。回调锁内 clone `Arc<HitTestCache>` 出（廉价引用计数，不 clone 数据），
//! 释放锁后调 [`HitTestCache::hit_test_element`] + [`selector_from_element_hit`] → 稳定选择器串。
//!
//! **为何复用 `HitTestCache` 而非自建遍历**：[`zero_layout_engine::LayoutBox`] 坐标「相对父内容区」，
//! 命中须像 [`hit_test_element`] 内部 `deepest_node_at` 那样逐层累积才正确；直接复用鼠标点击已验证的
//! 命中路径，避免坐标逻辑重复（重复实现会算错嵌套元素）。每 render 由 renderer/browser 把最新
//! `HitTestCache` 经 [`Arc`] swap 进共享槽（无数据 clone，仅引用计数），JS 调用时读最新一份。
//!
//! **回落**：cache 未注入（engine/reftest/polyfill 路径无渲染）、坐标非法（NaN）、或落点在所有元素外
//! → 返空串 → shim 返 `null`（spec：无元素时 `elementFromPoint` 返 `null`），零回归。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

use crate::hit_test::{HitTestCache, selector_from_element_hit};

/// 共享 hit-test 缓存槽：render 写（swap 最新 `Arc<HitTestCache>`）/ JS worker 读（命中查询）。
///
/// `Arc<Mutex<Option<Arc<HitTestCache>>>>`——外层 `Mutex` 守 swap，内层 `Arc` 让 worker 锁内仅
/// clone 引用计数后释放锁，再 `hit_test_element`（树遍历）在锁外执行，不阻塞 render 更新。
/// `HitTestCache` 为纯数据（`LayoutBox` = f32 + Vec + `NodeId`），`Send + Sync`，可跨线程共享。
pub type ElementFromPointCache = Arc<Mutex<Option<Arc<HitTestCache>>>>;

/// 新建空缓存槽。
pub fn new_element_from_point_cache() -> ElementFromPointCache {
    Arc::new(Mutex::new(None))
}

/// 在 [`HitTestCache`] 上求 `(x, y)` 命中的最深元素选择器（无命中 → `None`）。
///
/// 复用 [`HitTestCache::hit_test_element`]（鼠标点击命中路径，坐标累积正确）+ [`selector_from_element_hit`]。
/// 抽为自由函数便于单测（不依赖 sandbox）。
pub fn selector_at_point(cache: &HitTestCache, x: f32, y: f32) -> Option<String> {
    cache.hit_test_element(x, y).map(|hit| selector_from_element_hit(&hit))
}

/// 锁内 clone `Arc<HitTestCache>` 出 → `selector_at_point`。NaN 坐标提前拦截（见下方注释）。
/// `register` 回调与 [`ElementFromPointBridge::lookup`] 共用，避免逻辑重复。
fn lookup_in_cell(cache_cell: &ElementFromPointCache, x: f32, y: f32) -> Option<String> {
    // NaN 坐标：`deepest_node_at` 的比较对 NaN 恒 false 会绕过早退、误命中根，提前拦截。
    if x.is_nan() || y.is_nan() {
        return None;
    }
    let cache_opt: Option<Arc<HitTestCache>> = cache_cell.lock().ok().and_then(|c| c.clone());
    cache_opt.and_then(|cache| selector_at_point(&cache, x, y))
}

/// P1a `document.elementFromPoint` bridge——`__zw_elementFromPoint(x, y)` 注册 + 命中查询。
pub struct ElementFromPointBridge {
    cache_cell: ElementFromPointCache,
}

impl ElementFromPointBridge {
    /// 构造——绑共享缓存槽（与 worker handle 暴露的 `element_from_point_cache()` 同一 `Arc`）。
    pub fn new(cache_cell: ElementFromPointCache) -> Self {
        Self { cache_cell }
    }

    /// 查询 `(x, y)` 命中的元素选择器（未注入 cache / 非法坐标 / 无命中 → `None`）。可单测（不依赖 sandbox）。
    pub fn lookup(&self, x: f32, y: f32) -> Option<String> {
        lookup_in_cell(&self.cache_cell, x, y)
    }

    /// 注册 `__zw_elementFromPoint(x, y)` 同步回调——shim `document.elementFromPoint` 调此。
    /// 返稳定选择器串（命中）；未注入 cache / 坐标非法 / 无命中 → 空串（shim 返 `null`）。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let cache_cell = Arc::clone(&self.cache_cell);
        sandbox.register_callback(
            "__zw_elementFromPoint",
            Box::new(move |args: &[String]| -> String {
                // sandbox 回调契约 `&[String]`——x/y 经 JS 侧 String() 转串后传入，parse 回 f32。
                let x: f32 = args.first().and_then(|s| s.parse().ok()).unwrap_or(f32::NAN);
                let y: f32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(f32::NAN);
                lookup_in_cell(&cache_cell, x, y).unwrap_or_default()
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hit_test::{HitTestCacheSnapshot, HitTestLayoutSnapshot, HitTestNodeSnapshot, node_id_from_u64};

    /// 构造测试用 [`HitTestCache`]：root `div`(0,0,800,600) + 子 `p#inner`(10,20,100,50)。
    /// `doc_root` = id0（非元素，模拟 Document 节点）→ 落点在所有元素外时 `hit_test_element` 返 `None`。
    fn sample_cache() -> HitTestCache {
        let id0 = node_id_from_u64(0); // Document 节点（不在 nodes）
        let id1 = node_id_from_u64(1); // div（root）
        let id2 = node_id_from_u64(2); // p#inner
        let layout_root = HitTestLayoutSnapshot {
            node_id: Some(id1),
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
            children: vec![HitTestLayoutSnapshot {
                node_id: Some(id2),
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 50.0,
                children: vec![],
            }],
        };
        let snap = HitTestCacheSnapshot {
            doc_root: id0,
            layout_root,
            nodes: vec![
                (
                    id1,
                    HitTestNodeSnapshot {
                        tag_name: "div".to_string(),
                        id: None,
                        class_name: None,
                        href: None,
                        src: None,
                    },
                ),
                (
                    id2,
                    HitTestNodeSnapshot {
                        tag_name: "p".to_string(),
                        id: Some("inner".to_string()),
                        class_name: None,
                        href: None,
                        src: None,
                    },
                ),
            ],
            parents: vec![(id2, id1)],
        };
        HitTestCache::from_snapshot(snap)
    }

    /// (50,40) 落在子 `p#inner`（绝对 10..110, 20..70）内 → 最深元素 `#inner`。
    #[test]
    fn selector_at_point_hits_deepest_nested() {
        let cache = sample_cache();
        assert_eq!(selector_at_point(&cache, 50.0, 40.0).as_deref(), Some("#inner"));
    }

    /// (5,5) 仅在 root `div` 内、子外 → root `div`（无 id/class → tag 选择器）。
    #[test]
    fn selector_at_point_hits_root_when_outside_child() {
        let cache = sample_cache();
        assert_eq!(selector_at_point(&cache, 5.0, 5.0).as_deref(), Some("div"));
    }

    /// (900,900) 落在所有元素外 → `None`（`doc_root` 非元素，`hit_test_element` 返 `None`）。
    #[test]
    fn selector_at_point_outside_all_returns_none() {
        let cache = sample_cache();
        assert_eq!(selector_at_point(&cache, 900.0, 900.0), None);
    }

    /// bridge.lookup 镜像回调逻辑：注入 cache 后命中返选择器。
    #[test]
    fn bridge_lookup_hits_after_cache_injected() {
        let cell = new_element_from_point_cache();
        let bridge = ElementFromPointBridge::new(Arc::clone(&cell));
        // 未注入 → None。
        assert_eq!(bridge.lookup(50.0, 40.0), None);
        // 注入 → 命中子元素 #inner。
        *cell.lock().unwrap() = Some(Arc::new(sample_cache()));
        assert_eq!(bridge.lookup(50.0, 40.0).as_deref(), Some("#inner"));
        assert_eq!(bridge.lookup(5.0, 5.0).as_deref(), Some("div"));
        assert_eq!(bridge.lookup(900.0, 900.0), None);
    }

    /// NaN 坐标提前拦截（否则 `deepest_node_at` 的 NaN 比较会误命中根）。
    #[test]
    fn bridge_lookup_nan_returns_none() {
        let cell = new_element_from_point_cache();
        *cell.lock().unwrap() = Some(Arc::new(sample_cache()));
        let bridge = ElementFromPointBridge::new(cell);
        assert_eq!(bridge.lookup(f32::NAN, 40.0), None);
        assert_eq!(bridge.lookup(50.0, f32::NAN), None);
    }
}
