# zero-ui-adapter-android

Android 平台适配器（M4 stretch goal）。把 Android 触摸/键盘/IME/back gesture 事件转换为通用 `UiEvent`，实现 `PlatformRuntime`。不依赖 winit。

## 架构位置

```
ui/adapters/android  ←── Android Activity（JNI C ABI 调用）
     │
     ├── zero-ui-core（UiEvent 等事件类型）
     ├── zero-ui-runtime（PlatformRuntime + WidgetHost）
     └── zero-ui-gestures（手势识别器，由 runtime opt-in）
```

## 模块

### `event_map.rs` — Android → UiEvent 转换

| 函数 | 转换 |
|------|------|
| `android_touch_action_to_pointer_phase` | Android `MotionEvent` ACTION_*（0/1/2/3/5/6 masked）→ `PointerPhase`（未知→Cancelled） |
| `map_touch_event` / `map_touch_events` | 单指/多指触摸→`UiEvent::Pointer`（含 `pointer_id`） |
| `map_key_action` / `map_key_event` | 键盘事件→`UiEvent::Key` |
| `map_window_metrics` | 窗口度量→`WindowMetrics`（safe_area / keyboard / density=DEFAULT_DENSITY / orientation / text_scale） |
| `map_soft_keyboard` | 软键盘→`UiEvent::Ime` |
| `map_back_gesture` | 平台 back 手势→`UiEvent::Platform(BackGesture)` |
| `system_theme_from_dark_mode` | 系统暗色模式→`ColorScheme` |

### `runtime.rs` — AndroidRuntime

- `PlatformRuntime` impl：`launch` + `pump_events`
- 事件队列（`dequeue_event` / `enqueue_event` 供 JNI 推入）

### `ffi.rs` — JNI C ABI 导出

| 导出符号 | Android 调用时机 |
|----------|-----------------|
| `android_native_window_resize(w, h, scale)` | `onConfigurationChanged` |
| `android_native_dispatch_touch(id, action, x, y, ts)` | `onTouchEvent` |
| `android_native_dispatch_key(key_code, action)` | `onKeyDown` / `onKeyUp` |
| `android_native_back_pressed()` | `onBackPressed` |
| `android_native_soft_keyboard(height, visible)` | `onWindowInsetsChange` |
| `android_native_is_runtime_ready()` | runtime 初始化状态查询 |

## 依赖

- `zero-ui-core` / `zero-ui-runtime` / `zero-ui-gestures`
- 零 winit 依赖；零浏览器业务 crate 依赖

## 决策

Android 为 M4 stretch goal（不阻塞 DONE）。策略文档：`docs/goal/ui-sdk/decisions/m4-add-android-backend.md`。

## 工具链

```bash
# 交叉编译已验证通过
cargo build --target aarch64-linux-android -p zero-ui-adapter-android
```

Linker 经用户级 `$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` 配置（不提交到项目 `.cargo/config.toml`，遵守 AGENTS.md 路径通用化）。

## 测试

- `cargo test -p zero-ui-adapter-android` — 20 测
- 覆盖：触摸单/多指 / 键盘 / 窗口度量 / soft keyboard / back gesture / system theme / runtime retained 闭环 / FFI 事件产入
