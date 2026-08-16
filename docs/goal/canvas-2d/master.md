# Canvas 2D 运行时控制面板

**最后更新**: 2026-08-16（R57 第八批定稿：**oracle A/B 诚实化**——canvas 区域对比
+ DC-14 channel 容差 + 环境不支持排除；真通过 2（假测量）→ **9（20.9%）**，不一致
117→27；证据见 evidence/r57-m3-oracle-honest-2026-08-16.md）。

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
| testharness 面（全部目录） | ✅ 全 0 Fail（path-objects 202/0、drawing-images 37/37 等） |
| oracle A/B 147 可测 | ✅ 真通过 **9（20.9%）** / 近似 7（16.3%）/ 不一致 27 |
| oracle 环境不支持排除 | 221 用例（Chromium 150 无 CanvasFilter/beginLayer——tentative API 未实现，捕获帧无效，同 NotRun 语义） |

### Rust 层（crates/canvas）

- ✅ 800 测试全绿；**行覆盖率 91.18%**（≥70% 目标达成）
- ✅ R57：渐变插值空间补全 CSS Color 4 全 16 空间（+DisplayP3/DisplayP3Linear/
  A98Rgb/Rec2020/XyzD50，矩阵+EOTF）；CanvasStyle set_color_interpolation 直通
- （前轮记录见 git log：径向渐变全几何、文本真字体光栅、TextCluster 系列等）

### JS 接线层（js_dom_shim + engine canvas.rs）

- ✅ R57：**setFilterDropShadow + setGradientInterpolation bridge op 补齐**（R56h
  漏分发的同模式 2 处——shim 已发 + Rust API 已有 ≠ 链路通）；dropShadow 字符串
  filter 形式接线（`ctx.filter='drop-shadow(...)'`）+ filter 列表顶层逗号分割；
  **createElement('canvas') DOM 集成**（standalone canvas 同步 host handle +
  data-zw-canvas-ctx + 属性同步——append 可进布局）

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
| G10 | oracle A/B 不一致 27 项 | 🔄 聚类：grid 结构 ~15 项（22px IFC 偏移，深项）/ fontKerning 8.5%（字体度量，rendering-compat 域）/ drop-shadow AA 4.8%（无抗锯齿）/ text-outside 0.55%（退化 oracle） |

## 待用户决策清单

- [x] G5 DOM img 源 — ✅ 完成
- [x] ImageBitmap 全源类型 — ✅ 完成
- [x] shadowColor 'currentColor' — ✅ 完成
- [x] OffscreenCanvas × Web Worker 集成（G6）— ✅ 完成
- [x] index-from-offset 边界约定 — ✅ 完成
- [x] R56h 遗留 bridge 接线缺口（setFilterDropShadow/setGradientInterpolation）— ✅ R57 补齐
- [ ] **grid 结构用例 22px IFC 偏移**（gradient/composite.grid/TextCluster ~15 项）——
  canvas 盒在 R109 匿名片段内 y 偏移 22px（IFC 行内定位深层问题；探针证实 IFC
  run.y=0 正确但最终盒 y=20）——深结构项，待用户点名
- [ ] 抗锯齿光栅（AA 边差 280px 级——drop-shadow 4.8%/reset 0.17% 的近似残差）
- [ ] serif 字体度量对齐（fontKerning.none2 8.5%——rendering-compat 域）

## 下一步计划

1. **grid 结构 22px 偏移**（G10 最大聚类）：IFC sync/valign 链深挖——canvas 盒在
   R109 匿名片段内最终 y=20（IFC run.y=0 正确）——待决策后开工
2. oracle A/B 持续回归门：每轮 canvas 改动跑 `REFTEST_INCLUDE_CANVAS=1 make
   reftest-oracle canvas`（测量法已诚实，数值可追踪）
3. 浏览器 app form/input 快照测试（7）——本环境既有失败，浏览器流（非 canvas 面）处理

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge/canvas.rs` 核对 html-compat 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT canvas 基线建立 | ✅ 完成（919 文件导入，testharness 面 832/832 全绿） |
| M2 — API 语义补齐 | ✅ 完成（Path2D/OffscreenCanvas 主线程+worker/ImageBitmap/drawing.style/text 全系；G7 全灭） |
| M3 — 像素正确性冲刺 | 🔄 oracle A/B 诚实化完成（R57）：canvas 区域对比基线 真通过 9（20.9%）/ 不一致 27；剩余聚类 = grid 22px 偏移（~15 项，深项待决策）+ 字体/AA 残差 |

## 验证基线

- 测试基线：canvas **800** 全绿；layout **1380**；engine **2156**；webview **599**；wpt-runner 171；行覆盖率 ≥70% 达标
- WPT canvas testharness 面：全目录 0 Fail（含 path-objects 202/0、drawing-images 37/37）
- oracle A/B：147 可测（221 环境不支持排除）——真通过 9（20.9%）、近似 7、不一致 27
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过（零警告）
- **既有失败（非 canvas 面，a08d3064 复测确认）**：browser 4 个 form/input 快照测试
  （default_actions_work_without_javascript / form_fixture_complete_multiprocess_semantics /
  gpu_compositor_path_dispatches_input_events_to_form_controls /
  local_composite_cpu_gpu_matrix_for_form_interactions——表单提交导航/multiprocess/GPU
  输入路径，浏览器流处理；本环境既有，canvas 改动 A/B 无影响）
- 资产化：修复经 fetch-canvas-subset.sh 资产化（wpt-data 独立 repo 机制，gitignored）
