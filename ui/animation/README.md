# zero-ui-animation

通用 UI SDK 的动画基础设施。提供时钟抽象、曲线插值、弹簧物理和减少动效支持。浏览器无关。

## 模块

| 模块 | 核心类型 | 说明 |
|------|----------|------|
| `clock` | `AnimationClock` trait, `FakeClock`, `Clock` | 动画时钟抽象（`now()` + `request_frame()`）；`FakeClock` 用于测试 |
| `curve` | `Curve` | 缓动曲线（ease/ease-in/ease-out/linear 等） |
| `tween` | `Tween` | 补间动画（起止值 + 时长 + 曲线） |
| `spring` | `Spring` | **弹簧物理动画**（半隐式 Euler + 子步长稳定）。预设：smooth / snappy / bouncy。`launch` / `retarget` / `step` / `is_settled` |
| `motion` | `MotionPreference` | 减动效偏好（reduced-motion：`sample_tween`→终态 / `settle_spring`→终态） |

## 依赖

- `zero-ui-core` + `serde`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-animation` — 17 测
- 覆盖：clock 100% / spring 96.77%（含 fling 回弹/预设/retarget/子步长稳定性） / motion 96.77–100%

## 使用场景

- fling 惯性滚动（gestures + spring）
- UI 元素过渡动画（tween + curve）
- 按钮/列表等微交互动效
