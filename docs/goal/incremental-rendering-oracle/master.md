# 增量渲染对照控制面

**日期**：2026-10-23
**状态**：Active。M1 布局切片完成：layout-engine 冷/温全量双路径几何快照（5 序列 Equal，f32 精确相等契约落地）+ 撤用增量臂 `compute_incremental` 的 Divergent 边界固化（taffy style 快照边界，slice14 撤用依据回归锚）；纯测试切片零生产变更。累计：style oracle 12/12、layout oracle 5/5。
**当前卡点**：反例 A（`:only-child` 类兄弟失效）保持开放——属增量 API 的 changed 集完备性契约，生产现路由（单目标 paint-only mutation）不可达；不在增量入口内修，避免给全量路径加无用重算。
**下一步**：M3 生产帧/像素与释放验证：frame 复用与半透明帧（DC-4 前半，pipeline paint 面复用 parity 采集器）、多次导航/缓存生命周期（cached_styles/cached_layout/cached_doc 不保留旧文档）——style/layout 两层 oracle 均已在库，对照面上移到 engine pipeline 输出。
**入口**：[目标契约](../incremental-rendering-oracle.md) / [启动与证据协议](../ladybird-followups/README.md)

## 缺口与待决

| 缺口/选择 | 首步 | 状态 |
|---|---|---|
| 完整计算参考与最终输出快照 | style 层（[m1-style](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md)）+ 布局层冷/温双路径（[m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md)）已建；engine paint/像素输出快照待 M3 | **PARTIAL** |
| 继承/Shadow/伪元素/字体失效序列 | B/C 修复收口（切片 1）；切片 2：`:has()` 回退计数 + Shadow/lang/根度量序列 Equal 基线（[证据](evidence/2026-10-23-m2-s2-has-counter-shadow-font-sequences/ANALYSIS.md)）；slot 失效等 engine 层 flat-tree 切片 | **PARTIAL** |
| 缓存与帧复用生命周期 | 固定重复导航、缓存释放观测 | NOT_STARTED |
| 浮点与像素容差、检查模式成本阈值 | style 层精确相等已定；几何/像素容差待布局切片 | **PARTIAL** |

## 里程碑

| 阶段 | 产物 | 状态 | 证据 |
|---|---|---|---|
| M1 | 入口/缓存盘点与同输入双路径基线 | **DONE**（style 层 + 布局层双路径快照与几何容差契约落地；engine paint/像素输出归 M3） | [m1-style](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md) / [m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md) |
| M2 | 变更回放与首差异定位 | **IN_PROGRESS**（切片 1/2 完成：B/C 根因修复、`:has()` 回退计数、Shadow/lang/根度量序列；oracle 12/12） | [m2-s1](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) / [m2-s2](evidence/2026-10-23-m2-s2-has-counter-shadow-font-sequences/ANALYSIS.md) |
| M3 | 生产帧/像素与释放验证 | NOT_STARTED | 无 |
| M4 | 检查成本、规范回归和 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | INC-01-dom-cssom-mutations | **IN_PROGRESS**（style 层 4 序列 Equal + 反例 A 固化；B/C 修复回归入账；回放工具化待 M2） | 两份证据同上 |
| DC-2 | INC-02-inheritance-shadow-has | **IN_PROGRESS**（继承/`:has` 回退一致且回退有计数、Shadow/lang/根度量序列 Equal；slot 失效待 engine flat-tree 切片） | m2-s1 / m2-s2 证据同上 |
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
