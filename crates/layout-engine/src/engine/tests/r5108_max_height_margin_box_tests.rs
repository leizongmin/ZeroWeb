//! R5108（CSS2 §10.4 / §10.6.3 / §8.3.1）：块盒 px max-height 钳制 + 塌穿泄漏 margin
//! 的容器 margin-box 计入。
//!
//! ① converter 把 max_height 映射 taffy max_size，但两段式重跑路径下 auto 高块盒
//! 不被钳（margin-collapse-038 父 max-height:99px 渲 300 应 99）——后处理 1c) 臂
//! 以真实长度钳制（content 关键字/百分比由既有 1b)/1) 臂分管）。
//! ② 本盒 bottom margin 与末 in-flow 子 bottom margin 不 adjoining（有 bottom
//! border/padding）且 height:auto 时，内容高须延伸到末子 margin-box 底缘
//!（§10.6.3 + §8.3.1 冒泡阻断）——塌穿泄漏值（child.margin_bottom 含子树折叠链）
//! 计入容器高（margin-collapse-038 绿底容器 199 = 99 父钳高 + 100 塌穿 mb）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

fn find_by_id<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_by_id(id, doc, c))
}

fn compute(html: &str, css: &str) -> (Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[stylesheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// ① px max-height 钳制四形态：auto 高 / definite 高 / 纯 definite / 带 frame。
#[test]
fn r5108_px_max_height_clamps_auto_height_blocks() {
    let (_doc, result) = compute(
        r#"<html><head><style>
#a { max-height: 50px; }
#b { max-height: 50px; height: 200px; }
#e { height: 200px; }
#f { max-height: 50px; padding-top: 5px; border-top: 2px solid black; }
</style></head><body style="margin:0">
<div id="a"><div style="height:300px"></div></div>
<div id="b"><div style="height:300px"></div></div>
<div id="e"><div style="height:300px"></div></div>
<div id="f"><div style="height:300px"></div></div>
</body></html>"#,
        "#a { max-height: 50px; } #b { max-height: 50px; height: 200px; } #e { height: 200px; } #f { max-height: 50px; padding-top: 5px; border-top: 2px solid black; }",
    );
    for (id, expect) in [("a", 50.0), ("b", 50.0), ("e", 200.0), ("f", 57.0)] {
        let b = find_by_id(id, &_doc, &result.root).unwrap_or_else(|| panic!("box #{id}"));
        assert!(
            (b.height - expect).abs() < 0.5,
            "#{id} h={} 应为 {expect}（§10.4 max-height 钳制）",
            b.height
        );
    }
}

/// ② margin-collapse-038 结构：max-height 父（99）+ 塌穿子（mb 泄漏 100）+
/// bottom-border 容器——容器内容高 = 父 margin-box 底缘 199（99 + 100）。
#[test]
fn r5108_collapsed_through_margin_included_with_bottom_border() {
    let (doc, result) = compute(
        r#"<html><head><style>
#g { background: green; border-bottom: green solid 1px; }
#m { margin-bottom: 0px; max-height: 99px; }
#l { height: 300px; margin-bottom: 100px; }
</style></head><body style="margin:0">
<div id="g"><div id="m"><div id="l"></div></div></div>
</body></html>"#,
        "#g { background: green; border-bottom: green solid 1px; } #m { margin-bottom: 0px; max-height: 99px; } #l { height: 300px; margin-bottom: 100px; }",
    );
    let m = find_by_id("m", &doc, &result.root).expect("#m");
    assert!((m.height - 99.0).abs() < 0.5, "#m h={} 应钳到 99（§10.4）", m.height);
    assert!(
        (m.margin_bottom - 100.0).abs() < 0.5,
        "#m mb={} 应含塌穿泄漏 100",
        m.margin_bottom
    );
    let g = find_by_id("g", &doc, &result.root).expect("#g");
    assert!(
        (g.content_height - 199.0).abs() < 0.5,
        "#g ch={} 应为 199 = 99（钳高）+ 100（塌穿 mb，§10.6.3 margin-box）",
        g.content_height
    );
    assert!(
        (g.height - 200.0).abs() < 0.5,
        "#g h={} 应为 199 + 1px border",
        g.height
    );
}

/// 负例：无 bottom border/padding 时子 mb 与父 bottom 折叠（塌穿出父），容器高
/// 不含子 mb（margin-collapse 既有语义——折叠发生在 border-box 外）。
#[test]
fn r5108_no_bottom_frame_excludes_child_margin() {
    let (doc, result) = compute(
        r#"<html><head><style>
#p { background: blue; }
</style></head><body style="margin:0">
<div id="p"><div style="height:50px; margin-bottom:20px"></div></div>
</body></html>"#,
        "#p { background: blue; }",
    );
    let p = find_by_id("p", &doc, &result.root).expect("#p");
    assert!(
        (p.content_height - 50.0).abs() < 0.5,
        "#p ch={} 应为 50（子 mb 折叠出父）",
        p.content_height
    );
}
