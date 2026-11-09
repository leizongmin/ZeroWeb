//! s373（2026-11-10）：UA `dialog:modal { visibility: visible }` 注入的 compute 面
//! 钉测（HTML 渲染规范 #flow-content-3:is-modal——UA 样式表模态 dialog 强制可见，
//! 父链 `visibility: hidden` 不再隐匿；作者声明仍可覆盖）。runner 面
//! modal-dialog-in-visibility-hidden 三断言（非模态继承 hidden / 模态 UA visible /
//! 作者覆盖 hidden）同构三态，驱动案即 WPT
//! html/semantics/interactive-elements/the-dialog-element/modal-dialog-in-visibility-hidden.html。
//!
//! 须走全文档通道（[`StyleSystem::compute_styles`]，自动递归父链）——单元素
//! `compute_element_style` 传 `parent_style=None` 无继承上下文，visibility 继承面
//! 不可观测（勘根期实测假阳性）。
use crate::StyleSystem;
use zero_css_parser::ast::{
    ComplexSelector, CompoundSelector, Declaration, Rule, Selector, StyleRule, Stylesheet, TypeSelector,
};
use zero_css_parser::values::VisibilityValue;
use zero_dom::{Document, NodeId};

fn hidden_div_sheet() -> Vec<Stylesheet> {
    vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "visibility".to_string(),
                value: "hidden".to_string(),
                important: false,
            }],
        })],
    }]
}

fn dom_with_hidden_parent_dialog() -> (Document, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let dialog = doc.create_element("dialog");
    doc.append_child(outer, dialog).unwrap();
    (doc, dialog)
}

#[test]
fn test_ua_modal_dialog_visibility_overrides_parent_inheritance() {
    let (mut doc, dialog) = dom_with_hidden_parent_dialog();
    let sheets = hidden_div_sheet();

    // 非模态（show()）——visibility 按常规继承取 hidden（该案第一断言面；
    // UA `:modal` 注入不触发）。
    doc.set_modal_dialogs(Vec::new());
    let mut sys = StyleSystem::new();
    let styles = sys.compute_styles(&doc, &sheets);
    assert_eq!(
        styles.get(&dialog).map(|s| s.visibility.clone()),
        Some(VisibilityValue::Hidden)
    );

    // 模态（showModal()）——UA `dialog:modal{visibility:visible}` 反转父链继承
    //（该案第二断言面；UA 优先级 0,0,0，见 lib.rs 注入块）。
    doc.set_modal_dialogs(vec![dialog]);
    let mut sys2 = StyleSystem::new();
    let styles2 = sys2.compute_styles(&doc, &sheets);
    assert_eq!(
        styles2.get(&dialog).map(|s| s.visibility.clone()),
        Some(VisibilityValue::Visible)
    );
}
