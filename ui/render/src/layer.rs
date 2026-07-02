//! 合成层（spec §8.4.1 `layer.rs`）。
//!
//! 独立合成层用于 transform/opacity 动画与裁剪隔离（spec FR-012 局部失效的 composite 层）。

use serde::{Deserialize, Serialize};
use zero_ui_core::geometry::{Rect, Vec2};
use zero_ui_core::widget::WidgetId;

/// 合成层描述。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: WidgetId,
    pub bounds: Rect,
    /// 0..=1 不透明度。
    pub opacity: f32,
    /// 平移偏移（用于平移动画，避免重排）。
    pub offset: Vec2,
}

impl Layer {
    pub fn new(id: WidgetId, bounds: Rect) -> Layer {
        Layer {
            id,
            bounds,
            opacity: 1.0,
            offset: Vec2::ZERO,
        }
    }
}
