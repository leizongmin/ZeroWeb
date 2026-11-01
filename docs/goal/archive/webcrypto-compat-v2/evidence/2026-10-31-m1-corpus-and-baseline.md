# M1 语料补全 + 扩面基线（2026-10-31，DC-1 / DC-3 / W4 定性）

## 关键更正：pin 语料本已全量（W1/W2 前提系 coverage 口径假象）

- 一期 corpus（pin `315976933870b34d6ea30e3f6643403edae678ba`）在 M1→M6 过程中
  已按 git trees 深挖补到 **pin 全量**：上游 202 blob，本次 `fetch-wpt-webcrypto`
  （goals/26，deep + any.js 通道）复拉后 198 文件在库、0 次失败；4 个未拉 blob
  为非测试元数据（META.yml / README.md / WEB_FEATURES.yml / tools/generate.py），
  扩展名分流正确排除。
- test 入口 126（50 `.tentative.`）；runner 记账排除 idlharness ×2 后执行 123 案。
  「derive_bits_keys 32/38 未拉」的缺口行来自 coverage 快照的 universe 口径
  （38 = 转正重排后目录 + window/worker 变体路径），**pin 目录事实是 11 案全在库、
  全部执行**。W1/W2 的实质结论：pin 语料无未拉面；universe 相对 pin 的差集归
  [DC-2 转正甄别](2026-10-31-dc2-promotion-screening.md) 与
  [DC-4 对账](2026-10-31-dc4-coverage-attribution.md)。

## 基线运行（DC-3）

- 通道：`make` 构建依赖（target-disk-guard / test-guard / zero-wpt-runner-release）
  + `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 3600 --
  ./target/release/zero-wpt-runner testharness-webcrypto --json`（Makefile
  `testharness-webcrypto` 配方同参数；--json 由 runner 位置参数直传）。
- 连跑两轮（第二轮为复跑验证）：**逐字节一致**（cmp 全等），每轮 wall ~100s。
- 总量：**123 案 / 33,109 子测试 = Pass 25,788（77.89%）+ Fail 7,320 + Timeout 1**；
  全绿案 73。证据：[2026-10-31-m1-baseline-full.json.gz](2026-10-31-m1-baseline-full.json.gz)
  （runner `--json` 原始输出 gzip）。
- **零回归**：与一期终态锚 `archive/webcrypto-compat/evidence/2026-10-17-m6-residual-full.json.gz`
  逐 (case, subtest) 对照——33,109 对全等，**状态变化 0 例（P→非P 0 例）**，
  33,109 分母零回归底线保持。

## 分域分类通过率

| 域 | 案 | 子测试 | P | F | T | 全绿案 | 通过率 |
|---|---:|---:|---:|---:|---:|---:|---:|
| derive_bits_keys | 11 | 12,448 | 12,434 | 14 | 0 | 8 | 99.9% |
| generateKey | 39 | 9,047 | 5,449 | 3,598 | 0 | 25 | 60.2% |
| import_export | 21 | 7,668 | 5,319 | 2,349 | 0 | 10 | 69.4% |
| encrypt_decrypt | 7 | 1,753 | 1,449 | 304 | 0 | 5 | 82.7% |
| sign_verify | 9 | 813 | 635 | 178 | 0 | 6 | 78.1% |
| digest | 5 | 535 | 116 | 419 | 0 | 1 | 21.7% |
| encap_decap | 2 | 129 | 0 | 129 | 0 | 0 | 0.0% |
| serialization | 19 | 63 | 36 | 27 | 0 | 12 | 57.1% |
| wrapKey_unwrapKey | 1 | 299 | 299 | 0 | 0 | 1 | 100% |
| (top) | 9 | 354 | 51 | 302 | 1 | 5 | 14.4% |
| **合计** | **123** | **33,109** | **25,788** | **7,320** | **1** | **73** | **77.89%** |

失败归因（tentative / 非 tentative 分列）：

- `.tentative.` 案 49（idlharness.tentative 记账排除后）共 7,340 子测试 =
  P 20 + **F 7,320**——全部失败落在 tentative 草案域，与一期 M6 收口一致。
- 非 tentative Fail = **0**。
- Timeout 1 见下节定性。

## W4 定性：algorithm-discards-context.https.window.js 1T

- 定性：**runner/shim DOM 域边界（iframe 跨帧依赖），非 WebCrypto 语义域，
  非语料边界**。
- 事实：该案 13 个 subtest 全部经由 `runInChild`——`document.createElement("iframe")`
  挂载后向 `child.contentDocument` 注入 script，在子帧上下文触发
  `crypto.subtle.*`，断言算法归一化读取被标记丢弃的上下文属性；完成信号依赖
  子帧 getter 回调 `top.closeChild(testId)`（跨帧 `top` 引用）由父文档
  `removeChild` 收尾并调用 `t.done()`。runner window wrapper 为单窗形态，
  跨帧注入/回调链不闭合 → harness completion callback 未触发
  （runner Timeout 消息实证：`pending:13, phase:4`，自一期 M1 起每轮恒定，
  本轮两连跑逐字节同消息）。
- 归属：DOM/facade 域挂账（与一期口径一致，非本 goal 修复面——一期 M6 即挂账）。
- 对 coverage 分子的影响：该案 resolved（universe 13 子测试 den 入账），
  P=0、T=1——coverage「已覆盖的失败子测试」462 中的 13 即此案（余 449 为
  三家窗口变体与 ZW wrapper 子测试形态差，见 DC-4 对账）。

## 门禁与状态

- 本轮 runner 引擎/shim **零改动**（工作区 diff 仅 docs；曾试拟
  corpus_case_timeout 接线并完成双跑验证——两轮与锚逐字节一致，但机制归因
  证伪后按精准修改原则整体回退，未入库）。
- DC-3 门禁含义：`make test` / `make reftest` 非 M1 触发面（零代码变更），
  不重复空跑；后续轮次若有代码变更，按质量门禁执行并在本轮 evidence 记录。
