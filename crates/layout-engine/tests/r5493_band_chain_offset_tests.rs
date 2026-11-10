//! R5493 常驻锚（缺陷 Z）：nested float 级联 band 游标与 BFC strut 下限——CSS2
//! §9.5.1 × §10.6.7 × inline box 侧 margin 首 fragment 语义。chromium oracle
//! （/tmp/r5493-oracle fresh 15 形）定谳三规则：
//! 1. 候选目标带 band = max(锚行, prev 顶)（rule 2 顶不低于前 float）；
//! 2. 链偏移（wrapper chain Σ ml+bl+pl）只在目标带＝本 wrapper 首行时计入
//!    cursor——float 降带后同 wrapper 的后继 float 落在续行 fragment，不带链
//!    偏移（ml40 形 f3 band1 cursor 400 边界 fit 并排 (400,18)）；
//! 3. BFC 根 auto 高同样计零高行盒 strut：max(strut, float 带底)（p23 单
//!    10px float → 容器 18 非 10）。
//!
//! fit 判定非严格 ≤，fit 放置 x＝前 float 右缘，降带 x＝own ml／极值缘。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

/// float 位形断言条目：(span id, (x, y, w, h)) 绝对坐标。
type FloatRects<'a> = [(&'a str, (f32, f32, f32, f32))];

fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn abs_rect(root: &LayoutBox, id: zero_dom::NodeId) -> (f32, f32, f32, f32) {
    fn walk(b: &LayoutBox, id: zero_dom::NodeId, x: f32, y: f32) -> Option<(f32, f32, f32, f32)> {
        let ax = x + b.x + b.border_left + b.padding_left;
        let ay = y + b.y + b.border_top + b.padding_top;
        if b.node_id == Some(id) {
            return Some((ax, ay, b.width, b.height));
        }
        for c in &b.children {
            if let Some(r) = walk(c, id, ax, ay) {
                return Some(r);
            }
        }
        None
    }
    walk(root, id, 0.0, 0.0).expect("目标盒存在")
}

/// 断言 float 位形（绝对坐标，口径对齐 chromium gBCR）与 blk 容器高。
fn assert_form(html: &str, floats: &FloatRects<'_>, blk_w: f32, blk_h: f32) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let blk_id = doc.get_element_by_id("blk").expect("blk 存在");
    let (_, _, bw, bh) = abs_rect(&result.root, blk_id);
    assert!((bw - blk_w).abs() <= 0.5, "blk 宽应 {blk_w}，实际 {bw}");
    assert!((bh - blk_h).abs() <= 0.5, "blk 高应 {blk_h}，实际 {bh}");
    for (sid, want) in floats {
        let id = doc.get_element_by_id(sid).expect("float span 存在");
        let got = abs_rect(&result.root, id);
        assert!(
            (got.0 - want.0).abs() <= 0.5
                && (got.1 - want.1).abs() <= 0.5
                && (got.2 - want.2).abs() <= 0.5
                && (got.3 - want.3).abs() <= 0.5,
            "{sid} 位形应 {want:?}，实际 {got:?}"
        );
    }
}

fn three_f18(bfc: &str, ml: &str) -> String {
    format!(
        r#"<html><head><style>
body {{ margin: 0; }}
.block {{ display: {bfc}; background: green; }}
.inline {{ display: inline; {ml} }}
span {{ float: left; width: 50%; height: 18px; background: cyan; }}
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span><span id="s3"></span></div></div>
</body></html>"#
    )
}

/// 主形（p14_3f_colors oracle）：flow-root ＋ inline ml40 ＋ 3×50% float——f1
/// (0,0)；f2 band0 cursor 440 不 fit 降带 (0,18)；f3 band1（续行 fragment）不带
/// 链偏移，cursor 400 边界 fit 并排 (400,18)；blk＝float 带底 36。
#[test]
fn r5493_band1_fit_drops_chain_offset() {
    assert_form(
        &three_f18("flow-root", "margin-left: 40px;"),
        &[
            ("s1", (0.0, 0.0, 400.0, 18.0)),
            ("s2", (0.0, 18.0, 400.0, 18.0)),
            ("s3", (400.0, 18.0, 400.0, 18.0)),
        ],
        800.0,
        36.0,
    );
}

/// 非 BFC 同构（p13_block_3f oracle）：display:block 普通块同 float 位形，容器
/// 高＝strut 行盒 18（R5492 臂，float 溢出不计 §10.6.3）。
#[test]
fn r5493_non_bfc_strut_height_ignores_float_overflow() {
    assert_form(
        &three_f18("block", "margin-left: 40px;"),
        &[
            ("s1", (0.0, 0.0, 400.0, 18.0)),
            ("s2", (0.0, 18.0, 400.0, 18.0)),
            ("s3", (400.0, 18.0, 400.0, 18.0)),
        ],
        800.0,
        18.0,
    );
}

/// 4 float（p19_4f oracle）：f4 band1 cursor 800 不 fit → 二次降带 (0,36)，
/// blk＝带底 54。
#[test]
fn r5493_four_floats_second_drop_band() {
    let html = three_f18("flow-root", "margin-left: 40px;").replace(
        "<span id=\"s3\"></span>",
        "<span id=\"s3\"></span><span id=\"s4\"></span>",
    );
    assert_form(
        &html,
        &[
            ("s1", (0.0, 0.0, 400.0, 18.0)),
            ("s2", (0.0, 18.0, 400.0, 18.0)),
            ("s3", (400.0, 18.0, 400.0, 18.0)),
            ("s4", (0.0, 36.0, 400.0, 18.0)),
        ],
        800.0,
        54.0,
    );
}

/// ml0 回归（p21_ml0_3f oracle）：链偏移 0 时 f2 band0 cursor 400 边界 fit 并
/// 排 (400,0)、f3 降带 (0,18)——band0 逐 float 独立目标旧径不回归。
#[test]
fn r5493_ml0_band0_boundary_fit_regression() {
    assert_form(
        &three_f18("flow-root", ""),
        &[
            ("s1", (0.0, 0.0, 400.0, 18.0)),
            ("s2", (400.0, 0.0, 400.0, 18.0)),
            ("s3", (0.0, 18.0, 400.0, 18.0)),
        ],
        800.0,
        36.0,
    );
}

/// band0 第三 float 仍含链偏移（p20b_band0_ml oracle）：50%＋30% band0 cursor
/// 680（400＋240＋ml40）→ 130px 不 fit 降带 (0,18)——ml 只豁免降带后的续行，
/// band0 恒计入。
#[test]
fn r5493_band0_third_float_keeps_chain_offset() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; background: green; }
.inline { display: inline; margin-left: 40px; }
.f50 { float: left; width: 50%; height: 18px; background: cyan; }
.f30 { float: left; width: 30%; height: 18px; background: magenta; }
.f13 { float: left; width: 130px; height: 18px; background: yellow; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span class="f50" id="s1"></span><span class="f30" id="s2"></span><span class="f13" id="s3"></span></div></div>
</body></html>"#;
    assert_form(
        html,
        &[
            ("s1", (0.0, 0.0, 400.0, 18.0)),
            ("s2", (400.0, 0.0, 240.0, 18.0)),
            ("s3", (0.0, 18.0, 130.0, 18.0)),
        ],
        800.0,
        36.0,
    );
}

/// 右 float 镜像（p22_right_3f oracle）：f1 极值缘 (400,0)；f2 band0 avail
/// 360（400−ml40）不 fit 降带 (400,18)；f3 band1 avail 400 边界 fit (0,18)。
#[test]
fn r5493_right_float_band1_mirror() {
    let html = three_f18("flow-root", "margin-left: 40px;").replace("float: left", "float: right");
    assert_form(
        &html,
        &[
            ("s1", (400.0, 0.0, 400.0, 18.0)),
            ("s2", (400.0, 18.0, 400.0, 18.0)),
            ("s3", (0.0, 18.0, 400.0, 18.0)),
        ],
        800.0,
        36.0,
    );
}

/// BFC strut 下限（p23_flowroot_f10 oracle）：单 10px float 带底 10 < strut 18
/// → 容器高 18（§10.6.7 float 计入＋零高行盒 strut 参与）。
#[test]
fn r5493_bfc_strut_floor_beats_low_float() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; background: green; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 10px; background: cyan; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span></div></div>
</body></html>"#;
    assert_form(html, &[("s1", (0.0, 0.0, 400.0, 10.0))], 800.0, 18.0);
}

/// 带底压过 strut（p6/p7 形系）：150px float 带底 150 > strut → 容器高 150。
#[test]
fn r5493_bfc_float_bottom_beats_strut() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; background: green; }
.inline { display: inline; margin-left: 40px; }
span { float: left; width: 50%; height: 150px; background: cyan; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span></div></div>
</body></html>"#;
    assert_form(html, &[("s1", (0.0, 0.0, 400.0, 150.0))], 800.0, 150.0);
}
