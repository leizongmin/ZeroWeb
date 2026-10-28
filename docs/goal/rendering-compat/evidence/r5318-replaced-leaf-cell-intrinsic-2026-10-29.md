# R5318 — 替换叶盒 cell intrinsic 固有宽（td>img[display:block] 塌 0 收口）

- **日期**: 2026-10-29
- **驱动案**: `css/CSS2/css1/c42-ibx-pad-000.xht` ✗2.60% → **✓0.08%**
- **corpus**: 824F → **821F**（16592 案，FIXED=3，REGRESSED=0；trend 行 15771P/821F/95.1% @ `da90b10ad`）
- **域**: css-sizing-3 §5.1 max-content（替换叶盒）× CSS2 §17.5.2（cell intrinsic）× CSS2 §9.5.1（float shrink-to-fit 连带）

## 定性（探针链）

c42（inline span border/padding/margin 跨 4 行，CSS2 §8.6 域）首 suspected §8.6 切片结构域，
PNG 对比翻转定性：

| 探针 | 形态 | 修复前 | 修复后 |
|---|---|---|---|
| p1 | td 内 **inline** img | ✓ 双格正常（3.15% 对空白） | 同 |
| p2 | td 内 `display:block` img | ✗ **img 消失**（0.86% 仅剩边框） | ✓ 3.15% 与 p1 同值 |
| p5 | td 仅含 block img | ✗ 内容 0×0（0.18%） | ✓ 2.46% |
| p3/p4 | body/div 内 block img | ✓ 自然 210×120 正常 | 同 |

**根因**：c42 表格总宽 224 应 428（格2 = `.control` 红底 + `support/css1test42a.png`
210×120 全缺席）——td 内 block 级替换元素 cell intrinsic 塌 0 → 列宽 0 → 整格消失
（红底亦 0×0 不画）。c42 格1 的 span §8.6 切片渲染本已达标（格2 修复后 0.08% 即过），
「非 img inline-block 口径分裂」 suspicion 对本案不成立（该挂账维持另案）。

**机制链**：cell intrinsic 走 `compute_cell_intrinsic_width_impl` R4028 臂（auto 宽块级
子 → `box_content_max_width`）→ `box_content_max_width_inner` 叶盒回退只认
`own_explicit`（CSS 宽，auto→0）+ `text_w`（img 无文本→0）——**替换元素固有尺寸缺席**。
对照：inline img 走 IFC 路径有 attr/解码固有宽（p1 ✓）；第一趟 taffy 已把 attr/解码尺寸
写为 definite 宽（`tree.rs apply_replaced_element_sizing`），但 post-taffy 宽对非替换块是
拉伸伪影不可读（R4028 立场），替换叶被一并拒之门外。

## 实现（kill-switch `ZW_REPLACED_LEAF_INTRINSIC`，default-on，`=0` 回退）

`box_content_max_width_inner` 叶盒臂（`!has_in_flow_child`）新增替换叶分支：
`is_replaced && HorizontalTb` 时贡献 `(box_node.width − frame).max(0)`——替换盒主轴永不
fill-stretch，第一趟宽即真实固有值（无拉伸伪影）；CSS 显式宽已反映在第一趟宽中，与
`own_explicit`/`text_w` 取 max 合并安全。OnceLock 进程级缓存读 env（本函数热路径，
`mc_contain_size_fix_enabled` 同款口径）。垂直书写不开放（块轴语义不同域）。

连带收益同根自洽：
- **c43-rpl-ibx-000** ✓0.49%：容器内替换叶（img 固有宽贡献）。
- **floats-030** ✓0.96%：`img{float:left}` 的 float shrink-to-fit max-content 此前测 0，
  float 塌；现在固有 15×15。

## 回归测试

`crates/layout-engine/src/table/r5318_replaced_leaf_cell_tests.rs`（table.rs `mod` 声明，
每轮文件模式不增长超限的 tests.rs）：

- `r5318_block_replaced_leaf_cell_intrinsic_is_natural_width`：c42/p5 形态钉测——cell
  （taffy junk 宽 0）+ `is_replaced` img（第一趟 definite 210）→ cell intrinsic ≥ 209.5
  （修复前 R4028 臂叶盒双 0 → ~0）。
- **mutant**：`ZW_REPLACED_LEAF_INTRINSIC=0`（进程级）→ 该测试 FAIL——双向可判别（实测）。

## 门禁

- fmt 0；clippy `zero-layout-engine --all-targets -- -D warnings` 0。
- **layout-engine lib 2225P/0F**（2224 + 本轮 1；全量跑）。
- 本地 723 套件 722/723，唯一红 = `float-shrink-inline-children-zw-001`（R5311 既有记账）。
- product-smoke struct-check PASS（多 viewport 零 issue）。
- corpus 全量 ×2：822F（含 content-none-select-1 +1 负载伪红，load 9.8）→ 复跑
  **821F** 干净账；FIXED=3 REGRESSED=0；content-none-select-1 隔离 ✓0.00% 定谳
  （R5317 既有负载脆弱族，二次复现归因一致）。
- **bench-gate GATE FAIL 4→3/25 归因环境，静窗挂账维持（跨 R5317/R5318 两轮）**。
  三腿证据链：①ON @load 10.3 → medium/layout 193.53 / first_paint 389.03 FAIL；
  ②OFF（kill-switch 关 = 字节旧行为）@load ~8 → layout 181.5 贴预算帽 181.9；
  ③**ON @load 5.6 → layout 174.16 PASS**（比 OFF 腿更快）——ON 带 174.16-193.53 与
  OFF 重叠、随负载单调，Δ(flags) ≪ Δ(load)；同跑 `flex_layout_1000` 6.95M/预算
  3.97M（+75%）、`block_layout_1000` 5.52M/4.35M（+27%）——单次叶盒 bool 检查
  不可能致此量级，纯环境通胀。ZeroWeb-3 encoding rally 活跃窗（load 5.5-11.4），
  真静窗本会话不可得；零 relax 零基线改动。

## 挂账

- **收口**：无（本轮为独立新域修复；R5317「非 img inline-block 口径分裂」嫌疑对 c42
  不成立，该挂账维持另案）。
- **维持**：bench-gate 静窗复测（本轮三腿 A/B 增强证据在门禁节，跨两轮；基线重录属
  用户决策不自行放宽）、非 img inline-block content/border 口径分裂、content-none-select-1
  负载脆弱族（本轮二次复现）、c44-003 0.47% length-va 残差、va 族余 122/baseline-003/
  intrinsic-size-009/R5314/R5315。
- **下一步 R5319**：①821F 账本续扫择轻（避开 va 域与 OOF 锚域）②floats 族连带复查
  （float-replaced-width-007/008 与本域相邻，float 替换叶 shrink 已通，其 width 计算臂
  另案核对）③outline-028 lime 域 / balance 供给评估。
