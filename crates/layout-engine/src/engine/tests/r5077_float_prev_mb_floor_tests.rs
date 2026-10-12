//! R5077（CSS 2.1 §9.5.1 + §8.3.1）：float 的 margin-box 顶边不得高于前一流内兄弟的
//! **底边距边缘** = flow_bottom + last_flow_mb（float margin 不折叠，§8.3.1）。
//! 旧 Phase 2 钳制 floor 只用 flow_bottom（前驱 border 底）——正 clearance 后
//!（R1316 flow_bottom 已权威）缺 last_flow_mb，float 落在前驱 margin 底之上
//! 10px，后续 clear 链整体偏移。
//! driving: css/CSS2/floats-clear clear-on-replaced-element（canvas2 clear mt:10 mb:10
//! 后，canvas3 float mt:10 border 顶 40 应 50）。
//! kill-switch: `ZW_FLOAT_PREV_MB_FLOOR=0` 回退旧行为（floor = flow_bottom）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

/// kill-switch 用例以 env var 表达（`ZW_FLOAT_PREV_MB_FLOOR`），本文件内所有
/// 用例先抢锁防并行互踩；毒化容忍（前序 panic 不连坐）。
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

/// 复刻 clear-on-replaced-element 结构：90px 宽 border 5px 容器内
/// ① float div h:20 ② 块级 clear:both 画布 mt:10 h:10 mb:10 ③ float 画布 mt:10 h:10。
/// ③ 的 margin-box 顶边应在 ② 的底边距边缘（border 底 30 + mb 10 = 40，相对 wrapper
/// 内容系），border 顶 = 40 + 10 = 50；旧实现钳在 ② 的 border 底（30）→ border 顶 40。
#[test]
fn r5077_float_floor_respects_prev_sibling_margin_bottom() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0"><div style="width:90px;border-top:5px solid;border-bottom:5px solid"><div style="float:left;height:20px"></div><canvas style="display:block;clear:both;height:10px;margin:10px 0"></canvas><canvas style="display:block;float:left;height:10px;margin:10px 0"></canvas></div></body></html>"#;
    let (doc, result) = layout(html);
    let canvases = doc.get_elements_by_tag_name("canvas");
    assert_eq!(canvases.len(), 2, "两个 canvas 子元素");
    // ② clear 画布：clearance 后 border 顶 20-30（相对 wrapper 内容系）。
    let (_, clear_y, _) = find_box(&result.root, canvases[0]).expect("clear canvas box");
    assert!(
        (clear_y - 20.0).abs() < 0.5,
        "clear 画布应被 clearance 推到 float 底 20，实际 {}",
        clear_y
    );
    // ③ float 画布：margin-box 顶 = ② 底边距边缘 40，border 顶 = 50。
    let (_, float_y, _) = find_box(&result.root, canvases[1]).expect("float canvas box");
    assert!(
        (float_y - 50.0).abs() < 0.5,
        "float margin-box 顶应取前驱底边距边缘（40）+ mt 10 = border 顶 50，实际 {}",
        float_y
    );
}

/// kill-switch 双态：`ZW_FLOAT_PREV_MB_FLOOR=0` 回退旧行为（floor = flow_bottom，
/// float border 顶 40 = 旧行为锚），保证回退路径可用。
#[test]
fn r5077_kill_switch_restores_legacy_floor() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // edition 2024：set_var/remove_var 为 unsafe（仅单线程测试进程 + ENV_LOCK 串行）。
    unsafe {
        std::env::set_var("ZW_FLOAT_PREV_MB_FLOOR", "0");
    }
    let html = r#"<html><body style="margin:0"><div style="width:90px;border-top:5px solid;border-bottom:5px solid"><div style="float:left;height:20px"></div><canvas style="display:block;clear:both;height:10px;margin:10px 0"></canvas><canvas style="display:block;float:left;height:10px;margin:10px 0"></canvas></div></body></html>"#;
    let (doc, result) = layout(html);
    let canvases = doc.get_elements_by_tag_name("canvas");
    let (_, float_y, _) = find_box(&result.root, canvases[1]).expect("float canvas box");
    assert!(
        (float_y - 40.0).abs() < 0.5,
        "kill-switch 下应回到旧行为 border 顶 40，实际 {}",
        float_y
    );
    unsafe {
        std::env::remove_var("ZW_FLOAT_PREV_MB_FLOOR");
    }
}
