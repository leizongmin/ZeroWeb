//! 时钟抽象（spec §8.4.1 `clock.rs` / IF-010 `AnimationClock`）。
//!
//! 动画驱动用整数毫秒；`FakeClock` 供 `ui/testing` 确定性推进（DC 要求 animation 需 fake clock）。

use std::time::Duration;

/// 毫秒时钟（轻量，动画采样用）。
pub trait Clock {
    fn now_ms(&self) -> i64;
}

/// 动画时钟（spec IF-010 `AnimationClock`）。
///
/// `now` 供动画读取当前时间；`request_frame` 声明「还需下一帧」（宿主据此调度重绘）。
/// 真实后端由 winit 事件循环驱动；测试用 [`FakeClock`] 确定性推进。
pub trait AnimationClock {
    fn now(&self) -> Duration;
    fn request_frame(&mut self);
}

/// 可手动推进的假时钟（测试用）。
#[derive(Debug, Clone, Default)]
pub struct FakeClock {
    now_ms: i64,
    frame_requests: u64,
}

impl FakeClock {
    pub fn new() -> FakeClock {
        FakeClock::default()
    }
    pub fn advance(&mut self, delta_ms: i64) {
        self.now_ms += delta_ms;
    }

    /// 累计 `request_frame` 调用次数（断言动画是否请求了下一帧）。
    pub fn frame_requests(&self) -> u64 {
        self.frame_requests
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> i64 {
        self.now_ms
    }
}

impl AnimationClock for FakeClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.now_ms.max(0) as u64)
    }
    fn request_frame(&mut self) {
        self.frame_requests += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_clock_advances() {
        let mut c = FakeClock::new();
        assert_eq!(c.now_ms(), 0);
        c.advance(16);
        c.advance(16);
        assert_eq!(c.now_ms(), 32);
    }

    #[test]
    fn animation_clock_now_and_request_frame() {
        // IF-010：now 返回 Duration；request_frame 累计帧请求。
        let mut c = FakeClock::new();
        assert_eq!(c.now(), Duration::from_millis(0));
        c.advance(100);
        assert_eq!(c.now(), Duration::from_millis(100));
        assert_eq!(c.frame_requests(), 0);
        c.request_frame();
        c.request_frame();
        assert_eq!(c.frame_requests(), 2);
    }
}
