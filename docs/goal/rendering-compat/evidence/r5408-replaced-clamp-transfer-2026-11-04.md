# R5408 — 替换盒 min/max-height 钳后宽度回传（R2428 both-auto 注入域）

日期：2026-11-04
驱动案：`css/css-sizing/intrinsic-percent-replaced-017.html`（✗1.21% → ✓0.00%）
状态：已完成（corpus 真态 719F → 718F，FIXED=1，REGRESSED=∅）

## 1. 缘起与归因链

控制面 R5408②：017 前置探测（LayoutBox 替换元素 ratio 字段评估 walk 层可达性）。
案面：flex column `.outer` 100×100（align-items:center）> `.inner(max-height:100%)`
> `img`（解码固有 200×200 + `max-height:100%`）——断言 inner div 的 used main
size（高）作为 img `max-height` 的 % 基准，img 经 aspect-ratio 把钳后高传递到
inline 轴决定固有 inline 尺寸贡献，从而决定 inner 的 used cross size（宽）；
ref = 满绿 100px 方块（[`ref-filled-green-100px-square.xht`]）。

- **归因（上轮在途稿完成，本轮 A/B 复核定谳）**：R2428 both-auto 注入域
  （父非 flex/grid）把替换盒 `size.width/height` 写成固有值 definite、**不设
  aspect_ratio**（R325：taffy 强制比例会覆盖显式值）。`max-height` 钳住 used
  height 后 taffy 不按钳后高回传宽 → img 渲 200×100（宽保持注入固有 200），
  红底外溢 ✗1.21%。规范依据 = CSS2 §10.4 constraint-violation 表 +
  css-sizing-4 §4.1 transferred size 以 used size 为 basis。
- **断点定位**：非缺公式、非缺 ratio 数据（解码比已在 `img_intrinsic_ratios`），
  缺「钳后 used height → taffy width」的回传臂。

## 2. 修复（walk 臂 + 双回归门，单 kill-switch）

kill-switch `ZW_REPLACED_CLAMP_TRANSFER=0`（default-on，OnceLock 进程级快照，
`intrinsic_sizing.rs::replaced_clamp_transfer_enabled`，R5318 同口径）。

臂落点：`engine/sizing.rs::apply_flex_aspect_ratio_item_size` 的 walk（R5407
同一 sizing 期 walk，签名加 `intrinsic_sizes` 参数）。触发门全链：

1. 替换叶（`is_replaced` + children 空 + 非 abspos/fixed）+ 水平书写模式
   （walk 既有入口门，R4076 同门：vertical 主/交叉轴互换另案）；
2. 父 display 非 flex/grid（R2428 both-auto 注入域限定——flex/grid item 尺寸
   由布局期算法接管，非本臂域）；
3. CSS `width`/`height` 双 Auto（注入域特征）；
4. `has_real_min_max_height`：min/max-height 存在真实约束声明——Auto/
   MinContent/MaxContent/FitContent/Stretch 非约束；**`Px(INFINITY)` = none 的
   内部表示**（default_impl.rs:36，resolve_sizing_definite_real_length 同口径）
   排除。% 声明的解析结果已含于首趟 laid 高，此处只判「有声明」（CSS2 §10.4
   约束表触发条件）；
5. 固有尺寸取自**清洗 map** `intrinsic_for_clamp`（engine.rs 构建）：剔除
   ratio/no-ratio 信号键——img 路径 `build_img_intrinsic_all` 不做
   ratios/no_ratio 互斥（object 路径 continue 纪律 img 缺席），no-ratio/ratio-only
   SVG 的 bogus 解码尺寸会同时落 sizes，被当可传比错传。R695 % 高 pass 输入
   （`intrinsic_for_r695`）保持原样不受清洗影响；
6. `iw/ih > 0.5` + laid 高偏离固有高 >0.5（真发生钳制；`b.height` 已含
   min/max 钳结果）+ 目标宽偏离现宽 >0.5 → 写 `st.size.width = b.height ×
   (iw/ih)` + mark_dirty。

**双回归门（上轮 corpus A/B 定谳后追加，均保留为常驻测试锚）**：

- **no-ratio 门**（engine.rs `ratio_signal_ids`）：visudet
  replaced-elements-min/max-* 六案 ON/OFF A/B——no-ratio SVG（sizes 300×150
  bogus + no_ratio 信号共存）曾被按比错传 150×150。无固有比者 CSS2 §10.4
  本就不得传。
- **none 门**（`Px(INFINITY)`）：css-sizing aspect-ratio replaced-element-045
  ——none 内部表示漏判曾把**全部** both-auto 替换盒当有 max-height 约束，
  045 canvas（attr 40×10）被按 laid_h×(40/10) 错写 400×400（corpus 3×3 定谳）。

无钳（laid == 固有）不触发——014/dynamic-012 传递域逐案零扰动（家族 A/B 实证）。

## 3. 常驻测试

`engine/tests/r5408_replaced_clamp_transfer_tests.rs` 5 针：

- `r5408_max_height_clamp_transfers_width`（★load-bearing，017 根因锚）：
  钳后高 100 × 比率 1 → 宽 100、高守钳 100、inner cross 随传 100；
- `r5408_no_clamp_stays_intrinsic`（门）：无 min/max-height 声明不触发，
  both-auto 保持固有 200×200；
- `r5408_definite_width_not_overridden`（门）：CSS width definite 非注入域，
  本臂不触碰；definite 宽 + max-height 钳域存在既有独立覆盖（taffy ratio
  强制，chromium 应保持 150×100 per CSS2 §10.4），与本臂无关、另案挂账；
- `r5408_no_ratio_signal_suppresses_transfer`（回归锚，no-ratio 门）：
  min-height:300 钳高后宽保持注入 150、不按 bogus 比放大成 300；
- `r5408_max_height_none_is_not_a_constraint`（回归锚，none 门）：045 形态
  broken 门下会写 280，门修后臂惰性保持 160。

## 4. 验证（本轮完成段）

- 单测：layout-engine lib **2391/2391**（2386 + 5 新）；新针 5/5。
- 家族 A/B（43 案，BASE = HEAD 48b9e82be / FIX = 本稿，双二进制 grep
  kill-switch 串 0/1 命中核验身份）：BASE 5F（017 ✗1.21% + 011 8.33% +
  030.tentative 6.25% + dynamic-009/010 2.08%）→ FIX 4F——**017 ✓0.00%**
  翻绿，残留 4 案数值逐案不变（竖排 AR 传递域 ×2 / 动态改高域 ×2，在册
  分层挂账），族内零回归。
- kill-switch OFF：017 精确回 ✗1.21%（与 BASE 一字不差），家族其余逐案不变。
- 017 双侧 3×3：FIX ✓0.00%×3 / BASE ✗1.21%×3——稳定真修复。
- corpus 全量 A/B（16594 案）：BASE 窗口 725F / FIX 窗口 720F（另两窗
  722F/724F，mover 全为 table-anonymous-objects-127/128/129/130/153/154
  在册抖动族）。FIXED 候选 {017, insert-block-in-blocks-middle-001/003,
  table-anon-152/154}、REGRESSED 候选 ∅——4 个非修复 mover 双侧 3×3
  隔离全绿洗清（guidelines #35，与 R5396/R5398/R5400/R5404 同族同判）。
  **真态 719F → 718F，FIXED={017}，REGRESSED=∅**（账本连续性口径：
  R5407 真态 719F − 017）。
- 本地集 **725/725**；product-smoke 全腿 struct PASS（含 320/375 窄屏门）；
  product-smoke-legacy **struct-check failures: 0**。
- fmt ✓ / guarded-clippy 全工作区 EXIT 0 / bench-gate 定向
  zero-layout-engine **GATE PASS 25 指标（NEW=0）**。
- make test 唯一红腿 = js_dom_bridge CSSOM 序列化 5 案（R5398 起在册对端
  账同一名单，本轮改动不触 engine crate 域），其余 65 腿全绿。

## 5. 挂账（新增/维持）

新增：①definite CSS width + max-height 钳域的回传缺口（本臂限定 both-auto
注入域；taffy ratio 强制路径既有行为非 100×100，corpus 未现驱动案，案驱动
再接）；②vertical 写入模式同域缺口维持 R5407 分层（011/030 竖排 AR 传递
域，水平臂未覆盖）；③017 案 assert 的「inner used main size 作 % 基准」
两阶段语义已由本臂覆盖消费侧，intrinsic 阶段（max-% 作传递 basis 的
intrinsic contribution）残余与 dynamic-009/010 动态改高域维持 R5407 分层。

销案：intrinsic-percent-replaced-017（本轮修复）。
维持：R5407 残留五案其余四案分层、R5406 家族初判六面、R5405 特性级十面、
R5404 及以前挂账全量、对端账全量（js_dom_bridge CSSOM 5 案等）。

## 6. 下一步 R5409

①五域余量 misc 案续扫择轻（避开在册深域/对端面/intrinsic-percent-replaced
残留四案）②flexbox collapsed-item 算法切片评估（R5406 初判待深评项，两次
顺延）。**前置探测结论记档**（R5407 下一步②）：LayoutBox 替换元素 ratio
字段方案不必要——taffy style 既有 attr 比 + 清洗 map（engine.rs
`intrinsic_for_clamp`）已达 walk 层，零结构变更闭合 017 消费侧。
