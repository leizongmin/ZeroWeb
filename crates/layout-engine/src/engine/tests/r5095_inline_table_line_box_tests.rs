//! R5095（css-backgrounds-3 §5.6 driving 族定界）：原子行内级盒行盒组装两修复。
//!
//! ① shift_siblings_after_ifc_grow：prev 与 next **都是原子行内级盒**（inline-table 等）
//!    时豁免「prev.bottom > next.y」重叠下推——两盒共享同一行盒（CSS2 §9.2.1.1 匿名块盒
//!    模型），va:bottom 盒 y < 前者底边是 vertical-align 对齐的行盒内几何而非重叠。
//!    旧行为把 IFC 定位（bottom 对齐）吞回 taffy 块堆叠位（box-offsets-rel-pos 谱系 +
//!    border-image-repeat-round-003 族 ref 页 box2 16→224 实证）。
//! ② taffy 块堆叠位到 IFC 行位的同步由 adjust_inline_block_positions 消费（既有 pass），
//!    本测试断言终态。

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

/// R5095：inline-table #a（baseline，高 224）+ inline-table #b（va:bottom，高 100）
/// 共享同一行盒——#b 底边对齐行盒底（= #a 底边），y = 224 − 100 = 124。
/// 旧行为：shift pass 把 #b「重叠下推」回 taffy 块堆叠位 224（y = #a 底边）。
#[test]
fn test_r5095_inline_table_bottom_aligned_stays_in_line_box() {
    let html = r##"<html><head><style>
      body { margin: 0; }
      div { display: inline-table; table-layout: fixed; }
      #a { height: 224px; width: 224px; }
      #b { height: 100px; width: 208px; vertical-align: bottom; }
    </style></head><body>
      <div id="a"></div><div id="b"></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a = find("a", &doc, &result.root).expect("inline-table #a");
    let b = find("b", &doc, &result.root).expect("inline-table #b");
    // va:bottom：#b 底边 = 行盒底 = #a 底边（#a 是行内最高盒，行盒高 = 224）。
    let expected_y = a.height - b.height;
    assert!(
        (b.y - expected_y).abs() < 0.5,
        "R5095: va:bottom inline-table #b y={} should be {} (= #a height {} − #b height {}); \
         old shift pass pushed it back to taffy block-stacked y={} (ate the IFC line-box position)",
        b.y,
        expected_y,
        a.height,
        b.height,
        a.height
    );
}

/// 对照组（防豁免面扩大）：inline-table prev + **块级** next 的重叠下推保留——
/// 前序 inline-table 长高后块级兄弟 taffy 滞后位仍须下移（R1498 对称语义）。
#[test]
fn test_r5095_inline_table_prev_block_next_still_shifts() {
    let html = r##"<html><head><style>
      body { margin: 0; }
      #a { display: inline-table; table-layout: fixed; height: 224px; width: 224px; }
      #b { display: block; height: 50px; width: 100px; }
    </style></head><body>
      <div id="a"></div><div id="b"></div>
    </body></html>"##;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let a = find("a", &doc, &result.root).expect("inline-table #a");
    let b = find("b", &doc, &result.root).expect("block #b");
    // 块级 next 不与原子行内盒同行盒（匿名块分隔），须在 #a 底边之下不重叠。
    assert!(
        b.y + 0.5 >= a.y + a.height,
        "R5095: block sibling #b (y={}) must stay below inline-table #a bottom ({}), \
         the atomic-pair exemption must not extend to block-level next",
        b.y,
        a.y + a.height
    );
}
