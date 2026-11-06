---
date: 2026-11-06
modules: zero-engine
---

# 切片伪影与真实缺陷的判别：exec 是否在失败 variant 页内运行过才是决定性证据

## 问题描述

iol `:393`（`<ul><li>foo<ul><li>[bar]<li>baz</ul><li>quz</ul>` qState-after，?1001-last）与 iul `:428` div 变体（qIndeterm/qState-after，?1001-2000）三轮反复：sr1 轮隔离探针全绿判「切片伪影」除名；随后以「隔离探针过 + 语料终态败并存＝出口臂事后改位」为由推翻除名、重启归因（本文件旧版据此写「伪影定性错误」并立「确定性复红＝非伪影强信号」判据）；sr2 轮 variant 门控插桩终判：**伪影定性正确，推翻实为误判**——失败 variant 页内 execCommand 根本没有执行。

## 根因分析

editing 语料页用 `subset-tests.js` 切窗（?1-1000 / ?1001-last…）：`subsetTest` 对窗外断言**不创建 test 体**，而每案的 execCommand 包在 exec 断言的 test 体里——case 断言块（exec→compare→before×3→after×3，每命令 ~9 条）跨窗界时，后窗 variant **跳过 exec 但 after-reads 照跑**，读到未执行命令的原树。该伪影逐代确定性复现（窗界位置固定），与真实缺陷的「恒红」外部签名完全相同，「确定性复红＝非伪影」判据不成立。旧版「读点对齐」教训也不完整：隔离探针自己执行了 exec，读数等价于真跑管线终态，与语料失败 variant 读的「未执行树」根本不是同一状态面——分歧不在时点，在**两个 variant 页跑的是两条不同时间线**。

## 解决方案

- 伪影定性决定性证据 = **失败 variant 页内命令是否运行过**：variant 门控 DBG（包装 `runConformanceTest`，按输入串命中开 trace）+ `TRACE=undefined` 签名（toggle 从未入内）；或断言序号算术（exec 断言 ≤ 窗界 < query-after 断言）。
- 对照组三件套：同形兄弟 variant 的 exec 若落窗内（iul p 变体 ?1001-2000）则全绿；exec 窗内 compare PASS + DBG 全 trace 树/选区正确 ⇒ 引擎行为无缺陷；三 variant dump 配对（full/undefined/undefined）完整呈现时间线分裂。
- 隔离探针与语料行为分歧时，先查 harness 执行语义差异（subsetTest 跳过、窗界算术），再怀疑引擎； DBG 读点对齐原则仍有效，但前提是两边命令都真的跑了。
