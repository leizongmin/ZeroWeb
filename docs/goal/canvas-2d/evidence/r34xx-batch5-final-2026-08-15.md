# R34xx 第五批证据：第二批目录收口 + 深项清单（2026-08-15）

## 范围

第二批导入目录（11 新子目录 + 顶层文件 + offscreen worker 变体）的 testharness
面收口——主线程 + worker 全目录回归确认，剩余失败全为深项（记录待决策）。

## 本批修复（WPT driving，主+worker 双覆盖）

1. **arc 角度归一化**（与 js-dom 流 R56 的 arc span 方向感知合并 + 补 mod==0
   整圆特例）：全圆 = clockwise raw≥2π / anticlockwise raw≤-2π / |raw| mod 2π==0
   （arc(0,2π,true) 整圆——join.round 圆盘）；|raw| mod 2π 幅度 + wrap。
   driving: 2d.line.cap/join.round（回归恢复 33/33）+ 2d.path.arc.angle.1-6/
   default/negative/selfintersect.1（全过）
2. **worker OffscreenCanvas 接口面**：
   - getContext contextId = WebIDL 枚举（缺参/'2D'/'' → TypeError）
   - OffscreenCanvasRenderingContext2D 独立接口（懒创建、prototype 属性规则、
     方法分发按 ctx 实际原型重注册、self 懒 getter）
   - width/height setter = [EnforceRange] unsigned long（'100em' → TypeError、
     '0x96' → 150）+ 同值也复位 + ctx.canvas 只读 + Symbol.toStringTag
   - transferToImageBitmap/convertToBlob 层打开期 InvalidStateError；
     convertToBlob 实现（异步 PNG Blob）
   driving: offscreen canvas-context/canvas-host/layers 全族 worker 变体
3. **CanvasFilter 深校验收口**（filters 目录 API 表面 11/13）：
   - colorMatrix values 按 type 分流（matrix 20 有限数——Float32Array 类数组按
     length；hueRotate/saturate 单数字；luminanceToAlpha 无 values）
   - beginLayer filter 选项数组/DOMString 化值接受（beginLayer-options 主+worker）
   - floodColor 经 host validateColor 真实解析；dropShadow/turbulence WebIDL
     double 转换 + 非负/枚举校验；显式 undefined 属性也抛（hasOwnProperty）
   driving: 2d.filter.canvasFilterObject.*/layers.* 全族
4. **drawImage img 未加载 no-op**（incomplete.*/nonexistent/broken/self/svg/null/
   wrongtype 全过——spec：incomplete image 不绘制不抛；非 canvas 非 img 源仍
   TypeError）
5. **path-objects 排序 panic**：`partial_cmp.unwrap_or(Equal)` 遇 NaN 非传递 →
   sort panic——5 处排序改 f32::total_cmp（NaN 全序）+ arc 负半径 IndexSizeError

## 最终数字（testharness 面）

### 主线程

| 目录 | Pass | Fail（深项） |
|------|------|--------------|
| 旧 9 目录（state/rect/transform/pixel/line/shadows/composite/fill/text） | 832 | 0 |
| conformance-requirements + 顶层 | 8 | 0 |
| reset | 54 | 0 |
| canvas-host | 66 | 0 |
| canvas-context | 11 | 0 |
| global-hdr-headroom | 10 | 0 |
| layers | 25 | 0 |
| filters | 11 | 2（colorMatrix 像素渲染） |
| path-objects | 166 | 37（arc 非均匀变换/扇区几何） |
| drawing-images | ~36 | 1（animated.gif img 状态机时序） |
| color-type | 2 | 2（display-p3 转换） |
| wide-gamut-canvas | 6 | 6（display-p3） |
| **合计** | **~1227** | **~48 深项** |

### Worker 变体

| 目录 | Pass | Fail |
|------|------|------|
| text/fill-and-stroke/compositing/shadows/line-styles/state/rect/transform/pixel/conformance | 740+ | 0 |
| canvas-context / canvas-host / reset / layers / filters | 14/46/53/29/10 | 0/0/0/1/2（深项） |
| path-objects | 166 | 38（深项） |
| color-type / wide-gamut | 2/1 | 2/3（深项） |
| **合计** | **~1060** | **~46 深项** |

## 深项清单（记录待决策，非轻量面）

1. arc 非均匀变换（scale.1/2 + shape.3 等 ~37）——变换后椭圆展平
2. colorMatrix 像素渲染（2）——CanvasFilter 实际滤镜光栅
3. display-p3↔srgb 转换（color-type 2 + wide-gamut 6 + 主 6）——色彩管理
4. animated.gif（1）——GIF 解码 + img 加载状态机时序
5. M3 oracle A/B（351 reftest cases，1.7% 真通过）——差距集中在滤镜/层合成/
   颜色插值（与 1-3 同源）

## 验证

- canvas 782 / engine 2149 全绿；clippy --workspace 零警告
- GPU 路径 4 测试 lavapipe 真实执行（复验）
- M3 oracle 捕获 1339 shots + A/B 基线（REFTEST_INCLUDE_CANVAS）
