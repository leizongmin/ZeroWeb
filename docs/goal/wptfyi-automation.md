# WPT 95% · 10 WebDriver 与真实输入

**状态** Active\
**立项** 2026-11-09，按用户要求扩展到 95% 冲刺，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-automation/master.md)\
**共同计分与运行规则** [95% 冲刺计划](../compat/wptfyi-95-plan.md)

## 基线与目标

11 月 7 日 [全量快照](../compat/wptfyi/coverage-2026-11-07.md) 已通过 2,129 / 14,742 项，缺口 12,613 项。`webdriver/tests` 的 10,439 项通过数为零。仓库已有 [WebDriver 服务](../../apps/webdriver/README.md)、[协议实现归档](archive/webdriver.md) 和 [验收入口](../../.agents/skills/zeroweb-verify/features/embedding-automation.md)。

同口径净增 **10,000** 项，达到至少 **12,129** 项通过。固定 revision 的净增与当前周跑的上游漂移分别报告。

## 范围与协作

计分目录为 `webdriver/`、`pointerevents/`、`touch-events/`、`uievents/`。

负责上游 wdspec 与生产 WebDriver 会话、输入和结果接线。`editing/`、`selection/`、`input-events/` 归 04。其他 goal 复用执行链，结果按测试路径归属。多源和证书夹具由 07 维护。 共享文件按 [Rally 运行规则](../rally/run-rules.md) 分配归属。等待共享修复时，继续独立语料导入、对账和失败分类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。用守卫启动与当前源码匹配的 WebDriver 和浏览器子进程，在真实浏览器上跑少量 session、导航、执行脚本和 actions 用例。核对退出、清理、真实输入副作用，以及上游 pytest 参数化 ID 对 wpt.fyi 路径和子测试名的映射。 归档固定 revision、当前 commit、预期与实际执行路径，以及按路径和变体的差额。
- **DC-2 · 按簇收敛**。按 session、导航、frame/window、元素、脚本、actions 与截图分片。补 Pointer/Touch/UI Events 的真实事件顺序、捕获与焦点断言。分片中断后保留已完成结果，重跑同一条只入账一次。 修复随附常驻 WPT 或等价回归。
- **DC-3 · 全量对账**。接入周跑并验收净增至少 10,000 项，邻域无可归因回退。完成守卫下的构建、测试与 clippy，保存有效分片和全量结果，再归档。

每轮更新控制面与证据，最后一行按 [共同计划](../compat/wptfyi-95-plan.md#无人值守运行) 输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。
