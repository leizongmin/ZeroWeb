//! R5493 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_NESTED_FLOAT_CASCADE=0` 回退 R5446 逐 float 独立目标旧径：
//! 同 wrapper 三 float 无 band 序列避让，全钉 wrapper 首行锚位重叠（盒相对
//! x=−40＝inline 层 ml 编码契约，R5485 口径），blk＝float_bottom 18。

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

fn rel_pos(root: &LayoutBox, id: zero_dom::NodeId) -> (f32, f32) {
    let b = find_box(root, id).expect("目标盒存在");
    (b.x, b.y)
}

/// 关闭态锚：主形（flow-root ＋ inline ml40 ＋ 3×50%/18px float）回退逐 float
/// 独立目标——三 float 同钉锚位 (−40,0) 重叠，blk＝float_bottom 18。
#[test]
fn r5493_cascade_kill_switch_reverts_independent_targets() {
    unsafe { std::env::set_var("ZW_NESTED_FLOAT_CASCADE", "0") };

    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span><span id="s3"></span></div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    for sid in ["s1", "s2", "s3"] {
        let id = doc.get_element_by_id(sid).expect("float span 存在");
        let (x, y) = rel_pos(&result.root, id);
        assert!(
            (x + 40.0).abs() <= 0.5 && y.abs() <= 0.5,
            "{sid} 旧径应钉锚位 (−40,0)，实际 ({x},{y})"
        );
    }
    let blk = find_box(&result.root, doc.get_element_by_id("blk").expect("blk 存在")).expect("blk 盒存在");
    assert!(
        (blk.height - 18.0).abs() <= 0.5,
        "blk 旧径应为 float_bottom 18，实际 {}",
        blk.height
    );
}
