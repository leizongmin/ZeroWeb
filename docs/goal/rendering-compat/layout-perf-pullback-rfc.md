# layout-engine 热路径性能回拉专项（GB-20260907 批复①立项）

**状态**: Active
**批复**: 2026-09-07 用户批复「②先解阻+①立项跟进」——②三滞后平台（6973p/7763/8573c）基线 re-capture 已于同日执行完毕（见 `docs/specs/performance-and-resource-budget.md` §4 执行记录），①专项随本文件立项
**依据**: CI-GUARD 第二十轮（run 33875988456）归因证据链闭合 + 第二十四轮（run 34034764552）「间歇红转常态红」兑现
**工作域**: 渲染流（layout-engine 热路径）

---

## 1. 使命

把 8/30-8/31 布局正确性大潮（R3836-R3862 等 30+ 修复）引入的 layout-engine 热路径 +25-30% 性能成本**拉回回归前水平**。**正确性收益不回退**——本专项只找「合法的优化空间」，不做任何规范语义让步。

## 2. 背景与目标锚点

- 回归窗口 `ed3dfb4df`→`242871555`（8/30-8/31，19 提交）：bidi-override 行反转 / display:contents 子级穿透 ×3 / R3848 contents 文本子提升 / R3857 replaced boxes below floats / **R3858 abspos nested-CB re-resolve（提交说明自记「全树 walk +0.7ms」）** / aspect-ratio flex/grid 五连，及 R3867-R3872、9/1 R3901-R3903。
- 成本结构：**布局 pass 数量增加 + 每 pass 全树 walk**。
- 目标锚点（回归前水平，本地 A/B 端点）：
  - `block_layout_1000_elements` ≈ **1.91ms**（本地 dev box）
  - 对应 CI 平台回归前基线：7763 ≈ 3.60ms / 6973p ≈ 2.11ms / 8573c ≈ 2.49ms
- 进度表：re-capture 后的三平台基线 + weekly `--auto-tighten`——专项每步优化经门禁量化，实测回落即自动收紧新基线。

## 3. 已知候选（第二十/二十四轮归因遗留的具体抓手）

| 抓手 | 来源 | 预期 |
|---|---|---|
| abspos nested-CB re-resolve 预扫描覆盖缺口 | R3858 提交说明自记「styles 预扫描仅跳过无 positioned 页面，本 bench 树无 positioned 元素预扫描应跳过，残余成本在其余 pass 累积」 | 短平快第一刀 |
| pass 数量 × 全树 walk 结构性成本 | 第二十轮归因「正确性收益以布局 pass 数量与每 pass 全树 walk 为代价」 | 需测量驱动定位 |
| display:contents 子级提升路径 | R3848 及穿透 ×3 | 需测量驱动定位 |
| aspect-ratio flex/grid 五连 | 回归窗口内 | 需测量驱动定位 |

## 4. 切片纪律

- 每步优化 = 一笔独立提交，附本地 A/B（同机低负载交错测量）证明净收益 > 0 且正确性测试/reftest 全绿。
- 净 0 或净负的尝试按净值纪律回退，不挂账硬扛。
- 专项期间 benchmarks 门禁恢复 GATE PASS 为「解阻达成」判据；专项完成判据 = 新基线经 auto-tighten 收紧至回归前水平带（本地 A/B ≈1.91ms ± 噪声带）。
- 与渲染流其他工作面（SVG2 intrinsic sizing 专项等）不重叠承诺，按 rally run-rules §9 工作面规则排期。

## 5. 进度记录

- 2026-09-07：立项（GB-20260907 批复①）。②同日执行完毕，三平台新基线 GATE PASS 113/113 NEW=0。下一步：渲染流下一轮 rally 轮从 §3 第一刀（R3858 预扫描覆盖缺口）开始测量驱动切片。
