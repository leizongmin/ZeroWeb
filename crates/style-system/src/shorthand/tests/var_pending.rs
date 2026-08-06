//! R2873：`var()` 在简写值中的 pending-substitution 测试。
//!
//! driving：css-variables vars-font-shorthand-001 / vars-background-shorthand-001 /
//! wide-keyword-fallback-001。根因 = 简写在 cascade 前展开，含 `var()` 的值无法解析而被丢弃；
//! 修复 = 把简写各长属性标记为 pending，`var()` 解析后重新展开。

use super::super::*;

/// 辅助：清理环境，确保 kill-switch 默认开启（不设置 ZW_SHORTHAND_VAR=0）。
struct VarGuard;
impl VarGuard {
    fn new() -> Self {
        // 不移除环境变量（测试进程默认未设 = 开启）；仅作占位以便将来扩展。
        VarGuard
    }
}
impl Drop for VarGuard {
    fn drop(&mut self) {}
}

#[test]
fn test_font_shorthand_with_var_emits_pending_sentinels() {
    let _g = VarGuard::new();
    // `font: var(--foo)` 应为每个 font 长属性发一条 pending 标记（而非丢弃）。
    let out = expand_shorthands(&[("font".to_string(), "var(--foo)".to_string(), false, (0, 0, 0))]);
    let props: Vec<&str> = out.iter().map(|(p, _, _, _)| p.as_str()).collect();
    assert_eq!(
        props,
        ["font-style", "font-weight", "font-size", "line-height", "font-family"]
    );
    // 每条值都是指向 font 简写的 pending 标记。
    for (_, v, _, _) in &out {
        assert!(
            v.starts_with(ZWSP_SENTINEL_PREFIX),
            "value {v:?} not a pending sentinel"
        );
        assert!(v.contains("\x01font\x01"), "sentinel missing shorthand name");
        assert!(v.contains("var(--foo)"), "sentinel must carry raw value");
    }
}

#[test]
fn test_border_style_with_var_emits_pending_sentinels() {
    let _g = VarGuard::new();
    let out = expand_shorthands(&[(
        "border-style".to_string(),
        "var(--unknown, inherit)".to_string(),
        false,
        (0, 0, 0),
    )]);
    let props: Vec<&str> = out.iter().map(|(p, _, _, _)| p.as_str()).collect();
    assert_eq!(
        props,
        [
            "border-top-style",
            "border-right-style",
            "border-bottom-style",
            "border-left-style"
        ]
    );
    for (_, v, _, _) in &out {
        assert!(v.starts_with(ZWSP_SENTINEL_PREFIX));
    }
}

#[test]
fn test_background_with_var_emits_pending_sentinels() {
    let _g = VarGuard::new();
    let out = expand_shorthands(&[("background".to_string(), "var(--foo)".to_string(), false, (0, 0, 0))]);
    // background 8 长属性全部标记。
    assert_eq!(out.len(), 8);
    for (_, v, _, _) in &out {
        assert!(v.starts_with(ZWSP_SENTINEL_PREFIX));
    }
}

#[test]
fn test_non_var_font_shorthand_unaffected() {
    let _g = VarGuard::new();
    // 不含 var() 的简写走既有展开，不产生 pending 标记。
    let out = expand_shorthands(&[("font".to_string(), "16px/1.5 serif".to_string(), false, (0, 0, 0))]);
    for (_, v, _, _) in &out {
        assert!(
            !v.starts_with(ZWSP_SENTINEL_PREFIX),
            "non-var shorthand must not carry sentinel"
        );
    }
    // 应正确展开出 font-size / font-family。
    let props: Vec<&str> = out.iter().map(|(p, _, _, _)| p.as_str()).collect();
    assert!(props.contains(&"font-size"));
    assert!(props.contains(&"font-family"));
}

#[test]
fn test_pending_font_reexpands_after_var_resolution() {
    let _g = VarGuard::new();
    // 模拟 var() 已代入后的级联结果：font-family 仍是 pending 标记（携带已代入的 "0 Ahem"）。
    let mut resolved = std::collections::HashMap::new();
    let sentinel = format!("{ZWSP_SENTINEL_PREFIX}font\x010 Ahem");
    for lh in ["font-style", "font-weight", "font-size", "line-height", "font-family"] {
        resolved.insert(lh.to_string(), sentinel.clone());
    }
    let out = expand_pending_shorthands(resolved);
    // 重新展开 "0 Ahem" → font-size="0", font-family="Ahem"。
    assert_eq!(out.get("font-size").map(String::as_str), Some("0"));
    assert_eq!(out.get("font-family").map(String::as_str), Some("Ahem"));
    assert_eq!(out.get("font-weight").map(String::as_str), Some("normal"));
    assert_eq!(out.get("font-style").map(String::as_str), Some("normal"));
    assert_eq!(out.get("line-height").map(String::as_str), Some("normal"));
}

#[test]
fn test_pending_longhand_skipped_when_explicit_longhand_won() {
    let _g = VarGuard::new();
    // 场景：`font: var(--foo); font-size: 150px;` —— 级联后 font-size 为显式 "150px"（非标记），
    // 其余 font 长属性仍为 pending。重新展开只改写仍是标记的长属性，font-size 保持 150px。
    let mut resolved = std::collections::HashMap::new();
    let sentinel = format!("{ZWSP_SENTINEL_PREFIX}font\x010 Ahem");
    resolved.insert("font-style".to_string(), sentinel.clone());
    resolved.insert("font-weight".to_string(), sentinel.clone());
    resolved.insert("font-size".to_string(), "150px".to_string());
    resolved.insert("font-family".to_string(), sentinel.clone());
    let out = expand_pending_shorthands(resolved);
    assert_eq!(out.get("font-size").map(String::as_str), Some("150px"));
    assert_eq!(out.get("font-family").map(String::as_str), Some("Ahem"));
}

#[test]
fn test_pending_invalid_value_removes_longhand() {
    let _g = VarGuard::new();
    // 代入后值非法（展开失败）→ 该长属性按未声明处理（移除，由 compute 取 initial/inherit）。
    let mut resolved = std::collections::HashMap::new();
    let sentinel = format!("{ZWSP_SENTINEL_PREFIX}font\x01!!!garbage!!!");
    resolved.insert("font-family".to_string(), sentinel);
    let out = expand_pending_shorthands(resolved);
    // 非法值无法展开出 font-family → 移除。
    assert!(!out.contains_key("font-family"));
}

#[test]
fn test_non_pending_values_pass_through_unchanged() {
    let _g = VarGuard::new();
    let mut resolved = std::collections::HashMap::new();
    resolved.insert("color".to_string(), "red".to_string());
    resolved.insert("margin-top".to_string(), "10px".to_string());
    let out = expand_pending_shorthands(resolved);
    assert_eq!(out.get("color").map(String::as_str), Some("red"));
    assert_eq!(out.get("margin-top").map(String::as_str), Some("10px"));
}
