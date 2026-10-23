//! R5244：CSS direction 定段落基方向（UAX #9 HL1）锚点。
//!
//! 修前 `bidi_reorder_with_direction` 对 LTR 上下文传 `para_level=None`（内容
//! auto-detect，P2/P3 首强字符侦测）——R 开头 run（"א × ÷"）被判成 RTL 段
//! **整段反转**，违 HL1：CSS `direction:ltr` 块的中性字符应随嵌入方向解析为 L
//! （N1/N2，sor/eor=L），视觉序 = 逻辑序。bidi-breaking-001 test/ref 双页反排
//! 实证（ref 页 bdo ltr 与 test 页 R-first run 同错）。修后 LTR 基强制
//! `Some(Level::ltr())`；HL4 `unicode-bidi:plaintext` 例外保持内容 auto-detect。
//!
//! 断言介质 = stored IFC fragments（pure-Ahem 容器满足存储门
//! inline_finalization `is_pure_ahem`），按 x 排序拼接去空格 = 视觉字符序：
//! 修前签名 "÷×א"（反序），修后 "א×÷"（逻辑序）。
//!
//! kill-switch：`ZW_BIDI_LTR_PARA_BASE=0` 回退修前行为（单测进程内不设 env）。

use super::*;

/// 取首个存了 IFC 的盒（pure-Ahem 容器）。
fn find_stored_ifc_box(root: &LayoutBox) -> Option<&LayoutBox> {
    if root.inline_layout.is_some() {
        return Some(root);
    }
    for child in &root.children {
        if let Some(found) = find_stored_ifc_box(child) {
            return Some(found);
        }
    }
    None
}

/// stored IFC 全片段按 x 升序拼接、去空格 = 视觉字符序（单段单片段或分段多片段
/// 两种切分形态下均成立）。
fn visual_char_sequence(root: &LayoutBox) -> String {
    let ifc = root.inline_layout.as_ref().expect("stored IFC");
    let mut frags: Vec<_> = ifc.iter().flat_map(|line| line.fragments.iter()).collect();
    frags.sort_by(|a, b| a.x.partial_cmp(&b.x).expect("fragment x 非 NaN"));
    frags
        .into_iter()
        .map(|f| f.text.as_str())
        .collect::<Vec<_>>()
        .concat()
        .replace(' ', "")
}

/// R5244 锚①：默认（ltr）块内 R-first run "א × ÷"——中性字符随嵌入方向解析为 L，
/// fragments x 序 = 逻辑序。修前 P3 侦测判 RTL 段整段反转（签名 "÷×א"）。
#[test]
fn r5244_css_ltr_para_base_keeps_r_first_run_fragments_logical() {
    let html = "<html><body style=\"margin:0\"><div style=\"font:100px/1 Ahem; width:800px\">א × ÷</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_stored_ifc_box(&result.root).expect("pure-Ahem 容器须存 IFC");
    assert_eq!(
        visual_char_sequence(div),
        "א×÷",
        "LTR 块 R-first run 视觉序应为逻辑序（UAX #9 HL1 + N1/N2）"
    );
}

/// R5244 锚②（HL4 例外回归守）：`unicode-bidi:plaintext` 不受 CSS direction 约束，
/// R-first 内容保持 P2/P3 内容 auto-detect → 整段反转。修前即此行为，修复不得改变。
#[test]
fn r5244_plaintext_r_first_keeps_hl4_auto_detect() {
    let html = "<html><body style=\"margin:0\"><div style=\"font:100px/1 Ahem; width:800px; unicode-bidi:plaintext\">א × ÷</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_stored_ifc_box(&result.root).expect("pure-Ahem 容器须存 IFC");
    assert_eq!(
        visual_char_sequence(div),
        "÷×א",
        "plaintext 须保持内容 auto-detect（CSS Writing Modes §2.2，HL4 例外）"
    );
}

/// R5244 锚③（RTL 臂回归守）：`direction:rtl` 段落基 = RTL（HL1 RTL 侧），R-first
/// 内容整段反转，修复不动该臂。
#[test]
fn r5244_direction_rtl_reversal_unchanged() {
    let html = "<html><body style=\"margin:0\"><div style=\"font:100px/1 Ahem; width:800px; direction:rtl\">א × ÷</div></body></html>";
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let div = find_stored_ifc_box(&result.root).expect("pure-Ahem 容器须存 IFC");
    assert_eq!(
        visual_char_sequence(div),
        "÷×א",
        "direction:rtl 基下 R-first 内容应整段反转"
    );
}
