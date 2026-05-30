# ZeroWeb Host Runtime (`zero-host-runtime`)

> 跨平台窗口管理与事件循环运行时，基于 winit 提供统一的宿主环境抽象

## 概述

`ZeroWeb Host Runtime` (`zero-host-runtime`) 是 ZeroWeb 的平台宿主层，负责创建和管理跨平台窗口、驱动事件循环、以及处理输入事件。它将 winit 的平台特定细节封装为简洁的 API，向上层渲染引擎和应用提供统一的窗口与事件接口。

## 主要功能

- **窗口配置**：通过 `WindowConfig` 构建器设置标题、尺寸、是否可调整大小
- **基本事件循环**：`run()` 方法提供窗口事件回调，适合纯 CPU 渲染场景
- **GPU 事件循环**：`run_with_window()` 方法额外传递 `Arc<Window>` 引用，用于创建 wgpu Surface
- **统一事件类型**：`AppEvent` 枚举涵盖重绘、缩放、关闭、焦点、键盘输入等常见窗口事件
- **跨平台支持**：基于 winit，支持 macOS、Linux、Windows 等主流桌面平台

## 使用示例

```rust
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_host_runtime::event::AppEvent;

fn main() -> zero_host_runtime::HostResult<()> {
    let config = WindowConfig::new("ZeroWeb")
        .with_size(1280, 720);

    let runtime = HostRuntime::new(config);
    runtime.run(|event| match event {
        AppEvent::RedrawRequested => {
            // 执行渲染逻辑
        }
        AppEvent::Resized { width, height } => {
            println!("窗口大小变更: {}x{}", width, height);
        }
        AppEvent::CloseRequested => {
            println!("窗口关闭");
        }
        _ => {}
    })
}
```
