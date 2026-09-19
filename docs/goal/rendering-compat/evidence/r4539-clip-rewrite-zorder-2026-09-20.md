# R4539 证据：clip-path 覆盖改写 z 序修复（原位 op 替换）——corner-shape 凹角簇 10 案翻绿

- 日期：2026-09-20（R4539 C 轮——R4538 下轮方向①定谳转实施，1 painter 修复点 + 1 单测）
- 谱系：R4536 三缺陷（A path_fill 改写 z 序倒挂主犯 / B 条带顶采样 / C 嵌套快照）→ R4537
  draw_order 重放机制 → R4538「退化发射」假设（本轮证伪）。

## R4538「退化发射」假设证伪（基线探针取证）

R4538 的「abspos ::before 背景 fill 以退化矩形发射」结论系**实验代码伪影**（该 dump 取自
已回退的原位条带 splice/draw_order 重建实验态）。本轮在干净基线重渲四探针页：

- before-probe（::before 绿底无 clip）：绿 100×100 **正常**（发射路径无损）；
- clip-probe（::before 自身 clip）：红底 + 绿十字 **正确**；
- clip2-probe（父 clip + ::before 绿底 + 子 clip，= corner-shape-notch-ref 构型）：
  **全红十字、绿色整层消失**（缺陷 A 现形）；
- clip5-probe（真 div 子同构）：同全红（与伪元素无关）。

对照 R4535 遗留 dump（target/reftest-dump 的 notch ref PNG 同为全红十字）→ 基线可见失败
完全由缺陷 A 解释，与发射/layout 无关。

## 修复（Fix A'：原位 op 替换，draw_order 不变式内）

painter/mod.rs clip-path Polygon 臂：covered 填充/圆角矩形改写从「清零 + add_path_fill
尾部追加（渲染序最高）」改为——add_path_fill 后把**尾部追加的 `DrawOp::PathFill` 移回被
改写图元原 op 的位置**：

- z 序随 op 位置保留：背景仍在子树之下（嵌套 clip 页父背景不再盖死子层）；
- 被改写图元 rect 已清零且不再被任何 op 引用；typed 向量零 splice、其余 op 索引零位移
  （R4537 draw_order 重放不变式全程满足，无需整表重建）；
- 找不到原 op（防御）保留尾部追加旧行为；kill-switch `ZW_CLIP_REWRITE_INPLACE=0` 整体
  回退（实测复渲 = 基线 byte-identical）。
- opacity/transform 的 path_fills 不覆盖为既有缺口（改写前后一致，零行为变化）。

## A/B（reftest-upstream 全语料，同机前后台串行）

fail-list diff（基线 1470 → 修复 1459，**11 翻绿 / 0 新红**）：

- **corner-shape 凹角簇 10 案翻绿**：notch 0.30% / notch-mixed 0.78% / scoop 0.61% /
  bevel 0.13% / square-notch-mixed 0.23% / square-notch-superellipse 0.48% /
  superellipse-bevel 0.43% / superellipse-negative-100 0.78% /
  superellipse-negative-inf 0.78% / superellipse-scoop 0.61%——R4535 重实现采样未能收敛的
  簇，真因在 ref 页嵌套 clip z 序（test 页 R4248 形角臂本就正确），一次归位全簇解锁；
  corner-shape 目录 16 fail → 6（余 backdrop-filter / iframe / video / inset-shadow /
  bevel-overflow 1.02% 深域）。
- box-shadow-overlapping-003：家族已知 ±1 相位 flake（R4532 记档），顺带翻绿不算净收益。
- corpus **15135**/16594 = **91.20%**（R4534 基线 15125 = 91.15%，净 +10）。

## 探针复渲（修复后）

- clip2/clip5：绿十字 + 外缘 5px 红边（子多边形 inset 5px、父多边形到边，与 chromium 语义
  一致）；before/clip/clip3 **byte-identical**（非嵌套页零影响）。

## 门禁

- make test **19,367P/0F**（+1 新单测 `test_paint_clip_path_polygon_rewrite_z_order_inplace`：
  双覆盖填充改写后 op[0]=父红 PathFill、op 尾=子绿 PathFill、无 Fill op 残留）；
- fmt 干净；clippy 双 feature 组（默认 v8 / quickjs）`-D warnings` 零输出；
- make reftest（本地 687）0 failed；
- product-smoke welcome **15.40% 同值** struct PASS（零漂移）；
- bench-gate 定向 zero-engine **GATE PASS**（26 指标；首跑 FAIL 系 clippy 并发编译争用
  噪声——bench 页无 clip-path 不经新路径，隔离复测 414µs/82.4µs 远低于预算 657µs/118µs，
  空载复跑官方 PASS）。

## 余段（下轮方向）

1. corner-shape 余 6 fail 深域勘察（backdrop-filter ×2 / iframe-border / video-border /
   inset-shadow / bevel-overflow 1.02% 贴阈值）；
2. clip_fill_to_polygon 条带路径的 op 登记修复（缺陷 B/C——部分覆盖填充的条带仍无 op，
   需不破坏索引的登记方案；corner-shape 语料已清空该需求，border-shape slice 2 overflow
   裁剪若实施再评估）；
3. 或守成轮。
