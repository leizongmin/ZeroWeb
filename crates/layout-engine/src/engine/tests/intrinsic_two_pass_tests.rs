//! intrinsic sizing 两趟布局回归测试（从 engine.rs 抽出，保持 2000 行约束）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 辅助：按 id 在布局树中查找 LayoutBox。
fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let zero_dom::NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find(id, doc, c))
}

/// 回归：CSS intrinsic sizing — `width:max-content` 的 grid 容器应收缩到其
/// max-content 宽度（2 item × (50 content + 40 padding) = 180），而非塌缩为 ~0
/// （converter MaxContent→length(0)）或填满视口（旧 Auto→fill）。
/// 验证两趟固有宽度布局（apply_intrinsic_content_sizing）把 grid 提升到 intrinsic。
#[test]
fn test_grid_width_max_content_sized_to_intrinsic() {
    // 复刻 child-border-box-and-max-content-001 结构：
    // grid(width:max-content, grid-auto-columns:1fr, column flow) > 2 item >
    // .content(width:50px)。grid intrinsic = 2×(50+40 padding) + 2 border ≈ 182。
    let html = r#"<html><body style="margin:0">
          <div id="g" style="display:grid;grid-auto-columns:1fr;grid-auto-flow:column;border:1px solid red;width:max-content">
            <div style="max-width:max-content;box-sizing:border-box;padding:10px 20px">
              <div style="width:50px;height:50px"></div>
            </div>
            <div style="max-width:max-content;box-sizing:border-box;padding:10px 20px">
              <div style="width:50px;height:50px"></div>
            </div>
          </div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let g = find("g", &doc, &result.root).expect("grid #g");
    // 不应塌缩（~2px）也不应填满（~784），应在 ~180px。
    assert!(
        g.width > 100.0,
        "width:max-content grid should be sized to intrinsic (~182px), not collapsed (got w={})",
        g.width
    );
    assert!(
        g.width < 400.0,
        "width:max-content grid should shrink-to-fit (~182px), not fill viewport (got w={})",
        g.width
    );
    assert!(
        (g.width - 182.0).abs() < 5.0,
        "expected grid width ~182px (2×(50+40)+border), got w={}",
        g.width
    );
}

/// 回归：intrinsic 不可测的 max-content 容器（纯文本 item）保持塌缩，
/// 不应被填满（验证不可测回退不会引入旧 Auto→fill 的 net -5 回归）。
#[test]
fn test_unmeasurable_max_content_does_not_fill() {
    // 纯文本 flex item 无显式宽度 → intrinsic 测量返回 None（Round C IFC 文本测量未就绪）
    // → 容器应保持塌缩（length(0)），而非填满视口。
    let html = r#"<html><body style="margin:0">
          <div id="f" style="display:flex;width:max-content"><div>text</div></div>
        </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let f = find("f", &doc, &result.root).expect("flex #f");
    // 不应填满视口（<700），证明不可测容器未被 auto-fill。
    assert!(
        f.width < 700.0,
        "unmeasurable width:max-content flex must not fill viewport (got w={}); \
         would regress 5 cases like R181c",
        f.width
    );
}

/// R324：position:fixed 须视口相对，即使位于有偏移的 positioned 祖先内。
///
/// taffy 0.7 把 fixed 当 absolute 处理（containing block = 最近 positioned 祖先），
/// 故 fixed 的 left/top 被解析为相对该祖先。`adjust_fixed_to_viewport` 须从累积
/// 祖先偏移中**扣除**（而非旧实现的「加上」），使其最终绝对坐标 = (left, top) 视口相对。
/// 旧「加上」实现仅在 parent_offset==0 时碰巧正确，对有 margin-offset 的 relative
/// 祖先内的 fixed 会 over-correct。本测试构造该场景，断言 fixed 视口相对、absolute
/// 兄弟仍祖先相对。
#[test]
fn test_fixed_is_viewport_relative_inside_offset_positioned_ancestor() {
    let html = r#"<html><body style="margin:0">
      <div style="position:relative; margin-top:100px; margin-left:50px; width:400px; height:300px">
        <div style="position:absolute; top:20px; left:20px; width:50px; height:50px"></div>
        <div style="position:fixed; top:20px; left:20px; width:50px; height:50px"></div>
      </div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 与 painter 一致的绝对坐标累积：abs = parent_offset + box.x/y，
    // 子元素 offset = abs + padding + border（本测试盒均无 padding/border，故相等）。
    fn collect(b: &crate::types::LayoutBox, ox: f32, oy: f32, out: &mut Vec<(bool, bool, f32, f32)>) {
        let ax = ox + b.x;
        let ay = oy + b.y;
        out.push((b.is_fixed, b.is_absolute, ax, ay));
        let child_ox = ax + b.padding_left + b.border_left;
        let child_oy = ay + b.padding_top + b.border_top;
        for c in &b.children {
            collect(c, child_ox, child_oy, out);
        }
    }
    let mut positions = Vec::new();
    collect(&result.root, 0.0, 0.0, &mut positions);

    let fixed = positions
        .iter()
        .find(|(f, _, _, _)| *f)
        .expect("should have a position:fixed box");
    let absolute = positions
        .iter()
        .find(|(f, a, _, _)| !*f && *a)
        .expect("should have a position:absolute box");

    // R324：fixed 视口相对 = (left 20, top 20)，不受 relative 祖先 margin(50,100) 影响
    assert!(
        (fixed.2 - 20.0).abs() < 1.0,
        "fixed x should be viewport-relative ~20, got {}",
        fixed.2
    );
    assert!(
        (fixed.3 - 20.0).abs() < 1.0,
        "fixed y should be viewport-relative ~20, got {}",
        fixed.3
    );
    // absolute 仍祖先相对 = 祖先(50,100) + (20,20) = (70,120)
    assert!(
        (absolute.2 - 70.0).abs() < 1.0,
        "absolute x should be ancestor-relative ~70, got {}",
        absolute.2
    );
    assert!(
        (absolute.3 - 120.0).abs() < 1.0,
        "absolute y should be ancestor-relative ~120, got {}",
        absolute.3
    );
}
