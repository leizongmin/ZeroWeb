//! Scene — 扁平化的绘制命令缓冲（spec FR-004 Render·Scene tree 输出）。
//!
//! paint 阶段把 RenderNode 树拍平为带 clip 的图元序列，交给合成/光栅后端。

use crate::render_node::RenderPrimitive;
use serde::{Deserialize, Serialize};
use zero_ui_core::geometry::{Rect, Vec2};
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

    /// 返回一份所有图元沿向量平移后的场景（clip 也同步平移）。
    ///
    /// retained host paint 遍历用：每个 widget 以局部坐标 paint 进自己的 SceneRecorder，
    /// host 按节点绝对 origin 平移后并入全局 Scene。
    pub fn translated(&self, offset: Vec2) -> Scene {
        Scene {
            entries: self
                .entries
                .iter()
                .map(|e| SceneEntry {
                    source: e.source.clone(),
                    clip: e.clip.map(|c| c.translate(offset.x, offset.y)),
                    primitive: e.primitive.clone().translate(offset),
                })
                .collect(),
        }
    }

    /// 按 scale_factor 缩放所有几何 → 逻辑坐标到物理坐标。
    /// host 在逻辑坐标空间布局/paint 后，用此方法一步缩放到物理坐标再喂 bridge。
    pub fn scaled(&self, factor: f32) -> Scene {
        if (factor - 1.0).abs() < f32::EPSILON {
            return self.clone();
        }
        Scene {
            entries: self
                .entries
                .iter()
                .map(|e| SceneEntry {
                    source: e.source.clone(),
                    clip: e.clip.map(|c| c.scale(factor)),
                    primitive: e.primitive.clone().scale(factor),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Rounding;
    use zero_ui_core::theme::Color;

    #[test]
    fn translated_shifts_primitive_rect_and_clip() {
        // 一个局部坐标的 FillRect(0,0,10,10) + clip(0,0,10,10)，
        // 平移 (100, 50) 后 rect/clip 都到 (100,50,110,60)，source 不变。
        let mut scene = Scene::new();
        scene.push(SceneEntry {
            source: WidgetId::new("btn"),
            clip: Some(Rect::from_ltrb(0.0, 0.0, 10.0, 10.0)),
            primitive: RenderPrimitive::FillRect {
                rect: Rect::from_ltrb(0.0, 0.0, 10.0, 10.0),
                color: Color::BLACK,
                rounding: Rounding::ZERO,
            },
        });

        let moved = scene.translated(Vec2::new(100.0, 50.0));
        assert_eq!(moved.entries.len(), 1);
        assert_eq!(moved.entries[0].source, WidgetId::new("btn"));
        assert_eq!(moved.entries[0].clip, Some(Rect::from_ltrb(100.0, 50.0, 110.0, 60.0)));
        match &moved.entries[0].primitive {
            RenderPrimitive::FillRect { rect, .. } => {
                assert_eq!(*rect, Rect::from_ltrb(100.0, 50.0, 110.0, 60.0));
            }
            other => panic!("expected FillRect, got {other:?}"),
        }
        // 原场景不可变。
        match &scene.entries[0].primitive {
            RenderPrimitive::FillRect { rect, .. } => {
                assert_eq!(*rect, Rect::from_ltrb(0.0, 0.0, 10.0, 10.0));
            }
            _ => unreachable!(),
        }
    }
}
