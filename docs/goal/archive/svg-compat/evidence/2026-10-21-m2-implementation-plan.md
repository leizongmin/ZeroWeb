# M2 实施计划——SVG DOM 类层级 + SVGSVGElement viewBox/CTM（2026-10-21 侦察稿）

M1 基线运行期间的本轮侦察产出（零源码改动；基线数字落定后按簇修正优先级）。

## 基础设施现状（crates/engine/src/js_dom_shim/）

| 资产 | 位置 | 现状 |
|---|---|---|
| SVG 接口 stub 构造器（39 个） | part03.js:2143 `_zwSvgElementIfaces`（R100） | 空壳：仅 prototype 链（SVGElement→Element；余→SVGElement），**零 IDL 成员** |
| HTML tag→接口映射 | part03.js:2449 `__zwHtmlTagIface`（R11） | 仅 HTML；无 SVG 对应表 |
| 元素原型路由 | part05.js:1145-1151（R80） | 非 HTML ns 一律回落 `Element.prototype`——`svg instanceof SVGSVGElement` 恒 false |
| SVGAnimatedString | part03.js:2176（className [SameObject]） | 已有（plain baseVal/animVal）；part04 get trap 已接 className |
| 值类型 | — | SVGAnimatedLength/Rect/Number/TransformList、SVGLength/Point/Matrix 等全缺 |

## corpus 需求排序（DOM 面 grep 实测，2026-10-21）

SVGLength 198、viewBox 161、getTotalLength 140、getBBox 133、getPointAtLength 56、
currentTranslate 19、SVGAnimatedLength 19、getScreenCTM 14、SVGSVGElement 13、
SVGAnimatedRect 7、createSVGPoint 4。几何面（geometry/path/shapes 目录 +
struct/scripted）与 types/scripted（Animated* 值类型面）是两大主簇。

## 切片序（M2，每片独立可验证：`make testharness-svg FILTER=<dir>`）

- **S1 类层级路由**：`__zwSvgTagIface`（localName→接口名，spec SVG2 element index
  全表）+ part05.js:1145 R80 分支加 SVG ns 路由（命中表→接口 prototype；miss→
  SVGElement.prototype——spec HTML 规范未知元素接口解析：SVG ns → SVGElement 非
  HTMLUnknownElement）。MathML ns 同法（MathMLElement，R80 已回落 Element）。
- **S2 值类型底座**：SVGLength/SVGPoint/SVGMatrix/SVGRect/SVGNumber/SVGAngle/
  SVGTransform + SVGAnimated{Length,LengthList,Number,NumberList,Rect,TransformList,
  Angle,Boolean,Enumeration,Integer,PreserveAspectRatio}（types/scripted 68 案的
  直接靶面）。属性反射（baseVal 读属性、moveto 解析 unit 类型）按 spec SVG2
  types.html 逐类型做。
- **S3 SVGSVGElement 面**：viewBox（SVGAnimatedRect 反射 attribute）、width/height/x/y
  （SVGAnimatedLength）、currentTranslate（SVGPoint）、createSVGPoint/Matrix/Rect/
  Number/Length/Transform、getScreenCTM/getCTM（viewport 变换矩阵计算——coordinate
  systems 面）、getCurrentTime/setCurrentTime、pause/unpauseAnimations。
- **S4 SVGGraphicsElement + 几何**：transform（SVGAnimatedTransformList）、getBBox
  （依赖布局数据——与渲染管线边界核对，几何可能需解析近似）、getCTM；
  SVGGeometryElement：pathLength/getTotalLength/getPointAtLength（path/rect/circle/
  ellipse/line/polygon/polyline mixin——**纯几何计算不依赖渲染**，140+56 案靶面）。

## 边界与约束

- engine crate = 本 goal envelope（run-rules §9 zero-web 流侧）；渲染面
  （style-system/render-foundation）不碰，getBBox 若需布局数据先做解析近似并记 M4。
- 事件目标化（M3）：dispatched 事件路径对 SVG 元素的命中——part04/05 事件派发面
  侦察待 M3 轮。
- SMIL 评估素材：animations/ 275 案时序面（M4 挂账定稿）；SVGAnimationElement
  接口存在性属 S1 表内（stub 已有）。
- idlharness.window.js（svg 根）无法入通道（runner 无 .window.html 生成管线）——
  M1 evidence 记账。
