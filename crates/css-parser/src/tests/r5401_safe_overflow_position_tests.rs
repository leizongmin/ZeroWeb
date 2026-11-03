//! R5401：css-align-3 §overflow-values（`safe`/`unsafe` overflow-position 前缀）解析回归测试。
//!
//! 谱系：WPT `css/css-flexbox/flexbox-safe-overflow-position-001.html`（✗1.40% 隔离 →
//! ✓0.00% 真通过-可信）。旧 `parse_alignment` 只认裸关键字，`align-content: safe
//! flex-end` / `justify-content: safe center` / `align-items: safe center` 全部整条
//! 拒收 → 回退初始值，驱动案 SAFE 页全部按初始值摆位。
//!
//! 语法（css-align-3 §overflow-values）：`<overflow-position>? <alignment-position>`
//! ——`safe`/`unsafe` 前缀**仅**可与位置关键字组合（start/end/flex-start/flex-end/
//! center）；`safe stretch`、`safe space-between`、`safe baseline`、`safe normal`、
//! `safe auto` 按消费方 grammar 非法（guideline #1 共享 parser 不放宽消费方值域）。
//! `unsafe <position>` 与裸 `<position>` 同义（Unsafe 为默认），modifier 丢弃；
//! `safe` 前缀保留为 Safe* 变体由布局侧（taffy apply_alignment_fallback /
//! resolve_self_alignment_safety）在溢出时回退 start。
//! https://drafts.csswg.org/css-align-3/#overflow-values

use crate::values::{AlignmentValue, parse_alignment};

/// safe + 位置关键字组合逐变体解析（保留 Safe 前缀由布局侧消费）。
#[test]
fn test_safe_prefix_position_keywords_parse() {
    assert_eq!(parse_alignment("safe start"), Some(AlignmentValue::SafeStart));
    assert_eq!(parse_alignment("safe end"), Some(AlignmentValue::SafeEnd));
    assert_eq!(parse_alignment("safe flex-start"), Some(AlignmentValue::SafeFlexStart));
    assert_eq!(parse_alignment("safe flex-end"), Some(AlignmentValue::SafeFlexEnd));
    assert_eq!(parse_alignment("safe center"), Some(AlignmentValue::SafeCenter));
}

/// safe 前缀大小写不敏感（CSS 关键字 ASCII case-insensitive）。
#[test]
fn test_safe_prefix_case_insensitive() {
    assert_eq!(parse_alignment("SAFE CENTER"), Some(AlignmentValue::SafeCenter));
    assert_eq!(parse_alignment("Safe Flex-End"), Some(AlignmentValue::SafeFlexEnd));
    assert_eq!(parse_alignment("UNSAFE start"), Some(AlignmentValue::Start));
}

/// unsafe 前缀与裸位置关键字同义（Unsafe 为默认），modifier 丢弃。
#[test]
fn test_unsafe_prefix_folds_to_bare_keyword() {
    assert_eq!(parse_alignment("unsafe start"), Some(AlignmentValue::Start));
    assert_eq!(parse_alignment("unsafe end"), Some(AlignmentValue::End));
    assert_eq!(parse_alignment("unsafe flex-start"), Some(AlignmentValue::FlexStart));
    assert_eq!(parse_alignment("unsafe flex-end"), Some(AlignmentValue::FlexEnd));
    assert_eq!(parse_alignment("unsafe center"), Some(AlignmentValue::Center));
}

/// safe/unsafe 前缀仅可与位置关键字组合：分布值/基线/初始值关键字带前缀按
/// 消费方 grammar 非法，整条拒绝（不静默忽略 modifier）。
#[test]
fn test_overflow_prefix_with_non_position_keywords_rejected() {
    for bad in [
        "safe stretch",
        "safe space-between",
        "safe space-around",
        "safe space-evenly",
        "safe baseline",
        "safe last baseline",
        "safe normal",
        "safe auto",
        "unsafe stretch",
        "unsafe space-between",
        "unsafe baseline",
    ] {
        assert_eq!(
            parse_alignment(bad),
            None,
            "{bad} must be rejected (overflow-position only composes with self/position keywords)"
        );
    }
}

/// keyword token 边界（guideline #2）：`safe` 后必须跟空白，粘合形与残段整条拒绝。
#[test]
fn test_overflow_prefix_token_boundary() {
    assert_eq!(parse_alignment("safestart"), None);
    assert_eq!(parse_alignment("safe-start"), None);
    assert_eq!(parse_alignment("unsafestart"), None);
    assert_eq!(parse_alignment("safe"), None);
    assert_eq!(parse_alignment("unsafe"), None);
    // 前缀后多余尾段非法：`safe center extra` 整条拒绝，不忽略尾部输入。
    assert_eq!(parse_alignment("safe center extra"), None);
    // 多空白间隔仍合法（空白是 token 分隔符）。
    assert_eq!(parse_alignment("safe  center"), Some(AlignmentValue::SafeCenter));
    assert_eq!(parse_alignment("  safe center  "), Some(AlignmentValue::SafeCenter));
}

/// 裸关键字行为不变（驱动案 REF 页同款值 + 既有接受集回归锚）。
#[test]
fn test_bare_keywords_unchanged() {
    assert_eq!(parse_alignment("center"), Some(AlignmentValue::Center));
    assert_eq!(parse_alignment("flex-end"), Some(AlignmentValue::FlexEnd));
    assert_eq!(parse_alignment("space-between"), Some(AlignmentValue::SpaceBetween));
    assert_eq!(parse_alignment("normal"), Some(AlignmentValue::Normal));
    assert_eq!(parse_alignment("bogus"), None);
}
