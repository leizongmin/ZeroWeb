# WPT 95% · 09 SVG、MathML 与动画

**状态** Active\
**立项** 2026-11-09，按用户要求扩展到 95% 冲刺，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-graphics/master.md)\
**共同计分与运行规则** [95% 冲刺计划](../compat/wptfyi-95-plan.md)

## 基线与目标

11 月 7 日 [全量快照](../compat/wptfyi/coverage-2026-11-07.md) 已通过 3,974 / 30,250 项，缺口 26,276 项。`svg/interact` 缺 9,172 项，`mathml/relations` 缺 3,239 项，`web-animations/animation-model` 缺 2,680 项。复用 [SVG 二期](archive/svg-compat-v2.md)、[WebGL](archive/webgl-compat.md) 和 [动画](archive/timing-animation-compat.md) 的既有测试。

同口径净增 **22,000** 项，达到至少 **25,974** 项通过。固定 revision 的净增与当前周跑的上游漂移分别报告。

## 范围与协作

计分目录为 `svg/`、`mathml/`、`web-animations/`、`scroll-animations/`、`webgl/`。

仅计本入口所列目录的可计分子测试。`css/` 归 03，`html/` 归 05。由 rendering-compat 继续维护已在改的布局和渲染路径。行为涉及像素时保留真实 reftest 证据，涉及输入时复用 10 的输入链。 共享文件按 [Rally 运行规则](../rally/run-rules.md) 分配归属。等待共享修复时，继续独立语料导入、对账和失败分类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。在受守卫的 `testharness-svg`、`testharness-web-animations`、`testharness-webgl` 中选取真实变体，并加跑 MathML 几何样本。确认 SVG 事件和命中路径、MathML 几何、动画时钟、GPU 能力分别能产生可对账结果。 归档固定 revision、当前 commit、预期与实际执行路径，以及按路径和变体的差额。
- **DC-2 · 按簇收敛**。先处理 SVG 交互与几何，再扩 MathML 布局和动画模型。按时间线、取消与完成事件、滚动动画、WebGL 状态分别验证。记录 CPU/GPU 和无头环境的实际差异，并保留可观察的布局或绘制回归。 修复随附常驻 WPT 或等价回归。
- **DC-3 · 全量对账**。接入周跑并验收净增至少 22,000 项，邻域无可归因回退。完成守卫下的构建、测试与 clippy，保存有效分片和全量结果，再归档。

每轮更新控制面与证据，最后一行按 [共同计划](../compat/wptfyi-95-plan.md#无人值守运行) 输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。
