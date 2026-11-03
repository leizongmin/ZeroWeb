# R5407 — intrinsic-percent-replaced 族深评：循环 % 宽替换盒 AR 传递

日期：2026-11-03
驱动案：`css/css-sizing/intrinsic-percent-replaced-031.html`（✗2.08% → ✓0.00%）
状态：已完成（corpus 真态 720F → 719F，FIXED=1，REGRESSED=∅）

## 1. 缘起与归因链

控制面 R5407 ②：sizing intrinsic-percent-replaced 族深评（42 案 6 败，
R5406 家族初判判为「单一公式域候选——aspect-ratio 传递 % basis」）。

- **031 布局树 dump 定谳**（`layout-dump` + `ZW_LAYOUT_DUMP_REF=1`）：
  `.flex > .item(height:100px) > canvas(width:100%;height:100%,attr 10×10)`
  渲 `item w=0 / canvas w=0 h=100`——canvas 高 % 已解析、宽 % 循环不可解析
  归零，绿区整体消失（diff ≈100×100 = 2.08%）。
- **路径核实**：`.flex` 是满宽块级 flex 容器 → item 基准由 **taffy 自身**内容
  测量计算（不走 ZW `flex_row_intrinsic_width`）；taffy 对 `width:%` 替换叶在
  内容基准语境解析 0，attr AR 传递不触发（`width:Auto` 时 taffy 传递已生效
  ——014/dynamic-012 通过即证）。ZW 侧 R3792/R5040 臂完整但仅 shrink-to-fit
  链可达；R5381 替换 inline 臂读首趟 `child.width`（=0）。
- **结论**：断点 = 「循环 % 宽替换盒在 taffy 内容测量中归零，ZW 固有测量
  walk 未被 flex 布局消费」。非缺公式，缺**测量结果向 taffy 的回喂**。

## 2. 修复（双臂，单 kill-switch）

kill-switch `ZW_INLINE_REPLACED_AR_TRANSFER=0`（default-on，OnceLock 进程级
快照，同 R5318 热路径口径）：

1. **walk 臂**（`intrinsic_sizing.rs::inline_replaced_cyclic_pct_transfer`）：
   R5381 替换 inline 臂内，width ∈ {Auto,%}（循环语义，R5040 同口径）+
   显式 CSS height（禁 Auto——防 inline→taffy Block 拉伸伪影入账）+ 比例
   （CSS aspect-ratio 优先，HTML 尺寸属性回落——canvas 位图即属性值）+
   used height>0.5（R3792 叶回退同阈值）→ 复用
   `aspect_ratio_transferred_width`（box-sizing 分型）。水平书写模式限定
   （R4076 同门：vertical 主/交叉轴互换）。
2. **sizing 期 flex item 基准臂**（`engine/sizing.rs`
   `apply_flex_aspect_ratio_item_size` 第三分支）：非叶 flex item（行向、宽
   auto、basis auto）子树含 % 宽替换盒（`subtree_has_pct_width_replaced`
   扫描）时，用 ZW 测量 walk（`flex_item_base_size`，含①臂）算基准写入
   `flex_basis` + mark_dirty（R4991 写 basis 同法，共用重跑组）。
   `flex_item_base_size` 升 pub(crate)，pass 签名加 `doc`。
   031 链路：basis=100 → item 宽 100 → canvas `width:100%` 解析 100 ✓。

017 不触发（img 宽 Auto、比来自解码表——`img_intrinsic_ratios` 不入 walk，
属性比兜底只覆盖带属性场景），无回归面。

## 3. 常驻测试

- `intrinsic_sizing.rs::r5407_inline_replaced_cyclic_pct_width_transfers_used_height`
  （★load-bearing：031 根因锚——% 宽 canvas 传递 used height 100 × 属性比 1.0）
- `r5407_transfer_arm_gates_auto_height_and_definite_width`（门控：Auto 高
  不传递防拉伸伪影；definite Px 宽直读 outer_w）
- **R5406 漏网面修复**：8 个 table 单测（r5214/r5261×3/r5279×2/r5280×2）在
  R5406 提交即红（td border-box +2px UA padding），上轮 make test 账面未记
  （增量编译时序疑云，本轮 stash 对照 0a0f01720 全绿 / adb2f66e7 同 8 红定谳
  责任归属）。本轮按**恢复测试隔离意图**修复：测试内 td 补 `padding:0`
  （td UA 默认语义已由 R5406 自带测试覆盖，不更新断言数值）。

## 4. 验证

- 单测 2386/2386（2384 既有 + 2 新）；驱动族 42 案 38✓5✗——031 翻绿，
  011(8.33%)/030.tentative(6.25%) 竖排域、017(1.21%) 解码比+max-% basis
  两阶段、dynamic-009/010(2.08%) 动态改高域如实残留，族内零回归。
- kill-switch OFF：031 精确回 ✗2.08%（与 BASE 一字不差）。
- **corpus 全量 FIX 窗口 723F** vs R5406 FIX2 窗口 725F：转绿 =
  {031（修复）, table-anonymous-objects-146/147/148（在册抖动族）}；新红
  候选 2（table-anonymous-objects-153 = 同在册抖动族、
  background-image-gradient-currentcolor-linear-repaint）双双 3×3 隔离
  全绿洗清（guidelines #35 窗口并发抖动）。**真态 720F → 719F**。
- 本地集 **725/725**；product-smoke **struct PASS**；product-smoke-legacy
  **struct-check failures: 0**。
- fmt ✓ / guarded-clippy 全工作区 EXIT 0 / bench-gate 定向
  zero-layout-engine **GATE PASS 25 指标（NEW=0）**。
- make test 唯一红腿 = js_dom_bridge CSSOM 序列化 5 案（R5398 起在册对端
  账同名单），其余 65 腿全绿。

## 5. 挂账（新增/维持）

新增：①intrinsic-percent-replaced 残留 5 案分层——011/030 竖排 AR 传递域
（水平臂未覆盖，需 vertical 侧臂或 taffy vertical 传递）；017 img 解码比
plumbing（LayoutBox ratio 字段或 map 下传）+ max-height:% 作传递 basis 的
两阶段（R5062 同族）；dynamic-009/010 动态改高重测量域（脚本改高后 intrinsic
不重算）。②R5406 8 个 table 单测红已修（隔离意图），账面修正记本证据 §3。

销案：intrinsic-percent-replaced-031（本轮修复）。
维持：R5405 特性级十面、R5406 家族初判六面（intrinsic-percent-replaced 族
6F→5F）、R5404 及以前挂账全量、对端账全量。

## 6. 下一步 R5408

①五域余量 misc 案续扫择轻（避开在册深域/对端面/本轮残留五案）②017 前置
探测：LayoutBox 加替换元素 ratio 字段（extract 从 taffy style 读，attr/解码
比统一源）评估 walk 层 017 可达性（max-% basis 两阶段为第二断点，或特性级
挂账）③flexbox collapsed-item 算法切片评估（R5406 初判待深评项）。
