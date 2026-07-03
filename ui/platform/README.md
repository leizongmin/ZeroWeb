# zero-ui-platform

通用 UI SDK 的平台服务抽象。将系统平台能力（剪贴板、文件选择、拖拽、通知、back 手势、触觉反馈）定义为可 mock 的 trait，宿主注入真实实现。浏览器无关。

## 服务 trait

| Trait | 方法 | 内存 Mock |
|-------|------|-----------|
| `ClipboardService` | `get()` / `set(text)` / `has()` | `InMemoryClipboard` |
| `FilePickerService` | `pick(options)` / `save(path, data)` | `InMemoryFilePicker`（队列 + save 记录） |
| `DragDropService` | `begin_drag(item)` / `staged(mime)` / `receive()` | `InMemoryDragDrop`（记录 + 暂存） |
| `NotificationService` | `show(Notification)` / `cancel(id)` | `InMemoryNotifications`（通知记录） |
| `BackNavigationService` | `push_handler()` / `pop_handler()` / `on_platform_back()` | `InMemoryBackNavigation`（LIFO 栈仲裁） |
| `HapticFeedbackService` | `tap(HapticKind)` | `InMemoryHaptics` |

## PlatformServices

聚合结构体，持所有服务实例。`new()` 默认所有服务 = InMemory mock；宿主通过 builder 方法注入真实实现：
- `with_clipboard(impl ClipboardService)`
- `with_file_picker(impl FilePickerService)`
- `with_drag_drop(impl DragDropService)`
- `with_notifications(impl NotificationService)`
- `with_back_navigation(impl BackNavigationService)`
- `with_haptics(impl HapticFeedbackService)`

### BackNavigationService（Android OnBackPressedDispatcher 等价）

LIFO 栈仲裁：
- 弹层/菜单打开时 `push_handler()` → 插入栈顶
- 关闭时 `pop_handler()` → 移除
- 平台 back 键→`on_platform_back()`：消耗栈顶 handler → `Handled`；无 handler → `DefaultBack` → 宿主 `Navigator.pop()` / 退出

### HapticFeedbackService

触觉反馈类型：`Light` / `Medium` / `Heavy` / `Selection`

## 依赖

- `zero-ui-core`
- 零浏览器业务 crate 依赖
- 6 个 trait 均为基础设施层抽象（无平台特定依赖）

## 测试

- `cargo test -p zero-ui-platform` — 11 测
- 覆盖：clipboard / file_picker / drag_drop / notifications / **BackNavigationService LIFO 仲裁** / **HapticFeedbackService 记录**
- Coverage 99.47%
