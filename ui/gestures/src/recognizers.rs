//! 具体手势识别器（spec §8.8 tap/drag/pinch/fling 测）。

use zero_ui_core::geometry::{Point, Vec2};

use crate::event::{PointerEvent, PointerPhase};
use crate::recognition::{Gesture, GestureRecognizer, GestureResult, PanPhase};

/// 两点距离。
fn distance(a: Point, b: Point) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

/// 位移向量 a→b。
fn delta(a: Point, b: Point) -> Vec2 {
    Vec2::new(b.x - a.x, b.y - a.y)
}

// ── Tap ─────────────────────────────────────────────────────────────────────

/// Tap 识别器：press 后在移动阈值内 release → Tap。
///
/// 单指针（跟踪首个 Down 的 id）。大位移 → Yield（让 Pan 赢）。
#[derive(Debug, Clone)]
pub struct TapRecognizer {
    move_tolerance: f32,
    pointer: Option<u32>,
    pressed_at: Option<Point>,
}

impl Default for TapRecognizer {
    fn default() -> TapRecognizer {
        TapRecognizer {
            move_tolerance: 8.0,
            pointer: None,
            pressed_at: None,
        }
    }
}

impl TapRecognizer {
    pub fn new(move_tolerance: f32) -> TapRecognizer {
        TapRecognizer {
            move_tolerance,
            pointer: None,
            pressed_at: None,
        }
    }
}

impl GestureRecognizer for TapRecognizer {
    fn handle_pointer(&mut self, event: &PointerEvent) -> GestureResult {
        match event.phase {
            PointerPhase::Down => {
                // 跟踪首个指针；忽略其它指针。
                if self.pointer.is_none() {
                    self.pointer = Some(event.id);
                    self.pressed_at = Some(event.position);
                }
                GestureResult::Pending
            }
            PointerPhase::Move => {
                // 大位移（本指针且越过容差）→ 不是 tap，让出（Pan 会接手）。
                let cancel = matches!(
                    self.pressed_at,
                    Some(start) if self.pointer == Some(event.id)
                        && distance(start, event.position) > self.move_tolerance
                );
                if cancel {
                    self.pressed_at = None;
                    return GestureResult::Yield;
                }
                GestureResult::Pending
            }
            PointerPhase::Up => {
                let won = match (self.pressed_at, self.pointer == Some(event.id)) {
                    (Some(start), true) => {
                        if distance(start, event.position) <= self.move_tolerance {
                            Some(Gesture::Tap(start))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                self.reset();
                match won {
                    Some(g) => GestureResult::Won(g),
                    None => GestureResult::Yield,
                }
            }
        }
    }

    fn cancel(&mut self) {
        self.reset();
    }
}

impl TapRecognizer {
    fn reset(&mut self) {
        self.pointer = None;
        self.pressed_at = None;
    }
}

// ── Pan / Drag / Fling ───────────────────────────────────────────────────────

/// Pan 识别器：按下后位移越过 `pan_threshold` → 开始拖拽；松手时若速度超 `fling_threshold`
/// 额外发 [`Gesture::Fling`]（紧跟 Pan End 之后由 arena 决定如何呈现）。
#[derive(Debug, Clone)]
pub struct PanRecognizer {
    pan_threshold: f32,
    fling_threshold: f32, // px/ms
    pointer: Option<u32>,
    start: Option<Point>,
    last: Option<(Point, i64)>, // (position, timestamp_ms)
    panning: bool,
}

impl PanRecognizer {
    /// 默认：pan 阈值 10px，fling 阈值 2.0 px/ms。
    pub fn new() -> PanRecognizer {
        PanRecognizer {
            pan_threshold: 10.0,
            fling_threshold: 2.0,
            pointer: None,
            start: None,
            last: None,
            panning: false,
        }
    }

    pub fn with_thresholds(pan_threshold: f32, fling_threshold: f32) -> PanRecognizer {
        PanRecognizer {
            pan_threshold,
            fling_threshold,
            ..PanRecognizer::new()
        }
    }

    fn velocity_to(&self, pos: Point, t: i64) -> Vec2 {
        // 用上一采样点算瞬时速度（px/ms）。
        match self.last {
            Some((lp, lt)) if t > lt => {
                let d = delta(lp, pos);
                let dt = (t - lt) as f32;
                Vec2::new(d.x / dt, d.y / dt)
            }
            _ => Vec2::ZERO,
        }
    }

    fn speed(v: Vec2) -> f32 {
        (v.x * v.x + v.y * v.y).sqrt()
    }
}

impl Default for PanRecognizer {
    fn default() -> PanRecognizer {
        PanRecognizer::new()
    }
}

impl GestureRecognizer for PanRecognizer {
    fn handle_pointer(&mut self, event: &PointerEvent) -> GestureResult {
        match event.phase {
            PointerPhase::Down => {
                if self.pointer.is_none() {
                    self.pointer = Some(event.id);
                    self.start = Some(event.position);
                    self.last = Some((event.position, event.timestamp_ms));
                    self.panning = false;
                }
                GestureResult::Pending
            }
            PointerPhase::Move => {
                if self.pointer != Some(event.id) || self.start.is_none() {
                    return GestureResult::Pending;
                }
                let start = self.start.unwrap();
                let vel = self.velocity_to(event.position, event.timestamp_ms);
                if !self.panning {
                    if distance(start, event.position) >= self.pan_threshold {
                        self.panning = true;
                        self.last = Some((event.position, event.timestamp_ms));
                        return GestureResult::Won(Gesture::Pan {
                            phase: PanPhase::Start,
                            position: event.position,
                            delta: delta(start, event.position),
                            velocity: vel,
                        });
                    }
                    return GestureResult::Pending;
                }
                // 已在拖拽：发 Update。
                self.last = Some((event.position, event.timestamp_ms));
                GestureResult::Won(Gesture::Pan {
                    phase: PanPhase::Update,
                    position: event.position,
                    delta: delta(start, event.position),
                    velocity: vel,
                })
            }
            PointerPhase::Up => {
                if self.pointer != Some(event.id) {
                    return GestureResult::Pending;
                }
                let result = if self.panning {
                    let vel = self.velocity_to(event.position, event.timestamp_ms);
                    if Self::speed(vel) >= self.fling_threshold {
                        // 先发 Fling（arena 会把两条都返回；这里返回 Fling，Pan End 在 reset 前
                        // 由调用方按需补——为简化：Up 时若超 fling 阈值直接发 Fling，否则发 Pan End）。
                        GestureResult::Won(Gesture::Fling { velocity: vel })
                    } else {
                        GestureResult::Won(Gesture::Pan {
                            phase: PanPhase::End,
                            position: event.position,
                            delta: delta(self.start.unwrap_or(event.position), event.position),
                            velocity: vel,
                        })
                    }
                } else {
                    // 未越过 pan 阈值 → 不是拖拽，让出（Tap 可能赢）。
                    GestureResult::Yield
                };
                self.reset();
                result
            }
        }
    }

    fn cancel(&mut self) {
        self.reset();
    }
}

impl PanRecognizer {
    fn reset(&mut self) {
        self.pointer = None;
        self.start = None;
        self.last = None;
        self.panning = false;
    }
}

// ── Pinch ────────────────────────────────────────────────────────────────────

/// Pinch 识别器：跟踪两个活动指针，按双指距离比发 [`Gesture::Pinch`]（缩放系数 = 当前距离/起始距离）。
#[derive(Debug, Default, Clone)]
pub struct PinchRecognizer {
    pointers: [(Option<u32>, Option<Point>); 2],
    initial_distance: Option<f32>,
}

impl PinchRecognizer {
    pub fn new() -> PinchRecognizer {
        PinchRecognizer::default()
    }

    fn slot_of(&mut self, id: u32) -> Option<&mut (Option<u32>, Option<Point>)> {
        // 先找已记录该 id 的槽，否则找第一个空槽。
        let existing = self.pointers.iter().position(|(pid, _)| *pid == Some(id));
        match existing {
            Some(i) => Some(&mut self.pointers[i]),
            None => self.pointers.iter_mut().find(|(pid, _)| pid.is_none()),
        }
    }

    fn both(&self) -> Option<(Point, Point)> {
        match (self.pointers[0].1, self.pointers[1].1) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}

impl GestureRecognizer for PinchRecognizer {
    fn handle_pointer(&mut self, event: &PointerEvent) -> GestureResult {
        match event.phase {
            PointerPhase::Down => {
                if let Some(slot) = self.slot_of(event.id) {
                    slot.0 = Some(event.id);
                    slot.1 = Some(event.position);
                }
                if let Some((a, b)) = self.both() {
                    self.initial_distance = Some(distance(a, b));
                }
                GestureResult::Pending
            }
            PointerPhase::Move => {
                if let Some(slot) = self.pointers.iter_mut().find(|(pid, _)| *pid == Some(event.id)) {
                    slot.1 = Some(event.position);
                }
                if let (Some((a, b)), Some(init)) = (self.both(), self.initial_distance)
                    && init > 0.0
                {
                    let scale = distance(a, b) / init;
                    let pivot = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
                    return GestureResult::Won(Gesture::Pinch { scale, pivot });
                }
                GestureResult::Pending
            }
            PointerPhase::Up => {
                if let Some(slot) = self.pointers.iter_mut().find(|(pid, _)| *pid == Some(event.id)) {
                    slot.0 = None;
                    slot.1 = None;
                }
                if self.both().is_none() {
                    self.initial_distance = None;
                }
                GestureResult::Pending
            }
        }
    }

    fn cancel(&mut self) {
        *self = PinchRecognizer::new();
    }
}
