//! R5489 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_NESTED_FLOAT_CASCADE=0` 回退 R5446 逐 float 独立目标旧径：
//! 右 float 同层无序列避让，全钉极值缘 (200,0) 重叠（x 已由 R5487/R5489 极值缘臂
//! 修复为正确值 200——级联门只回退序列避让，不回退定位；fix 载体重写豁免随
//! `ZW_NESTED_INLINE_FLOAT` 另锚）。

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

/// 关闭态锚：r_orig 形（ml=40 两 50% 右 float）回退逐 float 独立极值缘——两 float
/// 同 target (0, 200)（容器 content-box 相对）→ abs (200,0)×2 重叠。
#[test]
fn r5489_right_cascade_kill_switch_reverts_independent_targets() {
    unsafe { std::env::set_var("ZW_NESTED_FLOAT_CASCADE", "0") };

    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span { float: right; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span></div></div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let s1 = doc.get_element_by_id("s1").expect("s1 存在");
    let s2 = doc.get_element_by_id("s2").expect("s2 存在");
    let f1 = abs_rect(&result.root, s1);
    assert!(
        (f1.0 - 200.0).abs() <= 0.5 && f1.1.abs() <= 0.5,
        "级联关闭态 float1 应极值缘 (200,0)，实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, s2);
    assert!(
        (f2.0 - 200.0).abs() <= 0.5 && f2.1.abs() <= 0.5,
        "ZW_NESTED_FLOAT_CASCADE=0 应回退逐 float 独立目标：float2 与 float1 同钉极值缘 \
         重叠 (200,0)，实际 {:?}",
        f2
    );
}
