//! caret 收敛形常驻 fixture（zinc B-1 返修：fresh/incremental 与 heal 物化对称 +
//! 绘制源真理单 caret）。github trending 形：`summary{list-style:none}` 抑制原生
//! disclosure 标记 + `::after` border 三角（content:"" + inline-block）自绘 caret。
//!
//! 断言三面：
//! 1. 全量渲染（heal 臂同构路径 render_html → inject）：caret 色 path_fill 恰 1
//!    （::after 三角物化），且 summary 盒内 path_fill 总数恰 1（原生 disclosure
//!    标记被 list-style:none 抑制——`paint_summary_marker` 绘制源真理门；
//!    css-lists-3 list-style-type:none 抑制标记语义）；
//! 2. `set_viewport` + `repaint_cached_viewport` 增量路径（live fresh 臂同链：
//!    Emulation.setDeviceMetricsOverride → set_viewport → render_incremental →
//!    incremental_paint）：summary / caret / .rgt 几何与全量全等（TOL 0.5）——
//!    重级联覆盖 styles 后 `regraft_pseudo_standin_styles` 回 graft 替身样式，
//!    caret 以原子盒参与布局（旧缺陷形 caret 失样式塌为 inline 文本、summary
//!    收窄、caret 色填充 0 个）；
//! 3. 增量路径 caret 色填充同样恰 1（fresh 不丢物化）。

use zero_engine::{RenderPipeline, RenderResult};
use zero_layout_engine::LayoutBox;

fn walk_abs(box_node: &LayoutBox, target: zero_dom::NodeId, ox: f32, oy: f32) -> Option<(&LayoutBox, f32, f32)> {
    let ax = ox + box_node.x;
    let ay = oy + box_node.y;
    if box_node.node_id == Some(target) {
        return Some((box_node, ax, ay));
    }
    box_node.children.iter().find_map(|c| walk_abs(c, target, ax, ay))
}

const TOL: f32 = 0.5;
// caret ::after 的 color（border:2px solid 取 currentColor）——fixture 内唯一色，
// path_fill 颜色过滤零歧义。
const CARET_R: u8 = 208;
const CARET_G: u8 = 74;
const CARET_B: u8 = 55;

const CSS: &str = "body{margin:0;font-size:16px}\
    .rgt{display:flex;align-items:center;justify-content:flex-end}\
    summary{list-style:none;white-space:nowrap}\
    summary:before{content:\"\";display:none}\
    summary:after{content:\"\";display:inline-block;width:0;height:0;\
border:2px solid;border-bottom-color:#0000;border-left-color:#0000;\
border-right-color:#0000;color:#d04a37;margin-left:4px}";

const FRAG: &str = "<div class=\"rgt\"><details><summary>Spoken Language: \
<span style=\"font-weight:700\">Any</span></summary></details></div>";

/// summary 盒内（几何包含）path_fill census：返回 (caret 色填充数, 填充总数, caret
/// 色填充联合 bbox)。caret 三角（painter 可拆两个半边三角形）与原生 disclosure 标记
/// （若未被抑制）都落在 summary 盒内；正文/背景类大填充不包含于其中。「单 caret」
/// 判据 = 填充总数 == caret 色填充数（summary 内无任何第三方填充 = 原生标记被
/// list-style:none 抑制，css-lists-3），不冻结 painter 的三角形拆分数。
fn fills_within_summary(r: &RenderResult, summary: (f32, f32, f32, f32)) -> (usize, usize, [f32; 4]) {
    let (sx, sy, sw, sh) = summary;
    let mut caret = 0usize;
    let mut total = 0usize;
    let mut union: Option<[f32; 4]> = None;
    for p in &r.primitives().path_fills {
        if p.vertices.len() < 4 {
            continue;
        }
        let xs = p.vertices.as_chunks::<2>().0.iter().map(|c| c[0]);
        let ys = p.vertices.as_chunks::<2>().0.iter().map(|c| c[1]);
        let min_x = xs.clone().fold(f32::MAX, f32::min);
        let max_x = xs.fold(f32::MIN, f32::max);
        let min_y = ys.clone().fold(f32::MAX, f32::min);
        let max_y = ys.fold(f32::MIN, f32::max);
        let within = min_x >= sx - 0.5 && min_y >= sy - 0.5 && max_x <= sx + sw + 0.5 && max_y <= sy + sh + 0.5;
        if !within {
            continue;
        }
        // 全透明填充（alpha=0，边框透明侧的形状图元）不可见，不入判据——原生
        // disclosure 标记以 currentColor 不透明绘制，仍会被总数捕获。
        if p.color.a == 0 {
            continue;
        }
        total += 1;
        eprintln!(
            "[census] fill #{total} color rgba({},{},{},{}) bbox [{min_x},{min_y},{max_x},{max_y}]",
            p.color.r, p.color.g, p.color.b, p.color.a
        );
        if (p.color.r, p.color.g, p.color.b) == (CARET_R, CARET_G, CARET_B) {
            caret += 1;
            union = Some(match union {
                None => [min_x, min_y, max_x, max_y],
                Some([ux0, uy0, ux1, uy1]) => [ux0.min(min_x), uy0.min(min_y), ux1.max(max_x), uy1.max(max_y)],
            });
        }
    }
    (caret, total, union.unwrap_or_default())
}

fn assert_single_caret(r: &RenderResult, summary_geom: (f32, f32, f32, f32), tag: &str) -> [f32; 4] {
    let (caret, total, union) = fills_within_summary(r, summary_geom);
    eprintln!(
        "[{tag}] summary 内填充 caret 色 {caret} / 总数 {total} union {:?} summary {summary_geom:?}",
        union
    );
    assert!(
        caret >= 1,
        "[{tag}] caret 色 path_fill 缺失（::after 三角未物化；fresh 失样式=0）"
    );
    assert_eq!(
        total, caret,
        "[{tag}] summary 盒内出现非 caret 填充（原生 disclosure 标记未被 \
         list-style:none 抑制，css-lists-3 绘制源真理）"
    );
    union
}

#[test]
fn caret_materializes_identically_on_full_and_incremental_paths() {
    let html = format!("<!doctype html><html><head><style>{CSS}</style></head><body>{FRAG}</body></html>");
    let mut p = RenderPipeline::new(1280.0, 800.0);
    let r1 = p.render_html(&html, "");
    let doc_rc = p.cached_doc_shared().expect("cached doc after render_html");
    let doc = doc_rc.borrow();

    // caret 替身 = summary 尾子（::after 注入契约；::before display:none 首子）。
    let summary_id = doc.query_selector(doc.root(), "summary").expect("summary in dom");
    let caret_id = doc
        .get(summary_id)
        .and_then(|n| n.children.last().copied())
        .expect("caret standin as summary last child");

    let geom_of = |root: &LayoutBox, id: zero_dom::NodeId| -> (f32, f32, f32, f32) {
        let (b, x, y) = walk_abs(root, id, 0.0, 0.0).expect("box in layout");
        (x, y, b.width, b.height)
    };

    // ① 全量（heal 同构）：caret 物化 + 原生标记被抑制 + caret 以原子盒参与布局。
    let (s1x, s1y, s1w, s1h) = geom_of(&r1.layout.root, summary_id);
    let (c1x, c1y, c1w, c1h) = geom_of(&r1.layout.root, caret_id);
    let rgt1 = doc.query_selector(doc.root(), ".rgt").expect(".rgt in dom");
    let g1 = geom_of(&r1.layout.root, rgt1);
    eprintln!("[full] summary {s1w}x{s1h}@({s1x},{s1y}) caret {c1w}x{c1h}@({c1x},{c1y})");
    assert!(
        c1w >= 2.0 && c1h >= 2.0,
        "caret 替身应为原子盒（width:0 + border 2px → 4x4），实际 {c1w}x{c1h}——伪元素盒化样式丢失"
    );
    let caret_union1 = assert_single_caret(&r1, (s1x, s1y, s1w, s1h), "full");

    // ② fresh 臂同链：viewport 变更 → 增量 repaint（1281）→ 还原 1280 → 增量
    //    repaint（同 live19 观测序：setDeviceMetricsOverride 1281 → clear → 1280）。
    p.set_viewport(1281.0, 800.0);
    let _r_mid = p.repaint_cached_viewport("").expect("incremental repaint at 1281");
    p.set_viewport(1280.0, 800.0);
    let r2 = p.repaint_cached_viewport("").expect("incremental repaint at 1280");

    // ③ 两路径几何全等 + 增量臂 caret 物化不丢。
    let (s2x, s2y, s2w, s2h) = geom_of(&r2.layout.root, summary_id);
    let (c2x, c2y, c2w, c2h) = geom_of(&r2.layout.root, caret_id);
    let g2 = geom_of(&r2.layout.root, rgt1);
    eprintln!("[incr] summary {s2w}x{s2h}@({s2x},{s2y}) caret {c2w}x{c2h}@({c2x},{c2y})");
    let cmp = |name: &str, a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)| {
        assert!(
            (a.0 - b.0).abs() < TOL && (a.1 - b.1).abs() < TOL && (a.2 - b.2).abs() < TOL && (a.3 - b.3).abs() < TOL,
            "{name} 增量重算与全量渲染分歧：full {:?} vs incremental {:?}",
            a,
            b
        );
    };
    cmp("summary", (s1x, s1y, s1w, s1h), (s2x, s2y, s2w, s2h));
    cmp("caret", (c1x, c1y, c1w, c1h), (c2x, c2y, c2w, c2h));
    cmp(".rgt", g1, g2);
    assert!(
        c2w >= 2.0 && c2h >= 2.0,
        "增量路径 caret 替身应为原子盒，实际 {c2w}x{c2h}——regraft 失效（styles 重级联后替身条目未回 graft）"
    );
    let caret_union2 = assert_single_caret(&r2, (s2x, s2y, s2w, s2h), "incr");
    // 绘制面收敛：两臂 caret 色填充联合 bbox 全等（单 caret 且同位）。
    for (i, (a, b)) in caret_union1.iter().zip(caret_union2.iter()).enumerate() {
        assert!(
            (a - b).abs() < TOL,
            "caret 绘制联合 bbox 维度 {i} 两臂分歧：full {a} vs incr {b}"
        );
    }
}
