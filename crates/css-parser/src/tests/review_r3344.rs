//! R3344 deep-review 修复回归测试（zero-css-parser）。
//!
//! 本轮 deep-review 发现并修复的三处真 bug 的常驻断言：
//!
//! 1. **panic（高危）**：`eval_calc`/`eval_calc_with_context` 对 `clamp(MIN, VAL, MAX)`
//!    在 `MIN > MAX` 时调用 `f64::clamp(min, max)`——std 在 `min > max` 或含 NaN 时
//!    **panic**。calc 长度走 style-system 计算样式热路径（computed.rs），任意页面作者/
//!    攻击者 CSS `calc(clamp(100px, 50px, 10px))` 即触发渲染进程 panic。改用 spec 退化
//!    公式 `max(MIN, min(VAL, MAX))`（min>max 时回退到 MIN，无 panic）。
//!    // https://www.w3.org/TR/css-values-3/#calc-range
//!
//! 2. **数据丢失（中危）**：tokenizer `consume_number` 科学计数法分支仅在 `e` 后跟
//!    `digit|+|-` 即吞 `e`（+可选符号），若符号后无 digit（`1e+`/`1e-`/`1e`）则
//!    `num_str="1e+".parse()` 失败 → `unwrap_or(0.0)` 把整段数字静默吞成 `0`。
//!    CSS Syntax §4.3.12 要求 `e` 后须跟 `[+-]? digit` 才属 numeric token；否则 `e`
//!    不属数字（`1e+` 应产 `Number(1)` + `Delim('e')` + `Delim('+')`）。改前置校验
//!    符号后是否真有 digit。
//!    // https://drafts.csswg.org/css-syntax-3/#consume-numeric-token
//!
//! 3. **解析不一致（中危）**：`parse_hex_color` 3/4 位 hex 用 `hex_char_to_byte`
//!    （`u8::from_str_radix(...).unwrap_or(0)`）吞非法 hex 字符为 0，而 6/8 位用
//!    `.ok()?` 拒绝——`#G00`（3 位）误返回黑色，`#GGGGGG`（6 位）正确拒绝。CSS Color
//!    规定 `#` 后须全为 hex digit，非法 hex 颜色应拒绝。改 `hex_char_to_byte` 返回
//!    `Option<u8>`，3/4 位路径遇非法字符返回 `None`。
//!    // https://drafts.csswg.org/css-color-4/#hex-notation

#![allow(clippy::float_cmp)]

use crate::tokenizer::{Token, Tokenizer};
use crate::values::color::parse_color;
use crate::values::{eval_calc, parse_calc};

// ── Bug 1：calc clamp MIN>MAX 不再 panic ────────────────────────────────

#[test]
fn test_calc_clamp_inverted_range_no_panic_r3344() {
    // MIN(100) > MAX(10)：spec 退化为 max(MIN, min(VAL, MAX)) = max(100, min(50,10)) = 100。
    // 修复前：f64::clamp(100.0, 10.0) panic。
    let expr = parse_calc("calc(clamp(100px, 50px, 10px))").expect("clamp parse");
    let result = eval_calc(&expr, Some(100.0)).expect("eval");
    assert_eq!(result, 100.0, "inverted clamp MIN>MAX 须退化为 MIN，不得 panic");
}

#[test]
fn test_calc_clamp_normal_range_r3344() {
    // 正常范围 clamp(10, 50, 100) = 50——确保修复未破坏正常路径。
    let expr = parse_calc("calc(clamp(10px, 50px, 100px))").expect("clamp parse");
    let result = eval_calc(&expr, Some(100.0)).expect("eval");
    assert_eq!(result, 50.0);
}

#[test]
fn test_calc_clamp_val_below_min_r3344() {
    let expr = parse_calc("calc(clamp(10px, 5px, 100px))").expect("clamp parse");
    let result = eval_calc(&expr, Some(100.0)).expect("eval");
    assert_eq!(result, 10.0, "val<min → min");
}

#[test]
fn test_calc_clamp_val_above_max_r3344() {
    let expr = parse_calc("calc(clamp(10px, 999px, 100px))").expect("clamp parse");
    let result = eval_calc(&expr, Some(100.0)).expect("eval");
    assert_eq!(result, 100.0, "val>max → max");
}

// ── Bug 2：科学计数法 e 后无 digit 不再吞数字 ────────────────────────────

#[test]
fn test_scientific_notation_e_without_digit_r3344() {
    // `1e+`（EOF）：`e` 后符号无 digit → `e` 不参与科学计数法。CSS Syntax 中 `e` 仍为合法
    // ident-start，故 token 化为 Dimension(1, "e") + Delim('+')——**关键不变量：数字值保留
    // 为 1，不被吞成 0**（修复前 num_str="1e+" parse 失败 → Number(0.0)，数据丢失）。
    let toks: Vec<Token> = Tokenizer::new("1e+").collect_tokens();
    let first_val = match toks.first() {
        Some(Token::Number(n)) => *n,
        Some(Token::Dimension(n, _)) => *n,
        other => panic!("1e+ 首须为 Number 或 Dimension(1)，实际 {:?}", other),
    };
    assert!(
        (first_val - 1.0).abs() < 1e-9,
        "1e+ 数值须保留为 1（不得吞成 0），实际 {:?}",
        toks
    );
}

#[test]
fn test_scientific_notation_e_minus_without_digit_r3344() {
    // `1e-x`：符号后非 digit → 数字值保留为 1。
    let toks: Vec<Token> = Tokenizer::new("1e-x").collect_tokens();
    let first_val = match toks.first() {
        Some(Token::Number(n)) => *n,
        Some(Token::Dimension(n, _)) => *n,
        other => panic!("1e-x 首须为 Number 或 Dimension(1)，实际 {:?}", other),
    };
    assert!((first_val - 1.0).abs() < 1e-9, "1e-x 数值须保留为 1，实际 {:?}", toks);
}

#[test]
fn test_scientific_notation_bare_e_r3344() {
    // `1e`（EOF）：`e` 后无字符 → 数字值保留为 1。
    let toks: Vec<Token> = Tokenizer::new("1e").collect_tokens();
    let first_val = match toks.first() {
        Some(Token::Number(n)) => *n,
        Some(Token::Dimension(n, _)) => *n,
        other => panic!("1e 首须为 Number 或 Dimension(1)，实际 {:?}", other),
    };
    assert!((first_val - 1.0).abs() < 1e-9, "1e 数值须保留为 1，实际 {:?}", toks);
}

#[test]
fn test_scientific_notation_digit_after_e_preserved_r3344() {
    // `1e5x`：合法科学计数法（e 后 digit）→ 数值 = 100000，`x` 作单位。
    // 修复不得破坏「e 后有 digit 时正常消费」。
    let toks: Vec<Token> = Tokenizer::new("1e5x").collect_tokens();
    let first_val = match toks.first() {
        Some(Token::Number(n)) => *n,
        Some(Token::Dimension(n, _)) => *n,
        other => panic!("1e5x 首须为 Number 或 Dimension(100000)，实际 {:?}", other),
    };
    assert!(
        (first_val - 100000.0).abs() < 1e-6,
        "1e5x 数值须为 100000，实际 {:?}",
        toks
    );
}

#[test]
fn test_scientific_notation_valid_still_works_r3344() {
    // 合法科学计数法不得被破坏。
    let toks: Vec<Token> = Tokenizer::new("1e3").collect_tokens();
    assert!(
        matches!(toks.first(), Some(Token::Number(n)) if (*n - 1000.0).abs() < 1e-9),
        "1e3 须为 Number(1000)，实际 {:?}",
        toks
    );
    let toks: Vec<Token> = Tokenizer::new("1.5e+2").collect_tokens();
    assert!(
        matches!(toks.first(), Some(Token::Number(n)) if (*n - 150.0).abs() < 1e-9),
        "1.5e+2 须为 Number(150)，实际 {:?}",
        toks
    );
    let toks: Vec<Token> = Tokenizer::new("2E-1").collect_tokens();
    assert!(
        matches!(toks.first(), Some(Token::Number(n)) if (*n - 0.2).abs() < 1e-9),
        "2E-1 须为 Number(0.2)，实际 {:?}",
        toks
    );
}

// ── Bug 3：3/4 位 hex 非法字符拒绝，与 6/8 位一致 ───────────────────────

#[test]
fn test_hex_invalid_char_3digit_rejected_r3344() {
    // #G00：G 非 hex digit → 须拒绝（None）。修复前：unwrap_or(0) → 黑色。
    assert_eq!(parse_color("#G00"), None, "#G00 非 hex digit 须拒绝");
    assert_eq!(parse_color("#00Z"), None, "#00Z 非 hex digit 须拒绝");
}

#[test]
fn test_hex_invalid_char_4digit_rejected_r3344() {
    assert_eq!(parse_color("#G000"), None, "#G000 4 位非法须拒绝");
    assert_eq!(parse_color("#FFFx"), None, "#FFFx 4 位非法须拒绝");
}

#[test]
fn test_hex_6digit_invalid_consistency_r3344() {
    // 6 位仍正确拒绝（回归保护）。
    assert_eq!(parse_color("#GGGGGG"), None);
}

#[test]
fn test_hex_valid_colors_still_work_r3344() {
    // 合法 hex 不得被破坏。
    assert!(parse_color("#000").is_some(), "#000 须合法");
    assert!(parse_color("#fff").is_some(), "#fff 须合法");
    assert!(parse_color("#00ff88").is_some(), "#00ff88 须合法");
    assert!(parse_color("#0123").is_some(), "#0123 4 位须合法");
    assert!(parse_color("#01234567").is_some(), "#01234567 8 位须合法");
}
