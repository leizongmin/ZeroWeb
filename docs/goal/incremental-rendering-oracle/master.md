# 增量渲染对照控制面

**日期**：2026-10-23
**状态**：Active。M1 DONE（style + layout 双路径快照）；M2 DONE（切片 1/2 + 人工注入演练：三处撤回修复全部被对应断言检出且无误报，注入不入库）。累计：style oracle 12/12、layout oracle 5/5、layout-engine 2076/2076。
**当前卡点**：M3 生产帧/像素与缓存生命周期（DC-4）的主战场在 engine pipeline（cached_styles/cached_layout/cached_doc、帧复用）——engine 是并行流活跃工作面（9e9924cac 等每轮提交），按 run-rules §9 属跨流协调面，不在本流擅自改动。反例 A（changed 集完备性契约）保持开放，生产不可达。
**下一步**：M3 拆面推进：layout-engine 侧"最终后处理 → 最终输出"对照（postprocess 族在 layout-engine，本流面内）先行固化；pipeline 帧复用/缓存释放观察（engine 面）记跨流协调待办——等面空或用户裁决后再动；M4 检查成本与 DC 审计（本流可做部分）随后收口。
**入口**：[目标契约](../incremental-rendering-oracle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 完整计算参考与最终输出快照 | style 层（[m1-style](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md)）+ 布局层冷/温双路径（[m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md)）已建；engine paint/像素输出快照待 M3 | **PARTIAL** |
| 继承/Shadow/伪元素/字体失效序列 | style 层全覆盖（B/C 修复 + `:has` 计数 + Shadow/lang/根度量，注入演练实证检出）；slot 无 style 层语义，失效路由在 engine 层归 M3 | **DONE**（style 层） |
| 缓存与帧复用生命周期 | 固定重复导航、缓存释放观测 | NOT_STARTED |
| 浮点与像素容差、检查模式成本阈值 | style/几何层精确相等（f32 位级）已定（[m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md)）；像素容差待 M3 paint 面按输出契约定 | **PARTIAL** |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 入口/缓存盘点与同输入双路径基线 | **DONE**（style 层 + 布局层双路径快照与几何容差契约落地；engine paint/像素输出归 M3） | [m1-style](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md) / [m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md) |
| M2 | 变更回放与首差异定位 | **DONE**（切片 1：B/C 根因修复；切片 2：`:has()` 回退计数 + Shadow/lang/根度量序列；注入演练：3 撤回修复全检出无误报——[m2-drill](evidence/2026-10-23-m2-drill-injected-invalidation-detection/ANALYSIS.md)） | [m2-s1](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) / [m2-s2](evidence/2026-10-23-m2-s2-has-counter-shadow-font-sequences/ANALYSIS.md) / [m2-drill](evidence/2026-10-23-m2-drill-injected-invalidation-detection/ANALYSIS.md) |
| M3 | 生产帧/像素与释放验证 | NOT_STARTED | 无 |
| M4 | 检查成本、规范回归和 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | INC-01-dom-cssom-mutations | **DONE**（style 层 12 序列 + DualPathRun 回放工具与首差异报告 + 注入演练检出实证；反例 A 固化为 Divergent 检出） | [m1-style](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md) / [m2-s1](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) / [m2-drill](evidence/2026-10-23-m2-drill-injected-invalidation-detection/ANALYSIS.md) |
| DC-2 | INC-02-inheritance-shadow-has | **DONE**（继承/伪元素/`:has` 回退一致且回退有计数、Shadow/lang/根字号度量 Equal；slot 边界已记录：style 层无 slot 感知、两路径遍历语义相同无分歧面，失效路由在生产 engine 层归 M3/跨流协调） | [m2-s1](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) / [m2-s2](evidence/2026-10-23-m2-s2-has-counter-shadow-font-sequences/ANALYSIS.md) |
| DC-3 | INC-03-layout-cache-inputs | **IN_PROGRESS**（布局面：宽度/字体/结构/视口冷温一致 + 撤用增量臂边界 Divergent 固化；生产缓存粒度=全有或全无已核查；命中断言待 M3） | [m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md) |
| DC-4 | INC-04-final-output-and-retention | NOT_RUN | 无 |
| DC-5 | 生产默认路径与检查成本 | NOT_RUN | 无 |

## 碰撞与执行记录

候选共享面：style/layout/engine/render-foundation/compositor/字体及 WPT corpus。本轮实际占用：`crates/style-system`（lib.rs 提取 + 增量链走修复 + oracle 测试）+ 本 goal 文档；未触碰 engine/layout/WPT 语料，无冲突（729e858ad 为 zero-web 流 engine 提交，无文件交叠）。保留现有 WPT pin。

启动后执行记录：
- 2026-10-23：启动门控解除；M1 style 切片（盘点 + oracle 8/8 + 反例 A/B 固化）。
- 2026-10-23：M2 切片 1：反例 B/C 根因修复（共享 helper + 链走容器上下文），oracle 9/9、style-system 全量 2445/2445、定向 bench-gate PASS；反例 A 记为 API 契约保持开放。`evidence/` 与 `archive/` 追加保存；内部等价不能替代规范正确性验收。
- 2026-10-23：M2 切片 2：`:has()` 回退计数（实例字段，DC-2 审计可精确断言）+ Shadow/lang/根度量序列，oracle 12/12、style-system 2448/2448；记录 rem 恒按常量解析的事实（规范疑点归 rendering-compat）。
- 2026-10-23：M1 布局切片：layout-engine 冷/温双路径几何快照 5/5（f32 精确相等契约）+ 撤用增量臂 Divergent 边界；生产布局失效路由核查（paint-only 复用整帧/其余快照全量重建，`compute_incremental` 生产不可达）；layout-engine 2076/2076；纯测试切片零生产变更。
- 2026-10-23：M2 收口注入演练：同时撤回反例 B/C 修复与 DC-2 计数（三处一行级注入），oracle 3/3 对应检出、8 序列无误报，还原后 11/11 复绿；注入不入库（`git status` clean 复核）。M2 标 DONE，DC-1/DC-2 标 DONE。
- 跨流协调事实：M3 的 pipeline 帧复用/缓存生命周期观察面（crates/engine）为并行流活跃工作面（9e9924cac 等），按 run-rules §9 不擅自改动；layout-engine 侧后处理对照（本流面内）先行。
