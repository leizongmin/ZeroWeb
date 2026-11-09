//! R5489：物化 inline 层右 float 镜像序列级联＋fix 载体修正认领豁免（CSS2 §9.5.1）。
//!
//! 蓝本＝R5488 chromium headless 五形 oracle（400 容器两 right float；断言 ABS 坐标
//! ＝LayoutBox 相对父编码沿父链累加 (x+bl+pl)，口径对齐 chromium gBCR）：①r_orig
//! （inline 层 ml=40，两 50%=200）堆叠 (200,0)+(200,18)；②r_c（ml=0）并排
//! (200,0)+(0,0)；③r_a（ml=40，40%=160）并排 (240,0)+(80,0)——fit 放**前 float 左
//! 缘−w**（镜像左 float fit 放前右缘；fit 判定可用＝前左缘−chain_offset）；④r_d
//! （两独立 inline 层各一 float）堆叠；⑤直子对照（direct 路径不回归）；⑥
//! inline-block 载体对照（fix 豁免谓词 BFC 判定天然 false，域逐字节不变）。
//!
//! 修复＝双臂：nested 目标循环右 float 镜像 placed_r 序列（R5488 缺陷 B 级联缺失）
//! ＋`fix_inline_carrier_right_float_abs` 对可下降 wrapper 内右 float 认领豁免
//! （R5488 缺陷 A＝该 pass content_left_abs 层 margin 双计侵蚀 x＋逐 float 独立
//! 覆写毁级联）。pre-fix 红证据谱系：五形两 float 重叠同位且 r_orig/r_a x 侵蚀
//! −40（160/200）。kill-switch：`ZW_NESTED_FLOAT_CASCADE=0`、
//! `ZW_NESTED_INLINE_FLOAT=0`（独立进程锚见 r5489_*_kill_switch.rs）。

use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::StyleSystem;

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

fn compute(html: &str) -> zero_layout_engine::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn span_id(doc: &zero_dom::Document, name: &str) -> zero_dom::NodeId {
    doc.get_element_by_id(name).expect("span 存在")
}

const TPL: &str = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; width: 400px; }
.inline { display: inline; MARGIN }
span { float: right; SPANW height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="inline"><span id="s1"></span><span id="s2"></span></div></div>
</body></html>"#;

fn blk_height(html: &str) -> f32 {
    let doc = zero_dom::parse_html(html);
    let blk = doc.get_element_by_id("blk").expect("blk 存在");
    let result = compute(html);
    abs_rect(&result.root, blk).3
}

/// ①r_orig（oracle 堆叠形）：ml=40 两 50%=200——可用=200−40=160<200 → float2
/// 降带 (200,18)；pre-fix 两 float 重叠 (160,0)×2（x 侵蚀＋零级联）红证据谱系。
#[test]
fn r5489_right_cascade_orig_stacks_second_float() {
    let html = TPL
        .replace("MARGIN", "margin-left: 40px;")
        .replace("SPANW", "width: 50%; ");
    let result = compute(&html);
    let doc = zero_dom::parse_html(&html);
    let f1 = abs_rect(&result.root, span_id(&doc, "s1"));
    assert!(
        (f1.0 - 200.0).abs() <= 0.5 && f1.1.abs() <= 0.5,
        "float1 极值缘应 (200,0)（chromium oracle；pre-fix x 侵蚀 160），实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 200.0).abs() <= 0.5 && (f2.1 - 18.0).abs() <= 0.5,
        "float2 应降带堆叠 (200,18)（chromium oracle；pre-fix 重叠 (160,0)），实际 {:?}",
        f2
    );
    assert!(
        (blk_height(&html) - 36.0).abs() <= 0.5,
        "两堆叠带计入容器高应 36（oracle 带至 y 35；pre-fix 18），实际 {}",
        blk_height(&html)
    );
}

/// ②r_c（oracle 并排形）：ml=0——可用=200−0=200≥200 → fit 并排 (0,0)，单带高 18。
#[test]
fn r5489_right_cascade_ml0_side_by_side() {
    let html = TPL
        .replace("MARGIN", "margin-left: 0px;")
        .replace("SPANW", "width: 50%; ");
    let result = compute(&html);
    let doc = zero_dom::parse_html(&html);
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        f2.0.abs() <= 0.5 && f2.1.abs() <= 0.5,
        "float2 应并排放前 float 左缘−w (0,0)（chromium oracle；pre-fix 重叠 (200,0)），实际 {:?}",
        f2
    );
    assert!(
        (blk_height(&html) - 18.0).abs() <= 0.5,
        "单带并排容器高应 18，实际 {}",
        blk_height(&html)
    );
}

/// ③r_a（oracle fit 左缘形）：ml=40，40%=160——可用=240−40=200≥160 → fit 放**前
/// float 左缘−w** (80,0)（非光标位）；pre-fix 重叠 (200,0)×2 红证据谱系。
#[test]
fn r5489_right_cascade_fit_places_at_prev_left_edge() {
    let html = TPL
        .replace("MARGIN", "margin-left: 40px;")
        .replace("SPANW", "width: 40%; ");
    let result = compute(&html);
    let doc = zero_dom::parse_html(&html);
    let f1 = abs_rect(&result.root, span_id(&doc, "s1"));
    assert!(
        (f1.0 - 240.0).abs() <= 0.5 && f1.1.abs() <= 0.5,
        "float1 极值缘应 (240,0)（chromium oracle；pre-fix x 侵蚀 200），实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 80.0).abs() <= 0.5 && f2.1.abs() <= 0.5,
        "fit float2 应放前 float 左缘−w (80,0)（chromium oracle 定谳），实际 {:?}",
        f2
    );
    assert!(
        (blk_height(&html) - 18.0).abs() <= 0.5,
        "单带并排容器高应 18，实际 {}",
        blk_height(&html)
    );
}

/// ④r_d（oracle 双层堆叠形）：两独立 inline 层各一 right float——镜像级联降带
/// (200,18)；pre-fix 重叠红证据谱系。
#[test]
fn r5489_right_cascade_two_layers_stack() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; width: 400px; }
.inline { display: inline; margin-left: 40px; }
span { float: right; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk">
  <div class="inline"><span id="s1"></span></div>
  <div class="inline"><span id="s2"></span></div>
</div>
</body></html>"#;
    let result = compute(html);
    let doc = zero_dom::parse_html(html);
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 200.0).abs() <= 0.5 && (f2.1 - 18.0).abs() <= 0.5,
        "双层形 float2 应降带堆叠 (200,18)（chromium oracle；pre-fix 重叠），实际 {:?}",
        f2
    );
    assert!(
        (blk_height(html) - 36.0).abs() <= 0.5,
        "双层堆叠容器高应 36（oracle 带至 y 35），实际 {}",
        blk_height(html)
    );
}

/// ⑤直子对照（R5488 已正确路径防回归）：float 块容器直子两 50% 并排
/// (200,0)+(0,0)——direct 路径不受嵌套臂/豁免影响。
#[test]
fn r5489_direct_float_children_stay_side_by_side() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; width: 400px; }
span { float: right; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><span id="s1"></span><span id="s2"></span></div>
</body></html>"#;
    let result = compute(html);
    let doc = zero_dom::parse_html(html);
    let f1 = abs_rect(&result.root, span_id(&doc, "s1"));
    assert!(
        (f1.0 - 200.0).abs() <= 0.5 && f1.1.abs() <= 0.5,
        "直子 float1 应 (200,0)，实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        f2.0.abs() <= 0.5 && f2.1.abs() <= 0.5,
        "直子 float2 应并排 (0,0)（R5488 对照形 oracle＝已正确路径），实际 {:?}",
        f2
    );
}

/// ⑥inline-block 载体对照（fix 豁免防误伤锚）：BFC 载体（descendable=false）内
/// 右 float 走自身容器 direct 路径——修复前后逐字节不变（(220,0)+(40,0)）。
#[test]
fn r5489_inline_block_carrier_domain_unchanged() {
    let html = r#"<html><head><style>
body { margin: 0; }
.block { display: flow-root; width: 400px; }
.carrier { display: inline-block; margin-left: 40px; }
span { float: right; width: 50%; height: 18px; }
</style></head><body>
<div class="block" id="blk"><div class="carrier"><span id="s1"></span><span id="s2"></span></div></div>
</body></html>"#;
    let result = compute(html);
    let doc = zero_dom::parse_html(html);
    let f1 = abs_rect(&result.root, span_id(&doc, "s1"));
    assert!(
        (f1.0 - 220.0).abs() <= 0.5 && f1.1.abs() <= 0.5,
        "inline-block 载体域 float1 应保持 (220,0)（BFC 直子路径不受豁免影响），实际 {:?}",
        f1
    );
    let f2 = abs_rect(&result.root, span_id(&doc, "s2"));
    assert!(
        (f2.0 - 40.0).abs() <= 0.5 && f2.1.abs() <= 0.5,
        "inline-block 载体域 float2 应保持 (40,0)，实际 {:?}",
        f2
    );
}
