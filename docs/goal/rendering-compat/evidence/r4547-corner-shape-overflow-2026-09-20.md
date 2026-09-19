# R4547 证据：corner-shape × overflow 内形状裁剪——bevel-overflow 翻绿 0.00%，余 5 fail 深域定性

- 日期：2026-09-20（R4547 C 轮——R4546 下轮方向①，painter 1 接线点 +12 行）
- 谱系：R4540 v2 条带机制 → R4541 border-shape overflow 裁剪 → 本轮把 corner-shape
  纳入同一入口（R4544 曾定性「strip 不渲染故不可行」，v2 落地后可行）。

## 实现（painter overflow 裁剪点 +12 行）

R4541 的 overflow 内形状裁剪入口（`clip_with_polygon_rewrite`）追加 corner-shape 臂：
`border_shape_overflow_polygon` 无（非 border-shape 元素）时回落
`shaped_corner_polygon`（R4248 形角化 border-box 轮廓；scoop/负指数 superellipse 返回
None → 回退纯矩形，与既有定界一致）。kill-switch `ZW_CORNER_SHAPE_OVERFLOW=0`（实测
回退 = 方块旧行为 byte-identical）。

## 验证

- **bevel-overflow**：`corner-shape:bevel + overflow:clip + 溢出子层` → 绿色菱形与
  SVG ref **0.00% 像素一致**（原 1.02% fail）；
- **bevel-overflow-composite**（R4544 风险页，translate3d + will-change 子层部分
  相交）：v2 条带渲染出与 ref 同形的切边菱形 → **0.00%**（原 0.67% pass 保持）；
- **A/B（全语料 vs R4546 巡检 1454）**：仅 bevel-overflow 翻绿（-1）+
  box-shadow-003 flake 绿相位（-1）→ **1452**（15142/16594 = **91.23%**），零新红；
  corner-shape-backdrop-filter-video-overflow（squircle + overflow:hidden 既有绿）
  与 corner-shape-overflow-clip-margin（scoop → None）均无扰动。

## 余 5 fail 深域定性（本轮勘察结论）

1. `corner-shape-backdrop-filter`：需 backdrop-filter（backdrop 采样 + 滤镜 +
   形角裁剪）管线——filter-effects 深域，RFC 级；
2. `corner-shape-iframe-border` / `corner-shape-video-border`：replaced 子文档/媒体
   元素内容 + 形角边框——iframe/video 在 reftest 环境的内容渲染域；
3. `corner-shape-inset-shadow`：inset box-shadow 形状跟随——ShadowPrimitive rect
   模型 path 化（R4535 挂账同源深域）；
4. `render-corner-shape`：JS 多 variant + shadow-spread + superellipse 全矩阵——
   JS 域（跨流）+ 形角渲染矩阵覆盖。

**corner-shape 目录 16→5 fail 收敛**（本轮后余 backdrop-filter/iframe/video/
inset-shadow/render-corner-shape 五案全为深域/跨流，无单轮可达项）。

## 门禁

make test **19,376P/0F**；fmt 干净；clippy 双 feature 组 `-D warnings` 零输出；
make reftest（本地 687）0 failed；product-smoke welcome **15.40% 同值** struct PASS；
bench-gate 定向 zero-engine **GATE PASS**（26 指标；首跑因 make test 并窗 loadavg 22
被 bench-report 繁忙中止——守卫按设计工作，空载复跑 PASS）。
