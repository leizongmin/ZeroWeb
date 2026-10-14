# 帧发布、字体资源与渲染进程隔离

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次未实跑 GPU 或进程。 -->

## Module Structure

覆盖 `publish_render_with_layout`、`paint_snapshot_from_primitives`、`CompositorPublishThread`、SharedWriter、下载字体和 renderer sandbox。图形算法归 graphics，本文处理写入原子性、状态/帧顺序与资源交付。

## Gotchas
- latest-wins 只能替换发布队列尾部的帧，不能跨过 Title/Url/LoadComplete/DispatchResult/FocusOwnerChanged；这些 regular 消息共用 FIFO 并始终追加。 (`apps/renderer/src/compositor_publish_thread.rs`, `apps/renderer/src/runtime.rs`, `git:e024e0922`)
- 图片 key 只有实际写出成功才回传并加入 sent 集合，构造快照时不标记；否则被合并掉的首个资源帧会留下永久“已发送”假象。 (`apps/renderer/src/paint_export.rs`, `apps/renderer/src/compositor_publish_thread.rs`, `apps/renderer/src/runtime.rs`)
- 每个 SharedWriter clone 有独立帧缓冲，共享底层 writer 锁；flush 把长度头与载荷一起写入，并在错误后清缓冲，不能在长度头与帧体之间释放锁。 (`apps/renderer/src/compositor_publish_thread.rs`)
- renderer sandbox 当前只有启用判断、剥离 LD_PRELOAD/LD_LIBRARY_PATH 和日志，seccomp 是占位；进程隔离不等于已安装完整 OS 沙箱。 (`apps/renderer/src/sandbox.rs`)

## Architecture
- renderer 录制图元并编码 IPC，compositor 承担像素光栅化；GPU 上下文是否被 renderer 创建要核对实际调用与隔离测试，不能从 render-foundation 依赖存在推断。 (`apps/renderer/src/paint_export.rs`, `apps/renderer/src/gpu_isolation_tests.rs`, `apps/compositor/src/lib.rs`)
- glyph 文本 run 在帧内按共享 Arc 身份去重，再传 run_id 和 UTF-8 cluster；font variations 与图元顺序也需完整编码，不能只验证 fills 非空。 (`apps/renderer/src/paint_export.rs`, `crates/protocol/src/paint_snapshot.rs`)
- 下载字体每帧按实际 glyph 引用筛选、按 ID 去重并附 face_index；未使用字体不发送，系统字体依赖双方基础字体表，字体资源和局部 ID 的重映射在消费端。 (`apps/renderer/src/font_payloads.rs`, `apps/compositor/src/lib.rs`)

## Patterns
- 发布线程写失败后 close mailbox，使入队返回 false，由主线程同步发送暴露错误；不能让关闭线程继续接收并静默吞帧。 (`apps/renderer/src/compositor_publish_thread.rs`, `apps/renderer/src/runtime.rs`)
- ZW_IPC_VALIDATE=1 在 SharedWriter flush 前检查完整长度帧序列；ZW_IPC_TRACE 在 transport 首次访问缓存文件并记录两侧头部 48 字节，二者定位阶段不同，不能把诊断器静默当成行为正确。 (`apps/renderer/src/compositor_publish_thread.rs`, `crates/protocol/src/transport.rs`)
- image-decoder 的桌面入口也复用 stdio 协议，日志写 stderr；角色解码处理不应把像素字节或诊断混入 stdout 帧流。 (`apps/image-decoder/src/main.rs`, `apps/image-decoder/src/lib.rs`)
