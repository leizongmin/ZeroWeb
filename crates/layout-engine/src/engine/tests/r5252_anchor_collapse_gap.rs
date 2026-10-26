//! R5252：块→匿名行盒边界锚定 collapse gap 锚（CSS2 §8.3.1 + §9.2.1.1）。
//!
//! 案源：wpt `css/CSS2/floats-clear/margin-collapse-121/123`（REF 页 Talbot 改写：
//! `<p>` UA margin-bottom 1em 后继 `display:inline-table; margin:1em 0em auto 1em`）。
//! spec：inline 级盒 margin 不 collapse（§8.3.1）——p 的 mb 16 与匿名块 mt=0 collapse
//! 出 16px 段间隙，inline-table 的 mt 16 在行盒内独立生效（margin box 参与行盒高度，
//! §10.8.1）。修复前：taffy 把提升的原子 inline 当块级兄弟伪堆叠（mt 与 prev mb 取
//! max），锚 = taffy 位 − fragment y 沿用伪堆叠位，两值非零时行盒整体上移
//! min(prev_mb, mt)（本案 16px，test/ref 可见内容错位一倍行高）。
//!
//! kill-switch `ZW_R5252_ANCHOR_GAP=0` 回退修正（本锚随回退失败属预期）。

use super::*;

/// body 一级子盒几何（过滤可见盒）。坐标相对 body content 顶。
fn body_child_boxes(html: &str) -> Vec<(f32, f32, f32, f32)> {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let body = result
        .root
        .children
        .iter()
        .find(|c| {
            c.node_id
                .and_then(|id| doc.get(id))
                .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "body"))
        })
        .unwrap_or_else(|| panic!("body 盒缺失"));
    body.children
        .iter()
        .filter(|c| c.height > 1.0)
        .map(|c| (c.x, c.y, c.width, c.height))
        .collect()
}

/// 段首原子 y = prev 块底 + prev mb（与匿名块 collapse）+ 自身 mt（行盒内生效）。
/// p mb16 + inline-block mt32：18 + 16 + 32 = 66（修复前伪堆叠 max(16,32)=50）。
#[test]
fn r5252_inline_block_after_block_margin_keeps_both_gaps() {
    let r = body_child_boxes(
        "<html><body style=\"margin:0\">\
<p style=\"margin:0 0 16px 0\">x</p>\
<span style=\"display:inline-block;width:64px;height:64px;margin-top:32px;background:aqua\"></span>\
</body></html>",
    );
    assert!(
        (r[1].1 - 66.0).abs() < 0.5,
        "inline-block y 应 66（段隙 16 + 行内 mt 32），实际 {:?}",
        r
    );
}

/// inline-table 同语义（§8.3.1 不分原子种类）：p mb16 + table mt32 → y=66。
#[test]
fn r5252_inline_table_after_block_margin_keeps_both_gaps() {
    let r = body_child_boxes(
        "<html><body style=\"margin:0\">\
<p style=\"margin:0 0 16px 0\">x</p>\
<table style=\"display:inline-table;margin:32px 0 0 0;border:0;border-spacing:0\">\
<tr><td style=\"width:64px;height:64px;background:aqua\"></td></tr></table>\
</body></html>",
    );
    assert!((r[1].1 - 66.0).abs() < 0.5, "inline-table y 应 66，实际 {:?}", r);
}

/// margin-collapse-121 REF 页形态：p UA mb16 与两表 mt16 等值——伪堆叠 max(16,16)
/// 吞 16px，两表整栈上移一倍行高。正确：段隙 16 + 行内 mt 16 → y=50。
#[test]
fn r5252_equal_margins_gap_not_swallowed() {
    let table = "<table style=\"display:inline-table;margin:1em 0em auto 1em;border:3px solid black;border-spacing:0\">\
<tr><td style=\"width:64px;height:64px;background:aqua\"></td><td style=\"width:64px;height:64px;background:lime\"></td></tr>\
<tr><td colspan=\"2\" style=\"width:128px;height:64px;background:yellow\"></td></tr>\
</table>";
    let r = body_child_boxes(&format!(
        "<html><body>\
<p>The following two squares should be identical.</p>\
{table}{table}\
</body></html>"
    ));
    assert!(
        r.len() >= 3 && (r[1].1 - 50.0).abs() < 0.5 && (r[2].1 - 50.0).abs() < 0.5,
        "两 inline-table y 应 50（段隙 16 + 行内 mt 16），实际 {:?}",
        r
    );
}

/// 控制组：prev mb=0 时伪堆叠位与真位本就一致（min(0, mt)=0），修正须零影响。
#[test]
fn r5252_zero_prev_margin_control_unchanged() {
    let r = body_child_boxes(
        "<html><body style=\"margin:0\">\
<p style=\"margin:0\">x</p>\
<table style=\"display:inline-table;margin:32px 0 0 0;border:0;border-spacing:0\">\
<tr><td style=\"width:64px;height:64px;background:aqua\"></td></tr></table>\
</body></html>",
    );
    assert!(
        (r[1].1 - 50.0).abs() < 0.5,
        "控制组 y 应 50（18 + 行内 mt 32），实际 {:?}",
        r
    );
}
