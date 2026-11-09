//! R5487 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_NESTED_FLOAT_CASCADE=0` 回退 R5446 逐 float 独立目标旧径：
//! 同层连续左 float 无序列避让，全钉带左缘原点重叠 (0,0)×2（pre-fix 红证据谱系
//! ＝本锚保留的回退行为契约）。

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

/// 关闭态锚：原形（inline 层 ml=40，两 50% float）回退逐 float 独立目标——两
/// float 同 target (0,0)（容器 content-box 相对；inline 层 x=40 下盒 x=-40）。
/// 断言用盒相对坐标（float1 x=-40＝相对父编码契约，R5485 勘误口径）。
#[test]
fn r5487_cascade_kill_switch_reverts_independent_targets() {
    unsafe { std::env::set_var("ZW_NESTED_FLOAT_CASCADE", "0") };

    let html = r#"<html><head><style>
body { margin: 0; font: 16px/18px Ahem, sans-serif; }
.block { display: block; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50% }
</style></head><body>
<div class="block" id="blk"><div class="inline" id="inl">
  <span id="s1">A</span><span id="s2">B</span>
</div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let s1 = doc.get_element_by_id("s1").expect("s1 存在");
    let s2 = doc.get_element_by_id("s2").expect("s2 存在");
    let p1 = rel_pos(&result.root, s1);
    assert!(
        (p1.0 + 40.0).abs() <= 0.5 && p1.1.abs() <= 0.5,
        "float1 相对层坐标应 (-40,0)（层 x=40 → abs (0,0)），实际 {:?}",
        p1
    );
    let p2 = rel_pos(&result.root, s2);
    assert!(
        (p2.0 + 40.0).abs() <= 0.5 && p2.1.abs() <= 0.5,
        "ZW_NESTED_FLOAT_CASCADE=0 应回退逐 float 独立目标：float2 与 float1 重叠 \
         （相对 (-40,0)，abs (0,0)），实际 {:?}",
        p2
    );
}
