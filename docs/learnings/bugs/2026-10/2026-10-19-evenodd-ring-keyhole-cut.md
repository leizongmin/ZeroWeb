---
date: 2026-10-19
modules: engine
---

# even-odd 填充串联内外多边形不切 keyhole 会撕出桥隙

## 问题

R5180 画 border-radius 弧带（外圈 rounded 多边形 + 内圈 rounded 多边形拼接成
环）用 `outer.extend(inner)` 直接串联后交给 CPU `render_path_fill`（even-odd
扫描线）：正方形角案带内出现 385px 竖向背景缝，角部 diff 0.03%→0.08%。

## 根因

even-odd 只数扫描线与边的交点奇偶，不管边来自哪个多边形。朴素串联在每个
接缝处生成两条「桥边」（outer 末点→inner 首点、inner 末点→outer 首点），两条
桥边跨同一扫描线的 x 不同（本例 14.39 vs 12.72），交点无法配对 → 该行奇偶
错乱，带内被撕出一条背景缝。这不是 even-odd 的 bug，是「多边形环」的拓扑
只能靠闭合边表达，而闭合边不能随便挑位置。

## 解决

keyhole 切口（`cut_polygon_at_left_edge`）：把每个多边形在「闭合边中点」
（约定首点=TL 起点、末点=BL 终点 → 左边中点）处剪开，首尾各插入复制顶点，
再串联。两条桥边重合为同一 x 上的反向边对，只影响带内扫描线且交点两两配对，
环带正确。参考 PDF/字体渲染的同名技巧。

消费端差异：
- `render_path_fill`（path_fill primitive）天然吃 keyhole 串联。
- 整行 rect 条带消费端（ImagePrimitive clip 等）用 `round_band_row_strips`
  在同一多边形上复刻 path_fill 的扫描线算法取行 span，保证与 path_fill
  逐像素一致（像素同源性，非近似）。

## 避免

- 给 even-odd 填充喂「多环」顶点集时，永远检查拼接处闭合边的走向；两条
  桥边 x 不同 = 必撕。多边形顶点序约定（首=左上起、逆时针）是切口位置的
  前提，改动采样起点要同步切口。
- 验证手段：渲染单页后逐行找带内背景色 run（本例 green[8,11]/black[12,14]/
  green[15,97] 直接定位桥边 x）。
