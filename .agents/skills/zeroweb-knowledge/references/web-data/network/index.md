# 资源网络与导航

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 HTTP 请求、导航最终 URL、资源调度、缓存、Cookie、WebSocket 和 file 资源。改动网络时先区分裸传输、缓存感知资源入口、browser 代理和 renderer 回包，避免只修某个宿主。

## Architecture
- 同一 URL 不足以证明可以合并请求；资源入口负责分区与完整身份，调度器只执行 key 契约，裸 HttpClient 不承担缓存或网页安全策略。 (`crates/net/src/resource_loader.rs`, `crates/net/src/fetch_scheduler.rs`, `crates/net/src/client.rs`)
- browser 在接收 FetchRequest 时执行每标签 SecurityContext URL 检查，renderer 管理在飞请求与最终 URL；安全判断与异步完成不是同一层职责。 (`apps/browser/src/fetch_proxy.rs`, `apps/renderer/src/ipc_fetch.rs`)
- 嵌入 net_pool 在 ResourceLoader 外另加共享负缓存与结果转换，产品代理不能仅因共用传输库就假定拥有完全相同的失败重试行为。 (`crates/webview/src/net_pool.rs`, `apps/browser/src/fetch_proxy.rs`)

## Child Knowledge Nodes
- [请求、重定向、URL、编码与抓取调度](fetch-and-navigation.md) — 导航最终 URL、旧回包、请求合并、优先级与字符解码。
- [HTTP 缓存、连接策略、Cookie 与隐私](cache-cookie-and-privacy.md) — Vary、再验证、分区、落盘、代理与 Cookie。
- [WebSocket、文件资源与响应策略](websocket-and-resource-policy.md) — 握手上下文、消息代际、wire、防资源洪水与 MIME。
