//! R5073（CSS2 §10.3.5/§10.3.7 shrink-to-fit）：abspos `width:auto` 的内容测量在
//! taffy `Definite` available space 下此前丢弃 float 子贡献（
//! `FloatIntrinsicWidthCalculator::add_float` Definite 臂零贡献，而 taffy 自身
//! `perform_absolute_layout_on_absolute_children` 以 `Definite(CB 宽)` 进入
//! content-based 宽度计算）→ abspos 只有 float 子时 shrink-to-fit 宽塌 0。
//! driving: css/CSS2/positioning absolute-non-replaced-width-023/024（float 子
//! max-width 钳 120 应为 preferred 宽）+ css-flexbox column-flex-child-with-max-width。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

/// kill-switch 用例以 env var 表达（`ZW_ABSPOS_FLOAT_SHRINK`），与同文件其余用例
/// 并行时会互踩——本文件内所有用例先抢这把锁再布局。
static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.width));
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

/// absolute-non-replaced-width-023 形态（body ltr 简化，只验宽）：abspos width:auto
/// + float 子 max-width:4em → shrink-to-fit 宽 = 120（taffy 旧塌 0）。
#[test]
fn r5073_abspos_float_child_shrink_to_fit_width() {
    // mutex 可容忍毒化（前序用例 panic 时不连坐本文件其余用例）。
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0">
<div style="position: absolute; width: auto; font: 30px/4 Ahem, sans-serif;"><div style="float: left; max-width: 4em;">12345678</div></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.first().expect("abspos div");
    let (_x, _y, w) = find_box(&result.root, tid).expect("abspos box");
    assert!(
        (w - 120.0).abs() < 1.0,
        "R5073: abspos float 子 shrink-to-fit 宽应为 120（float 贡献入 Definite 臂），实际 {w}"
    );
}

/// absolute-non-replaced-width-023 全形态（body rtl + margin 8）：宽 120 + 静态位镜像
/// 右缘贴 body content 右缘（800−8=792）→ x = 792−120 = 672。
#[test]
fn r5073_rtl_abspos_float_child_width_and_static_position() {
    // mutex 可容忍毒化（前序用例 panic 时不连坐本文件其余用例）。
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin: 8px; direction: rtl;">
<div style="position: absolute; left: auto; right: auto; width: auto; font: 30px/4 Ahem, sans-serif;"><div style="float: left; max-width: 4em;">12345678</div></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.first().expect("abspos div");
    let (x, _y, w) = find_box(&result.root, tid).expect("abspos box");
    assert!(
        (w - 120.0).abs() < 1.0,
        "R5073: rtl abspos float 子宽应为 120，实际 {w}"
    );
    // find_box 返回相对 body border-box 原点的 x（body abs x=8）：664 + 8 = 绝对 672
    //（右缘贴 body content 右缘 792 − 宽 120）。
    assert!(
        (x - 664.0).abs() < 1.0,
        "R5073: rtl 静态位镜像 x 应为 664（绝对 672 = 792 右缘 − 120），实际 {x}"
    );
}

/// kill-switch 双态：`ZW_ABSPOS_FLOAT_SHRINK=0` 回退 Definite 臂零贡献 → 宽塌 0
/// （修复前基线行为）。
#[test]
fn r5073_kill_switch_reverts_to_zero_width() {
    // mutex 可容忍毒化（前序用例 panic 时不连坐本文件其余用例）。
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0">
<div style="position: absolute; width: auto; font: 30px/4 Ahem, sans-serif;"><div style="float: left; max-width: 4em;">12345678</div></div>
</body></html>"#;
    // SAFETY: 测试单线程内同步设置/读取；无其他测试并发读 ZW_ABSPOS_FLOAT_SHRINK。
    unsafe {
        std::env::set_var("ZW_ABSPOS_FLOAT_SHRINK", "0");
    }
    let (_, result) = layout(html);
    unsafe {
        std::env::remove_var("ZW_ABSPOS_FLOAT_SHRINK");
    }
    let doc = zero_dom::parse_html(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.first().expect("abspos div");
    let (_x, _y, w) = find_box(&result.root, tid).expect("abspos box");
    assert!(
        w < 1.0,
        "R5073: kill-switch 下 float 贡献应回落零贡献（宽塌 0 基线），实际 {w}"
    );
}

/// float 贡献语义（§10.3.5 preferred 宽按 float 求和）：双 float 子 80+80 →
/// shrink-to-fit 宽 160（MaxContent 臂同语义，非单 float 取 max）。
#[test]
fn r5073_two_floats_sum_contribution() {
    // mutex 可容忍毒化（前序用例 panic 时不连坐本文件其余用例）。
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0">
<div style="position: absolute; width: auto;"><div style="float: left; width: 80px; height: 10px;"></div><div style="float: left; width: 80px; height: 10px;"></div></div>
</body></html>"#;
    let (doc, result) = layout(html);
    let divs = doc.get_elements_by_tag_name("div");
    let tid = *divs.first().expect("abspos div");
    let (_x, _y, w) = find_box(&result.root, tid).expect("abspos box");
    assert!((w - 160.0).abs() < 1.0, "R5073: 双 float 贡献应求和 160，实际 {w}");
}
