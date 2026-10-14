# 网络、持久存储与安全边界

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

本域覆盖网络与导航资源、HTTP 缓存/Cookie/WebSocket、Web Storage/IndexedDB/Cache API/OPFS、Origin/PSL/CSP/CORS/HSTS/权限及 Service Worker 跨层生命周期。入口位于 net/storage/security/page-runtime，产品消费者包括 WebView、browser 代理和 renderer IPC。

页面通用 JS API 及 Dedicated Worker 不在本域，协议格式和进程生命周期由 runtime 领域维护。这里的条目基于当前可见源码和静态测试断言，不代表执行了浏览器或安全/离线兼容性验收。

### User Task Navigation

| 用户任务 | 源码入口 | 知识导航 | 既有验证范围 |
|---|---|---|---|
| 重定向后资源错误、抓取饥饿 | apps/browser/src/fetch_proxy.rs、apps/renderer/src/ipc_fetch.rs | network/index.md | net client 与 scheduler 的常驻测试，未实跑 |
| 存储重开丢数据或事务 blocked | crates/storage/src/storage_manager.rs、apps/browser/src/process_backend.rs | storage/index.md | [存储地图](../../../zeroweb-verify/features/storage.md)：底座恢复与页面边界 |
| CSP/混合内容策略绕过 | crates/security/src/context.rs、crates/webview/src/webview.rs | security/index.md | WebView csp_gate/mixed_content_gate，未实跑 |
| SW 更新、旧 controller 或离线请求 | crates/page-runtime/src/service_worker_manager.rs、apps/browser/src/service_worker_owner.rs | service-workers/index.md | WebView SW runtime/fetch 与 protocol 断言，未实跑 |

## Gotchas
- HTTP 缓存、页面 CacheStorage 与 SW registration-local CacheStorage 是三种不同数据域，清理或落盘某一域不代表另外两域同步变化。 (`crates/net/src/http_cache.rs`, `crates/page-runtime/src/cache_storage_host.rs`, `crates/page-runtime/src/service_worker_manager.rs`)
- 无痕不是一个跨层相同粒度的键：fetch proxy 用独立 private loader，IDB 用 private StorageManager，SW 使用 Private(TabId)；跨域共享和清理必须按真实 owner 核对。 (`apps/browser/src/fetch_proxy.rs`, `apps/browser/src/process_backend.rs`, `apps/browser/src/service_worker_owner.rs`)

## Architecture
- browser 存储请求采用已提交导航 origin，SW 注册依据可信 document 校验，renderer 只发送操作；源验证必须留在宿主授权边界，不能因页面已序列化 URL 而省略。 (`apps/browser/src/process_backend.rs`, `crates/page-runtime/src/service_worker_registration.rs`)

## Child Knowledge Nodes
- [资源网络与导航](network/index.md) — 请求、最终 URL、缓存、Cookie、连接与 WebSocket。
- [存储后端与宿主所有权](storage/index.md) — 配额、事务、对象身份、OPFS 与持久恢复。
- [来源、策略与隔离](security/index.md) — Origin/PSL、资源门、权限与隔离的真实接线边界。
- [Service Worker 跨层生命周期](service-workers/index.md) — 注册、版本、controller、fetch/client 与 runtime 路由。
