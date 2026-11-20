# 刀 OM-10：StylePropertyMap maplike 迭代协议 15 靶（2026-11-21 01:1x 收账）

## 池

OM-9 后 `the-stylepropertymap/{inline,declared,computed}/iterable.tentative.html`
各 5F＝15F/3 页（OM-9 b1 json 字节提取 15 靶名逐一核对
present+Fail）。失败消息自带 ground truth：`styleMap.keys/entries/values
is not a function or its return value is not iterable` ×14＋
`getComputedStyle is not a function or its return value is not
iterable` ×1——两处都是 GetIterator 协议死点：展开【方法返回值】需要
返回值自身带 @@iterator；map 本体 `[...map]`（TD1 已挂
@@iterator=entries）走首取豁免不在死点。

## 根因分解（语料源直读＋探针 v1 定谳）

三根因：① keys/values/entries 迭代器对象为裸 `{next}`，无
Symbol.iterator 自涉（inline/declared 10 靶＋computed 4 靶全死此）；
② computed face 对象无 length/item——`_zwOmMapNames` 空转，computed
键集空（但 get/getAll 通路健康：探针 G5 get('width')＝CSSUnitValue
10px）；③ part01 gCS Proxy get 陷阱符号键返 undefined——
`[...getComputedStyle(body)]` 不可展开（computed test 1 的 expectedKeys
运行时派生断言死）。

语料细节两定谳：**同源自洽约束**——computed test 1 expectedKeys＝
`[...getComputedStyle(document.body)].sort(comparePropertyNames)
.concat('--A','--B','--C')`，gCS 与 computedStyleMap 两侧枚举必须同一
名单（构造性自洽）；customs 由 concat 单独追加 ⇒ gCS 枚举不计入
custom properties（计入即双计失配）。**码元序判别**——test 5 自定义
属性对 U+1F4A9/U+F900：引擎串比较 ES 码元序 `lt=true`（探针 v2），
普通升序 sort 即满足 `['--💩','--豈']`；码点序则反向失配。探针 v1
字面 `豈` 落盘漂移成 U+8C48 未构成判别对——判别字符一律显式转义。

## face（patch `4b230fd5`，双文件 part01.js +16/−0＋part10-typed-om.js +67/−5，本刀）

三臂：

- **A. iterator-om10**：keys/values/entries 迭代器对象挂
  Symbol.iterator 自涉（WebIDL maplike 迭代器自身可迭代，
  css-typed-om-1 §stylepropertymapreadonly maplike iterable 面）；
  keys 名单改走 `_zwOmMapNames` 统一（inline/declared 同逻辑零漂移）。
- **B. computed 面枚举**：`_zwOmMapNames` computed 分流＝
  `__zwOmComputedEnumSource()`（cmpPN 序＝语料 comparePropertyNames
  同判；键集＝typed-om 属性语法表 335 键——引擎计算通道无全属性宇宙
  枚举 FIXME：宿主 cascade 属性枚举接入后换真值名单）＋元素 customs
  （TD1 `_zwOmComputedEnumNames` 滤 `--` 前缀码位升序）；face 携
  `__zwOmComputedEl` 回执；`size` 未接线（仍 0，语料无 computed size
  断言面 FIXME）。
- **C. part01 gCS `Symbol.iterator` 工厂臂**：迭代同一名单（customs
  不计防双计；裸 sandbox 无 part10 时空迭代器兜底零漂移）。

v1 face 直接返迭代器对象——b1val 校验构建探针 P6 定谳
`typeof cs[Symbol.iterator]`＝'object' spread 死（get 陷阱须返工厂
函数）→ 函数包裹返修，patch 重绑 `4b230fd5`（v1 `8f562772` 作废）。

## 探针（zz-om10probe v1/v2＋b1val 校验构建，跑后页删、json 留 `.acceptance/wptfyi-css/`）

v1（b1-runner＝OM-9 态二进制复用，预 face）：G1-G4 裸 `{next}`
selfIter undefined 展开 TypeError 实证；G5 computed face 键集空＋get
通路健康实证；G6 gCS 符号键 undefined/cs.length 0/spread 死实证。
v2：判别对显式转义实证码元序。b1val（face 重放构建 `0071faff`）：
P1-P8 全绿（G6 spread len=335 hasWidth=true hasCustom=false）——P6
首跑抓 face 工厂失误后返修再证。

## 电池（knife-om10-battery.sh v1 一遍过，log knife-om10-battery.log）

b0=`64958028`（face 剥除＝OM-9 提交态）、b1=`0071faff`（face 重放；
≡b1val 校验构建同 md5），M2 异值 ✓，FACE-MARKER 三门（enumSource
定义/computedEl 回执/zz-om10probe 注释×2）各 b0=0→b1=1/1/2。
01:02:44 B0 build 起、01:11:15 GATE-DONE（8.5 分钟，全程 test-guard
包裹：build per-proc 6/total 14/1800s＋leg per-proc 4/total
8/3000s＋CASE_WALL_S=60）。

判读 knife-om10-judge.py **OM10-GATE PASS（rc=0）**：

- TARGET-GAIN **15/15**（三页各 5 靶恰集 F2P）；靶页 gone=5 new=0
  ×3 零改名零溢出。
- css-typed-om 全语料 P 12230→12245（**+15 零回归**）、F 176→161。
- 双哨兵 SET-IDENTICAL：css-variables（56 页，P 281/F 237 恒）＋
  css-logical（86 页，P 218/F 1146 恒）。
- 旁页 computed.tentative.html Fail 名集恒 {contains every CSS
  property, contains computed values and not resolved values}——T1
  消息文本变体（gCS 不可迭代错 → `expected 335 but got 336` 长度
  失配：断言 properties.length===gCS 展开长，rule 层 `--foo` 不在
  inline 口径 customs 枚举差 +1，本刀枚举口径残面；T6 resolved-width
  残面不动）。

## 残面

`the-stylepropertymap/` 池余 **127F/59 页**（本电池 b1 实测，
142F/62 页直减 15 靶 3 页；三靶页 0F）：attribute-changed-callback
9F、clip-path 8F（CLIP-URL 残面留核）、get-auto-min-size 6F、
column-rule-width 6F、width/height 各 6F（OM-6 STAY-FAIL 挂账）、
column-count/opacity/scroll-snap-align 各 4F、offset-rotate/
orphans/text-decoration-line/widows/flex-basis 各 3F。

## 池账订正（OM-7/8/9 页脚，本刀 docs 同笔）

对账脚本 basename 键控把 inline/declared/computed 三子目录同名页折叠
少计（6 撞名 basename 16 实例 → −18F/−10 页恒差）：OM-7 页脚
137F/56 页→**155F/66 页**、OM-8 页脚 133F/55 页→**151F/65 页**、
OM-9 页脚 124F/52 页→**142F/62 页**（各电池 b1 json 逐页复测，证据
页脚与控制面条款同步订正；OM-7/8 "iterable.tentative 5F" 同因实为
15F/3 页）。OM-2 `properties/` 238F 复测恒（子目录口径正确）；
OM-5 133F/48 页系 pre-rebase 态带显式 caveat 不订正。

[控制面](../master.md)
