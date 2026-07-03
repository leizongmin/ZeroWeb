# zero-ui-adapter-winit

winit 平台适配器。把 winit 桌面窗口/事件循环桥接到通用 UI runtime。winit 类型**不泄漏**到 widgets 层（产出一律浏览器无关 `UiEvent`）。

## 架构位置

```
ui/adapters/winit  ←── apps/browser / 外部桌面应用
     │
     ├── zero-ui-core（UiEvent 等事件类型）
     ├── zero-ui-runtime（WidgetHost + UiApp）
     └── winit（窗口/事件循环/IME/surface）
```

## 模块

### `event_map.rs` — winit → UiEvent 转换（15 个 pub fn）

| 函数 | 转换 |
|------|------|
| `to_logical_point/size` | 物理坐标→逻辑坐标（scale ≤ 0 防御除零） |
| `map_window_metrics` | winit resize → `WindowMetrics`（safe_area/keyboard 初始 0） |
| `map_mouse_input` / `map_cursor_moved` | 鼠标按下/释放 → `UiEvent::Pointer` |
| `map_key_action` / `map_logical_key` / `key_text` / `map_key_event` | 键盘 → `UiEvent::Key`（Named→Debug 形态，与 host-runtime 逐字一致；16 个浏览器关键键契约锁） |
| `map_touch` / `map_touch_phase` | 触摸 → `UiEvent::Pointer`（`pointer_id`＝平台 touch id，支持多指） |
| `map_mouse_wheel` | 滚轮 → `UiEvent::Wheel`（LineDelta×20 + PixelDelta；**y 取反** = winit y-up vs UI delta.y-down） |
| `map_ime` | IME → `UiEvent::Ime`（Preedit 选区→单光标） |

### `driver.rs` — WinitDriver

将 winit 事件循环的可测试核心从阻塞 `EventLoop::run` 解耦为无窗口可驱动模块：

- `begin()` / `pump_event(UiEvent)` / `pump_frame()` / `set_metrics()` / `set_tokens()` / `host_mut()`
- 端到端 retained 闭环：事件→dispatch→reducer→Handled→重建→Scene
- 6 headless 测（inline CounterApp）覆盖完整驱动路径

### `runtime.rs` — WinitRuntime

- `launch(app, metrics, register) → WinitDriver`：setup 核心（工厂注册 + 首帧）
- 真实 `EventLoop::run` 阻塞壳（需 GUI）在 `launch` 外包一层 EventLoop + Window + surface

## 依赖

- `zero-ui-core` / `zero-ui-runtime` / `winit`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-adapter-winit` — 32 测
- 覆盖：event_map 全部 16 键浏览器契约 / adapter↔runtime 端到端契约 / WinitDriver retained 闭环 + 失效 + resize + theme + gesture arena + per-event 集成 + launch setup
