# Navigation、Redirect 与 Referrer WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[navigation review TSV](2026-08-19-navigation-review.tsv)
**关键资源**：[navigation resources TSV](2026-08-19-navigation-resources.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 15 source / 16 URL / 224 subtest | 一手事实 | 高 |
| worker/handler/fixture 闭包 | 32 个关键资源、blob SHA | 一手事实 | 高 |
| Goal support envelope | 单页面基础 fetch、多客户端排除 | 项目契约 | 高 |
| decision 分类 | defer/gated/skip 实施边界 | 作者综合 | 待 runtime 验证 |

## 0. 执行摘要

- 从 IDL harness 批次后的 70 个逻辑 `review` 中，审计 15 个
  navigation/redirect/referrer source。
- `navigation-redirect.https.html` 有 default/client 两个 variant，因此本批是
  **15 source / 16 generated URL / 224 subtest**。
- 裁决为：
  - **2 source / 2 subtest**：目标内 navigation defer；
  - **10 source / 126 subtest**：动态 server、redirect、header、Cache 或 timing gated；
  - **3 source / 96 subtest**：多窗口/多 registration client 与 SameSite cookie skip。
- `navigation-headers` 单文件有 83 项；`navigation-redirect` 的 37 个 helper case 在两个
  variant 中各执行一次，加 setup/cleanup 后共 78 项；浅层 `promise_test` 计数会严重低估。
- 全量 inventory 的逻辑剩余 review 从 70 降为 **55**。

机器清单 SHA-256：

- review：`0091c836a1e5dce649c532b5670c5e3150a38e8cf1ccf5723a88a9fc6493bdb4`
- resources：`b998bccf63e5df7a910a44199d288f1c671fe59ae5779a9a90d9a37647d9fed7`

## 1. 分母核算

| Case | URL | Subtest | 核算方式 |
|------|----:|--------:|----------|
| `navigation-headers` | 1 | 83 | 显式 setup + header matrix + cleanup |
| `navigation-redirect` | 2 | 78 | 每 variant：setup + 37 `redirect_test` + cleanup |
| `navigation-sets-cookie` | 1 | 16 | setup + 14 `navigate_test` + cleanup |
| `redirected-response` | 1 | 26 | setup 内动态注册 2 cleanup test + 23 场景 |
| 其余 11 source | 11 | 21 | 显式/helper 声明 |
| **合计** | **16** | **224** | |

整个上游 source/variant 是验收单位。不能从 83 项 header matrix、redirect client variant 或
cookie popup 文件中抽出“看起来简单”的断言替代完整分母。

## 2. 目标内 navigation defer

### Intercepted referrer（1 subtest）

静态 worker 在 install/activate 中 `skipWaiting()` + `clients.claim()`，对 scoped navigation
以合成 HTML `respondWith()`，子页面回传 `document.referrer`。它直接验证 M2 navigation
interception 和合成 document，不依赖动态 server。

### Extended navigation timing（1 subtest）

静态 worker 延迟 activate 500ms，并在 fetch handler 合成页面回传时间点；页面核对
`workerStart < activateWorkerEnd < fetchStart < handleFetchEvent`。该案属于目标内
navigation lifecycle/timing，但须等真实事件循环和 PerformanceNavigationTiming 接线。

## 3. 动态与高阶 gate

### Header matrix（83 subtest）

`navigation-headers` 组合：

- GET / POST
- same-origin / same-site / cross-site
- no SW / pass-through / fallback / request rewrite / navigation preload
- 0、1、2 段跨 site redirect
- Origin、Referer、Sec-Fetch-Site/Mode/Dest

Python handler读取真实请求头并生成回传页面；静态响应无法表达被测语义。

### Redirect 与 Cache（34 subtest）

- POST redirect body：动态 302 后确认 body 清除。
- redirect resolution：opaqueredirect 的相对 Location 基于 response URL list，并跨
  CacheStorage/clone 保留。
- HTTPS→HTTP：手动 redirect 必须得到 opaqueredirect。
- redirected response：follow/error/manual、generated/relative/20-hop redirect，
  并验证 clone、CacheStorage 后的 type/redirected/url。
- navigation timing redirect：合成、network fallback 和动态 redirect 同文件。

### Timing size 与 referrer（9 subtest）

- body size 文件同时覆盖 HTML、text 和 server-pipe gzip。
- request header echo handler验证页面 fetch、SW pass-through/rewrite 的 Referer。
- Referrer-Policy 文件依赖 response `.headers` 和 server pipe redirect。
- top-level script fetch handler把 register/update 请求头与 UUID 注入 worker，再经消息回传。

## 4. 明确 skip

### Multi-window clients（2 subtest）

`navigate-window` 打开独立 window，执行 navigate/back/forward/reload，并通过
`clients.matchAll()` 比较 auxiliary/top-level client URL、frameType 和 includeUncontrolled。
它属于明确排除的多窗口逐 client 语义。

### Multi-origin redirect clients（78 subtest）

`navigation-redirect` 同时持有两个同源 registration 和一个跨源 registration；client variant
检查 `resultingClientId`、`clients.get()` 与最终 Client.url。即便 default variant 主要验证
redirect 链，上游 source 通过两个 META variant 形成统一 family，因此整体排除当前分母。

### SameSite cookie popup（16 subtest）

两个 origin 上分别通过 popup 注册 SW，再用 popup GET/POST navigation 设置 Strict/Lax/None/
unspecified cookie，最后跨窗口清理。它属于 cookie policy + 多窗口/跨站 orchestration，不是
基础 fetch interception。

## 5. 关键资源

资源清单共 32 个对象、42,984 bytes，覆盖：

- fetch rewrite / redirect workers
- navigation header/body/redirect Python handlers
- timing/pass-through workers
- referrer policy `.headers` 与 header echo handler
- SameSite cookie handler和 popup fixtures
- registration/unregistration cross-origin fixtures

关键事实：

- `navigation-headers-server.py` 从真实 request 读取 Origin/Referer/Sec-Fetch 头。
- `redirect-worker.js` 用 CacheStorage 记录 request URL/resultingClientId，并调用
  `clients.get()`。
- `setSameSite.py` 返回四个不同 SameSite 属性的 Set-Cookie。
- `test-request-headers-worker.py` 每次注入请求头和 UUID，强制 update 产生新 worker。

## 6. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 15 source / 16 URL / 224 subtest | 页面/helper/META | review TSV | 一致 | 高 |
| navigation-redirect 为 78 | 37 helper 调用 × 2 variant | setup/cleanup 各 variant 2 项 | 一致 | 高 |
| 两案属于目标内 defer | 静态 worker 正文 | Goal navigation fetch 范围 | 一致 | 高 |
| 三案超出 support envelope | popup/multi-registration/client/cookie | 多客户端明确排除 | 一致 | 高 |
| 逻辑剩余 review 为 55 | 前序 70 | 70 - 15 | 一致 | 高 |

## 7. 后续输入

1. M2 navigation interception 落地后优先恢复 intercepted-referrer。
2. PerformanceNavigationTiming 接线后恢复 extended timing。
3. dynamic WPT response adapter 后按 header/redirect/referrer 组恢复 10 个 gated source。
4. multi-window/client 与 SameSite cookie 三案保持 skip，除非扩大 support envelope。
5. 继续裁决剩余 55 个 request/response、timing、media/security review。

## 8. 质量审查

- [x] 15/15 source 正文已读，manifest blob SHA 匹配。
- [x] 32 个关键 worker/handler/fixture 已读并记录 blob SHA。
- [x] helper 与 META variant 已展开，224 个 subtest 无文本计数低估。
- [x] navigation core、dynamic gate 与 multi-client skip 已分开。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
