# M2 S2a 值类型底座——SVGLength 单位模型 + Animated 反射落地（2026-10-21）

本 session 持有 S2a 架构落地（S1 收尾同日接续）。实现中途出现并行 session
就地精修（04:52，q 单位第二表 / 8 位序列化 / 语境因子 / 校验序 / 缺参
TypeError），未另起结构、复用本方 prototype-accessor 骨架与 part04 门控；
本方按 master.md 恢复约定验证合并态后统一落地，提交同时署名两处工作。
落点两处：

## 实现

### part03.js S2a 块（SVGAnimatedString 块之后）

- **SVGLength 接口对象**：`SVG_LENGTHTYPE_*` 0..10 常量 + toStringTag；
  串解析 `[+-]?数[单位]?`（单位大小写不敏感——'1pX' 合法；deg/空串/垃圾 →
  null 即 SyntaxError 面）。
- **CSS 单位第二表**：q 等无 SVG unitType 常量的合法 CSS 单位——unitType
  落 UNKNOWN(0) 但 setter 不抛、value 按 CSS 物理换算（q = 1/4mm）、
  valueAsString 写回保真原后缀（'48q' 往返）。rem/vw/ch/ic 族依赖字体/
  视口度量，暂不录入（setter 抛 SyntaxError，对应案保持现状，FIXME 随
  上下文面后续切片）。
- **序列化 8 位有效数字剥尾零**（Blink SerializeNumber %.8g 语义）：
  `toPrecision(8)` + 尾零剥离——48px→mm 断言 `toFixed(1)` 形态 "12.7"，
  JS String 直排 12.699999999999998 失配。
- **单位域校验**：WebIDL unsigned short 归一（负数模 65536）后合法域
  1..10，0（UNKNOWN）/11+/65535 → NotSupportedError。
- **三态实例工厂**：detached（createSVGLength 面，闭包自持状态）/
  reflected live（解析内容属性 + 写透 setAttribute，invalid parse 回落
  初值）/ readOnly（animVal 面，变更抛 NoModificationAllowedError）。
- **语境换算因子**（spec SVG2 convertToSpecifiedUnits 步骤 4 字体基准 /
  步骤 5 视口基准）：percentage 沿最近 svg 祖先（≤32 hop）width/height
  属性取视口，方向性按属性名表——x/x1/x2/dx/width/refX/markerWidth 横向、
  y 族/refY/markerHeight 纵向、名单外对角线 sqrt(w²+h²)/√2（WPT
  percentage-directionality），detached 因子 1；em 沿链 style 内联
  `font-size:` 优先、次 presentation 属性、缺省 16px（ex 保持常量 8
  近似，FIXME）。
- **校验序**：convertToSpecifiedUnits/newValueSpecifiedUnits required
  参数缺失 TypeError（先于 unitType 域检查）；newValueSpecifiedUnits 的
  value float 校验先于 unitType——WPT ('aString','aString') → TypeError
  而非 NotSupportedError；value/valueInSpecifiedUnits 非有限 TypeError；
  valueAsString 非法 SyntaxError。
- **反射 wrapper**：SVGAnimatedLength（baseVal getter-only——sloppy 赋值
  静默）+ SVGAnimatedNumber（**baseVal 数值型 accessor**——getter 返原始
  number、float 校验 setter 写透，非 SVGNumber 对象面；WPT 实测语义）。
  SameObject 经 per-element 缓存（键 sel+handle+prop，className 缓存同款）。
- **反射定义表**：17 接口的 SVGAnimatedLength 面（svg/rect/circle/ellipse/
  line/foreignObject/image/marker/pattern/filter/mask/linearGradient/
  radialGradient/use + textLength/startOffset + filter primitive mixin
  x/y/width/height 0%/100%）+ 16 接口的 SVGAnimatedNumber 面（fe 族 +
  pathLength/stop.offset；kernelUnitLengthX/Y → kernelUnitLength 同名内容
  属性映射）。初值面取 WPT initial-values per-attr 配置（-10%/120%/100%/3/
  50% 等）。落点形态：**接口 prototype 上的 accessor**（get trap R93 尾部
  原型链回落以 proxy 为 this 派发——js-dom M3 既有机制）；accessor 内
  this.getAttribute/setAttribute 走既有同步面。HTML 元素零新路径。
- **S1 表扩容**：fe 族 20 个具体接口进 `_zwSvgIfaceParents`/
  `_zwSvgElementIfaces`/`__zwSvgTagIface` 三表——primitive 族挂
  SVGFilterPrimitiveElement 抽象层、灯光三元素（feDistantLight/
  fePointLight/feSpotLight）spec : SVGElement、feMergeNode 无 x/y 面改判
  SVGElement、feComponentTransfer 保持 primitive mixin。
- **SVGSVGElement.createSVGLength**：detached 工厂（单位模型族测试载体；
  createSVGMatrix/Rect/Point 留 S2b/S3）。

### part04.js R3037 放行（1 处）

width/height 对 SVG ns 元素跳过 reflected-string FLAT 串读
（`_REFLECTED_STRING_FLAT` 含 width/height——HTML 面表；SVG 的 width/
height 是 SVGAnimatedLength，须落 R93 接口 accessor；`_zwIsSvgNsEl` 门控）。
HTML 元素路径不变。这是本切片唯一触碰共享路径的改动。

## 验证

- 机制探针 7/7 Pass（gitignored wpt-data/svg/s2a-probe.html）：反射
  instanceof/写透/baseVal 只读赋值静默/SameObject（wrapper+baseVal+animVal）/
  初值面（filter -10%、marker 3、linearGradient 100%）/reflect-remove-invalid
  三步/detached 单位模型（parse/convert/双校验错）/数值面（校验 setter、共享
  内容属性联动）/animVal readonly。
- **全 corpus（486 案，探针文件剔除）：1563 子测试 430P/1083F/32T/18NR =
  27.5%**——v2 基线（134P/1339F/36T/18NR = 8.8%）以来 **+296P/-256F/-4T**，
  全绿案 20→33。
- **逐案对基线零真回归**：无既有 Pass 翻 Fail、无新失败案。4 条表面 F 增
  全部为披露面变化——font-change（1T→4F）、px-with-context（1T→10P/1F，
  净 +10P）、rem（+1F）、viewport（1F→8F），失败点全落在语境换算挂账面
  （getComputedStyle/offsetHeight 测试侧缺口 + rem/viewport 字号 FIXME）；
  另 18 条释放案（Timeout→跑满或 F 减，geometry/pservers/historical 等
  面外收益来自 early-crash 解除）。
- **types 子面（全量跑内）：70 案 549 子测试 279P/270F/0T**（S1 后账本
  0P/523F/4T）。两大 initial-values 案全绿（SVGAnimatedLength 130P、
  SVGAnimatedNumber 102P）；换算簇 readonly 5P、reserialization 7P、
  non-px-source 7P、percentage-directionality 7P、px-with-context 10P/1F、
  px 7P/3F、zoom 1P、SVGLength 1P 全部激活。
- 残余对位（全在既有挂账面）：getBBox 族（S4）、SVGMatrix/SVGAnimatedRect/
  createSVGRect（S2b 值对象）、SVGAnimatedInteger-initial-values 10F +
  Animated* 存在性（S2b 批）、SVGList（S2c）、getScreenCTM（S3）、
  ownerSVGElement 16F（S1/S2 边界项）、SVGLength-viewport 8F +
  font-change 4F + rem 2F（上下文换算 FIXME 面）、SameObject-identity 41
  断言（transform.baseVal.getItem 死，S2c TransformList 解锁）、
  setPathData（S4c）。
- 全 workspace `make test` DC-4 门禁：合并态源码上 EXIT=0（本轮提交前跑，
  结论见 master.md 轮次记录）。

## 排片对位

S2a 靶面 259F（S2 盘点）实收 430 子测试 Pass（含 Timeout/早死释放的跑满面
与面外收益）。下一步 S2b：SVGMatrix/Rect/Point/Angle 值对象 +
Animated{Integer,Enumeration,...} 接口批 + SVGAnimatedRect（viewBox 前置）。
