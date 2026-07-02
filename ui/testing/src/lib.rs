//! # zero-ui-testing
//!
//! 测试工具（spec §8.4.1 `zero-ui-testing` / FR-016 / DC-14 CI snapshot）。
//!
//! - [`scene_snapshot`]：把 Scene 序列化为确定性字符串（golden test / 回归）。
//! - [`semantics_snapshot`]：把 a11y 树序列化为确定性字符串。
//! - [`FakeClock`]：确定性推进时间（动画/调度）。

pub use scene_snapshot::snapshot_scene;
pub use semantics_snapshot::snapshot_semantics;

pub mod scene_snapshot;
pub mod semantics_snapshot;

/// 确定性假时钟（与 ui/animation::FakeClock 等价的独立实现，避免 testing 依赖 animation）。
#[derive(Debug, Clone, Default)]
pub struct FakeClock {
    now_ms: i64,
}

impl FakeClock {
    pub fn new() -> FakeClock {
        FakeClock::default()
    }
    pub fn now_ms(&self) -> i64 {
        self.now_ms
    }
    pub fn advance(&mut self, delta_ms: i64) {
        self.now_ms += delta_ms;
    }
}
