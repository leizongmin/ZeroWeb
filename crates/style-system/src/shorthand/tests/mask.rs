use super::super::*;

// ── mask 简写（css-masking-1 §3.10）钉测 ─────────────────────────────
// 驱动页：css/css-masking/mask-shorthand-subproperties-reset.html（§3.10
// reset 语义——mask 声明重置全部 mask 长手与 mask-border-* 五长手）。

#[test]
/// 单 url 形展开：`url("#mask")` → mask-image 保形 + reset 全名单投影
/// （mask 长手初值 + mask-border-* 五长手初值，共 13 声明）。
fn test_mask_single_url_expands_with_reset() {
    let result = expand_one("mask", "url(\"#mask\")", false, (0, 0, 1));
    assert_eq!(result.len(), 13);
    assert_eq!(result[0].0, "mask-image");
    assert_eq!(result[0].1, "url(\"#mask\")");
    let names: Vec<&str> = result.iter().map(|d| d.0.as_str()).collect();
    assert_eq!(
        names,
        [
            "mask-image",
            "mask-mode",
            "mask-position",
            "mask-size",
            "mask-repeat",
            "mask-origin",
            "mask-clip",
            "mask-composite",
            "mask-border-source",
            "mask-border-slice",
            "mask-border-width",
            "mask-border-outset",
            "mask-border-repeat",
        ]
    );
    // mask-border-* 初值与 border-image 简写不同（slice 0 / width auto）。
    assert_eq!(result[8].1, "none");
    assert_eq!(result[9].1, "0");
    assert_eq!(result[10].1, "auto");
    assert_eq!(result[11].1, "0");
    assert_eq!(result[12].1, "stretch");
}

#[test]
/// none 与渐变单值形接受。
fn test_mask_none_and_gradient() {
    let none = expand_one("mask", "none", false, (0, 0, 1));
    assert_eq!(none.len(), 13);
    assert_eq!(none[0].1, "none");
    let grad = expand_one("mask", "linear-gradient(45deg, red, blue)", false, (0, 0, 1));
    assert_eq!(grad.len(), 13);
    assert_eq!(grad[0].1, "linear-gradient(45deg, red, blue)");
}

#[test]
/// 组合形与多层形拒收（FIXME(mask-shorthand-grammar)——展开失败=声明丢弃）。
fn test_mask_composite_and_multi_layer_rejected() {
    assert!(expand_one("mask", "url(a) 10px 20px", false, (0, 0, 1)).is_empty());
    assert!(expand_one("mask", "url(a) repeat", false, (0, 0, 1)).is_empty());
    assert!(expand_one("mask", "url(a), url(b)", false, (0, 0, 1)).is_empty());
    assert!(expand_one("mask", "10px 20px / 50%", false, (0, 0, 1)).is_empty());
}
