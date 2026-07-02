//! Render·Scene tree 节点与图元（spec §8.4.2 `RenderNode` / FR-004）。
//!
//! Render tree 负责 layout 后的几何、绘制、命中、裁剪、合成、a11y；按失效标记增量更新。

use serde::{Deserialize, Serialize};
use zero_ui_core::geometry::{Rect, Rounding};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::WidgetId;

/// 单个绘制图元（spec FR 13 种 RenderPrimitives 的 M1 子集；M2 接 render-foundation 全集）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderPrimitive {
    /// 填充矩形（含圆角）。
    FillRect {
        rect: Rect,
        color: Color,
        rounding: Rounding,
    },
    /// 描边矩形。
    StrokeRect {
        rect: Rect,
        color: Color,
        stroke_width: f32,
        rounding: Rounding,
    },
    /// 文本（M1 承载字符串 + 位置；M2 由 foundation/text 产出 TextBlob/glyph 引用）。
    Text {
        text: String,
        position: zero_ui_core::geometry::Point,
        size_px: f32,
        color: Color,
    },
    /// 预 shape 文本（DC-11 phase 2：由 foundation/text 的 `TextShaper` + `TextMeasurer`
    /// 产出的 `TextBlob`，后端直接光栅 glyph，不再 reshape）。
    TextBlob {
        blob: zero_text_foundation::TextBlob,
        position: zero_ui_core::geometry::Point,
        color: Color,
    },
}

/// Render tree 节点（spec §8.4.2）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderNode {
    pub id: WidgetId,
    pub rect: Rect,
    pub clip: Option<Rect>,
    pub primitives: Vec<RenderPrimitive>,
    pub children: Vec<RenderNode>,
}

impl RenderNode {
    pub fn new(id: WidgetId, rect: Rect) -> RenderNode {
        RenderNode {
            id,
            rect,
            clip: None,
            primitives: Vec::new(),
            children: Vec::new(),
        }
    }
}
