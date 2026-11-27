# WPT 95% · 06 DOM 与 Shadow DOM

**状态** Active\
**立项** 2026-11-09，按用户要求扩展到 95% 冲刺，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-dom/master.md)\
**共同计分与运行规则** [95% 冲刺计划](../compat/wptfyi-95-plan.md)

## 基线与目标

11 月 7 日 [全量快照](../compat/wptfyi/coverage-2026-11-07.md) 已通过 64,947 / 106,550 项，缺口 41,603 项。11 月 8 日周跑的 `custom-elements` 和 `shadow-dom` 产物缺失。11 月 7 日快照中，`shadow-dom/reference-target` 缺 6,233 项，`quirks/unitless-length` 缺 4,789 项。先复用 [DOM 扩面归档](archive/dom-expansion.md) 与 [Shadow DOM 扩面归档](archive/shadow-dom-expansion.md)。

同口径净增 **35,000** 项，达到至少 **99,947** 项通过。固定 revision 的净增与当前周跑的上游漂移分别报告。

## 范围与协作

计分目录为 `dom/`、`shadow-dom/`、`custom-elements/`、`quirks/`、`domparsing/`、`domxpath/`、`sanitizer-api/`、`trusted-types/`、`xml/`、`webidl/`。

仅计本入口列出的一级目录。`html/` 归 05。与 04 及现有编辑流共享 DOM 变异和 Range 入口时，先划分代码写面。Quirks 牵涉样式解析时与 03 和 rendering-compat 协作。 共享文件按 [Rally 运行规则](../rally/run-rules.md) 分配归属。等待共享修复时，继续独立语料导入、对账和失败分类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。在干净语料上通过守卫跑 `testharness-dom`、`testharness-shadow-dom` 和 `testharness-web-components`。对照周跑检查缺失产物、变体映射和已通过结果，优先恢复原先可计分的 Shadow DOM 与 Custom Elements 路径。 归档固定 revision、当前 commit、预期与实际执行路径，以及按路径和变体的差额。
- **DC-2 · 按簇收敛**。先收敛 Reference Target、Quirks 解析及 DOM Parsing，再处理 slot 分发、树操作、IDL 和安全 API。Sanitizer 与 Trusted Types 的验证要覆盖输入拒绝和策略约束。用真实宿主验证跨文档、realm 和 Shadow DOM 可观察行为。 修复随附常驻 WPT 或等价回归。
- **DC-3 · 全量对账**。接入周跑并验收净增至少 35,000 项，邻域无可归因回退。完成守卫下的构建、测试与 clippy，保存有效分片和全量结果，再归档。

每轮更新控制面与证据，最后一行按 [共同计划](../compat/wptfyi-95-plan.md#无人值守运行) 输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。
