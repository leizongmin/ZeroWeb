# 刀 T3D：part10 typed-om transform 序列化/parse 缺口 + list-valued opaque 折叠（2026-11-20）

## 池与根因链

CLR/FONT-1 收账后转独立语料对账（对端 ZeroWeb-2 在飞 CS1 corner-shape 刀，
18 脏文件与本刀 part10 单文件零交集）。三语料残池普查（CLR 电池 b1-sent
JSON：css-typed-om 382F/css-variables 248F/css-cascade 379F）筛出 css-typed-om
可修面，探针页（wpt-data/css/css-typed-om/zz-t3dprobe.html，gitignored）
双探针定谳归因：

1. **序列化族 6F**（cssTransformValue.tentative）：组件 toString 面缺口——
   translate/scale 3 参形缺 translate3d/scale3d 分支；rotate is2D 按轴零判定
   而非参量（4 参 rotate3d(0,0,0,…) 恒回落 rotate 形）；skewX/skewY 序列化
   小写形；CSSSkew 零 ay 不省略；CSSPerspective 负 <length> 无 calc 包裹。
2. **parse 族 4F**（transformvalue-normalization.tentative）：_zwOmParseTransform
   switch 缺 translatez/scalez case；perspective(none) 参数解析拒 keyword；
   calc() 参数拒（函数括号正则 `\([^)]*\)` 在 calc 内 `(` 提前截断——整列表
   解析 null 落 opaque，parse 门 TypeError）。
3. **opaque 折叠族 3F**（the-stylepropertymap testsuite 三页
   'Unsupported value can be set on different element'）：mapGetAll 对
   list-valued 属性（transition-property/will-change/text-shadow）顶层逗号
   逐段解析（part10 line 2671），段级不可分型（表长 1 非 <custom-ident>，
   'width' 段落 _zwOmOpaque 仅携段文本），get 回首段 → set 到另一元素恒丢
   其余段（'width, height' → 'width'）。css-typed-om-1 §normalize an
   unsupported value：unsupported 值归一为携**全文本**的单基类实例。

引擎域残面（探针通道 1/3 实证不入本刀）：engine setProperty('z-index',
'3.14') 静默丢弃（回显空串，typed-om get instanceof false）——归引擎
css_style_declaration/apply 面（对端 CS1 脏文件域，挂残面对账）。

## 码面（单文件 part10-typed-om.js +60/−20，patch md5 75616802604f4893cec9a77609762e0a）

- **A 序列化族**：CSSTranslate/CSSScale toString is2D 分支出 translate3d/
  scale3d 形；CSSRotate 构造器 is2D 改参量定谳（`arguments.length <= 2`，
  1-2 参 true/4 参 false——css-typed-om-1 §cssrotate is2D 与轴零无关）+
  toString is2D 分支恒 rotate3d 形（删 rotatex/rotatey 序列化形，spec 无此
  形）；CSSSkew ay 零角（CSSUnitValue value===0）省略；skewX/skewY 大写形；
  CSSPerspective 负 <length> calc 包裹（'perspective(calc(-1px))'）。
- **B parse 族**：_zwOmParseTransform 正则补 translate[XYZ]/scale[XYZ] 且
  函数括号一层嵌套（calc 参数形）；switch 补 translatez/scalez case；
  perspective(none) keyword 臂（CSSPerspective('none') → CSSKeywordValue）；
  calc() 参数经 _zwOmCalcParser math 组件化。
- **C opaque 折叠族**：mapGetAll list-valued 任一段不可分型（kind opaque）→
  整值回落单基类实例携全文本（_zwOmAssociatedProp 关联供 set 门；set 侧
  kind opaque 无文本门 line 2310 直受理）。
- 孤儿清理：_zwOmIsZeroNum/_zwOmIsOneNum 唯一消费方（rotate 轴零臂）随本刀
  移除，一并删除。

## 探针与预验（b1 二进制 21f30b73）

- 探针页 12 通道 10 绿 2 红（2 红即引擎域 z-index 残面）——translate3d/
  scale3d/rotate3d 零轴/4 参/updated-is2D/parse translateZ/perspective none/
  parse calc 参数/opaque set-on-other-element 精确流程全绿。
- 靶页预验：stylevalue-serialization/cssTransformValue 12P/1F（仅 mathvalues
  残）、transformvalue-normalization 28P/0F、properties/{transition-property,
  text-shadow,will-change} 全页 0F。

## A/B 电池（knife-t3d-battery.sh v2，log knife-t3d-battery-v2.log）

b0 = HEAD（face 剥除）md5 `383262bb912769566d42e68a210e536c`；b1 = HEAD +
face md5 `21f30b73f2b916b04c50326580cfee41`（M2 distinct ✓；FACE-MARKER
增量门 translatez b0=1→b1=2、skewX( b0=4→b1=5——v1 零基线门误设 b0=0
rc=7 早退，b0 二进制本含他面同名字面；v1 log 存
knife-t3d-battery-v1-m3fail.log）。build/leg 全经 test-guard（build
per-proc 6/total 14/1800s；leg per-proc 4/total 8/3000s + CASE_WALL_S=60）。
14:11:01→14:18:49 全程 7 分 48 秒。

**主电池（css-typed-om 全语料 360 页 + css-variables 哨兵 56 页）：
T3D-GATE PASS——gained=24 lost=0 regressed=0 pages_true_reg=0**
- 语料真增益 **+16 整**（页级 P 计数）：stylevalue-serialization/
  cssTransformValue.tentative **9F 全翻**（21P/1F 残 mathvalues）、
  stylevalue-normalization/transformvalue-normalization.tentative **4F 全翻**
  （28P/0F 满贯）、the-stylepropertymap properties/{text-shadow,
  transition-property,will-change} opaque 3F 全翻（三页全页 0F）。
- css-typed-om 页级账：P 12026→12042、F 393→377。
- css-variables SET-IDENTICAL（P 281→281，56 页零差异）。
- **改名对 324**（TD learning 同日机制实录）：'Setting X to a transform:
  translate(0px, 1px, 2px)…' → 'translate3d(…)' 组件示例串入子测试名，
  ~300 属性页 1-4 对/页；页级 P 计数恒等（RENAMED 桶仅 d==0 时计）＝
  纯改名零行为回归；判读器按页计数门免疫改名伪报。
- 判读器 v1 双计缺陷（页级循环+全局恰集各记 16→32 伪 FAIL）修正后
  复判 PASS；靶名恰集 16/16 F2P 无 MISS。
- zz-t3dprobe.html +8 为探针页自身翻转（gitignored 本地页被 corpus
  FILTER 扫入——learning #39 页键碰撞已知形态，不入账本刀）。

净 **+16 零回归**（哨兵零差异；全程无人值守 test-guard 包裹零 OOM/超时）。

## 残面

- **mathvalues 1F**（cssTransformValue.tentative 'CSSTransformValue containing
  CSSMathValues'）：需 CSSMathValue 序列化规范化（unit 规范化 turn→deg、
  Sum/Product 操作数排序 'calc(1em + 1px)'/'calc(2 * 90deg)'）——MATH 刀
  候选面（mathvalue 语料族同门）。
- z-index 非整数引擎静默丢弃 2F（引擎域，见上）。
- list-valued 段级 typed 值中间态：collapse 门仅对 opaque 段触发，typed 段
  （'1s, 2s'）行为不变（绿面零扰动，b1 腿页计数门实证）。

[控制面](../master.md)
