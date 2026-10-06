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
//! `(html, drain_gen, style_version)` 代际键下的 parsed doc + 全文档计算样式，规避
//! `Send` 约束。同代际内所有 selector 查询摊销为一次 parse+cascade。
//!
//! **键契约**（与视图缓存精确命中键同构，见 callbacks.rs `MUT_DRAIN_GEN` 注）：
//! `drain_gen`（mutation 队列 drain 代际）把「drain 后重长回同 len 但内容不同」的
//! 批次隔到不同代；`style_version`（R3030 mutation 计数）在单个 drain 代内单调递增，
//! len 相等 ⇒ 队列前缀相同；`html` 全文兜底快照换代。三者任一变化即换代重算。
//! webview 文档换代就地 clear 且不 bump drain_gen 的已知例外，由
//! [`register_dom_callbacks`] 每次注册时调 [`clear_generation_cache`] 覆盖（注册装新
//! 快照 → 旧代际不得复用；同线程先后多沙箱同理隔离）。
//!
//! **内存驻留**：单槽有界、换代即 drop 旧代际，但在 js worker 线程存活期内持有最后
//! 一代的整页 `Document` + 全文档 styles + html 全文拷贝（重页页级内存常驻，直到该
//! 线程下一次 gCS 或线程结束）——相对旧实现（仅 per-selector 值）的内存形态阶跃，
//! 换取 E14 停摆消除。

use std::cell::RefCell;
use std::collections::HashMap;

use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

use super::DomMutation;
use super::computed_style::compute_document_styles_with_inline_overrides;

thread_local! {
    /// `(html 快照, drain 代际, style_version, parsed doc, NodeId→ComputedStyle)` 代际槽。
    /// 非 `Send` 类型（`Document`）仅存于创建线程（js worker），无跨线程共享。
    static GENERATION_CACHE:
        RefCell<Option<(String, usize, usize, Document, HashMap<NodeId, ComputedStyle>)>> =
            const { RefCell::new(None) };

    /// 测试观测点：本线程 doc 全量重算次数（仅 `cfg(test)` 编译，生产零开销）。
    /// `clear_generation_cache` 的直接单测靠它判定「清槽后同键查询重算」——计数与
    /// 槽同线程，天然隔离并行测试。
    #[cfg(test)]
    pub static GENERATION_RECOMPUTE_COUNT: std::cell::Cell<usize> =
        const { std::cell::Cell::new(0) };
}

/// 清空本线程的代际槽（换代即 drop 旧代际的 `Document` 与 styles）。
/// `register_dom_callbacks` 注册时调用：新注册装新快照，旧代际一律不复用
///（覆盖 webview 就地 clear 不 bump drain_gen 的例外与同线程多沙箱隔离）。
pub fn clear_generation_cache() {
    GENERATION_CACHE.with(|cell| *cell.borrow_mut() = None);
}

/// 在 `(html, drain_gen, style_version)` 代际缓存的 `(Document, NodeId→ComputedStyle)`
/// 上执行 `f`。
///
/// 代际未变 → 复用 parsed doc（不重算）；代际变（html 快照更新 / mutation 队列
/// drain 前进 / mutation 计数前进任一）→ 全量 parse + inline override + cascade 一次
/// 后换代。返回 `f` 的结果。
///
/// **契约**：同代际调用必须传等价的 `mutations` 内容（键不含内容摘要——由键契约
/// 保证同代际队列前缀相同）；`f` 不得重入本函数（RefCell 借用期内；host 回调单线程
/// 执行且 `f` 体无 JS/宿主回流，现状唯一调用方满足）。
pub fn with_cached_document_styles<T>(
    html: &str,
    drain_gen: usize,
    style_version: usize,
    mutations: &[DomMutation],
    f: impl FnOnce(&Document, &HashMap<NodeId, ComputedStyle>) -> T,
) -> T {
    GENERATION_CACHE.with(|cell| {
        {
            let mut slot = cell.borrow_mut();
            let valid = matches!(
                &*slot,
                Some((h, g, v, _, _)) if h == html && *g == drain_gen && *v == style_version
            );
            if !valid {
                #[cfg(test)]
                GENERATION_RECOMPUTE_COUNT.with(|c| c.set(c.get() + 1));
                let (doc, styles) = compute_document_styles_with_inline_overrides(html, mutations);
                *slot = Some((html.to_string(), drain_gen, style_version, doc, styles));
            }
        }
        // 不变式：上方 borrow_mut 块保证槽必为 Some（命中沿用或换代重填），故 expect 不可达。
        let binding = cell.borrow();
        let (_, _, _, doc, styles) = binding.as_ref().expect("generation cache populated");
        f(doc, styles)
    })
}
