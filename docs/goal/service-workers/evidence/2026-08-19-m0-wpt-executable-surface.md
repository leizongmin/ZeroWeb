# Service Worker M0 WPT 可执行面分析

**日期**：2026-08-19
**基线提交**：`e3f5271c2`
**上游观察点**：WPT `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零源码改动）

### 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT 官方 manifest API（version 9） | 完整文件分母、test URL 展开 | 一手事实 | 高 |
| 固定 revision 的 jsDelivr 源文件镜像 | 294/294 个 testharness 正文 | 一手事实 | 高 |
| ZeroWeb 当前源码与 runner | 当前可执行能力 | 一手事实 | 高 |
| Service Workers 规范、Chromium/MDN 文档 | 规范与架构补证 | 外部官方资料 | 高 |
| A-E 分层、文件名聚类 | 实施排序 | 作者综合 | 待实现验证 |

## 0. 结论

- 当前 `zero-wpt-runner` 对 `service-workers` 的**端到端可执行文件数为 0**：本地
  Tier A 资产虽已恢复到独立 WPT root，但 CLI 没有 service-worker testharness 入口，页面
  注册 shim 也不抓取或执行 worker 脚本。
- 固定 revision 的完整 manifest 包含 **801 个源文件**：294 个 testharness 源生成
  331 个测试 URL，另有 499 个 support、2 个 crashtest、5 个 manual 和 1 个 reftest。
  核心 `service-worker/` 子树占 276 个 testharness 源 / 277 个 URL。
- [逐文件清单](2026-08-19-m0-wpt-case-inventory.tsv) 记录全部 294 个 testharness 源的
  manifest SHA、URL/context、里程碑、文件名聚类、直接依赖信号和候选裁决。
- [M1 候选资源闭包](2026-08-19-m0-m1-candidate-resource-closure.md) 将初筛 12 个 case
  校准为 8 个静态首批、3 个高阶事件案和 1 个动态 server 阻塞案。
- [Tier A 验收合约](2026-08-19-m1-tier-a-baseline-contract.md) 固定首批
  8 case / 28 subtest / 18 asset 及五阶段验收顺序。
- [M1 第二批裁决](2026-08-19-m1-next-wave-review.md) 将 14 个
  `M1 + review + no-signal` 文件校准为 3 core / 5 advanced / 5 dynamic-server / 1 update；
  3 个 core case 已资产化。
- [M1 iframe 裁决](2026-08-19-m1-iframe-review.md) 将 11 个 iframe-only 文件校准为
  single-iframe/worker/controller defer、fetch gated 和 multi-client skip。
- [M1 message-channel 裁决](2026-08-19-m1-message-review.md) 将 7 个文件校准为
  lifecycle result-channel defer、controller/fetch/update gate 和 cross-origin client skip。
- [M1 剩余裁决](2026-08-19-m1-final-review.md) 完成最后 25 个 M1 review：
  19 个 dynamic/server gated，6 个 support-envelope skip；M1 review 57/57 已收口。
- [Static Routing 裁决](2026-08-19-static-routing-review.md) 将 11 case / 70 subtest
  明确排除出基础 fetch 分母；连同前批 1 案，family 12/12 已裁决。
- [Worker Global/import 裁决](2026-08-19-worker-global-import-review.md) 将 13 case /
  53 subtest 分为 static core、runtime defer、server gated、M2 defer 与 worker-client skip；
  其中 scriptURL 1 case / 4 subtest 已资产化。
- [IDL harness 裁决](2026-08-19-idlharness-review.md) 固定 4 个 generated URL /
  787 个 subtest，并按 window、dedicated/shared、serviceworker context 分层。
- [Navigation/redirect 裁决](2026-08-19-navigation-review.md) 将 15 source / 16 URL /
  224 subtest 分为 navigation defer、dynamic gated 与 multi-client/cookie skip。
- [Request/response 裁决](2026-08-19-request-response-review.md) 将 17 source / 83 subtest
  分为 fetch/response defer、timing/server gated 与 form/File skip。
- 这不等于全部上游用例都超出 ZeroWeb 环境。M1 完成后，单页面、单注册、静态资源的
  生命周期用例可形成第一批真实基线；M2 完成后再加入单客户端 fetch/respondWith 用例。
- iframe、多客户端、SharedWorker、跨 origin、动态服务端 handler、WebSocket 和
  navigation preload 依赖项不进入首批分母。每个 skip 必须记录具体依赖，不能用目录级
  blanket skip 抬高通过率。
- `make import-wpt` 是 reftest pair 导入器，不适合本目录的 testharness 用例。应沿用
  IndexedDB/DOM 的 pinned fetch script + `imported-testharness.txt` 账本模式。

## 1. 当前运行能力

| 能力 | 当前事实 | 证据 | 裁决 |
|------|----------|------|------|
| 上游语料 | Tier A 18 资产在独立 WPT root，主 runner 未接线 | fetch target + 本地校验 | 资产就绪、不可运行 |
| CLI 入口 | 只有 DOM、Canvas、IndexedDB 等专用 testharness 入口 | `tests/wpt-runner/src/main.rs` | 不可发现 SW case |
| 页面注册 | `register()` 只建 JS 对象并用两个 timer 推进状态 | `crates/engine/src/js_dom_shim/part02.js:2496-2591` | 仅表面近似 |
| worker 脚本 | `register(scriptURL)` 不调用 `ScriptSourceFetcher`/网络 | 同上；`part05.js` 仅 Dedicated Worker 使用 fetcher | 不执行 |
| 生命周期事件 | 无 install/activate 事件与 `waitUntil()` | 同上 | 不可验证 |
| fetch 事件 | 页面 `fetch()` 直达宿主 handler | `crates/engine/src/fetch_bridge.rs` | 不经 SW |
| 本地资源映射 | WPT 图片、脚本和 fetch 可映射到 `wpt-data` | `tests/wpt-runner/src/testharness.rs:1069-1209` | 可复用 |
| testharness 轮询 | 单 WebView 可执行 Promise test 并收结果 | `tests/wpt-runner/src/testharness.rs:1211-1320` | 可复用 |
| 静态 SW registry | storage 有状态机和 cache-first 静态拦截 | `crates/storage/src/service_worker.rs` | 不是事件执行环境 |
| WebView 导航拦截 | 手工激活后主文档可命中 registry cache | `crates/webview/src/webview.rs:954-978` | 仅已有底座 |

**基线口径**：现有 engine/WebView 单测验证的是 shim 或 Rust registry，不计作上游 WPT
通过。M0 不制造 inline 测试代替 WPT。

## 2. 上游依赖模型

上游典型页面 `activation-after-registration.https.html` 依赖：

1. `/resources/testharness.js` 和 `resources/test-helpers.sub.js`；
2. `navigator.serviceWorker.register()` 抓取 `resources/empty-worker.js`；
3. 注册 Promise resolve 时 `registration.installing` 可见；
4. worker 经真实状态变更事件到 `activated`；
5. 清理阶段按绝对 scope 精确注销。

其中 1 可由现有本地资源内联机制承载；2-4 是 M1 的 driving 缺口；5 需要把页面对象与
Rust 注册记录统一，而不是保留 shim 私有数组。

上游公共 helper 还包含 iframe、MessageChannel、跨 origin 登录、WebSocket、SharedWorker
和服务端动态脚本。存在 helper 不代表每个 case 都依赖全部能力，因此分类必须按 case
实际调用链，而不是看到公共 helper 后整目录跳过。

## 2A. 完整 manifest 普查

### 文件与 URL 分母

| 类型 | 源文件 | 展开测试 URL | 说明 |
|------|-------:|-------------:|------|
| testharness | 294 | 331 | 276/277 属核心 `service-worker/` |
| support | 499 | 不适用 | worker、fixture、server handler、header 配置 |
| crashtest | 2 | 2 | 核心与 cache-storage 各 1 |
| manual | 5 | 5 | 全在核心子树，不纳入自动基线 |
| reftest | 1 | 1 | SVG target，非 M1 首批 |
| **合计** | **801** | **339 个自动/手动入口** | testharness + crash/manual/ref |

testharness 的 331 个 URL 由核心 service-worker 277、cache-storage 50 和根
`idlharness.https.any.js` 的 4 个 global 变体组成。核心 URL 中 274 个标记为 HTTPS，
11 个属于 navigation preload，10 个为 tentative，1 个要求 HTTP/2。

### 核心 testharness 文件名聚类

以下分类按路径关键词互斥归类，是实施规模上界，不是 WPT 官方分类：

| 聚类 | 源文件 | 最早里程碑 | 裁决 |
|------|-------:|-----------|------|
| registration/lifecycle | 68 | M1 | 逐案检查资源闭包后纳入 |
| fetch/interception | 60 | M2 | 等 fetch 管线门禁 |
| clients/control | 31 | M3 | 单客户端子集可纳入，多客户端 skip |
| message | 11 | M3 | 等 postMessage |
| navigation preload | 11 | 排除/远期 | 当前 goal 不实现 |
| other API/security | 95 | 分散 | 按 secure context、imports、routing 等拆分 |
| **合计** | **276** | | |

### support 资源结构

| 子树 | support 文件 | JS | HTML | Python handler | headers/asis | 其他 |
|------|-------------:|---:|-----:|---------------:|-------------:|-----:|
| service-worker | 483 | 246（含 5 个 `.sub.js`） | 133（含 2 个 `.sub.html`） | 66 | 11 | 27 |
| cache-storage | 14 | 6 | 3 | 2 | 0 | 3 |

66 个核心 Python handler 是 WPT server 依赖的直接证据。它们不能由当前静态
`wpt_data_fetch_handler` 正确模拟，因此相关 case 必须标 `Unsupported`，或先实现明确的
fixture adapter；不能将 `.py` 当普通文本响应。

> **来源说明（第 2A 章）**
>
> - **一手事实**：WPT manifest API `sha=04067ce...` 的完整 `items` 树，经确定性遍历统计。
> - **作者综合**：文件名聚类与里程碑映射；数量来自完整 manifest，分类规则不是上游定义。

## 3. 分层导入建议

| 层级 | 首批主题 | 开启条件 | 预期处理 |
|------|----------|----------|----------|
| A | 单页面 register、install、activate、statechange、scope、unregister | M1 | 纳入通过率分母 |
| B | worker 内 `self`、`registration`、`location`、`importScripts` 的静态资源用例 | M1 | 纳入通过率分母 |
| C | 单受控客户端 fetch、pass-through、`respondWith(new Response)` | M2 | 纳入通过率分母 |
| D | CacheStorage 驱动的 cache-first fetch | storage-cache-api M1 + M2 | 纳入通过率分母 |
| E | `postMessage`、`skipWaiting()`、单客户端 `clients.claim()` | M3 | 纳入通过率分母 |
| S | 多 iframe/window、SharedWorker、多客户端枚举 | 超出当前 headless 单页面 envelope | skip，逐案注明 |
| S | `.py` 动态响应、stash/counter、WebSocket、认证、跨 origin TLS | WPT server infra 未具备 | skip，逐案注明 |
| S | navigation preload、push、background sync | goal 明确排除或远期 | skip，逐案注明 |

首个 driving case 建议固定为
`service-workers/service-worker/activation-after-registration.https.html`。它不需要 iframe
或动态服务端，直接验证 M1 的核心链路，失败信号也能区分“脚本未抓取”“install 未派发”
和“状态事件未推进”。

## 3A. 正文全量依赖信号

固定 revision 的 CDN 正文已取得 294/294 个 testharness 源，其中核心子树 276/276。
下表按源码正文正则扫描，信号可重叠：

每个正文均按 Git blob 规则重新计算 SHA-1，并与 manifest 的对象 SHA 比较，294/294
匹配。逐文件 inventory 的 SHA-256 为
`8905f3de41dd53432758461b64cf68a59ebcdecd970f3d0add724957e709a3e7`。

| 信号 | 命中文件 | 对首批 runner 的含义 |
|------|---------:|----------------------|
| iframe 创建/helper | 174 | 当前单 WebView runner 无真实子 browsing context |
| 动态 server（`.py`/stash/pipe） | 96 | 静态文件映射不足 |
| fetch/respondWith | 79 | M2 前不纳入 |
| cross-origin host helper | 62 | 需要多 origin + TLS fixture |
| MessageChannel/MessagePort | 56 | M3 或专门消息基础设施 |
| navigation preload | 11 | 当前 goal 排除/远期 |
| SharedWorker | 8 | 多 worker client，当前排除 |
| testdriver | 4 | 当前 testdriver adapter 需逐项核对 |
| WebSocket | 2 | 需要 WSS fixture |
| HTTP/2 | 1 | 当前静态 fixture 不提供 H2 |

核心全量中 228/276 至少命中一个上述重依赖信号；其余 48 个只是“未命中已知信号”的筛选
队列，不能直接当可执行分母，因为依赖还可能藏在外链 helper 或资源响应语义中。

### M1 首批初筛候选

从这 48 个文件中再按目标范围和资源复杂度筛出 12 个候选：

1. `activate-event-after-install-state-change.https.html`
2. `activation-after-registration.https.html`
3. `install-event-type.https.html`
4. `onactivate-script-error.https.html`
5. `oninstall-script-error.https.html`
6. `register-default-scope.https.html`
7. `registration-basic.https.html`
8. `registration-scope.https.html`
9. `registration-script-url.https.html`
10. `registration-script.https.html`
11. `registration-service-worker-attributes.https.html`
12. `rejections.https.html`

这些文件是 M1 资产化的初始审计队列，不是通过率承诺。传递资源闭包审计现已完成：
第 3、4、5 项归 Tier B（worker testharness/error event），第 10 项归 Tier C（Python
动态 handler），其余 8 项归静态 Tier A 并已由独立 fetch target 资产化。首个 driving
case 仍是第 2 项。

> **来源说明（第 3A 章）**
>
> - **一手事实**：固定 revision 的 294 个 testharness 正文。
> - **作者综合**：依赖信号正则与 12 个候选筛选。
> - **限制**：信号只扫描 testharness 主文件正文；外链 helper 的传递依赖仍需资源闭包分析。

## 4. 导入与 runner 设计约束

1. Tier A pinned fetch script 已落，只抓 8 case 的 18 个闭包对象并固定 WPT commit。
2. 新增 `testharness-service-workers` CLI，复用 `run_testharness_html_inner`，但由 SW fixture
   host 提供脚本 URL、origin、注册清理和事件循环 drain。
3. Tier A 8 case 已记入 `imported-testharness.txt`；`wpt-data` 仍按现有约定不入主仓。
4. runner 对未满足依赖返回 `Unsupported/NotRun` 并给出枚举原因；不得把超时算作 skip。
5. 第一份通过率报告同时列文件数、subtest 数和 skip 原因分布，防止只报通过率百分比。
6. fetch script 只负责资产恢复，不定义 SW host 契约；CLI/runner 继续等 M0 RFC 批准后
   与 runtime 实现一起落地。

## 5. 证据矩阵

| 关键结论 | 来源 1 | 来源 2 | 一致性 | 置信度 | 处理 |
|----------|--------|--------|--------|--------|------|
| 当前无真实 SW 脚本执行 | `part02.js:2496-2591` | R3318 测试仅断言模拟状态 | 一致 | 高 | 直接采用 |
| 当前 WPT SW 可执行数为 0 | 本地无目录 | CLI/runner 无入口 | 一致 | 高 | 直接采用 |
| 通用 testharness 与本地资源映射可复用 | `testharness.rs:1044-1209` | DOM/IndexedDB runner 已使用同内核 | 一致 | 高 | 直接采用 |
| activation case 可作为 M1 首案 | 上游 case 实际调用链 | 上游 `test-helpers.sub.js` | 一致 | 高 | 直接采用 |
| 不能整目录跳过 | helper 含可选重依赖 | activation case 未调用这些 helper | 一致 | 高 | 逐案分类 |
| fetch 用例必须等 M2 | goal 依赖约束 | 当前 FetchBridge 直达网络 handler | 一致 | 高 | M2 开启 |
| cache-first 必须等兄弟 goal | storage-cache-api master 显示 M1 未启动 | 当前页面无 `caches` | 一致 | 高 | 联合门禁 |
| 完整 testharness 分母为 294 源/331 URL | WPT manifest version 9 | 本地确定性遍历结果 | 一致 | 高 | 直接采用 |
| 重基础设施用例占多数 | 核心 228/276 命中直接信号 | support 中有 68 个 Python handler | 一致 | 高 | 逐案依赖闭包 |
| 逐文件清单无遗漏 | inventory 294 唯一路径/331 URL | manifest 294 源/331 URL | 一致 | 高 | 直接采用 |
| 首批静态 case 为 8 个 | 12 case 传递资源闭包 | 39/39 闭包对象 blob SHA 匹配 | 一致 | 高 | Tier A |

## 6. 来源与限制

### 一手事实

1. [Service Workers 规范](https://www.w3.org/TR/service-workers/)
2. [Chromium Service Worker 架构说明](https://github.com/chromium/chromium/blob/main/content/browser/service_worker/README.md)
3. [WPT 项目说明](https://github.com/web-platform-tests/wpt/blob/master/README.md)
4. [WPT activation case](https://github.com/web-platform-tests/wpt/blob/master/service-workers/service-worker/activation-after-registration.https.html)
5. [WPT Service Worker helper](https://github.com/web-platform-tests/wpt/blob/master/service-workers/service-worker/resources/test-helpers.sub.js)
6. [WPT manifest API](https://wpt.fyi/api/manifest?sha=04067ce9c7c2165e71ad7d0dde10a4c5cb394a83)
7. `crates/storage/src/service_worker.rs`
8. `crates/webview/src/webview.rs`
9. `crates/engine/src/js_dom_shim/part02.js`
10. `crates/engine/src/fetch_bridge.rs`
11. `tests/wpt-runner/src/testharness.rs`
12. `tests/wpt-runner/src/main.rs`

### 外部补证

13. [MDN: Using Service Workers](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API/Using_Service_Workers)
14. [MDN: ServiceWorkerGlobalScope](https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerGlobalScope)

### 限制

- GitHub/Gitiles clone 在本机网络不可达。完整**文件与 URL 分母**由 WPT 官方 manifest
  API 获取；294 个 testharness 正文由固定 revision 的 jsDelivr 镜像补齐。
- 因当前 runner 没有 SW 入口，未运行伪基线。这里的“0”指当前环境可由标准入口执行的
  真实 SW WPT 文件数，不是未来层级 A/B 的预估通过数。
- 层级 A-E 是实施排序（作者综合），不是上游 WPT 自带分类。

> **勘误说明**：本报告上一版仅取得 221/294 个 testharness 正文，因此把依赖信号标为
> 75.2% 样本下界。本轮已补齐剩余 73 个文件，§3A 和逐文件清单现覆盖 294/294；旧的
> 140/63/58 等样本计数由全量 174/96/79 等计数替代。

## 7. 质量审查

- [x] 核心结论均有两处本地源码或源码 + 上游 case 交叉证据。
- [x] 区分“当前不可执行”和“未来可纳入”，未把 skip 当 pass。
- [x] 未把 shim 单测计入 WPT。
- [x] 已记录网络取证限制和固定上游 commit。
- [x] 完整 manifest 分母与 294/294 正文信号逐文件对齐。
- [x] inventory 可反算 294 个唯一路径、331 个 URL 和 12 个候选。
- [x] 12 个初筛候选已完成资源闭包并校准为 8/3/1。
- [x] 14 个 M1/no-signal + 11 个 M1/iframe-only + 7 个 M1/message-channel +
      25 个 M1/final review 已完成传递审计，M1 review 57/57。
- [x] Static Routing family 12/12 已裁决，剩余逻辑 review 84。
- [x] 剩余 no-signal review 14/14 已裁决；剩余逻辑 review 70。
- [x] Navigation/redirect family 15 source / 224 subtest 已裁决；剩余逻辑 review 55。
- [x] Request/response/timing family 17 source / 83 subtest 已裁决；剩余逻辑 review 38。
- [x] 未修改源码、WPT 数据或共享账本。
