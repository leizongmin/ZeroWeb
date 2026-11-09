//! R5491 kill-switch 独立进程锚（R3858 纪律：env 翻转与并行单测互踩，独立文件
//! 独立进程）——`ZW_INLINE_CB_PCT_WIDTH=0` 整臂回退（含 R5491 auto 宽 fill 上溯）：
//! taffy 物化层解算旧径，span width:50% 相对物化 fill 宽（blk 800 − inline ml 40
//! = 760）解算 → 380（缺陷 Y 红态契约）。

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

/// 关闭态锚：blk 无宽（R5491 新域）→ 50% 回物化层分母 760 解算 = 380（pre-R5491
/// 红态：级联 fit 判定 `380<=380` 边界翻转 s2 并排重叠）。
#[test]
fn r5491_inline_cb_auto_width_kill_switch_reverts_materialized_denominator() {
    unsafe { std::env::set_var("ZW_INLINE_CB_PCT_WIDTH", "0") };

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
    let s1 = doc.get_element_by_id("s1").expect("s1 存在");
    let box1 = find_box(&result.root, s1).expect("s1 盒存在");
    assert!(
        (box1.width - 380.0).abs() <= 0.5,
        "ZW_INLINE_CB_PCT_WIDTH=0 应回退物化层分母 760 解算 = 380，实际 {}",
        box1.width
    );
}
