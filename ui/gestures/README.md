# zero-ui-gestures

通用 UI SDK 的手势识别系统。提供 `GestureArena` 仲裁机制和 Tap/Pan/Pinch/Fling 四类手势识别器。浏览器无关（winit 类型不泄漏）。

## 模块

| 模块 | 核心类型 | 说明 |
|------|----------|------|
| `event` | `PointerEvent` | 浏览器无关的指针事件（id/phase/position/timestamp_ms）；`pointer_id` 支持多指 |
| `recognition` | `GestureRecognizer` trait, `GestureResult` | 手势识别器接口 |
| `recognizers` | `TapRecognizer`, `PanRecognizer`, `PinchRecognizer` | 四类手势实现。Pan 含 `pan_threshold` → Start/Update；pinch = 双指距离比；Pan 超 fling_threshold→Fling |
| `arena` | `GestureArena` | 多识别器竞争仲裁，首个 Won 胜出其余 cancel，胜后独占路由，指针全抬复位 |

## 手势类型

| 手势 | 触发条件 |
|------|----------|
| `Tap` | 按下→抬起（同指，无长位移） |
| `Pan` | 位移超阈值（`pan_threshold` 像素）|
| `Fling` | Pan 结束时速度超 `fling_threshold` |
| `Pinch` | 双指同时按下→移动 |

## 与 WidgetHost 集成

`WidgetHost` 支持 opt-in `GestureArena`（`set_gesture_arena`），指针事件流经 arena 识别手势（additive 不替代 hit-test），`arena=None` 向后兼容。gestures 通过 `take_gestures()` 取出。

## 依赖

- `zero-ui-core`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-gestures` — 12 测
- 覆盖：event/recognition 100% / arena 90% / recognizers 88%
- 端到端测试：phone_demo 中 Tap + Pinch 经 GestureArena 正确识别
