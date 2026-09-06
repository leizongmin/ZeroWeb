# M1 候选 WPT 资源闭包与首批分母裁决

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 evidence（零源码改动）
**逐案数据**：[candidate resource closure TSV](2026-08-19-m0-m1-candidate-resource-closure.tsv)
**Tier A 合约**：[fixed assets and subtests](2026-08-19-m1-tier-a-baseline-contract.md)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest version 9 | 文件路径与 Git blob SHA | 一手事实 | 高 |
| 固定 revision CDN 正文 | 12 个 case + 38 个依赖对象 | 一手事实 | 高 |
| ZeroWeb runner/goal | 当前 fixture 与里程碑边界 | 一手事实 | 高 |
| tier/decision/blocker | 首批实施排序 | 作者综合 | 待 M1 运行验证 |

## 0. 执行摘要

- 原先 12 个 M1 候选不能整体作为首批分母。
- **Tier A（8 个）**仅依赖静态脚本/fixture，可保留为 M1 首批。
- **Tier B（3 个）**依赖 worker-side testharness 或 ErrorEvent/嵌套事件派发，应在基本
  生命周期跑通后纳入。
- **Tier C（1 个）**依赖 3 个 Python response handler 和非法 chunked encoding，当前静态
  fixture 无法执行，应移出首批而不是计 Timeout/Fail。
- 12 个 case 的已知闭包共有 39 个唯一文件；全部对象按 Git blob 算法重新计算 SHA-1，
  与 manifest **39/39 匹配**。
- 首个 driving case 仍为 `activation-after-registration.https.html`。

## 1. 审计方法

1. 从逐文件 inventory 读取 `first_batch=yes` 的 12 个 testharness 源。
2. 解析页面 `<script src>`，区分 common harness 与 test-specific helper。
3. 在页面及 test-specific helper 中解析静态 URL 字面量；URL 按 document URL 解析。
4. 对注册的 worker 脚本解析 `importScripts()`；URL 按 worker script URL 解析。
5. 将引用与完整 WPT manifest 对齐；记录动态 handler、fixture、故意缺失资源。
6. 下载闭包对象并按 `sha1("blob <len>\\0" + bytes)` 与 manifest SHA 比较。

静态分析不会把 scope 字符串当文件，也不会把编码斜杠、data URL 或故意不存在的资源伪造为
可下载文件。外链 helper 内按运行时分支才会触发的依赖仍需 M1 runner 实测，因此
`keep-first` 代表“资源层无阻塞”，不代表测试已通过。

> **来源说明（第 1 章）**
>
> - **一手事实**：固定 revision case/helper/worker 正文与 manifest SHA。
> - **作者综合**：URL 解析与 tier 裁决规则。

## 2. 逐案裁决

| Case | Tier | 闭包文件 | 裁决 | 原因 |
|------|------|---------:|------|------|
| `activate-event-after-install-state-change.https.html` | A | 6 | keep-first | empty worker + blank fixture，验证 install statechange 与 activate 顺序 |
| `activation-after-registration.https.html` | A | 6 | keep-first | empty worker + blank fixture，最小 register→activated 链 |
| `register-default-scope.https.html` | A | 7 | keep-first | 静态 empty worker/empty script；null scope 必须在 fetch 前拒绝 |
| `registration-basic.https.html` | A | 5 | keep-first | 空 registration worker，scope/fragment 基础语义 |
| `registration-scope.https.html` | A | 6 | keep-first | 静态 empty worker，URL/scope 规范化与拒绝 |
| `registration-script-url.https.html` | A | 6 | keep-first | 静态 empty worker；编码斜杠/data URL 在 fetch 前拒绝 |
| `registration-service-worker-attributes.https.html` | A | 5 | keep-first | empty worker，installing/waiting/active 对象投影 |
| `rejections.https.html` | A | 3 | keep-first | 跨 origin HTTP URL 必须在网络前 reject 为 DOMException |
| `install-event-type.https.html` | B | 7 | defer-after-core | worker `importScripts(worker-testharness.js)`，需要 SW→harness 结果通道 |
| `onactivate-script-error.https.html` | B | 10 | defer-after-core | 5 个 worker，依赖 ErrorEvent、onerror、preventDefault、嵌套 MessageEvent |
| `oninstall-script-error.https.html` | B | 11 | defer-after-core | 6 个 worker，另含 waitUntil Promise 内 throw |
| `registration-script.https.html` | C | 8 | remove-first | 3 个 Python handler 生成 parse/runtime/chunking 响应，且含故意 404 |

### Tier A 边界

Tier A 的 8 个 case 仍共同依赖 `/resources/testharness.js`、
`/resources/testharnessreport.js`；其中 7 个还加载 `test-helpers.sub.js`。现有 runner 已能
内联通用 testharness，但 M1 runner 必须增加：

- registration 清理隔离，避免 case 间 scope 污染；
- service worker statechange 的事件循环 drain；
- worker 脚本 URL 本地映射与正确 MIME；
- register 的 secure-context/same-origin/path 校验；
- `DOMException`、`ServiceWorker`、`ServiceWorkerRegistration` 对象投影。

`register-default-scope` 中出现 `resources/simple-fetch-worker.js` 字面量，但只用于计算
expected scope，实际注册的是 `resources/empty.js`；上游 manifest 也没有前者。该字面量不是
资源缺失，不应导入占位文件。

### Tier B 边界

`install-event-type-worker.js` 导入 `worker-testharness.js`，后者再导入
`/resources/testharness.js`。它虽不调用 Cache helper，但 testharness 需要把 SW 内断言结果
发送回页面，因此不能用“脚本执行成功”替代真实 subtest 结果。

install/activate error 两案不只验证生命周期失败，还验证多 error listener、`onerror` 返回
true、`preventDefault()` 和嵌套 MessageEvent 的未处理异常传播。把这些断言塞进 M1 最小
runtime 会扩大首切片；应在 Tier A 全绿后单独驱动。

### Tier C 边界

`registration-script.https.html` 依赖：

- `invalid-chunked-encoding.py`
- `invalid-chunked-encoding-with-flush.py`
- `malformed-worker.py` 的多个 query 分支
- 故意不存在的 `no-such-worker.js`

这些 handler 验证 HTTP framing、动态生成 parse/runtime/module 错误和 import 失败。当前
`wpt_data_fetch_handler` 只能静态读文件，不能产生非法 chunked response。该 case 必须
`Unsupported(dynamic-server)`，直到有等价 fixture adapter；不能把 Python 源码当 worker
脚本返回。

> **来源说明（第 2 章）**
>
> - **一手事实**：12 个 case、test-specific helper、worker 和 Python handler 正文。
> - **作者综合**：A/B/C tier 与 M1 顺序。

## 3. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 | 处理 |
|------|--------|--------|--------|--------|------|
| 8 个 case 资源层可静态承载 | case/helper 字面量闭包 | manifest 对象 + 39/39 blob SHA | 一致 | 高 | Tier A |
| install-event-type 需结果通道 | worker 导入 worker-testharness | worker-testharness 再导入 testharness | 一致 | 高 | Tier B |
| error 两案超出最小事件面 | 页面各注册 5/6 个 worker | worker 使用 error/onerror/MessageEvent/waitUntil | 一致 | 高 | Tier B |
| registration-script 需动态 server | helper 指向 3 个 `.py` | handler 正文生成非法/动态响应 | 一致 | 高 | Tier C |
| no-such-worker 不应补资源 | helper 明确期望注册失败 | manifest 无该对象 | 一致 | 高 | 保持 intentional missing |
| rejections 不需真实外网 | case 使用 cross-origin HTTP URL | 预期是 register Promise 本地拒绝 | 一致 | 高 | Tier A |

## 4. 对既有 evidence 的校准

> **勘误说明**：上一版
> `2026-08-19-m0-wpt-executable-surface.md` 将 12 个文件统一称为“M1 首批人工复核候选”。
> 完成传递资源闭包后，确认其中只有 8 个可作为静态首批；3 个属于 M1 高阶事件面，1 个依赖
> 动态 WPT server。该处“候选”本来不是通过率承诺，但实施队列现按 8/3/1 校准。

逐案 TSV 共 12 行，SHA-256：
`9ae8b6c6c3f54364ec0f35c664b0241366e2d59671b718ca2859f01437bea18a`。

## 5. 实施输入

RFC 批准后的 M1 WPT 顺序：

1. 按 Tier A 合约导入 8 个 case / 18 个固定资产，建立 28-subtest 真实 red baseline。
2. 以 `activation-after-registration.https.html` 驱动 manager/runtime 最小全链。
3. 收敛其余 Tier A scope/object/state 语义。
4. 增加 worker test result channel 后纳入 `install-event-type`。
5. 单独实现 error event dispatch，再纳入 oninstall/onactivate error 两案。
6. `registration-script` 保持 `Unsupported(dynamic-server)`，不计入当前可执行分母。

## 6. 质量审查

- [x] 12/12 case 已读正文。
- [x] 39/39 已知闭包对象与 manifest Git blob SHA 匹配。
- [x] common page harness、test-specific helper、worker import 和动态 handler 已分层。
- [x] 故意不存在和只作 URL 计算的字面量未伪造资源。
- [x] keep-first/defer/remove 均有可检查的具体原因。
- [x] 未修改源码、WPT 数据或共享账本。
