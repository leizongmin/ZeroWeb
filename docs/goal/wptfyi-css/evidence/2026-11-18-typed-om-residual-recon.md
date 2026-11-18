# typed-om 残面侦查分类（2026-11-18，只读探针）

样本：results-knife-q2/css-typed-om.b1.json（Q2 三修后终态，链 C 电池
13:06-13:07 窗）。残面 **1285F / 166 页**，无单页主导（最大 logical.html
58F）。三族靶图：

## 族 1｜CSSNumericValue / CSSRGB OM math API（~30%，JS API 语义面）

典型：`arithmetic.tentative.any.js` 47F、`add-two-types.tentative` 26F、
`cssRotate` 23F、`cssColorValue` 20F、`cssRGB` 28F。

- `CSSUnitValue.add` 无参应返自身（现返 CSSMathSum）；numberish 系数
  （number → CSSUnitValue）强转缺（"can take numberish values" 挂）
- `new CSSRGB(...)` 构造类型校验：角度单位/CSSMathValue 形参/undefined
  通道应 throw TypeError（现不抛）
- 同族或含 sum/min/max/sub/mul/div 与 to()/toSum() 语义

属 typed-om 对象模型深水区（js 桥层类语义），刀量级 > 刀 Q3 round 槽。
驱动页集中（tentative any.js 群），可独立成刀不碰 gCS 串化。

## 族 2｜CSSStyleValue 子类归型（~40%，属性特化面）

典型：`grid-template-columns-rows` 54F、`grid-auto-columns-rows` 40F、
`grid-start-end` 36F、`gap` 28F、`logical` 58F、`scroll-padding` 48F、
`break` 35F、`marker` 20F、`font-synthesis` 24F。

- 设值读回应按值形归型：cssWideKeyword→CSSKeywordValue、单长度→
  CSSUnitValue（现恒返泛型 CSSStyleValue）
- `"Specified value must be a CSSStyleValue"`：setProperty 特化路径未接
- css-typed-om §cssstylevalue-subclass（attribute/serde 归型表）

面广但机制单一（归型查表一处），随属接入逐属性解锁，刀性价比较高分。

## 族 3｜computed calc 归约（嵌入族 2 页内）

`logical.html` 形：`calc(0% + 0%)` computed 应坍缩 CSSUnitValue
（现 CSSMathSum）——computed-time calc 简化（同值单位合并、零元消去）
经 typed-om 暴露。与 css-parser calc 归一化相关，动 parser 域需 fv1
避让协调。

## 选刀建议

族 2 归型查表为首选（机制集中、页数覆盖最大）；族 1 次之（API 语义
独立成刀）；族 3 挂起等 calc 归一化域清。与控制面池的 typed-om
favourable +2（delay 秒值串化，Fix C 已收）同域不同面。
