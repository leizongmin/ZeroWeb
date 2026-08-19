# M1 第二批生命周期 WPT 裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零 runtime 源码改动）
**逐案裁决**：[next-wave review TSV](2026-08-19-m1-next-wave-review.tsv)
**资产清单**：[next-wave assets TSV](2026-08-19-m1-next-wave-assets.tsv)
**Subtest 清单**：[next-wave subtests TSV](2026-08-19-m1-next-wave-subtests.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 14 case、helper、worker、handler、SHA | 一手事实 | 高 |
| M0 inventory | `M1 + review + direct_dependency_signals=none` 初筛 | 前期调研 | 高 |
| decision/requirement group | M1 实施排序 | 作者综合 | 待 runtime 验证 |

## 0. 执行摘要

- 从 inventory 的 152 个 `review` 中，先审计最可能可执行的 14 个
  `M1 + direct_dependency_signals=none` 文件。
- 14 案共 **78 个 subtest**，裁决为：
  - **3 case / 4 subtest**：`next-wave-core`
  - **5 case / 8 subtest**：`defer-advanced`
  - **5 case / 65 subtest**：`gated-dynamic-server`
  - **1 case / 1 subtest**：`defer-update`
- 第二批 core 是 `state.https.html`、`synced-state.https.html`、`unregister.https.html`。
- 第二批共有 **7 个唯一资产 / 217,890 bytes**；其中 4 个 support/worker 已在 Tier A
  corpus，只需新增 3 个 case HTML。
- 初筛的 `none` 只表示页面正文没有命中浅层关键词，**不表示无隐藏依赖**：11/14 案在
  helper/worker 中发现动态 handler、module graph、message/error、job scheduling 或 watchdog。

机器清单 SHA-256：

- review：`18a46675ff9752d83b20fef8a4db08411fd9bf02d32759b52cd3efbb14b2186a`
- assets：`7a9a4e1f511b6999e10ec9bbe0f856f7fe8a9f4e49860cea88da9aa39a2bbcde`
- subtests：`60fcced049cc96d199f665c21ad0e02e037062f6d871c142bb9c37992888c1ac`

> **来源说明（第 0 章）**
>
> - **一手事实**：固定 revision 页面/helper/worker 正文和 manifest。
> - **作者综合**：四类裁决及实施顺序。

## 1. 审计方法

1. 从逐文件 inventory 筛出 `disposition=review`、`milestone=M1`、
   `direct_dependency_signals=none` 的 14 个源文件。
2. 读取每个 case 的外链 helper、注册脚本、`importScripts()` 和静态 module import。
3. 对运行时生成的测试（MIME 数组、动态 cleanup test）展开实际 subtest 数。
4. 识别 Python response handler、redirect、故意无限脚本、message/error 和 module graph。
5. 对 case 和 next-wave 资产按 Git blob 算法重算 SHA，与 manifest 逐项比较。

该方法只把整个上游 case 作为验收单位，不把一个含动态 handler 的文件拆成“静态子集”来抬高
可执行率。URL 在 worker/helper 中动态拼接时人工复核，不依赖页面浅层关键词。

## 2. 第二批 core

| Case | Subtest | 新增资产 | 验证行为 |
|------|--------:|---------:|----------|
| `state.https.html` | 1 | 1 | installing→installed→activating→activated→redundant 与 statechange |
| `synced-state.https.html` | 1 | 1 | 同一 worker entity 的多个 JS 对象状态同步 |
| `unregister.https.html` | 2 | 1 | 首次 unregister=true，重复 unregister=false |
| **合计** | **4** | **3** | |

三案复用 Tier A 的：

- `/resources/testharness.js`
- `/resources/testharnessreport.js`
- `resources/test-helpers.sub.js`
- `resources/empty-worker.js`

因此第二批不是新执行环境方向，只是 Tier A 激活链跑通后的状态机/对象投影/注销扩展。M1
runner 应按 `Tier A 28/28 → next-wave 4/4` 顺序验收，不能用第二批失败阻塞首个 driving case。

> **来源说明（第 1-2 章）**
>
> - **一手事实**：14 个 case 及其传递依赖正文；next-wave 7/7 manifest 对象。
> - **作者综合**：next-wave 的先后关系。

## 3. 延后与门控

### Advanced runtime（5 case / 8 subtest）

| Case | 原因 |
|------|------|
| `ServiceWorkerGlobalScope/close.https.html` | SW 内 worker-testharness 结果通道 |
| `ServiceWorkerGlobalScope/service-worker-error-event.https.html` | message 抛错、ErrorEvent 字段、client source 回传 |
| `register-wait-forever-in-install-worker.https.html` | 永不 resolve 的 waitUntil、同/异 scope job 调度 |
| `registration-schedule-job.https.html` | scriptURL/updateViaCache/type job 去重、module 与 message |
| `unregister-immediately-before-installed.https.html` | parse infinite loop、永不完成 install、Clear-Site-Data |

这些用例不是外部环境不可执行，而是要求超过最小生命周期 runtime 的能力。应在对应 typed
事件、message 或 watchdog 切片落地时逐案纳入，不应永久 skip。

### Dynamic server（5 case / 65 subtest）

| Case | 动态依赖 |
|------|----------|
| `import-scripts-updated-flag.https.html` | `import-scripts-echo.py` query 响应 |
| `registration-mime-types.https.html` | 两个 Python handler 生成 MIME 组合 |
| `registration-scope-module-static-import.https.html` | module static import + Python redirect |
| `registration-script-module.https.html` | parse/runtime/TLA/instantiation 与非法 chunked 响应 |
| `registration-security-error.https.html` | 7 个静态安全断言 + 1 个 Python redirect 断言 |

这些 case 必须等 runner 有等价 response adapter；不能返回 Python 源码或静默跳过文件内的动态
subtest。`registration-security-error` 即使 7/8 断言可静态执行，也不拆分上游 case 分母。

### Update（1 case / 1 subtest）

`update-result.https.html` 的静态资源很轻，但要求真实 update job 与脚本重抓取。它归 update
语义切片，而非紧跟激活链的第二批 core。

## 4. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| next-wave 为 3 case / 4 subtest | case 断言正文 | subtests TSV | 一致 | 高 |
| next-wave 仅需 3 个新文件 | 7-file closure | Tier A assets 交集为 4 | 一致 | 高 |
| `none` 信号存在 11/14 假阴性 | 页面 inventory | helper/worker 传递审计 | 一致 | 高 |
| dynamic-server 共 65 subtest | helper 循环/测试声明 | Python/redirect 资源正文 | 一致 | 高 |
| update-result 不属于 core | case 调用 `registration.update()` | goal 将 update 语义列入长期面 | 一致 | 高 |

## 5. 后续输入

1. RFC 批准前可将 3 个 next-wave case 资产化到独立 corpus。
2. RFC 批准后先跑 Tier A 28/28，再启用 next-wave 4/4。
3. error/message、job/module、watchdog 能力各自落地时消费 `defer-advanced`。
4. runner response adapter 落地前，5 个 dynamic-server case 保持 Unsupported。
5. 剩余未经人工裁决的 inventory `review` 文件由 152 降为逻辑上的 **138**；原 inventory
   保留初筛事实，当前裁决以本 evidence 为准。

## 6. 质量审查

- [x] 14/14 case 与 test-specific helper/worker 已读。
- [x] 14/14 case manifest blob SHA 匹配。
- [x] next-wave 7/7 资产 blob SHA 匹配。
- [x] 78 个 subtest 已展开并按裁决反算（4 + 8 + 65 + 1）。
- [x] 动态 server、advanced runtime、update 未混入 core 分母。
- [x] 未修改 runtime 源码或既有 inventory 初筛记录。
