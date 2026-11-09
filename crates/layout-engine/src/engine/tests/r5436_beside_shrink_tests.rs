//! R5436（CSS2 §10.3.5/§17.5.2 + §9.5）：float 旁 auto-width 表 beside 目标宽收口。
//!
//! 现象（floats-wrap-bfc-006 T2/T4 REFTEST_DUMP 像素 + bfc-002-left-table 基线）：C 步
//! beside 臂对 auto-width 表无条件填 gap（R1613 语义）——caption 190 > cell 100 的表被
//! 填成 gap 宽（chromium 表盒 = grid 宽，caption 溢出表盒），definite-exact grid（cell
//! 唯一 definite 宽子）同理（grid 227 < gap 230 应保 pref）。bfc-002 系（spans run 度量
//! 欠测算 150）靠填 gap 到 200，须维持。
//!
//! 机制：B 步 layout_table 在盒上记录纯 grid extent（`table_grid_max_width`），C 步
//! beside 目标宽 = min(grid extent, gap)，但仅当 `grid_extent_exact`（每 cell 恰一个
//! definite Px 宽元素子、无直接文本——欠度量形态回退填 gap）。
//! kill-switch `ZW_TABLE_FLOAT_BESIDE_SHRINK=0` 回退 beside 一律填 gap。
//!
//! 断言取结构不变式（definite 宽、beside x 带内、fill 保持），不锚像素（harness
//! estimate metrics 域差异见 R5426 判例）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_style_system::StyleSystem;

/// 按 id 属性定位盒（仓库既有 idiom，含 doc.get 校验）。
fn find<'a>(id: &str, doc: &zero_dom::Document, b: &'a LayoutBox) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(n) = doc.get(nid)
        && let zero_dom::NodeKind::Element(elem) = &n.kind
        && elem.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find(id, doc, c))
}

/// 真管线探针：布局后取 (width, x)。
fn probe_width_x(html: &str, id: &str) -> (f32, f32) {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let b = find(id, &doc, &result.root).expect("probe box not found");
    (b.width, b.x)
}

/// ★ driving 案形（floats-wrap-bfc-006 T2 同构）：caption definite 宽 190 > grid 100，
/// beside 落位后表盒宽应 = grid extent 100（非填 gap 190、非 caption 190），
/// caption 溢出表盒语义（§17.4.1）下 caption 盒保持 definite 190。
#[test]
fn r5436_beside_table_shrinks_to_grid_extent_with_caption_overflow() {
    const HTML: &str = r#"<!DOCTYPE html><html><head><style>table{margin:0;border-spacing:0}</style></head><body style="margin:0">
<table width="300"><tr><td>
<div style="float:left; clear:left; width:150px; height:1px"></div>
<div style="float:left; clear:left; width:145px; height:1px"></div>
<div style="float:left; clear:left; width:140px; height:1px"></div>
<div style="float:left; clear:left; width:135px; height:1px"></div>
<div style="float:left; clear:left; width:130px; height:1px"></div>
<div style="float:left; clear:left; width:125px; height:1px"></div>
<div style="float:left; clear:left; width:120px; height:1px"></div>
<div style="float:left; clear:left; width:115px; height:1px"></div>
<div style="float:left; clear:left; width:110px; height:1px"></div>
<div style="float:left; clear:left; width:105px; height:1px"></div>
<div style="float:left; clear:left; width:100px; height:1px"></div>
<div style="float:left; clear:left; width:95px; height:1px"></div>
<div style="float:left; clear:left; width:90px; height:1px"></div>
<div style="float:left; clear:left; width:85px; height:1px"></div>
<div style="float:left; clear:left; width:80px; height:1px"></div>
<div style="float:left; clear:left; width:75px; height:1px"></div>
<div style="float:left; clear:left; width:70px; height:1px"></div>
<div style="float:left; clear:left; width:65px; height:1px"></div>
<div style="float:left; clear:left; width:60px; height:1px"></div>
<div style="float:left; clear:left; width:55px; height:1px"></div>
<div style="float:left; clear:left; width:50px; height:1px"></div>
<div style="float:left; clear:left; width:45px; height:1px"></div>
<div style="float:left; clear:left; width:40px; height:1px"></div>
<div style="float:left; clear:left; width:35px; height:1px"></div>
<div style="float:left; clear:left; width:30px; height:1px"></div>
<div style="float:left; clear:left; width:25px; height:1px"></div>
<div style="float:left; clear:left; width:20px; height:1px"></div>
<div style="float:left; clear:left; width:15px; height:1px"></div>
<div style="float:left; clear:left; width:10px; height:1px"></div>
<div style="float:left; clear:left; width:5px; height:1px"></div>
<table id="nst" style="background:purple">
<caption style="caption-side:top; height:30px; width:190px; background:yellow">Caption</caption>
<tr><td><div style="height:30px; width:100px">Cell</div></td></tr>
</table>
</td></tr></table>
</body></html>"#;
    let (w, x) = probe_width_x(HTML, "nst");
    // 表盒宽 = grid extent（cell div definite 100），非填 gap 230、非 caption-inclusive 190。
    // 容差 3：单测环境无 UA 默认（border-spacing 残差 2px；wpt 管线为 100 chrome-exact）。
    assert!(
        (w - 100.0).abs() < 3.0,
        "beside 表盒应缩到 grid extent 100（±spacing 残差），got {w}"
    );
    // beside 落位：x = 左 float 带右缘（非 0 推下形态）。
    assert!(x > 0.5, "表应 beside 左 float 带，got x={x}");
    // caption 溢出表盒：盒树内 caption 保持 definite 190。
    let doc = zero_dom::parse_html(HTML);
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let nst = find("nst", &doc, &result.root).expect("nested table");
    let cap = nst.children.iter().find(|c| c.is_table_caption).expect("caption child");
    assert!((cap.width - 190.0).abs() < 0.5, "caption 应保持 190，got {}", cap.width);
}

/// ★ fill 保持卫（floats-wrap-bfc-002-left-table 同构）：cell 含两个显式宽 inline-block
///（R5261 run 被空白断，欠度量形态）→ `grid_extent_exact` false → beside 维持填 gap
/// 语义（R1613），表宽 = float 右可用宽 200，不缩到欠度量的 150。
#[test]
fn r5436_beside_fill_kept_for_non_exact_grid() {
    const HTML: &str = r#"<!DOCTYPE html><html><head><style>table{margin:0;border-spacing:0}</style></head><body style="margin:0">
<table width="300"><tr><td>
<div style="float:left; width:100px; height:100px; background:blue"></div>
<table id="nst2" style="background:yellow"><tr><td>
<span style="display:inline-block; width:150px; height:50px; background:purple"></span>
<span style="display:inline-block; width:150px; height:50px; background:purple"></span>
</td></tr></table>
</td></tr></table>
</body></html>"#;
    let (w, _x) = probe_width_x(HTML, "nst2");
    // 填 gap 保持：宽 = float 右可用宽（单测环境 298 可用 → 198，WPT 管线 200 chrome-exact）；
    // 若误缩到欠度量 grid（≈150）则语义破坏，故下界取 190（caption-inclusive 域之上）。
    assert!(w > 190.0, "欠度量 grid 须维持填 gap 语义（>190），got {w}");
}
