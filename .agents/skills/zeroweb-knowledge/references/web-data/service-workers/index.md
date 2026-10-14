# Service Worker 跨层生命周期

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖注册校验、版本槽、安装激活、controller、更新、fetch 拦截、clients/message 与注册持久化。Dedicated Worker 与通用脚本线程归 scripting，IPC 帧格式归 runtime；Service Worker 跨层语义集中维护在本域。

## Gotchas
- browser owner 管普通共享与每标签 private 注册，renderer host 才执行脚本；修改“注册 owner”不等于把 JS 引擎引入 browser 主进程。 (`apps/browser/src/service_worker_owner.rs`, `apps/renderer/src/service_worker_host.rs`)
- unregister 结束未来匹配却可能保留已有 controller，fetch 与 controller 的状态要求又不同；排查旧 worker 活着应先查版本槽和受控 client，而非只查注册列表。 (`crates/page-runtime/src/service_worker_manager.rs`)

## Architecture
- ServiceWorkerRuntimeHost trait 分离本地与 IPC runtime，manager 对两者使用相同 lifecycle/client/cache 回调；替换宿主实现必须保留异步完成和独立线程路由语义。 (`crates/page-runtime/src/service_worker_manager.rs`, `apps/renderer/src/service_worker_host.rs`, `apps/renderer/src/ipc_service_worker.rs`)

## Child Knowledge Nodes
- [注册、安装、激活、更新与 controller](registration-and-lifecycle.md) — 安全校验、版本替换、waitUntil、注销与恢复。
- [fetch 拦截、客户端、消息与宿主路由](fetch-and-clients.md) — controller 选择、respondWith、缓存 wire、clients 与死锁边界。
