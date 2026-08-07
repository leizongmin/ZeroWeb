# AI 大规模重构/移植验收规范（调研 P5 成文）

版本：v1.0 ｜ 日期：2026-08-07 ｜ 状态：Active

> 依据：Ladybird 2026-02 C++→Rust 移植实践（LibJS 前端管线 ~2.5 万行/2 周，
> 验收 = lockstep 字节级输出一致 + 52,898 test262 + 12,461 回归测试 0 回归 +
> 无基准回退 + 环境变量回退）。来源：调研报告
> `docs/research/research-ladybird-browser-2026-08-06.md` §5.2 P5。

## 适用场景

ZeroWeb 是 AI-first 开发项目，以下场景必须遵守本规范：

- AI 大规模重写/移植（如 layout-engine 重构、css-parser 重写、taffy 升级）
- 核心管线（解析→样式→布局→绘制）任一阶段的实现替换
- 跨组件接口契约变更（engine/dom/style-system/layout-engine 边界）

## 核心条款

### 1. 双管线对照（lockstep）

- 新旧实现必须**同时存在且可切换**（feature flag 或环境变量），同输入逐值/逐字节对比
- 对照范围：解析 AST、计算样式、布局树、绘制命令、最终像素
- 任何差异（除明确声明的行为变更）都是缺陷，不是可接受噪声

### 2. 全套件回归 0 差异（硬门禁）

| 套件 | 要求 |
|---|---|
| `cargo test --workspace` | 0 失败（含移除 `#[ignore]`） |
| `make reftest` / `reftest-upstream` 全量 | 通过率不下降（对比迁移前基线） |
| `make reftest-oracle`（Chromium Oracle） | 一致率不下降（诚实口径） |
| `make product-smoke` | diff ≤ 阈值（产品回归门禁） |
| `make layout-golden` | golden 0 diff（布局树回归） |
| `make reftest-smoke` | smoke 清单全过 |

- 全量跑不完（本地超时）时：至少跑 smoke + golden + oracle 代表性目录，并记录覆盖范围
- **通过率数字以迁移前同口径基线为准**（trend.csv / evidence 历史）

### 3. 性能回退检查

- 跑 `scripts/run-benchmarks.sh`，与迁移前基线对比
- 关键路径（布局/样式/绘制）无 >5% 回退；有回退须说明原因与恢复计划

### 4. 环境变量回退（fail-open）

- 新实现默认启用，旧实现经环境变量可切回（如 `ZW_REFACTOR_OLD=1`）
- 回退开关在合入后保留至少一个发布周期，用于线上问题快速回滚

### 5. 分片落地（不一次性大爆炸）

- 大重构拆分为可独立验收/可回退的切片（参考 taffy 迁移裁决：每切片
  A/B 确认 net≥0 且无关键回归后才继续）
- 每个切片独立走本规范验收，独立提交

## 验收清单（每切片合入前逐项勾选）

- [ ] 双管线对照已建（新旧可切换 + 对照点明确）
- [ ] 对照差异全部归因（无未解释差异）
- [ ] `cargo test --workspace` 0 失败
- [ ] reftest/oracle 通过率 ≥ 迁移前基线（同口径）
- [ ] product-smoke 无回归（diff ≤ 阈值）
- [ ] layout-golden 0 diff
- [ ] 基准无 >5% 关键路径回退
- [ ] 回退环境变量已实现
- [ ] 覆盖范围与例外已记录（提交说明）

## 与现有机制的关系

- `scripts/check-test-flakiness.sh`（A3）覆盖「测试稳定性」
- `make reftest-trend`（P2）提供迁移前基线数字
- `docs/goal/rendering-compat.md` 的 A/B 文化（net≥0 判定）与本规范一致
