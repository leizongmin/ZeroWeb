# R5328 — overflow computed value 传播（css-overflow-3 §3.1 × css-flexbox-1 §4.5）

- **日期**: 2026-10-29
- **驱动案**: `css/css-flexbox/flexbox-min-width-auto-004.html` ✗1.45% → **✓**（真通过-可信）
- **corpus**: 810F（基线，含 content-none-select-1 flake）→ **808F**（16590 案；
  **FIXED=1**，REGRESSED=0），通过率 15782/16590 = **95.1%**；同二进制复跑 808F
  fail-list 逐案一致
- **代码**: `0e21b0bc9`（converter + pin 同提交）
- **域**: flex/grid item automatic minimum size × overflow 联动（§4.5 min-size:auto
  禁用条件）

## 定性

flexbox-min-width-auto-004（水平排版，非 sideways 域）：4 个 flex item（容器
30px）各含 80×40 内容，overflow-y 分别 visible/hidden/scroll/auto。期望：item 1
（双 visible）min-width:auto 生效保持 84px；item 2-4 overflow-y 非 visible →
overflow-x 计算为非 visible → **automatic minimum size 禁用**（min-width:0）→
收缩到 30px。ZW 四行全宽（test ✗）：REFTEST_DUMP 对比 ref（2-4 窄条 + item 3 滚动
条角）坐实 min-width:auto 未被禁用。

根因（converter/mod.rs `convert_overflow`）：CSS overflow-x/y **两轴独立映射**
taffy，无 §3.1 computed value 传播——「visible/clip 与非 visible/clip 配对时
visible→auto、clip→hidden」。taffy flexbox（compute/flexbox.rs:852）按
`Overflow::maybe_into_automatic_min_size`（Hidden/Scroll → 0）消费 overflow——
x=Visible（未传播）→ automatic min = min-content 80px → item 撑开。

## 实现（无 kill-switch——R5320/R5321 单侧域先例；规范行为一处收口）

1. `computed_overflow_pair`：§3.1 传播对——一轴 propagated（Visible→Auto /
   Clip→Hidden）、另一轴非 visible/clip 族时替换；两轴同族不传播。converter 入口
   统一计算。
2. `convert_overflow_computed`：传播产生的 auto（raw 声明 visible）并入 taffy
   `Overflow::Hidden`——automatic min size 禁用语义与 auto 一致（
   `is_scroll_container()` 同真），但 **scrollbar 槽位不预留**：taffy 对 Scroll
   轴无条件预留 `scrollbar_width`，chromium 对 auto 仅实际溢出才出滚动条
   （R4265 同判）。**scrollbar-empty-001 守卫腿**：首版传播 auto→Scroll 实测该案
   +1 红（overflow-y:scroll 空 div 传播 x=auto → x 轴 Scroll → 底部横条槽位 →
   空 div 长高 15px 红条 ✗2.45%），收窄后回绿。
3. **raw 声明值路径不动**：raw auto→Scroll（R4265）、raw Hidden/Scroll/Clip 原映射；
   scrollbar_width 预留门维持读原始声明值（传播 auto 与 raw auto 同不无条件预留）。

## A/B 与回归

- corpus 同树 A/B（基线 R5327 交付态 fail-list）：→ **808F**，逐案 diff 恰
  {−flexbox-min-width-auto-004, −content-none-select-1}；后者为负载脆弱族 flake
  （本轮基线跑伪红，两轮 WIP 树均 ✓0.00%）。首版树 +scrollbar-empty-001 红，收窄
  版两跑均绿。**REGRESSED=0**。
- **+1 pin**（converter/tests/basic.rs `r5328_overflow_computed_propagation`）：
  visible×hidden / visible×scroll / clip×hidden 三传播腿 + visible×clip、双 visible
  两同族不传播腿 + raw auto 不回退腿（auto×visible → x=Scroll 且 y 传播 Hidden——
  §3.1 对 auto 一致，chromium computed overflow-y 亦为 auto）。**mutant**：stash
  converter hunk 实测 pin 于第一传播腿 FAIL，双向可判别。

## 门禁

- fmt：本轮 2 文件 0 漂移（`cargo fmt --all` 一次收敛）。
- clippy：workspace `--all-targets -D warnings` 0（guarded 入口）。
- **zero-layout-engine lib 2247P/0F**（2246+1）。
- **本地 725 套件 724/725**：唯一红 = R5311 既有记账
  float-shrink-inline-children-zw-001 ✗3.68% 零位移。
- **product-smoke**：welcome vs chromium **15.68%** ≤ 20%（R5324 起零位移）+
  struct ×6 PASS；**legacy** 42 fixtures struct 0 失败。
- **bench-gate（定向 zero-layout-engine）GATE FAIL 3/25，NEW=0 → 同窗 clean-HEAD
  A/B INCONCLUSIVE（负载守卫标记 suspect，loadavg=20）＝环境归因**：并行流
  （encoding rally）同期重载（testharness-html-semantics 569% CPU），基线 medium
  total p95 834ms 比本轮 FAIL 值（first_paint 385.69ms / layout 184.05ms /
  block_layout 4.61M ns vs 预算 362.35/181.9/4.35M）更劣一倍级，分布重叠——变更面
  为每节点一次 enum match（ns 级），不可能解释 43% 漂移。R5316/R5317/R5318 同判例
  （活跃窗 360→794ms 振荡）。**静窗复测挂账维持**（零 relax 零基线改动）。

## 挂账

- **收口**：flexbox-min-width-auto-004。
- **静窗复测**：bench-gate 定向 zero-layout-engine（环境归因 A/B 证据在案）。
- **观察（不挂账）**：taffy `Overflow::Clip` 不禁用 automatic min size
  （`is_scroll_container()` 含 Clip= false）——spec 上 clip 非 visible 应禁用；
  corpus 无红案驱动，留待 clip×flex 交互案出现再修（本刀传播 Clip→Hidden 面已
  使配对形态达标）。
- **维持**：non-img inline-block 口径分裂、content-none-select-1 脆弱族、c44-003
  残差、va 族余案、c548 口径待用户决策、zero-engine fetch 10F + details 闪断、
  fmt drift 移交、R5327 fragment 臂内容区锚深结构域。
- **下一步 R5329**：①808F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与
  OOF 锚域）②css-flexbox 域内续挑（flexbox-safe-overflow-position-001 1.40%、
  baseline-synthesis 族）③multicol 嵌套分片专项预研（8×1.04% 聚簇）。
