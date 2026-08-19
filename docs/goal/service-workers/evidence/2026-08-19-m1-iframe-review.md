# M1 iframe 生命周期 WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[iframe review TSV](2026-08-19-m1-iframe-review.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 11 case、helper、worker、fixture、SHA | 一手事实 | 高 |
| Goal support envelope | 多客户端排除、controller/skipWaiting 长期范围 | 项目契约 | 高 |
| decision 分类 | runner/runtime 实施顺序 | 作者综合 | 待运行验证 |

## 0. 执行摘要

- 从 inventory 的逻辑剩余 138 个 `review` 中，审计 11 个
  `M1 + direct_dependency_signals=iframe` 文件。
- 11 案实际产生 **41 个 subtest**，裁决为：
  - **3 case / 11 subtest**：`defer-single-iframe`
  - **1 case / 2 subtest**：`defer-worker-harness`
  - **3 case / 13 subtest**：`defer-controller-ready`
  - **1 case / 3 subtest**：`gated-fetch-controller`
  - **3 case / 12 subtest**：`skip-mixed-multi-client-update`
- 本批没有可直接加入静态 core 的 case。单 iframe 用例仍在实现范围内，但须等 runner 的
  iframe browsing context 和每 Document `ServiceWorkerContainer` 投影。
- 3 个 mixed 文件把无 iframe 的 update 断言与多客户端 controller 断言放在同一上游 case；
  不拆分文件分母，按 support envelope 整体 skip。
- `unregister-controller.https.html` 的页面浅层信号只有 iframe，但传递依赖实际包含
  fetch interception/XHR，说明初筛信号不能直接作为可执行性裁决。

TSV SHA-256：
`87933946ec499517998a38a496ade2c622f5f4ec5bc8d20cf389153435d91130`。

> **来源说明（第 0 章）**
>
> - **一手事实**：固定 revision case/helper/worker/fixture 正文与 manifest。
> - **作者综合**：五类裁决和实施排序。

## 1. 审计方法

1. 从初始 inventory 筛出 `disposition=review`、`milestone=M1`、
   `direct_dependency_signals=iframe` 的 11 个源文件。
2. 读取页面全部断言和 iframe fixture；继续追踪 worker testharness、skipWaiting worker、
   install reject worker、intercept worker。
3. 将 worker 内通过 `fetch_tests_from_worker()` 回传的测试计入实际 subtest。
4. 区分单 iframe global/client 语义、controller/ready 长期语义、fetch 依赖和多客户端排除项。
5. 对 11 个 case 与 5 个补充传递资源重新计算 Git blob SHA。

整个文件是验收单位。即使文件前四个 subtest 不使用 iframe，只要同文件含排除的多客户端
断言，也不把前四项包装成自建测试来替代上游分母。

补充传递资源的固定 revision Git blob SHA：

| Resource | Blob SHA |
|----------|----------|
| `resources/register-iframe.html` | `f5a040e41d96a5e6995b92542ec98cc5c926eeba` |
| `resources/reject-install-worker.js` | `41f07fd5db81d6fc7a81fddd2c6bac5d5171974f` |
| `resources/simple-intercept-worker.js` | `f8b5f8c5cb77b153bc98703821e13f00d8702fd7` |
| `resources/skip-waiting-worker.js` | `3fc1d1e237aacfb86aa1e3e36287db9f45ebf756` |
| `resources/unregister-controller-page.html` | `18a95ee892b1c9ee7ca597cb2655e6c4504a7912` |

## 2. 可后续纳入

### 单 iframe harness（3 case / 11 subtest）

| Case | Subtest | 所需能力 |
|------|--------:|----------|
| `getregistration.https.html` | 6 | iframe Document 的 container、controlled client 注销后查询 |
| `installing.https.html` | 2 | 未控制 iframe 与跨 registration getter 对象身份 |
| `registration-iframe.https.html` | 3 | URL 按 iframe relevant global 解析 |

这些用例不要求 client 枚举或多 iframe 协调，属于 M1 runner/Document bridge 可扩展面。它们
不能在当前单一页面 shim 上运行，但不进入永久 skip list。

### Worker harness（1 case / 2 subtest）

`ServiceWorkerGlobalScope/isSecureContext.https.html` 包含页面 setup test 和 SW 内
`isSecureContext` test。后者经 worker-testharness 回传，因此等 Tier B 结果通道后纳入。

### Controller/ready（3 case / 13 subtest）

- `ready.https.window.js`：9 个 ready Promise identity、scope 匹配、registration 替换和
  controller 断言。
- 两个 skipWaiting case：各 1 个页面 test + 1 个 SW 内 test，验证受控/未受控 client 与
  `controllerchange`。

这些行为在目标长期范围内，不属于 M1 最小 register→activate 链；应在 controller、
skipWaiting 和每 Document container 接线后纳入。

> **来源说明（第 1-2 章）**
>
> - **一手事实**：11 个页面、5 个补充资源及 worker test 声明。
> - **作者综合**：M1 runner 与长期 controller 阶段的分界。

## 3. 门控与 skip

### Fetch/controller 门控（1 case / 3 subtest）

`unregister-controller.https.html` 使用 intercept worker 和 iframe XHR，验证：

- 注销不影响既有 controller，且既有 controller 继续拦截；
- 注销后新 navigation 不受控且走网络；
- registration 仍被旧 client 使用时也不能控制新 client。

该文件同时依赖 M2 fetch interception 和 controller 生命周期，必须等 fetch 主路径门控解除。

### 多客户端/update skip（3 case / 12 subtest）

| Case | Subtest | 排除原因 |
|------|--------:|----------|
| `register-same-scope-different-script-url.https.html` | 5 | update job + 多个旧/新 iframe controller |
| `unregister-then-register-new-script.https.html` | 3 | 旧 client 存活时新 registration/404/reject install |
| `unregister-then-register.https.html` | 4 | 旧新 registration 同时对应不同 iframe controller |

这些文件验证的 update/注销基础语义并非全部排除，但上游 case 将其与多客户端行为绑定。
当前 support envelope 明确排除多客户端枚举/逐 client 控制，因此记
`Unsupported(multi-client-update)`；若未来扩大 envelope，应恢复整个上游 case。

## 4. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| 本批为 11 case / 41 subtest | 页面 test 声明 | 3 个 worker-side test | 一致 | 高 |
| 3 个 case 仅需单 iframe bridge | 页面只创建一个 iframe | 无 clients API/跨 client 枚举 | 一致 | 高 |
| ready/skipWaiting 属长期范围 | 页面 controller/ready 断言 | goal controller/skipWaiting 覆盖项 | 一致 | 高 |
| unregister-controller 依赖 M2 | intercept worker respondWith | iframe XHR 断言响应内容 | 一致 | 高 |
| 3 个 mixed case 当前整体 skip | 多 iframe 同时存活 | support envelope 排除多客户端 | 一致 | 高 |

## 5. 后续输入

1. M1 runner 支持 iframe Document/container 后，优先纳入 3 个 single-iframe case。
2. worker result channel 落地后纳入 `isSecureContext`。
3. controller/ready/skipWaiting 切片落地后纳入 3 个 controller-ready case。
4. M2 fetch interception 落地后纳入 `unregister-controller`。
5. 3 个 mixed 文件保持明确 skip，不拆 subtest 伪造分母。
6. 剩余未经人工裁决的 inventory `review` 文件由逻辑 138 降为 **127**。

## 6. 质量审查

- [x] 11/11 case 正文已读，manifest blob SHA 匹配。
- [x] 5/5 补充 worker/fixture 资源 blob SHA 匹配。
- [x] 41 个 subtest 已包含 worker-side test（11 + 2 + 13 + 3 + 12）。
- [x] defer、M2 gated、support-envelope skip 已分开。
- [x] 初筛 iframe 信号未直接等同于多客户端 skip。
- [x] 未修改 runtime 源码或既有 inventory 初筛记录。
