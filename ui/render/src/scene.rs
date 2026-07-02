//! Scene — 扁平化的绘制命令缓冲（spec FR-004 Render·Scene tree 输出）。
//!
//! paint 阶段把 RenderNode 树拍平为带 clip 的图元序列，交给合成/光栅后端。

use crate::render_node::RenderPrimitive;
use serde::{Deserialize, Serialize};
use zero_ui_core::geometry::Rect;
use zero_ui_core::widget::WidgetId;

/// 单条带 clip 与来源 id 的场景命令。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneEntry {
    pub source: WidgetId,
    pub clip: Option<Rect>,
    pub primitive: RenderPrimitive,
}

/// 扁平化场景。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub entries: Vec<SceneEntry>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene::default()
    }

    pub fn push(&mut self, entry: SceneEntry) {
        self.entries.push(entry);
    }

    /// 把另一场景的命令追加进来（overlay/合成用）。
    pub fn extend(&mut self, other: &Scene) {
        self.entries.extend(other.entries.iter().cloned());
    }
}
