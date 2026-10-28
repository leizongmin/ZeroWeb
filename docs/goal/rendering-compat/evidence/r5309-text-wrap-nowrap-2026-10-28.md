# R5309 — `text-wrap: nowrap` layout 期消费落地（CSS Text 4 §text-wrap）——text-wrap-nowrap-001 ✓ 翻绿 + css-ruby break-within-bases/nowrap 附带翻绿，corpus 15743P/842F→15752P/840F 零意外翻转（layout-engine 两文件 + 一处签名改读整个 ComputedStyle）

**日期**: 2026-10-28
**驱动案**: `css/css-text/white-space/text-wrap-nowrap-001.html`（R5308 下一步①842F 账本续扫命中）
**corpus**: 干净树复跑 842F 与 R5308 定版逐案一致；修复后两次全量 **16592 / 15752P / 840F / 841F**（差异恰为已知 flake content-none-select-1 摆入）。

## 定性

`text-wrap` 属性已解析已存储（`TextWrapComputedValue::{Wrap,Nowrap,Balance,Pretty}`，apply_advanced.rs + `ComputedStyle.text_wrap`），paint 侧已消费（text.rs `resolve_text_wrap`），但 **layout 侧零消费**——测/排仍按容器宽软换行。案形：`width:20ch`（ch 解析 160px）+ `text-wrap:nowrap` 页本应单行溢出，ZW 渲染 3 行换行形，与 mis-ref（同宽无 nowrap）**渲染同形 0/480000px 全同 → mismatch-ref 命中误判红**。

修复期定位到**两层缺口**：

1. **容器级**：`compute_final_inline_layouts` 主配置与 `resolve_no_wrap_for_ifc_measure`（测量期盒高判定）只读 `white_space`，`text-wrap:nowrap` 不抑制——测量路径盒高按换行形（54 vs 单行 18）。
2. **run 级（根因，首层修复后测试仍红暴露）**：run 级 `ws_override` 在 break_lines 处**遮蔽**容器级 IFC `no_wrap` 旗标（`Some` 优先），而 `white-space:normal` 的带样式 run 覆盖恰为 `Some(default)`（no_wrap=false）——容器级 nowrap 对带样式文本 run 无效。text-wrap 与 white-space 均 inherited 且独立生效，run 侧语义 = 任一 nowrap 即禁软换行。

## 实现（layout-engine 两文件，无新 kill-switch）

- `collect_items.rs`：`run_white_space(&WhiteSpaceValue)` → `run_white_space(&ComputedStyle)`——改读整个 ComputedStyle，`text-wrap:Nowrap` 折入 run 级 `no_wrap`（单向加严：不新造换行点、不动 preserve/break_at_newline 折叠维度；Balance/Pretty/Stable 未实现维持 Wrap 语义，与 tree.rs `container_has_balancing_text_wrap` 同口径）。全部 6 处调用点随签名联改。
- `inline_finalization.rs`：主配置 `no_wrap` 与测量 resolver 在 `white_space ∈ {Pre,Nowrap}` 基础上 OR `text_wrap == Nowrap`（与 paint Path B 同源消费；pre 族本无软换行机会，text-wrap 不新造）。
- 规范链接：https://drafts.csswg.org/css-text-4/#text-wrap 。

## 交付

- 驱动案 **text-wrap-nowrap-001 ✓0.00% 翻绿**（定向复跑；单行溢出形 vs mis-ref 3 行换行形，mismatch 语义恢复成立）。
- 附带翻绿 **css-ruby/break-within-bases/nowrap**（run2/run3 双绿）：案内 `.tw-nowrap` 段直接声明 `text-wrap:nowrap`——与驱动案同域直击，因果可溯。
- 全量 corpus A/B 定谳：干净树（R5308 HEAD）复跑 **842F 逐案复现**；修复后 run2 **840F**、run3 **841F**。逐案 diff = **{−text-wrap-nowrap-001（驱动）, −break-within-bases/nowrap（附带）}**，新增红仅 content-none-select-1（R5308 既有 flake 账双向摆动，run3 摆入）；multicol-fill-auto-block-children-002 维持通过态。**零意外翻转**。账本：15743P/842F → **15752P/840F**（净 −2）。
- 回归测试 +2（`crates/engine/tests/r5309_text_wrap_nowrap_layout.rs`，引擎级 RenderPipeline API）：①nowrap 页单行（目标 div 160×18）②mis-ref 页维持换行形（160×54，mismatch 对侧不误伤）。

## 门禁

- layout-engine 全套 **2206P/0F**（lib 2189 + 集成 17）；r5309 引擎回归 ×2 绿。
- fmt 0 diff；clippy zero-layout-engine + zero-engine `--all-targets -D warnings` 0 告警（测试文件初版 4 处修正：doc 注释行首 `+` 被解析为列表项致 doc_lazy_continuation ×3、`filter().next_back()` 改 `rev().find()`）。
- product-smoke：struct ×6 全 PASS + form-perf PASS；welcome oracle **15.68%**（75254px 与 R5308 同值，welcome.html 无 text-wrap 声明零因果路径）。
- **make test 全量 EXIT=0**（改动面为 inline 热路径共享代码，回归面大于 R5308 单常量，故补全量）。
- bench-gate 定向 zero-layout-engine：WIP 跑 GATE FAIL 6 指标（NEW=0，incremental 12.4ms vs 基线 3.0ms 等 3-4×）→ **同窗 clean 树 stash A/B GATE FAIL 10 指标且共享项绝对值更差**（block 5.69ms vs WIP 4.80ms、flex 5.26 vs 4.61、grid 453 vs 413µs）——超预算为对端流 sweep 负载签名非本修复（三处 `matches!` 枚举判断对 block/flex/grid 微基准物理不可达），R5299/R5303/R5307/R5308 判例挂账空闲窗复跑，零 relax 零基线改动。

## 挂账

- bench-gate 空闲窗并账复跑（R5307 残余 flex +3%、R5308 全量 + 本轮三项并账）。
- R5307/R5308 挂账全维持：同 walk 直接兄弟 float flat 形态叠压（Phase 1 结构轮）、earliest-fit kill-switch 回退行模型无 inherited 带集（FIXME 已注）、float_positioning.rs 3813+ 行超 2000 行规则、welcome 15.68% 位移移交对端流（50569142e 归因在案）。
- flake 账本无新增（content-none-select-1 / size-adjust.tentative / multicol-fill-auto-block-children-002 维持既有双向摆动记账）。

## 下一步 R5310

①840F 账本续扫择轻 ②013 ellipsis host 传播评估（结构轮，R3768 先例，视账本轻案供给）③bench-gate 空闲窗并账复跑。
