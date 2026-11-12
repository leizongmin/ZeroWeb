//! R5508（css-sizing-3 §5.2 used-value 约束）：max-width/min-width 裸 `fit-content`
//! 关键字的 fit-content 公式语义——`min(W_max, max(W_min, stretch-fit))`。
//!
//! 缺陷机制（三面同根）：apply 层裸 fit-content 经共享 parse_length 映射 MaxContent
//!（R1018 别名），max-width 域丢失 `max(W_min, avail)` 地板：
//! ① `width:300px + max-width:fit-content` + 不可断子 90px 于 50px CB——used 应 90
//!   （fit-content 值 = min(90, max(90,50)) = 90，溢出容器），旧写入臂清 size.width=auto
//!   后 taffy 块级拉伸走 stretch-fit 解析基 → 塌 min(CB=50, target) = 50；
//! ② `max-width:max-content` + 定宽 > 内容 max-content 于窄 CB——max-content cap 无
//!   avail 地板，used 应 max-content（50px CB 内 2×50px 原子子 → 100），旧法 auto 清法
//!   塌 CB 地板 50；
//! ③ min-width 裸 fit-content 窄 CB 地板（floor 应 min(W_max, max(W_min, CB)) 非
//!   W_max——R5507 臂的 max(声明宽, 内容测量) 在 minc < maxc 时过抬）。
//!
//! 修复：apply 层裸 fit-content（min/max-width 物理水平轴槽位）映射 FitContent(100%)
//!（≡ fit-content(stretch-fit)，与显式 fit-content(100%) 拼写收敛）＋ sizing pass
//! 公式臂 % arg 按 cb_w 解析（fit_target = min(W_max, max(W_min, cb_w))）＋写入臂保留
//! converter size.width 不再清 auto（taffy 约束求解 max ≥ size 取小/ min ≥ size 取大）。
//!
//! chromium oracle 定谳（/tmp 探针 19 形，2026-11-12）：S9=90 S12=132.5 S14=90
//! S15=100 S15b=50 S16=50 S17=100（宽 CB R5507 驱动不受影响）。
//! 锚全用定宽原子子，字体无关。
//!
//! 谱系：R1018（bare fit-content→MaxContent）、R4149/R4150（宽轴关键字约束写入）、
//! R5036（fit-content(L) 公式臂＋% arg 排除）、R5037（content-only 贡献口径）、
//! R5375（max-width:max-content 内容基贡献）、R5506（grow 腿自身 max-width 钳）、
//! R5507（kw_min 写入保留 size.width）。本钉收裸 fit-content × min/max-width ×
//! 窄 CB/定宽交叉域（stretch-fit 地板语义整体缺失）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.width);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// 主判例（修复前塌 CB 地板 50）：50px 容器内 `width:300px; max-width:fit-content`
/// 含单个 90px 定宽 inline-block——fit-content 值 = min(90, max(90, 50)) = 90，
/// used = min(300, 90) = 90（溢出容器，不是 stretch-fit 50）。
#[test]
fn r5508_max_fitcontent_keeps_definite_width_in_narrow_cb() {
    let html = r#"<html><body style="margin:0">
<div style="width: 50px;">
  <div style="max-width: fit-content; width: 300px;">
    <span style="display: inline-block; width: 90px;"></span>
  </div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.get(1).expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 90.0).abs() < 0.5,
        "R5508: 窄 CB 定宽 300 × max-width:fit-content × 原子子 90 应 min(300, 90)=90（修复前塌 CB 50），实际 {w}"
    );
}

/// max-content cap 无 avail 地板（修复②）：50px 容器内 `width:300px;
/// max-width:max-content` 含 2×50px 原子子——max-content = 100，used = min(300, 100)
/// = 100（旧 auto 清法塌 CB 50）。
#[test]
fn r5508_max_content_cap_not_cb_floored_in_narrow_cb() {
    let html = r#"<html><body style="margin:0">
<div style="width: 50px;">
  <div style="max-width: max-content; width: 300px;">
    <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
  </div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.get(1).expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R5508: max-content cap 应 100 非 CB 地板 50（修复前 50），实际 {w}"
    );
}

/// fit-content 窄 CB 公式地板（修复①公式侧）：50px 容器内 `width:300px;
/// max-width:fit-content` 含 2×50px 原子子——fit-content 值 = min(100, max(50, 50))
/// = 50，used = min(300, 50) = 50（chromium 50 定谳；无定宽时公式才允许 CB 地板）。
#[test]
fn r5508_fitcontent_formula_floors_at_min_content() {
    let html = r#"<html><body style="margin:0">
<div style="width: 50px;">
  <div style="max-width: fit-content; width: 300px;">
    <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
  </div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.get(1).expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 50.0).abs() < 0.5,
        "R5508: fit-content 公式地板 min(W_max, max(W_min, avail)) = 50，实际 {w}"
    );
}

/// 宽 CB 定宽 cap 不受影响（回归守卫）：800px body 直下 `width:300px;
/// max-width:fit-content` 含单 90px 子——used = min(300, 90) = 90（R5506 域邻位）。
#[test]
fn r5508_wide_cb_cap_unchanged() {
    let html = r#"<html><body style="margin:0">
<div style="max-width: fit-content; width: 300px;">
  <span style="display: inline-block; width: 90px;"></span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.first().expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 90.0).abs() < 0.5,
        "R5508: 宽 CB cap min(300, 90) = 90 保持（写入臂去 auto 清法不回抬定宽），实际 {w}"
    );
}

/// min-width 裸 fit-content 窄 CB 公式地板（修复③）：50px 容器内
/// `min-width: fit-content; width: 10px` 含 2×50px 原子子——floor =
/// min(100, max(50, 50)) = 50，used = max(10, 50) = 50（chromium 50 定谳；
/// R5507 臂的 W_max 地板会过抬到 100）。
#[test]
fn r5508_min_fitcontent_formula_floor_narrow_cb() {
    let html = r#"<html><body style="margin:0">
<div style="width: 50px;">
  <div style="min-width: fit-content; width: 10px;">
    <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
  </div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.get(1).expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 50.0).abs() < 0.5,
        "R5508: min-width:fit-content 窄 CB 地板 min(W_max, max(W_min, CB)) = 50（R5507 W_max 地板过抬 100），实际 {w}"
    );
}

/// R5507 宽 CB 驱动 parity（回归守卫）：body 直下 `min-width: fit-content;
/// width: 10px` 含 2×50px 原子子——floor = min(100, max(50, 800)) = 100 不变。
#[test]
fn r5508_min_fitcontent_wide_cb_parity_r5507() {
    let html = r#"<html><body style="margin:0">
<div style="min-width: fit-content; width: 10px;">
  <span style="display: inline-block; width: 50px;"></span><span style="display: inline-block; width: 50px;"></span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let id = *divs.first().expect("inner div");
    let w = find_box(&result.root, id).expect("inner box");
    assert!(
        (w - 100.0).abs() < 0.5,
        "R5508: 宽 CB min-width:fit-content 地板 100（R5507 行为保持），实际 {w}"
    );
}
