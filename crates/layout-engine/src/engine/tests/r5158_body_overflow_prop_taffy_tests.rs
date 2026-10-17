//! R5158（css-overflow-3 §3.3 body→viewport overflow 传播的 taffy 半边）：html 两轴
//! visible 且 body 非 visible 时，body used overflow = visible。R3923 已落实 LayoutBox
//! 层（engine.rs），但 taffy 样式仍按 computed Hidden 建 is_scroll_container → taffy
//! 父子 margin 折叠被抑制：body{overflow:hidden} 的首子 `<p>` margin 不与 body 塌缩，
//! 内容整体 +8px。此处把传播同步进 taffy 样式使折叠行为与 used value 一致。
//! driving: css/css-contain/contain-size-grid-002 残差 0.62%（文字区 8px 竖移）。
//! kill-switch `ZW_BODY_OVERFLOW_VISIBLE=0`。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

/// 返回 #g 盒 y（文档坐标）。
fn layout_green_y(html: &str) -> f32 {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let id = doc.get_element_by_id("g").unwrap();
    fn find(root: &LayoutBox, id: NodeId) -> Option<f32> {
        if root.node_id == Some(id) {
            return Some(root.y);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    find(&result.root, id).expect("#g 无布局盒")
}

/// corpus case 骨：`<p>`（margin-top 1em=16px）+ 100px 方块。body 默认 margin-top
/// 8px 与 p 塌缩穿透 → 方块 y=34（16 collapsed + 18 行高）。
const PLAIN: &str =
    r#"<html><body><p>xxx</p><div id="g" style="background:green;width:100px;height:100px"></div></body></html>"#;

/// body{overflow:hidden} 传播后塌缩照常：方块 y 与无 overflow 页逐位一致（修复前
/// 24px 不塌缩 → y=42，+8px）。
#[test]
fn r5158_body_overflow_hidden_collapses_like_plain() {
    let y_ovf = layout_green_y(
        r#"<html><body style="overflow: hidden"><p>xxx</p><div id="g" style="background:green;width:100px;height:100px"></div></body></html>"#,
    );
    let y_plain = layout_green_y(PLAIN);
    assert!(
        (y_ovf - y_plain).abs() < 0.5,
        "body{{overflow:hidden}} 应与 plain 同塌缩几何：ovf={y_ovf} plain={y_plain}"
    );
    assert!((y_plain - 34.0).abs() < 0.5, "plain 基线应为 34，实际 {y_plain}");
}

/// html 自身 overflow:hidden → html 的 overflow 传播给视口，body 保持自身 overflow
///（不传播）→ body 仍是 taffy 滚动容器，body↔p 父子塌缩被抑制。#g 盒 y 为**局部**
/// 坐标：p local 16 + 行高 18 + 兄弟塌缩(p mb 16 ⊓ g mt 16)=16 → 50。
#[test]
fn r5158_no_propagation_when_html_hidden() {
    let y = layout_green_y(
        r#"<html style="overflow: hidden"><body style="overflow: hidden"><p style="margin-top:16px">xxx</p><div id="g" style="background:green;width:100px;height:100px;margin-top:16px"></div></body></html>"#,
    );
    assert!((y - 50.0).abs() < 0.5, "html hidden 时 body 不塌缩应 y=50，实际 {y}");
}

/// body 自带 containment（css-contain-1：传播被抑制）→ body 保持 hidden 语义，
/// 塌缩被抑制（与 R3923 containment 臂同口径），#g 局部 y 同上 = 50。
#[test]
fn r5158_no_propagation_with_containment() {
    let y = layout_green_y(
        r#"<html><body style="overflow: hidden; contain: layout"><p style="margin-top:16px">xxx</p><div id="g" style="background:green;width:100px;height:100px;margin-top:16px"></div></body></html>"#,
    );
    assert!(
        (y - 50.0).abs() < 0.5,
        "containment 抑制传播应 y=50（不塌缩），实际 {y}"
    );
}
