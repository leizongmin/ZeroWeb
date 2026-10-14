# 请求、重定向、URL、编码与抓取调度

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取场景：主文档重定向后相对资源错误、重复抓取、排队饥饿、旧导航响应或 CSS 编码问题。入口为 `HttpClient`、`ResourceLoader`、`PerOriginFetchScheduler`，产品路径经 browser `TabFetchProxy` 与 renderer `InflightIpcFetches` 协作；嵌入路径由 `net_pool` 提供宿主。

## Gotchas
- HTTP 客户端关闭 reqwest 自动重定向，自行处理 301/302/303/307/308；缺少 Location 与超过跳数上限都是错误，不能当作普通响应继续解析。 (`crates/net/src/client.rs`)
- POST 经 301/302、请求经 303 转 GET 时同时清除 body 和内容相关请求头；307/308 保留方法与 body，修改任一传输实现必须核对另外两条同步、异步和流式路径。 (`crates/net/src/client.rs`)
- 跨源重定向会剥离 Authorization、Cookie 等敏感头；头一旦从活动头集合删除，后续跳回原源也不会自动恢复。 (`crates/net/src/client.rs`)
- 非流式异步请求的瞬态网络重试仅适用于尚未推进重定向、无 body 的 GET；Timeout、Proxy 和非 GET 不走此重试，不能用它推断所有请求有统一重试策略。 (`crates/net/src/client.rs`, `git:8d8d0bdae89c87b2cb19e3e8c2895feb2da5d719`)
- CSS 解码当前实现按 BOM、开头 @charset、Content-Type charset、UTF-8 顺序选择；参数名必须恰为 charset，xcharset 不会命中，这描述当前代码而非宣称完整规范一致性。 (`crates/net/src/charset.rs`)

## Architecture
- 主文档最终 URL 经内部响应头 X-Zero-Final-Url 传到 renderer；只有成功的 document 响应更新 document_url，重定向后的基准 URL 不能仍使用最初请求地址。 (`apps/renderer/src/ipc_fetch.rs`, `apps/browser/src/fetch_proxy.rs`)
- renderer 流式资源响应会移出再插回同一个在飞条目累积字节，最终响应才完成通道；导航 clear 同时丢弃待完成条目和主文档 URL。 (`apps/renderer/src/ipc_fetch.rs`)
- 共享抓取只按显式身份 key 合并；ResourceRequest 将分区、cache mode、超时、去 fragment URL 和全部排序请求头纳入身份，未知 Vary 时保守少合并而不混淆变体。 (`crates/net/src/resource_loader.rs`, `crates/net/src/fetch_scheduler.rs`)
- 调度器同时限制每源和总并发，队列上限由总并发乘 16 后钳制到 64–1024；后到消费者能提升仍排队的共享请求优先级，但不会抢占已经运行的任务。 (`crates/net/src/fetch_scheduler.rs`)
- browser 导航会递增 tab epoch 并清除该标签 pending 与流式登记；这是宿主取消/回包边界，不等于底层所有网络任务立即终止。 (`apps/browser/src/fetch_proxy.rs`)
