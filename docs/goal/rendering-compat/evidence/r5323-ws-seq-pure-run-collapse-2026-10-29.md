# R5323 — 纯空白 run 与前词尾随空格跨边界折叠（css-text-3 §4.1 phase-1）

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/text/white-space-007.xht` ✗10.42% → **✓0.00%**（像素级）
- **corpus**: 813F → **812F**（16590 案；**FIXED=1**，REGRESSED=0），通过率
  15778/16590 = **95.1%**
- **域**: css-text white-space（跨 inline 边界空白序列折叠）× R5297/R5115/R5229 断行空白链

## 定性（R5322 下一步 ②CSS2/text 邻案）

`white-space-007`：test 页 10 个 `<span>X </span>`（`white-space:nowrap`）以单空格
相邻，容器 `white-space:normal` 500px 宽（Ahem 50px/1）。期望：span 尾随空格与
容器空格折叠为单空格（词间 50px），每行 5 X 共 2 行（orange 100px 高）；red 参考
页 100px 橙块全遮。ZW 实渲染 3 行（150px）：layout dump 走查——span 间距 **100px**
（2 空格），每行 4 X。

共享根因（break_lines.rs 纯空白 run 臂）：R5297 修「上一内容词尾随空格 + 后继 run
**词内前导**空格」时只加了词臂门 `!last_ended_collapsible_ws`，纯空白 run 臂
（`words.is_empty()`）的门仍是旧旗标 `last_was_collapsible_ws`——只在**纯空白 run**
自身之间同步，内容词尾随空格（词臂 `trailing_space_width` 剥离、
`last_ended_collapsible_ws=true`）不同步它 → `<span>X </span> <span>` 双计 2 空格。
R5297 文档头自述「后继 run 的前导空白与之同属一个空白序列——只保留前者 advance」，
本刀为该域的对偶补全（CSS Text 3 §4.1 phase-1：Any space immediately following
another collapsible space—even one outside the boundary of the inline containing
that space, provided both spaces are within the same inline formatting context—is
collapsed to a single collapsible space）。

## 实现（break_lines.rs 一处，kill-switch `ZW_WS_SEQ_PURE_RUN_COLLAPSE=0` 回退）

纯空白 run 臂加 `collapse_with_prev_trailing = !run_preserve &&
ws_seq_pure_run_collapse() && last_ended_collapsible_ws`——命中时**只吞 advance**
（序列保留首空格 = 词尾随空格的宽度，CSS 折叠取文档序首空格）。

**关键收窄**：仅吞 advance，`last_content_owner`/`last_ws_no_wrap` 照常更新——
R5115 断行机会治理（`boundary_ws_break` 经 `last_ws_no_wrap`）维持「序列最后一个
空格所属 run」口径，与修复前完全一致 → **折行位不变、仅间距归一**（本轮 corpus
REGRESSED=0 的结构性原因：断行治理路径零改动）。preserve run 空格不可折叠，不参与
序列（`!run_preserve` 门）；原子盒后 `last_ended_collapsible_ws` 已复位（既有口径），
序列不跨原子。

## 回归测试

`crates/layout-engine/src/inline/tests/r5323_ws_seq_pure_run_collapse_tests.rs` +4 pin：

1. `r5323_pure_ws_run_collapses_with_prev_trailing_ws_across_runs`——`z ` + ` ` +
   `x` ≡ 单空格锚（修前双空格 +1 字宽）；
2. `r5323_ws007_mirror_nowrap_span_boundary_two_lines`——white-space-007 真镜像：
   10× nowrap span 500px 容器 = **2 行 × 5 X、行内间距 50px**（修前 3 行、间距
   100px；兼断行治理腿——序列最后空格 normal 治理折行，nowrap 不外溢禁止折行）；
3. `r5323_pure_ws_run_kept_when_prev_word_has_no_trailing_ws`——前词无尾随空白时
   纯空白 run advance 照常计（门不得卡死为真）；
4. `r5323_preserve_pure_ws_run_not_collapsed`——preserve 语境纯空白 run 不吞
   advance（run_preserve 门）。

**mutant 方向**：`ZW_WS_SEQ_PURE_RUN_COLLAPSE=0` 实测 ①② FAIL ③④ PASS——双向
可判别。

## 门禁

- fmt：本轮四文件 0（`cargo fmt --all --check` 现存 `crates/engine/src/js_dom_bridge.rs`
  1 处 drift 为并行流 net-api 既有态，非本轮修改面，移交知悉）。
- clippy zero-layout-engine `--all-targets -D warnings` 0。
- **zero-layout-engine lib 2235P/0F**（2231+4；2 ignored 既有）。
- **corpus 全量（最终树）**：16590 案 **812F**，A/B 对照 813F 旧账逐案 diff 恰
  {−white-space-007}，**REGRESSED=0**；同族 white-space-001..006 维持绿、008 维持
  R5322 修复态（✓0.65%）、white-space-pre-001 维持 8.26% 另案。
- **product-smoke**：welcome vs chromium oracle **15.68%**（≤20%，与 R5322 零位移）
  + struct ×6 PASS + form-perf PASS；**legacy** 42 fixtures struct 0 失败。
- **bench-gate（定向 zero-layout-engine）GATE PASS 25 指标 NEW=0 全预算内**
  （medium/layout 163.07ms ≤ 181.9；first_paint 347.28 ≤ 362.35）——纯空白 run 臂
  每 whitespace-only run 一次（非逐词热循环），LazyLock bool 读零增量；**顺带收口
  跨 R5317/R5318 的 bench-gate 静窗复测挂账**（当前树含 R5307-R5323 全部改动正式
  过门，门为树态判定，零 relax 零基线改动）。

## 挂账

- **收口**：white-space-007（nowrap span 边界空格序列折叠）。
- **另案维持**：white-space-pre-001（img 网格 oracle + 页面级偏移，8.26% 与本刀
  无关）；非 preserve `<br>` 空行/浮动排除估算 20px 地板换 strut（ruby-align-001/001a
  承重，R5322 另案）；first-line-line-height-003/abspos-block-level-001 随之复评。
- **维持**：非 img inline-block content/border 口径分裂、content-none-select-1 负载
  脆弱族、c44-003 0.47% 残差、va 族余 122/baseline-003/intrinsic-size-009/R5314/
  R5315；c548 类「结构性案」排除口径待用户决策；zero-engine js_dom_bridge fetch 8 案
  + details 闪断案；js_dom_bridge fmt drift 1 处（并行流既有态，移交知悉）。
- **下一步 R5324**：①812F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与 OOF
  锚域）②CSS2/text 邻案（white-space-pre-001 独立定性）③css-multicol 103F /
  css-transforms 67F / css-flexbox 52F 域内挑轻。
