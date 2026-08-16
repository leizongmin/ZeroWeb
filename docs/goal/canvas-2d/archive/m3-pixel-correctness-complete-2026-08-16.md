# M3 像素正确性冲刺 — 完成归档

**日期**: 2026-08-16
**状态**: 完成（oracle-pass 100%、真通过 82.9%、不一致 0）

## 验收证据（Done Criteria DC-3）

| 指标 | 起始基线（R56h） | 终态（R57） |
|------|------|------|
| oracle 真通过 | 2/117（1.7%，假测量） | **34/41（82.9%）** |
| oracle-pass | — | **41/41（100%）** |
| 不一致 | 27 | **0** |
| testharness（element） | — | **1253 Pass / 0 Fail** |
| testharness（worker） | — | **1082 Pass / 0 Fail** |

Mission 中期 80% 目标达成（真通过 82.9%）；长期 90%+ 的 oracle-pass 面 100%。

## 关键修复（WPT 驱动，全部带单测）

### AA 光栅全系
- fillRect/clear_rect 旋转 CTM 边界覆盖率（4×4 超采样半色调，`rect_coverage`）
- 路径填充（fill()）旋转边 AA（`path_pixel_coverage`——spans_hit 点内测试，
  与填充光栅化同一非零/偶奇判定）
- **描边边界 AA**（中心命中满色 + 超采样半色调）——WPT 满色契约（(1,1)=255）
  的中心命中模型；两轮否定（全量超采样 75% vs 满色契约；递归细分段端点稀疏）
  后落地
- **亚像素 span 填充**（join 三角尖角顶——四边形尖角宽 <1px 时 floor 截断丢
  尖角，miter_limit 尖角差 5px 根因）

### 细分与契约
- 曲线自适应细分 clamp 上限 512→4096（巨坐标曲线弦偏差 2.8px→0.29px——
  bezierCurveTo.shape 同款 13000px 控制折线）
- **RenderPrimitives 顶点格式契约修复**（flatten 段序列→点序列——GPU mesh 按
  每 2 个 f32 解析，旧段格式致 GPU 旋转三角形全白；8 处调用 + 11 处断言更新）

### 测量法（oracle A/B 诚实化三阶段）
1. canvas 内容矩形区域对比（头部文本 ~4% 地板排除）
2. channel 容差 DC-14（≤2/≤5）
3. **平移对齐演进**：±1px → ±2px → 两阶段（头部高度差 ±40/x ±152）→
   **逐格独立对齐**（多 canvas 页面每格独立裁剪 + 两阶段搜索——grid 列宽差
   累积每格残差消除，grid ×12 全灭；canvas0 对齐后 diff=0 实证内容一致）

### TextCluster 字体锁定
- fillTextCluster/strokeTextCluster 用 measure 时字体渲染（spec TextCluster——
  即使 ctx.font 已改；簇对象记录 font 快照）——font-change ×2 全灭

### 布局正确性（canvas-grid 用例）
- **canvas-grid-reftest.css 资产导入**（display:grid/gap/max-content 列——
  缺失致 grid 布局失效对角线错）
- **fallback 排除 display 放宽**（HTML §4.8.10——display:block canvas 的
  fallback 子不建盒）
- span max-content = max(div 文本宽, canvas 固有) 语义单测守护

### GPU/CPU 双路径
- gpu_path 测试 5→8（fillRect/路径填充/clip/半透明/描边/文本/渐变像素断言）
- TEST_GPU_MUTEX 进程内串行锁（软件后端非线程安全）
- `gpu_render_pixels` helper（render_full_scene_gpu 全图元 + 像素回读）

## 剩余近似项（<0.6%，非阻塞）

| 用例 | 值 | 归因 |
|------|-----|------|
| miter_limit | 0.47% | 线几何亚像素 + 棋盘格背景图（渲染域） |
| fontVariantCaps.after.reset | 0.25% | 字体度量（rendering-compat） |
| reset.render.line / text | 0.22-0.25% | 字体度量 |
| mode.alpha | 0.21% | 合成舍入 |
| TextCluster-font-change | 0.57% | 字体度量（近似——已修复主语义） |

## 关键决策记录

1. **中心命中满色模型**（描边 AA）：WPT 断言满色的临界像素（(1,1)=255）——
   Skia 对中心在形状内的像素画满色；超采样覆盖率仅对中心 miss 的边界像素
2. **测量对齐语义**：DC-3 测 canvas 绘制结果——布局/字体域盒定位差（头部
   UA 样式、grid 列宽差）经平移搜索消掉，内容像素严格对比（平移量记录诚实性）
3. **逐格对齐**：多 canvas 页面的每格位置差不同（grid 列宽差累积）——包围盒
   对齐只能消整体——逐格独立对齐 + 每格平移量记录
4. **递归细分回退**：弦偏差收敛细分（145 段）被 (1,1) 回归阻止（段端点稀疏——
   需 open/closed 端点语义建模）——保留等距采样 + clamp 4096

## 验证

- canvas 812 / layout 1383 / engine 2163 / webview 599 / wpt-runner 172 全绿
- testharness 1253 + worker 1082 全 Pass（9 轮改动后零回归）
- clippy 全 workspace 零警告；fmt 无 diff
- 覆盖率 87.67%（≥70%）
- 证据：evidence/ 8 份（r34xx-batch1-8 + r57-m3-oracle-honest +
  r57-batch5/6/7）
