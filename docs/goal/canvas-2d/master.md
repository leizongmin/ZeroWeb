# Canvas 2D 运行时控制面板

**最后更新**: 2026-08-15（R34xx 第十三批定稿：**第二批全目录收口**——主线程
reset 54/canvas-host 66/canvas-context 11/layers 25/filters 11/conformance 8/
global-hdr 10/path-objects 166/drawing-images 36 全绿；worker 变体 text 146/
fill-and-stroke 236/layers 29/canvas-context 14/canvas-host 46/filters 10 全绿；
剩余 = arc 非均匀变换 3 + colorMatrix 像素渲染 2 + display-p3 8 + img 状态机 1
——深项记录；M3 oracle 基线 1.7% 真通过（差距=滤镜/层合成/颜色插值深项）；
证据见 evidence/r34xx-batch4-m3-oracle + r34xx-batch5-final）。

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
| pixel-manipulation | 14 | ✅ **71/71**（float16 覆盖层 + ctor.basics 重载回退） |
| line-styles | 33 | ✅ 33/33 |
| shadows | 61 | ✅ 61/61 |
| compositing | 124 | ✅ 98 Pass / **0 Fail**（26 = reftest 格式 grid 文件超时，非 canvas 面） |
| fill-and-stroke-styles | 261 | ✅ 254 Pass / **0 Fail** / 7 超时（既有） |
| text | 144 | ✅ **193 Pass / 0 Fail**（variationSelectors 呈现感知 + edge-cases 中点边界）/ 19 超时（reftest 格式） |
| offscreen worker 变体 | 662 文件 | ✅ **715 Pass / 0 Fail**（G6——OffscreenCanvas × Web Worker 集成：importScripts 内联 + fetch_tests_from_worker 聚合 + worker 字体面） |
| **合计** | **1634 文件** | **testharness 面 0 Fail**（65 Timeout 全为 reftest-format 文件，非 canvas 面） |

### Rust 层（crates/canvas）

- ✅ 774 测试全绿；**行覆盖率 91.18%**（≥70% 目标达成）
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

（前轮记录见 git log；第十二批（2026-08-15，第二批导入 + layers + M3 oracle）——按 driving 用例聚类）

| 修复 | 驱动用例 |
|------|----------|
| 第二批导入（11 新目录 559+ 用例 + offscreen 变体 ~2000 文件）+ reftest-format 判定（NotRun） | fetch-canvas-subset.sh + runner 目录清单 |
| ctx.reset() 全量状态复位（27 项镜像）+ ctx.filter 属性 + DOMMatrix isIdentity/is2D/is3D | 2d.reset.state.* 全族 |
| canvas width/height WebIDL ToUint32 + standalone 尺寸 accessor + 固有尺寸双层注入 + MAX_CANVAS_DIM 钳制 + ctx.canvas 只读/同 identity + toStringTag + 缺参 TypeError | 2d.canvas.host.* 全族 |
| ctx 原型链重构（_methods 包 + prototype 分发）+ 构造器 prototype 属性规则 | 2d.canvas.context.type.*/prototype/readonly |
| beginLayer/endLayer 状态机 + 层渲染状态复位 + 打开期操作限制 + options 校验 | 2d.layer.*（invalid-calls/malformed/valid-calls/ctm/options/exceptions） |
| drawImage 负坐标（f32 as usize 饱和为 0 → 显式负值跳过）+ DOM canvas 源 ctx 查找 | 2d.drawImage.3arg / 2d.drawImage.canvas |
| M3 oracle：REFTEST_INCLUDE_CANVAS + 全量捕获 1339 shots + A/B 基线 | reftest-oracle html/canvas |

（更早轮次记录见 git log）

| 修复 | 驱动用例 |
|------|----------|
| float16 覆盖层补全（DOM canvas/OffscreenCanvas getContext `_f16` 标记 + `_zwBitmapF16` 原始浮点 + drawImage 记录/getImageData 回读 + 写像素失效） | createImageBitmap.srgb.rgba.float16（主+worker） |
| 外链样式表 url() 按样式表 URL 绝对化 + headless 本地提供 | variationSelectors（variation-sequences.css 字体引用） |
| 呈现感知字体选择（VS15 text 呈现 → 回落 sans-serif；VS16 保持 emoji 字体） | variationSelectors（⚓+FE0E vs ⚓+FE0F 宽度差） |
| ImageData 构造器 WebIDL 重载回退（data union 失败 → (sw,sh) 重载；settings 非对象 → TypeError） | ctor.basics（Uint8Array 2 参 INDEX_SIZE_ERR + (self,4,4) TypeError） |
| getIndexFromOffset/caretPositionFromPoint 字形中点边界语义（相邻原点中点，严格 <） | index-from-offset-edge-cases（主+worker） |
| VS 字体资产（wpt-data ×7 + fonts/ ×2 subset 提交） | variationSelectors @font-face 族 |

（更早轮次记录见 git log）

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
| G7 | 剩余失败聚类 | ✅ 全灭（testharness 面 0 Fail）——float16 覆盖层/variationSelectors 呈现感知/ctor.basics 重载回退/edge-cases 中点边界；65 Timeout 全为 reftest-format 文件（非 canvas 面） |
| G8 | 第二批新目录 | 🔄 reset 54/canvas-host 66/canvas-context 11/layers 28/conformance 8/global-hdr 10/filters 11（API 表面全绿）/path-objects 141/drawing-images 36（img 面全过）全绿；剩余 = arc 几何 62 + filters colorMatrix 渲染 2 + color-type 2 + wide-gamut 6 + animated.gif 1——深项记录 |

## 待用户决策清单

- [x] G5 DOM img 源（drawImage/createPattern）— ✅ 完成（headless 图片加载链路 + img 元素状态机 + shadow/composite/pattern 全解锁）
- [x] ImageBitmap 全源类型 — ✅ 完成（img/canvas/ImageBitmap/ImageData 源全通）
- [x] shadowColor 'currentColor' — ✅ 完成（设值时解析 + 元素 style 属性串）
- [x] OffscreenCanvas × Web Worker 集成（G6）— ✅ 完成（.worker.js 变体全通，715 Pass；真独立 worker 线程运行时 OffscreenCanvas 为浏览器架构面，非 WPT 通过率分母）
- [x] index-from-offset 边界约定 — ✅ 完成（字形中点规则，主+worker edge-cases 全过）

## 下一步计划

1. **M3 冲刺**：oracle A/B 基线 1.7% → 按 worst-diff 聚类修复（filters dropShadow/
   layers opaque-canvas/gradient 颜色插值）——每项修复经 oracle A/B 验证
2. filters 目录（17 Fail）——CanvasFilter 渲染（blur/colorMatrix/dropShadow 等）——
   深项，待决策
3. layers 像素面（beginLayer 离屏缓冲合成）——深项，待决策
4. color-type/wide-gamut-canvas（display-p3↔srgb 转换）——深项，待决策
5. drawing-images img 加载面（svg/incomplete/broken）——img 元素状态机，待决策
6. 浏览器 app form/input 快照测试（7）——本环境既有失败（a08d3064 复测确认），
   浏览器流（非 canvas 面）处理

**碰撞管理**：开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/ crates/engine/src/js_dom_bridge/canvas.rs` 核对 html-compat 流活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT canvas 基线建立 | ✅ 完成（919 文件导入，testharness 面 832/832 全绿） |
| M2 — API 语义补齐 | ✅ 完成（Path2D/OffscreenCanvas 主线程+worker/ImageBitmap/drawing.style/text 全系；G7 全灭） |
| M3 — 像素正确性冲刺 | 🔄 GPU 路径测试就位（lavapipe 4 测试）；Chromium oracle 待环境 |

## 验证基线

- 测试基线：canvas **774** 全绿（+1 presentation fallback）；render-foundation **640**（+1 VS cmap14）；engine **2129**（+1 float16 overlay）；wpt-runner 171；行覆盖率 **91.18%**（≥70% 达标）
- WPT canvas 主线程 **832 Pass / 0 Fail / 65 Timeout**（全 reftest-format）/ worker **715 Pass / 0 Fail**（evidence/r34xx-batch3 存档）
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` + `make test` 全过
- 资产化：修复经 fetch-canvas-subset.sh 资产化（wpt-data 独立 repo 机制，gitignored；CanvasTest.ttf/yellow*.png/vs/*.ttf 已入脚本）；VS subset ×2 提交 tests/wpt-runner/fonts/ 供单测
