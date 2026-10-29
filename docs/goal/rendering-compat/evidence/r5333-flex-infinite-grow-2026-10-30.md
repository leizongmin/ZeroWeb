# R5333 — 无穷 flex 因子语义（css-values-4 math × css-flexbox-1 §resolve-flexible-lengths）

- **日期**: 2026-10-30
- **驱动案**: `css/css-flexbox/flex-grow-009.html` ✗2.08% → **✓0.00%**
- **corpus**: 800F（R5332 交付态 801F；同树 A/B）→ **800F**（16590 案；**FIXED=1**，REGRESSED=0），通过率 15790/16590 = **95.2%**
- **域**: flex-grow/shrink 长手 math 求值 × taffy 无穷 flex 因子分配
- **代码**: `07f12bd82`（fix(flex): 无穷 flex 因子语义——flex-grow-009 ✓ calc(infinity) 吸收全部 free space）

## 定性

009 结构：100×100 flex 容器，item A `flex: calc(infinity) 0 0px`（绿）、item B
`flex: 1 0 0px`（红）。期望 ∞ 因子吸收全部 free space → 全绿；ZW 渲 **全红
10000px 恰 = 2.08%**（两 item target 全 0，容器红背景暴露）。

**两段接力缺陷**（inline style 观测测试定谳——文档内 `<style>` 不经
`compute_styles(&doc, &[])` 第二参数，首版观测误判全默认值，改 inline style 后链路干净）：

1. **style 层拒收**：flex-grow 长手解析 `value.parse::<f64>()` 对
   `calc(infinity)` 字符串直接失败 → 整条拒收回退初始 0。R3345 旧定性
   「负值/Infinity 非法」**证伪**：css-flexbox-1 值域是 `<number [0,∞]>`——
   只有负值非法，∞ 是合法上界（flex-grow-009 为 webkit 提交，三引擎均过）。
2. **taffy 分配 NaN**：即使 ∞ 到达 flex 因子，比例分配
   `free × (flex_grow / sum_flex_grow)` = `inf/inf` = **NaN**（IEEE）→ 全员
   target = basis + free×NaN → NaN → `.max(0.0)` 钳 0。

规范语义（css-values-4 §math-constants ∞ 相等比较 + css-flexbox
§resolve-flexible-lengths）：无穷因子吸收全部 free space——**多个 ∞ 项均分**，
有限因子项份额 = free×(f/∞) = 0。

## 实现

- **style-system `apply.rs`**：flex-grow / flex-shrink 两臂加 math 求值
  （`parse_math_function` + `eval_calc`，裸数字 parse 优先不变）——
  - flex-grow：放行 +∞（`v >= 0.0` 同时拒负值/-∞/NaN）；
  - flex-shrink：值域同含 ∞，但 taffy shrink 分配臂无 ∞ 语义
    （scaled factor ∞ 比例同 NaN），**维持拒 ∞**（挂账 taffy shrink ∞ 臂），
    普通 math 表达式放行。
- **taffy-local `flexbox.rs` 分配臂**（growing 分支）：sum 无穷时 ∞ 项均分
  free_space、有限项份额 0；kill-switch `ZW_FLEX_INFINITE_GROW=0` 回退
  （default-on）。
- vendored 格式纪律：rustfmt 全文件重排 flexbox.rs 516 行**已回退重放手工
  补丁**（22+/−3，保持上游格式——taffy-local 非格式化面）。

## 交付

- corpus 同树 A/B（基线 R5332 交付态 801F）→ **800F**，集合 diff 恰
  {−flex-grow-009}，REGRESSED=0；kill-switch =0 复现基线全红 10000px
  （双向可判）；trend 行 @ `07f12bd82` 已录。
- 择轻过程：R5332 挂账头号候选 flexbox-collapsed-item-horiz-002（1.59%）
  dump 图定性 = **多行 strut 迁移重排**（taffy 分行级中型改动，001/003 已过
  仅 002 单案受益）→ 转挂账换案；baseline-synthesis 聚簇 5 案核查 = 全族
  writing-mode（排除域）。

## 回归

+3 pin（`r5333_flex_infinite_grow_tests`）：

1. **驱动腿**：∞ grow 吸收全部 free space 100px、有限因子 0；
2. **对照腿**：普通 math `flex-grow: calc(1 + 3)` = 4 求值生效（4:1 分 80/20，
   非 infinity 特例）；
3. **守卫腿**：`flex-shrink: calc(infinity)` 维持拒收回退初始 1（等比收缩
   50/50，taffy shrink ∞ 臂现状守卫）。

## 门禁

- fmt 0；clippy `zero-layout-engine + zero-style-system --all-targets
  -D warnings` 0（guarded）。
- workspace 测试腿 46 binary 45 ok，唯一红 = zero-engine fetch 家族
  **9-10F 闪断**（既有维持项，3600P 本轮 vs 上轮 3599P，家族成员 9↔10 闪断）。
- 本地 725 套件 724/725（唯一红 = R5311 既有 float-shrink，零位移）。
- product-smoke welcome **15.68%** ≤ 20% 零位移 + struct PASS + legacy 42
  fixtures 0。
- **bench-gate 定向 `zero-layout-engine,zero-style-system`**：两次 ABORT
  （loadavg 19>12）+ 一次 INCONCLUSIVE（测量段被并行流污染，layout 4 指标
  假性超预算）+ 一次 GATE FAIL（同污染）后静窗 **GATE PASS 25 指标全在
  预算内**；**NEW=8 = zero-style-system 微基准首次进定向测量**
  （cascade×4/computed/compute_styles×2/selector_matching，非回归）。
- NEW=8 基线纳入**挂账**：`record-bench-baseline.sh` 收紧优先拒绝（报告混合
  page wall 时钟噪声 +20.65% 超 1.005 阈值，--relax 全局放宽会掩盖真实回归
  不采用）——R5325 NEW=7 同先例（基线 style 条目 = 0），NEW 条目留在基线外
  每轮重报不影响 GATE。

## 挂账

- **收口** flex-grow-009。
- **定性更新**：flexbox-collapsed-item-horiz-002 ✗1.59% = 多行 strut 迁移
  重排（taffy 分行级中型改动，非轻案——001/003 strut 基础已在
  `ZW_VC_NONFLEX_STRUT`，仅 002 行迁移单案受益）。
- **菜单/维持**（R5332 承接）：flexbox-collapsed-item-horiz-002（上述）、
  flex-wrap-006 / flexbox-flex-wrap-flexing-003（后者 writing-mode 排除域）、
  auto-margins-003 ✗2.20%、align-self-016 ✗4.17%、flex-svg-no-intrinsic-column-001
  ✗4.69%、flex-container-max/min-content-001 ✗7.33%/13.35%、definite-sizes-003/004
  各 ✗9.38%、fieldset-as-item-overflow ✗1.91%、flexbox_align-items-baseline
  ✗1.77%、css-flexbox-test1 ✗1.99%；015（AR transferred × % min-height）、
  taffy automatic min 短板、zero-webdriver 11F、inline-flex 内在宽 0 观察、
  zero-engine fetch 9-10F + details 闪断、R5327 fragment 臂结构域、safe/unsafe
  overflow-position 域、content-none-select-1 脆弱族、c44-003 残差、va 族余案、
  c548 待用户决策、bench NEW=8 基线纳入（静窗窄捕获或下游整编）、taffy
  shrink ∞ 臂。
- **下一步 R5334**：①800F 账本续扫择轻（避开 va 域、OOF 锚域、writing-mode
  域——本轮再实证 5 案聚簇落此域）②fieldset-as-item-overflow（1.91%）/
  flexbox_align-items-baseline（1.77%）分诊驱动择机 ③multicol 嵌套分片专项
  预研（8×1.04% 聚簇）。
