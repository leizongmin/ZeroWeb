//! 表单容器中 block 与原生 inline 控件混排后的高度修正。

use std::collections::HashMap;

use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

use crate::types::LayoutBox;

/// Taffy 把 inline-block 控件作为 block 子参与 form 初始测高；IFC 随后把这些控件
/// 重排到同一行，但旧的过大 form 高度仍会推开后续内容。仅在普通 auto-height form
/// 含 block 子和原生 inline 控件、且无复杂定位/float 时按最终子盒底边收紧。
fn shrink_mixed_control_form(box_node: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    let Some(form_id) = box_node.node_id else { return };
    let is_form = doc
        .get(form_id)
        .is_some_and(|node| matches!(&node.kind, NodeKind::Element(element) if element.local_name() == "form"));
    let is_auto_block = styles.get(&form_id).is_some_and(|style| {
        matches!(style.height, LengthValue::Auto)
            && matches!(
                style.display,
                DisplayValue::Block | DisplayValue::Flow | DisplayValue::FlowRoot
            )
    });
    if !is_form || !is_auto_block || box_node.children.is_empty() {
        return;
    }
    layout_direct_fieldsets(box_node, doc, styles);

    let mut has_block = false;
    let mut has_control = false;
    for child in &box_node.children {
        if child.is_absolute
            || child.is_fixed
            || !matches!(child.float, FloatValue::None)
            || child.margin_top < 0.0
            || child.margin_bottom < 0.0
        {
            return;
        }
        let is_control = child.node_id.and_then(|id| doc.get(id)).is_some_and(|node| {
            matches!(&node.kind, NodeKind::Element(element) if matches!(
                element.local_name(),
                "button" | "input" | "select" | "textarea"
            ))
        });
        if is_control {
            has_control = true;
        } else if child.is_block_level {
            has_block = true;
        } else {
            return;
        }
    }
    if !has_block || !has_control {
        return;
    }

    let content_height = box_node
        .children
        .iter()
        .map(|child| child.y + child.height + child.margin_bottom.max(0.0))
        .fold(0.0f32, f32::max);
    if content_height + 0.5 >= box_node.content_height {
        return;
    }
    box_node.content_height = content_height;
    box_node.height =
        content_height + box_node.padding_top + box_node.padding_bottom + box_node.border_top + box_node.border_bottom;
}

fn layout_direct_fieldsets(form: &mut LayoutBox, doc: &Document, styles: &HashMap<NodeId, ComputedStyle>) {
    let mut fieldset_indices = Vec::new();
    for (index, child) in form.children.iter().enumerate() {
        let tag = child
            .node_id
            .and_then(|id| doc.get(id))
            .and_then(|node| match &node.kind {
                NodeKind::Element(element) => Some(element.local_name()),
                _ => None,
            });
        match tag {
            Some("fieldset") => fieldset_indices.push(index),
            Some("button" | "input" | "select" | "textarea") => {}
            _ => return,
        }
    }
    if fieldset_indices.is_empty() {
        return;
    }
    if fieldset_indices
        .iter()
        .any(|&index| !direct_child_has_tag(doc, form.children[index].node_id, "legend"))
    {
        return;
    }

    for &index in &fieldset_indices {
        let fieldset = &mut form.children[index];
        let mut legend_index = None;
        let mut flow_indices = Vec::new();
        for (child_index, child) in fieldset.children.iter().enumerate() {
            let tag = child
                .node_id
                .and_then(|id| doc.get(id))
                .and_then(|node| match &node.kind {
                    NodeKind::Element(element) => Some(element.local_name()),
                    _ => None,
                });
            match tag {
                Some("legend") if legend_index.is_none() => legend_index = Some(child_index),
                Some(_) if child.is_block_level => flow_indices.push(child_index),
                _ => {}
            }
        }
        let Some(legend_index) = legend_index else { continue };
        if let Some(legend_id) = fieldset.children[legend_index].node_id {
            let legend = &mut fieldset.children[legend_index];
            let content_width = crate::intrinsic_sizing::text_content_max_width(legend_id, doc, styles);
            legend.content_width = content_width;
            legend.width =
                legend.border_left + legend.padding_left + content_width + legend.padding_right + legend.border_right;
        }
        let legend_height = fieldset.children[legend_index].height;
        fieldset.children[legend_index].y = -(fieldset.border_top + fieldset.padding_top);

        let mut previous: Option<usize> = None;
        for child_index in flow_indices {
            for nested in &mut fieldset.children[child_index].children {
                let is_checkbox = nested.node_id.is_some_and(|id| {
                    doc.get(id).is_some_and(
                        |node| matches!(&node.kind, NodeKind::Element(element)
                            if element.local_name() == "input"
                                && element.get_attribute("type").is_some_and(|value| value.eq_ignore_ascii_case("checkbox"))),
                    )
                });
                if is_checkbox {
                    nested.y += nested.margin_bottom.max(0.0);
                }
            }
            if direct_child_has_tag(doc, fieldset.children[child_index].node_id, "textarea") {
                let descent = fieldset.children[child_index]
                    .node_id
                    .and_then(|id| styles.get(&id))
                    .and_then(|style| match style.font_size {
                        LengthValue::Px(value) => Some((value as f32 * 0.32).round()),
                        _ => None,
                    })
                    .unwrap_or(5.0);
                fieldset.children[child_index].height += descent;
                fieldset.children[child_index].content_height += descent;
            }
            let y = if let Some(previous_index) = previous {
                let previous = &fieldset.children[previous_index];
                previous.y
                    + previous.height
                    + previous
                        .margin_bottom
                        .max(fieldset.children[child_index].margin_top)
                        .max(0.0)
            } else {
                legend_height + fieldset.children[child_index].margin_top.max(0.0) - fieldset.border_top
            };
            fieldset.children[child_index].y = y;
            previous = Some(child_index);
        }

        if let Some(last_index) = previous {
            let last = &fieldset.children[last_index];
            let content_bottom = last.y + last.height + last.margin_bottom.max(0.0);
            fieldset.content_height = content_bottom;
            fieldset.height = fieldset.border_top
                + fieldset.padding_top
                + content_bottom
                + fieldset.padding_bottom
                + fieldset.border_bottom;
        }
    }

    let mut previous_fieldset: Option<usize> = None;
    for &index in &fieldset_indices {
        let y = if let Some(previous_index) = previous_fieldset {
            let previous = &form.children[previous_index];
            previous.y + previous.height + previous.margin_bottom.max(form.children[index].margin_top).max(0.0)
        } else {
            0.0
        };
        form.children[index].y = y;
        previous_fieldset = Some(index);
    }
    let last = *fieldset_indices.last().unwrap();
    let controls_y = form.children[last].y + form.children[last].height + form.children[last].margin_bottom.max(0.0);
    for child in form.children.iter_mut().skip(last + 1) {
        child.y = controls_y;
    }
}

fn direct_child_has_tag(doc: &Document, parent: Option<NodeId>, tag: &str) -> bool {
    let Some(parent) = parent else { return false };
    doc.child_nodes(parent).into_iter().any(|child| {
        doc.get(child)
            .is_some_and(|node| matches!(&node.kind, NodeKind::Element(element) if element.local_name() == tag))
    })
}

pub(crate) fn shrink_mixed_control_forms(
    box_node: &mut LayoutBox,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
) -> f32 {
    let old_height = box_node.height;
    let mut index = 0usize;
    while index < box_node.children.len() {
        let delta = shrink_mixed_control_forms(&mut box_node.children[index], doc, styles);
        if delta.abs() > 0.01 {
            let child = &box_node.children[index];
            let old_child_bottom = child.y + child.height - delta;
            let preserved_gap = box_node.children.get(index + 1).map_or(0.0, |sibling| {
                let existing_gap = sibling.y - old_child_bottom;
                let desired_gap = child.margin_bottom.max(sibling.margin_top).max(0.0);
                (desired_gap - existing_gap).max(0.0)
            });
            let flow_delta = delta + preserved_gap;
            for sibling in box_node.children.iter_mut().skip(index + 1) {
                if !sibling.is_absolute && !sibling.is_fixed && matches!(sibling.float, FloatValue::None) {
                    sibling.y += flow_delta;
                }
            }
            if box_node.declared_height_auto {
                box_node.content_height = (box_node.content_height + flow_delta).max(0.0);
                box_node.height = (box_node.height + flow_delta).max(0.0);
            }
        }
        index += 1;
    }
    shrink_mixed_control_form(box_node, doc, styles);
    box_node.height - old_height
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shrinks_auto_form_after_inline_controls_are_repositioned() {
        let doc = zero_dom::parse_html("<form><fieldset></fieldset><button>Go</button></form>");
        let form = doc.get_elements_by_tag_name("form")[0];
        let fieldset = doc.get_elements_by_tag_name("fieldset")[0];
        let button = doc.get_elements_by_tag_name("button")[0];
        let mut styles = HashMap::new();
        let mut form_style = ComputedStyle::default();
        form_style.display = DisplayValue::Block;
        styles.insert(form, form_style);
        let mut root = LayoutBox::default();
        root.node_id = Some(form);
        root.content_height = 300.0;
        root.height = 300.0;
        let mut block = LayoutBox::default();
        block.node_id = Some(fieldset);
        block.is_block_level = true;
        block.height = 100.0;
        let mut control = LayoutBox::default();
        control.node_id = Some(button);
        control.y = 120.0;
        control.width = 60.0;
        control.height = 40.0;
        root.children = vec![block, control];

        shrink_mixed_control_form(&mut root, &doc, &styles);
        assert_eq!(root.content_height, 160.0);
        assert_eq!(root.height, 160.0);
    }

    #[test]
    fn preserves_following_sibling_margin_after_form_shrink() {
        let doc = zero_dom::parse_html(
            "<main><form><fieldset></fieldset><button>Go</button></form><output>x</output></main>",
        );
        let form = doc.get_elements_by_tag_name("form")[0];
        let fieldset = doc.get_elements_by_tag_name("fieldset")[0];
        let button = doc.get_elements_by_tag_name("button")[0];
        let output = doc.get_elements_by_tag_name("output")[0];
        let mut styles = HashMap::new();
        let mut form_style = ComputedStyle::default();
        form_style.display = DisplayValue::Block;
        styles.insert(form, form_style);

        let mut form_box = LayoutBox {
            node_id: Some(form),
            content_height: 300.0,
            height: 300.0,
            ..LayoutBox::default()
        };
        form_box.children = vec![
            LayoutBox {
                node_id: Some(fieldset),
                is_block_level: true,
                height: 100.0,
                ..LayoutBox::default()
            },
            LayoutBox {
                node_id: Some(button),
                y: 120.0,
                width: 60.0,
                height: 40.0,
                ..LayoutBox::default()
            },
        ];
        let output_box = LayoutBox {
            node_id: Some(output),
            y: 300.0,
            height: 24.0,
            margin_top: 16.0,
            ..LayoutBox::default()
        };
        let mut root = LayoutBox {
            content_height: 340.0,
            height: 340.0,
            declared_height_auto: true,
            children: vec![form_box, output_box],
            ..LayoutBox::default()
        };

        shrink_mixed_control_forms(&mut root, &doc, &styles);

        assert_eq!(root.children[0].height, 160.0);
        assert_eq!(root.children[1].y, 176.0);
        assert_eq!(root.height, 216.0);
    }
}
