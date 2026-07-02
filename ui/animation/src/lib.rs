//! # zero-ui-animation
//!
//! 动画与过渡（spec §8.4.1 `zero-ui-animation` / FR-016）。
//!
//! M1 提供：fake clock（确定性测试）、缓动曲线、tween、reduced-motion 判定。
//! controller/spring/transition 在后续里程碑填实。

pub mod clock;
pub mod curve;
pub mod tween;

pub use clock::{Clock, FakeClock};
pub use curve::{Curve, evaluate};
pub use tween::Tween;
