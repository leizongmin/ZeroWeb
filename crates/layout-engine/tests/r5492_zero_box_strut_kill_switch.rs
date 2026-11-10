//! R5492 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_INLINE_ZEROBOX_STRUT=0` 回退 R5470 float_bottom 近似旧径：
//! 零行盒普通块 auto 高取 text_height.max(float_bottom)，p2/p3 红态 36（chromium
//! 0/18）为回退契约。

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

/// 绝对几何（沿父链累加 border/padding 原点，口径对齐 chromium gBCR）。
fn abs_rect(root: &LayoutBox, id: zero_dom::NodeId) -> (f32, f32, f32, f32) {
    fn walk(b: &LayoutBox, id: zero_dom::NodeId, x: f32, y: f32) -> Option<(f32, f32, f32, f32)> {
        let ax = x + b.x + b.border_left + b.padding_left;
        let ay = y + b.y + b.border_top + b.padding_top;
        if b.node_id == Some(id) {
            return Some((ax, ay, b.width, b.height));
        }
        for c in &b.children {
            if let Some(r) = walk(c, id, ax, ay) {
                return Some(r);
            }
        }
        None
    }
    walk(root, id, 0.0, 0.0).expect("目标盒存在")
}

/// 关闭态锚：两形均回 float_bottom 近似——无 ml 形 36（float 两行 extent）、
/// ml40 形 36（同 extent，strut 语义丢失）。
#[test]
fn r5492_zero_box_strut_kill_switch_reverts_float_bottom_approximation() {
    unsafe { std::env::set_var("ZW_INLINE_ZEROBOX_STRUT", "0") };

    for (want, inline_style, text) in [
        (
            36.0,
            "",
            "AAAAAAAAAA BBBBBBBBBB CCCCCCCCCC DDDDDDDDDD EEEEEEEEEE FFFFFFFFFF GGGGGGGGGG HHHHHHHHHH",
        ),
        (
            36.0,
            " margin-left: 40px;",
            "AAAAAAAAAA BBBBBBBBBB CCCCCCCCCC DDDDDDDDDD EEEEEEEEEE FFFFFFFFFF GGGGGGGGGG HHHHHHHHHH",
        ),
    ] {
        let html = format!(
            r#"<html><head><style>
body {{ margin: 0; }}
.block {{ display: block; background: green; }}
.inline {{ display: inline;{inline_style} }}
span {{ float: left; width: 50%; background: cyan; }}
</style></head><body>
<div class="block" id="blk"><div class="inline"><span>{text}</span></div></div>
</body></html>"#
        );
        let doc = zero_dom::parse_html(&html);
        let mut sys = StyleSystem::new();
        sys.set_viewport(800.0, 600.0);
        let styles = sys.compute_styles(&doc, &sheets(&html));
        let mut engine = LayoutEngine::new(800.0, 600.0);
        let result = engine.compute(&doc, &styles);
        let id = doc.get_element_by_id("blk").expect("块容器存在");
        let (_, _, _, h) = abs_rect(&result.root, id);
        assert!(
            (h - want).abs() <= 0.5,
            "ZW_INLINE_ZEROBOX_STRUT=0 应回退 float_bottom 近似 = {want}，实际 {h}",
        );
    }
}
