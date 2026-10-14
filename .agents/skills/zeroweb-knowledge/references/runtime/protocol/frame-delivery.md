# 绘制快照、共享帧、mailbox 与 FD 交付

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未运行帧交付。 -->

## Module Structure

覆盖 `PaintSnapshotParams` 的资源身份及 compositor 输出的 inline RGBA、POSIX shm、GPU mailbox、SCM_RIGHTS FD。图元转换算法在 graphics，角色发布顺序在 renderer；本文说明跨进程载荷与所有权。

## Gotchas
- `gpu_image` 不必是真 GPU 纹理：POSIX shm mailbox 是头加 RGBA，mmap consume 仍通过 to_vec 复制，不能把 GPU_ZERO_COPY 名称当成端到端零拷贝证明。 (`crates/protocol/src/frame_shm.rs`, `crates/protocol/src/gpu_mailbox.rs`)
- mailbox 先验证 28 字节头、magic 和载荷完整性，再对照 descriptor 验证尺寸、sync_token、最小 fence 及 width×height×4；二进制头解析成功不等于可接受该帧。 (`crates/protocol/src/gpu_mailbox.rs`, `crates/protocol/src/frame_shm.rs`)
- `publish_fd` 接管 RawFd 所有权；SCM_RIGHTS 只为接收者复制句柄，不会关闭发送者本地 FD，OwnedFd 必须覆盖成功和错误返回。 (`crates/protocol/src/fd_socket_linux.rs`, `git:9befc525e`)
- 非零 drm_modifier 当前被拒绝，FD socket 名优先取 fd_socket_name，缺失才用 mailbox_name；把任意 descriptor 当线性可导入纹理会越过真实支持边界。 (`crates/protocol/src/frame_shm.rs`)

## Architecture
- 下载字体 payload 中的 font_id 是 surface/document 局部身份，每帧重复本帧实际引用的下载字体，避免 latest-wins 丢掉先前资源注册。 (`crates/protocol/src/paint_snapshot.rs`, `apps/renderer/src/font_payloads.rs`)
- 字体单个与单帧总字节预算均为 32 MiB，数量最多 128；接收侧还验证字体字节，发送侧预算不替代接收侧信任边界。 (`crates/protocol/src/paint_snapshot.rs`, `apps/renderer/src/font_payloads.rs`, `apps/compositor/src/lib.rs`)
- glyph 的源文本 cluster 使用 UTF-8 字节区间，文本控件 caret boundary 使用 UTF-16 offset；两个偏移体系服务不同消费者，不能直接互换。 (`crates/protocol/src/paint_snapshot.rs`, `apps/renderer/src/paint_export.rs`)
- 普通 shm consume 在读取后删除文件并核对期望长度；它是一次性交付资源，不是可重复读取的永久帧缓存。 (`crates/protocol/src/frame_shm.rs`)

## Patterns
- RGBA 期望长度在 usize 域饱和相乘；FD 获取有两秒连接等待，接收返回 OwnedFd，后续 importer 承担其生命周期。 (`crates/protocol/src/frame_shm.rs`, `crates/protocol/src/fd_socket_linux.rs`)
- 只有最新帧确由 GPU 光栅化、scroll transform 关闭且导出开关与能力均满足时 compositor 才尝试 dma-buf；GPU 预热就绪不证明当前 front buffer 来自 GPU。 (`apps/compositor/src/lib.rs`, `git:5af8cc846`)
