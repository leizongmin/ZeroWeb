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
- 2026-09-10（R4214-P 测量轮，0 net code）：代码复核确认 §3 第一刀（R3858 预扫描覆盖缺口）**已在 R3858 自身执行完毕**（engine.rs:860 OPTIMIZATION 注释：无 positioned 页面全树 LayoutBox 指针追踪 ~0.7ms → contiguous HashMap 扫描 ~20µs）。`ZW_PROFILE_LAYOUT` 分段计时探针（临时，已回退）产出 **block_layout_1000_elements pass 级成本表**（HEAD ~2.6ms @ load~1.3，同代码环境带 2.43-3.03ms ±20%）：**tree build/converter（S1）~0.92ms（35%，最大单杠杆）** ＞ adjust_float_positions ~0.29ms（结构性全树 walk：flow_bottom 独立追踪 + y 重写；**非** per-node env::var 所致）＞ intrinsic pass2 ~0.27ms ≈ taffy pass1 ~0.25ms ≈ compute_final_inline_layouts ~0.2ms ＞ extract ~0.17ms；abspos 家族已有预扫描仅 ~0.11ms。**两项试验净负/净零按纪律回退**：①form pass DOM 预扫描门——净零（原 0.25ms 读数为标记间隙误归因，pass 本身 ~15µs）；②float walk kill-switch env::var 提升（13 旗标函数口快照透传）——交替 A/B 一致 **+0.27ms 净负**（env::var 兼作 LLVM 优化栅栏，提升反劣化代码布局），两坑根因与探针规范入 [`docs/learnings/performance/2026-09/2026-09-10-layout-pass-profile-env-hoist-net-negative.md`](../../learnings/performance/2026-09/2026-09-10-layout-pass-profile-env-hoist-net-negative.md)。**下一刀定向**：S1 converter（tree.rs `build_layout_tree_with_r109`，1000 节点 0.92ms = 460ns/节点，候选：dom_to_taffy 反向 map 构建、ComputedStyle 逐字段搬运热点、R109 接线分支密度）；float walk 结构性重写列为第二候选（高风险域，需切片设计）。本机 ambient 负载带（±20%）下 gate FAIL 判读须先静默窗口复测，避免把噪声当回归（ZRG-2026-09-09-01 TREND-WATCH 的升立案条件据此修正：静默窗口 + 交替 A/B 双确认）。
- 2026-09-10（R4215-F 切片 1 落地）：**sizing 子 pass 共享预扫描门**（engine.rs，+84/−14）——pass2 段三个纯样式驱动全树 walk（`apply_intrinsic_content_sizing` / `apply_indefinite_percent_height_to_auto` / `resolve_percentage_padding`）前置一次 contiguous `styles.values()` 预扫描（~20µs，同 R3858 量级），派生三目标形态位标（宽轴 content 关键字 ∪ auto+float（R1015 臂）／height Percentage∪Stretch（R4086 Stretch 臂）／任意 margin/padding Percentage），无目标即整 pass 跳过（changed 恒 false 语义不变）。**A/B（同机交错，含 R4214 探针法教训：mark 成对贴语句 + 门 fire 实证 + 负结果回退）**：block −2.5%（quiet）～ −6~14%（负载带）、incremental −0.9%、flex ±0；定向 bench-gate FAIL 4-5 → 2（flex/grid 翻 PASS；block/incremental 负载下仍边缘）。**首版门缺 Stretch 臂被 corpus 当轮抓获**（block-height-006/stretch-anonymous-block-001/stretch-quirk-002 三案翻红，−3 漂移）——域加宽后 ZERO DRIFT 复位 14681。**验证**：make test 67 套件全绿（service_worker flake 隔离复跑惯例）、make reftest 687/687、corpus 14681 零漂移、product-smoke 23.49% 同值 struct PASS、legacy 0 struct FAIL、fmt、clippy -D warnings。**下一刀**：pass2 余下未门 walk（gather_replaced_html_attr_intrinsic / AR 家族 ×4 / abspos shrink / vertical / flex-cross，残 ~0.12ms）+ postprocess 段同模式扫荡（float walk ~0.29ms 需切片设计——结构性，非门控）。
- 2026-09-10（R4216-F 切片 2 落地）：**pass2 余下 walk 扫荡**（engine.rs）——R4215 预扫描扩展三新位标并门控六个子 pass：AR 家族四 pass（flex/grid item、container cross、flexed-main cross，域 = styles aspect_ratio/aspect_ratio_auto ∪ ratio-only img 输入 ∪ **替换元素 HTML width/height 属性**（corpus 当轮抓获的第三源：flex-aspect-ratio-img-row-006/flex-aspect-ratio-028 翻红后以 DOM 多 tag DFS 兜底，is_replaced_element_tag 同表））+ abspos shrink（position Absolute|Fixed）+ vertical sizing（任一非 HorizontalTb）。**A/B（同机交错两轮一致）**：block −2.3~−3.6%、incremental −5.5~−9.6%（叠加切片 1 后 block quiet ≈2.20ms，R4213 时代 ~2.43ms）。**corpus 全量 14681 零漂移**（首版 AR 域缺 attr 源被抓获 −2，加宽后逐案 diff 为空——连续两切片同型缺口均被 corpus 门禁当轮拦截，全量 A/B 对 pass 门控类修改的必要性二次实证）。make test 67 套件全绿、make reftest 687/687、product-smoke 23.49% 逐字节同值 struct PASS、legacy 0 struct FAIL、fmt、clippy -D warnings。定向 bench-gate 两跑（load 2.3-2.8，并行流 zero_integration_tests 137% CPU 共租）5 FAIL——负载污染读数不作数，依 R4214 修正的判读条件以同条件交错 A/B 为准；静默窗口复测留待下轮巡检。**剩余**：pass2 仅余 gather_replaced_html_attr_intrinsic 未门（DOM DFS≈LayoutBox walk 无净收益，维持现状）+ S1 converter（~0.87ms 最大单杠杆，需采样 profiler 或 tree.rs 分段探针）+ float walk 结构性重写（~0.29ms，切片设计）。