# M3 Batch 3 — 簇 D 面 1 module 载入链剔池决策落地

**日期**: 2026-11-06
**前置**: [cluster D triage](2026-11-06-m3-batch3-cluster-d-triage.md)（四面拆分，
面 1 module/fetch CSS module 载入链 ~17F 环境受限坐实，剔池候选挂账
script-sandbox）；本轮按 wc-after（残面落地后二进制）逐文件 F 签名复核后
落地剔池——M3 余项③收口。

## 甄别复核（对 triage 计数的修正）

triage 按「modulepreload-*（7 文件）+ async-fetch-*（8 文件）+ importmap
duplicate/ordering」估 ~17F；逐文件复核后纯载入链家族实际为 **27 文件
27F + 1T**（triage 计数漏了 async-fetch mutation/reinsert 变体与
nested/placeholder 面）。甄别判据（三条同时满足）：

1. 载入机制依赖 CSS module（`type="module"` / `rel=modulepreload` /
   importmap——边界文件 nested-async/nested-sync/placeholder-identity/
   async-fetch-disconnect-iframe 已逐文件 grep 确认）；
2. P=0 且 F 签名 = 载入链死连锁（`shadowRoot` null 连锁 / module 拒约
   `Unhandled rejection`）；
3. 断言链整体依赖模块载入成功，shadow-dom 范围内无可修面。

**出池 27 文件**：async-fetch-* ×15、modulepreload-* ×7、clonenode-async-fetch、
nested-async、nested-sync、placeholder-identity、fetched-module（T 面，
module 载入挂死页级超时）。

**留池 9 文件**（+document-parse-crash 既有 skip）：basic(2F)/multiple(2F)/
invalid-specifier(2F)/clonenode(11F)/idl-absent-and-empty(1P1F)/
idl-feature-detection(1F)/idl-reflection(2F)/same-document(1F)/
serialization(1P3F)——机制断言面 F 为真信号（constructable stylesheet
接线/计算样式/IDL 反射/序列化），归口 rendering-compat 不双改；
clonenode 3F = M3 尾段 shadow-dom 范围内小刀候选。

## 落地面（两侧同源）

- runner：`wc_case_skipped` 新增 `MODULE_LOAD_CHAIN_PREFIXES`（7 前缀，
  name 级 starts_with，document-parse-crash 同款模式），注释含回池条件。
- fetch 脚本：`SD_DECL_TENTATIVE_CASES` 27 行移入
  `SD_DECL_TENTATIVE_MODULE_PARKED`（定义不消费——停拉；回池 = 数组名移回
  fetch 循环），头部 skip 域注释与 tentative 分档注记同步翻转（M2「不预
  排除、如实入账」→ M3「载入链死 F 为伪信号，剔池挂回池条件」）。
- 本地 wpt-data 残留文件不删（runner skip 防误扫双保险；fetch 幂等
  `[[ -s ]]` 跳过已存在文件）。

## 验证

- bash -n 语法过；retained 10 + parked 27 + helper 1 = 38 与目录文件数对账。
- fmt 干净、clippy `-p zero-wpt-runner --all-targets` 0 告警。
- 组级复测（构建后二进制，filter=shadowrootadoptedstylesheets，守卫同参）：
  **parked 27 文件全部出池**；retained 9 文件（document-parse-crash 既有 skip
  外）逐子测试与 wc-after 基线**零移动**，合计 25F+2P 与基线留池 F/P 精确
  一致。通道级 files 414→387 由 skip 组装层推定（skip 在 corpus 组装层生效，
  组级 solo 已实证同一路径）。

## 门值影响（按 M2 修订规则）

- WC 通道分母 −27 文件，F −27、T −1（簇 D 54F → 25F 留池口径，其中
  rendering-compat 归口 ~19F + clone/IDL 尾段候选 ~6F）。
- 簇 D 可翻转上限修订：triage 预演「面 3+4 ~21F + 面 2 剔池 ~17F」→
  实际剔池 27F（面 1 全量）+ 面 3/4 随 m3c2 复测定格。
- A/B 基线口径注记：本刀落地后，此前 A/B（wc-before/wc-after）为剔池前
  414 文件口径；后续 A/B 以 387 文件为新基线，跨刀比对需注明偏移。
- 回池触发条件：script-sandbox 模块载入（module loader / modulepreload /
  importmap fetch）落地，届时移回数组 + 移除 runner 前缀 + 复测。
