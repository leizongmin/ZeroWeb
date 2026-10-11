# File API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../fileapi-compat.md](../fileapi-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-11（M1 基线落地）

## 当前状态

**目标 Active（M1 完成，M2 语义修齐待启动）**：M1 通道 + 甄别 + 基线落地——
可执行子集 **35 案 499 子测试 311 Pass = 62.3%**（6 案全绿，0 Timeout）。
通道：`testharness-fileapi` 子命令（独立模块 `src/testharness_fileapi.rs`，不并入
10.5k 行 testharness.rs）+ `fetch-wpt-fileapi`/`testharness-fileapi` Makefile target +
`scripts/goals/16-fileapi-compat.sh` 语料预置（96 文件，排除面双处记账）。
基线与甄别：[evidence/2026-10-11-m1-baseline.md](evidence/2026-10-11-m1-baseline.md)。

既有实现底（核查于 M1 时点）：Blob/File/FileReader/FormData/createObjectURL 均已在
`crates/engine/src/js_dom_shim/part02.js` 纯 JS 落地（R2791 FileReader、R31xx worker
blob URL 先例）；`<img src=blob:>`/fetch/XHR 消费路径实测存活（url 面基线 84%）。

## 缺口清单（M2 簇，按预期收敛顺序）

| # | 簇 | 规模（F） | 状态 |
|---|---|---|---|
| P1 | Blob/File 构造器 WebIDL 转换（blobParts/fileBits sequence+iterator、TypeError、interface object） | ~110 | ⬜ |
| P2 | endings 规范化（Blob/File 构造器 native line ending，.any.js+.html 双形态同根因） | ~40 | ⬜ |
| P3 | readAs* 读取状态机（result 阶段 null、结果类型、multiple-reads 队列、abort、Determining-Encoding） | ~31 | ⬜ |
| P4 | 零散面：Blob.bytes() 缺失、detached ArrayBuffer、FileList 空表、url origin 序列化残余 | ~15 | ⬜ |

## 排除面（挂账，非待办）

- multi-global/第二文档族（url/ 全族 ~12 案，内容标记 `iframe`/`window.open`）→
  navigation-compat iframe 管道同门槛重入。
- dedicated worker 面 5 案 → workers-compat 管道域。
- streams 消费端 2 案（Blob-stream/TextStream）→ net-api streams 域同门槛。
- 表单提交回显 10 案（wptserve `.py`）→ runner 服务器回环，暂无计划。
- idlharness 3 案（WebIDLParser build 资产缺失）→ 上游资产导入评估。
- BlobURL 分区/HTTPS 5 案 → security-hardening 分区面。

## 后续重入条件（记账）

- DC-2 门禁参照 webstorage 先例：分级通过率不再下行两个连续轮次 + DC-3 逐修复
  imported-tests.txt 账本 + make test 全绿 / make reftest 零回归 / product-smoke 恒值。
- 数字维护：每轮修齐后向 `docs/compat/trends/wpt-suites.csv` 追加行（M1 行
  2026-10-11 = active，499/311）。
- js_dom_shim 的 blob 已知限制注释（part02.js 「blob: 不被 net/fetch 实际解析」）与
  基线实测不符，M2 动 part02 时顺手更正注释（不单独开轮）。
