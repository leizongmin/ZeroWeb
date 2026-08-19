# M1 Tier A WPT 固定资产与验收合约

**日期**：2026-08-19
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：M0 implementation input（RFC 批准后执行）
**资产清单**：[Tier A assets](2026-08-19-m1-tier-a-assets.tsv)
**Subtest 清单**：[Tier A subtests](2026-08-19-m1-tier-a-subtests.tsv)

## 来源分级

| 来源 | 覆盖 | 类型 | 置信度 |
|------|------|------|--------|
| WPT manifest + 固定 revision 正文 | 文件、SHA、subtest 名称 | 一手事实 | 高 |
| M1 candidate closure | 资源角色与 A/B/C 裁决 | 前期调研 | 高 |
| phase/requirement group | 实施与验收排序 | 作者综合 | 待运行验证 |

## 0. 合约摘要

- Tier A 固定为 **8 个 case / 28 个 subtest / 18 个唯一资产**。
- 18 个资产总计 **235,111 bytes**，包含 8 个 case、6 个页面脚本、3 个 worker 脚本和
  1 个 HTML fixture。
- 18/18 资产按 Git blob 算法与 WPT manifest SHA 匹配。
- 当前 ZeroWeb 没有 SW testharness runner，因此这些 case 仍是 `NotRun(no-runner)`；
  该状态不是 Pass，也不是产品失败基线。
- M1 首个真实 driving subtest 是 `activation occurs after registration`。
- Tier A 完成条件是 28/28 Pass、0 Timeout、0 Unsupported，且重复运行不残留 registration。

机器清单 SHA-256：

- assets：`c9b8089dc425873e3249d0e834176139c054f3e33845ba6c4080521f23fa6bc0`
- subtests：`23b3073c0471857b6b61167a438e0f8bf803c3c2d08846ea409ed218af27d302`

## 1. 固定资产

| 角色 | 唯一文件 | 说明 |
|------|---------:|------|
| case | 8 | `.https.html` testharness 页面 |
| page-script | 6 | testharness/report、SW helper、3 个 registration helper |
| worker-script | 3 | empty worker、empty script、registration worker |
| fixture | 1 | `resources/blank.html` |
| **合计** | **18** | 无 Python handler、无故意 404 |

`test-helpers.sub.js` 是唯一 `.sub.js` 资产。Tier A 使用的 helper 路径不读取其中 host/port
模板；fetch script 仍必须保留原文件并记录其模板属性，不能静默改写上游正文。

批准后的 pinned fetch 流程必须：

1. 只下载 assets TSV 中的 18 个路径；
2. 固定 revision，不跟随 `master`；
3. 下载后计算 Git blob SHA 并逐项匹配；
4. testharness 用例记入 `imported-testharness.txt`，不误用 reftest `make import-wpt`；
5. 任一对象缺失/SHA 不符时 fail closed，不运行缩水分母。

> **来源说明（第 1 章）**
>
> - **一手事实**：assets TSV、manifest 对象与固定 revision 字节。
> - **作者综合**：导入门禁顺序。

## 2. 驱动阶段

| 阶段 | Case | Subtest | 主要行为 |
|------|-----:|--------:|----------|
| 1-core-activation | 1 | 1 | register 后真实执行脚本并最终 activated |
| 2-state-projection | 2 | 2 | installed 先于 activating；installing/waiting/active 投影 |
| 3-basic-scope | 2 | 6 | 默认 scope、fragment、script directory、null scope 拒绝 |
| 4-url-validation | 2 | 18 | 编码分隔符、scheme、多字节、`.`/`..`、连续斜杠 |
| 5-error-shape | 1 | 1 | register rejection 是 DOMException 且也是 Error |
| **合计** | **8** | **28** | |

阶段 1 只要求最小空 worker；阶段 2 才验证状态时序和对象槽；阶段 3/4 扩展 URL 与安全校验；
阶段 5 固定异常对象形状。后续阶段失败不能用来掩盖前一阶段回归。

### Requirement group 分布

| Requirement group | Subtest |
|-------------------|--------:|
| lifecycle-activation | 1 |
| lifecycle-ordering | 1 |
| registration-object-state | 1 |
| default-scope | 2 |
| scope-security | 1 |
| scope-normalization | 7 |
| scope-validation | 3 |
| scope-scheme | 3 |
| script-url-validation | 4 |
| script-url-scheme | 2 |
| script-url-normalization | 2 |
| exception-shape | 1 |
| **合计** | **28** |

## 3. 行为验收

### Phase 1：最小真实激活

```text
场景: activation occurs after registration
  假设页面位于 secure origin，empty-worker.js 可按正确 MIME 获取
  当 register() 完成并监听 worker statechange
  那么脚本在独立 SW runtime 执行，状态最终为 activated
  验证: activation-after-registration.https.html
```

不得用页面 shim timer 直接置 `activated` 满足该场景；runner 必须证明 worker 脚本确实被抓取
和求值。

### Phase 2：状态时序与对象投影

- install 完成后的 `installed` statechange 必须早于 `activating`。
- register Promise resolve 时 newest worker 可从 `installing/waiting/active` 之一取得。
- `registration.installing/waiting/active` 随 manager 状态迁移，不由独立 JS 私有数组维护。

### Phase 3：基础 scope

- scope 缺省或显式 `undefined` 时取 script URL 所在目录。
- scope fragment 被移除。
- scope 等于 script directory 可注册。
- `null` 转为 `"null"` URL 后超出脚本允许路径，必须以 `SecurityError` reject。

### Phase 4：URL 与 scheme

- script/scope 中 URL-encoded slash/backslash 必须 reject。
- script 的 data URL 和 scope 的 data/ftp/filesystem URL 必须 reject。
- 多字节、`.`、`..` 和连续斜杠按 URL 标准规范化；不得用裸字符串前缀代替 URL parser。

### Phase 5：异常形状

- cross-origin HTTP script URL 必须在网络请求前 reject。
- rejection reason 同时满足 `instanceof DOMException` 与 `instanceof Error`。

> **来源说明（第 2-3 章）**
>
> - **一手事实**：28 个 WPT subtest 原始名称与断言正文。
> - **作者综合**：五阶段编排和 BDD 摘要。

## 4. Runner 与完成门禁

Tier A runner 必须提供：

1. `https://wpt.test/` secure origin 与固定 document URL；
2. assets manifest 到本地字节的确定性映射；
3. 每个 case 前后精确 unregister，失败清理也执行；
4. worker/lifecycle task 与 Promise microtask 的 drain；
5. case timeout 与 subtest 结果分离，顶层异常不能吞掉已有结果；
6. 文件、subtest、Timeout、Unsupported 四种计数；
7. 同一批次连续运行两次，第二次不得受第一次 registration 污染。

### Tier A 完成判据

- [ ] 8/8 case 被 runner 发现，不能静默少文件。
- [ ] 28/28 subtest Pass。
- [ ] 0 Fail / 0 Timeout / 0 Unsupported。
- [ ] 每个 case 后 registration 数回到 0。
- [ ] 连续两轮结果逐 case/subtest 一致。
- [ ] V8 与 QuickJS 生命周期核心单测一致；WPT runner 至少在 production 默认引擎通过。
- [ ] driving case 与资产记入 testharness 账本。

Tier A 全绿只证明 M1 的静态注册/生命周期/URL 基础，不证明 fetch interception、CacheStorage、
message、claim/skipWaiting 或 update 已完成。

## 5. 证据矩阵

| 结论 | 来源 1 | 来源 2 | 一致性 | 置信度 |
|------|--------|--------|--------|--------|
| Tier A 有 8 个 case | candidate closure TSV | assets manifest 中 8 个 case role | 一致 | 高 |
| Tier A 有 28 个 subtest | 平衡括号解析 testharness 调用 | subtests TSV 28 唯一 case/ordinal | 一致 | 高 |
| 固定资产为 18 个 | 8 case 的闭包 union | assets TSV 18 唯一路径 | 一致 | 高 |
| 资产字节可信 | manifest Git SHA | 18/18 本地 blob 重算 | 一致 | 高 |
| Phase 1 是最小 driving case | case 仅 empty worker/blank/helper | WPT 断言只等待 activated | 一致 | 高 |
| Tier A 不覆盖 fetch/message | 28 个 requirement group | candidate closure 无 fetch worker | 一致 | 高 |

## 6. 质量审查

- [x] 8/8 Tier A case 已纳入资产与 subtest 清单。
- [x] 28 个 subtest 保留上游原始名称。
- [x] 18/18 资产有 manifest type、角色、大小和 Git blob SHA。
- [x] 五阶段计数可反算为 8 case / 28 subtest。
- [x] 已区分 NotRun、Fail、Timeout、Unsupported、Pass。
- [x] 未把 Tier A 扩张为完整 Service Worker Done Criteria。
- [x] 未修改源码、WPT 数据或共享账本。
