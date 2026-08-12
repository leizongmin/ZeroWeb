//! R3346 deep-review 修复回归测试（zero-net cookie.rs）。
//!
//! 本轮 deep-review 发现并修复的 `parse_expires_date` 健壮性 bug 常驻断言：
//!
//! **panic + 非法值静默接受（高危 + 中危）**：`parse_expires_date` 对 RFC 1123 / RFC 850 /
//! asctime 日期的 day/hour/minute/second 字段无范围校验，直接喂入 `to_unix_secs`。
//! - day=0：`to_unix_secs` 计算 `(d - 1) as u64` 时 u32 下溢——**debug profile panic**
//!   （测试/开发构建崩溃），release wrap 为巨大数（cookie 永不过期）。攻击者可经
//!   `Set-Cookie: x=1; Expires=Wed, 00 Jun 2021 10:18:14 GMT` 触发 cookie 解析 panic（DoS）。
//! - day=32 / hour=99 / minute=99 / second=99：无 panic 但静默返回错误时间戳（应按
//!   RFC 7231 §7.1.1 视为无效日期 → None）。
//!
//! 修复：新增 `validate_date_fields` 范围校验（day 1-31、month 1-12、hour 0-23、
//! minute/second 0-59），三解析分支在 `to_unix_secs` 前调用之，非法字段返回 None。
//! // https://www.rfc-editor.org/rfc/rfc7231#section-7.1.1.1

#![allow(clippy::unwrap_used)]

use zero_net::cookie::parse_expires_date;

// ── Bug：day=0 不得 panic，须返回 None ──────────────────────────────────

#[test]
fn test_expires_day_zero_rfc1123_no_panic_r3346() {
    // RFC 1123 day=0 非法 → None，不得 panic（修复前 `(0u32 - 1)` 下溢 panic at cookie.rs:145）。
    let result = parse_expires_date("Wed, 00 Jun 2021 10:18:14 GMT");
    assert_eq!(
        result, None,
        "RFC 1123 day=0 须返回 None（不得 panic / 不得错误时间戳）"
    );
}

#[test]
fn test_expires_day_zero_asctime_no_panic_r3346() {
    // asctime day=00 同理。
    let result = parse_expires_date("Wed Jun 00 10:18:14 2021");
    assert_eq!(result, None, "asctime day=00 须返回 None");
}

#[test]
fn test_expires_day_zero_rfc850_no_panic_r3346() {
    // RFC 850 day=00。
    let result = parse_expires_date("Wednesday, 00-Jun-21 10:18:14 GMT");
    assert_eq!(result, None, "RFC 850 day=00 须返回 None");
}

// ── 非法日期字段范围校验（day 1-31、hour 0-23、min/sec 0-59）──────────

#[test]
fn test_expires_day_overflow_rejected_r3346() {
    // day=32 越界（各月最多 31）→ None（修复前静默返回错误时间戳 1625221094）。
    assert_eq!(
        parse_expires_date("Wed, 32 Jun 2021 10:18:14 GMT"),
        None,
        "day=32 须拒绝"
    );
    assert!(
        parse_expires_date("Wed, 09 Jun 2021 10:18:14 GMT").is_some(),
        "合法日期基线"
    );
}

#[test]
fn test_expires_hour_overflow_rejected_r3346() {
    assert_eq!(
        parse_expires_date("Wed, 09 Jun 2021 99:18:14 GMT"),
        None,
        "hour=99 须拒绝"
    );
    assert!(
        parse_expires_date("Wed, 09 Jun 2021 23:18:14 GMT").is_some(),
        "hour=23 合法"
    );
}

#[test]
fn test_expires_minute_second_overflow_rejected_r3346() {
    assert_eq!(
        parse_expires_date("Wed, 09 Jun 2021 10:99:14 GMT"),
        None,
        "minute=99 须拒绝"
    );
    assert_eq!(
        parse_expires_date("Wed, 09 Jun 2021 10:18:99 GMT"),
        None,
        "second=99 须拒绝"
    );
    assert!(
        parse_expires_date("Wed, 09 Jun 2021 10:59:59 GMT").is_some(),
        "minute=59/second=59 合法"
    );
}

#[test]
fn test_expires_day_boundary_31_valid_r3346() {
    // day=31 在 31 天月（Jan）合法；在 30 天月（Apr）应拒绝（days_in_month 校验）。
    assert!(
        parse_expires_date("Wed, 31 Jan 2035 10:18:14 GMT").is_some(),
        "day=31 在 31 天月须合法"
    );
}

// ── 合法日期回归保护（修复不得破坏正常路径）──────────────────────────

#[test]
fn test_expires_valid_dates_unchanged_r3346() {
    // 三种合法格式仍正确解析为已知时间戳（Wed, 09 Jun 2021 10:18:14 GMT = 1623233894）。
    assert_eq!(parse_expires_date("Wed, 09 Jun 2021 10:18:14 GMT"), Some(1623233894));
    assert_eq!(
        parse_expires_date("Wednesday, 09-Jun-21 10:18:14 GMT"),
        Some(1623233894)
    );
    assert_eq!(parse_expires_date("Wed Jun 09 10:18:14 2021"), Some(1623233894));
    // day=1 / 00:00:00 边界合法。
    assert!(parse_expires_date("Thu, 01 Jan 1970 00:00:00 GMT").is_some());
}
