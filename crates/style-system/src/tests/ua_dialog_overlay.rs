//! s377（2026-11-10）：UA `dialog:modal { overlay: auto }` 注入的 compute 面钉测
//!（CSS Position 4 §overlay——top layer 成员资格由 UA 规则授予，模态 dialog 处于
//! top layer 期间 computed overlay 为 auto，关闭/非模态回落 none；::backdrop 不注入
//! ——`pseudo.is_none()` 门，backdrop 的 overlay 恒 none）。runner 面
//! dialog-overlay 双子测（初始 none / modal auto / close 回 none）同构三态，驱动案
//! 即 WPT html/semantics/interactive-elements/the-dialog-element/dialog-overlay.html。
//!
//! 须走全文档通道（[`StyleSystem::compute_styles`]）——与 ua_dialog_visibility 同形。
use crate::StyleSystem;
use zero_dom::{Document, NodeId};

fn dom_with_dialog() -> (Document, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let dialog = doc.create_element("dialog");
    doc.append_child(body, dialog).unwrap();
    (doc, dialog)
}

#[test]
fn test_ua_modal_dialog_overlay_top_layer_membership() {
    let (mut doc, dialog) = dom_with_dialog();
    let sheets: Vec<zero_css_parser::Stylesheet> = Vec::new();

    // 非模态（show()）——overlay 初始 none（UA `:modal` 注入不触发）。
    doc.set_modal_dialogs(Vec::new());
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &sheets);
    assert_eq!(
        styles.get(&dialog).map(|s| s.overlay.clone()),
        Some(crate::OverlayValue::None)
    );

    // 模态（showModal()）——UA `dialog:modal{overlay:auto}` 授予 top layer 成员
    //（UA 优先级 0,0,0，见 lib.rs 注入块）。
    doc.set_modal_dialogs(vec![dialog]);
    let mut sys2 = StyleSystem::new();
    let styles2 = sys2.compute_styles(&doc, &sheets);
    assert_eq!(
        styles2.get(&dialog).map(|s| s.overlay.clone()),
        Some(crate::OverlayValue::Auto)
    );
}
