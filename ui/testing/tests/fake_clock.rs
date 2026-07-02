//! 覆盖率补充：testing crate 的 FakeClock。

use zero_ui_testing::FakeClock;

#[test]
fn fake_clock_advances_and_reads() {
    let mut c = FakeClock::new();
    assert_eq!(c.now_ms(), 0);
    c.advance(16);
    c.advance(16);
    assert_eq!(c.now_ms(), 32);
}
