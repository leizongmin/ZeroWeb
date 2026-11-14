//! `<position>` 写面 canonical 序列化。
//!
//! g47（css-values-5 §4.2 序列化注记 + object-position-valid 语料实钉）：
//! Chrome 写入面序列化为水平轴先行、缺省轴补 center 的双分量形（`10%` →
//! `10% center`、`top` → `center top`、`bottom right` → `right bottom`、
//! `bottom 10% right 20%` → `right 20% bottom 10%`）。供 shim 写入面
//! （`__zw_gradient_normalize_decl` object-position 臂）canonical 化。

use super::parse_extended_visual::{BackgroundEdge, BackgroundPositionValue, parse_position_strict};

/// canonical 序列化 `<position>` 声明值（严格文法，css-values-5 §4.2）。
///
/// 解析失败 → `None`（调用方按非法拒收）；解析成功 → 水平先行双分量串。
/// 含 `calc()` 数学函数或非 px 长度等无法保真重排的原子时原文透传
///（`Some(原文)`），保持最小实现、不引入串化漂移。
pub fn canonicalize_position_decl(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let parsed = parse_position_strict(trimmed)?;
    match canonical_string(&parsed) {
        Some(s) => Some(s),
        // 透传：无法保真重排（calc / 非 px 长度）不等于非法
        None => Some(trimmed.to_string()),
    }
}

/// 双分量 canonical 串（水平 垂直，缺省轴补 center）。
fn canonical_string(v: &BackgroundPositionValue) -> Option<String> {
    match v {
        // 数学函数延迟解析（R2313），原文透传
        BackgroundPositionValue::Calc(_) => None,
        BackgroundPositionValue::Center => Some("center center".to_string()),
        BackgroundPositionValue::Left => Some("left center".to_string()),
        BackgroundPositionValue::Right => Some("right center".to_string()),
        BackgroundPositionValue::Top => Some("center top".to_string()),
        BackgroundPositionValue::Bottom => Some("center bottom".to_string()),
        // g47：逻辑关键字 specified 保形（`x-start` → `x-start center`、
        // `y-start` → `center y-start`，css-values/position tentative 语料钉）
        BackgroundPositionValue::XStart => Some("x-start center".to_string()),
        BackgroundPositionValue::XEnd => Some("x-end center".to_string()),
        BackgroundPositionValue::YStart => Some("center y-start".to_string()),
        BackgroundPositionValue::YEnd => Some("center y-end".to_string()),
        BackgroundPositionValue::Percent(p) => Some(format!("{}% center", p)),
        BackgroundPositionValue::Length(super::types::LengthValue::Px(px)) => Some(format!("{}px center", px)),
        BackgroundPositionValue::Length(_) => None,
        BackgroundPositionValue::TwoValue(h, v2) => Some(format!("{} {}", atom(h)?, atom(v2)?)),
        // 单轴缘偏移对不是完整 <position>（解析面不会返回此形），防御性透传
        BackgroundPositionValue::EdgeOffset { .. } => None,
    }
}

/// 单原子串化（关键字 / 百分比 / px 长度 / 缘偏移对）。
fn atom(v: &BackgroundPositionValue) -> Option<String> {
    match v {
        BackgroundPositionValue::Center => Some("center".to_string()),
        BackgroundPositionValue::Left => Some("left".to_string()),
        BackgroundPositionValue::Right => Some("right".to_string()),
        BackgroundPositionValue::Top => Some("top".to_string()),
        BackgroundPositionValue::Bottom => Some("bottom".to_string()),
        BackgroundPositionValue::XStart => Some("x-start".to_string()),
        BackgroundPositionValue::XEnd => Some("x-end".to_string()),
        BackgroundPositionValue::YStart => Some("y-start".to_string()),
        BackgroundPositionValue::YEnd => Some("y-end".to_string()),
        BackgroundPositionValue::Percent(p) => Some(format!("{}%", p)),
        BackgroundPositionValue::Length(super::types::LengthValue::Px(px)) => Some(format!("{}px", px)),
        BackgroundPositionValue::EdgeOffset { side, offset } => {
            let side = match side {
                BackgroundEdge::Left => "left",
                BackgroundEdge::Right => "right",
                BackgroundEdge::Top => "top",
                BackgroundEdge::Bottom => "bottom",
                BackgroundEdge::XStart => "x-start",
                BackgroundEdge::XEnd => "x-end",
                BackgroundEdge::YStart => "y-start",
                BackgroundEdge::YEnd => "y-end",
            };
            Some(format!("{} {}", side, atom(offset)?))
        }
        // 嵌套 TwoValue / calc / 非 px 长度无法保真 → 上层透传
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::canonicalize_position_decl;

    /// object-position-valid 全 18 案语料钉（tests/wpt-runner/wpt-data
    /// /css/css-images/parsing/object-position-valid.html sound/canonical 数组）。
    #[test]
    fn test_canonicalize_position_corpus_pins() {
        let cases = [
            ("10%", "10% center"),
            ("right 30% top 60px", "right 30% top 60px"),
            ("-20% -30px", "-20% -30px"),
            ("30px center", "30px center"),
            ("40px top", "40px top"),
            ("bottom 10% right 20%", "right 20% bottom 10%"),
            ("bottom right", "right bottom"),
            ("center 50px", "center 50px"),
            ("center bottom", "center bottom"),
            ("center left", "left center"),
            ("left", "left center"),
            ("left bottom", "left bottom"),
            ("left center", "left center"),
            ("right 40%", "right 40%"),
            ("top", "center top"),
            ("top center", "center top"),
            ("center", "center center"),
            ("center center", "center center"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                canonicalize_position_decl(input),
                Some(expected.to_string()),
                "{input} canonical 应为 {expected}"
            );
        }
    }

    /// g47：逻辑关键字 17 形语料钉（css/css-values/position/position-valid.tentative.html）。
    #[test]
    fn test_canonicalize_position_logical_keywords() {
        let cases = [
            ("x-start", "x-start center"),
            ("x-start 10px", "x-start 10px"),
            ("x-start 10%", "x-start 10%"),
            ("x-start 10% top 20px", "x-start 10% top 20px"),
            ("top 20px x-start 10%", "x-start 10% top 20px"),
            ("x-end", "x-end center"),
            ("x-end 10%", "x-end 10%"),
            ("x-end 10px top 20px", "x-end 10px top 20px"),
            ("top 20px x-end 10px", "x-end 10px top 20px"),
            ("y-start", "center y-start"),
            ("10px y-start", "10px y-start"),
            ("left 10px y-start 20%", "left 10px y-start 20%"),
            ("y-start 20px left 10px", "left 10px y-start 20px"),
            ("y-end", "center y-end"),
            ("10px y-end", "10px y-end"),
            ("left 10px y-end 20%", "left 10px y-end 20%"),
            ("y-end 20% left 10px", "left 10px y-end 20%"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                canonicalize_position_decl(input),
                Some(expected.to_string()),
                "{input} logical canonical 应为 {expected}"
            );
        }
    }

    /// 非法形（g47 收紧组）→ None；calc / 非 px 长度 → 原文透传。
    #[test]
    fn test_canonicalize_position_invalid_and_passthrough() {
        // g47 收紧拒收形（object-position-invalid 语料组）
        for invalid in [
            "auto",
            "1px 2px 3px",
            "left right",
            "bottom 10%",
            "bottom 10% top 20%",
            "center left 1px",
            "right 3% center",
            "left 4px top",
            "right top 5px",
            "bottom 7% left",
            "bottom right 8%",
            // <position> 无 3 值形（css-values-5 §4.2 注记）；逻辑关键字不收
            // x-start 与物理关键字混搭（同轴重复）
            "x-start left",
            "top bottom",
        ] {
            assert_eq!(canonicalize_position_decl(invalid), None, "{invalid} 应拒收");
        }
        // calc 原文透传
        assert_eq!(
            canonicalize_position_decl("calc(10px + 2%)"),
            Some("calc(10px + 2%)".to_string())
        );
        // 非 px 长度透传（最小实现不串化非 px 单位）
        assert_eq!(canonicalize_position_decl("1.5em left"), Some("1.5em left".to_string()));
    }
}
