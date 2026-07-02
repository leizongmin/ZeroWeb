//! 时钟抽象（spec §8.4.1 `clock.rs`）。
//!
//! 动画驱动用整数毫秒；`FakeClock` 供 `ui/testing` 确定性推进（DC 要求 animation 需 fake clock）。

/// 时钟trait。
pub trait Clock {
    fn now_ms(&self) -> i64;
}

/// 可手动推进的假时钟（测试用）。
#[derive(Debug, Clone, Default)]
pub struct FakeClock {
    now_ms: i64,
}

impl FakeClock {
    pub fn new() -> FakeClock {
        FakeClock::default()
    }
    pub fn advance(&mut self, delta_ms: i64) {
        self.now_ms += delta_ms;
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> i64 {
        self.now_ms
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
}
