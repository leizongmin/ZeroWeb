# 渲染兼容性 rally — 多 session 架构 roadmap（plateau 后攻击计划）

**日期**: 2026-07-12（R1311-R1315 收敛后）
**状态**: plateau firm，aggregate chromium-oracle ~55%（DC-2~5 目标 ≥95% 远未达）
**目的**: 单 session clean lever 穷尽后，剩余 yield 全在多 session 架构 / C-dep。本文档汇总各 lever 的 blocker / 可行性 / yield / 攻击顺序，供后续 session 增量推进。

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
