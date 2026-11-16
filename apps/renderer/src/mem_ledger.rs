//! T78 域账面（读数式）：renderer 进程保留对象的分域字节读数，随 alloc-trace
//! dump 落盘，与 PSS/alloc 采样互校定位常驻构成。
//!
//! 形制：发布点录数（[`record`]）+ dump 点快照（[`snapshot_value`]）——主线程
//! 在 `publish_webview_now` 读数一次（估算均为 O(entries) 容量/长度求和，微秒
//! 级），dump 线程只读 `Mutex` 内的既有数字，不触 WebView/render-foundation
//! 状态（dump 线程无跨线程借用面）。进程级门 = alloc-trace feature 编译 +
//! `ZW_ALLOC_TRACE=1`（armed 与 [`crate::alloc_trace`] 同源）；域键为稳定字面
//! 量，分析侧按键判读。
//!
//! 账外域（读数不入账，分析侧用 smaps 差分归因）：V8 cage 不经 `System`
//! 分配器（既有 mem_probe 探针覆盖）；shaping/hmtx 等有界脚本缓存（另域）。

use std::collections::BTreeMap;
use std::sync::Mutex;

/// 域读数表（发布点覆写写，dump 点整体读）。`BTreeMap` 使快照键序确定。
///
/// 键形约定：字节域以 `_bytes` 结尾（快照 `sum_bytes` 只计这些），计数域以
/// `_entries` 结尾。
static LEDGER: Mutex<Option<BTreeMap<&'static str, u64>>> = Mutex::new(None);

/// 发布点录数：覆写同名域读数（每帧读数 = 最新帧口径）。
///
/// 调用方（runtime 发布路径）先查 [`crate::alloc_trace::armed`]，门关不进——
/// 产品构建整块 `#[cfg]` 摘除，测试构建零 armed 亦可直调（单测用独立键）。
pub(crate) fn record(entries: &[(&'static str, u64)]) {
    let mut guard = LEDGER.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(BTreeMap::new);
    for (key, value) in entries {
        map.insert(key, *value);
    }
}

/// dump 点快照（alloc-trace `snapshot_json` 的 `ledger` 段）。
///
/// 未录数（armed 前触发/空表）返回 `null`——分析侧判「无读数」而非空域集。
pub(crate) fn snapshot_value() -> serde_json::Value {
    let guard = LEDGER.lock().unwrap_or_else(|e| e.into_inner());
    let Some(map) = guard.as_ref() else {
        return serde_json::Value::Null;
    };
    if map.is_empty() {
        return serde_json::Value::Null;
    }
    let domains: Vec<serde_json::Value> = map
        .iter()
        .map(|(key, value)| serde_json::json!({ "domain": key, "bytes": value }))
        .collect();
    let sum: u64 = map
        .iter()
        .filter(|(key, _)| key.ends_with("_bytes"))
        .map(|(_, value)| value)
        .sum();
    serde_json::json!({ "domains": domains, "sum_bytes": sum })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 录数→快照回读：域键/值/求和恒等；未录数快照为 null；计数域不入 sum。
    #[test]
    fn record_then_snapshot_roundtrip() {
        assert_eq!(snapshot_value(), serde_json::Value::Null);
        record(&[
            ("t78_test_alpha_bytes", 100),
            ("t78_test_beta_bytes", 24),
            ("t78_test_count_entries", 3),
        ]);
        let snap = snapshot_value();
        assert_eq!(snap["sum_bytes"], 124); // 计数域不入求和
        let domains = snap["domains"].as_array().expect("domains array");
        assert_eq!(domains.len(), 3);
        assert_eq!(domains[0]["domain"], "t78_test_alpha_bytes");
        assert_eq!(domains[0]["bytes"], 100);
        assert_eq!(domains[1]["domain"], "t78_test_beta_bytes");
        assert_eq!(domains[1]["bytes"], 24);
        // 同键覆写（每帧最新口径）
        record(&[("t78_test_alpha_bytes", 50)]);
        assert_eq!(snapshot_value()["sum_bytes"], 74);
    }
}
