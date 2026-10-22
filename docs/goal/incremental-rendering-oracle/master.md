# 增量渲染对照控制面

**日期**：2026-10-23
**状态**：Active。M1/M2 DONE。M3 拆面 a 切片完成：后处理敏感路径（margin 折叠/float 避让/overflow 滚动容器）冷温最终输出一致 + 溢出枚举入快照 + fixture 有效性自检（oracle 9/9，layout-engine 2080/2080）；`make product-smoke` 生产路径运行 PASS（DC-13 门禁 exit 0）。
**当前卡点**：DC-4 主体（半透明帧复用、多次导航/缓存释放）与命中测试对照在 engine pipeline（并行流活跃工作面 9e9924cac 等），按 run-rules §9 跨流协调待办；反例 A 保持开放（生产不可达）。
**下一步**：M4 本流可做部分：DC 审计汇总（五 DC 逐项证据核对表）、DC-5 收口（检查模式成本：生产默认路径仅 `:has` 回退分支一行计数的事实陈述 + bench-gate + 常驻 oracle 回归；规范回归审计边界），并继续评估 M3 engine 面（跨流协调）是否解封。
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
| M3 | 生产帧/像素与释放验证 | **IN_PROGRESS**（a 切片：后处理敏感路径冷温一致 + 溢出面 + 生产 smoke 运行 PASS；帧复用/缓存释放/命中在 engine 面跨流协调中） | [m3-a](evidence/2026-10-23-m3-a-postprocess-final-output/ANALYSIS.md) |
| M4 | 检查成本、规范回归和 DC 审计 | NOT_STARTED | 无 |

## 验证基线

| 判据 | 场景 | 状态 | 证据 |
|---|---|---|---|
| DC-1 | INC-01-dom-cssom-mutations | **DONE**（style 层 12 序列 + DualPathRun 回放工具与首差异报告 + 注入演练检出实证；反例 A 固化为 Divergent 检出） | [m1-style](evidence/2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md) / [m2-s1](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) / [m2-drill](evidence/2026-10-23-m2-drill-injected-invalidation-detection/ANALYSIS.md) |
| DC-2 | INC-02-inheritance-shadow-has | **DONE**（继承/伪元素/`:has` 回退一致且回退有计数、Shadow/lang/根字号度量 Equal；slot 边界已记录：style 层无 slot 感知、两路径遍历语义相同无分歧面，失效路由在生产 engine 层归 M3/跨流协调） | [m2-s1](evidence/2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md) / [m2-s2](evidence/2026-10-23-m2-s2-has-counter-shadow-font-sequences/ANALYSIS.md) |
| DC-3 | INC-03-layout-cache-inputs | **IN_PROGRESS**（布局面：宽度/字体/结构/视口/后处理敏感路径冷温一致 + 撤用增量臂边界 Divergent；命中逻辑在 engine 面跨流协调） | [m1-layout](evidence/2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md) / [m3-a](evidence/2026-10-23-m3-a-postprocess-final-output/ANALYSIS.md) |
| DC-4 | INC-04-final-output-and-retention | **IN_PROGRESS**（布局面：溢出枚举入快照 + 后处理敏感路径一致 + production smoke PASS；帧复用/缓存释放/命中在 engine 面跨流协调） | [m3-a](evidence/2026-10-23-m3-a-postprocess-final-output/ANALYSIS.md) |
| DC-5 | 生产默认路径与检查成本 | **IN_PROGRESS**（production smoke PASS + bench-gate PASS + 生产默认路径仅 `:has` 回退分支一行计数的事实核查；M4 收口审计） | [m3-a](evidence/2026-10-23-m3-a-postprocess-final-output/ANALYSIS.md) |

## 碰撞与执行记录

候选共享面：style/layout/engine/render-foundation/compositor/字体及 WPT corpus。本轮实际占用：`crates/style-system`（lib.rs 提取 + 增量链走修复 + oracle 测试）+ 本 goal 文档；未触碰 engine/layout/WPT 语料，无冲突（729e858ad 为 zero-web 流 engine 提交，无文件交叠）。保留现有 WPT pin。

启动后执行记录：
- 2026-10-23：启动门控解除；M1 style 切片（盘点 + oracle 8/8 + 反例 A/B 固化）。
- 2026-10-23：M2 切片 1：反例 B/C 根因修复（共享 helper + 链走容器上下文），oracle 9/9、style-system 全量 2445/2445、定向 bench-gate PASS；反例 A 记为 API 契约保持开放。`evidence/` 与 `archive/` 追加保存；内部等价不能替代规范正确性验收。
- 2026-10-23：M2 切片 2：`:has()` 回退计数（实例字段，DC-2 审计可精确断言）+ Shadow/lang/根度量序列，oracle 12/12、style-system 2448/2448；记录 rem 恒按常量解析的事实（规范疑点归 rendering-compat）。
- 2026-10-23：M1 布局切片：layout-engine 冷/温双路径几何快照 5/5（f32 精确相等契约）+ 撤用增量臂 Divergent 边界；生产布局失效路由核查（paint-only 复用整帧/其余快照全量重建，`compute_incremental` 生产不可达）；layout-engine 2076/2076；纯测试切片零生产变更。
- 2026-10-23：M2 收口注入演练：同时撤回反例 B/C 修复与 DC-2 计数（三处一行级注入），oracle 3/3 对应检出、8 序列无误报，还原后 11/11 复绿；注入不入库（`git status` clean 复核）。M2 标 DONE，DC-1/DC-2 标 DONE。
- 2026-10-23：M3-a 切片：后处理敏感三序列（折叠/float/overflow）冷温一致 + 溢出枚举入快照 + fixture 有效性自检，oracle 9/9、layout-engine 2080/2080；`make product-smoke` exit 0（三 fixture struct-check PASS）——零生产变更纯测试切片。
- 跨流协调事实：M3 的 pipeline 帧复用/缓存生命周期观察面（crates/engine）为并行流活跃工作面（9e9924cac 等），按 run-rules §9 不擅自改动；layout-engine 侧后处理对照已按 a 切片收口。
