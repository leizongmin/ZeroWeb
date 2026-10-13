//! R5099（css-tables-3 §3.10.1 行最小高 / CSS2 §17.5.3）：行指定高（行元素 computed
//! height / 独占行 cell 的 definite height）与 table target 无关地作行高 floor——
//! auto 高表的指定行此前随 target 缺失被塌成内容高（空 cell 行 0 高）。
//! driving: css/CSS2/normal-flow/height-applies-to-004（row height:1in 空 cell
//! 黑方块整缺 1.92%）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::Document;
use zero_style_system::StyleSystem;

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

fn layout(html: &str) -> (Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 正例：auto 高表 + 指定行高（空 cell）→ 行高 = 指定值，表随行和增长。
#[test]
fn test_r5099_row_spec_floor_in_auto_table() {
    let html = r#"<html><body style="margin:0">
      <div id="tbl" style="display:table;table-layout:fixed;width:96px">
        <div id="row" style="display:table-row;height:96px">
          <div id="cell" style="display:table-cell;background:#000"></div>
        </div>
      </div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let row = find("row", &doc, &result.root).expect("row box");
    let tbl = find("tbl", &doc, &result.root).expect("table box");
    assert!(
        (row.height - 96.0).abs() < 0.5,
        "R5099: 指定行高在 auto 表中须为行高下限（height-applies-to-004 旧塌 0），got {}",
        row.height
    );
    assert!(
        (tbl.height - 96.0).abs() < 0.5,
        "R5099: 表高应随行和增长到 96（cell 黑背景覆盖 96×96），got {}",
        tbl.height
    );
}

/// 锚例：definite 高表 + 指定行（R5095 auto 行优先分配语义不变）——
/// 指定行保持指定高、auto 行吸收余量，floor 不双计。
#[test]
fn test_r5099_row_spec_floor_no_double_count_with_table_target() {
    let html = r#"<html><body style="margin:0">
      <div id="tbl" style="display:table;table-layout:fixed;width:96px;height:224px">
        <div id="r1" style="display:table-row;height:64px">
          <div style="display:table-cell"></div>
        </div>
        <div id="r2" style="display:table-row">
          <div style="display:table-cell"></div>
        </div>
        <div id="r3" style="display:table-row;height:64px">
          <div style="display:table-cell"></div>
        </div>
      </div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let r1 = find("r1", &doc, &result.root).expect("r1 box");
    let r2 = find("r2", &doc, &result.root).expect("r2 box");
    let r3 = find("r3", &doc, &result.root).expect("r3 box");
    let tbl = find("tbl", &doc, &result.root).expect("table box");
    // R5095 判例：64/96/64（css-tables-3 §3.10.5 auto 行优先吸收余量）。
    assert!(
        (r1.height - 64.0).abs() < 0.5 && (r3.height - 64.0).abs() < 0.5,
        "R5099: 指定行高保持 64（不因 floor 双计），got r1={} r3={}",
        r1.height,
        r3.height
    );
    assert!(
        (r2.height - 96.0).abs() < 0.5,
        "R5099: auto 行吸收余量到 96（R5095 语义不变），got {}",
        r2.height
    );
    assert!(
        (tbl.height - 224.0).abs() < 0.5,
        "R5099: 表高 = target 224，got {}",
        tbl.height
    );
}
