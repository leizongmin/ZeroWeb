//! R5078（CSS2 §10.3.2 + §10.6.2 replaced sizing）：iframe 无固有尺寸——CSS auto 侧走
//! default object size 300×150 + HTML width/height 属性（attr % 相对 CB 高可解则解析，
//! 不可解或无 attr → default 150）。旧实现 iframe 落 block stretch（784×0）。
//! driving: css/CSS2/normal-flow inline-block-replaced-height-004/005/007。
//! kill-switch: `ZW_IFRAME_DEFAULT_SIZE=0` 回退旧行为。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

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

fn find_height(root: &LayoutBox, node_id: NodeId) -> Option<f32> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b.height);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 004 形态：裸 iframe（CSS width/height auto，无 attr）→ default 300×150。
#[test]
fn r5078_iframe_default_object_size() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0"><div style="position:relative"><iframe style="display:inline-block;border:solid red;height:auto;width:auto"></iframe></div></body></html>"#;
    let (doc, result) = layout(html);
    let iframes = doc.get_elements_by_tag_name("iframe");
    assert_eq!(iframes.len(), 1);
    let (x, _, w) = find_box(&result.root, iframes[0]).expect("iframe box");
    let h = find_height(&result.root, iframes[0]).expect("iframe height");
    // LayoutBox w/h 为 border-box（default 300×150 + border 3px×2 = 306×156）
    assert!(
        (w - 306.0).abs() < 0.5,
        "iframe border-box 宽应 300+6=306，实际 {w}（x={x}）"
    );
    assert!((h - 156.0).abs() < 0.5, "iframe border-box 高应 150+6=156，实际 {h}");
}

/// 005 形态：iframe height="50%" 相对 definite 父高（2in=192）→ 96。
#[test]
fn r5078_iframe_attr_percent_resolves_against_definite_cb() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0"><div style="position:relative;height:2in;width:300px"><iframe style="border:solid red;display:inline-block;width:auto" height="50%"></iframe></div></body></html>"#;
    let (doc, result) = layout(html);
    let iframes = doc.get_elements_by_tag_name("iframe");
    let h = find_height(&result.root, iframes[0]).expect("iframe height");
    // content 96 + border 6 = 102 border-box
    assert!(
        (h - 102.0).abs() < 0.5,
        "height=50% 相对 definite 父高 192 应 96（border-box 102），实际 {h}"
    );
}

/// 007 形态：iframe height="50%" 父高 auto 不可解 → computed auto → default 150。
#[test]
fn r5078_iframe_attr_percent_unresolvable_falls_back_to_default() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0"><div style="position:relative;width:300px"><iframe style="border:solid red;display:inline-block;width:auto" height="50%"></iframe></div></body></html>"#;
    let (doc, result) = layout(html);
    let iframes = doc.get_elements_by_tag_name("iframe");
    let h = find_height(&result.root, iframes[0]).expect("iframe height");
    // content 150 + border 6 = 156 border-box
    assert!(
        (h - 156.0).abs() < 0.5,
        "height=50% 不可解应落 default 150（border-box 156），实际 {h}"
    );
}

/// flex 行 item：default size 只入主轴（flex base 300），交叉轴留 auto 保 align
/// stretch——definite 150 会破坏 stretch（对比案 flexbox-iframe-intrinsic-size-001：
/// 双 iframe flex:1 容器高 400，应 400×400 等分）。
#[test]
fn r5078_iframe_flex_row_default_size_main_axis_only() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let html = r#"<html><body style="margin:0"><div style="display:flex;height:400px"><iframe style="flex:1 1 auto;border:0;background:blue"></iframe><iframe style="flex:1 1 auto;border:0;background:green"></iframe></div></body></html>"#;
    let (doc, result) = layout(html);
    let iframes = doc.get_elements_by_tag_name("iframe");
    assert_eq!(iframes.len(), 2);
    for id in iframes {
        let h = find_height(&result.root, id).expect("iframe height");
        assert!(
            (h - 400.0).abs() < 0.5,
            "行 flex iframe 交叉轴应 stretch 到容器高 400（definite 150 会破坏），实际 {h}"
        );
    }
}

/// kill-switch 双态：`ZW_IFRAME_DEFAULT_SIZE=0` 回退旧 block-stretch 行为。
#[test]
fn r5078_kill_switch_restores_legacy_stretch() {
    let _env = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    unsafe {
        std::env::set_var("ZW_IFRAME_DEFAULT_SIZE", "0");
    }
    let html = r#"<html><body style="margin:0"><div style="position:relative;width:784px"><iframe style="display:inline-block;border:solid red;height:auto;width:auto"></iframe></div></body></html>"#;
    let (doc, result) = layout(html);
    let iframes = doc.get_elements_by_tag_name("iframe");
    let (_, _, w) = find_box(&result.root, iframes[0]).expect("iframe box");
    assert!(
        (w - 784.0).abs() < 0.5,
        "kill-switch 下应回旧 block-stretch 宽 784，实际 {w}"
    );
    unsafe {
        std::env::remove_var("ZW_IFRAME_DEFAULT_SIZE");
    }
}
