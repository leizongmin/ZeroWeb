# 测试、构建、官网与 Agent 工程工具

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖测试方法、资源门禁、工具链/发布、官网与 Agent 工作流治理，全部业务源码只读。算法实现归页面、脚本、数据、图形、媒体、产品与 runtime 各领域；工程主题保留如何定位证据及哪些结果不能互相替代。关键词：WPT、test-guard、CI、packaging、Rally、parity、learnings。

### User Task Navigation
| 用户问题 | 源码入口 | 相关知识 | 验证入口 |
|---|---|---|---|
| 上游案例未执行或分母变化 | `tests/wpt-runner/src/main.rs`、`testharness.rs` | 测试子索引 | [项目验证](../../../zeroweb-verify/SKILL.md)，按请求选范围 |
| 编译绿但产品子进程失败 | `Makefile`、平台 package 脚本 | 构建交付子索引 | 同一 verify 的产品/IPC 入口，不代替真实运行 |
| 截图/性能报告能否支持结论 | parity 比较器、`scripts/perf-gate.sh` | 测试与 Agent 工具子索引 | 现有 parity/bench 入口，本文未实跑 |
| 官网数据未更新 | metrics generator、`website/progress.js` | 官网主题 | 静态数据检查和 Pages 终态另验 |
| 贡献需谁接纳、哪些来源有效 | governance、CODEOWNERS | 治理主题 | 人类 adoption，不由知识核验替代 |

## Gotchas
- Makefile、CI、Rally 各自限定触发、feature、阈值与权限；相同名称的“通过”可能覆盖不同阶段，不能把本地 test、CI、产品验收与知识来源检查互换。 (`Makefile`, `.github/workflows/ci.yml`, `docs/rally/jobs.yaml`)
- 知识维护只读取工具实现与证据契约，不继承网站优化/巡检/cron 的修复、通知、推送或自动合并授权。 (`.agents/skills/zeroweb-site-optimizer/SKILL.md`, `.agents/skills/zeroweb-regression-guard/SKILL.md`)

## Architecture
- WPT 成绩经可恢复资产和 runner 生成，性能经测量/门禁分离，官网再读取已提交趋势；公开页面数字不是新一轮独立验收。 (`tests/wpt-runner/imported-tests.txt`, `scripts/bench-report.sh`, `scripts/perf-gate.sh`, `scripts/generate-website-metrics.py`)
- 平台包包含四个角色，运行前必须核对实际执行体而非只有 Cargo Fresh；GPU/CPU、独立角色和 test-support 路径分别验证。 (`Makefile`, `scripts/package-macos.sh`, `scripts/package-linux.sh`, `scripts/package-windows.ps1`)

## Child Knowledge Nodes
- [兼容性与回归证据](testing/index.md) — 查案例发现、断言有效性、oracle/性能口径及缺测范围。
- [构建、门禁与交付资源](build-and-release/index.md) — 查 feature 工具链、编译/运行相位、CI 与包布局。
- [Agent 工作流与证据工具](agent-tools/index.md) — 查恢复、审查/附件身份、巡检缓存和经验维护授权。
- [官网页面、进度数据与发布链](website.md) — 查趋势 CSV 到页面、双语周报和 Pages 触发/陈旧数据。
- [文档、许可证、贡献与证据来源边界](governance.md) — 查 G0 人类责任接纳、风险等级、目标归档和许可边界。
