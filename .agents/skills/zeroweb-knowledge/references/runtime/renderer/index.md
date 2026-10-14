# renderer 与浏览器进程协调

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->
<!-- 核验基线：当前 Git 可见源码；本次仅静态核验。 -->

## Module Structure

`zero-renderer` library 提供 desktop/Android 角色入口；desktop main 负责日志、role 校验和进程退出。运行时持有 WebView、外部 RendererJsWorker、fetch 回包表及帧发布状态，browser ProcessBackend 管理宿主快照和资源所有权。

### Key Entry Points

| 问题 | 源码入口 | 阅读方向 |
| --- | --- | --- |
| 导航不提交或旧页面回包 | `apps/renderer/src/runtime.rs`、`apps/browser/src/process_backend.rs` | epoch、pending、抢占和预算 |
| 图片偶发缺失或状态倒序 | `apps/renderer/src/compositor_publish_thread.rs`、`apps/renderer/src/paint_export.rs` | 帧尾合并和写后确认 |
| 同步 JS 等待卡住 | `apps/renderer/src/ipc_indexed_db.rs`、`apps/renderer/src/js_worker.rs` | reader/router 旁路和有界执行 |

## Gotchas
- desktop main 正常返回后直接进程退出，不能把 library role 返回、worker Drop 完成和 OS 进程退出当成同一事实。 (`apps/renderer/src/main.rs`, `apps/renderer/src/lib.rs`, `apps/renderer/src/runtime.rs`)
- 帧 ID 与 navigation_epoch 不同：surface 的 frame id 单调递增，导航换代另外清资源和身份；不能仅按 ID 大小跨文档接收。 (`apps/renderer/src/runtime.rs`, `apps/browser/src/process_backend.rs`)

## Architecture
- WebView external_script 指向 RendererJsWorker 的 executor，避免再创建第二个页面脚本执行器；媒体 registry 与泵时钟也从同一 WebView 注入 worker。 (`apps/renderer/src/runtime.rs`)
- stdout transport 由主循环与专用发布线程共享，帧缓冲的完整写入锁与消息 FIFO 是两项不同约束，缺一项仍可能产生流损坏或可观察状态倒序。 (`apps/renderer/src/compositor_publish_thread.rs`, `apps/renderer/src/runtime.rs`)

## Child Knowledge Nodes
- [导航 epoch、加载抢占、预算与消息循环](navigation-and-loop.md) — 页面间跳转卡住、旧加载覆盖或阶段超时。
- [帧发布、字体资源与渲染进程隔离](publication-and-isolation.md) — 帧/状态乱序、资源丢失、管道污染和沙箱边界。
