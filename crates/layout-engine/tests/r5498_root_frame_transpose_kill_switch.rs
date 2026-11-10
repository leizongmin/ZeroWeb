//! R5498 kill-switch 回退锚（独立测试进程——`ZW_ROOT_FRAME_TRANSPOSE=0` 时根节点
//! 不交换进 taffy、根 available space 不转置：viv 根链行内 fill 回旧错轴契约
//! body/div=784=ICB 宽−16、p=752、html h=34 content junk＝R5497 缺陷 4 定谳几何；
//! 进程内 set_var 与并行单测互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 几何同 r5498_root_frame_transpose.rs 主判例 V1 形态（R5497 探针 V1 verbatim）。

use zero_layout_engine::types::LayoutBox;
use zero_layout_engine::{LayoutEngine, LayoutResult};
use zero_style_system::StyleSystem;

#[test]
fn kill_switch_restores_mixed_frame_axis_source() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_ROOT_FRAME_TRANSPOSE", "0") };

    let css = "html { writing-mode: vertical-rl; } body { margin: 8px; }";
    let html = "<html><head><style>html { writing-mode: vertical-rl; } body { margin: 8px; }</style></head>\
<body><div><p>Sentence before.</p></div></body></html>";
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result: LayoutResult = engine.compute(&doc, &styles);

    fn find_by_tag<'a>(node: &'a LayoutBox, tag: &str, doc: &zero_dom::Document) -> Option<&'a LayoutBox> {
        let hit = node
            .node_id
            .and_then(|id| doc.get(id))
            .and_then(|n| match &n.kind {
                zero_dom::NodeKind::Element(e) => Some(e.local_name() == tag),
                _ => None,
            })
            .unwrap_or(false);
        if hit {
            return Some(node);
        }
        node.children.iter().find_map(|c| find_by_tag(c, tag, doc))
    }

    let html_box = find_by_tag(&result.root, "html", &doc).expect("html 盒");
    let body = find_by_tag(&result.root, "body", &doc).expect("body 盒");
    let div = find_by_tag(&result.root, "div", &doc).expect("div 盒");
    let p = find_by_tag(&result.root, "p", &doc).expect("p 盒");
    // 旧错轴契约（R5497 探针 V1 实测）：body/div 行内 fill 消费 ICB 物理宽
    //（784=800−16）、p=752；html 物理高=内容块轴 extent junk 34。
    assert!(
        (html_box.height - 34.0).abs() <= 0.5,
        "kill-switch=0 html 物理高回内容 extent junk 34，实得 {}",
        html_box.height
    );
    assert!(
        (body.height - 784.0).abs() <= 0.5,
        "kill-switch=0 body 行内回 ICB 宽源 784（错轴契约），实得 {}",
        body.height
    );
    assert!(
        (div.height - 784.0).abs() <= 0.5,
        "kill-switch=0 div 行内回 784，实得 {}",
        div.height
    );
    assert!(
        (p.height - 752.0).abs() <= 0.5,
        "kill-switch=0 p 行内回 752，实得 {}",
        p.height
    );
    // 块轴链不受开关影响（两态同值）：html 物理宽 content shrink 34。
    assert!(
        (html_box.width - 34.0).abs() <= 1.0,
        "kill-switch=0 html 物理宽 content shrink 34（块轴链两态同值），实得 {}",
        html_box.width
    );
}
