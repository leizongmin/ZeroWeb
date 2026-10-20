# M2 S2b 值对象批——SVGMatrix/Rect/Point/Angle + Animated{Integer,Enumeration,Boolean,Rect,PAR} 落地（2026-10-21）

本 session 持有 S2b 编码面（S2a 收口同日接续，工作区 clean 基础上起步）。
落点两处：

## 实现

### part03.js S2b 块（S2a 块之后，+838 行）

- **接口对象批**：SVGAngle（SVG_ANGLETYPE_* 0..4）、SVGAnimatedAngle、
  SVGAnimatedInteger、SVGAnimatedEnumeration、SVGAnimatedBoolean、SVGRect、
  SVGAnimatedRect、SVGPreserveAspectRatio（SVG_PRESERVEASPECTRATIO_* 0..10 +
  SVG_MEETORSLICE_* 0..2）、SVGAnimatedPreserveAspectRatio、SVGMatrix、
  SVGPoint、SVGNumber、SVGUnitTypes（常量容器）——`new Function` stub +
  toStringTag，S2a 同款。
- **接口枚举常量挂载**：SVGMarkerElement（MARKERUNITS/ORIENT 族含
  AUTO_START_REVERSE=3、TEXTPATH_SIDETYPE 1..2）、SVGGradientElement
  （SPREADMETHOD）、SVGComponentTransferFunctionElement（FECOMPONENTTRANSFER
  TYPE 1..5）、SVGFEBlendElement（MODE 1..16 含 CSS blend mode 扩展）、
  SVGFEColorMatrixElement（**HUEROTATE/LUMINANCETOALPHA 无下划线**——SVG1.1
  IDL 拼写）、SVGFECompositeElement（OPERATOR 1..7 含 **LIGHTER=7**）、
  SVGFEConvolveMatrixElement（EDGEMODE）、SVGFEDisplacementMapElement
  （CHANNEL R/G/B/A=1..4）、SVGFEMorphologyElement（SVG_MORPHOLOGY_OPERATOR）、
  SVGFETurbulenceElement（STITCHTYPE/TURBULENCE）、SVGTextContentElement
  （LENGTHADJUST）、SVGTextPathElement（METHODTYPE/SPACINGTYPE/SIDETYPE）。
- **SVGAnimatedEnumeration**：baseVal 数值枚举面——getter 关键词精确查表
  （大小写敏感；invalid/缺失回落 initial，orientType 的角度态 '0'/'20deg'
  由 initial 兜底）；setter Number 归一后查反表写回关键词，域外（0/负/越界/
  非数）TypeError。17 接口反射定义表（clipPathUnits/filterUnits/
  primitiveUnits/gradientUnits/spreadMethod/markerUnits/orientType/maskUnits/
  maskContentUnits/patternUnits/patternContentUnits/lengthAdjust/method/
  spacing/side + fe 族 type/mode/operator/edgeMode/xChannelSelector/
  yChannelSelector/stitchTiles）；setMap 缺省=关键词表反查，orientType 显式
  （AUTO→'auto'、ANGLE→'0'、AUTO_START_REVERSE→'auto-start-reverse'）。
  SameObject 经 _zwSvgAnimCache（S2a 同款键）。
- **SVGAnimatedInteger**：baseVal 数值面 getter **整数校验**（非整数内容值
  回落 initial——WPT order '-1.5' → 3）；setter ToInt32 归一（NaN → 0 不抛
  ——WPT 'aString'/元素 → 0，与 AnimatedNumber 的 TypeError 面**不同**）。
  defs：feConvolveMatrix orderX/orderY（→'order' mapProperty）/targetX/
  targetY、feTurbulence numOctaves。
- **SVGAnimatedBoolean**：baseVal/animVal 原始 boolean 面（getter
  raw==='true'；setter ToBoolean 写 'true'/'false'，全值接受）。落点
  feConvolveMatrix.preserveAlpha（wrapper 层——首版裸 accessor 漏
  `.baseVal` 结构，探针后修正为 Animated wrapper）。
- **SVGAngle/SVGAnimatedAngle**：deg/rad/grad + 裸数（UNSPECIFIED）+ turn
  （UNKNOWN + CSS 因子 360 + 后缀保真——S2a q 单位同款，'2turn' 往返）；
  'auto'/'auto-start-reverse' 非角度串回落 UNSPECIFIED 0（orientAngle 与
  orientType 同一属性解析天然联动）；value（度数域）/valueInSpecifiedUnits/
  valueAsString + newValueSpecifiedUnits/convertToSpecifiedUnits（合法域
  1..4、required 参数 TypeError、value 校验先于 unit 域——S2a 校验序同款）。
  落点 marker.orientAngle + createSVGAngle 工厂。
- **SVGRect/SVGAnimatedRect**：x/y/width/height float 校验 setter（非有限
  TypeError、null→0）；viewBox 反射 'x y w h' 四数解析（invalid 回零矩形）
  写透 setAttribute（8 位有效数字 fmt）；SVGSVGElement/SVGSymbolElement/
  SVGMarkerElement/SVGPatternElement 四接口。
- **SVGPreserveAspectRatio/SVGAnimatedPreserveAspectRatio**：align
  （none/xMinYMin..xMaxYMax → 1..10）+ meetOrSlice（meet 1/slice 2）；
  invalid/缺失回落 xMidYMid meet；setter 写回 keyword（write 直接收当前态
  ——首版 write 内重新解析旧属性致 setter 丢写，探针 round 1 修正）；
  svg/symbol/image/marker/pattern 五接口。
- **SVGMatrix**：2D 仿射 {a..f}（列主形 [a c e; b d f]）float 校验 accessor；
  **multiply(second?)** 收 DOMMatrix2DInit 字典——a..f 与 m11..m42 别名拼写
  fixup（别名对不一致 TypeError、SameValueZero 0/-0 放行、非有限 TypeError、
  缺省成员按单位阵）；translate/scale/scaleNonUniform/rotate/
  rotateFromVector/skewX/skewY/flipX/flipY/inverse 全 post-multiply 返回新
  实例（**rotate 公式首版符号写反、skewX/skewY 公式互换——探针 round 2
  抓出修正**：post-multiply 旋转为 a·cos+c·sin / -a·sin+c·cos，skewX 改
  c/d 列、skewY 改 a/b 列）。S3 CTM 前置。
- **SVGPoint/SVGNumber**：float 校验面 + matrixTransform（S3 消费）；
  detached 工厂。
- **SVGSVGElement 工厂族收尾**：createSVGMatrix/createSVGRect/
  createSVGPoint/createSVGNumber/createSVGAngle（createSVGLength 见 S2a）。
- **SVGElement.ownerSVGElement**：沿 parent 链找最近 svg 祖先（≤32 hop，
  树结构计算脱离文档不变）；**foreignObject 边界清零**（WPT
  ownerSVGElement-01：fo 内外层 svg → null）；无 svg 祖先 → null。
  16 案全绿（in document/not in document/detached 三态）。
- **SVGMarkerElement.setOrientToAuto/setOrientToAngle**：写 orient 属性，
  orientType/orientAngle 经同一解析联动（spec SVG2 SVGMarkerElement）。

### part04.js R3037 放行扩展（+12 行）

type/method 对 SVG ns 元素放行 R3037 FLAT 串读（同 S2a width/height 模式
ns 门控）——SVG 的 type 是枚举反射面（feTurbulence/feColorMatrix/feFuncX、
textPath.method）；**例外 tag 门控**：svg style/script 的 type 是
reflected string（HTMLStyleElement 同款语义——首版无门控致
svg/styling attr-style-type 3P→3F 回归，styling 通道验证修复后全通道
round 2 确认零回归）。HTML 元素路径不变。

## 验证

- 机制探针 8/8 Pass（gitignored wpt-data 下 __s2bprobe.html，三轮迭代后
  跑完即删——语料卫生 #31）：enumeration 面（initial/setter/TypeError/
  ToUint16 串/writethrough）、orientType/orientAngle 联动（rad→deg 值、
  grad 400→360、turn 保真 '2turn'）、animated integer（order '-1.5' 回落
  3/ToInt32）、viewBox（live/SameObject/写透/remove 清零）、PAR、matrix
  multiply（别名/m41 别名/冲突 TypeError/0 与 -0）、matrix 方法链
  （translate/rotate/inverse/matrixTransform）、ownerSVGElement（嵌套/
  foreignObject 清零/detached null）。
- **types 子面：399P/208F/0T**（S2a 终账 279P/281F，**+120P/-73F**，
  Timeout 0 维持）。S2b 全部靶面案全绿：SVGAnimatedEnumeration-initial-
  values **48/48**、SVGAnimatedEnumeration-SVGMarkerElement 5/5、
  SVGAnimatedEnumeration-SVGTextPathElement 3/3（side 属性补齐后）、
  SVGAnimatedInteger-initial-values 10/10、SVGAnimatedAngle-initial-values
  2/2、SVGMatrix 9/9、SVGAnimatedRect 5/5、SVGRect 3/3、SVGPoint 1/1、
  SVGAnimatedBoolean 1/1、SVGAnimatedPreserveAspectRatio 1/1、
  SVGAnimatedEnumeration 17 接口案全绿、SVGElement.ownerSVGElement-01
  16/16。
- **全 corpus（486 案，探针剔除）：554P/1006F/32T = 34.8%**——S2a
  （430P/1083F/32T = 27.5%）以来 **+124P/-77F**，Timeout 32 持平，全绿案
  33→63（+30），逐案对账**零真回归**（type/method 放行的 styling 回归在
  首轮全通道对账暴露、tag 门控修复后 round 2 复跑确认）。
- 残余对位（全在既有挂账面）：getBBox 族 ~151F（S4）、SVGList/SameObject-
  identity（S2c）、getScreenCTM/getCTM（S3）、SVGLength 语境 FIXME 面
  （S2a 挂账）。
- `make test` DC-4 门禁：EXIT=0（结论见 master.md 轮次记录）。

## 排片对位

S2b 靶面（盘点 ~65-80F + ownerSVGElement 16F）实收 124 子测试 Pass。
下一步 S2c：SVGNumberList/LengthList（numberOfItems/parse-invalid-clears
语义）+ SVGPointList/SVGStringList 存在性——解锁 SameObject-identity 41
断言（transform.baseVal.getItem 死点）。
