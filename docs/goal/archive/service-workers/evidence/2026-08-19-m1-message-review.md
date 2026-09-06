# M1 message-channel 生命周期 WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[message review TSV](2026-08-19-m1-message-review.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 7 case、helper、worker、fixture、SHA | 一手事实 | 高 |
| Goal support envelope | 消息/controller 长期范围、多客户端排除 | 项目契约 | 高 |
| decision 分类 | runner/runtime 实施顺序 | 作者综合 | 待运行验证 |

## 0. 执行摘要

- 从 iframe 批次后逻辑剩余的 127 个 `review` 中，审计 7 个
  `M1 + direct_dependency_signals` 含 `message-channel` 的文件。
- 7 案实际产生 **18 个 subtest**，裁决为：
  - **2 case / 2 subtest**：`defer-message-lifecycle`
  - **1 case / 4 subtest**：`defer-worker-unregister-controller`
  - **1 case / 1 subtest**：`defer-controller-skipwaiting`
  - **1 case / 2 subtest**：`gated-fetch-update`
  - **1 case / 3 subtest**：`gated-dynamic-update`
  - **1 case / 6 subtest**：`skip-mixed-cross-origin-clients`
- `registration-end-to-end` 和 `registration-events` 直接验证 M1 真生命周期，但以
  `MessageChannel` 从 worker 回传结果；它们是 worker result channel 落地后的优先用例。
- message-channel 信号不等于消息 API 本身是唯一被测行为。本批其余文件分别绑定
  controller/skipWaiting、fetch、update、跨源 client 或动态 server。
- `update-not-allowed` 的 Python handler 每次向 worker 响应注入随机值以强制字节变化，
  不能把 Python 源码当静态 JavaScript 资产返回。

TSV SHA-256：
`bd5b3fa1a595b1a9dc3abfd30e51395f61f838dc6c26877052a695bd35bfd19e`。

## 1. 审计方法

1. 从初始 inventory 筛出 `disposition=review`、`milestone=M1`、依赖信号含
   `message-channel` 的文件。
2. 扣除 next-wave 和 iframe 批次已裁决路径，得到 7 个未审计 case。
3. 读取 7 个页面的全部断言，并追踪 MessagePort 对端、iframe fixture、worker、
   `importScripts()` 和 Python handler。
4. 区分消息作为结果通道、消息 API 语义、controller/跨 client、fetch 和 update 依赖。
5. 对 7 个 case 与 10 个补充传递资源计算固定 revision Git blob SHA。

整个文件是验收单位。上游将基础 `getRegistrations()` 断言与受控/跨源 iframe 断言放在
同一文件，因此不拆出前四项包装成自建分母。

补充传递资源的固定 revision Git blob SHA：

| Resource | Blob SHA |
|----------|----------|
| `resources/frame-for-getregistrations.html` | `7fc35f18914c1345e0f5ccab93305938180fe9eb` |
| `resources/end-to-end-worker.js` | `d45a50556a93c7e38d94ac05b90d2d733c04896a` |
| `resources/events-worker.js` | `80a2188677ba5a9cc728158dbf10fd973bbc0f27` |
| `ServiceWorkerGlobalScope/resources/registration-attribute-worker.js` | `315f43759324e46209db5c21da07571a6545b440` |
| `ServiceWorkerGlobalScope/resources/registration-attribute-newer-worker.js` | `44f3e2e8e9bdbe8e756ecd94c758bc752a5fc1a3` |
| `ServiceWorkerGlobalScope/resources/unregister-worker.js` | `6da397dd15268bec568fef1b5f8112d862e88970` |
| `ServiceWorkerGlobalScope/resources/unregister-controlling-worker.html` | `e69de29bb2d1d6434b8b29ae775ad8c2e48c5391` |
| `resources/skip-waiting-installed-worker.js` | `6f7008bddcdfa044dbedd2e8f2aceb89ec980a71` |
| `resources/update-during-installation-worker.py` | `3e15926185642a240ffa0223a2836a12fa45bf6d` |
| `resources/update-during-installation-worker.js` | `f1997bd824e8e165cc119a52082d742bd63f4e4d` |

## 2. M1 消息结果通道

### 生命周期核心（2 case / 2 subtest）

| Case | 被测行为 | 消息用途 |
|------|----------|----------|
| `registration-end-to-end.https.html` | installing→installed→activating→activated→redundant | 双向确认 worker 已可处理消息 |
| `registration-events.https.html` | worker 实际观察 install→activate | 回传 worker 内事件序列 |

两案均不依赖 iframe、fetch 或动态 server。它们不能进入当前静态 core，因为现有 SW runner
还没有独立 worker 执行环境和 MessagePort 结果通道；M1 bridge 落地后应优先纳入。

### Worker 内注销（1 case / 4 subtest）

`ServiceWorkerGlobalScope/unregister.https.html` 的前三项分别在脚本求值、install 和 activate
中调用 `self.registration.unregister()`；第四项经受控 iframe 的 controller 发消息注销，
并比较既有与新 client。整个文件须等 worker registration 投影、单 iframe controller 和
消息通道完成后纳入。

### 等待 worker skipWaiting（1 case / 1 subtest）

`skip-waiting-installed.https.html` 先让旧 worker 控制 iframe，再向 waiting worker 发消息
触发 `skipWaiting()`，同时验证 Promise、activate 与 `controllerchange` 的相对时序。该案
属于长期 controller/skipWaiting 面。

## 3. 门控与 skip

### Fetch/update 门控（1 case / 2 subtest）

`ServiceWorkerGlobalScope/registration-attribute.https.html`：

- 首项由受控 iframe 发起 fetch，worker 用响应正文回传 evaluate/updatefound/install/
  activate/fetch 状态序列；
- 次项注册新脚本，验证 update job 期间新旧 worker 所见的 `self.registration` 投影。

该文件同时依赖 M2 fetch interception 和 update job，不能仅凭 MessageChannel 支持运行。

### 动态 update 门控（1 case / 3 subtest）

`update-not-allowed.https.html` 使用 `update-during-installation-worker.py`。handler 每次把
`random.random()` 写入返回脚本，再导入静态 worker，以保证 `registration.update()` 总能
发现新版本。三项分别验证 client、installing worker、active worker 发起 update 的合法性；
必须标记 `Unsupported(dynamic-server)`，不计入静态可执行分母。

### 跨源/client 混合 skip（1 case / 6 subtest）

`getregistrations.https.html` 含四个基础注册表断言、一个受控 iframe 注销断言，以及一个
跨 origin iframe 自行注册后验证同源隔离的断言。最后一项还通过 MessageChannel 完成跨源
清理。当前 support envelope 排除跨源和多 client 控制，整个上游文件记
`Unsupported(cross-origin-clients)`。

## 4. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 7 case / 18 subtest | 页面 test 声明 | worker/fixture 无额外 testharness test | 一致 | 高 |
| 2 案可驱动 M1 消息结果通道 | 页面生命周期断言 | worker 仅回传确认/事件序列 | 一致 | 高 |
| registration-attribute 不能归消息 core | iframe fetch 响应断言 | 新旧 worker update 投影 | 一致 | 高 |
| update-not-allowed 必须动态执行 | Python 每次注入随机值 | 三项依赖发现新 worker | 一致 | 高 |
| getRegistrations 整案超出当前范围 | 受控与跨源 iframe 同文件 | support envelope 排除跨源/多 client | 一致 | 高 |

## 5. 后续输入

1. M1 worker result channel 落地后，优先纳入 2 个 message-lifecycle case。
2. worker registration 投影与单 iframe controller 完成后纳入 SW 内 unregister case。
3. controller/skipWaiting 接线后纳入 waiting-worker case。
4. fetch interception 和 update job 完成后纳入 registration-attribute。
5. dynamic update 与跨源/client 混合文件保持明确 gated/skip。
6. 剩余未经人工裁决的 inventory `review` 文件由逻辑 127 降为 **120**。

## 6. 质量审查

- [x] 7/7 case 正文已读，manifest blob SHA 匹配。
- [x] 10/10 补充 worker/fixture/handler 资源 blob SHA 已计算。
- [x] 18 个 subtest 无 worker-side test 遗漏（2 + 4 + 1 + 2 + 3 + 6）。
- [x] 消息结果通道与消息语义、fetch/update/controller 依赖已分开。
- [x] Python handler 未被误判为静态 JavaScript 资产。
- [x] 未修改 runtime 源码或既有 inventory 初筛记录。
