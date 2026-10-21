# M2 S2c List/Transform 批——SVG{Length,Number,Point,String}List + SVGTransform(List) 落地（2026-10-21）

本 session 端到端持有 S2c（S2b 收口后接续，工作区 clean 基础上起步；无会话
重叠）。S2c-a（值列表族）与 S2c-b（Transform 面）两块同批落地 part03.js
（+777 行），一个提交。

## 实现

### _zwMakeSVGLength 扩展（S2a 工厂，listCtx/seedState 两参）

- listCtx → cur/write 路由列表引擎的按位读写（条目 i 读属性串第 i 个
  token、写重建串）；seedState → detached 初始态。S2a 既有路径零改动
 （两分支前置短路），SVGLength 完整接口（unitType/value/valueInSpecifiedUnits/
  valueAsString/convert/newValue + 校验序）直接复用于 LengthList 条目。

### part03.js S2c-a 块：值列表引擎（spec SVG2 types.html List 接口族）

- **ctor stub 批**：SVGLengthList/NumberList/PointList/StringList +
  Animated{Length,Number}List（toStringTag）。
- **解析语义**（WPT SVGList-parse-invalid-clears-items 对位）：length/number
  列表任一 token 非法 → 整表清空（0 items）；point 列表坐标对贪心配对、
  奇数尾截断（'0,0 100,0 20' → 2 items）但非法 token 仍整表清空；string
  列表逗号分隔 + ASCII whitespace trim、**空 token 保留**（HTML
  set-of-comma-separated-tokens：'' → ['']、',' → ['','']、双逗号产空 token
 ——WPT systemLanguage-parsing 13/13）。
- **条目混合态**（WPT appendItemFromClearedList + AnimatedLengthList
  'dynamic' 对位）：getItem(i) 返回绑定 (el, attr, i) 的条目 wrapper——getter
  重读内容属性、越界回落 getItem 时快照（来源列表 clear 后旧条目值仍可读）；
  setter 写透重建属性串；每次调用返回新 wrapper（列表条目无 SameObject 约束）。
- **变更面**（WPT SVGLengthList-basics/getItem/appendItem 对位）：
  initialize/getItem/insertItemBefore/replaceItem/removeItem/appendItem/clear +
  legacy length getter + 索引 getter（Proxy get trap 实时解析）；索引 setter
  TypeError（Proxy set trap）。条目类型不符 TypeError（先于 index 归一）；
  index WebIDL unsigned long（'aString'/对象/null → 0、负 → mod 2³²）；
  getItem/removeItem 越界 IndexSizeError、replaceItem index ≥ numberOfItems
  IndexSizeError、insertItemBefore/appendItem clamp 尾插；removeItem 返回
  **脱离条目**（seedState 快照——live 绑定会读到后移位条目）；animVal 只读
  抛 NoModificationAllowedError。
- **序列化写透**：length/number 空格分隔（8 位有效数字 fmt 复用）、point
  'x,y' 对空格分隔、string 逗号连接。
- **反射挂点**：SVGTextPositioningElement x/y/dx/dy（LengthList）+ rotate
  （NumberList）——spec 仅 text/tspan 有定位列表（textPath 是
  SVGTextContentElement 无 x/y 面，探针锁定 `textPath.x === undefined`）；
  SVGPolygon/PolylineElement points/animatedPoints；SVGElement
  requiredExtensions/systemLanguage（SVGTests 面——readonly SVGStringList
  无 Animated wrapper，spec SVG2 struct.html §InterfaceSVGTests）。SameObject
  经 _zwSvgAnimCache。

### part03.js S2c-b 块：SVGTransform + SVGTransformList（spec SVG2 coords.html）

- **SVGTransform**：type（SVG_TRANSFORM_* 0..6）+ matrix SameObject（单
  SVGMatrix 实例就地更新——set* 族经 S2b matrix 校验 setter 重写 a..f）；
  setTranslate/setScale（y 缺省 = x）/setRotate（中心点公式 e =
  cx(1-cos)+cy·sin）/setSkewX/setSkewY；setMatrix 未实现（FIXME——corpus
  未消费）。
- **transform 属性解析**：锚定扫描（fn 名严格 + 全串须为合法 fn 序列，残渣
  非分隔符 → 整表清空——parse-invalid 'translate(10,20) INVALID' → 0 items）；
  单参 translate(100) → e=100/f=0（WPT transform-translate-single-parameter）。
- **TransformList live 对象存储**（与值列表的本质差异，WPT list-02 对位）：
  条目是 SVGTransform 对象非值快照——属性串只是派生序列化（raw 变更才重
  解析，保跨读对象身份）；**所有权语义**：无主条目（createSVGTransform/
  removeItem 后）按引用收编（live——set* 后列表可见，list-02 'appendItem()
  with SVGTransform that already exist'），已属任意列表（含本列表）→ 插入
  **副本**（值快照、原位保留——list-01 replaceItem/insertItemBefore 断言
  3/4 项列表非移动）。
- **consolidate**：空表 null、只读抛 NoModificationAllowedError、按序
  post-multiply（S2b multiply 同式）归并单 MATRIX 条目（'translate(100 50)
  scale(2 3) translate(25 75)' → {2,0,0,3,150,275}）。
- **反射挂点**：SVGGraphicsElement.transform + SVGPatternElement
  patternTransform + SVGGradientElement gradientTransform（Animated wrapper
  同款 SameObject 缓存）+ SVGSVGElement.createSVGTransform 工厂。

## 验证

- 机制探针 7/7 Pass（gitignored wpt-data/svg/s2c-probe.html，一轮修正——
  首轮 1 断言为探针自身记账错：clamp 追加后列表 5 项按 4 项推演，实现
  输出正确）：parse-clears 四类/变更面全表/index 归一/条目快照回落/live
  写透/PointList 截断/StringList 空 token/identity/`textPath.x === undefined`。
- **types 子面：491P/191F/0T**（S2b 终账 399P/208F/0T，**+92P/-17F**，
  Timeout 0 维持；绿案 46/70）。
- **全 corpus（486 案，探针剔除）：681P/965F/32T/18NR = 40.2%**（含 NR
  分母口径；不含 NR 40.6%）——S2b（554P/1006F/32T/18NR = 34.4%）以来
  **+127P/-41F**，Timeout 32 持平，全绿案 63→76（+13，恰为提升案数），
  逐案对账**零真回归**（无 DOWN 案）。分母 1610→1696 为 SameObject-identity
  早崩释放披露面（死案 1 子测试 → 跑满 82，非新增语料）。
- **S2c 靶面 13 案全绿**：SVGTransformList.list-01 **14/14**、list-02 4/4、
  consolidate-01 3/3、transform-translate-single-parameter 1/1、
  **SameObject-identity 81/81**（S2a 挂账的 41 断言死点 + 全案 81 断言解锁
 ——transform.baseVal.getItem 死点随 transform 反射落地解除）、
  systemLanguage-parsing 13/13、SVGLengthList 四案（basics 153 行全表/
  getItem/appendItem/appendItemFromClearedList）、SVGAnimatedLengthList/
  NumberList、SVGList-parse-invalid-clears-items 5/5。
- 残余对位（全在既有挂账面）：svgtransformlist-replaceitem 1F（getBBox——
  S4）、systemLanguage-matching（条件处理匹配评估——M4 前评估）、
  getScreenCTM/getCTM 族（S3）。
- 全 workspace `make test` DC-4 门禁：EXIT=0（结论见 master.md 轮次记录）。

## 排片对位

S2c 靶面（S2 盘点 List ~46F + SameObject-identity 41 断言死点 + Transform
评估面）实收 127 子测试 Pass。下一步 M2 S3：SVGSVGElement 面——
pauseAnimations 族动画控制 stub（释放动画面 275 案脚手架）+ currentTranslate
（SameObject-identity 的 DOMPointReadOnly 断言当前经 undefined 恒等假绿，
S3 落地后须复核为真对象）+ CTM 族（盘点见
evidence/2026-10-21-m2-s3-svgsvgelement-prep.md）；S3 落地后 M4 SMIL 分类
账须重跑。
