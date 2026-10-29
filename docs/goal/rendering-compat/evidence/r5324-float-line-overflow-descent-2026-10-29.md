# R5324 — 行中 float 锚遇 nowrap 溢出行降行（CSS2 §9.5 + bugzilla 488725）

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/floats/float-nowrap-1.html` ✗0.00%（mismatch 结构同像）→ **✓0.40%**
- **corpus**: 812F → **811F**（16590 案；**FIXED=1**，REGRESSED=0），通过率
  15779/16590 = **95.1%**
- **域**: css/CSS2 floats（float 行内锚放置）× R3784 FloatAnchor 锚 y 链

## 定性（811F 账本续扫择轻，0.00% 段）

`float-nowrap-1`（mismatch 对 `-notref`）：div `width:10ch white-space:nowrap
monospace`，文本 ≈290px 溢出 80px 行带后接 `span float:right`（5ch×5ch 蓝块）。
chromium 语义：float 不与饱和行共行，**降到下一行**（headless chromium 截图实证：
test 页文本行下方蓝块；notref 页 float 先行落行 1 右缘、文本穿行底）。ZW 旧行为：
行中 FloatAnchor 无条件锚 = 当前行盒顶（`(current_y − line.height).max(0)`）→ float
落满溢行右缘，test ≡ notref 逐像素同像 → mismatch 恒红 0.00%。

同轮 chromium 截图旁证（会话级临时目录，事后不留档）：`float-nowrap-5`
（`Kittie<span float:left>Hello&nbsp;</span>`）渲染 = ref「Hello Kittie」单行——
ZW 该案已绿（✓0.00% 近纯色段），未动。

## 实现（break_lines.rs FloatAnchor 臂一处，kill-switch `ZW_FLOAT_LINE_OVERFLOW_DESCENT=0`）

行中锚（`current_line.runs` 非空）加**合取门**：

```
descent ⇔ gate_on ∧ line_nowrap ∧ (band_left + band_width − current_x) < 0.0
```

命中时锚 = `current_y + descent_est`（与本臂尾随推行的 `est_height` 同式同态：
FloatAnchor 路径两者间不改 `current_line.height`）= 尾随推行后的下一行槽顶。
`line_nowrap` 从 `current_line.runs` 片段 `ws_override` 派生（None 回退 ctx 级
`self.no_wrap`），零新增循环状态、零热路径增量（仅 mid-line FloatAnchor 时一次
`any()`）。

**门形定案过程**（两轮 corpus A/B 收敛，记实证依据）：

1. 初版 `remaining ≤ 0`（恰满即降）→ corpus 新红 `zindex/stack-floats-003`（1.67%）。
   探针（ZW_DBG 临时 eprintln，已移除）实测：该案 FloatAnchor 处 `current_x=120,
   band=(0,100)`、行片段**非 nowrap**——120 = 100 内容 + **20 既有空白折叠偏差**
   （block 断行后行首可折叠空格未塌，chromium 语义 remaining=0 共行）。
2. 次版 `remaining < 0`（严格）→ stack-floats-003 仍红（−20 触发）→ 收窄为
   **nowrap ∧ 溢出合取门**：非 nowrap 行溢出（含该案折叠偏差）不降，stack-003
   回绿；float-nowrap-1（nowrap 溢出）保持降行。终版 corpus A/B 恰 {−nowrap-1}。

非 nowrap 的 unbreakable 长词溢出行是否也该降行：chromium 语义应为是，但当前无
corpus 案驱动且会与该类折叠偏差纠缠——留待有驱动案时再放门（见挂账）。

## 回归测试

`crates/layout-engine/src/inline/tests/r5324_float_line_overflow_descent_tests.rs`
+5 pin：

1. `r5324_float_descends_after_saturated_nowrap_line`——nowrap 长词流吃满 80px 带
   后接 FloatAnchor：锚 = 32（下一行槽顶），且前文单行盒（排除断行伪降行）；
2. `r5324_float_stays_when_non_nowrap_line_fills_band`——非 nowrap 行满带宽（Ahem
   2×50=100）：锚 = 0（stack-floats-003 免疫腿）；
3. `r5324_float_stays_on_line_when_room_remains`——短词有余量：锚 = 0（floats-006
   镜像）；
4. `r5324_leading_float_keeps_line_top_anchor`——float 先行（行首空行）：锚 = 0
   （notref 镜像，空行分支不受扰）；
5. `r5324_nowrap_line_with_room_keeps_anchor`——nowrap 行有余量：锚 = 0（合取门
   另一腿，防 nowrap 一票降）。

**mutant 方向**：`ZW_FLOAT_LINE_OVERFLOW_DESCENT=0` 实测仅 ① FAIL、②③④⑤ PASS——
驱动腿双向可判别；②③④⑤ 为「维持共行」守卫腿（防未来过度放门），两态均过属预期
（合取门的单侧判别面）。

## 门禁

- fmt：本流文件 0（`cargo fmt --all --check` 现存 `crates/engine/src/js_dom_bridge.rs`
  7 处 + `tests/wpt-runner/src/testharness.rs` 3 处 drift 为并行流活跃态，非本轮
  修改面，移交知悉）。**附带收口**：R5323 测试文件 3 处空 reflow 尾改 drift（上轮
  门禁后未复检）本轮一并归位。
- clippy zero-layout-engine `--all-targets -D warnings` 0（systemd scope 守卫）。
- **zero-layout-engine lib 2240P/0F**（2235+5；2 ignored 既有）。
- **corpus 全量（最终树）**：16590 案 **811F**，A/B 对照 812F 旧账逐案 diff 恰
  {−float-nowrap-1}，**REGRESSED=0**；nowrap 族 -2..-9 与 -5/-6 维持绿。
- **本地 725 套件**：724/725，唯一红 = `css-float/float-shrink-inline-children-zw-001`
  ✗3.68%（**R5311 既有记账**，stash A/B 已证与本刀无关——kill-switch 关同值红；
  对端流 d68f3aa16 引入的回归钉）。本轮 `make reftest` 在该腿报错属既有态。
- **product-smoke**：welcome vs chromium oracle **15.68%**（≤20%，与 R5323 零位移）
  + struct ×6 PASS + form-perf PASS；**legacy** struct 0 失败。
- **bench-gate（定向 zero-layout-engine）GATE PASS 25 指标 NEW=0 全预算内**
  （medium/layout 163.07ms ≤ 181.9；first_paint 334.47 ≤ 362.35）——树态判定，
  覆盖 R5307-R5324 全部改动，零 relax 零基线改动。

## 挂账

- **收口**：float-nowrap-1（nowrap 溢出行 float 降行锚）。
- **新线索（本轮探针发现）**：stack-floats-003 第二行 advance 120 = 100 内容 + 20
  行首空格——**block 断行后行首可折叠空格未塌**（css-text-3 §4.1 phase-1「collapsible
  space at the beginning of a line is removed」对偶面；R5323 收口的是跨 run 序列
  折叠，此为行首位对块断行的塌缩）。该案现绿（旧行为锚 0 恰与 chromium 一致），
  无案驱动，留待 0.00% 段账本再扫时定性。
- **另案维持**：white-space-pre-001（img 网格 oracle + 页面级偏移，8.26%）；非
  preserve `<br>` 空行/浮动 20px 地板换 strut（ruby-align-001/001a 承重）；
  first-line-line-height-003/abspos-block-level-001 复评。
- **维持**：非 img inline-block content/border 口径分裂、content-none-select-1 负载
  脆弱族、c44-003 0.47% 残差、va 族余案；c548 类「结构性案」排除口径待用户决策；
  zero-engine js_dom_bridge fetch 8 案 + details 闪断案；js_dom_bridge fmt drift 7 处
  + testharness 3 处（并行流活跃态，移交知悉）。
- **unbreakable 长词溢出行 + float 的降行语义**：chromium 语义应为降（同 §9.5），
  现合取门不含（无驱动案 + 与折叠偏差纠缠）；有案时把 `line_nowrap` 放宽为
  「行内容溢出不可断单元」信号。
- **下一步 R5325**：①811F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与 OOF
  锚域）②block 断行后行首空格未塌定性（新线索，与 white-space 域相邻）③
  css-multicol 103F / css-transforms 67F / css-flexbox 52F 域内挑轻。
