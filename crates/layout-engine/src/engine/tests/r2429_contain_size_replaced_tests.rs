//! R2429：`contain: size` 替换元素固有尺寸忽略回归测试。
//!
//! CSS Containment 1：`contain: size` 使元素按「无内容」sized，替换元素固有尺寸须忽略
//!（intrinsic → 0）。`apply_replaced_element_sizing`（tree.rs）旧实现无条件用固有尺寸覆盖
//! converter 的 contain:size→0，破坏 size containment。fix：contain:size 时早返回，让
//! converter（mod.rs:123 `contain.has_size()`，含 contain-intrinsic-size 覆盖）生效。
//! driving：css-contain/contain-size-013（`<img contain:size padding:50>` 固有 60×60 应
//! padding-only=100×100，非 160×160）。承接 R2427（让 `/css/...` 图片加载暴露此 bug）+
//! R2428（aspect_ratio sizing）。
use std::sync::Arc;

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout_with_img_intrinsic(html: &str, intrinsic: (f32, f32)) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut sizes = std::collections::HashMap::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        sizes.insert(img_id, intrinsic);
    }
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute_with_img_intrinsic(
        &doc,
        &styles,
        sizes,
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    (doc, Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone()))
}

fn find_img<'a>(r: &'a LayoutBox, d: &zero_dom::Document) -> Option<&'a LayoutBox> {
    if r.node_id.is_some_and(|nid| {
        d.get(nid)
            .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "img"))
    }) {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_img(c, d) {
            return Some(b);
        }
    }
    None
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// `contain:size` + padding:50 + 固有 60×60 → box 100×100（padding-only，固有忽略）。
/// driving: css-contain/contain-size-013。
#[test]
fn test_contain_size_img_ignores_intrinsic() {
    let html = r#"<html><body style="margin:0">
<img style="contain: size; padding: 50px; background: green" src="x.png">
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 100.0),
        "contain:size: box = padding 50×2 = 100 wide (intrinsic ignored); got {}",
        img.width
    );
    assert!(
        approx(img.height, 100.0),
        "contain:size: box = padding 50×2 = 100 tall; got {}",
        img.height
    );
}

/// 对照：无 contain:size → 固有 60×60 + padding 50×2 = 160×160（fix 不破坏既有替换元素 sizing）。
#[test]
fn test_no_contain_size_img_uses_intrinsic() {
    let html = r#"<html><body style="margin:0">
<img style="padding: 50px; background: green" src="x.png">
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 160.0),
        "no contain:size: 60 intrinsic + 50×2 padding = 160; got {}",
        img.width
    );
    assert!(
        approx(img.height, 160.0),
        "no contain:size: 160 tall; got {}",
        img.height
    );
}

/// R4055（css-contain-1 §containment-size + HTML §attributes-for-embedded-content-and-images）：
/// contain:size 只抑制 content-based 尺寸；attr 双值经 `aspect-ratio: auto w/h` 提供
/// transferred 比例（非 content 尺度）——CSS 宽 definite + CSS 高 auto 时高 = 宽 × attr 比。
/// driving: css-contain/contain-size-replaced-007（60×60 attr + width:100px + height:auto
/// + contain:size 应 100×100，非 100×0）。布局层窄臂不经 computed aspect_ratio（R4055-N 教训）。
#[test]
fn test_contain_size_img_attr_ratio_transferred() {
    let html = r#"<html><body style="margin:0">
<img src="x.png" width="60" height="60" style="width: 100px; height: auto; contain: size">
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 100.0),
        "contain:size + attr ratio: width = CSS 100px; got {}",
        img.width
    );
    assert!(
        approx(img.height, 100.0),
        "contain:size + attr ratio: height = 100 × (60/60) = 100 (transferred, non-content); got {}",
        img.height
    );
}

/// R2429 守卫不破坏：contain:size + 无 attr（仅 decoded 固有）仍按 padding-only 100×100
///——attr 比窄臂仅消费 HTML width/height 属性，decoded 固有尺寸不参与（013 语义不变）。
#[test]
fn test_contain_size_no_attr_still_padding_only() {
    let html = r#"<html><body style="margin:0">
<img src="x.png" style="width: 100px; height: auto; contain: size; padding: 50px">
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let img = find_img(&root, &doc).expect("img");
    assert!(
        approx(img.width, 200.0),
        "no attr: CSS width 100 definite + padding 50×2 = 200 box wide; got {}",
        img.width
    );
    assert!(
        approx(img.height, 100.0),
        "no attr: contain:size 抑制 content-based 高（decoded 固有不传比）→ content 高 0 + padding 50×2 = 100; got {}",
        img.height
    );
}

/// R4060（css-contain-1 §containment-size）：contain:size **button** + block 子——
/// button 的 BFC auto-height「含浮动/流内后代」重算（float_positioning）不得覆盖
/// converter 的 CIS-or-0 definite 高（旧实现重算把 34 撑到 117）。chromium 对
/// contained button 按规范折叠内容（UA button 无 min-height 强制），与 select 的
/// 「控件默认高」豁免语义不同——R4034b 表单豁免收窄为 input/select/textarea。
/// driving: css-contain/contain-size-button-002。
#[test]
fn r4060_contain_size_button_collapses_to_chrome() {
    let html = r#"<html><body style="margin:0">
<button style="contain: size; margin: 0; border: 1em solid green"><div style="height: 100px; width: 100px;">inner</div></button>
</body></html>"#;
    let (doc, root) = layout_with_img_intrinsic(html, (60.0, 60.0));
    let btn = find_button(&root, &doc).expect("button");
    // 内容高 0 → border-box 高 = 32（1em 边框 ×2）+ UA padding 2 ≈ 34。
    assert!(
        (btn.height - 34.0).abs() < 1.0,
        "contain:size button 应塌到 chrome-only ≈34（内容 100 不泄漏），got {}",
        btn.height
    );
}

fn find_button<'a>(r: &'a LayoutBox, d: &zero_dom::Document) -> Option<&'a LayoutBox> {
    if r.node_id.is_some_and(|nid| {
        d.get(nid)
            .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "button"))
    }) {
        return Some(r);
    }
    for c in &r.children {
        if let Some(b) = find_button(c, d) {
            return Some(b);
        }
    }
    None
}
