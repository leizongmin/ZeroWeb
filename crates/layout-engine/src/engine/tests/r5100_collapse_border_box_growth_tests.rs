//! R5100（CSS2 §17.6.1 collapsing border model）：行/行组声明边框经格线冲突胜出后，
//! 胜出宽度入 cell border-box（底缘 → cell 高、右缘 → 末列 cell 宽）——此前 override
//! 只改 border 字段不动尺寸，0 内容高的 collapse 表行/表高塌 0，底缘带绘于 cell 底边
//! 之 upside（border-bottom-width-applies-to-001 族 2.93% ×4）。
//! 右缘宽度增长按胜出来源 gate：Row/RowGroup 外扩（right-001..004 1.68% 族）；
//! Table/ColumnGroup/Column 胜出走原路径带绘于内容区内缘（right-005 colgroup 判例，
//! 无差别外扩翻红）。

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

/// 正例（border-bottom-width-applies-to-001 形态）：rowgroup border-bottom 96px
/// 胜出末行 cell 底缘 → 末行 cell 高 96、首行 cell 不增长。
#[test]
fn test_r5100_collapse_bottom_border_grows_cell_height() {
    let html = r#"<html><body style="margin:0">
      <div style="display:table;border-collapse:collapse;table-layout:fixed;width:96px" id="tbl">
        <div style="display:table-row-group;border-bottom-style:solid;border-bottom-width:96px" id="rg">
          <div style="display:table-row">
            <div style="display:table-cell"></div>
            <div style="display:table-cell"></div>
          </div>
          <div style="display:table-row">
            <div style="display:table-cell" id="c1"></div>
            <div style="display:table-cell" id="c2"></div>
          </div>
        </div>
      </div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let c1 = find("c1", &doc, &result.root).expect("c1 box");
    let c2 = find("c2", &doc, &result.root).expect("c2 box");
    assert!(
        (c1.height - 96.0).abs() < 0.5 && (c2.height - 96.0).abs() < 0.5,
        "R5100: 末行 cell 高应含胜出底缘边框 96（旧行/表塌 0、底缘带 upside），         got c1={} c2={}",
        c1.height,
        c2.height
    );
}

/// gate 正例（border-right-width-applies-to-001 形态）：rowgroup border-right 96px
/// 胜出末列右缘（side 4）→ 末列 cell 宽 48+96=144，首列不增长。
#[test]
fn test_r5100_collapse_right_border_rowgroup_winner_grows_width() {
    let html = r#"<html><body style="margin:0">
      <div style="display:table;border-collapse:collapse;table-layout:fixed;width:96px" id="tbl">
        <div style="display:table-row-group;border-right-style:solid;border-right-width:96px" id="rg">
          <div style="display:table-row">
            <div style="display:table-cell" id="first"></div>
            <div style="display:table-cell" id="last"></div>
          </div>
        </div>
      </div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let first = find("first", &doc, &result.root).expect("first cell box");
    let last = find("last", &doc, &result.root).expect("last cell box");
    assert!(
        (last.width - (48.0 + 96.0)).abs() < 0.5,
        "R5100: 行组胜出右缘的末列 cell 宽应 48+96=144，got {}",
        last.width
    );
    assert!(
        (first.width - 48.0).abs() < 0.5,
        "R5100: 非末列 cell 宽不增长，got {}",
        first.width
    );
}

/// gate 负例（border-right-width-applies-to-005 形态）：colgroup（含 column 子元素）
/// border-right 胜出走 side 1——末列 cell 宽不增长（005 A/B 判例：无差别外扩翻红）。
#[test]
fn test_r5100_collapse_right_border_colgroup_winner_no_growth() {
    let html = r#"<html><body style="margin:0">
      <div style="display:table;border-collapse:collapse;table-layout:fixed;width:96px" id="tbl">
        <div style="display:table-column-group;border-right-style:solid;border-right-width:96px">
          <div style="display:table-column"></div>
          <div style="display:table-column"></div>
        </div>
        <div style="display:table-row-group">
          <div style="display:table-row">
            <div style="display:table-cell"></div>
            <div style="display:table-cell" id="last"></div>
          </div>
        </div>
      </div>
    </body></html>"#;
    let (doc, result) = layout(html);
    let last = find("last", &doc, &result.root).expect("last cell box");
    assert!(
        (last.width - 48.0).abs() < 0.5,
        "R5100: colgroup 胜出右缘不外扩（side 1），末列 cell 宽应保持 48，got {}",
        last.width
    );
}
