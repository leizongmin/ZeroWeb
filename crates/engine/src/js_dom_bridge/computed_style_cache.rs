//! getComputedStyle per-generation `(Document, styles)` 缓存——真站 js worker 停摆（E14）
//! 根因修复。从 [`super::computed_style`] 拆出以控制主文件行数。
//!
//! [`super::computed_style::compute_document_styles_with_inline_overrides`] 每次全量
//! parse+cascade（真实大页面单次 ~600ms）；生产回调 `__zw_get_computed_style` 因
//! `Document` 非 `Send`（observer/listener 闭包 + html5ever tendril `Cell`）不能把
//! parsed doc 放进 `Send + Sync` 回调闭包缓存，退化为**每新 selector 重算一次**——
//! 真站（bilibili 首页）轮播/懒加载每 tick 量测几十个新元素，每个 30s 臂 15-48 次
//! 全量重算，js worker 被 30s V8 watchdog 击杀占死，CDP evaluate 全部排队超时
//! （E14 诊断证据 hang-diag-r9~r13：170 次慢调用均值 628ms，合计 106.8s/轮）。
//!
//! **解法**：host 回调只在 js worker 单线程执行（`register_callback` 闭包经
//! `host_callback_invoke` 于脚本执行期同步调用）——`thread_local` 持有
//! `(html, style_version)` 代际键下的 parsed doc + 全文档计算样式，规避 `Send`
//! 约束。同代际内所有 selector 查询摊销为一次 parse+cascade；html 快照或
//! style_version（R3030 mutation 计数）任一变化即换代重算。

use std::cell::RefCell;
use std::collections::HashMap;

use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use super::DomMutation;
use super::computed_style::compute_document_styles_with_inline_overrides;

thread_local! {
    /// `(html 快照, style_version, parsed doc, NodeId→ComputedStyle)` 代际槽。
    /// 非 `Send` 类型（`Document`）仅存于创建线程（js worker），无跨线程共享。
    static GENERATION_CACHE: RefCell<Option<(String, usize, Document, HashMap<NodeId, ComputedStyle>)>> =
        const { RefCell::new(None) };
}

/// 在 `(html, style_version)` 代际缓存的 `(Document, NodeId→ComputedStyle)` 上执行 `f`。
///
/// 代际未变 → 复用 parsed doc（不重算）；代际变（html 快照更新或 mutation 计数前进）
/// → 全量 parse + inline override + cascade 一次后换代。返回 `f` 的结果。
///
/// **契约**：`f` 不得重入本函数（RefCell 借用期内）；host 回调单线程执行天然满足。
pub fn with_cached_document_styles<T>(
    html: &str,
    style_version: usize,
    mutations: &[DomMutation],
    f: impl FnOnce(&Document, &HashMap<NodeId, ComputedStyle>) -> T,
) -> T {
    GENERATION_CACHE.with(|cell| {
        {
            let mut slot = cell.borrow_mut();
            let valid = matches!(&*slot, Some((h, v, _, _)) if h == html && *v == style_version);
            if !valid {
                let (doc, styles) = compute_document_styles_with_inline_overrides(html, mutations);
                *slot = Some((html.to_string(), style_version, doc, styles));
            }
        }
        let binding = cell.borrow();
        let (_, _, doc, styles) = binding.as_ref().expect("generation cache populated");
        f(doc, styles)
    })
}
