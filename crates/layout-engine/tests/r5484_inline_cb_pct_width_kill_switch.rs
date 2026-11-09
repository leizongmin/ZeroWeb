//! R5484 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_INLINE_CB_PCT_WIDTH=0` 回退 taffy 物化层解算旧径：inline
//! 中间层子百分比宽相对物化 fill 宽（blk 400 − inline ml 40 = 360）解算 → 180。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_box(root: &LayoutBox, id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 关闭态锚：span width:50% 回物化层分母 360 解算 = 180（pre-fix 旧径；盒其余
/// 几何不动——窄臂只改宽度预解算）。
#[test]
fn r5484_inline_cb_pct_width_kill_switch_reverts_materialized_denominator() {
    unsafe { std::env::set_var("ZW_INLINE_CB_PCT_WIDTH", "0") };

    let html = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; }
</style></head><body>
<div class="block" id="blk"><div class="inline" id="inl">
  <span>Ahem Ahem</span><span></span>
</div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let inline_div = doc.get_element_by_id("inl").expect("div.inline 存在");
    let span_ids: Vec<zero_dom::NodeId> = doc
        .child_nodes(inline_div)
        .into_iter()
        .filter(|&c| {
            doc.get(c)
                .is_some_and(|n| matches!(n.kind, zero_dom::NodeKind::Element(_)))
        })
        .collect();
    let span1 = find_box(&result.root, span_ids[0]).expect("span1 盒存在");
    assert!(
        (span1.width - 180.0).abs() <= 0.5,
        "ZW_INLINE_CB_PCT_WIDTH=0 应回退物化层分母 360 解算 = 180，实际 {}",
        span1.width
    );
}
