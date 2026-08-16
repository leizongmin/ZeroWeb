# R57 batch-7 证据：描边边界像素 AA + 亚像素 span 填充（2026-08-16）

## 修复 1：描边边界像素 AA（中心命中满色 + 超采样半色调）

- 斜线描边的段矩形判定拆分为两级：中心命中（距中心线 ≤ seg_half 且投影 ∈
  [0,1]）→ 满色（WPT 满色契约——bezierCurveTo.shape 的 (1,1)=255 等临界
  像素）；h2 像素方形补偿命中（中心距 ∈ (seg_half, seg_half+0.5]）→ 4×4
  超采样覆盖率（子采样点距中心线 ≤ seg_half 且投影 ∈ [0,1]）→ 源 alpha ×
  coverage 半色调（Chromium/Skia 对临界像素按覆盖面积混合）。
- 与 batch-5 否定版本的区别：batch-5 对**全部**命中像素超采样（中心命中的
  (1,1) 也被降为 75% 半色调——WPT 断言满色 Fail）；本版中心命中保持满色。
- 单测：45° 斜线（half=20）边界半色调 + 内部满色。

## 修复 2：亚像素 span 填充（join 三角尖角顶）

- 根因：blit_path_to_pixels 的扫描线 span 填充用 floor/ceil 截断——miter
  join 四边形在尖角处宽 < 1px 时 span [10.39,10.39] → [10,10) 空——尖角顶
  丢失（2d.reset.render.miter_limit 的 miter 尖角 vs Chromium 差 5px——
  本地实测尖角顶 y=11 vs 理论 1.4；修复后 y=2 ✓）。
- 修复：亚像素 span（full_start ≥ full_end 且 sx < ex）填与 span 相交的像素
  （k < ex && k+1 > sx——floor(sx) 恒交 + floor(ex) 当非整数且不同）。
- 单测：对称 V 形折线的 miter 尖角顶（y=1/2）着色。

## 修复 3：oracle 对齐 ±1px → ±2px

- 布局域盒定位差可达 2px（miter_limit 实测 canvas 内容下移 2px——IFC 行高
  累积），±1px 搜索消不干净。25 种平移搜索；平移量分布统计更新。

## oracle 复测（141 可测）

| 指标 | batch-6 | batch-7 | 变化 |
|------|------|------|------|
| 真通过 | 12（29.3%） | 12（29.3%） | 持平 |
| 近似 | 4 | 4 | 持平 |
| 不一致 | 25 | 25 | 持平 |
| drop-shadow | 0.00% | 0.00% | **真通过**（batch-5 前 4.8%） |
| reset | 7/8 | 7/8 | — |
| miter_limit | 1.40% | 1.40% | 归因见下 |

**miter_limit 1.40% 归因**（dump 像素级分析）：test/ref 的折线灰度带（AA 边）
位置差——**canvas 元素亚像素定位的相位差**（我们的布局 y 浮点 vs Chromium
整数——内容绘制跨像素网格 → 线带边缘半色调 vs 硬边）——±2px 平移消不了
亚像素相位（batch-4「0.3px 覆盖相位差」同根因）——布局域深项（R834/R631）。

## 验证

- canvas 812 全绿（+2）；testharness 1253 + worker 1082 零回归
- clippy 零警告；fmt 无 diff
- 与前轮关系：batch-6 clamp 4096（弦偏差前提）→ batch-7 描边 AA 落地
  （中心命中模型不破坏 (1,1)——(1,1) 覆盖来自 join，不在段矩形判定内）
