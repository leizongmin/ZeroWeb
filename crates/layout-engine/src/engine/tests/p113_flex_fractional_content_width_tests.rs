//! t-p113 回归：taffy `round_layout` 累计整数取整把文本固有宽削到内容宽以下时，
//! Phase A IFC 以取整后的盒宽断行产生假折行（flex li 定尺 127.40625→127，文本
//! 127.40625 装不下断两行；Chrome 保 LayoutUnit 1/64 精度单行）。修复：IFC 约束
//! 宽/高改取 taffy 未取整层（kill-switch `ZW_IFC_UNROUNDED_CONTENT_WIDTH=0` 回退），
//! 盒几何（width/height，绘制与 gBCR）维持取整层不变。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 取第一个指定标签名的元素盒（深度优先）。
fn find_tag<'a>(box_node: &'a LayoutBox, doc: &Document, tag: &str) -> Option<&'a LayoutBox> {
    let hit = box_node
        .node_id
        .and_then(|nid| doc.get(nid))
        .and_then(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) => (e.local_name() == tag).then_some(()),
            _ => None,
        });
    if hit.is_some() {
        return Some(box_node);
    }
    box_node.children.iter().find_map(|c| find_tag(c, doc, tag))
}

/// Ahem 每字形 advance = 1em："Contact GitHub Support"（22 字形）× 5.79px = 127.38px。
/// 取整层 li 定尺 = round(127.38) = 127 < 127.38——修复前 IFC 假断行 li 高 36，
/// 修复后约束宽取未取整层 127.38，单行 li 高 18；盒几何仍为取整值（width=127）。
#[test]
fn test_p113_flex_li_fractional_width_no_spurious_wrap() {
    let html = r#"<html><body style="margin:0">
      <ul style="display:flex;list-style:none;margin:0;padding:0;font:400 5.79px/18px Ahem">
        <li><span>Contact GitHub Support</span></li>
      </ul>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let li = find_tag(&result.root, &doc, "li").expect("li box");
    assert!(
        (li.content_width - 127.38).abs() < 0.02,
        "li content_width should be unrounded 127.38, got {}",
        li.content_width
    );
    assert!(
        li.height >= 17.0 && li.height <= 19.0,
        "li should stay single-line (height≈18), got {} — spurious wrap from rounded constraint width",
        li.height
    );
    assert!(
        (li.width - 127.0).abs() < 0.01,
        "li border-box width stays on the rounded layer (127), got {}",
        li.width
    );
}

#[test]
fn test_p113_vertical_unbounded_fallback_matches_rounded_layer() {
    // DEF-1：!is_finite() 回退判定须在竖排轴交换之前——回退副本随后随交换块一同
    // 轴交换，竖排下与 kill-switch off 的取整层表达式逐项一致（R1964 场景
    // p.content_width 应为 18，回退错轴时经后处理落 0）。
    let html = r#"<html><body style="margin:0">
<div style="float:left; writing-mode:vertical-rl; background:red;">
  <p>hello world test text vertical</p>
</div>
<div style="width:200px;height:200px"></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let p = find_tag(&result.root, &doc, "p").expect("p box");
    assert!(
        (p.content_width - 18.0).abs() < 0.01,
        "vertical fallback should match kill-switch-off rounded-layer semantics (content_width=18), got {}",
        p.content_width
    );
}

/// 窗口对照臂：22 × 5.8px = 127.6，取整层 = 128 ≥ 127.6——修复前也已单行
///（round_layout 累计取整窗口效应），钉死该语义不被修复破坏。
#[test]
fn test_p113_flex_li_rounding_up_window_stays_single_line() {
    let html = r#"<html><body style="margin:0">
      <ul style="display:flex;list-style:none;margin:0;padding:0;font:400 5.8px/18px Ahem">
        <li><span>Contact GitHub Support</span></li>
      </ul>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let li = find_tag(&result.root, &doc, "li").expect("li box");
    assert!(
        (li.content_width - 127.6).abs() < 0.02,
        "li content_width should be unrounded 127.6, got {}",
        li.content_width
    );
    assert!(
        li.height >= 17.0 && li.height <= 19.0,
        "li should stay single-line (height≈18), got {}",
        li.height
    );
}

/// block 容器同源对照：显式 width 的纯块不受影响（127px 容器装 127.38px 文本，
/// 两引擎一致断行是正常行为）——钉修复不改变显式宽语义。
#[test]
fn test_p113_block_explicit_width_wrap_unchanged() {
    let html = r#"<html><body style="margin:0">
      <div style="width:127px;font:400 5.79px/18px Ahem">Contact GitHub Support</div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_tag(&result.root, &doc, "div").expect("div box");
    assert!(
        div.height >= 35.0,
        "explicit 127px < text 127.38px should wrap to 2 lines (height≥36), got {}",
        div.height
    );
}
