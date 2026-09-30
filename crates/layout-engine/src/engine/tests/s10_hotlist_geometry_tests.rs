//! slice10：热榜行布局级中间几何钉值——float li > [float a > [inline-block 图标,
//! inline 标题 span], inline-block mark 兄弟]（百度热榜 li 形态的最小共享形状）。
//!
//! 钉的是 **布局层契约**（paint 层修复的依赖前提）：
//! 1. float li 行落 36px 网格（CSS2 §9.5 float 定位 + §10.6.7）；
//! 2. float a 盒占满 li 内容行（y=li.y，h=36）；
//! 3. inline-block 图标盒由 sync_inline_block_positions_from_ifc 对齐到 A 容器
//!    IFC 组合行（fs16/lh36 基线 ≈24.85 → 图标 fragment y=6.85；盒 y≈7 = 相对
//!    定位 top:-2 参与后的同步位）——此即布局层「IFC 已算对行位」的中间证据；
//! 4. mark（inline-block）作为 li 第二子盒存在（触发形态不可缺项）。
//!
//! 已知残余（不钉、防误锚化）：span 标题盒的 taffy 块堆叠位（+18）——盒级悬挂
//! 对无背景/边框文本不可见，标题墨迹 y 由 paint Path B 从 A 容器 IFC 片段取位
//!（见 engine s10_hotlist_paint_tests 墨迹级回归）。
//! CSS2 §9.2.1（inline formatting context）/ §9.5（floats）/ §10.8.1（行高与基线）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

const LI_STYLE: &str =
    "float:left;width:369px;height:36px;line-height:36px;font-size:12px;white-space:nowrap;clear:both;";
const A_STYLE: &str = "float:left;display:block;width:100%;height:36px;line-height:36px;font-size:14px;overflow:hidden;white-space:nowrap;";
const I_STYLE: &str = "display:inline-block;width:18px;height:18px;line-height:18px;font-size:18px;";
const T_STYLE: &str = "display:inline;line-height:36px;font-size:16px;";
const MARK_STYLE: &str = "display:inline-block;width:4px;height:16px;margin-left:4px;";

fn page() -> String {
    format!(
        r##"<html><body style="margin:0">
<ul style="list-style:none;margin:0;padding:0;width:760px;">
<li class="row" style="{LI_STYLE}"><a class="lnk" style="{A_STYLE}"><i class="ico" style="{I_STYLE}"></i><span class="ttl" style="{T_STYLE}">AAAA</span></a><span class="mk" style="{MARK_STYLE}"></span></li>
<li class="row" style="{LI_STYLE}"><a class="lnk" style="{A_STYLE}"><span class="ttl" style="{T_STYLE}">BBBB</span></a><span class="mk" style="{MARK_STYLE}"></span></li>
</ul>
</body></html>"##
    )
}

fn find_by_class<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, cls: &str) -> Vec<&'a LayoutBox> {
    let mut hits = Vec::new();
    fn walk<'a>(b: &'a LayoutBox, doc: &zero_dom::Document, cls: &str, hits: &mut Vec<&'a LayoutBox>) {
        if let Some(nid) = b.node_id
            && doc
                .get_attribute(nid, "class")
                .is_some_and(|v| v.split_whitespace().any(|c| c == cls))
        {
            hits.push(b);
        }
        for c in &b.children {
            walk(c, doc, cls, hits);
        }
    }
    walk(b, doc, cls, &mut hits);
    hits
}

#[test]
fn s10_hotlist_row_intermediate_geometry() {
    let html = page();
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 1. float li 行落 36px 网格（含 mark 兄弟不改变行高/行位）。
    let rows = find_by_class(&result.root, &doc, "row");
    assert_eq!(rows.len(), 2, "应有两个 li 行盒");
    for (i, r) in rows.iter().enumerate() {
        assert!(
            (r.y - 36.0 * i as f32).abs() < 0.5,
            "li[{i}] y 应在 36px 网格上（{}），实际 {}",
            36.0 * i as f32,
            r.y
        );
        assert!(
            (r.height - 36.0).abs() < 0.5,
            "li[{i}] 高度应 36（mark 兄弟不撑行），实际 {}",
            r.height
        );
    }

    // 2. float a 盒：嵌套 float 盒 y 按 li 内容原点存储（li-relative 0），高 36。
    for (i, lnk) in find_by_class(&result.root, &doc, "lnk").iter().enumerate() {
        assert!(
            lnk.y.abs() < 0.5 && (lnk.height - 36.0).abs() < 0.5,
            "a[{i}] 应贴 li 内容顶（嵌套 float 相对原点）且高 36，实际 y={} h={}",
            lnk.y,
            lnk.height
        );
    }

    // 3. 图标盒 = IFC 组合行同步位：fs16/lh36 基线 ≈24.85、图标 fragment y≈6.85
    //    （relative top:-2 参与前后盒 y≈6.85）。这是布局层「IFC 已算对行位并同步
    //    原子盒」的中间证据——若回归 taffy 堆叠位（≈18）则行内同步链断裂。
    let icons = find_by_class(&result.root, &doc, "ico");
    assert_eq!(icons.len(), 1, "应有一个图标盒");
    assert!(
        (icons[0].y - 6.85).abs() < 1.0,
        "图标盒应同步到 IFC 行位（≈6.85，基线 24.85 对齐 + va:middle），实际 {}（taffy 堆叠位 ≈18）",
        icons[0].y
    );

    // 4. 触发形态完整性：每行恰一个 mark 兄弟盒。
    for (i, mk) in find_by_class(&result.root, &doc, "mk").iter().enumerate() {
        assert!(
            (mk.height - 16.0).abs() < 0.5,
            "mark[{i}] 盒高应 16，实际 {}",
            mk.height
        );
    }
    assert_eq!(find_by_class(&result.root, &doc, "mk").len(), 2);
}
