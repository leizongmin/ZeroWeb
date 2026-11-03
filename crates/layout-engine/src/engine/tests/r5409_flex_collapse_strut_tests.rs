//! R5409：visibility:collapse flex item 多线 strut 迁移（css-flexbox-1 §4 Collapsed
//! Items，flexbox-collapsed-item-horiz-002）。
//!
//! 语义：collapsed item 以零主尺寸参与断线（可迁移到其他线），但携带「规范首趟
//! （按可见布局）所在线的交叉尺寸」作为 strut 贡献落线。kill-switch
//! `ZW_FLEX_COLLAPSE_STRUT_MIGRATE=0`（OFF 轮不设进程内单测——R3858 口径，
//! 由 corpus 002 ✗1.59% 精确回退覆盖）；converter 前提 `ZW_VC_NONFLEX_STRUT`
//! 关断时本臂自动惰性（同一进程内 env 竞态不可靠，同 R3858）。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn find_by_id<'a>(root: &'a LayoutBox, doc: &zero_dom::Document, id: &str) -> Option<&'a LayoutBox> {
    let target = doc.get_element_by_id(id);
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == target {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, LayoutBox) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut eng = LayoutEngine::new(800.0, 600.0);
    let r = eng.compute(&doc, &styles);
    (
        doc,
        std::sync::Arc::try_unwrap(r.root).unwrap_or_else(|arc| (*arc).clone()),
    )
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// ★load-bearing（002 Row2-C1 形态）：wrap 容器 30px 宽，[collapsed 15×25, full 30×20,
/// full 30×20]。首趟 collapsed 独占线（strut 25）；迁移后 collapsed（零主尺寸）并入
/// line1：line1 = collapsed+full1（交叉 25）、line2 = full2（y=25），容器高 45。
#[test]
fn r5409_collapsed_migrates_into_first_line() {
    let html = r#"<html><body style="margin:0">
<div id="c" style="display:flex;flex-wrap:wrap;width:30px;align-content:flex-start;align-items:flex-start">
  <div id="k" style="visibility:collapse;width:15px;height:25px"></div>
  <div id="f1" style="width:30px;height:20px"></div>
  <div id="f2" style="width:30px;height:20px"></div>
</div>
</body></html>"#;
    let (doc, root) = layout(html);
    let c = find_by_id(&root, &doc, "c").expect("container");
    assert!(approx(c.height, 45.0), "2 lines (25+20); got {}", c.height);
    let k = find_by_id(&root, &doc, "k").expect("collapsed");
    let f1 = find_by_id(&root, &doc, "f1").expect("full1");
    let f2 = find_by_id(&root, &doc, "f2").expect("full2");
    assert!(
        approx(k.y, 0.0) && approx(f1.y, 0.0),
        "line1 top: k {} f1 {}",
        k.y,
        f1.y
    );
    assert!(
        approx(f1.x, 0.0),
        "zero main-size collapsed packs full1 at x=0; got {}",
        f1.x
    );
    assert!(approx(f2.y, 25.0), "line2 at y=25; got {}", f2.y);
    assert!(approx(k.width, 0.0), "post-collapse main size 0; got {}", k.width);
}

/// ★load-bearing（002 Row3-C1 形态）：[full 30×20, collapsed 15×25, tall 15×40]。
/// 首趟 collapsed 与 tall 同线（线交叉 40）→ strut 40 随迁移进 line1：line1 交叉
/// 40（full+collapsed）、tall 落 line2（y=40），容器高 80。strut ≠ 自身高（25）。
#[test]
fn r5409_strut_carries_first_pass_line_cross_size() {
    let html = r#"<html><body style="margin:0">
<div id="c" style="display:flex;flex-wrap:wrap;width:30px;align-content:flex-start;align-items:flex-start">
  <div id="f" style="width:30px;height:20px"></div>
  <div id="k" style="visibility:collapse;width:15px;height:25px"></div>
  <div id="t" style="width:15px;height:40px"></div>
</div>
</body></html>"#;
    let (doc, root) = layout(html);
    let c = find_by_id(&root, &doc, "c").expect("container");
    assert!(
        approx(c.height, 80.0),
        "line1 cross 40 (strut) + line2 40; got {}",
        c.height
    );
    let k = find_by_id(&root, &doc, "k").expect("collapsed");
    let f = find_by_id(&root, &doc, "f").expect("full");
    let t = find_by_id(&root, &doc, "t").expect("tall");
    assert!(
        approx(k.height, 40.0),
        "strut 40 (first-pass line cross) replaces own 25; got {}",
        k.height
    );
    assert!(
        approx(f.y, 0.0) && approx(k.y, 0.0),
        "line1: full + migrated collapsed; got f {} k {}",
        f.y,
        k.y
    );
    assert!(approx(t.y, 40.0), "tall on line2 y=40; got {}", t.y);
}

/// 单线容器守卫（horiz-001 判例语义，R1834 basis-keep 现状冻结）：无 flex-wrap 时
/// 本臂不触发——converter strut basis 语义原样保留（collapsed 20×40 占主位 20，
/// short 打包其后 x=20；float shrink-to-fit 容器内容宽 20 为既存测量面）。多线
/// 迁移臂（本 pass）不得扰动该形态。单线零主尺寸候选面（001 ref 几何 = 主 0 +
/// strut，chromium oracle 2026-11-04 定谳 case/ref 两侧不一致、上游不过）另案挂账。
#[test]
fn r5409_nowrap_container_keeps_strut_basis() {
    let html = r#"<html><body style="margin:0">
<div id="c" style="display:flex;float:left;align-items:flex-start">
  <div id="k" style="visibility:collapse;width:20px;height:40px"></div>
  <div id="s" style="width:20px;height:10px"></div>
</div>
</body></html>"#;
    let (doc, root) = layout(html);
    let c = find_by_id(&root, &doc, "c").expect("container");
    let k = find_by_id(&root, &doc, "k").expect("collapsed");
    let s = find_by_id(&root, &doc, "s").expect("short");
    assert!(
        approx(c.width, 20.0),
        "single-line float width as measured; got {}",
        c.width
    );
    assert!(
        approx(c.height, 40.0),
        "strut maintains container cross size; got {}",
        c.height
    );
    assert!(
        approx(k.width, 20.0),
        "collapsed keeps basis width (R1834); got {}",
        k.width
    );
    assert!(approx(s.x, 20.0), "short packs after strut basis; got {}", s.x);
    assert!(approx(s.height, 10.0), "short own height; got {}", s.height);
}
