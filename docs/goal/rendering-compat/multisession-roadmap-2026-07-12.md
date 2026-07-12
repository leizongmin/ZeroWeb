# 渲染兼容性 rally — 多 session 架构 roadmap（plateau 后攻击计划）

**日期**: 2026-07-12（R1311-R1315 收敛后；R1346-R1348c 更新）
**状态**: plateau firm，aggregate chromium-oracle ~55%（DC-2~5 目标 ≥95% 远未达）
**目的**: 单 session clean lever 穷尽后，剩余 yield 全在多 session 架构 / C-dep。本文档汇总各 lever 的 blocker / 可行性 / yield / 攻击顺序，供后续 session 增量推进。

---

## R1346-R1348 更新（2026-07-12 同日续）

- **R1346 border-conflict §17.6.2.1 ruled out**：算法已实现（table_borders.rs），tiebreak
  + currentColor 真 bug 定位，但修复 0 flip（001a-001e diff 主导 = Ahem 文本 + swatch 图像，
  非 border 颜色）+ 001e 微退 → 全 revert。R1337 forward #2 关闭。
- **R1347 white-space -052 簇 = Phase A 阻塞**：LAYOUT_DUMP 实证 `<span inline>` 在 inline-block
  内被拉到 784px 满宽 + 块级堆叠（inline-box-model/R109 结构缺口），非 clean lever。
  pre-wrap-align 簇 = R1338 已闭（ch C-dep + font-wall）。R1337 forward #3 关闭。
- **★ R1348 empirical multicol 测量基础设施 LANDED（L3 解锁路径）**：R1344/R1345 推荐
  落地——product-oracle-shot.mjs CDP-connect（WSL2 兼容）+ 8 受控变体（distinct-color，
  `docs/goal/rendering-compat/empirical/multicol-section-height/`）+ chromium oracle PNG +
  PIL measured-data.txt。
  ★ **纠正 R1345 误读**：004a region c 实测 = **100/100 balanced**（非 R1345 称的「50px
  first-col」；004a 测试注释本身也不符 chromium 实际）。4 条 reliable 算法事实：
  (1) wrapper 显式 H ≠ 渲染高度（实测 < H）；(2) block balance 仅当 content > per-region
  available；(3) 末 region overflow（E：container 300 但 block3 到 y=700）；(4) 非末 region
  总 balance。**multicol 嵌套 spanner 从「blind net-negative 风险」升级为「empirical-grounded
  实现就绪」**。
  ★ **R1348b region-c（末 region）模型钉死**（J/K 变体，b=100 变 H）：末 region available
  A = H − (a+b) − 2×span。block3=100 over 2 cols：A=250(H450)→100/0（content≤A，填 col0 不
  balance）；A=150(H350)→75/25（spill，region=A/2）；A=50(H250)→50/50（content>A，forced
  balance，region=A）。**末 region：content>A 时 balance 成 content/N；否则按 A 顺序填 col0 再
  col1**。剩 per-region a/b available 分配（equal? greedy?）+ container 总高公式待最后钉。
  下一步：补 a/b 分配变体 → 据 empirical 模型重写 try_layout_nested_spanner（紧 gate + 全量
  chromium A/B）。详见 [`evidence/r1348-empirical-multicol-infra-groundtruth-2026-07-12.txt`](./evidence/r1348-empirical-multicol-infra-groundtruth-2026-07-12.txt)
  + `empirical/multicol-section-height/measured-data.txt`。
- **★ R1348c multicol 嵌套 spanner = LayoutNG balancing 移植（multi-session 定性）**：补 4
  基础变体（M/N = 0-span 单 block 基线；L/O = 1-span 双 block），共 **12 变体 PIL ground
  truth**。决定性发现：**container 渲染高度 = Σ region_heights + spans，无 closed-form**——
  每 region 高度是 chromium LayoutNG multicol balancing（binary search 列高 + fragmentation）
  的输出。列填充模式：定 definite-H + 无 span → content<<H 时 auto/sequential（M: 200/0），
  边界 balance（N: 100/100）；有 span → 各 region balance 到 content/N，末 region 进 binary
  search（L block200 A=300 → 125/75；O block200 A=200 → 100/100；同 block 不同 A 不同 split，
  无 clean formula）。**裁决：正确实现 = 移植 chromium LayoutNG balancing 算法（multi-session
  porting）**；R1341 content-driven +3 flip 是现实近似 yield，精确匹配（~40+ flip + 004a/004b
  残余 19%）须完整 balancing 移植。★ empirical 解除 R1344/R1345 understanding block（ground
  truth 可靠，纠正 R1345 误读），implementation 是 well-scoped porting 任务。详见
  [`evidence/r1348c-multicol-balancing-layoutng-port-2026-07-12.txt`](./evidence/r1348c-multicol-balancing-layoutng-port-2026-07-12.txt)
  + `empirical/multicol-section-height/measured-data.txt`（12 变体 + findings）。
- **R1348d LayoutNG balancing research（chromium 设计文档 + spec，源码 zread 受 auth 限）**：
  research agent 确认算法为**迭代 balancing**（初猜 content/N，按 overflow/space-shortage
  迭代精化，非 closed-form），证 R1348c 结论。★ **关键可实施 insight**：核心是 **column-fill
  auto vs balance 切换**——definite height + content fit 进更少列 → auto（顺序填 col0）；
  auto-height 或 content 溢出 → balance（迭代均分）。flow-thread 模型（单条带 layout → 列
  映射）。Phase 1 实现起点：port 迭代 balancing（input: content items + available + N →
  output: column_height + per-col 分配）+ auto/balance 切换，dead 模块 + 单测对 12 empirical
  点验证（D 100/0 auto / J 75/25 binary-search / K 50/50 balance-force / L 125/75）。chromium
  源码 `third_party/blink/renderer/core/layout/ng/multicol/ng_*_layout_algorithm.cc` 须手工读
  （zread 受 auth 限，下 session 可 git clone chromium 局部读或 WebSearch 精确片段）。

---

---

## 当前基线（chromium-confirmed）

已落地 clean wins（皆 chromium 真 yield，R1313 完整性确认）：
R1291 grid justify-content stretch +10 / R1293 relpos % top/bottom +1 / R1297 IFC skip OOF +1 /
R1298 empty inline width +1 / R1304 block min-content +5 / R1307 伪元素独立盒 +2 / R1308 fixed inset +2 /
R1311b br-in-p narrow gate +1。

plateau = aggregate ~55% + 上述累计。

---

## 剩余 lever 清单（按 yield × 可行性排序）

### L1. ★ C-dep 解锁：FreeType/Phase A 完整 font-stack（用户决策）— 最高杠杆
- **blocker**: CI 计费 6-target 全 failure + policy（R1088）；本 agent 无法单方面启用。
- **解锁**: font-wall residual（FreeType(ZW) vs FreeType+Skia(CHR) AA/subpixel）+ Phase A line-box
  metric（leaf-measure 高度 0.4px 过冲，R1311e inline-fold blocker）+ ::first-letter（436 案 lever）。
- **yield**: 数百案（最大簇）。welcome/morning.work 产品 smoke 亦受益。
- **可行性**: 待用户决策；技术路径 R1068 FreeType 已 default-on，须扩到完整 font-stack + Phase A。
- **行动**: ⚠️ 卡用户决策。agent 侧无可推进代码工作。

### L2. margin-collapse-clear §8.3.1 clearance-containment（6 案 17-35%，最高 diff in-scope 非 C-dep）
- **blocker**: ZW 当前定位已乱（margin-collapse-clear-012 实测 #following-sibling@132.6 在
  #clear-left@152.6 之前 = clearance 未推后续兄弟）+ §8.3.1 containment math intricate（012 的
  (140−40) cleared-top-consumed 规则，R1314 deep-research）。taffy 0.7 不建模 collapse-through。
- **规则**（R1314）: clearance applied（clear_bottom > hypothetical）→ cleared 子起 collapse-through
  链不与 parent bottom 折叠，留 parent 内。per-case 子规则：015（clear+第一子 top adjoining 仍折叠）、
  016（无前置 float 无 clearance 正常折叠）。
- **可行性**: 须先修 clearance 推后续兄弟定位，再加 containment。R1047 postprocess net-negative
  precedent → 须 layout-time（float_positioning.rs clearance 分支 + backfill 协同），高回归风险
  （margin-collapse pervasive）。非单 round，须设计 + 增量（012/013→014/015/016）+ 全量 chromium A/B。
- **yield**: 6 案（012-016, 157），17-35% each。
- **行动**: 起 RFC（collapse-through 链追踪 + clearance-gate + 推兄弟定位 + parent-height containment），
  首片 012/013。

### L3. column-span:all parse + sizing + layout（multicol-span-all-* 簇 5+ 案 12-28%）
- **blocker**: ZW 未解析 column-span（R1020 用「无元素孙」proxy，text-only spanner 误触 N× sizing）。
  须三步：① parse column-span:all；② spanner-aware intrinsic sizing（spanner 宽 = 容器宽，非 N×）；
  ③ column-span layout（spanner 跨全列，拆 column flow）。
- **可行性**: ① additive 安全但无 layout 无 yield（推测基建）；②③ 是难点（spanner 拆列流动 = multicol
  fragmentation 谱系，R1292 已 ruled out fragmentation）。整体多 session。
- **yield**: 5+ 案（multicol-span-all-children-height/rule/span-float/nested 等），12-28% each。
- **行动**: ① parse（CSS 支持补全，替换 R1020 proxy 减误触）；②③ 后续。

### L4. vertical-mode R1043（block-flow 方向 + inline-flow 推进）
- **blocker**: ZW IFC 对 vertical-lr/rl 文本水平布局（非垂直），双层缺口（R1043 block-flow + R1050
  inline-flow）。container_width=0 致每字符一列横向排列（R1052）。
- **可行性**: 耦合系统（block-flow + inline-flow + line-height vertical + emphasis 须四层同修），
  单修 net-negative 三证（R1047/R1050/R1052）。多 session 大架构。
- **yield**: vertical-rl/lr 簇（writing-modes 等），但 R164 证 4 轮「正确 CSS」均 net-negative（同源
  REF 怪异），真实 yield 不确定。
- **行动**: 须完整 vertical IFC，低优先（yield 不确定）。

### L5. collapsed-border paint-phase separation（collapsed-border-paint-phase-002 等）
- **blocker**: ZW painter 单循环递归 paint 整棵 child subtree 无 block/inline descendant phase 分离
  （R1296）。collapsed border 卡在 block descendant bg 与 inline descendant 之间。
- **可行性**: 须 painter 深重构（phase 分离）= 多 session，invasive + border-collapse 广回归风险。
- **yield**: ~1-2 案（+paint 正确性）。
- **行动**: 低优先（yield 小，风险高）。

---

## 推荐攻击顺序

1. **L1 C-dep**（若用户解锁）= 最高 yield，数百案。等用户决策。
2. **L2 margin-collapse-clear §8.3.1** = 最高 diff in-scope 非 C-dep（6 案 17-35%），须 RFC + 增量。
3. **L3 column-span:all** = 次高 yield（5+ 案），①parse 可独立起步。
4. **L4/L5** = 低优先（yield 不确定 / 风险高）。

---

## 单 session clean lever 状态

跨全 14 dir（CSS2 visudet/normal-flow/floats/visuren/linebox + css-flexbox/grid/multicol/position/
tables/text/text-decor/writing-modes/fonts/values）**conclusive 穷尽**（R1305/R1306/R1310/R1312/R1315
五证）。残余 high-diff 全多 session 结构性（上述 L2-L5）；near-pass 带（1-3%）全 font-wall 指令文本
（R1155，勿挖）；high-diff 无文本案全多 session。单 session clean lever 稀有但非绝对（R1307/R1308/
R1311b tail 证），续 render+PIL 偶有 +1-2，但期望低。

## 方法论约束（R1311c/d/e 提炼）

- **布局路径变更一律 chromium per-case oracle A/B**（ORACLE_DUMP_ALL），self-source / oracle pass-count
  阈值敏感可误导（linebox self-source +5 实 chromium -6；borderline 0.99→1.01 伪回归）。
- **leaf-measure 路径**有 Phase A 0.4px 高度过冲，经全区域错位放大成大 diff（inline-fold blocker）。
- **postprocess margin 调整 net-negative**（R1047），margin 类改须 layout-time/converter 层。
