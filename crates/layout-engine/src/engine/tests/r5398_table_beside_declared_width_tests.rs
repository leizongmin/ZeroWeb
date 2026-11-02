//! R5398：table-among-floats beside 分支 declared-width 保声明宽回归测试。
//!
//! 谱系：WPT `css/CSS2/floats-clear/clear-applies-to-014.xht`（CSS 2.1 §9.5/§10.3.5，
//! 案面 assert「clear does not apply to inline-table」→ inline-table 旁置 float 右、
//! 保持声明宽 96）。step 8.5 `fix_table_among_floats` C 步（§9.5 BFC float-avoidance）
//! 的 beside 写宽臂原对**一切** beside 表填 float 旁可用 gap 宽（R1613 语义，为
//! auto-width BFC 表按 floats-wrap-bfc-002 ref 显式 width:200 调优）——定宽表被改写：
//! 014 的 inline-table width:1in 在 step 8 表布局后已是正确 96×96（旁置 float 右），
//! 被本臂填成 gap 宽 464（800−16−320）。修复按 declared 分流：auto 维持填 gap、
//! declared 取 `min(declared, gap)`（band-step 用 effective_w=declared 判 beside，
//! 能 beside 即 declared ≤ gap，min 仅防 step5 收缩残余越界）。
//!
//! kill-switch 敏感性（外部 env 实跑取证，2026-11-03，test-guard 包裹 runner）：
//! `ZW_TABLE_FLOAT_BESIDE_KEEP_DECLARED=0`（回退 beside 一律填 gap）下 014 形态
//! 隔离单跑 ✗（0/1）——本文件 beside 保宽断言转红；below 恢复声明宽臂（R1723）
//! 不受该开关影响。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::{Document, NodeKind};
use zero_style_system::StyleSystem;

fn find_box<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_box(id, doc, c))
}

fn compute(html: &str, css: &str) -> (Document, std::sync::Arc<LayoutBox>) {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result.root)
}

/// 014 主形态：定宽 inline-table 旁置 float 右保声明宽 96（旧 beside 填 gap 臂写 464）。
/// inline-table 不适用 clear（案面 assert），表与 float 同顶 beside——不推 float 下方。
#[test]
fn test_inline_table_beside_float_keeps_declared_width() {
    let html = r#"<html><body style="margin:8px">
<p id="float-p" style="float:left;line-height:1.25;margin:1em 0;width:320px">Filler text</p>
<div id="table" style="background:blue;clear:both;display:inline-table;table-layout:fixed;width:96px">
<div style="display:table-row"><div style="display:table-cell;height:48px">a</div><div style="display:table-cell;height:48px">b</div></div>
<div style="display:table-row"><div style="display:table-cell;height:48px">c</div><div style="display:table-cell;height:48px">d</div></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let float_p = find_box("float-p", &doc, &root).expect("should find float p");
    let table = find_box("table", &doc, &root).expect("should find inline-table");

    // ★ R5398 load-bearing：beside 保声明宽（旧 beside 填 gap 臂 = 784−320 = 464）。
    assert!(
        (table.width - 96.0).abs() <= 0.5,
        "declared-width inline-table beside float must keep 96 (gap-fill wrote {})",
        table.width
    );
    assert!(
        (table.height - 96.0).abs() <= 0.5,
        "table h must be 96; h={}",
        table.height
    );
    // 旁置：x = float 右缘（content 相对 320），y 与 float 同顶（clear 不适用 inline-table，
    // 不下推 float 底）。
    let float_right = float_p.x + float_p.width;
    assert!(
        (table.x - float_right).abs() <= 0.5,
        "table must sit beside float right edge; table.x={} float_right={}",
        table.x,
        float_right
    );
    assert!(
        table.y + 0.5 < float_p.y + float_p.height,
        "cleared inline-table must not be pushed below float (clear does not apply); \
         table.y={} float_bottom={}",
        table.y,
        float_p.y + float_p.height
    );
}

/// 定宽 display:table（块级 BFC 表）旁置同样保声明宽——R5398 分流谓词按 declared
/// 判（与 display 形态无关），旧臂同把此形态填成 gap 宽。
#[test]
fn test_declared_block_table_beside_float_keeps_declared_width() {
    let html = r#"<html><body style="margin:8px">
<div id="float-p" style="float:left;width:100px;height:50px;background:yellow"></div>
<div id="table" style="display:table;width:150px;background:blue">
<div style="display:table-row"><div style="display:table-cell;height:40px">a</div></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let table = find_box("table", &doc, &root).expect("should find declared table");

    // 容器 content 784，float 右缘 100 → gap 684 ≥ 150 → beside 保 150（旧填 684）。
    assert!(
        (table.width - 150.0).abs() <= 0.5,
        "declared-width table beside float must keep 150 (gap-fill wrote {})",
        table.width
    );
    assert!(
        (table.x - 100.0).abs() <= 0.5,
        "table must sit at float right edge; x={}",
        table.x
    );
}

/// R1613 既有语义守卫：auto-width 表 beside 仍填 float 旁可用 gap 宽（R5398 分流
/// 只动 declared 臂，auto 行为逐字节保留）。
#[test]
fn test_auto_table_beside_float_still_fills_gap() {
    let html = r#"<html><body style="margin:8px">
<div id="float-p" style="float:left;width:100px;height:50px;background:yellow"></div>
<div id="table" style="display:table;background:blue">
<div style="display:table-row"><div style="display:table-cell;height:40px">a</div></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let table = find_box("table", &doc, &root).expect("should find auto table");

    // auto 表 beside 填 gap = 784 − 100 = 684（R1613 beside 填 gap 语义不变）。
    assert!(
        (table.width - 684.0).abs() <= 0.5,
        "auto-width table beside float must still fill gap 684; w={}",
        table.width
    );
}

/// R1723 既有语义守卫：定宽表放不下 beside gap 时推到 float 下方并恢复声明宽
///（below 恢复臂不受 R5398 beside 分流影响）。
#[test]
fn test_declared_table_pushed_below_float_restores_declared_width() {
    let html = r#"<html><body style="margin:8px">
<div id="float-p" style="float:left;width:700px;height:50px;background:yellow"></div>
<div id="table" style="display:table;width:150px;background:blue">
<div style="display:table-row"><div style="display:table-cell;height:40px">a</div></div>
</div>
</body></html>"#;
    let (doc, root) = compute(html, "");
    let float_p = find_box("float-p", &doc, &root).expect("should find float");
    let table = find_box("table", &doc, &root).expect("should find declared table");

    // gap = 784 − 700 = 84 < 150 → beside 放不下 → band-step 推到 float 底（y=50），
    // x 回流位 0，R1723 below 恢复声明宽 150。
    let float_bottom = float_p.y + float_p.height;
    assert!(
        (table.y - float_bottom).abs() <= 0.5,
        "table must be pushed below float bottom; y={} float_bottom={}",
        table.y,
        float_bottom
    );
    assert!(
        (table.x - 0.0).abs() <= 0.5,
        "below table must return to flow x=0; x={}",
        table.x
    );
    assert!(
        (table.width - 150.0).abs() <= 0.5,
        "below table must restore declared width 150; w={}",
        table.width
    );
}
