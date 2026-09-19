# R4544 证据：flex item definite 高向子代百分比 CB 传播修复——5 案翻绿零新红

- 日期：2026-09-20（R4544 C 轮——R4543 下轮方向①，sizing.rs 1 修复点 +24 行）
- 谱系：R4543 P 轮（taffy 层排除、ZW 集成面定位）→ 本轮定位修复。

## 定位（本轮插桩链）

ZW 管线插桩（三处 dump：converter 出口 taffy Style / compute 后 taffy 树 / extract 后
LayoutBox / 各 post-pass 边界）逐层判定：

1. converter 出口：child `height:100%` → `Dimension::percent(1.0)`、item
   `height:100px` → `length(100)`（style 正确）；
2. compute 后 taffy 树：child layout **50×100**（taffy 解析正确，与 R4543 纯 API 基线
   一致）；
3. extract 后 LayoutBox：child **50×100**（extract 正确）；
4. 各 post-pass 边界：**已塌 0** → 锁定 extract 与首个 post-pass 之间的区间；
5. 区间内 = **3.2 R695 pass**（百分比 height 在不明确 CB 上 compute-to-auto）的重跑组
   （set_style + mark_dirty + 重跑 taffy + 重新提取）。

## 根因（sizing.rs `apply_indefinite_percent_height_to_auto` walk）

walk 的 `cb_definite`（子元素 % 高解析基准）传播被 flex-item gate 整体短路：gate
`(!is_abs && (!parent_is_flex_grid || grid_replaced_pct_indefinite))` 为跳过 item 自身
%/stretch 解析语义（R691 域）而设，但它把**整段 match**（含 `other =>` definite 高的
`my_definite = Some(内容高)` 传播臂）一并跳过 → flex item 自身 `height:100px`
（definite）不传播 → 子代 `cb_definite = None` → 子 `height:100%` 被误 compute-to-auto
塌 0。容器 definite 的场景不影响（cb_definite 从容器链传入 Some → 子在 R695 前已正确），
与 R4543 探针「容器 auto 塌 0 / 容器 100px 正常」完全吻合。

## 修复（窄修 +24 行）

gate 跳过后补一段 definite 传播：`parent_is_flex_grid && !grid_replaced_pct_indefinite`
且 `resolve_sizing_definite_real_length(&s.height, s)` 为 Some（**仅 definite 实长度**，
%/auto/stretch 语义仍归上方各臂/R691）时 `my_definite = Some(box_sizing 折算内容高)`
（与 `other` 臂同构，不改写 taffy——taffy 第一趟已按 definite 正确布局）。非 flex/grid
item 路径不入此臂，行为零变化。kill-switch `ZW_FLEX_ITEM_DEFINITE_CB=0`（实测回退 =
旧行为 byte-identical）。

## A/B（reftest-upstream 全语料 vs R4542 基线 1458）

**5 翻绿零新红**：

- `border-shape-overflow-child-clip`（0.02%——R4541 残差项，flex 子 bug 修复直接解锁）；
- `intrinsic-percent-replaced-024/025/027/032`（R4129/R4018 家族——replaced % 高在
  flex/grid indefinite 语境，definite 传播使其走正确 cb_definite 链）。

corpus **15141**/16594 = **91.24%**（R4542 记账 15136 → +5，box-shadow-003 flake 绿相位
维持）。

## 门禁

make test **19,376P/0F**；fmt 干净；clippy 双 feature 组 `-D warnings` 零输出；
product-smoke welcome **15.40% 同值** struct PASS；bench-gate 定向 zero-layout-engine
**GATE PASS**（25 指标）。调试插桩（tree.rs/engine.rs 三处 dump）已全部回退。

## 下轮方向

1. corner-shape 余 6 fail 深域勘察（backdrop-filter ×2/iframe/video/inset-shadow/
   render-corner-shape 多 variant）；
2. 资产激活新红二案勘察（display-none-inline-img / dynamic-filter-changes-001，
   dynamic reftest-wait + filter/JS img 域）；
3. 或守成轮。
