//! R5435（CSS2 §17.5.2.1/§17.5.3）：col 指定宽钳臂缩列后行高随动＋valign 沉底。
//!
//! slice 1：R5434 把 col 指定宽列的 auto cell 贡献钳到 used width，但 cell content
//! 仍是钳前宽度的单行度量——行高不随钳后列宽换行增高（vertical-align-baseline-003
//! ref 侧：列 80 内容 12 行、行高塌 20）。钳臂实际减宽时并入 R4298 remeasure 链。
//! kill-switch `ZW_TABLE_COL_SPEC_REMEASURE`。
//!
//! slice 2：行高随动后矮 cell 拉伸满行高，R1717 臂的 `height - extra` 推导在 auto
//! 高表 extra=0 → avail=0，valign:bottom 箭头停顶部行。IFC 存储臂（compute_final，
//! 行 y 真底）按均衡化后 content_height 重算 valign_offset（§17.5.3）。
//! kill-switch `ZW_CELL_VALIGN_IFC_TRUTH`。
//!
//! 断言取结构不变式（换行→行高增长、无钳→无偏移），不锚像素（harness estimate
//! metrics 与真实字体域换行数不同，见 R5426 estimate-fonts 伪绿判例）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::Document;
use zero_style_system::StyleSystem;

/// 按 id 属性定位盒（仓库既有 idiom，含 doc.get 校验）。
fn find<'a>(id: &str, doc: &Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let zero_dom::NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find(id, doc, c))
}

/// 真管线探针：布局表，收集三 cell 的 (content_height, height, valign_offset)。
fn probe_cells(html: &str) -> Vec<(f32, f32, f32)> {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    ["side", "mid", "side2"]
        .iter()
        .map(|id| {
            find(id, &doc, &result.root)
                .map(|b| (b.content_height, b.height, b.valign_offset))
                .unwrap_or((f32::NAN, f32::NAN, f32::NAN))
        })
        .collect()
}

const REF_LIKE_HTML: &str = r#"<!DOCTYPE html><html><body style="margin:0">
<table style="border-spacing:0"><col style="width:40px"><col style="width:80px"><col style="width:40px">
<tr><td id="side" style="vertical-align:bottom">--&gt;&nbsp;</td><td id="mid">Filler Text Filler Text Filler Text Filler Text Filler Text Filler Text</td><td id="side2" style="vertical-align:bottom">&nbsp;&lt;--</td></tr>
</table>
</body></html>"#;

const NO_COL_HTML: &str = r#"<!DOCTYPE html><html><body style="margin:0">
<table style="border-spacing:0">
<tr><td id="side" style="vertical-align:bottom">--&gt;&nbsp;</td><td id="mid">Filler Text Filler Text Filler Text Filler Text Filler Text Filler Text</td><td id="side2" style="vertical-align:bottom">&nbsp;&lt;--</td></tr>
</table>
</body></html>"#;

/// ★ driving 案形（vertical-align-baseline-003-ref 同构）：col 指定宽 80 钳臂缩列后
/// mid cell 内容按 80 重排换行、行高随动增长，单行矮 cell 拉伸到行高；
/// valign:bottom 箭头 cell 按 IFC 真文本底下沉（slice 2，avail = 行 content 高 − 文本高）。
#[test]
fn r5435_col_spec_clamp_row_height_follows_reflow() {
    let cells = probe_cells(REF_LIKE_HTML);
    let (side_ch, side_h, side_vo) = cells[0];
    let (mid_ch, mid_h, _mid_vo) = cells[1];
    let (side2_ch, _side2_h, side2_vo) = cells[2];
    // mid cell 内容按 80px 列宽换行（6 句 Filler Text ≥ 2 行，单行 strut ≤ 30）。
    assert!(
        mid_ch > 30.0,
        "mid cell content_height 应随钳后列宽换行增长，got {mid_ch}"
    );
    // 行高随动：side cell（单行内容）被拉伸到行高 ≈ mid cell 高。
    assert!(
        (side_h - mid_h).abs() < 0.5,
        "side cell 应拉伸到行高 side={side_h} mid={mid_h}"
    );
    // 箭头 cell 不抬高行高：content_height 被行高均衡化覆写（= mid 的 12 行高），
    // 若 remeasure 误把箭头内容按窄列增高超过 mid，side_h 会超过 mid_h（上一断言覆盖）。
    let _ = (side_ch, side2_ch);
    // valign:bottom 沉底（slice 2，IFC 真文本高）：avail = 行 content 高 − 单行文本高 > 0。
    assert!(side_vo > 0.0, "side valign_offset 应 > 0（沉底），got {side_vo}");
    assert!(side2_vo > 0.0, "side2 valign_offset 应 > 0（沉底），got {side2_vo}");
    // 沉底量 = 行 content 高 − 单行文本高（≈ 差一整行文本带，非微量）。
    assert!(side_vo > 20.0, "side 沉底量应超过一行文本带，got {side_vo}");
}

/// ★ 负向回归卫：无 col 指定宽（钳臂不入臂）时维持旧行为——mid 单行、行高不增长、
/// 箭头 cell avail=0 无偏移。
#[test]
fn r5435_no_col_spec_keeps_legacy_row_height() {
    let cells = probe_cells(NO_COL_HTML);
    let (_side_ch, _side_h, side_vo) = cells[0];
    let (mid_ch, _mid_h, _mid_vo) = cells[1];
    let (_side2_ch, _side2_h, side2_vo) = cells[2];
    // 800px 可用宽下 6 句 Filler Text 单行装下（钳臂未触发，行高不随动）。
    assert!(mid_ch < 30.0, "无 col 宽时 mid cell 应保持单行内容高，got {mid_ch}");
    assert_eq!(side_vo, 0.0, "无钳臂时 side valign_offset 应为 0");
    assert_eq!(side2_vo, 0.0, "无钳臂时 side2 valign_offset 应为 0");
}
