//! R5446：嵌套 inline float（wrapper span 子树内的 float）锚位 + 极值缘 + 容器高。
//!
//! §9.7 blockification 把 float span 算 display:Block → wrapper 命中
//! `has_block_level_child` → FLAT_CHILD_WALK 跳过 → float 从宿主 IFC 整体消失
//! （无 FloatAnchor → 锚 y 空 → 容器高塌、文本不绕排）。修复：谓词处排除 float
//!（CSS2 §9.3.1 离流盒，与 abs/fixed 同款）+ walk/flatten 臂发 FloatAnchor +
//! remeasure 嵌套目标回写 + compute_final 排除带并入。driving：
//! css/CSS2/floats/floats-placement-vertical-001b（✗3.64%→✓0.72%）。
//! `ZW_NESTED_INLINE_FLOAT=0` 回退旧行为。
//!
//! 坐标：LayoutBox.x/y 相对直接父级；inline wrapper 的 x 由 IFC sync 收紧到自身
//! 内容缘（可为非 0），嵌套 float 经 writeback 以 `目标 − 父链偏移` 反解保持同一
//! 绝对位。断言统一用 dump 同款父链累计 abs（x 累计含 padding/border）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

/// 返回目标盒与其父链累计绝对位（off_x/off_y 同 dump_layout_tree 的 walk 口径）。
fn find_abs(root: &LayoutBox, id: NodeId) -> Option<(&LayoutBox, f32, f32)> {
    fn walk(b: &LayoutBox, off_x: f32, off_y: f32, id: NodeId) -> Option<(&LayoutBox, f32, f32)> {
        let abs_x = off_x + b.x;
        let abs_y = off_y + b.y;
        if b.node_id == Some(id) {
            return Some((b, abs_x, abs_y));
        }
        let child_off_x = abs_x + b.padding_left + b.border_left;
        let child_off_y = abs_y + b.padding_top + b.border_top;
        for child in &b.children {
            if let Some(hit) = walk(child, child_off_x, child_off_y, id) {
                return Some(hit);
            }
        }
        None
    }
    walk(root, 0.0, 0.0, id)
}

fn compute(html: &str, ids: &[&'static str]) -> (crate::types::LayoutResult, Vec<(&'static str, NodeId)>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let pairs: Vec<(&'static str, NodeId)> = ids
        .iter()
        .map(|n| (*n, doc.get_element_by_id(n).expect("id")))
        .collect();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    (engine.compute(&doc, &styles), pairs)
}

/// 001b 形（wrapper 含前置文本 + float，Kitty 直挂）：float 绝对位=容器 content 左缘
///（§9.5.1 rule 6 方向极值）、p 高不被 float 撑高（§10.6.3 float 脱流不进普通块
/// auto 高；chromium oracle 真值 p_h=18.0，R5471 勘误——旧断言「≥50 容纳 float 底」
/// 是 nested float_bottom 折返时代的巧合固化，R5470 BFC 门翻默认后巧合断，R5446
/// 提交态即红非本轮回归）。
/// 旧缺陷签名：float 完全离场消失于 IFC + 容器高塌 0。
#[test]
fn r5446_nested_wrapper_float_at_left_edge_and_contained() {
    let html = r#"<html><body style="margin:0">
<p id="p" style="margin:0"><span>Hello<span id="f" style="float:left; width:50px; height:50px"></span></span>Kitty</p>
</body></html>"#;
    let (result, ids) = compute(html, &["p", "f"]);
    let p = find_abs(&result.root, ids[0].1).expect("p box");
    let f = find_abs(&result.root, ids[1].1).expect("float box");
    assert!(
        (f.1 - p.1).abs() < 0.5,
        "嵌套 float 应在容器 content 左缘（rule 6），实际 float abs x={} vs p abs x={}",
        f.1,
        p.1
    );
    assert!(
        p.0.height + 0.5 < 50.0,
        "p 高不应被 50px float 撑高（§10.6.3，chromium 18），实际 {}",
        p.0.height
    );
    // FIXME(pre-existing)：ZW 此形 2 行（36px）vs chromium 1 行（18px）——Kitty 换行
    // 域行排差异，行盒归属缺陷在册案的邻域，另案归因。
}

/// RTL wrapper（probe g 形）：dir=rtl 宿主内 float:right 落右缘
///（rule 6 与 direction 无关的 extreme inline-edge：Right→content 右缘 −mr）。
#[test]
fn r5446_rtl_wrapper_float_right_at_edge() {
    let html = r#"<html><body style="margin:0">
<p id="p" dir="rtl" style="margin:0"><span>سلام<span id="f" style="float:right; width:50px; height:50px"></span></span></p>
</body></html>"#;
    let (result, ids) = compute(html, &["p", "f"]);
    let p = find_abs(&result.root, ids[0].1).expect("p box");
    let f = find_abs(&result.root, ids[1].1).expect("float box");
    let right_edge = p.1 + p.0.width;
    assert!(
        (f.1 + f.0.width - right_edge).abs() < 0.5,
        "RTL float:right 应贴 content 右缘（abs x+w=右缘），实际 float abs x={} w={} 右缘={}",
        f.1,
        f.0.width,
        right_edge
    );
    assert!(
        p.0.height + 0.5 < 50.0,
        "p 高不应被 float 撑高（§10.6.3，chromium 同形 18），实际 {}",
        p.0.height
    );
}

/// 直挂 float 双堆叠回归卫（floats-006 形）：直接 float 子归既有 direct 路径，
/// 嵌套机制不得二次搬运（旧缺陷签名：第二个 float 被搬到极值缘与首个重叠）。
#[test]
fn r5446_direct_floats_stacking_untouched() {
    let html = r#"<html><body style="margin:0">
<div id="d" style="margin:0"><span id="a" style="float:left; width:50px; height:50px"></span><span id="b" style="float:left; width:50px; height:50px"></span></div>
</body></html>"#;
    let (result, ids) = compute(html, &["a", "b"]);
    let a = find_abs(&result.root, ids[0].1).expect("a box");
    let b = find_abs(&result.root, ids[1].1).expect("b box");
    assert!((a.1 - 0.0).abs() < 0.5, "首 float 应在左缘，实际 abs x={}", a.1);
    assert!(
        (b.1 - 50.0).abs() < 0.5,
        "次 float 应堆叠在首 float 右侧（abs x=50），实际 abs x={}",
        b.1
    );
}

/// floats-149 形（wrapper 只含 float、宿主无直系文本）：text float h=0 欠计补测
///（R3787 同款）→ 容器高 ≥ text float 底（绿条不消失）+ float 在 content 左缘。
/// 旧缺陷签名：容器高塌 0（float_bottom 恒 0）。
#[test]
fn r5446_float_only_wrapper_container_height_backfilled() {
    let html = r#"<html><body style="margin:0">
<div id="d" style="margin:0; background:green"><div style="display:inline; margin-left:40px"><span id="t" style="float:left; width:50%">text</span><span id="e" style="float:left; width:50%"></span></div></div>
</body></html>"#;
    let (result, ids) = compute(html, &["d", "t"]);
    let d = find_abs(&result.root, ids[0].1).expect("div.box");
    let t = find_abs(&result.root, ids[1].1).expect("text float box");
    assert!(
        (t.1 - 0.0).abs() < 0.5,
        "text float 应在 content 左缘，实际 abs x={}",
        t.1
    );
    assert!(
        d.0.height + 0.5 >= t.0.height && d.0.height >= 15.0,
        "容器高应 ≥ text float 底（补测后 {}），实际 {}",
        t.0.height,
        d.0.height
    );
}
