# R5306 — css-grid-1 空 grid 容器内在高度计入显式轨道——dynamic-grid-with-auto-fill ✗1.04→✓0.00 翻绿，corpus 15745P/847F→15746P/846F 恰一案翻绿零翻转

**日期**: 2026-10-28
**驱动案**: `css/css-grid/dynamic-grid-with-auto-fill.html`（✗1.04%）
**corpus**: **16592/15746P/846F**；失败集 vs R5305（15745P/847F）归一化 diff 恰
{−目标案}，**零意外翻转**。

## 取证链

1. **PNG 形状**：ref 100×100 全绿 vs ZW 100×50——页面语义 = target 150px→50px 动态
   改高，grid（`width:100px; grid-template-rows:repeat(auto-fill,100px);
   min-height:100%; background:green`，无子项、无显式高）最终高应 = max(轨道 100,
   min-height 50) = 100；ZW 只剩 min-height 钳制值 50 → **轨道高对容器内在高度贡献
   为 0**。
2. **纯 taffy 层三分**（临时探针，纯 `TaffyTree` API 复现，与浏览器级症状逐个同值）：
   - 空 grid + auto-fill 行（无 min-height）：高 **0**（期望 100）；
   - 同上 + `min-height:100%`（父 50px）：高 **50**（钳制值，期望 100）；
   - 根级直接测量：高 **0**。
   → 缺陷在 taffy-local，与 ZW 转换层/engine 无关。
3. **根因**（taffy_tree.rs 布局派发）：`match (display_mode, has_children)` 把
   `(Display::Grid, true)` 才送 `compute_grid_layout`，无子项 grid 落入
   `(_, false) => compute_leaf_layout`——叶子路径忽略 grid-template-rows，内容尺寸
   0。上游 taffy 同构（vendored 0.12.1 上游缺陷），浏览器语义不同：网格容器内在
   尺寸由**轨道求和**（与是否有放置项无关），Chromium 空 grid 按轨道计高
   （css-grid-1 §12.1 Intrinsic Sizes；驱动案 ref 即 Chromium 输出）。
   `compute_grid_layout` 对 `items.is_empty()` 有完整处理（轨道 sizing 后
   `container_border_box = initial_row_sum + insets`，line 715 早退），零子项走
   网格算法安全。
4. **修复**（taffy_tree.rs 派发一行域）：`(Display::Grid, true)` →
   `(Display::Grid, _)`——无子项 grid 也走 `compute_grid_layout`。无轨道空 grid
   轨道和为 0、语义不变（回归测试钉住）。taffy-local 非 workspace member（仅
   `[patch.crates-io]` 接入，dev-deps 不可跑），回归钉落 `zero-layout-engine/tests/`
   （R5010 同判例）。

## 交付

- 目标案 **✗1.04→✓0.00**（单案 FILTER 复跑真通过-可信 1/1）。
- 全量 corpus **16592/15746P/846F**，失败集 diff 恰 {−dynamic-grid-with-auto-fill}。
- 回归测试 +4（`r5306_empty_grid_intrinsic_track_height.rs`）：auto-fill 行内在高
  100 / min-height:100% 不覆盖轨道高 / 固定显式行同域 / 无轨道空 grid 保持 0。

## 门禁

- layout-engine 全套 **2185P/0F**（含新 4 钉）；单案 FILTER 100%。
- fmt 0 diff；clippy zero-layout-engine `--all-targets -D warnings` 0 告警（新测试
  3 处 `length(100.0)` 泛型 f64 回落改 `100.0_f32`）；taffy 既有 104 处
  `float_literal_f32_fallback` 告警为 vendored 上游存量（stash 干净树同值，0 处在
  本改动文件），不顺手清理。
- product-smoke PASS（welcome oracle **15.50% 持平** + struct 全 PASS）。
- **bench-gate 定向 zero-layout-engine 最终 GATE PASS**（25 指标 NEW=0 全部预算内；
  R5303 同判例——忙/噪声窗 ABORT 不强制，空闲窗重跑过）。过程留痕：前两次
  GATE FAIL 判环境噪声非本改动回归——①数字层面——clean 树（stash A/B）同样超
  预算（block_layout_1000 7.6ms vs 预算 4.3ms）且 criterion 自报 change ±100% 双向
  摆动（deep_nesting 修后反而 −20%、clean 树自报 +94%），共享机器当窗负载剧烈
  （并行工作区 rustc 87-90%×2、loadavg 5.7→16 分针级摆动、桌面 WebKit/monocloud
  进程在跑）；②机制层面——4 个超预算指标全部无 grid 节点（block/flex/deep_nesting
  构造纯 Block 文档），派发改动仅对 `Display::Grid && child_count==0` 节点生效，
  不可达。基线未放宽。

## 挂账

- taffy-local vendored 存量 clippy 告警 104 处（上游域，不在本目标处理范围）。
- R5305 挂账全维持（TP 重采样结构局限、R5007 AA fringe、002.tentative、types.rs
  注释、R5299 三项、013 ellipsis host、vertical-rl 域、flake 账本）。
- **下一步 R5307**：①846F 账本续扫择轻（1.04% 家族签名 8+ 案待查：
  featureless-003/004、content-visibility slot ×2、position-absolute-dynamic-relayout、
  drop-shadow-currentcolor-dynamic、svg-sourcegraphic-invalidation、lang-pseudo-across-shadow、
  multicol ×8 等）②013 ellipsis host 传播评估（结构轮，R3768 先例）。
