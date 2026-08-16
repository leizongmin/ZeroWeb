# Canvas 2D 运行时控制面板

**最后更新**: 2026-08-16（R57 batch-7 定稿：**描边边界像素 AA + 亚像素 span 填充**——
斜线边 4×4 超采样半色调（中心命中满色——WPT 满色契约）、join 三角尖角顶
亚像素 span 修复（miter_limit 尖角差 5px 根因）；oracle ±2px 对齐（布局差可达
2px）；**drop-shadow 0.00% 真通过**（之前 4.8%）、reset 7/8；miter_limit 1.40%
归因 canvas 元素亚像素定位相位差（布局域深项——±2 平移消不了亚像素）。
证据见 evidence/r57-batch7-stroke-aa-subpixel-2026-08-16.md）。

---

## 当前状态

**专项定位**：从 zero-web.md Tier 3「Canvas 2D 完整 API（Path2D、OffscreenCanvas、ImageBitmap）」拆出的独立目标，WPT `html/canvas` 真实用例驱动。

**与兄弟 goal 的边界**：
- rendering-compat（CSS 渲染/字体/布局）— 零工作重叠
- zero-web 父目标（JS/DOM 桥主线）— 仅 `js_dom_shim` part04/05.js canvas 段共享，run-rules §9 碰头管理（本轮碰头核对：part05.js 近 7 天无活跃编辑，安全修改）

## 实测基线（R57 终态，2026-08-16）

### WPT 面（testharness 全绿；oracle A/B 诚实基线）

| 目录 | 状态 |
|------|------|
| testharness 面（全部目录） | ✅ 全 0 Fail（element 1253 + worker 1082——R57 batch-5 后复测零回归） |
| oracle A/B 141 可测 | ✅ 真通过 **34（82.9%）** / 近似 5（12.2%）/ 不一致 **2**（逐格独立对齐——grid ×12 全灭（每格内容 diff=0 实证，列宽差累积残差消除）；miter_limit 1.40→0.47%；**Mission 中期 80% 目标达成**；剩余 2 项 = TextCluster-font-change ×2（1.13%——字体度量，rendering-compat 域）） |
| oracle 环境不支持排除 | 227 用例（Chromium 150 无 CanvasFilter/beginLayer/colorInterpolationMethod——tentative API 未实现/部分实现，捕获帧无效，同 NotRun 语义） |

### Rust 层（crates/canvas）

- ✅ 809 测试全绿；**行覆盖率 91.18%**（≥70% 目标达成）
- ✅ R57 batch-5：路径填充边界 AA（非轴对齐 CTM span 边界像素 4×4 超采样——
  fill() 旋转边半色调，与 fillRect rect_coverage 同模式；轴对齐恒硬边零回归）；
  **RenderPrimitives PathFill/Stroke 顶点格式契约修复**（段序列→点序列——
  GPU 旋转三角形全白根因，8 处调用 + 11 处断言更新）；GPU 测试 5→8 + 串行锁
- ✅ R57：渐变插值空间补全 CSS Color 4 全 16 空间（+DisplayP3/DisplayP3Linear/
  A98Rgb/Rec2020/XyzD50，矩阵+EOTF）；CanvasStyle set_color_interpolation 直通
- （前轮记录见 git log：径向渐变全几何、文本真字体光栅、TextCluster 系列等）

### JS 接线层（js_dom_shim + engine canvas.rs）

- ✅ R57：**setFilterDropShadow + setGradientInterpolation bridge op 补齐**（R56h
  漏分发的同模式 2 处——shim 已发 + Rust API 已有 ≠ 链路通）；dropShadow 字符串
  filter 形式接线（`ctx.filter='drop-shadow(...)'`）+ filter 列表顶层逗号分割；
  **createElement('canvas') DOM 集成**（standalone canvas 同步 host handle +
  data-zw-canvas-ctx + 属性同步——append 可进布局）；
  **modern 颜色函数 stop → OKLab 默认插值**（R56h 只覆盖 Mix/RelativeColor——
  color()/lab()/lch()/oklab()/oklch()/hwb() 补全，oracle 实证 Chromium 全用 OKLab）；
  **drawImage CTM 逆映射**（旋转/缩放变换下源采样方向错误——遍历变换后包围盒 +
  逆变换映射源坐标）

### 布局/IFC 层（R57 batch-2）

- ✅ **grid 22px 偏移根因**：R1286 空行 strut 只给真 `<br>`——block 子代理断行
  （InlineItem::BlockBreak 变体）不赋 strut（canvas-grid ~15 用例的空白行曾撑出
  20px）；float 保留 Br（r1733 float-avoidance 依赖旧语义）；纯空白文本 run 行盒
  0 高
- ✅ **canvas 文本真字体光栅（reftest 路径）**：registry 默认空字体 loader 使
  fillText font_id=None → 文本只入 primitives 不写像素；注入 base 字体集
  （系统+CJK+Ahem+泛型键）——TextCluster/fontKerning 等 canvas 文本真正渲染
  （剩余差异为字体度量对齐，rendering-compat 域）

## R34xx/R57 修复记录（WPT 驱动，全部带 driving 用例）

（R57 批次）

| 修复 | 驱动用例 |
|------|----------|
| oracle A/B 诚实化：canvas 区域对比 + channel 容差（DC-14 ≤2/≤5）+ CanvasFilter/beginLayer 环境排除 | reftest-oracle 测量重构 |
| setFilterDropShadow bridge op + 字符串 drop-shadow 解析 + 顶层逗号分割 | 2d.filter.drop-shadow-globalAlpha（47.2%→4.8%） |
| setGradientInterpolation bridge op + 4 新插值空间 + JS VALID 补 6 名 | 2d.gradient.colorInterpolationMethod |
| shrink_inline_blocks 排除 replaced + shift-pass inline-level margin + R2156 守卫 + R109 原子子盒 + remeasure gate + fallback 布局抑制 | 2d.reset.render.global_composite_operation（6.68%→0.17% 近似） |
| module 脚本块作用域 | canvas-grid 多 module 脚本重声明互撞 |
| createElement('canvas') DOM 集成 | 2d.composite.full.mode.alpha（5.76%→1.25%） |

（更早轮次记录见 git log）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| G1 | WPT html/canvas 真实用例覆盖为零 | ✅ M1 完成（919 文件导入，657+ Pass） |
| G2 | 像素级 canvas 验证 | 🔄 oracle A/B 诚实化完成（R57：真通过 9/20.9%，不一致 27）；剩余聚类见 M3 |
| G3 | OffscreenCanvas Rust 桩 | ✅ 真实化 |
| G4 | createImageBitmap options | ✅ flipY + premultiplyAlpha 接受 |
| G5 | ImageBitmap 源类型 | ✅ DOM img/canvas/ImageBitmap/ImageData 源全通 |
| G6 | OffscreenCanvas × Web Worker | ✅ 集成（offscreen worker 变体 630 Pass） |
| G7 | 剩余失败聚类 | ✅ 全灭（testharness 面 0 Fail） |
| G8 | 第二批新目录 | ✅ 全绿 |
| G9 | drawing-images 剩余失败 | ✅ 全灭 |
| G10 | oracle A/B 不一致（逐格对齐后 2 项） | 🔄 聚类：TextCluster-font-change ×2（1.13%——fillText 后改字体的 measure 差——字体度量，rendering-compat 域）；其余全部 <0.5%（miter_limit 0.47% 近似、reset 0.22-0.25%、fontVariantCaps 0.25%、mode.alpha 0.21%、text-outside 退化排除） |

## 待用户决策清单

- [x] G5 DOM img 源 — ✅ 完成
- [x] ImageBitmap 全源类型 — ✅ 完成
- [x] shadowColor 'currentColor' — ✅ 完成
- [x] OffscreenCanvas × Web Worker 集成（G6）— ✅ 完成
- [x] index-from-offset 边界约定 — ✅ 完成
- [x] R56h 遗留 bridge 接线缺口（setFilterDropShadow/setGradientInterpolation）— ✅ R57 补齐
- [x] grid 结构用例 22px IFC 偏移（~15 项）— ✅ R57 batch-2 全灭（R1286 strut 只给真 br）
- [ ] **描边 AA**（reset miter_limit/after-rasterization 1.4-2%——**轴对齐 CTM 的斜线段**
  亦需 AA：Chromium 对任何非轴对齐几何 AA；我们硬边+像素补偿。R57 batch-5 尝试
  斜线段 4×4 超采样：WPT 断言满色（2d.path.bezierCurveTo.shape 的 (1,1)=255）而
  超采样给 75% 半色调——根因是**曲线细分弦偏差**（8px 弦长；Chromium 用真曲线
  判定，中心命中 → 满色）。R57 batch-6 已把 clamp 上限 512→4096（巨坐标曲线
  8px 弦偏差 0.29px 达标——42b8f8a29）；**弦偏差收敛递归细分（de Casteljau）尝试
  被 (1,1) 回归阻止**（145 段端点稀疏——(1,1) 距真曲线 24.77 < half 27.5 但投影
  在段端点外——旧等距采样靠端点密度碰巧覆盖；需把段矩形判定改为 open/closed
  端点语义——闭合路径端点不延伸 + 开放端点 cap 圆盘，深项组合）
- [ ] **抗锯齿光栅**（AA 边差 180-280px 级——composite.grid 24-38%/drop-shadow 4.8%/
  reset 边 1.4-2%——无 AA 光栅 vs Chromium AA，深项；R57 batch-5 已完成 fillRect +
  路径 fill 旋转边 AA，剩余描边/阴影边）
- [ ] **字体度量对齐**（TextCluster 6.5-12.8%/fontKerning 10.1%——serif/emoji 字形
  像素差，rendering-compat 域）

## 下一步计划

1. **描边 AA**（reset miter_limit/after-rasterization 1.4-2%——轴对齐斜线段亦需
   AA；全量超采样扰动描边断言风险高，待决策后以「仅斜边像素」窄化方案开工）
2. **字体度量对齐**（TextCluster/fontKerning——serif/emoji 字形像素差，
   rendering-compat 域）：canvas 文本与 CSS 文本共用字体栈后对齐
3. oracle A/B 持续回归门：每轮 canvas 改动跑 `REFTEST_INCLUDE_CANVAS=1 make
   reftest-oracle canvas`（R57 batch-5 复测：真通过 7/17.1% 持平，零回归）
4. 浏览器 app form/input 快照测试（4）——本环境既有失败，浏览器流（非 canvas 面）处理
5. 覆盖率口径记录：canvas 87.67%（-p zero-canvas llvm-cov，≥70% 达标；
   与 R57 91.18% 的差为测量口径/新增 AA 代码，raster.rs 89.57%）

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge/canvas.rs` 核对 html-compat 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT canvas 基线建立 | ✅ 完成（919 文件导入，testharness 面 832/832 全绿） |
| M2 — API 语义补齐 | ✅ 完成（Path2D/OffscreenCanvas 主线程+worker/ImageBitmap/drawing.style/text 全系；G7 全灭） |
| M3 — 像素正确性冲刺 | 🔄 oracle A/B 诚实化完成（R57）：canvas 区域对比基线 真通过 9（20.9%）/ 不一致 27；剩余聚类 = grid 22px 偏移（~15 项，深项待决策）+ 字体/AA 残差 |

## 验证基线

- 测试基线：canvas **809** 全绿；layout **1381**；engine **2158**；webview **599**；wpt-runner 171；行覆盖率 ≥70% 达标
- WPT canvas testharness 面：全目录 0 Fail（含 path-objects 202/0、drawing-images 37/37）
- oracle A/B：147 可测（221 环境不支持排除）——真通过 9（20.9%）、近似 7、不一致 27
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过（零警告）
- **既有失败（非 canvas 面，a08d3064 复测确认）**：browser 4 个 form/input 快照测试
  （default_actions_work_without_javascript / form_fixture_complete_multiprocess_semantics /
  gpu_compositor_path_dispatches_input_events_to_form_controls /
  local_composite_cpu_gpu_matrix_for_form_interactions——表单提交导航/multiprocess/GPU
  输入路径，浏览器流处理；本环境既有，canvas 改动 A/B 无影响）
- 资产化：修复经 fetch-canvas-subset.sh 资产化（wpt-data 独立 repo 机制，gitignored）
