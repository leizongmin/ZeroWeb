# R5329 — 替换内容叶 intrinsic 测量（css-sizing-3 × css-flexbox-1 §4.5）

- **日期**: 2026-10-29
- **驱动案**: `css/css-flexbox/flex-minimum-width-flex-items-005.xht` ✗1.88% → **✓0.00%**
  （同构 006 ✗1.88% → **✓0.00%**）
- **corpus**: 808F（R5328 交付态）→ **804F**（16590 案；**FIXED=4**，REGRESSED=0），
  通过率 15786/16590 = **95.10%**
- **域**: 替换元素叶盒 intrinsic 测量 × flex/grid item automatic minimum size
  content size suggestion

## 定性

两案同构：10px flex 容器 + img item（`width:100px`；005 固有 200×200、006 固有
100×100），ref = 100px 绿方块无红。期望（§4.5）：content size suggestion =
img min-content = 固有宽（css-sizing-3：replaced element 的 min/max-content =
intrinsic size），taffy `maybe_min(显式 size)` 得 specified/content suggestion
取小（005：min(200,100)=100；006：min(100,100)=100）→ item 保持 100px 溢出容器。
ZW 渲 ~10px 绿条（红底外露）→ automatic min 塌 0。

链路取证（代码级闭环）：

1. taffy flexbox automatic-min fallback（taffy-local `compute/flexbox.rs:851-875`）：
   `style_min_main_size = min_size.or(overflow→automatic_min)`；overflow Visible →
   None → fallback `measure_child_size(child, ContentSize, Size::MIN_CONTENT...)`，
   结果 `maybe_min(child.size.main).maybe_min(max)`（:872-873）——测量回固有 200 →
   min=100 ✓；回 0 → min=0 ✗。
2. 测量入参：`child_known_dimensions = child.size.with_main(dir, None)`（:700）——
   §4.5 content 测量主轴 known = None（flex 控制主轴），img 的 CSS width:100px
   不经 known 传入。
3. ZW measure 回调（layout-engine `inline_finalization.rs` `measure_text_content`）：
   img 叶无子节点 → `has_inline_content` false → **空叶分支**；MinContent 面旧仅
   `cis_intrinsic`（contain:size 替代值，img 无 containment → None）→ **回 0**。

根因：measure 空叶分支「无内容回 0」不适用于**替换内容**——替换元素的固有尺寸
即其内容尺寸，img 登记表（`img_intrinsic_sizes`）已在参数位但该分支从未消费
（R5318 修 ZW 侧 `box_content_max_width_inner` 为同族不同臂，本刀修 taffy 叶测量臂）。

## 实现（无 kill-switch——R5320/R5321 单侧域先例）

`measure_text_content` 空叶分支：img（`local_name()=="img"`，仅 img 登记表——svg
ratio-only/no-ratio 信号走 converter aspect_ratio/默认对象尺寸面）且在登记表时，
MinContent/MaxContent 测量的 width/height content 面回**固有尺寸**（content-box——
taffy `compute/leaf.rs:145` 自加 padding/border）。containment CIS 优先级不变
（contain:size img 仍按 CIS 回报）；Definite 测量面维持显式 width/height 回退
不变；**非替换空叶仍回 0**（R4008 flex-shrink-001 族守卫语义：显式 width 不回填）。

## A/B 与回归

- corpus 同树 A/B（基线 R5328 交付态 808F fail-list `/tmp/r5328-E.txt`，与 D 份
  逐案一致）：→ **804F**，逐案 diff 恰 {−005, −006,
  −css/css-grid/img-src-changes.html, −css/css-sizing/aspect-ratio/intrinsic-size-009.html}，
  四案全 ✓0.00% 真通过。**REGRESSED=0**。两 bonus 案 = grid img item 同面 +
  fit-content/min-content img 固有臂，同根因外溢收益。
- **+3 pin**（`engine/tests/r5329_replaced_leaf_intrinsic_min_tests.rs`）：
  005 形（固有 200×200 → 100px）、006 形（固有 100×100 → 100px）、守卫腿（非替换
  空叶 div width:100px 仍收缩到 10px——R4008 语义锚）。**mutant**：stash 修复
  hunk 实测两 img 腿 FAIL、守卫腿 PASS，双向可判别。

## 门禁

- fmt：本轮 0 漂移（新文件经 `cargo fmt --all` 一次收敛）。
- clippy：workspace `--all-targets -D warnings` 0（guarded 入口）。
- **zero-layout-engine lib 2250P/0F**（2247+3）。
- **本地 725 套件 724/725**：唯一红 = R5311 既有记账
  float-shrink-inline-children-zw-001 ✗3.68% 零位移。
- **product-smoke**：welcome vs chromium **15.68%** ≤ 20%（R5324 起零位移）+
  struct ×6 PASS；**legacy** 42 fixtures struct 0 失败。
- **bench-gate（定向 zero-layout-engine）GATE FAIL 5/25，NEW=0 → 同窗 clean-HEAD
  A/B INCONCLUSIVE ＝环境归因**：clean-HEAD 3/25 FAIL 且分布与 WIP 重叠
  （block 4.575M ns ≥ WIP 4.526M；flex 4.111M ≥ 4.081M；medium layout_ms 179.6
  PASS / WIP 188.1 FAIL 跨骑预算 181.9），测量期并行流 encoding rally
  `testharness-encoding-legacy-mb` 163% CPU（loadavg 8.95）。变更面 = 空叶测量
  每 img 一次 local_name match + HashMap get（ns 级），不可能解释 layout 域
  均匀 +40~52%。R5316-R5318/R5328 同判例。**静窗复测挂账（与 R5328 项合并）**，
  零 relax 零基线改动。

## 挂账

- **收口**：flex-minimum-width-flex-items-005/006、img-src-changes、
  intrinsic-size-009。
- **静窗复测**：bench-gate 定向 zero-layout-engine（R5328+R5329 合并清单；环境
  归因 A/B 证据在案）。
- **同族余案（未驱动，转菜单）**：flex-minimum-height-flex-items-027 ✗2.08%、
  029 ✗8.33%、flex-minimum-width-flex-items-015 ✗6.25%——column/height 臂与
  非 replaced 臂，待逐案分诊（本刀未触 height main 轴 flex，勿臆断同修）。
- **维持**：non-img inline-block 口径分裂、content-none-select-1 脆弱族、c44-003
  残差、va 族余案、c548 口径待用户决策、zero-engine fetch 10F + details 闪断、
  fmt drift 移交、R5327 fragment 臂内容区锚深结构域、safe/unsafe overflow-position
  解析+语义域（flexbox-safe-overflow-position-001）、taffy Clip 不禁用 automatic
  min size 观察。
- **下一步 R5330**：①804F 账本续扫择轻（0.00% 段 + 低 diff 段，避开 va 域与
  OOF 锚域）②flex-minimum 同族余案分诊（027/029/015）③multicol 嵌套分片专项
  预研（8×1.04% 聚簇）。
