# fetch 拦截、客户端、消息与宿主路由

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取 respondWith、clients.claim、消息通道、renderer 死锁、离线响应与跨层缓存问题。browser 管权限/profile/持久化，renderer host 独立线程执行 typed runtime，manager 负责事件与 client 控制关系。

## Gotchas
- fetch 的 request origin 与传入 origin 不同时 manager 直接 PassThrough；已有 client 优先选其 controller，导航带 resulting_client_id 时按目标 URL scope 选择，不能一律取请求 URL 的最长 scope。 (`crates/page-runtime/src/service_worker_manager.rs`)
- 存在 active registration 不自动控制既有页面，clients.claim 才能接管符合最长 scope 的 client；重复 observe 不会把原本 uncontrolled 的 client 自动改成 controlled。 (`crates/page-runtime/src/service_worker_manager.rs`)
- browser begin_page_fetch 先校验 authority 与请求 http(s)，请求 body 无法转 UTF-8 时走网络 PassThrough；manager 的缓存响应转换也要求 UTF-8，因此这条路径不是任意二进制 body 的透明代理。 (`apps/browser/src/service_worker_owner.rs`, `crates/page-runtime/src/service_worker_manager.rs`)
- SW runtime 命令由 IPC reader 直接投到独立 host 线程，不经过 renderer 主循环；同步 automation 占住主循环时，register 正等待求值，改成主循环路由会互等。 (`apps/renderer/src/service_worker_host.rs`, `apps/renderer/src/ipc_indexed_db.rs`)
- SW IPC 请求上限 64、超时 20 秒，id 从 1<<62 起分配；响应路由按 id 解除等待，断连 fail_all，不能把已超时请求永久留在 pending。 (`apps/renderer/src/ipc_service_worker.rs`)
- 每注册 CacheStorage 与页面 StorageManager 的 per-origin CacheStorage 是不同存储实例；当前 SW 离线缓存随注册快照保存，不能宣称两个 contexts 的 caches 已天然共享同一个数据库。 (`crates/page-runtime/src/service_worker_manager.rs`, `crates/storage/src/service_worker.rs`, `crates/page-runtime/src/cache_storage_host.rs`)

## Architecture
- Cache API 的 Open/Match/MatchAll/Keys/Put 和 query options 需同时穿过 runtime 事件、renderer wire、browser 转换与 manager；至少三次共同变更证实这是协议映射耦合，不能只新增一侧枚举。 (`apps/renderer/src/service_worker_host.rs`, `apps/browser/src/service_worker_owner.rs`, `crates/page-runtime/src/service_worker_manager.rs`, `git:47043ede3977cea96dab89a944b94b5c5d53f4b7`, `git:2e3563e09cd87f60b8f57a50af6a57411e26070f`, `git:b20f07361b1cc07aedaa01389111f128d3ad22da`)
- clients 的 frame_type 只接受 top-level、auxiliary、nested；消息保留 source 与 transferred ports，window client 与 worker peer 不能混成仅有 URL 的消息发送者。 (`crates/page-runtime/src/service_worker_manager.rs`, `crates/script-sandbox/src/service_worker.rs`)
- manager 记录 (registration_id,event_id) 的 pending fetch，默认最多 1024；未知或迟到 body cancel 静默忽略，当前 cancel 还保留 headless 序列化后取消的近似语义。 (`crates/page-runtime/src/service_worker_manager.rs`)
- module import 的跨源响应在 browser owner 验证 Access-Control-Allow-Origin，禁止 HTTPS 降级 HTTP、带 userinfo 的 final URL、无 JavaScript MIME 或坏 UTF-8；classic import 与 module import 不能使用完全相同的来源策略。 (`apps/browser/src/service_worker_owner.rs`)

## Patterns
- WebView service_worker_fetch 用受控 iframe 和测试 fetcher检查导航请求投影，protocol 测试只验证 wire roundtrip；它们分别约束语义与消息结构，均不能单独证明真实 GUI 离线导航已通过。 (`crates/webview/src/tests/service_worker_fetch.rs`, `crates/protocol/src/tests/service_worker_protocol.rs`)
