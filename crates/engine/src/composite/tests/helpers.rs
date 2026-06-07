//! 共享测试辅助函数。

use super::super::*;
use zero_layout_engine::types::OverflowClip;

pub fn make_box(node_id: Option<zero_dom::NodeId>, x: f32, y: f32, w: f32, h: f32, is_fixed: bool) -> LayoutBox {
    LayoutBox {
        node_id,
        x,
        y,
        width: w,
        height: h,
        content_x: 0.0,
        content_y: 0.0,
        content_width: w,
        content_height: h,
        border_top: 0.0,
        border_right: 0.0,
        border_bottom: 0.0,
        border_left: 0.0,
        padding_top: 0.0,
        padding_right: 0.0,
        padding_bottom: 0.0,
        padding_left: 0.0,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        children: vec![],
        is_absolute: false,
        is_fixed,
        is_sticky: false,
        overflow_x: OverflowClip::Visible,
        clear: zero_layout_engine::ClearValue::None,
        z_index: 0,
        float: zero_css_parser::values::FloatValue::None,
        overflow_y: OverflowClip::Visible,
    }
}
