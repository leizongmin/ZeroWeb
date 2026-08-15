# R34xx 第四批证据：第二批 WPT 导入 + layers 全绿 + M3 Oracle 基线（2026-08-15）

## 范围

第二批范围内子目录导入（11 新目录 559+ element 用例 + offscreen worker 变体）+
修复聚类（reset/canvas-host/canvas-context/layers 目录全绿）+ M3 Chromium oracle
A/B 基线建立（R3344「无 Chromium」记录已过时——本机 /usr/bin/chromium 可用）。

## 第二批导入

fetch-canvas-subset.sh + runner 目录清单同步补全（manual/video 面不在范围）：
element 11 新目录（drawing-images-to-the-canvas 37/path-objects 205/reset 45/
conformance-requirements 4/canvas-context 11/canvas-host 52/color-type 4/filters 42/
layers 144/global-hdr-headroom 3/wide-gamut-canvas 12）+ 顶层 testharness 文件
（2d.conformance.requirements.*/2d.putImageData/2d.text-outside-of-the-flat-tree
等）+ offscreen 变体（.worker.js 镜像，~2000 文件）。

**reftest-format 判定**：`rel="match"` / `-ref.html` / `-expected.html` 后缀 →
NotRun 中性状态（走 reftest/oracle 面）——原 65 Timeout 从 testharness 分母消除。

## 修复（WPT driving）

### reset 目录（30 Fail → 全绿）
- ctx.reset() 复位全部 27 项 client 状态镜像（抽 `_zwResetCtxMirrors` 共用：
  filter/letterSpacing/wordSpacing/fontKerning/fontStretch/fontVariantCaps/
  textRendering/imageSmoothing* 等）
- ctx.filter 属性补全（值接受 + 复位语义；渲染未实现记录）
- DOMMatrix isIdentity/is2D/is3D（此前缺失 → getTransform().isIdentity undefined）
- driving: 2d.reset.state.* 全族

### canvas-host 目录（24 Fail → 66/66 全绿）
- canvas width/height IDL setter 改 WebIDL ToUint32（200-2^32 → 200、'400x' → 0）
- standalone canvas 尺寸 accessor（设值——即使同值——重置 bitmap + 全状态）
- canvas 固有尺寸双层注入（computed_style + layout：rules for parsing
  non-negative integers 前导数字/+ 前缀——'100.999'→100、'0x100'→0）
- host bitmap 尺寸钳制 MAX_CANVAS_DIM=16384（2^31-1 用例防 429GB 分配 abort）
- ctx.canvas 只读 + 与 getElementById 同 identity（_proxyCache 键统一）
- Symbol.toStringTag 按 tag 返接口名（[object HTMLCanvasElement]）
- getContext() 缺参 → TypeError（WebIDL 必参）
- driving: 2d.canvas.host.* 全族

### canvas-context 目录（5 Fail → 11/11 全绿）
- ctx 原型链重构：实例方法移 `_methods` 包 + prototype 薄分发层（_zwDispatch）——
  getPrototypeOf(ctx)===CanvasRenderingContext2D.prototype、proto 扩展/覆写生效
  （type.extend/replace）、prototype 属性不可写/不可删（type.prototype）
- driving: 2d.canvas.context.type.*/invalid.args/prototype/readonly

### layers 目录（37 Fail → 25/25 全绿，testharness 面）
- beginLayer/endLayer 状态机：层内 save 允许但 endLayer 栈深校验；restore/reset
  抛 InvalidStateError；层渲染状态复位（alpha/gco/shadow/filter，transform 保留）
- options WebIDL 校验（非对象 TypeError；colorMatrix values 串 → TypeError 且层
  不打开——exceptions-are-no-op）
- 打开期操作限制：getImageData/putImageData/createPattern/drawImage(canvas 源)/
  toDataURL/toBlob/createImageBitmap(canvas) → InvalidStateError
- 顺带修复：DOM canvas 作 drawImage 源 ctx 查找（_zwCanvasCtx 键兜底）
- **诚实范围**：层内绘制不经离屏缓冲合成（filters/blur/composite 层效果像素面
  待 host 层合成——记录）
- driving: 2d.layer.invalid-calls.*/malformed-operations*/valid-calls.*/ctm.*/
  beginLayer-options/exceptions-are-no-op/layer-rendering-state-reset-in-layer

### drawImage 负坐标（drawing-images 目录）
- `f32 as usize` 对负数**饱和为 0**（Rust 浮点→整型转换）——dx=-100 的像素
  dst=-1 → 0，整图错误画到 (0,0)。显式负值跳过（2d.drawImage.3arg 等）
- driving: 2d.drawImage.3arg（red(-100,0) 污染 (0,0) 断言）

## M3 — Chromium Oracle A/B 基线

**环境更新**：/usr/bin/chromium headless 可用（R3344 记录过时）——
capture-oracle-per-dir.mjs 全量捕获 html/canvas/element **1339 shots**（0 fail）。

**A/B 管线**：wpt_file_loader 加 `REFTEST_INCLUDE_CANVAS=1`（canvas 专项测量，
忽略 rendering-compat 的 canvas skip，不影响兄弟 goal 分母）；reftest-oracle
对比 our renderer framebuffer vs chromium oracle-shot。

**基线（351 canvas reftest cases）**：
- 真通过（严格容差）：**2（1.7%）**——html/canvas/element 顶层 2/2
- 近似通过：0；不一致（≥1%）：117
- 差距集中在深项：filters（dropShadow 26.6%——滤镜渲染未实现）、layers
  （opaque-canvas 14%——层合成未实现）、gradient 颜色插值（hueInterpolation
  8.7-10.6%）——记录为待决策深项，非轻量修复面

**意义**：M3 的 oracle 机制（捕获 + A/B + 严格容差分类）端到端建立——环境受限
条款解除；像素级冲刺的测量与修复闭环就位。

## 验证

- WPT：reset 54 / canvas-host 66 / canvas-context 11 / conformance-requirements 4 /
  layers 25（testharness 面全绿）
- Rust：canvas 774 / engine 2131 全绿；clippy --workspace 零警告
- 全量回归（775+559 用例）运行结果见 r34xx-batch4 终稿（evidence 追加）

## 待决策深项（记录，非轻量面）

1. filters 目录（17 Fail）——CanvasFilter 渲染（blur/colorMatrix/dropShadow 等）
2. layers 像素面（beginLayer 离屏缓冲合成）
3. color-type/wide-gamut-canvas（display-p3↔srgb 转换）
4. drawing-images img 加载面（svg/incomplete/broken——img 元素状态机 + 缺失资源）
