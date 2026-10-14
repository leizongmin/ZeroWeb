---
date: 2026-10-14
modules: apps/browser, crates/net
---

# 流式图片路径绕过 HTTP 缓存

## 表现

同一页面重复请求带 `Cache-Control: max-age` 的图片时，服务器仍收到完整 GET；有 ETag 的图片也无法通过 304 复用旧 body。

## 根因

`TabFetchProxy::enqueue` 把带 `X-Zero-Stream-Image` 的图片请求直接送入 `start_image_stream`，该函数调用 `HttpClient::send_async_stream_with_timeout`，绕过了 `ResourceLoader` 的缓存查找、条件再验证和写入。

## 修复

流式请求先查分区缓存。新鲜命中直接返回缓存响应；过期项转入常规加载器发送验证器并合并 304；未命中继续渐进传输，成功后才写入缓存。流式缓存副本限制为 8 MiB；更大的图片继续渐进传输但不保留额外缓存副本。`no-store` 不写入缓存，`only-if-cached` 未命中不发网络请求。

常驻本地回归用真实 loopback HTTP 请求计数与响应体断言覆盖首次 GET、热命中、ETag/304、`no-store`、`only-if-cached` 及流式分块。仅测 `ResourceLoader` 本身不足以发现此类绕过，因为缺陷位于 browser 的特殊图片分支。

## 避免

新增渐进式或旁路传输前，检查其是否经过缓存分区、请求合并、优先级与错误处理。无法复用常规加载器时，至少保留“新鲜命中、条件再验证、成功写回”三条同一资源生命周期回归，并为额外 body 缓冲设明确上限。
