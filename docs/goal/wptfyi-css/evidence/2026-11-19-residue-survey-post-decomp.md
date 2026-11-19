# 残面普查（post-decomp 批后靶向，2026-11-19 11:1x）

**口径**：分解电池 pristine b1 JSON（results-knife-decomp-pristine/，五刀
落库前树态账），五语料残 Fail 总计 ~4540。在飞占用面已剔除（clip-path
插值 ~778F=css-parser 位语流、transitions properties-value 族 ~260F=事件
系统鲸面占用、keyframes-name-valid 39F + animation-shorthand 36F=刀#2
a3 流已声明、FI 三属性臂=在飞 WIP）。本文件只列**未认领**连贯面。

## 候选面（按连贯度排序）

1. **typed-om grid/logical 族 ~350F**（css-typed-om 单语料）：logical 58 +
   grid-template-columns-rows 54 + scroll-padding 48 + grid-auto-columns-rows
   40 + grid-start-end 36 + break 35 + arithmetic.tentative 47——**探针定性
   （pristine b1 失败断言样本，11:1x）**：全数「Can set '<prop>' to …」形
   ＝typed-om 写面白名单门缺 grid-template-*/grid-*/margin|padding|inset
   逻辑属性/scroll-padding-*/break-* 属性域，单一连贯根因；刀 P/R/R2/TP
   写面门扩面同配方可复制（白名单 + 逐族文法臂 + 序列化往返）。
   同车道刀 P P2 池（axes 简写 computed）延伸。
2. **animation-range 族 ~141F**（css-animations）：animation-range-shorthand
   111 + animation-range-start-computed 30——scroll-driven registry + range
   解析（post-collection 候选#3 在案）。
3. **font 单位循环 ~102F**（css-properties-values-api）：
   font-property-unit-cycles.tentative 79 + unit-cycles 23——注册属性
   font-relative 单位循环检测（@property syntax `<length>` em 端点回环），
   与 ATP 残面同语料。
4. **ATP 残面 ~59F**（css-properties-values-api）：registered-property-
   computation 32 + registered-property-initial 19 + registered-properties-
   inheritance 8——刀 ATP 六 fix 未覆盖的计算值面（var() 引用链/继承代算）。
5. 零散：custom-property-transition-mismatched-list 24、url-resolution 18、
   at-property-animation 18、KeyframeEffect-getKeyframes 32、
   text-decoration-inset-auto 49（css-animations）。

## 占用面（避让，不计入本批）

- css-masking clip-path 插值族 ~778F → css-parser 位语 calc 热修流。
- css-transitions properties-value/事件族 ~260F+ → transitions 事件系统
  鲸面（master 在册 ~857F+30F）。
- css-animations keyframes-name-valid 39F + animation-shorthand 36F →
  刀#2（NOTES 08:1x 节 a3 流接刀）。
- css-animations animation-range 族若刀#2 车道顺延可并入。

## 用法

下批发射前按本表认领认领面 → 探针定谳 → 分解电池复用（battery 脚本
face 清单替换）。批后 master.md 批账段的靶向注记由 docs 笔引用本文件。

## 批二刷新（17:1x，arith2 b1 JSON 口径——face B/faceC/ARITH 三刀落库后）

- **candidates #1 分解兑现**：animation-range 族已被 face A（+34）/face B
  （+97）收清；typed-om 数值面被 faceC part 1（+209）+ ARITH（+84）收清。
- **typed-om 残面改判**（stylepropertymap 页 b1 口径）：U 形 75F
  「Invalid unit for property」＝已知属性 `<number>/<integer>` set 门残缺
  （~25 页 ×3F）——**G2-a 修复在飞**（part10 工作树 8+/3−，17:0x 窗），
  本流避让。G 形 175F「Unsupported property」＝白名单门缺属性域
  （gap 28 / font-synthesis 24 / overflow 20 / marker 20 / vertical-align
  16 / overscroll-behavior 16 / grid-area 7 / offset/grid-template/grid-gap/
  font/font-variant/text-decoration/grid/font-family/flex-flow 3-6 各），
  未认领；扩面配方同刀 P/R/R2（白名单+逐族文法臂+序列化往返）。
- **新认领候选：registered-property computation 簇 ~84F**
  （css-properties-values-api，style-system/engine registered_properties 域，
  与 part10 零交叠）：registered-property-computation 32F（`<length>`
  computed 空值根因待查——vw/vmin/calc/lh 全形）+
  registered-property-initial 19F（calc 折叠 1in→96px、transform-list
  canonical、url 相对→绝对、color 初始替换）+
  registered-properties-inheritance 8F（invalid→inherit/initial 语义）+
  unit-cycles 23F + var-reference-unit-cycles 2F（font-relative
  em/ex/ch 引用 font-size 的 cycle 检测与 fallback）。
- **避免**：font-property-unit-cycles.tentative 79F＝calc trig 数学面
  （`tan(atan2(1em,1px))` 形），css-parser 位语 calc 热修流邻接。

## 批三刷新（23:3x，knife-g b1 五语料口径 + fonts fontcycle r1-pristine——刀 G 落库 `3b3297310` 后）

五语料树态残面：typed-om 636F / animations 604F / pva 417F / masking
1489F / transitions 501F，fonts 1949F（r1-pristine 口径，早于 ffix2/optical
在飞面）。批二候选全部兑现或转在飞：G 形 175F→刀 G +152（残 23F＝gap
`normal` keyword get 回形 5 + marker `<url>` opaque 18）；registered-property
computation 簇→atpr2 在飞（registered_properties/computed_style 等 5 文件
树内 hunks）；U 形已由 G2-a 收清。

### 未认领候选池（按连贯度 × 体量排序）

1. **typed-om stylevalue-subclasses 双池 ~240F**（单语料单文件域
   part10-typed-om.js，刀 G 落库后同文件可续作）：transform 子类
   ~101F（cssRotate 23 / cssSkew 16+SkewX 11+SkewY 11 / cssTranslate 14 /
   cssPerspective 15 / cssScale 9 / toMatrix 8 / 2d-flattening 4）+ color
   子类 ~139F（cssRGB 28 / cssColorValue 20 / cssHWB 18 / cssOKLCH 17 /
   cssLCH 17 / cssHSL 15 / cssOKLab 14 / cssLab 14 / normalize-tokens 10）
   ——构造器属性面 + `to()` 跨类型转换 + serialization 往返，配方承 face C
   part 1 / ARITH（已落库数值面不重叠）。
2. **mask-border composition/interpolation 簇 ~419F**（css-masking）：
   mask-border width/slice/outset composition 93×3 + width-interpolation 66
   + source-interpolation 42 + slice-interpolation 31——mask-border 长手
   插值臂 + composition 通道，配方承 B1 border-image 动画族（+939 已落库，
   border-image ≠ mask-border 族）。另有 clip-path 插值鲸面 ~780F 与
   shapes 刀（parse_basic_shape/serialize/css_wire 在飞，23:27 电池在跑）
   邻接，待其落库后再估插值引擎臂。
3. **custom-property-animation 家族 ~150F**（css-properties-values-api）：
   custom-property-transition-mismatched-list 24 + at-property-animation 18
   + custom-property-animation-{type,number,integer,length,percentage,
   angle,time,color,transform,resolution} × plain/space-list/comma-list
   ~24 页 ×5F——@property syntax 感知插值/离散通道。邻接警示：
   animation.rs 为 ffix2/optical 共享热文件（DISCRETE_TRANSITIONABLE 在飞），
   发射前需与两流核窗口。
4. **WAAPI interface 簇 ~150F**（animations+transitions 双语料）：
   AnimationEffect-getComputedTiming 41+20 / event-dispatch 32+11 /
   getAnimations 族 21+17+4+5+3+4 / CSSAnimation·CSSTransition·
   KeyframeEffect 生命周期 ~40——shim 动画对象语义面。邻接警示：part06.js
   为刀 KF 在飞 face（keyframes insertRule），KF 落库后开放。
5. **fonts 插值/composition 鲸面 ~900F**（r1-pristine 口径）：size-adjust
   composition 250 / variation-settings-interpolation 234 / palette-
   interpolation 114 / variation-settings-composition 60 / size-composition
   63 / size-adjust-interpolation 43——optical/ffix2 在飞同 corpus，等两刀
   收账后按残面重估（composition 臂：interpolation 页 43F/363P 而
   composition 页 250F/250P，插值已有 composition 缺，单根因候选）。
6. **零散连贯小池**：text-decoration-inset-auto 49F（animations）/
   scroll-padding 28 + cursor 18 + logical 16（typed-om TO-GRID 残）/
   column-rule-width 12 + column-rule-style 10（typed-om）/
   *-opacity 五属性 8F×5（typed-om 同构形）/ url-resolution 18（pva）/
   text-shadow-composition 15 + vertical-align-composition 10 +
   z-index 8（transitions composition 臂残，A2/A3/Z 已落库插值面的
   composition 通道残）。

### 占用面现状（避让）

- transitions properties-value 族 263F＝事件系统鲸面在册；animations
  parsing keyframes 族＝刀 KF 在飞；fonts synthesis/stretch/variant 臂＝
  ffix2/optical 在飞；pva registered-property-*＝atpr2 在飞；es_module.rs
  s419 模块语义＝script-sandbox 流在树 WIP（与本目标无关，census 对称）。
- 口径警示：五语料 b1 JSON 含 RIDE 在飞 hunks（ffix2 animation.rs、
  atpr2 registered_properties +455、KF part06 等），对应域残面计数在其
  落库后会收缩，认领前按当树 b0/b1 重探。

### 批三 recon 附注：池①失败形分类（knife-g b1 JSON message 抽样，23:4x）

- **cssColorValue 20F 全形 `CSSColorValue.parse is not a function`**——
  静态方法整体缺位，单根因；实现需保留分量类型（rgb→`<number>` 0-255、
  hsl→hue `<angle>` + s/l `<percentage>`、hwb、hex→CSSRGB、color()），
  现有宿主钩子 `__zw_parse_color_rgba`（part01 同源 zero_css_parser
  parse_color）只出 rgba 终值，typed 面需新钩子或 shim 侧按文法子集解析。
- **颜色子类构造器（cssRGB 28F 为代表，HWB/HSL/LCH/OKLab/OKLCH 同构）**：
  `assert_throws` did-not-throw 形——通道类型门缺（angle 入色通道须
  TypeError、undefined 通道 SyntaxError、alpha 数/百分互斥门、CSSMathValue
  `<number>` 匹配门）+ setter 同门 + 6 行属性值规一化 assert_equals。
- **transform 子类（cssRotate 23F 为代表，Skew/Scale/Translate/Perspective
  同构）**：构造器 angle/number 类型门 + 属性更新门（expected CSSUnitValue
  but got Number＝setter 未把 raw number 归一化为 CSSUnitValue）。
- **toMatrix 8F 双因**：skew/skewX/skewY 的 component.toMatrix 未实现 +
  DOMMatrix shim 缺 translate/scale/skewX/skewY 方法（rotate 臂 Invalid
  axis）。
- **normalize-tokens 10F**：normalize() 对 `var()` 返回 String，期望
  CSSVariableReferenceValue（无 fallback）/CSSUnparsedValue（有 fallback）
  ——归一化分道缺 var 识别臂。
