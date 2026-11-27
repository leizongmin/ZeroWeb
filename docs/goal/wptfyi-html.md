# WPT 全量通过率 · HTML、XHTML 与 DOM 接口

**状态** Active\
**立项** 2026-11-09，按用户要求拆分高贡献 WPT 缺口，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-html/master.md)\
**共同计分与运行规则** [五目标计划](../compat/wptfyi-90-plan.md)

## 基线与目标

2026-11-07 已发布快照中，本目标目录已通过 71,096 / 159,323。`html/the-xhtml-syntax` 25,607 项为零入账。其余大头是 `html/semantics` 15,793、`html/dom` 13,305、`html/rendering` 11,705 项。已有 [HTML 语义目标](html-semantics-compat.md) 和 [HTML 语法归档](archive/html-syntax-compat.md)。

把 `html` 的上游语料接入可复跑的 ZeroWeb 执行链，在同口径全量报告中净增 **81,000** 项通过，达到至少 **152,096** 项（跨周期/升级后全量基线）。保留同 revision 路径级证据，并随最新周跑另报上游漂移。

**2026-11-27 维持 `pin 31597693` 校准口径**（依据 `97c619357` 选 B 拍板与 [三方互斥核账证据](wptfyi-dom/evidence/2026-11-27-wpt-rev-pin-drift-audit-and-dc3-calibration.md)）：经 `pin 31597693` 完整 GitHub 树核账，实测互斥 `post_pin` 漂移为 **3,705** 项（962 文件，修正批十九将 14,067 项 `pin` 内未覆盖误归漂移至 15,429S 的口径）；Chrome `pin` 内通过上限为 `151,503`（低于原线 593）。本周期严格仅扣真实漂移后的 **DC-3 校准验收线为至少 148,391 项通过**（相对 11-07 基线净增 **≥ 77,295** 项）；同时不将载体边界混入漂移核销，单列 `pin` 内三类非单文档硬边界 **5,660** 项（`worker/shadowrealm` 2,443 + `manual` 2,144 + `COOP/COEP/DIP/anonymous-iframe/capability-delegation` 1,073，Chrome 通过 5,404，对应单文档 Chrome 上限约 `146,099`），快照解冻后按实测进展复议是否单列豁免。

## 范围与协作

只按 `html/` 记账，含 HTML DOM 接口与渲染断言。一级 `dom/`、`svg/` 各有独立缺口。HTML 语义流继续维护表单、脚本与元素语义，布局根因与 rendering-compat 协调，先复用已有修复。XML 输入和外部实体处理遵守现有安全边界。 按 [run-rules](../rally/run-rules.md) 处理共享面冲突。有活跃流在同一写面时，先推进独立语料、对账和失败归类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。核对 11 月 8 日 html-sem/web-components 分片缺失，恢复受影响周跑产物。用真实 MIME、命名空间和文档载入路径复跑 XHTML 实体矩阵，核对预期子测试数和外部实体安全边界。
- **DC-2 · 收敛**。扩语料到 DOM 反射、HTML 解析/语义、表单和默认渲染。按主流浏览器对照拆分 iframe、window、worker、资源与 testdriver 缺口。先补共享载体，再修元素和脚本行为。 每批保留结果增减和常驻回归。
- **DC-3 · 全量对账**。按 2026-11-27 维持 `pin 31597693` 严格漂移校准口径验收至少 **148,391** 项通过（净增 ≥ 77,295 项；原跨周期基线 152,096 项、真实 `post_pin` 漂移 3,705 项与 `pin` 内跨载体硬边界 5,660 项同步单列），最新周跑出有效产物，邻域无可归因回退。按共同规则完成守卫下的构建、测试与 clippy，并归档证据。

每轮只更新控制面及证据。最终单行使用 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`，按共同计划中的判据选择。
