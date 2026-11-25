//! T112 layout 探针的 taffy 缓存命中/未命中计数（perf-mem-20261124）。
//!
//! [`crate::compute::compute_cached_layout`] 在每次节点查找处调
//! [`note_cache`]——命中/未命中各一原子桶；`zero-layout-engine` 的
//! `layout_probe` flush 时经 [`take_counts`] 取走并清零（同进程同线程域——
//! taffy 计算发生在 renderer 布局线程，探针读取在同线程 flush 点）。
//!
//! env 门控：`ZEROWEB_LAYOUT_PROBE=<sink 路径>`——与 zero-layout-engine
//! `layout_probe` 模块解析同一 env、同一三态语义（`1`/`true`/`0`/`false`/
//! `off`/空 = 关闭，其余非空 = 启用）。本 crate 不依赖该 crate（依赖方向
//! 相反），故在此按相同规则独立解析；两侧解析须逐字一致（任一侧改动须同步
//! 审查另一侧——判读口径冻结面）。默认关闭；关闭时每查找一次
//! [`AtomicBool::load`]（Relaxed），零分配零 IO。
//!
//! 初始化：[`init_env`] 由 zero-layout-engine `layout_probe::enabled()` 在
//! 任何 compute 前调用（OnceLock 单次）；`note_cache` 在未初始化时按关闭
//! 处理（首趟前的计数不存在的窗口不存在——init 先于第一次 compute）。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
static HITS: AtomicU64 = AtomicU64::new(0);
static MISSES: AtomicU64 = AtomicU64::new(0);
static INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

/// env 三态解析（与 zero-layout-engine `sink_path_from_value` 逐字同规则：
/// 路径语义启用；truthy/falsy/空关闭）。
fn env_says_enabled(value: Option<&str>) -> bool {
    let Some(value) = value else { return false };
    let truthy = value == "1" || value.eq_ignore_ascii_case("true");
    let falsy = value == "0" || value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("off");
    if truthy || falsy {
        return false;
    }
    !value.trim().is_empty()
}

/// 进程级单次 env 读取（由 zero-layout-engine 探针启用路径触发）。
pub fn init_env() {
    INIT.get_or_init(|| {
        let on = env_says_enabled(std::env::var("ZEROWEB_LAYOUT_PROBE").ok().as_deref());
        ENABLED.store(on, Ordering::Relaxed);
    });
}

/// 缓存查找结果记账（每节点每趟一次；禁用时一次布尔判零）。
#[inline]
pub fn note_cache(hit: bool) {
    if ENABLED.load(Ordering::Relaxed) {
        if hit {
            HITS.fetch_add(1, Ordering::Relaxed);
        } else {
            MISSES.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// 取走并清零计数（探针 flush 点调用）。
pub fn take_counts() -> (u64, u64) {
    (HITS.swap(0, Ordering::Relaxed), MISSES.swap(0, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_semantics_match_layout_probe() {
        // 与 zero-layout-engine sink_path_from_value 同表：truthy/falsy/空 = 关。
        assert!(!env_says_enabled(Some("1")));
        assert!(!env_says_enabled(Some("true")));
        assert!(!env_says_enabled(Some("0")));
        assert!(!env_says_enabled(Some("false")));
        assert!(!env_says_enabled(Some("off")));
        assert!(!env_says_enabled(Some("")));
        assert!(!env_says_enabled(Some("  ")));
        assert!(!env_says_enabled(None));
        assert!(env_says_enabled(Some("/tmp/x.sink")));
    }

    #[test]
    fn take_counts_swaps_to_zero() {
        // 原子桶 swap 语义直测（不经 env 门）。
        HITS.store(3, Ordering::Relaxed);
        MISSES.store(5, Ordering::Relaxed);
        let (h, m) = take_counts();
        assert_eq!((h, m), (3, 5));
        let (h2, m2) = take_counts();
        assert_eq!((h2, m2), (0, 0));
    }
}
