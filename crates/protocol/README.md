# ZeroWeb Protocol (`zero-protocol`)

> 多进程 IPC 消息定义、序列化与通道抽象

## 概述

`ZeroWeb Protocol` (`zero-protocol`) 定义了 ZeroWeb 多进程架构中浏览器主进程与渲染进程之间的通信协议。它包含 IPC 消息类型（导航、网络请求、存储操作、输入事件、进程管理）、基于 bincode 的二进制序列化/反序列化，以及传输无关的通道 trait。该 crate 是进程间通信的基础层，不依赖具体的传输机制。

## 主要功能

- **IPC 消息类型** — 涵盖导航命令（Navigate / GoBack / GoForward / Reload）、页面事件（TitleChanged / LoadComplete）、网络请求/响应、存储操作、鼠标/键盘/滚动输入事件、心跳与崩溃通知
- **二进制序列化** — 基于 `bincode` 的高效序列化与反序列化，支持消息 ID 匹配请求与响应
- **通道抽象** — `IpcChannel` trait 定义统一的 `send` / `recv` / `try_recv` / `close` 接口，传输层（管道、socket、共享内存）由宿主实现
- **进程角色** — `ProcessRole` 区分 Browser / Renderer / Network（网络当前由 Browser 承载）
- **Chromium 式子进程** — `child_process_args()` 生成 `--type=renderer` 等启动参数；独立地址空间 + 管道 IPC，非 fork/CoW 共享页状态
- **错误处理** — 统一的 `ProtocolError` 类型覆盖序列化、通道、进程错误

## 使用示例

```rust
use zero_protocol::{
    IpcMessage, IpcMessageKind, NavigateParams,
    serialize, deserialize, IpcChannel, ProcessRole,
};

// 构造一条导航消息
let msg = IpcMessage {
    id: 1,
    kind: IpcMessageKind::Navigate(NavigateParams {
        url: "https://example.com".into(),
        referrer: None,
        navigation_epoch: 0,
    }),
};

// 序列化为二进制
let bytes = serialize(&msg).expect("序列化失败");

// 反序列化还原
let decoded: IpcMessage = deserialize(&bytes).expect("反序列化失败");
assert_eq!(msg.id, decoded.id);
```
