//! 指针事件（spec §8.4.1B 手势先进入 gesture arena；winit 类型不泄漏到本 crate）。
//!
//! 宿主把平台指针事件归一为 [`PointerEvent`] 喂入手势识别器/arena。`timestamp_ms` 供 fling
//! 速度计算（由宿主用动画时钟打戳）。

use zero_ui_core::geometry::Point;

/// 指针阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerPhase {
    Down,
    Move,
    Up,
}

/// 归一化指针事件。`id` 标识指针（鼠标 = 0，触摸点 = 1..N，支持多指 pinch）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    pub id: u32,
    pub phase: PointerPhase,
    pub position: Point,
    /// 事件时间戳（ms，宿主用动画时钟打戳；fling 速度用）。
    pub timestamp_ms: i64,
}

impl PointerEvent {
    pub fn down(id: u32, position: Point, timestamp_ms: i64) -> PointerEvent {
        PointerEvent {
            id,
            phase: PointerPhase::Down,
            position,
            timestamp_ms,
        }
    }
    pub fn move_(id: u32, position: Point, timestamp_ms: i64) -> PointerEvent {
        PointerEvent {
            id,
            phase: PointerPhase::Move,
            position,
            timestamp_ms,
        }
    }
    pub fn up(id: u32, position: Point, timestamp_ms: i64) -> PointerEvent {
        PointerEvent {
            id,
            phase: PointerPhase::Up,
            position,
            timestamp_ms,
        }
    }
}
