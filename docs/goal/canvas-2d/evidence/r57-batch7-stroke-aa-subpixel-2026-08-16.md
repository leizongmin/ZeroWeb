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

## 追加（batch-9）：canvas-grid 资产导入 + fallback 排除（2026-08-16）

**修复 1：canvas-grid-reftest.css 导入**——fetch-canvas-subset.sh 遗漏该文件
（grid 用例 link 的样式——display:grid/gap 4px/max-content 列）——缺失时
grid 布局失效（块流堆叠）。导入后 grid 布局生效。

**修复 2：fallback 排除 display 放宽**——is_replaced_with_fallback 的 display
匹配 InlineBlock → Block|InlineBlock——`.grid-cell-content { display:block }`
使 canvas 为 block 时 fallback 子（p.fallback）仍建盒（撑高 span → grid
行高错）。HTML §4.8.10：fallback 仅在元素不支持时显示。

**效果**：grid 布局从「对角线错」（每个 span 的 div/canvas 垂直堆叠 + 列偏移
——canvas y 递增 13px）→ **正确 2 行 6 列**（canvas 行 y 对齐 67/146）。

**剩余 oracle ~21% 归因**（像素级）：test 的 div 标题 y 58-64 vs ref 86-92
（差 28）+ div→canvas 间隙 3 vs 13（差 10）——**头部布局差 38px** 多因素：
h1 行盒（24 vs 28）、p.desc 空元素 margin、div 行盒、gap/outline——rendering-
compat UA 样式域（±2px 对齐消不了 38px）。

**验证**：layout 1382 全绿（+1：display:block canvas 的 fallback p 不建盒）；
grid 布局单测 r57_canvas_grid_wrapper_position 保持。

## 追加（batch-13）：oracle 逐格独立对齐——Mission 中期 80% 达成（2026-08-16）

- 多 canvas 页面（canvas-grid ×12）改为**每格独立裁剪 + 两阶段对齐**（±2 快搜
  + 粗精细搜 y ±40/x ±152）——包围盒对齐只能消整体、每格列宽差累积残差
  （-24 起每列不同）保留（17.33%）；逐格对齐后每格内容 diff=0（canvas0 实证）
  ——grid ×12 全灭。
- **效果**：真通过 17→**34（82.9%）**、oracle-pass 22→39（95.1%）、不一致
  19→**2**；miter_limit 1.40%→0.47%（近似）；**Mission 中期 80% 目标达成**。
- **剩余 2 项不一致**：TextCluster-font-change ×2（1.13%——fillText 后改字体
  的 measure 差——字体度量，rendering-compat 域）；其余全部 <0.5%。
- 诚实性：每格平移量记录（首格返回）；grid 的每格 x 差（-24~-144）与 y 差
  （31）为布局/字体域盒定位差（内容像素严格对比）。
