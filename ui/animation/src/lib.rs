//! # zero-ui-animation
//!
//! 动画与过渡（spec §8.4.1 `zero-ui-animation` / FR-016 / IF-010 `AnimationClock` /
//! §8.8 fake clock/tween/spring 测 / §8.4.1B sheet fling + reduced motion）。
//!
//! 提供：[`AnimationClock`]（IF-010）+ [`FakeClock`]（确定性测试）+ [`Curve`]/[`Tween`]（时长驱动）+
//! [`Spring`]（弹簧物理，fling/回弹）+ [`MotionPreference`]（reduced-motion 直接到终态）。

pub mod clock;
pub mod curve;
pub mod motion;
pub mod spring;
pub mod tween;

pub use clock::{AnimationClock, Clock, FakeClock};
pub use curve::{Curve, evaluate};
pub use motion::{MotionPreference, sample_tween, settle_spring, should_animate};
pub use spring::Spring;
pub use tween::Tween;
