//! R4000（css-sizing-3 §intrinsic-sizes + SVG2 sizing + csswg #1801581）：
//! inline `<svg>`（非 outermost，HTML 文档内嵌）的 default object size 三件套。
//!
//! chromium 110 / Safari 16.4 实证语义（svg-intrinsic-size-006 案内注释 + 005）：
//! - **max-content 贡献**（intrinsic width）：无 abs width（attr/CSS）时——
//!   viewBox/CSS-ar-only → **0**；width %（attr/CSS）→ **300**；完全无来源 → **300**。
//! - **used size**：width auto（无 attr/CSS 宽）时——viewBox/ar-only → **0×0**；
//!   完全无来源 → **300×150**。width % 时 height 落 default **150** 且**不**与
//!   viewBox 比复合（006 ref：div 300 宽、svg 10%×150，绿色来自 div 背景）。
//!
//! R3935 警告遵从：默认尺寸**不走** attr 双 Some definite 路径——attr abs 值仍走
//! R1683/attr 既有路径；本模块只在 attr/CSS 均无 abs 值的分支给 default。
//!
//! kill-switch `ZW_SVG_DEFAULT_SIZE=0`。

use zero_css_parser::values::LengthValue;
use zero_dom::ElementData;
use zero_style_system::ComputedStyle;

/// CSS 默认对象尺寸（css-sizing-3 §5.1.1：300px × 150px）。
pub(crate) const SVG_DEFAULT_W: f32 = 300.0;
pub(crate) const SVG_DEFAULT_H: f32 = 150.0;

/// inline `<svg>` 的 max-content 宽度贡献（css-sizing-3 intrinsic width）。
///
/// 返回 `None` = 走既有 abs width 路径（attr/CSS abs——调用方按既有逻辑解析）。
pub(crate) fn svg_max_content_contribution(elem: &ElementData, style: &ComputedStyle) -> Option<f32> {
    if std::env::var("ZW_SVG_DEFAULT_SIZE").as_deref() == Ok("0") {
        return None;
    }
    let attr_w = elem.get_attribute("width");
    // 0 值 = 存在的 abs 声明（R34xx 零维语义），非「缺失」。
    let attr_abs_w = attr_w.as_deref().and_then(parse_attr_abs);
    let css_abs_w = matches!(style.width, LengthValue::Px(v) if v.is_finite() && v >= 0.0);
    // 既有 abs 路径优先——不动。
    if attr_abs_w.is_some() || css_abs_w {
        return None;
    }
    // (c) width 来源为百分比（attr 或 CSS，± 比例/viewBox）→ default 300
    //（chromium 006：% 宽 div 均 300 宽，比不改变贡献）。
    let width_pct =
        matches!(style.width, LengthValue::Percentage(_)) || attr_w.as_deref().is_some_and(|s| s.trim().ends_with('%'));
    if width_pct {
        return Some(SVG_DEFAULT_W);
    }
    // (a) viewBox / CSS aspect-ratio-only（无任何 width 来源）→ 0。
    if svg_has_ratio_source(elem, style) {
        return Some(0.0);
    }
    // (b) 完全无来源 → default 300。
    Some(SVG_DEFAULT_W)
}

/// inline `<svg>` 的 used size 补全（width auto 或 width % 时的 height/全尺寸语义）。
///
/// 返回 `Some((w, h))` = 应写入 taffy 的 definite 尺寸；`None` = 走既有路径。
/// width % 时不返回宽度（taffy 对 CB 已定时自行解析 %），仅由调用方钳 height。
pub(crate) fn svg_default_used_size(elem: &ElementData, style: &ComputedStyle) -> Option<(Option<f32>, f32)> {
    if std::env::var("ZW_SVG_DEFAULT_SIZE").as_deref() == Ok("0") {
        return None;
    }
    let attr_w = elem.get_attribute("width");
    let attr_h = elem.get_attribute("height");
    // 0 值 = 存在的 abs 声明（R34xx 零维语义），非「缺失」。
    let parse_abs = |v: &Option<String>| v.as_deref().and_then(parse_attr_abs);
    let css_abs = |v: &LengthValue| matches!(v, LengthValue::Px(x) if x.is_finite() && *x >= 0.0);
    // R4000b（csswg #6286）：attr height=0（退化比标记）+ attr width abs → 比退回
    // viewBox 推导（used = w × viewBox ratio）。
    if let Some(aw) = parse_abs(&attr_w)
        && attr_h.as_deref().map(str::trim) == Some("0")
        && let Some(ratio) = svg_ratio_value(elem, style)
    {
        return Some((Some(aw), aw / ratio));
    }
    // 既有 abs 路径（attr/CSS 任一 abs）优先——不动。
    if parse_abs(&attr_w).is_some() || parse_abs(&attr_h).is_some() || css_abs(&style.width) || css_abs(&style.height) {
        return None;
    }
    let width_pct =
        matches!(style.width, LengthValue::Percentage(_)) || attr_w.as_deref().is_some_and(|s| s.trim().ends_with('%'));
    // (a) viewBox / CSS-ar-only，width auto → **隐式 width:100%**（SVG 根缺省
    // width/height = 100%，chromium：body 块流 definite 宽下 fills、max-content
    // 语境贡献 0——与 (c) 同一条 % 路径）。R4002 勘察统一：(a) 与 (c) 同语义，
    // 006 max-content=0 由 contribution 谓词承载（intrinsic_sizing），used size
    // 走 % 解析 + 比推高（aspect-ratio-intrinsic-size-007 ref：viewBox 2:1 svg
    // 在 body → 784×392）。
    if !width_pct && svg_has_ratio_source(elem, style) {
        let ratio = svg_ratio_value(elem, style);
        return Some((None, -ratio.unwrap_or(SVG_DEFAULT_H)));
    }
    // (c) width %：高度由解析宽 × 比推导（viewBox/ar），无比 → default 150。
    //（006 ref：% 宽 div 高度 ≠ 150——比随解析宽生效；宽 % 解析归调用方容器语境。
    //  以 `h < 0` 承载比信号：|h| = ratio，调用方解析宽后 h_used = w_resolved / ratio。）
    if width_pct {
        let ratio = svg_ratio_value(elem, style);
        return Some((None, -ratio.unwrap_or(SVG_DEFAULT_H)));
    }
    // (b) 完全无来源 → default 300×150。
    Some((Some(SVG_DEFAULT_W), SVG_DEFAULT_H))
}

/// svg 的有效比例值（CSS aspect-ratio 优先，回退 viewBox w/h）。
pub(crate) fn svg_ratio_value(elem: &ElementData, style: &ComputedStyle) -> Option<f32> {
    if let Some(r) = style.aspect_ratio.filter(|r| r.is_finite() && *r > 0.0) {
        return Some(r);
    }
    let vb = elem
        .get_attribute("viewBox")
        .or_else(|| elem.get_attribute("viewbox"))?;
    let nums: Vec<&str> = vb.split([' ', ',']).filter(|t| !t.is_empty()).collect();
    if nums.len() != 4 {
        return None;
    }
    let vw: f32 = nums[2].parse().ok()?;
    let vh: f32 = nums[3].parse().ok()?;
    (vh > 0.0 && vw > 0.0).then_some(vw / vh)
}

/// SVG 尺寸 attr 值解析：剥离 `px` 后缀后按 f32（SVG2 允许 px 单位；% 由调用方分支）。
fn parse_attr_abs(v: &str) -> Option<f32> {
    let t = v.trim();
    if t.is_empty() || t.ends_with('%') {
        return None;
    }
    let t = t.strip_suffix("px").unwrap_or(t);
    t.parse::<f32>().ok().filter(|n| n.is_finite())
}

/// R4016：svg 的 attr/CSS 绝对固有高（height attr px 值 → CSS Px → None）。
///
/// R3929 replaced-collapse h 臂的固有高来源——`<svg height="50">`（仅 attr 高）的
/// abspos 盒塌 0 时，高应回填固有 50 而非 default 150（009/023/030：used h=50）。
/// 与 [`svg_default_used_size`] 的「attr/CSS abs 走既有路径」语义一致：本 helper 只
/// 读取不参与 gate，% 高（50%）不在此解析（CB 链归深域，返回 None 落 default 150）。
pub(crate) fn svg_attr_intrinsic_height(
    node_id: Option<zero_dom::NodeId>,
    doc: &zero_dom::Document,
    style: &ComputedStyle,
) -> Option<f32> {
    let id = node_id?;
    let node = doc.get(id)?;
    let zero_dom::NodeKind::Element(elem) = &node.kind else {
        return None;
    };
    if elem.local_name() != "svg" {
        return None;
    }
    if let Some(h) = elem.get_attribute("height").as_deref().and_then(parse_attr_abs) {
        return Some(h.max(0.0));
    }
    if let LengthValue::Px(v) = style.height {
        if v.is_finite() && v >= 0.0 {
            return Some(v as f32);
        }
    }
    None
}

/// svg 是否有比例来源（viewBox 或 CSS aspect-ratio）。
fn svg_has_ratio_source(elem: &ElementData, style: &ComputedStyle) -> bool {
    if style.aspect_ratio.is_some() {
        return true;
    }
    elem.get_attribute("viewBox")
        .or_else(|| elem.get_attribute("viewbox"))
        .is_some_and(|vb| {
            let nums: Vec<&str> = vb.split([' ', ',']).filter(|t| !t.is_empty()).collect();
            nums.len() == 4
                && nums
                    .iter()
                    .all(|n| n.parse::<f32>().map(|v| v.is_finite()).unwrap_or(false))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::Parser;

    fn setup(html: &str, css: &str) -> (ElementData, ComputedStyle) {
        let doc = zero_dom::parse_html(html);
        let svg_id = doc.get_elements_by_tag_name("svg").into_iter().next().expect("svg");
        let sheet = Parser::parse_stylesheet(css);
        let mut sys = zero_style_system::StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &[sheet]);
        let node = doc.get(svg_id).expect("node");
        match &node.kind {
            zero_dom::NodeKind::Element(e) => (e.clone(), styles.get(&svg_id).cloned().expect("style")),
            _ => unreachable!(),
        }
    }

    /// (a) viewBox-only → max-content 贡献 0；used = 隐式 width:100%（负值比
    /// 信号，taffy/IFC 对容器解析——R4002 勘察：body 块流 fills、max-content 0）。
    #[test]
    fn viewbox_only_contributes_zero() {
        let (elem, style) = setup(r#"<html><body><svg viewBox="0 0 1 1"></svg></body></html>"#, "");
        assert_eq!(svg_max_content_contribution(&elem, &style), Some(0.0));
        let (dw, dh) = svg_default_used_size(&elem, &style).expect("used");
        assert_eq!(dw, None, "ratio-only = 隐式 100% 宽，交容器解析");
        assert!(dh < 0.0, "负值承载比信号：{dh}");
    }

    /// (b) 完全无尺寸来源 → 贡献 300，used 300×150。
    #[test]
    fn no_dims_get_default_object_size() {
        let (elem, style) = setup(r#"<html><body><svg></svg></body></html>"#, "");
        assert_eq!(svg_max_content_contribution(&elem, &style), Some(300.0));
        assert_eq!(svg_default_used_size(&elem, &style), Some((Some(300.0), 150.0)));
    }

    /// (c) width %（attr/CSS）→ 贡献 300；used 高 = 比信号（负值承载）或 default。
    #[test]
    fn percent_width_contributes_default() {
        let (elem, style) = setup(
            r#"<html><body><svg width="10%" viewBox="0 0 1 1"></svg></body></html>"#,
            "",
        );
        assert_eq!(svg_max_content_contribution(&elem, &style), Some(300.0));
        let (dw, dh) = svg_default_used_size(&elem, &style).expect("used");
        assert_eq!(dw, None, "% 宽交 taffy/容器解析");
        assert!(dh < 0.0, "% 宽 + viewBox → 负值比信号：{dh}");
    }

    /// attr/CSS abs 值 → None（走既有路径，不干预）。
    #[test]
    fn abs_dims_take_existing_path() {
        let (elem, style) = setup(r#"<html><body><svg width="100" height="50"></svg></body></html>"#, "");
        assert_eq!(svg_max_content_contribution(&elem, &style), None);
        assert_eq!(svg_default_used_size(&elem, &style), None);

        let (elem2, style2) = setup(
            r#"<html><body><svg style="width: 100px"></svg></body></html>"#,
            "svg { width: 100px }",
        );
        assert_eq!(svg_max_content_contribution(&elem2, &style2), None);
    }

    /// 0px attr = 存在的 abs 声明（R34xx 零维语义）→ None 不干预。
    #[test]
    fn zero_px_attr_is_present_abs() {
        let (elem, style) = setup(r#"<html><body><svg width="0px" height="0px"></svg></body></html>"#, "");
        assert_eq!(svg_max_content_contribution(&elem, &style), None);
        assert_eq!(svg_default_used_size(&elem, &style), None);
    }

    /// R4000b（csswg #6286）：attr width abs + height=0（退化比）→ viewBox 比推导 used。
    #[test]
    fn zero_height_attr_falls_back_to_viewbox_ratio() {
        let (elem, style) = setup(
            r#"<html><body><svg viewBox="0 0 1 1" width="100" height="0"></svg></body></html>"#,
            "",
        );
        let (dw, dh) = svg_default_used_size(&elem, &style).expect("used");
        assert_eq!(dw, Some(100.0));
        assert!((dh - 100.0).abs() < 0.5, "viewBox 1:1 → h = 100/1：{dh}");
    }
}
