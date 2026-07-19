# 渲染兼容性 rally — 多 session 架构 roadmap（plateau 后攻击计划）

**日期**: 2026-07-12（R1311-R1315 收敛后；R1346-R1351 更新）；**R1496 校准 2026-07-16**
**状态**: plateau firm，aggregate chromium-oracle ~55%（DC-2~5 目标 ≥95% 远未达）
**目的**: 单 session clean lever 穷尽后，剩余 yield 全在多 session 架构 / C-dep。本文档汇总各 lever 的 blocker / 可行性 / yield / 攻击顺序，供后续 session 增量推进。

---

## R1496 校准更新（2026-07-16，roadmap 过时纠正）

> 本文档下方「剩余 lever 清单」写于 R1311-R1351（2026-07-12/13）。R1317-R1495 期间多 lever 已兑现，
> 下方原描述过时。**读下方各 lever 前先读本节**，避免重复尝试已完成工作。

- **L2 margin-collapse-clear §8.3.1 = ✅ DONE**（R1317-R1332 收口）：012/013/014/015/016/017
  全 < 1% PASS（见上方 R1349 更新 line 15-17）。下方「L2」节描述的 blocker 已不成立。
- **L3 column-span:all = ①② DONE，③ open**：
  - ① parse：`column-span` 已解析入 `ComputedStyle.column_span`（`apply_advanced.rs:778`，
    `ColumnSpanComputedValue::None|All`）。
  - ② spanner-aware intrinsic sizing：**R1431 LANDED**（`intrinsic_sizing.rs:247`，替 R1020 proxy；
    区分 spanner/non-spanner block 子，6 case 验证，doc
    `spanner-aware-multicol-intrinsic-sizing.md`）。
  - ③ column-span **layout / fragmentation**（spanner 拆列流）= 仍 open，属 multicol balancing 谱系
    （见下方「当前真实 lever」multicol Phase 2）。
- **R1489-R1495 期间新增进展（DC-13 结构门 + R1492 真 bug）**：DC-13 product-smoke 结构自动检查
  （sibling-overlap R1489 + element-count R1490 + line-count R1491）+ wintertc/morning 入门禁；
  **R1492 真 R109 bug 发现并修**（plain block + inline 元素子 → taffy 仅按 inline 子定块高 → compute_final
  长高后后续兄弟重叠）→ **R1495 post-process `shift_siblings_after_ifc_grow` LANDED**（default-on，
  oracle A/B 五目录 NET 0；`inside_multicol` 祖先 gate 排除 multicol 子树）。详见 master.md R1489-R1495。
- **Phase-A-authoritative-storage 谱系 = 已关闭（R1526）**：`store_inline_layout_results`
  （inline_finalization.rs:146，`#[allow(dead_code)]`）wiring 非 yield lever——(1) 函数已 dead，
  存储职责被 `compute_final_inline_layouts`（line 521/933）inline 取代（R1379）；(2) broad
  authoritative-storage 机制（paint 经 `use_stored` 复用 layout 行盒不重跑 IFC）经 R1487 env-gate
  A/B 决定性证伪（normal-flow NET -7 revert）——layout 行断 `estimate_char_width` 与 chromium 分歧
  **大于** paint Path B fontdue 重跑，强制 paint 用 layout 结果更差；(3) narrow ascent/baseline
  override 变体 net-negative（R1194 / R1206 NET -22 / R1208）。R630/R632/R1280 yield 皆 mechanism A
  narrow override/坐标修，非 mechanism B broad storage。**勿再以 wiring 本函数 / Phase-A-broad-storage
  / ascent 单点 override 为 lever**。详见 master.md R1526。
- **font-wall 主指标阻塞 = 仍 user-gated**（L1 C-dep）：R1068 FreeType 已 default-on，剩完整 font-stack
  （HarfBuzz/Skia coherence）+ Phase A line-box metric + ::first-letter，受 CI 计费 6-target 全失败 +
  policy 约束，agent 无法单方面推进。

### 当前真实 lever（R1560 更新视角，按 yield × 可行性）

1. **font-wall = 光栅化方向已穷尽，剩 layout/metric coherence（Phase A）**。**R1560 决定性证伪**：real Skia（skia-safe 0.80，chromium 实际光栅器）over FreeType hinted 轮廓 = writing-modes net+1（噪声）/ welcome −0.09pp / **css-text net−24**（font-wall 文本簇真回归）。光栅化 coverage 算法 swap 非 font-wall unlock。**光栅化 C-dep 全角度穷尽**（metric DEAD R1090/R1095/R1160/R1206 / LoadFlag R1069 / gamma R1553 / tiny-skia R1555 / **real Skia R1560**）→ 永久移出 lever 清单。残余 font-wall real-yield 唯一路径 = **Phase A IFC 统一（layout/metric coherence，estimate-vs-fontdue，line-height/baseline）**，R125-R213 6 轮 deadlock 须 fresh architectural attempt。HarfBuzz shaping slice 理论低优先（rustybuzz 已用，shaping 差主影响字形选择非 coverage）。
2. **multicol column-breaking R1035**（multicol nested-spanner wrapper 工作已 R1352-R1361 完成：
   R1359 FLIP 004a PASS、R1360 004b→1.51%；**R1535（2026-07-16）再推导末列 bg 闭式公式
   `last_h = capped_h − spans_total`（逐像素 PIL，替 capped_h − c），004b 1.51%→0.55% FLIP，
   css-multicol NET +1 零回归**；R1350 dormant `multicol_balancing.rs` 功能已被 R1357-R1360
   region_available 路径覆盖，无须再 wire）。**残余 multicol 前沿 = column-breaking**
   （004b 已 PASS，残余=block 文本 font-wall；multicol-breaking-004/nobackground-004 ~9%）：
   overflow block 须跨列拆分（block2-col1=50 sequential vs 100 balance），R1352-R1361 九轮深调
   后定性 deferred——属 R1035 multi-session 高风险（影响 all multicol balance），须 RFC + 紧 gate +
   全量 A/B。span-all-children-height 其他成员（002/005/007/008/009/012/013）各为不同结构
   （auto-height / column-count:1 / border 碎片化），残余非 004b-class 闭式可解，均 multi-session。
   其余 multicol top fail = font-wall（column-rule subpixel / multicol-basic Ahem 文本）/ @media print
   OOS / JS-dynamic / nested+margin 结构性 / form-control，均非 clean lever。css-multicol 178/452 (39.4%)。
3. **DC-11 host-layer** — overflow:scroll 真滚动容器（交互滚动，静态 clip 已工作）/ position:sticky 动态
   （静态已按 relative 渲染）/ scroll-snap。需 browser/display 验证环境，非纯 headless reftest 可验。
   注：css-overflow / position:sticky 在当前 oracle corpus 无独立 shot（0 case），无法 reftest A/B。
4. **DC-13 扩展**（diminishing）：narrow viewport 结构检查、更多产品 fixture struct-check（morning inline
   badge 受 R109 限 0 box，须先 R109/Phase A）。

### 单 session clean lever = conclusive 穷尽（重申）

跨全 14 dir 五证（R1305/R1306/R1310/R1312/R1315）+ R1481-R1488 八连 0-yield。R1489-R1495 的进展全在
**DC-13 结构门 + struct-check 抓 R1492-class 真 layout bug**（非 oracle 扫描），续此路径偶有真 yield
（struct-check on 新 fixture 抓 overlap 类 bug），但期望低、非主指标路径。

---

## R1349 更新（2026-07-12，multicol 源码 blocked + L2 floats-clear 审计）

- **multicol Phase 1 源码访问 blocked（4 路）**：mcp__zread（仅 design doc 摘要）/
  raw.githubusercontent（404）/ googlesource format=TEXT（fetch 损坏）/ R1348d research agent
  （auth-limited）。Phase 1 须先解源码访问（git sparse-clone chromium 或 source.chromium.org
  web 读）或据 12 empirical 点逆向迭代 balancer。
- **★ L2 §8.3.1 工作已兑现**：margin-collapse-clear **012/013/014/015/016/017 全 < 1% PASS**
  （012/013/014=0.61, 015=0.69, 016=0.87, 017=0.83）——R1317-R1332 clearance positioning +
  containment + sibling-shift + 016 has-floats gate 收口主簇。
- **L2 floats-clear 残余（214 案 94 pass 43.9%）分类**：
  (a) **adjoining-float 簇（4 案 4.79-9.98%）= nested-float clearance 结构缺口**——float 嵌在
      wrapper（container > wrapper > float），ZW `has_active_float_context`/float_geometries 仅
      收集**直接** float 子，嵌套 float 不被 clearance 见 → clear:left 失效，margin-top:400 被
      当实空间涂（实测 39900 红 px）。修须递归收集非 BFC 后代 float bottom（结构改动，影响全
      float 案，高回归风险，非单 session）。
  (b) **float-width 簇（007/009-012 ~4.64%, 008/009 6.07%）= font-wall**——PIL 实测 green 像素
      ZW=CHR=10800 完全一致（shrink-to-fit 几何正确），diff = 黑 Ahem 文本 stripe。非 lever。
  (c) margin-collapse-12x/16x（5-7%）= float+clear+border 结构性 clearance 变体。
- **裁决**：R1346/R1347/R1348c/R1349 四证 clean single-session lever 穷尽。残余全 multi-session
  架构（multicol balancing 移植 / nested-float clearance / Phase A / bidi）或 C-dep（user-blocked）。
  详见 [`evidence/r1349-l2-floats-clear-audit-2026-07-12.txt`](./evidence/r1349-l2-floats-clear-audit-2026-07-12.txt)。

---

## R1350 更新（2026-07-12，★ multicol balancing 模型突破 + Phase 1 模块 LANDED）

- **★ multicol balancing 模型推导成功（11/12 验证 + 004a）**：重新据 12 empirical 点推导，
  得 closed-form per-region 模型（R1348c 曾判定「无 closed-form」，本轮纠正）：
  - 非末 region：列高 = `ceil(content/N)`，内容均匀分片（A/D/F 的 a/b）。
  - 末 region：`content > available` → forced balance（列高 `ceil(content/N)`，溢出，A/K/E/004a）；
    否则列高 = `available/N`，col0=min(content,h)、col1=min(rem,h)（D/J/O）。
  - **11/12 匹配**（仅 1-span 末区域 L 型 C=200/A=300 → 模型 150/50 vs chromium 125/75，
    LayoutNG binary-search split，未解 outlier，测试标 documented）。**004a（2-span 目标案）匹配**。
- **★ R1350 Phase 1 dormant 模块 LANDED（commit 4e9edd1a）**：
  `crates/layout-engine/src/multicol_balancing.rs` = `RegionBalance` +
  `balance_nonlast_region` + `balance_last_region` + 4 单测（7-case empirical 验证 + 非末
  content/N + num_cols=0 守卫 + L-outlier documented）。`#[allow(dead_code)]` Phase 2 wiring
  待定（R885 font-bridge 模式）。fmt/clippy `-D warnings`/make test 全绿（+4 测试，零回归）。
- **未解（Phase 2 阻塞）**：容器总高 overflow 案（emergent LayoutNG row-height；简单 overflow
  ≈ H−spans 如 004a=350/B=250，但复杂案 variant E a/b 自身溢出不符）。+ painter bg 分段
  （004a 残余 19% = pink bg 未按 region 分布，R1342 net-negative precedent）。
- **forward**：Phase 2 = (1) 钉死 container 高（须 LayoutNG 源码或更多变体）；(2) wire
  `balance_last_region` 进 try_layout_nested_spanner（替 / 补 R1035 multirow，紧 gate +
  全量 A/B）；(3) painter bg per-region（依赖 region geometry）。目标兑现 ~40+ flip 潜力。

---

## R1351 更新（2026-07-13，multicol 004a 残余诊断 + painter-core 实验 net -1 REVERTED）

- **★ 004a 残余精确诊断 = R1343 painter-core**：PIL 实测 ZW blocks（yellow）单列 x[8,107]
  vs CHR 双列 x[8,315]。根因：blocks 是 breaking 子（200px 在 ~150px region 须跨列拆分），
  position_multicol_children 在 synthetic 上**正确**设了 column_span_offsets，但 R1341 backfill
  只复制 x/y 不复制 cso + painter column loop（`if is_multicol`）仅对 multicol 容器触发 →
  depth-2 breaking 子只绘首片段（col0）。
- **★ painter-core 实验（ATTEMPTED，net -1 flip，REVERTED）**：实现 R1343 三部分（backfill
  复制 cso + painter `paint_as_multicol = is_multicol || any_child_has_cso` + normal paint 排除），
  css-multicol 452 案 A/B：004a 19.01→**12.76**%（-6.25pp，block 双列分布**修对** yellow 59272≈CHR
  59255）/ 004b 18.84→14.68%（-4.16pp）/ remove-transform-descendant-becomes-spanner 0.63→1.84%
 （+1.21pp，pass→fail，deep-nesting spanner regression）/ **NET 155→154 (-1 flip)** → 全 revert。
- **★ painter column-loop 扩展 + cso 传播验证工作**（004a block 双列分布对）= R1343 实证突破。
  残余 = pink bg 容器高（ZW 80000 vs CHR 41400，container height 未解）致 004a/004b 未 flip。
- **forward（重试须三件套，net ≥ 0 才留）**：(1) 用 `is_nested_spanner_wrapper` flag 替
  `any_child_has_cso` gate（精确锁 R1341 wrapper，排除 deep-nesting normal-multicol 路径，解
  regression）；(2) 修 container height（pink bg 残余）使 004a/004b flip；(3) 全量 A/B 守 net ≥ 0。
  详见 [`evidence/r1351-004a-painter-core-diagnosis-2026-07-13.txt`](./evidence/r1351-004a-painter-core-diagnosis-2026-07-13.txt)。
- css-grid fresh 扫描（49 案 55.1%）：synthesized-baseline 簇（~16%）= vertical-modes（R109-
  blocked）；table-grid-item-dynamic（JS+table sizing）；无 clean single-session lever。

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

### L1. ~~★ C-dep 解锁：FreeType/Phase A 完整 font-stack~~ — 光栅化 slice 已 R1560 证伪，剩 Phase A layout/metric
- **R1560 决定性更新**：原「font-wall = FreeType(ZW) vs FreeType+Skia(CHR) AA/subpixel 差，须 Skia C-dep 对齐」**已被 real Skia S2 证伪**——skia-safe 0.80（chromium 实际光栅器）over FreeType 轮廓 = css-text **net−24**（真回归）/ writing-modes net+1 / welcome −0.09pp。**光栅化 coverage 算法非 font-wall unlock**，永久移出 lever。
- **blocker（残余）**: font-wall 真根因 = **layout/metric coherence（Phase A IFC 统一 / estimate-vs-fontdue / line-height-baseline 定位）**，R125-R213 6 轮 deadlock，须 fresh architectural attempt，非 C-dep 可解。
- **解锁**: Phase A line-box metric（~~leaf-measure 高度 0.4px 过冲，R1311e inline-fold blocker~~ → **R1779 REFUTED**：empirical probe 证当前 leaf-measure 对非-Ahem 16px 返 18.624 = chromium 真值，零过冲；ZW_LEAF_MEASURE_FIX no-op；general inline-fold 独立 net-negative 双证 R1311c/R1492-R1494）+ ::first-letter（436 案 lever，亦 font-metric 依赖）。
- **yield**: 数百案（最大簇，但路径从「光栅化 C-dep」改为「layout/metric 架构」）。
- **可行性**: Phase A 多 session 架构（IFC 统一），非单 round；HarfBuzz shaping slice 低优先。
- **行动**: 光栅化方向（Skia/FreeType/tiny-skia/gamma/LoadFlag/metric-swap）全角度穷尽，勿再试；转 Phase A IFC-result-reuse fresh attempt，或他 lever（multicol/content-list），或接受 plateau。

### L2. margin-collapse-clear §8.3.1 clearance-containment（6 案 17-35%，最高 diff in-scope 非 C-dep）

> **⚠ R1770 STALE 复测（2026-07-20）**：本节「6 案 17-35%」基于 pre-R1393 快照。R1393 adjoining-float clearance LANDED（2026-07-16，adjoining-float-before-clearance 9.98→0.63）已解 012/013/014 主几何——fresh reftest-oracle 实测现仅 **016=17.19%** 残余 high-diff，012/013/014=0.56-0.60%（font-wall `<p>` floor，R1155 勿挖），余 <1.2%。**L2 现仅 1 案真 lever（016）**，yield 估计须下调；016 根因 = `float_positioning.rs:1329` containment gate 漏 no-float clear-only 容器（无 float → containment SKIP → taffy 把 collapsed-through mt 当 content）。详见 master.md R1770。

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
- **leaf-measure 路径**~~有 Phase A 0.4px 高度过冲，经全区域错位放大成大 diff（inline-fold blocker）~~ → **R1779 REFUTED**：empirical probe 证当前 leaf-measure 对非-Ahem 16px 返 18.624 = 16×1.164 = chromium 真值，零过冲；该 blocker 已不存在，general inline-fold 仍独立 net-negative（R1311c/R1492-R1494 双证）。
- **postprocess margin 调整 net-negative**（R1047），margin 类改须 layout-time/converter 层。
