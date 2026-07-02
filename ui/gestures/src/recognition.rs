//! 手势识别抽象（spec IF-010 `GestureRecognizer` / §8.8 arena tap/drag/pinch/fling 测）。
//!
//! 每个识别器把 [`PointerEvent`](crate::event::PointerEvent) 流映射为 [`GestureResult`]；
//! 多个识别器在 [`crate::arena::GestureArena`] 中竞争，首个 [`GestureResult::Won`] 胜出，
//! 其余被取消（arena 决策）。

use zero_ui_core::geometry::{Point, Vec2};

use crate::event::PointerEvent;

/// Pan（拖拽/平移）阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanPhase {
    /// 越过阈值，开始拖拽。
    Start,
    /// 拖拽中。
    Update,
    /// 松手。
    End,
}

/// 识别出的手势。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// 单击（按下→抬起在阈值内）。
    Tap(Point),
    /// 拖拽/平移：阶段、当前位置、相对起点的累计位移、当前速度（px/ms）。
    Pan {
        phase: PanPhase,
        position: Point,
        delta: Vec2,
        velocity: Vec2,
    },
    /// 双指捏合：相对起始的缩放系数、双指中点。
    Pinch { scale: f32, pivot: Point },
    /// 松手时速度超阈值的 fling（惯性滚动 / sheet fling dismiss）。
    Fling { velocity: Vec2 },
}

/// 单个识别器对一次指针事件的结果（IF-010 `GestureResult`）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureResult {
    /// 仍未决，继续喂事件。
    Pending,
    /// 识别成功，声明胜出（arena 据此取消其它识别器）。
    Won(Gesture),
    /// 主动放弃（事件序列与本手势不匹配，如 Tap 遇到大位移）。
    Yield,
}

/// 手势识别器 trait（spec IF-010 `GestureRecognizer`）。
///
/// `handle_pointer` 喂入一个指针事件，返回当前结果；`cancel` 由 arena 在被别的识别器抢先时
/// 调用，识别器应复位内部状态。
pub trait GestureRecognizer {
    fn handle_pointer(&mut self, event: &PointerEvent) -> GestureResult;
    fn cancel(&mut self) {}
}
