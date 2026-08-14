# Canvas 2D 运行时控制面板

**最后更新**: 2026-08-15（R34xx 第十批定稿：**getImageData/putImageData float16 options**——
put.basic.rgba.float16 主+worker 通过；主线程 **828 Pass** / worker **713 Pass**
（基线 718/630 → +110/+83）；GPU 路径 4 测试 lavapipe 真实执行确认；
证据见 evidence/r34xx-batch2）。

---

## 当前状态

**专项定位**：从 zero-web.md Tier 3「Canvas 2D 完整 API（Path2D、OffscreenCanvas、ImageBitmap）」拆出的独立目标，WPT `html/canvas` 真实用例驱动。

**与兄弟 goal 的边界**：
- rendering-compat（CSS 渲染/字体/布局）— 零工作重叠
- zero-web 父目标（JS/DOM 桥主线）— 仅 `js_dom_shim` part04/05.js canvas 段共享，run-rules §9 碰头管理（本轮碰头核对：part05.js 近 7 天无活跃编辑，安全修改）

## 实测基线（2026-08-14）

### WPT 面（9 目录 919 文件全量）

| 目录 | 用例数 | 状态 |
|------|--------|------|
| the-canvas-state | 23 文件 / 68 subtest | ✅ 68/68 全绿 |
| drawing-rectangles-to-the-canvas | 32 | ✅ 32/32 |
| transformations | 22 | ✅ 21 Pass（1 = reftest 格式文件超时，非 canvas 面） |
| pixel-manipulation | 14 | ✅ 14/14 |
| line-styles | 33 | ✅ 33/33 |
| shadows | 61 | ✅ 60 Pass（1 = current.removed——js-dom 流 DOM remove 不置空 parentNode，移交） |
| compositing | 124 | ✅ 98 Pass / **0 Fail**（26 = reftest 格式 grid 文件超时，非 canvas 面） |
| fill-and-stroke-styles | 261 | ✅ 251+ Pass / 3 Fail（halftransparent alpha 精度 + gradient.colormix/relativecolor 插值空间）/ 7 超时（既有） |
| text | 144 | ✅ 110+ Pass（draw 像素面 51；drawing.style 25；measure 全系：真度量/bbox 锚定/getActualBoundingBox 11/TextCluster 7/emHeight/font.parse 7）；剩余 = index-from-offset/selection-rects（DOM 布局面）+ lang + 超时 33（既有） |
| offscreen worker 变体 | 715 文件 | ✅ **630 Pass**（G6——OffscreenCanvas × Web Worker 集成：importScripts 内联 + fetch_tests_from_worker 聚合 + worker 字体面） |
| **合计** | **1634 文件** | **1314+ Pass**（基线 533 → +781） |

### Rust 层（crates/canvas）

- ✅ 759 测试全绿（基线 737 +22）；**行覆盖率 89.28%**（≥70% 目标达成）
- ✅ radial 渐变全几何：线性插值圆族二次方程（f64 精度 + 相对容差 + 半径穿零伪根过滤 +
  较大有效根）——cone.behind/beside/bottom/front/shape1/touch*/equal/transform.* 全族
- ✅ drawImage 阴影（源 alpha mask + blur）；pattern 平铺锚定 fill 空间（tile_transform）
- ✅ 文本真字体光栅：@font-face FontLoader 注入 → shape（rustybuzz，rtl 方向）→ glyph
  位图 blit；基线偏移真实 ascent/descent（fontdue descent 为负）；maxWidth 缩放；
  letterSpacing/wordSpacing（原始串 + em/% 随字号重解析）；measureText 真度量 +
  actualBoundingBox 按 align/direction 锚定 + 真实墨迹边界
- ✅ TextCluster 系列：getTextClusters（UAX#29 字素分段 + options 定位）、
  fillTextCluster/strokeTextCluster（options align/baseline/x/y）；stroke_text 真字体路径；
  TextMetrics emHeightAscent/Descent
- ✅ 空 stops 渐变透明；stroke 单次调用去重 mask（段/join/cap 重叠只合成一次）
- ✅ Transform2D::inverse；TextBaseline 补 Hanging/Ideographic

### JS 接线层（js_dom_shim + engine canvas.rs）

- ✅ fetch 同步返回契约（headless __zw_fetch 直返 wire）+ wpt-data fetch handler
- ✅ CSS Typed OM 最小面（CSSRGB/CSSHSL/CSS.percent/degs + color object duck-typing
  `a:`/`alpha:` 双键）；'currentColor' 设值时解析（内联 style + style 属性串；remove 后黑）
- ✅ 渐变构造 TypeError/负半径 IndexSizeError/addColorStop 非有限 TypeError（spec）
- ✅ drawing.style 面：letterSpacing/wordSpacing/fontKerning/fontStretch/fontVariantCaps/
  textRendering（值大小写敏感 + 归一化）
- ✅ grad_refs live 重放（addColorStop 后引用 context 更新）；ctx.reset()；SVG <image>
  元素 createPattern + fetch 提取（pipeline extract_img_srcs）

## R34xx 修复记录（WPT 驱动，全部带 driving 用例）

（前轮记录见 git log；本轮新增——按 driving 用例聚类）

| 修复 | 驱动用例 |
|------|----------|
| stop 含 CSS Color 4 现代函数（color-mix/相对色）→ 渐变 OKLab 插值；legacy 直 sRGB | gradient.colormix / gradient.relativecolor |
| alpha 序列化最短可回滚十进制（u8 量化 0.5→128 回读 '0.5'） | fillStyle.get.halftransparent / semitransparent |
| 缺参 TypeError vs 非有限忽略分流（_zwNumArg/_zwAllFinite） | 8 个 .nonfinite 系列 |
| setTransform 双重重载（0 参 → DOMMatrix2DInit identity） | setTransform.multiple / missingargs |
| ImageData 构造器 spec 化（WebIDL union/长度算法/pixelFormat float16） | imageData.object.ctor.* |
| fontKerning 'none' → shaping 关 kern；系统默认字体预载；resolve_font_id 大小写不敏感 | drawing.style.fontKerning / reset.fontKerning.none |
| fillText maxWidth ≤ 0 不绘制；ctx.font 保留 fontKerning；% / em / lh 字号（canvas 元素样式基准） | maxWidth.zero/negative / reset.fontKerning.none / percentage* / parent-style-relative-units |
| 显式 undefined vs 缺参（DOMString 转换语义） | gradient.object.invalidcolor / pattern.repeat.undefined |
| 文本路径 sample_at（零长渐变不绘制） | gradient.interpolate.zerosize.fillText/strokeText |
| bbox 符号约定去钳制（Left 正值=向左）；ASCII whitespace → U+0020；亚像素墨迹（轮廓 bbox） | actualBoundingBox.whitespace / getActualBoundingBox*.tentative / small-font / space.* 全族 |
| testharness 定时器记录式触发（t.step_timeout 回调最终执行） | draw.fontface.repeat |
| radial 二次方程全几何（f64 + 容差 + 有效根） | cone.behind/beside/bottom/front/shape1/touch*/equal/transform.* |
| 渐变半径随 CTM 缩放 | radial.transform.1/2/3 |
| drawImage 阴影 | shadow.image.* / shadow.canvas.* |
| pattern 平铺锚定 fill 空间 | pattern.paint.repeat.coord1/3 等 |
| 空 stops 透明 | gradient.empty |
| 文本真字体光栅 + 基线/对齐/maxWidth/rtl | text.draw.baseline.*/align.*/maxWidth.*/rtl |
| letterSpacing/wordSpacing 全语义 | drawing.style.letterSpacing.*/wordSpacing.*/spacing.* |
| drawing.style 属性面 | fontKerning/fontStretch/fontVariantCaps/textRendering.settings |
| measureText 真度量 + bbox 锚定 | measure.width.*/fontBoundingBox*/measure.direction/textAlign |
| fetch 同步契约 + 相对 URL | composite.image.*/canvas.*（62→98） |
| drawImage source 独占未覆盖清除 | uncovered.image.* |
| stroke 去重 mask | strokeStyle.colorObject.transparency |
| CSS 颜色 NaN/尾点/混合 % 拒绝 + 溢出钳制 | parse.invalid.*/rgb-clamp-5 |
| 0 尺寸 SVG 解码兜底 | pattern.image.zerowidth/zeroheight |
| SVG <image> 元素源 + href 提取 | svgimage.zerowidth/zeroheight/nonexistent |
| color object `a:` 键 | colorObject.transparency |
| 'currentColor' 设时解析 + style 属性串 | fillStyle.parse.current.*/shadowColor.current.* |

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| G1 | WPT html/canvas 真实用例覆盖为零 | ✅ M1 完成（919 文件导入，657+ Pass） |
| G2 | 像素级 canvas 验证 | 🔄 GPU 路径测试就位；Chromium oracle 待环境 |
| G3 | OffscreenCanvas Rust 桩 | ✅ 真实化 |
| G4 | createImageBitmap options | ✅ flipY + premultiplyAlpha 接受 |
| G5 | ImageBitmap 源类型 | ✅ DOM img/canvas/ImageBitmap/ImageData 源全通 |
| G6 | OffscreenCanvas × Web Worker | ✅ 集成（offscreen worker 变体 630 Pass） |
| G7 | 剩余失败聚类 | 🔄 createImageBitmap float16 1（画布 float16 原始像素存储——colorType float16 上下文）+ variationSelectors 1（emoji 字体回退，系统无 emoji 字体）+ ctor.basics 陈旧子断言 1（spec WebIDL 冲突）+ index-from-offset-edge-cases 2 断言（边界约定不一致）+ reftest-format 超时（非 testharness 面） |

## 待用户决策清单

- [x] G5 DOM img 源（drawImage/createPattern）— ✅ 本轮完成（headless 图片加载链路 + img 元素状态机 + shadow/composite/pattern 全解锁）
- [x] ImageBitmap 全源类型 — ✅ 本轮完成（img/canvas/ImageBitmap/ImageData 源全通）
- [x] shadowColor 'currentColor' — ✅ 本轮完成（设值时解析 + 元素 style 属性串）
- [ ] OffscreenCanvas × Web Worker 集成（G6）— 深结构（worker 运行时 + canvas 桥跨面；当前 WPT 面不含 .worker.js 变体，非通过率分母）— 2026-08-14

## 下一步计划

1. **text .tentative DOM 布局面**（~74 用例：TextMetrics.getIndexFromOffset/getSelectionRects 与
   document.caretPositionFromPoint / Range.getClientRects 对照——DOM 布局深结构，记待决策）
2. **emHeight/baseline 字体度量深面**（5 用例：OS/2 usWinAscent 与 em square 定位算法——需字体表
   解析扩展 + Chromium 算法确认）
3. **float16 ImageData 数据路径**（2 用例：getImageData/putImageData options + display-p3↔srgb 转换）
4. **M3**：Chromium 环境可用后补像素 oracle A/B（G2）
5. **current.removed**（1）：同步脚本内 remove 后 host 快照未应用 mutation——js-dom 流共享面

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge/canvas.rs` 核对 html-compat 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT canvas 基线建立 | ✅ 完成（919 文件导入 + 修复 657+ Pass） |
| M2 — API 语义补齐 | 🔄 大部分完成（Path2D/OffscreenCanvas 主线程/ImageBitmap/drawing.style 面；G6 Worker 待办） |
| M3 — 像素正确性冲刺 | 🔄 GPU 路径测试就位；Chromium oracle 待环境 |

## 验证基线

- 测试基线：canvas 772 全绿（基线 759，本轮 +13 单测）；WPT canvas 主线程 747 Pass / worker 698 Pass（evidence/r34xx-batch2 存档）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` + `make test`
- 资产化：修复经 fetch-canvas-subset.sh 资产化（wpt-data 独立 repo 机制，gitignored；CanvasTest.ttf/yellow*.png 已入脚本）
