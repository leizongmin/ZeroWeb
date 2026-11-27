# WPT 全量通过率 · HTML、XHTML 与 DOM 接口

**状态** Active\
**立项** 2026-11-09，按用户要求拆分高贡献 WPT 缺口，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-html/master.md)\
**共同计分与运行规则** [五目标计划](../compat/wptfyi-90-plan.md)

## 基线与目标

2026-11-07 已发布快照中，本目标目录已通过 71,096 / 159,323。`html/the-xhtml-syntax` 25,607 项为零入账。其余大头是 `html/semantics` 15,793、`html/dom` 13,305、`html/rendering` 11,705 项。已有 [HTML 语义目标](html-semantics-compat.md) 和 [HTML 语法归档](archive/html-syntax-compat.md)。

把 `html` 的上游语料接入可复跑的 ZeroWeb 执行链，在同口径全量报告中净增 **81,000** 项通过，达到至少 **152,096** 项。保留同 revision 路径级证据，并随最新周跑另报上游漂移。

## 范围与协作

只按 `html/` 记账，含 HTML DOM 接口与渲染断言。一级 `dom/`、`svg/` 各有独立缺口。HTML 语义流继续维护表单、脚本与元素语义，布局根因与 rendering-compat 协调，先复用已有修复。XML 输入和外部实体处理遵守现有安全边界。 按 [run-rules](../rally/run-rules.md) 处理共享面冲突。有活跃流在同一写面时，先推进独立语料、对账和失败归类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。核对 11 月 8 日 html-sem/web-components 分片缺失，恢复受影响周跑产物。用真实 MIME、命名空间和文档载入路径复跑 XHTML 实体矩阵，核对预期子测试数和外部实体安全边界。
- **DC-2 · 收敛**。扩语料到 DOM 反射、HTML 解析/语义、表单和默认渲染。按主流浏览器对照拆分 iframe、window、worker、资源与 testdriver 缺口。先补共享载体，再修元素和脚本行为。 每批保留结果增减和常驻回归。
- **DC-3 · 全量对账**。同口径净增至少 81,000 项，最新周跑出有效产物，邻域无可归因回退。按共同规则完成守卫下的构建、测试与 clippy，并归档证据。

每轮只更新控制面及证据。最终单行使用 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`，按共同计划中的判据选择。
