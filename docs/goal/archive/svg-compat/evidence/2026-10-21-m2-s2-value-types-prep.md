# M2 S2 值类型底座——types/ 70 案需求盘点（2026-10-21）

看护轮产出（S1 由基线原 session 持有实现中，本方不碰源码面；数据源 v2 基线
JSON——路径见 m4-smil-assessment-prep.md 同款临时产物约定）。S1（类层级路由）
落地后 S2 直接按本盘点排片。

## types/ 70 案 512F+4T 需求聚类

| 簇 | F 信号 | 案 | S2 落点 |
|---|---|---|---|
| **SVGAnimatedLength/Number 初值面** | 232 | 2（SVGAnimatedLength-initial-values 130F / SVGAnimatedNumber 102F） | **S2 最大单簇**：SVGLength 单位模型（value/valueInSpecifiedUnits/valueAsString/unit + convertToSpecifiedUnits/newValueSpecifiedUnits 换算与 reserialization）+ SVGAnimatedLength.baseVal/animVal 反射初始语义（spec SVG2 types.html§InterfaceSVGLength） |
| SVGGraphicsElement.getBBox 族 | 174 | 10（getBBox-01~10） | **S4 域**（布局数据依赖，解析近似后重评）——types/ 内大簇但不属 S2 |
| SVGLength 换算面 | 27 | 3（SVGLength-px 10 / convertToSpecifiedUnits-reserialization 7 / -readonly 5） | 同簇 1（unit 换算 + readonly 校验） |
| SVGSVGElement 工厂/CTM | 20 | 4（createSVGLength 11 / createSVGMatrix 9 / createSVGRect 3 / createSVGPoint 1） | S3（SVGSVGElement 面） |
| SVGAnimatedEnumeration 存在性 | 19 | 16 | S2：接口对象存在 + per-element 枚举反射（SVGMarkerElement 5 / SVGTextPathElement 3 等） |
| getScreenCTM/getCTM | 10 | 3 | S3（viewport 变换矩阵） |
| SVGMatrix 值对象 | 9 | 1（SVGMatrix.html） | S2：matrix 乘法/scale/translate/rotate 不可变语义 |
| ownerSVGElement | 16 | 1（SVGElement.ownerSVGElement-01） | S1/S2 边界：SVGElement 级 IDL（svg 元素反向指根）——S1 表落地后可顺带 |
| Animated* 接口对象存在性 | 8 | 8 | S2：SVGAnimated{Angle,Integer,Length,LengthList,Number,NumberList,PreserveAspectRatio,Rect} + SVGUnitTypes 构造器 |
| SVGList 面 | 2 | 1（SVGList-parse-invalid-clears-items 5F） | S2：list 类型 parse 无效值清空语义（numberOfItems） |

## S2 排片建议（S1 落地后）

1. **S2a SVGLength 单位模型 + AnimatedLength/Number 反射**——直接靶面 259F
   （簇 1+3+enumeration 一部分），单点收益最大；
2. **S2b 值对象族**：SVGMatrix/Rect/Point/Angle/Number + Animated* 接口对象
   补齐（存在性面 ~30F）；
3. **S2c List 类型**（SVGNumberList/LengthList，numberOfItems/parse-invalid）。

getBBox 174F 留 S4；createSVG*/getScreenCTM 30F 留 S3——S2 完成后 types/ 预估
残 ~210F（getBBox+CTM 簇），其余应转绿。

（注：types/ 各案失败消息点名成员分类账在轮内分析记录，本盘点为汇总口径；
复算入口 = v2 基线 JSON 同款解析。）
