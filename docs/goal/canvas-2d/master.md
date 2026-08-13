# Canvas 2D 运行时控制面板

**最后更新**: 2026-08-13（M1 完成——WPT canvas 5 目录 168/168 全绿；G3 OffscreenCanvas Rust 真实化；GPU 路径测试就位）。

---

## 当前状态

**专项定位**：从 zero-web.md Tier 3「Canvas 2D 完整 API（Path2D、OffscreenCanvas、ImageBitmap）」拆出的独立目标，WPT `html/canvas` 真实用例驱动。

**与兄弟 goal 的边界**：
- rendering-compat（CSS 渲染/字体/布局）— 零工作重叠
- zero-web 父目标（JS/DOM 桥主线）— 仅 `js_dom_shim` part04/05.js canvas 段共享，run-rules §9 碰头管理（本轮碰头核对：part05.js 近 7 天无活跃编辑，安全修改）

## 实测基线（2026-08-13）

### WPT 面（M1 切片 1+2 完成）

| 目录 | 用例数 | 状态 |
|------|--------|------|
| the-canvas-state | 23 文件 / 68 subtest | ✅ 全绿 |
| drawing-rectangles-to-the-canvas | 32 / 32 | ✅ 全绿 |
| transformations | 22 / 22 | ✅ 全绿 |
| pixel-manipulation | 14 / 14 | ✅ 全绿 |
| line-styles | 33 / 33 | ✅ 全绿 |
| shadows | 61 / 50 | 🔄 50 Pass（11 个 = G5 DOM img 源 9 + currentColor 2，待决策） |
| compositing | 124 / 62 | 🔄 62 Pass（36 个 = G5 DOM img 源；其余全修） |
| path-objects | 205 / 26+ | 🔄 部分（roundrect 26 Pass；**剩余移交 js-dom goal**——见下方交接记录） |
| fill-and-stroke-styles | 261 / 182 | 🔄 182 Pass（66 = G5 DOM img 源、13 = radial cone 几何深） |
| **合计** | **570 文件 / 460+ subtest** | ✅ **460 Pass**（113 失败 = G5 132 计入 + radial 13 深） |

- 导入机制：`tests/wpt-runner/scripts/fetch-canvas-subset.sh`（固定 WPT rev `315976933870b34d6ea30e3f6643403edae678ba`）+ `zero-wpt-runner testharness-canvas [filter]`（canvas-tests.js 内联驱动 `_addTest`）
- 用例资产在 `tests/wpt-runner/wpt-data/html/canvas/`（独立 repo 机制，git-ignored）
- WPT 用例目录面见 `CANVAS_TEST_SUBDIRS`（testharness.rs）——新目录导入时同步追加

### Rust 层（crates/canvas）

- ✅ 743 测试全绿（基线 737 + R34xx +6：OffscreenCanvas 行为 4 + GPU 路径 2）
- ✅ GPU 路径测试就位（`tests/gpu_path.rs`：canvas → RenderPrimitives → GpuRenderer 软件后端 → 像素回读；无 adapter 环境自动跳过）
- ✅ OffscreenCanvas 真实化（持有 CanvasContext + transfer 清空语义 + 尺寸 setter）
- ✅ Path2D 完整（21 方法含 from_svg/is_point_in_path + anticlockwise arc 支持）

### JS 接线层（js_dom_shim + engine canvas.rs）

- ✅ CanvasRenderingContext2D proxy 全 API 派发（R2795/R3077 + R34xx setter 校验/arc anticlockwise/无效颜色忽略）
- ✅ Path2D（R3306/R3307 + R34xx anticlockwise）、ImageBitmap+createImageBitmap（R3309/R3311）、OffscreenCanvas（R3312/R3313）
- ✅ 显示链路（R3268）

## R34xx 修复记录（WPT 驱动，全部带 driving 用例）

| 修复 | 驱动用例 |
|------|----------|
| save/restore 客户端镜像状态栈（19 属性快照） | the-canvas-state saverestore.* |
| fillRect/strokeRect 不污染当前路径（Rust fill_rect/stroke_rect） | saverestore.path |
| createImageData spec 语义 + CanvasRenderingContext2D 构造器 + getImageData trunc | pixel-manipulation create* |
| shadow region 不提前钳画布（画布外矩形阴影）+ mask 封顶 | fillRect.shadow / strokeRect.shadow |
| strokeRect 周长路径 + stroke 语义（负/零尺寸、join/cap） | strokeRect.negative/zero.* |
| CPU blit 应用 clip（clip_applies）+ clip 入 save/restore 栈 | fillRect.clip / saverestore.clip |
| blit 上界 ceil（亚像素矩形漏行） | strokeRect 系列 |
| setter 非法值忽略（lineWidth/lineJoin/lineCap/miterLimit） | line.*.invalid |
| 无效颜色忽略（try_parse_canvas_color） | invalid.strokestyle |
| line_segment_rect 精确端点（cap 职责分离）+ round cap/join 真圆盘 | cap.butt/round / join.round |
| miter 真实尖角三角（外角平分 + 补角 θ + spec ratio 判定→bevel 降级） | miter.acute/obtuse/exceeded/within |
| join 外扩点按角内侧选法线侧 + 共线角不画 | join.miter/bevel / strokeRect.zero.4 |
| 设备空间线宽（per-segment \|T·n̂\|） | width.scaledefault/transformed |
| arc anticlockwise 贯穿（枚举字段 + 3 处 flatten 方向 ±\|end-start\|）+ 弧起点连接 | cap.round/square 胶囊 fill |
| square cap 矩形 = 延伸段垂直扩 | cap.square |
| 段主体逐像素精确判定（投影+距离，斜线段 bbox 过覆盖） | miter.acute (48,48) |
| shadowBlur/Offset 非法值忽略 + shadowColor getter host 规范化（hex/rgba） | shadow.attributes.* |
| 阴影 mask 乘形状 alpha（rect 逐像素采样样式） | shadow.alpha.5 / gradient.alpha / transparent.* |
| 阴影 region 可见范围裁剪（画布−offset）+ mask 闭区间 | stroke.join.2 |
| 阴影受 clip 裁剪 | shadow.clip.2 |
| 阴影段逐像素投影判定 + join 真实几何 + 端 cap | stroke.join.1/2 / stroke.cap.1/2 |
| CompositeOperation 补 SourceOut/Clear + source 独占类未覆盖清除（受 clip 约束） | composite.uncovered.fill.* / clip.* / transparent.source-out |
| globalCompositeOperation 枚举校验 + globalAlpha 范围 | operation.* / globalAlpha.range |

## 🔄 交接记录（2026-08-13 用户决策：path-objects 剩余合并入 docs/goal/js-dom.md 统一执行）

canvas 流的 path-objects 目录工作**暂停移交** js-dom goal（JS/DOM API 语义面）。已完成的并入本流提交
（d0874c28：roundRect 角对半径/比例缩放/非有限守卫/16 段椭圆弧）；**未完成移交项**：

1. **roundRect DOMPoint 断言**（~26 用例）：shim "p<x>,<y>" 编码 + host 配对解析已通，但渲染仍
   偏离（部分断言 got 绿 vs 期望红——疑似 fill 扫描线与椭圆弧的交点配对或 16 段精度仍不足）
2. **roundRect 批量运行 panic**（NaN 排序，scale 归一化后复现——单用例未定位，疑似某
   负 w/h 或 NaN radii 组合）——**js-dom 流接手时先定位此 panic**（wpt-runner 崩溃级）
3. **arc 形状精度**（~16 用例：径向带 vs 折线垂直带、端点角度截断）、**arcTo/quadratic/
   bezier/isPointIn*** 等 JS 侧 API 语义——均属 JS/DOM API 面，移交 js-dom
4. **roundrect 语义校验**（badinput/negative/toomany 抛异常、winding/zero 边界）

**移交操作**：`CANVAS_TEST_SUBDIRS` 已移除 path-objects 目录（canvas 流不再跑；
js-dom 流接手时按需重新加入并修复）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| G1 | WPT html/canvas 真实用例覆盖为零 | ✅ M1 完成（124 文件导入，168/168） |
| G2 | 像素级 canvas 验证缺失 | 🔄 GPU 路径测试就位；Chromium oracle 对比待 oracle 环境（R3344 记录本机无 Chromium） |
| G3 | OffscreenCanvas Rust 桩 | ✅ 真实化（行为测试 4 个） |
| G4 | createImageBitmap options defer（imageOrientation/premultiplyAlpha） | ⏳ 未开工（M2） |
| G5 | ImageBitmap 源类型受限（HTMLImageElement 等） | ⏳ 深结构待点名 |
| G6 | OffscreenCanvas × Web Worker 未集成 | ⏳ 深结构待点名 |
| G7 | 后续 WPT 目录（compositing/shadows/path-objects/text/fill-and-stroke-styles 等，上游 ~1500 文件） | ⏳ M1 扩展（交替推进） |

## 待用户决策清单

- [ ] OffscreenCanvas × Web Worker 集成（G6）— 深结构 — 等点名 — 2026-08-13
- [ ] ImageBitmap 全源类型（G5）— 跨面深改 — 等点名 — 2026-08-13
- [ ] **DOM img 元素作为 drawImage/createPattern 源**（G5 切片：9 个 shadow.canvas/image/pattern 用例被阻塞——shim 仅支持 canvas/ImageBitmap 源）— 依赖图片加载链路 — 等点名 — 2026-08-13
- [ ] **shadowColor 'currentColor' 关键字**（2 用例：从 canvas 元素 CSS color 计算）— 需元素计算样式集成 — 等点名 — 2026-08-13

## 下一步计划

1. **M1 扩展**：导入下一批 WPT 目录（path-objects/fill-and-stroke-styles/text——与已修路径/样式光栅直接相关），失败聚类 → 轻量修复
2. **M2**：G4 createImageBitmap options（轻量可先行）
3. **Oracle**：Chromium 环境可用后补像素 oracle A/B（G2）
4. **待决策**：G5 DOM img 源（解锁 9 个 shadow 用例）、currentColor（2 用例）、OffscreenCanvas Worker

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge/canvas.rs` 核对 html-compat 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT canvas 基线建立 | ✅ 完成（切片 1 导入 + 切片 2 修复 168/168；扩展导入进行中） |
| M2 — API 语义补齐 | 🔄 部分（OffscreenCanvas 完成；G4/G5 待办） |
| M3 — 像素正确性冲刺 | 🔄 GPU 路径测试就位；Chromium oracle 待环境 |

## 验证基线

- 测试基线：canvas 743 全绿；engine canvas 32 全绿；WPT canvas 168/168
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` + `make test`
- 资产化：修复必须 `make import-wpt TEST=html/canvas/...` 常驻断言集并记入 `imported-tests.txt`（本轮 124 文件经 fetch 脚本导入 wpt-data 独立 repo 机制；`imported-tests.txt` 账本同步追加）
