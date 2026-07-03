# zero-ui-adapter-harmonyos

HarmonyOS 平台适配器。把 OHOS 触摸/键盘/IME/back gesture 事件转换为通用 `UiEvent`，实现 `PlatformRuntime`。不依赖 winit。

## 架构位置

```
ui/adapters/harmonyos  ←── HarmonyOS Ability（Naitve C ABI 调用）
     │
     ├── zero-ui-core（UiEvent 等事件类型）
     ├── zero-ui-runtime（PlatformRuntime + WidgetHost）
     └── zero-ui-gestures（手势识别器，由 runtime opt-in）
```

## 模块

### `event_map.rs` — OHOS → UiEvent 转换

| 函数 | 转换 |
|------|------|
| `ohos_touch_phase_to_pointer_phase` | OHOS TouchAction（0/1/2/3）→ `PointerPhase`（未知→Cancelled） |
| `map_touch_event` / `map_touch_events` | 单指/多指触摸→`UiEvent::Pointer`（含 `pointer_id`） |
| `map_key_action` / `map_key_event` | 键盘事件→`UiEvent::Key` |
| `map_window_metrics` | 窗口度量→`WindowMetrics`（safe_area / keyboard / density=DEFAULT_DENSITY / orientation / text_scale） |
| `map_soft_keyboard` | 软键盘→`UiEvent::Ime` |
| `map_back_gesture` | 平台 back 手势→`UiEvent::Platform(BackGesture)` |
| `system_theme_from_dark_mode` | 系统暗色模式→`ColorScheme` |

### `runtime.rs` — HarmonyOSRuntime

- `PlatformRuntime` impl：`launch` + `pump_events`
- 事件队列（`dequeue_event` / `enqueue_event` 供 FFI 推入）

### `ffi.rs` — C ABI 导出

| 导出符号 | HarmonyOS Naitve 调用时机 |
|----------|--------------------------|
| `harmonyos_window_size_change(width, height, scale, safe_area_top, dpi)` | 窗口尺寸/配置变化 |
| `harmonyos_dispatch_touch(pointer_id, action, x, y, timestamp)` | 触摸事件 |
| `harmonyos_back_pressed()` | 系统返回键 |
| `harmonyos_input_method_change(h, visible)` | 软键盘高度变化 |
| `harmonyos_is_runtime_ready()` | runtime 初始化状态查询 |

## 依赖

- `zero-ui-core` / `zero-ui-runtime` / `zero-ui-gestures`
- 零 winit 依赖；零浏览器业务 crate 依赖

## 决策

M4 选定 HarmonyOS 为 DONE 硬指标（不阻塞条件），Android 为 stretch goal。策略文档：`docs/goal/ui-sdk/decisions/m4-mobile-backend-harmonyos.md`。

## 工具链

交叉编译目标：`aarch64-unknown-linux-ohos`，已验证通过：
```bash
cargo build --target aarch64-unknown-linux-ohos -p zero-ui-adapter-harmonyos
```

## 测试

- `cargo test -p zero-ui-adapter-harmonyos` — 21 测
- 覆盖：触摸单/多指 / 键盘 / 窗口度量 / soft keyboard / back gesture / system theme / runtime retained 闭环 / FFI 事件产入
