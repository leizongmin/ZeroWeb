# Static Routing WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[static routing review TSV](2026-08-19-static-routing-review.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 11 case、关键 worker/helper、blob SHA | 一手事实 | 高 |
| 页面运行时 test 生成逻辑 | 70 个 subtest | 一手事实 | 高 |
| Goal support envelope | 基础生命周期/fetch/Cache 消费边界 | 项目契约 | 高 |
| skip 分类 | Static Routing API 不在当前目标范围 | 作者综合 | 高 |

## 0. 执行摘要

- 从 M1 review 收口后的 95 个逻辑 `review` 中，审计 11 个 Static Routing API 文件。
- 11 案实际产生 **70 个 subtest**，全部以 `InstallEvent.addRoutes()` 为被测入口。
- 它们不是普通 `fetch` interception 用例：路由规则可在 fetch handler 之前直接选择
  network、CacheStorage、fetch-event 或 network/fetch race。
- 当前 Goal 只覆盖基础 SW 生命周期、普通 fetch/respondWith/passThrough 和消费 Cache API；
  未包含 `InstallEvent.addRoutes()`、URLPattern 路由规则或 router timing 扩展，因此 11 案
  全部记为 `Unsupported(static-routing)`。
- 加上 M1 final 批次已裁决的
  `static-router-multiple-router-registrations.https.html`，初始 inventory 中 Static Routing
  family **12/12 已完成裁决**。
- 全量 inventory 的逻辑剩余 review 从 95 降为 **84**。

TSV SHA-256：
`59a81ca376f9ed2dee78bac557d559e562639bcea1ac1b7ce32ae728e0b690cb`。

## 1. 审计方法

1. 从剩余 review 中筛出 `add-routes.https.html`、`static-router-*` 以及 tentative
   static-router resource timing 文件。
2. 读取全部页面断言，并展开 `iframeTest()` 生成的 testharness 用例。
3. 追踪 module worker、router rules、MessageChannel 结果通道、CacheStorage 写入、
   dynamic response 和跨源 iframe。
4. 以完整上游文件为验收单位，不把普通 fetch 或 Cache API 断言拆出伪造基础分母。

浅层 `promise_test()` 文本计数仅得到 28。实际还包括：

- `static-router-main-resource`：9 个 `iframeTest`
- `static-router-request-method`：4 个 `iframeTest`
- `static-router-subresource`：18 个 `iframeTest`
- `static-router-resource-timing`：9 个 `iframeTest` + 5 个 `promise_test`

因此本批真实分母是 70，而不是 28。

## 2. 能力分层

### 路由注册与校验（2 case / 9 subtest）

- `add-routes.https.html`：验证绑定后的 `addRoutes()` 只可在 installing 阶段执行。
- `static-router-invalid-rules.https.html`：验证非法 ByteString/HTTP method、递归深度、
  规则总数及 condition/source 缺失。

这些断言直接要求 `InstallEvent.addRoutes()` 和静态路由 grammar，不属于普通 fetch event。

### Network/cache 路由（4 case / 34 subtest）

| Case | Subtest | 路由面 |
|------|--------:|--------|
| `static-router-main-resource` | 9 | main navigation 的 URLPattern/network/cache |
| `static-router-no-fetch-handler` | 3 | 无 fetch handler 时的 cache 与非法 source |
| `static-router-request-method` | 4 | GET/POST/PUT/DELETE network source |
| `static-router-subresource` | 18 | URLPattern/request/or/not/cache 与跨源子资源 |

worker 在 install 中写入 `cache.txt`。跨源 navigation case 的 `cache.html` 不是上游漏文件，
而是 `register-static-router-iframe.html` 动态写入 CacheStorage 的虚拟 URL。

### 条件、destination 与 race（3 case / 12 subtest）

- `static-router-mutiple-conditions`：search/mode/method/destination 的 AND 组合。
- `static-router-request-destination`：script destination 走 network，style 保持 fetch handler。
- `static-router-race-network-and-fetch-handler`：动态 server 与 fetch handler 竞速，并覆盖
  204/404 fallback。

race worker 使用 busy wait 制造 SW 慢响应，`direct.py` 用延迟制造 network 慢响应。该算法
不是普通 respondWith 的单一路径。

### 跨源 cache 与 timing（2 case / 15 subtest）

- `static-router-cross-origin-navigation`：远端 iframe 注册路由，本地 iframe 点击跨源链接，
  最终从远端 CacheStorage 载入虚拟 HTML。
- tentative resource timing：验证 `workerMatchedRouterSource`、
  `workerFinalRouterSource`、`workerRouterEvaluationStart` 和
  `workerCacheLookupStart` 等静态路由扩展字段。

## 3. 关键资源

| Resource | Bytes | Blob SHA |
|----------|------:|----------|
| `resources/add-routes.js` | 810 | `796acd19c12f4f79b587fa816516ed5e0993da31` |
| `resources/static-router-helpers.sub.js` | 2,605 | `0ab1f1fae1dd234d4e9206a10ea70a13fa10ed99` |
| `resources/static-router-sw.js` | 1,100 | `c0bd683f9182689b1c586f360beb74103939c9d2` |
| `resources/router-rules.js` | 5,051 | `27462b6c1d74f5cc673431ce3a2ae3ac1838282b` |
| `resources/static-router-sw.sub.js` | 418 | `04f9c5533a4890ef10f3cd3c1abed94dffcc424f` |
| `resources/imported-sw.js` | 299 | `04a894d77f87a432c4a14fc79819f3c7cd63e6d9` |
| `resources/static-router-no-fetch-handler-sw.js` | 822 | `1ba5fd7d463b7913fd13425bcf977e383c4795cd` |
| `resources/static-router-race-network-and-fetch-handler-sw.js` | 1,489 | `904ff0f46d8fd244d3d08c07f8dd0c724f9111c8` |
| `resources/register-static-router-iframe.html` | 1,272 | `f23353af9cf98e182cb69aa2ecedde4aa73061e0` |
| `resources/direct.py` | 499 | `d30d41b44e27a8426e78ef01724f0ab903a12b7b` |

## 4. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 11 case / 70 subtest | 页面 test/iframeTest 声明 | TSV 机器求和 | 一致 | 高 |
| 全部依赖 Static Routing API | 页面注册 helper | worker 的 `event.addRoutes()` | 一致 | 高 |
| cache.html 是运行时资源 | 页面目标 URL | fixture 的 `cache.put()` | 一致 | 高 |
| family 12/12 已裁决 | 本批 11 | M1 final 已裁决 1 | 一致 | 高 |
| 逻辑剩余 review 为 84 | 前序 95 | 95 - 11 | 一致 | 高 |

## 5. 后续输入

1. 当前 M2 基础 fetch 分母不包含本批 11 案。
2. 若未来 Goal 明确加入 Static Routing API，应新增独立里程碑并恢复 12 个完整上游 case。
3. 继续从剩余 84 个 review 中裁决普通 fetch/navigation/request-response 语义。

## 6. 质量审查

- [x] 11/11 case 正文已读，manifest blob SHA 匹配。
- [x] 10 个关键 worker/helper/handler 已读并记录 blob SHA。
- [x] `iframeTest()` 已展开，70 个 subtest 无文本计数低估。
- [x] Cache API 消费与 Static Routing API 本身已区分。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
