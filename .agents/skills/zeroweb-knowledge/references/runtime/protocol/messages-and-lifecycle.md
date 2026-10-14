# 消息、序列化、路由、进程与断连

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未运行进程或测试。 -->

## Module Structure

定位管道污染、响应路由、renderer 断连及子进程回收时读取。`IpcMessage` 的 `id` 配对请求响应，`IpcMessageKind` 描述载荷；导航代际和帧序号是载荷契约，不由传输层自动解释。入口为 `PipeTransport`、`RendererHandle`、`ProcessManager`。

## Gotchas
- `PipeTransport::try_recv` 明确返回不支持；生产非阻塞轮询由后台 reader 加消息接收器实现，不能用这个方法替代 renderer 的 reader/router。 (`crates/protocol/src/transport.rs`, `crates/protocol/src/process.rs`, `apps/renderer/src/runtime.rs`)
- 管道帧先验证 64 MiB 上限再分配；旧 16 MiB 限制会拒绝高 DPI UI/present 位图，不能把小型输入消息的预算应用到整帧。 (`crates/protocol/src/transport.rs`, `git:cf514e7f2`)
- 子进程 stdout 承载二进制协议，renderer 与 image-decoder 的 tracing 写 stderr；把日志写入 stdout 会产生完整长度帧中的异质字节，接收端会打印最多 256 字节的反序列化诊断。 (`apps/renderer/src/main.rs`, `apps/image-decoder/src/main.rs`, `crates/protocol/src/transport.rs`)
- renderer 的 stderr 默认保存在有界 crash tail，只有启动时 `ZW_RENDERER_STDERR_ECHO=1` 才转发；browser 没显示日志不等于子进程没产生日志。 (`crates/protocol/src/process.rs`)

## Architecture
- 管道使用四字节小端长度加 bincode 载荷并逐消息 flush；`IpcMessageKind` 的枚举布局参与序列化，配套 peer 必须使用一致协议，字段上的 serde default 不能单独证明任意旧二进制可互通。 (`crates/protocol/src/transport.rs`, `crates/protocol/src/serialize.rs`, `crates/protocol/src/message.rs`)
- `SharedMemoryChannel` 的 recv 是队列 pop，空队列即错误，并非生产管道那种阻塞等待；同进程模拟不能证明双向管道背压或进程断连行为。 (`crates/protocol/src/transport.rs`)
- Android socket transport 接管 Kotlin detachFd 交出的 FD，并复制写侧句柄；drop 关闭两份句柄，调用方不得再关闭原 FD。 (`crates/protocol/src/transport.rs`)

## Patterns
- 仅 MouseEvent、KeyboardEvent、ScrollEvent 使用 48 字节预分配的单遍序列化；带 String/Vec 的消息保留精确容量路径，避免小容量倍增使导航等变长消息回退。 (`crates/protocol/src/serialize.rs`)
- `RendererHandle::shutdown` 先 kill/wait 子进程，再释放发送端、清 pending automation 并 join reader/stderr；只关闭 stdin 不能保证长脚本线程及时退出。 (`crates/protocol/src/process.rs`, `apps/renderer/src/main.rs`)
- Windows Job Object 设置 KILL_ON_JOB_CLOSE，支持创建时预绑定和 spawn 后 assign 兜底；预绑定失败不替代显式 shutdown，后绑定仍有短竞态窗口。 (`crates/protocol/src/job.rs`, `crates/protocol/src/process.rs`)
- 断连判断集中覆盖 EOF、broken pipe、Windows 109/232 和连接重置等文本；反序列化错误不应无条件当成正常 EOF。 (`crates/protocol/src/transport.rs`, `apps/renderer/src/runtime.rs`)
