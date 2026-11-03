//! R5400 kill-switch 关断锚（独立进程）：`ZW_CSS_QRULE_SWALLOW=0` 回退 `;` / `}`
//! 重同步旧语义——`;;; div { color: red; }` 下 div 规则被 `;` 重同步救回（旧行为）。
//!
//! 进程内 OnceLock 不可复写（env 竞态，R3858/R5391 口径），故独立集成测试进程。

use zero_css_parser::Parser;
use zero_css_parser::ast::Rule;

#[test]
fn qrule_swallow_off_resyncs_at_semicolon() {
    // 显式置 0（进程即锚）：关断后旧恢复在首个 `;` 处返回，div 规则存活。
    // Rust 2024：set_var 为 unsafe（进程全局可变）。
    unsafe { std::env::set_var("ZW_CSS_QRULE_SWALLOW", "0") };
    let sheet = Parser::parse_stylesheet(";;; div { color: red; } ;;;");
    let n = sheet.rules.iter().filter(|r| matches!(r, Rule::Style(_))).count();
    assert_eq!(
        n, 1,
        "kill-switch off must restore semicolon resync (old behavior); got {n} style rules"
    );
}
