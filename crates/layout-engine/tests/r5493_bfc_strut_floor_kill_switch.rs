//! R5493 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_INLINE_ZEROBOX_STRUT=0` 关闭零高行盒 strut（R5492 缺陷 X
//! 臂＋R5493 BFC strut 下限同门控）：p23 形单 10px float 容器高回退纯
//! float_bottom 10。

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

/// 关闭态锚：p23 形（flow-root ＋ inline ml40 ＋ 单 50%/10px float）blk＝
/// float_bottom 10（strut floor 18 被门禁关闭）。
#[test]
fn r5493_strut_kill_switch_pure_float_bottom() {
    unsafe { std::env::set_var("ZW_INLINE_ZEROBOX_STRUT", "0") };

    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 10px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span></div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let blk = find_box(&result.root, doc.get_element_by_id("blk").expect("blk 存在")).expect("blk 盒存在");
    assert!(
        (blk.height - 10.0).abs() <= 0.5,
        "blk 关闭 strut 应为 float_bottom 10，实际 {}",
        blk.height
    );
}
