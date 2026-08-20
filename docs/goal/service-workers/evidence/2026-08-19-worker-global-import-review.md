# Worker Global 与 importScripts WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M3 `importScripts(data:)` promoted to core
**逐案裁决**：[worker global/import review TSV](2026-08-19-worker-global-import-review.tsv)
**静态资产**：[static assets TSV](2026-08-19-worker-global-static-assets.tsv)
**静态 subtest**：[static subtests TSV](2026-08-19-worker-global-static-subtests.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 13 case、22 个关键资源、blob SHA | 一手事实 | 高 |
| testharness remote worker 协议 | wrapper 与 worker-side subtest 计数 | 一手事实 | 高 |
| Goal support envelope | worker runtime、基础 fetch 与单页面边界 | 项目契约 | 高 |
| decision 分类 | core/defer/gated/skip 实施顺序 | 作者综合 | 待运行验证 |

## 0. 执行摘要

- 从 Static Routing 批次后的 84 个逻辑 `review` 中，筛出 14 个 no-signal 文件。
- 本批审计其中 13 个 worker-global/import/interface 文件；`idlharness.https.any.js` 另由
  [IDL harness evidence](2026-08-19-idlharness-review.md) 专门解析，避免把 4 个生成 context
  的 IDL 子测试误算为一个。
- 13 案实际产生 **53 个 subtest**：
  - **2 case / 5 subtest**：静态 `scriptURL` + `importScripts(data:)` core；
  - **5 case / 15 subtest**：worker global/interface/import runtime defer；
  - **4 case / 29 subtest**：动态 server/cross-origin import gated；
  - **1 case / 1 subtest**：M2 uncontrolled fetch bypass defer；
  - **1 case / 3 subtest**：dedicated module worker interception skip。
- 初始 `direct_dependency_signals=none` 漏掉了 worker script 内的动态 MIME、redirect、server
  stash、跨源 import、worker-testharness 和 module import；不能作为可执行性结论。
- static wave 已固定为 **2 case / 5 subtest / 4 asset / 1,681 bytes**，记入
  testharness 账本并落地 fetch/audit/regression Make targets。
- 全量 inventory 的逻辑剩余 review 从 84 降为 **71**。

TSV SHA-256：

- review：`c5760e774913f52452ac5af0c825c12053c972ad00a4f903e5400d286b28f951`
- static assets：`970b19b3eb95233c197ef8539d36bbb43f4adf79f001cabc581b56401c758d73`
- static subtests：`48c028ad34572bc1f7fd25b8a708e52e272d303fcc127f170660076b8c191627`

## 1. 分母核算

`fetch_tests_from_worker()` 会将远端 worker test 加入页面 harness，同时页面包装
`promise_test` 仍是一个独立 subtest。因此：

- `import-scripts-mime-types`：1 setup + 1 wrapper + 5 invalid + 16 valid = **23**
- `interface-requirements-sw`：1 wrapper + 3 worker tests = **4**
- classic/module `no-dynamic-import`：各 3 个 URL = **3 + 3**
- `serviceworkerobject-scripturl`：`url_test()` 调用四次 = **4**
- dedicated interception helper 调用三次 = **3**

其余页面按显式 test 声明计数，合计 53。

## 2. M1 runtime 输入

### 静态 core（2 case / 5 subtest）

`serviceworkerobject-scripturl.https.html` 使用同一个静态 `empty-worker.js`，验证相对 URL、
fragment、query 和 absolute URL 的 `ServiceWorker.scriptURL`。它不依赖 iframe、动态 server、
fetch interception 或消息结果通道，已加入 M1 runner 的下一静态 wave。

`import-scripts-data-url.https.html` 使用静态 worker 调用
`importScripts('data:text/javascript,')`；M3 typed import fetch/evaluate graph 落地后已提升为
core，固定 revision baseline 为 Pass。

资产恢复与审计：

- `make fetch-wpt-service-workers-static-wave`
- `make audit-wpt-service-workers-static-wave`
- `make test-wpt-service-workers-static-wave-assets`

当前环境已验证 4/4 restore 与 verify-only；篡改、缺失必须失败，restore 可修复篡改。
Tier A 18/18 与 next-wave 7/7 的共享根审计同时保持通过。

### Worker global 与 interface（3 case / 9 subtest）

- `global-serviceworker.https.any.js`：在 SW global 直接验证 `self.serviceWorker` 的对象身份、
  parsed/installing/activating 状态和 startup 自消息。
- `historical.https.any.js`：验证 `FetchEvent.prototype.targetClientId` 不存在。
- `interface-requirements-sw.https.html`：验证 ExtendableEvent/FetchEvent constructor、
  Request 投影，以及 XHR/createObjectURL 不暴露。

这些用例要求 runner 原生驱动 serviceworker global 或完成 remote worker result channel，
属于 typed SW runtime 的验收输入。

### Import runtime（2 case / 6 subtest）

- 两个 no-dynamic-import 文件：classic/module SW 中三种 `import()` 都必须拒绝；module case
  还先静态 import 同一模块，验证已加载模块也不能动态导入。

这些不需要动态 server，但依赖 classic/module worker loader 与 import policy。

## 3. 动态 importScripts 门控

| Case | Subtest | 动态语义 |
|------|--------:|----------|
| `import-scripts-cross-origin` | 1 | 远端 HTTPS origin + 动态版本脚本 |
| `import-scripts-mime-types` | 23 | handler 按 query 设置或省略 Content-Type |
| `import-scripts-redirect` | 3 | redirect、stash 请求次数和 update body 变化 |
| `import-scripts-resource-map` | 2 | 时间版本脚本与 query 生成不同变量 |

这些测试不能用 Python 文件字节替代响应，也不能固定成单一 JavaScript；被测行为正是响应头、
重定向、请求次数或每次返回值。

## 4. Fetch 与 scope 边界

`uncontrolled-page.https.html` 激活一个总是返回 `ERROR` 的 fetch worker，再从 scope 外页面用
XHR 请求静态文本。正确行为是请求绕过 SW 并返回网络内容，属于 M2 scope routing/pass-through
的直接验收用例。

`dedicated-worker-service-worker-interception.https.html` 则要求 Service Worker 拦截 dedicated
module worker 的顶层脚本、静态 import 和动态 import。当前 Goal 的基础 fetch 面以页面请求为
中心，且 support envelope 不包含 worker client 控制，因此整案记
`Unsupported(dedicated-worker-interception)`。

## 5. 关键资源

关键闭包共 22 个对象、10,657 bytes。代表性资源：

| Resource | Bytes | Blob SHA |
|----------|------:|----------|
| `resources/service-worker-interception-service-worker.js` | 371 | `6b43a3769637dcbc3574b7010ea5c93be73d7b00` |
| `resources/import-scripts-mime-types-worker.js` | 1,494 | `7658eeace695e3debd361b1f0c05282ffc420b28` |
| `resources/interface-requirements-worker.sub.js` | 2,492 | `a3f239b654811317bfcedd09149afab928235c1b` |
| `resources/no-dynamic-import.js` | 533 | `ecedd6c5d75c7d667543a2cbd2ea849194d31bc8` |
| `resources/import-scripts-version.py` | 459 | `cde28544e60a0613debe98bfe6c44bdfb610317b` |
| `resources/import-scripts-get.py` | 246 | `ab7b84e3e34e3f6ccff48a497d10f9bdc356fda3` |
| `resources/mime-type-worker.py` | 141 | `92a602e634cbf8b66d28f36eb0d1616f06ba239c` |
| `resources/redirect.py` | 855 | `bd559d5d1e252e33863fe2ae369370556cfd4477` |
| `resources/update-worker.py` | 2,257 | `5638a8849cb749471ef01413bb95386aa4857712` |
| `resources/empty-worker.js` | 15 | `49ceb2648a93410bdd5ee53ef0e114146210741b` |
| `resources/fail-on-fetch-worker.js` | 142 | `517f289fbc8e43b1d540a47761538fe84b121c48` |

## 6. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 13 case / 53 subtest | 页面/worker test 声明 | TSV 机器求和 | 一致 | 高 |
| MIME 文件为 23 subtest | 页面 2 项 | worker 21 项 | 一致 | 高 |
| static wave 为 2 case / 5 subtest | 页面显式 promise_test | 四个固定 revision asset | 一致 | 高 |
| 四个 import 文件需动态 server | worker URL/handler | header/redirect/stash/time 语义 | 一致 | 高 |
| 逻辑剩余 review 为 71 | 前序 84 | 84 - 13 | 一致 | 高 |

## 7. 后续输入

1. RFC 批准后，在 Tier A 和 next-wave 后执行 static-wave 4 个 subtest。
2. typed SW runtime/worker result channel 落地后恢复剩余 5 个 defer case。
3. 动态 WPT server adapter 落地后恢复 4 个 gated importScripts case。
4. M2 scope routing 落地后恢复 uncontrolled-page。
5. `idlharness.https.any.js` 已固定为 4 个 generated URL / 787 个 subtest。

## 8. 质量审查

- [x] 13/13 case 正文已读，manifest blob SHA 匹配。
- [x] 22/22 关键 worker/handler/module 已读并记录 blob SHA。
- [x] remote worker 和 helper 生成测试已展开，53 个 subtest 无文本计数低估。
- [x] 静态 import runtime 与动态 server import 语义已分开。
- [x] static-wave 4/4 资产已固定并记入 testharness 账本。
- [x] static-wave restore/verify/fail-closed 回归通过，Tier A/next-wave 审计无退化。
- [x] 未修改 runtime 源码、WPT 数据或既有 inventory 初筛记录。
