# 共享 IPC 与进程协议

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次仅静态核验。 -->

## Module Structure

`zero-protocol` 连接 browser、renderer、compositor、Android 角色和 image-decoder。修改消息类型、跨进程资源描述符或断连行为先读本文；业务存储与 Service Worker 语义归 web-data，CDP 命令语义归 products。

## API Surface

| 真实 API | 用途与边界 | 来源 |
| --- | --- | --- |
| `IpcChannel::{send,recv,try_recv,close}` | 通道契约，try_recv 能力取决于实现 | `crates/protocol/src/channel.rs` |
| `IpcMessage`、`IpcMessageKind` | id 配对、角色消息和载荷 | `crates/protocol/src/message.rs` |
| `PipeTransport::new`、`stdio_transport` | 长度前缀管道，stdout 只供协议 | `crates/protocol/src/transport.rs` |
| `RendererHandle`、`ProcessManager` | 子进程、消息读取及终止管理 | `crates/protocol/src/process.rs` |
| `PaintSnapshotParams`、`GpuSharedImageDescriptor` | 页面图元和像素/纹理交付描述 | `crates/protocol/src/paint_snapshot.rs`, `crates/protocol/src/message.rs` |
| `android_socket_transport_from_fd` | Android FD 所有权交接 | `crates/protocol/src/transport.rs` |

## Usage Examples

renderer 的响应复用请求 id，不能另分配一个无关联 id。来源：`apps/renderer/src/runtime.rs`，`send_with_id`。

```rust
let msg = IpcMessage { id, kind };
self.outbound.send(msg).map_err(|e| format!("IPC 发送失败: {e}"))
```

image-decoder 的桌面入口把同一协议接入角色实现。来源：`apps/image-decoder/src/main.rs`。

```rust
let mut transport = zero_protocol::transport::stdio_transport()
    .unwrap_or_else(|error| panic!("image-decoder: stdio transport init: {error}"));
zero_image_decoder::run_role(&mut transport);
```

## Gotchas
- 同一协议需要同步核对 browser 的接收/转发、renderer 的产出和 compositor 的消费；给载荷加字段不会自动穿过图元显式转换点。 (`crates/protocol/src/message.rs`, `apps/browser/src/process_backend.rs`, `apps/renderer/src/paint_export.rs`, `apps/compositor/src/lib.rs`)
- 同进程 SharedMemoryChannel 适合契约模拟，但绕开序列化、管道容量和 OS 句柄交付，不能替代真实 peer 验证。 (`crates/protocol/src/transport.rs`)

## Architecture
- desktop stdio 与 Android socket 复用 IpcChannel 角色入口，平台负责 bootstrap/FD，角色实现负责消息循环；不能把 desktop stdout 假设写死到通用角色 API。 (`crates/protocol/src/transport.rs`, `apps/compositor/src/lib.rs`, `apps/renderer/src/lib.rs`)

## Consumer Analysis
- browser 使用 ProcessManager/RendererHandle 协调 renderer，并把 CompositorFrame 转交 compositor；surface、epoch 和 paint 内 epoch 须一致，不能只按消息 id 认定页面帧新鲜。 (`apps/browser/src/process_backend.rs`)
- renderer 用 PipeTransport 与 reader/router 交换输入、fetch/storage 回包，并产生 ViewPainted/CompositorFrame；同步 JS 等待需要 reader 旁路，不可只加主循环分支。 (`apps/renderer/src/runtime.rs`)
- compositor 用泛型 IpcChannel 运行 surface 消息循环，回执复用请求 id，旧帧回报当前 epoch/frame；回执不等于收到新图元已呈现到窗口。 (`apps/compositor/src/lib.rs`)
- image-decoder 用 stdio_transport 消费解码请求，协议像素载荷与宿主 image cache 身份衔接；日志必须避开 stdout。 (`apps/image-decoder/src/main.rs`, `apps/image-decoder/src/lib.rs`)

## Child Knowledge Nodes
- [消息、序列化、路由、进程与断连](messages-and-lifecycle.md) — 排查管道污染、响应等待或子进程退出。
- [绘制快照、共享帧、mailbox 与 FD 交付](frame-delivery.md) — 修改字体/图像身份、共享帧或 FD 交接。
