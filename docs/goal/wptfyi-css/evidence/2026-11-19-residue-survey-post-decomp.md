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
