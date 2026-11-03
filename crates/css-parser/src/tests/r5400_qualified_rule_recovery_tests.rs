//! R5400：qualified rule 规范恢复（前奏只到 `{`/EOF，`;`/`}` 不重同步）回归测试。
//!
//! 谱系：WPT `css/CSS2/fonts/font-family-invalid-characters-003.xht`（CSS 2.1
//! syndata.html#parsing-errors ✗99.96% 隔离 / ✗1.00% 全量 → ✓）双红源之一：
//! 规则 6 `#div3 { font-family: test}` 提前闭块后的孤儿段
//! `foo, Ahem; } body { background: red; }`——chromium 按 CSS Syntax L3 §5.3.2 把
//! 孤儿段连同 body 选择器整体并入非法 qualified rule 前奏（`;` / 游离 `}` 都是前奏
//! component value，不触发重同步），连块丢弃 → 整页零红；ZW 旧恢复在 `;` / `}`
//! 处重同步，`body {...}` 逃逸为顶层合法规则 → 全页红底。
//!
//! chromium oracle 实证（2026-11-03，headless computed style）：
//! - `garbage; body { red }` → body 不应用（`;` 不拯救后随规则）
//! - `junk1 junk2 } body { red }` → body 不应用（游离 `}` 同样不重同步）
//! - `foo, Ahem; bar { blue } body { red }` → body 应用（一个 `{...}` 块后重同步）
//! - `;;; #x { red } ;;;` → 规则不应用（覆盖测试 `test_stylesheet_with_garbage_tokens`
//!   断言随判例翻转，见 coverage_round6.rs 注记）
//!
//! kill-switch `ZW_CSS_QRULE_SWALLOW=0` 回退旧重同步语义（关断锚见包根
//! `tests/r5400_qrule_kill_switch.rs` 独立进程）。
//!
//! 配套修复（同轮）：`consume_nested_at_rule` 块体消耗改为按 `{}` 计深到匹配 `}`——
//! 旧 `skip_simple_block` 在开括号已消耗（深度从 0 起算）场景于首个内部 `}` 提前返回，
//! at-rule 自身闭 `}` 泄漏到外层流；旧顶层恢复静默丢弃掩盖了该泄漏，本规范恢复把游离
//! `}` 当前奏吞掉后级联吞掉后随规则（css-nesting/supports-rule ✗4.17% 双红源之二）。
//! 见 `test_nested_at_rule_extent_does_not_leak_close_brace`。

use crate::ast::{Declaration, Rule, StyleRule, SubclassSelector};
use crate::parser::Parser;

fn style_rules(css: &str) -> Vec<StyleRule> {
    let sheet = Parser::parse_stylesheet(css);
    sheet
        .rules
        .iter()
        .filter_map(|r| match r {
            Rule::Style(sr) => Some(sr.clone()),
            _ => None,
        })
        .collect()
}

fn first_decl<'a>(sr: &'a StyleRule, property: &str) -> Option<&'a Declaration> {
    sr.declarations.iter().find(|d| d.property == property)
}

/// P1 形：`;` 结尾垃圾 + 后随合法规则——垃圾前奏吞到块（含）整体丢弃，body 不存活。
#[test]
fn test_semicolon_junk_swallows_following_rule() {
    let rules = style_rules("garbage; body { background: red; }");
    assert!(
        rules.is_empty(),
        "junk prelude must swallow following rule to its block (syntax-3 5.3.2); got {} rules",
        rules.len()
    );
}

/// P2 形（驱动案孤儿段）：`test}` 提前闭块后，孤儿段连 body 选择器整体吞掉。
#[test]
fn test_orphan_segment_swallows_stray_brace_and_body_rule() {
    let rules = style_rules("#d { font-family: test}foo, Ahem; } body { background: red; }");
    assert_eq!(rules.len(), 1, "orphan segment + body must be swallowed; got {rules:?}");
    // 存活规则 = #d { font-family: test }（stray `}` 提前闭块，chromium 同语义）。
    let ff = first_decl(&rules[0], "font-family").expect("font-family decl survives");
    assert_eq!(ff.value, "test", "early-closed rule keeps font-family: test");
}

/// P4 形：垃圾吞到第一个 `{...}` 块后重同步——其后的合法 body 规则正常存活。
#[test]
fn test_recovery_resyncs_after_first_block() {
    let rules = style_rules("foo, Ahem; bar { color: blue; } body { background: red; }");
    assert_eq!(
        rules.len(),
        1,
        "junk+bar block dropped, body rule must survive after resync; got {rules:?}"
    );
    assert!(
        first_decl(&rules[0], "background").is_some(),
        "surviving rule must be the body rule"
    );
}

/// P3 形：游离 `}` 不重同步——`}` 与后随规则一起并入垃圾前奏吞掉。
#[test]
fn test_stray_rbrace_does_not_resync() {
    let rules = style_rules("junk1 junk2 } body { background: red; }");
    assert!(
        rules.is_empty(),
        "stray }} must not resync recovery; got {} rules",
        rules.len()
    );
}

/// 干净样式表零触碰守卫：恢复不吞合法规则（`spec` 恢复仅在畸形规则返回 None 后进入）。
#[test]
fn test_clean_stylesheet_unaffected() {
    let rules = style_rules(".a { color: red; } body { background: blue; } #i { color: green; }");
    assert_eq!(rules.len(), 3, "clean rules must all survive; got {rules:?}");
}

/// 嵌套 @规则 extent 守卫（回归 pin，driving: css-nesting/supports-rule ✗4.17%）：
/// `consume_nested_at_rule` 块体消耗须按 `{}` 计深到匹配 `}`——内容含嵌套规则时
/// 不得在首个内部 `}` 提前停（开括号已消耗、深度从 0 起算的 skip_simple_block 会
/// 使 at-rule 自身闭 `}` 泄漏到外层流；旧恢复静默丢弃掩盖，规范恢复把游离 `}`
/// 当前奏吞掉后级联吞掉 `.test-3` 整条）。后随规则必须完整存活。
#[test]
fn test_nested_at_rule_extent_does_not_leak_close_brace() {
    let sheet = Parser::parse_stylesheet(
        ".test { > .test-2 { color: green; } @supports (selector(> .test-2)) { > .test-2 { color: red; } } } \
         .test-3 { @supports (selector(&)) { & { color: green; } } }",
    );
    // 期望 5 条：.test 本体、`.test > .test-2`（green）、嵌套 @supports 提升体
    //（含 2 条）、.test-3 本体、`.test-3` 嵌套 @supports 提升体。
    assert_eq!(
        sheet.rules.len(),
        5,
        "nested @supports extent must not leak its close brace; got {:?}",
        sheet.rules
    );
    let has_test3 = sheet.rules.iter().any(|r| {
        matches!(r, Rule::Style(sr) if sr.selectors.iter().any(|s| {
            s.complex.parts.iter().any(|(c, _)| {
                c.subclass_selectors
                    .iter()
                    .any(|sc| matches!(sc, SubclassSelector::Class(n) if n == "test-3"))
            })
        }))
    });
    assert!(has_test3, ".test-3 rule must survive; got {:?}", sheet.rules);
}
