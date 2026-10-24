# M4 本流收口：五 DC 审计汇总 + DC-5 检查成本事实核查 + 规范回归审计边界

**运行身份**：基线 HEAD `f99fbc50a`（M3-a）。**纯审计/文档切片，零代码变更**。
**目的**：兑现 M4 行"检查成本、规范回归和 DC 审计"中本流（style/style-system/layout-engine 面）可完成部分；engine pipeline 面（帧复用/缓存释放/命中/像素）维持跨流协调待办（控制面"当前卡点"）。

## DC-5 事实核查（当前 HEAD 逐条源码核对）

goal 契约 DC-5："关闭检查模式运行生产路径——检查默认不增加生产全量工作或诊断输出；对应基准与常驻回归有据"。

本 goal 引入的生产代码改动全集（其余均为测试/文档）：

| 改动 | 位置 | 性质 | 生产成本 |
|---|---|---|---|
| 链走容器上下文 + 祖先伪元素挂载修复 | `crates/style-system/src/lib.rs`（`compute_styles_incremental` 链走，m2-s1） | 正确性修复（非 instrumentation） | 定向 bench-gate PASS（m2-s1，修复当时 HEAD） |
| `incremental_has_fallbacks` 计数 | 同文件 :314（字段）/:335（初始化）/:340-341（getter）/:482（自增） | 唯一 instrumentation | 见下 |

计数自增的触达面：`compute_styles_incremental` 生产调用点两处——`crates/engine/src/pipeline/mod.rs:2400`（管线增量重算，首帧后稳态路径）与 `crates/engine/src/js_dom_bridge/computed_style_cache.rs:484`（增量 cascade）。自增语句位于 `:has()` 回退分支首语句（lib.rs:481-485）：

- 仅当样式表含 `:has()` 才进入该分支，且分支主体随即全量重算——一次 u64 自增相对全量计算可忽略；
- 无 `:has()` 页面该分支不进入，成本为零；
- getter `incremental_has_fallback_count()` 全仓仅 `incremental_oracle.rs` 三处测试消费，无生产调用（grep 核对）。

诊断输出：本 goal 未新增任何 `tracing`/打印/环境变量开关；"检查模式"维度上生产默认路径的新增面为零。

## 三重证据（DC-5 判据"对应基准与常驻回归有据"）

| 证据 | 内容 | 结果 |
|---|---|---|
| 定向基准 | `ZERO_WEB_BENCH_CRATES=zero-style-system make bench-gate`（本轮复跑，绑定当前 HEAD——m2-s2 之后 style-system 生产面有一次并行流改动 813706ca6，故不复用旧结论） | **PASS**（exit 0：microbench 1/1、absolute page-total/retained-form budgets 通过；输出 `benchmark_20261023_043128.json`。基线硬件不匹配 WARN 为既有信息项，相对门禁未启用） |
| 生产路径运行 | `make product-smoke`（m3-a，exit 0）：全生产管线 style→layout→paint 对齐 chromium oracle，即"关闭检查模式运行生产路径"的实证 | PASS |
| 常驻回归 | style oracle 12/12（1d5624454 起，其后 813706ca6 并行改动经 make test 复绿）+ 布局 oracle 9/9（f99fbc50a） | 两套常驻 |

## 五 DC 审计汇总

| DC | 契约判据 | 本流证据 | 结论 | 余项（归属） |
|---|---|---|---|---|
| DC-1 | dom/cssom 变更序列一致 + 回放/差异报告 | [m1-style]（DualPathRun + 首差异）+ [m2-s1]（反例 B/C 根因修复）+ [m2-drill]（3 注入全检出、8 序列无误报） | **DONE** | 反例 A 开放：changed-set 完整性是调用方契约，生产不可达（pipeline 由 mutations 派生 changed），Divergent 断言常态守护 |
| DC-2 | 继承/伪元素/Shadow/slot/`:has`；全量回退有计数 | [m2-s1] + [m2-s2]（计数 0→0→1 精确断言；Shadow/lang/根度量 Equal） | **DONE** | slot 失效路由在生产 engine 层（style 层两路径遍历语义相同，无分歧面）——跨流协调 |
| DC-3 | 缓存按真实输入失效/保留，移动复用几何正确；命中/必须失效断言 | [m1-layout]（宽度/字体/结构/视口冷温一致 + 撤用增量臂 Divergent 边界）+ [m3-a]（后处理敏感三序列 + fixture 有效性自检） | **布局面 DONE** | 命中/必须失效断言在 engine 面——盒树几何冷温一致已钉，命中消费输入稳定；命中逻辑对照归跨流协调 |
| DC-4 | 几何/溢出/命中/像素；缓存不保留旧文档 | [m3-a]（溢出枚举入快照 + 后处理一致 + production smoke PASS） | **布局面 DONE** | 半透明帧复用、多次导航/缓存释放、命中/像素在 engine paint 面——跨流协调 |
| DC-5 | 检查默认不增生产工作/诊断输出；基准与常驻回归有据 | 本证据（事实核查 + 当前 HEAD bench-gate + product-smoke + 常驻 oracle） | **DONE**（本流） | 若 engine 面未来引入检查 UI/检查模式开关，再按同一口径评估 |

## 规范回归审计边界

内部等价 ≠ 规范正确性。本 goal 的 oracle 断言"同实现内增量路径与全量路径一致"，不断言"实现符合 CSS 规范"：

- 规范回归验收归 rendering-compat 面（WPT/reftest 常驻集 + product-smoke 外部 oracle）。
- 已移交 rendering-compat 的疑点：`rem` 恒按常量 `ROOT_FONT_SIZE` 解析而非 html font-size（[m2-s2] 记录）；margin 折叠/float 避让本身的规范正确性（[m3-a] 边界注记，本流只钉双路径一致）。
- 撤用臂边界（`compute_incremental` 生产不可达）与反例 A 属**调用方契约**缺口，非规范缺口：前者已固化 Divergent 检出 + 重启用前置（[m1-layout]），后者保持开放。

[m1-style]: ../2026-10-23-m1-style-dualpath-baseline/ANALYSIS.md
[m2-s1]: ../2026-10-23-m2-s1-chain-walk-context-fix/ANALYSIS.md
[m2-s2]: ../2026-10-23-m2-s2-has-counter-shadow-font-sequences/ANALYSIS.md
[m1-layout]: ../2026-10-23-m1-layout-dualpath-snapshot/ANALYSIS.md
[m2-drill]: ../2026-10-23-m2-drill-injected-invalidation-detection/ANALYSIS.md
[m3-a]: ../2026-10-23-m3-a-postprocess-final-output/ANALYSIS.md
