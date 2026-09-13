//! 纯文本 shrink-to-fit 与行布局共用已加载文档字体的测量路径。

use std::collections::HashMap;
use taffy::prelude::{AvailableSpace, Size};
use zero_dom::{Document, NodeKind};
use zero_style_system::ComputedStyle;

use crate::{
    LayoutBox,
    inline_finalization::{InlineFontContext, measure_text_content},
};

pub(crate) fn downloaded_text_border_width(
    node: &LayoutBox,
    doc: &Document,
    styles: &HashMap<zero_dom::NodeId, ComputedStyle>,
    fonts: InlineFontContext<'_>,
) -> Option<f32> {
    let id = node.node_id?;
    let style = styles.get(&id)?;
    fonts
        .metric_provider?
        .downloaded_line_metrics(&style.font_family, 1.0)?;
    if fonts.advance_source.is_none()
        || doc.child_nodes(id).iter().any(|child| {
            doc.get(*child)
                .is_some_and(|node| matches!(node.kind, NodeKind::Element(_)))
        })
    {
        return None;
    }
    // https://drafts.csswg.org/css-sizing-3/#max-content-inline-size
    // 本切片只处理纯文本叶盒；复杂嵌套仍由既有 intrinsic 算法负责。
    let measured = measure_text_content(
        doc,
        styles,
        id,
        Size::NONE,
        Size {
            width: AvailableSpace::MaxContent,
            height: AvailableSpace::MaxContent,
        },
        &HashMap::new(),
        fonts,
    );
    Some(measured.width + node.padding_left + node.padding_right + node.border_left + node.border_right)
}
