# R57 batch-6 证据：oracle 测量法 ±1px 平移对齐（2026-08-16）

## 问题

oracle A/B 的 canvas 区域对比用**本渲染布局定位**的 canvas 盒位置裁剪两边帧。
布局域盒定位差（IFC strut/行高近似——R834/R631 深项）使 canvas 盒 y 系统性差
1px（R57 batch-3 实测 ref 114 vs 我们 115）——裁剪区域错位把布局差泄漏进
canvas 内容测量（composite.grid 24-38%、reset/text 部分用例的主导差异）。
DC-3 测的是 **canvas 绘制结果**，页面家具的布局差不应计入。

## 修复（wpt-runner reftest-oracle）

- `compare_pixels_shifted`（reftest_compare.rs）：fb1 相对 fb2 平移 (dx,dy) 的
  像素对比——内容整体平移 = 盒定位差；非重叠边缘忽略（平移搜索只对齐内容）。
- canvas 区域（`canvas_rects` 非空）±1px 平移搜索最优对齐：9 种平移取最小 diff；
  非 canvas 页面保持全页对比（零行为变化——CSS reftest 不受影响）。
- 诚实性：平移量 (dx,dy) 返回并统计输出（「对齐平移分布」）——平移消掉的是
  布局域整体盒定位差，内容像素仍严格对比（channel 容差同前）；局部绘制错位
  不会被整体平移掩盖。

## 结果（oracle A/B 复测，141 可测）

| 指标 | batch-4/5 基线 | batch-6（对齐） | 变化 |
|------|------|------|------|
| 真通过 | 7（17.1%） | **12（29.3%）** | +5 |
| 近似通过 | 7（17.1%） | 4（9.8%） | -3 |
| 不一致 | 27 | 25 | -2 |
| oracle-pass | 14（34.1%） | 16（39.0%） | +2 |
| reset 目录 | 6/8 | **7/8（88%）** | +1 |
| **平移分布** | — | **0=0 / ±1=41（100%）** | 全部用例有 1px 盒定位差 |

- 平移分布 41/41 全 ±1：证实布局 1px 偏移是**系统性**的（所有 WPT canvas 页面
  头部 h1/p.desc 行高差），此前一直泄漏进 canvas 测量——对齐后测量回到 DC-3
  语义（canvas 绘制结果）。
- text 5/16 不变：字体度量差异是内容级（非平移）——聚类归属正确。
- composite.grid 0/13 不变（23-37.5%）：对齐后仍主导——**格子内相对布局差 +
  AA 边**（非整体 1px），深项组合（R834/R631 + 描边/旋转边 AA）。

## 验证

- wpt-runner 172 全绿（+1：compare_pixels_shifted 对齐语义单测——整体偏移
  1px 帧经 (0,-1) 对齐 diff 归零、错位平移产生 diff 不掩盖）
- clippy 零警告（type_complexity 用 OracleResult 别名解决）；fmt 无 diff
- canvas 809 / wpt-runner 172 全绿；CSS reftest 面零行为变化（非 canvas 页面
  走原全页对比路径）

## 追加：composite.grid 残留根因（像素级分析，2026-08-16）

±1px 平移对齐后 composite.grid 仍 23-37.5%——像素级 dump 分析（REFTEST_DUMP
+ 绿 over 蓝/蓝平坦区 bbox 对比）：

- **canvas 内容光栅精确**：本地验证旋转矩形 = (7,8)-(43,38) 轴对齐（CTM 数学
  f32 误差 ~1e-8；x=20 列绿色像素 y∈[8,37] 精确）——canvas 域无偏移。
- **grid 行定位非均匀差**：蓝平坦区行带 test [(114,143),(351,380),(429,458),
  (588,599)] vs ref [(113,142),(359,388),(441,470)]——行 1 差 1px、行 2 差 8px、
  行 3 差 12px；每带高度一致（30px）——**grid 行高渲染兼容性差**（span>div+
  canvas 的 grid item 高度：div 标题行高 + canvas 盒——rendering-compat 域）。
- 每格内容形态一致（绿 over 蓝/绿 over 白/蓝平坦区结构相同），仅行位置不同。

**结论**：composite.grid ×12 = grid 行高布局差（非 canvas 绘制）——±1px 对齐
只能消包围盒级统一偏移（行 1），行 2/3 的 8-12px 相对差在 canvas 专项域内无解，
归 rendering-compat 域（grid 行高渲染兼容性）。
