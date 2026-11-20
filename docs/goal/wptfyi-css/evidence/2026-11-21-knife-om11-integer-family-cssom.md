# 刀 OM-11：整数族/scroll-snap-align/text-decoration-line computed 面 15 靶（2026-11-21 04:5x 收账）

## 池

OM-10 后 `the-stylepropertymap/properties/` 六页 19F：orphans/widows 各 3F
（`Can set to a number: 0/-3.14/3.14`）、column-count 4F（加
`calc(2 + 3)`）、order 2F（`-3.14/3.14`）、scroll-snap-align 4F（四关键词
臂）、text-decoration-line 3F（blink/spelling-error/grammar-error）。

## 根因分解（探针 zz-om11probe 28 事实 + b1 失败消息定谳）

三根因：①整数族 association 合同缺位——css-typed-om-1 §normalize
specified 面非整数/低于下限包 Sum([v])、computed 面 css-values-4
§<integer> 取整（ties +∞）＋orphans/widows/column-count 下限 1
（css-break-3/css-multicol-1）；探针 G2/G3/G4 实证引擎计算串分数形有损
（order -3.14 回 0、column-count 分数/calc 回 'auto'）须声明文本 reify；
②text-decoration-line 计算值＝指定关键词原文（css-text-decor-3），引擎
计算串对未实现词回 'none'（G7 实证）；③scroll-snap-align 四臂＝**上游
模板 authoring bug 不可修族**（见订正节）。

## face（patch `9209f6da`，单文件 part10-typed-om.js +75/−2）

三臂：

- **A. 整数族**：`_zwOmIntProps`＝orphans/widows/column-count/order＋
  `_zwOmIntMin1Props` 前三；declared 面单值 Unit 非整数/越下限包 Sum
  （语料 specified 臂 expected CSSMathSum 实证，与既有非负臂同型）；
  computed 面 `_zwOmComputedCanonicalize` Unit/折叠后 Math.round 取整＋
  钳 1（Math.round 同判 ties +∞）；computed gCS 声明文本源集（calc 前缀
  或纯数字 token 单值形，var()/css-wide 系回落引擎）。
- **B. scroll-snap-align**：`_zwOmComputedUnsupportedProps` 入列（computed
  基类合同本义；OM-5 column-rule-* 同型）。
- **C. text-decoration-line**：computed 词表内单关键词声明文本原样回显
  （OM-4 cursor/OM-6 touch-action 同型；多词并置/var()/css-wide 回落
  引擎零漂移）。

## 探针（跑后页删、json 留 `.acceptance/wptfyi-css/`）

zz-om11probe G1-G7 28 事实定形（整数族两臂期望形＋引擎计算串有损锚）；
probe2 P1-P12 b1 全绿合同验（含 P8 computed 基类/P9 词表回显/P10-P12
哨兵）；probe3-5 关键词/css-wide/var 序列 computed 基类实证。
**probe8/probe9 订正对**（04:39 补证，b1 同二进制）：probe8 手写
`attributeStyleMap.set(CSSKeywordValue)` → `computedStyleMap().get` 五
流向全返基类 CSSStyleValue——引擎满足 assert_is_unsupported 合同本义；
probe9 挂真 testsuite.js 的 runPropertyTests 复现四臂 Fail——失败消息
`got "[object CSSKeywordValue]"` 的断言对象是 input 非 computedResult。

## 电池（knife-om11-battery.sh v2 一遍过，log knife-om11-battery.log）

b0=`568d497f`（face 剥除＝OM-10 提交态）、b1=`23788e37`（face 重放；≡
target/release 现树复跑同 md5），M2 异值 ✓，FACE-MARKER（intMin1 0→4/
OM-11 注释 0→7）。v1 曾 M3-FAIL 一次（probeComment token 被 OM-1 已
提交注释子串污染，b0 基线 1 非零——换 OM-11 token 重标定）。04:00:34
build 起、04:08:59 对账毕（8.5 分钟，全程 test-guard 包裹：build
per-proc 6/total 14/1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60）。

判读 v1 19 靶恰集 15/19、双哨兵 SET-IDENTICAL、typed-om P 12246→12261
（+15）/F 160→145；四 scroll-snap-align 臂 TARGET-NOT-F2P → judge v2
订正后 **OM11-GATE-V2 PASS（rc=0，log knife-om11-judge-v2.log）**：
15 靶恰集 F2P＋四臂 STAY-FAIL 4/4＋页级 P 计数门＋双哨兵恒。

## 订正：scroll-snap-align 四臂不可修族定谳（上游 master 字节比对 2026-11-21）

语料 `scroll-snap-align.html` 四臂 `computed: assert_is_unsupported`
**裸函数传递**——properties/ 全目录唯一裸用法（其余页均
`(_, result) =>` wrapper 弃 input）。testsuite.js testPropertyValid 计算
臂调用序 `(computed)(example.input, computedResult)`（上游 master L365
同字节），assert_is_unsupported(result) 单参——断言对象＝模板自建
`CSSKeywordValue(keyword)`，toString "[object CSSKeywordValue]" ≠
"[object CSSStyleValue]"，引擎无关恒 Fail（Chromium 同面必败；上游
fetch 2026-11-21 字节同一）。face B 臂保留：computed 基类合同本义零
回归，上游若改 wrapper 形即自动翻绿。

## 残面

`the-stylepropertymap/` 池余 **111F/54 页**（本电池 b1 json 实测；
OM-10 订正后 127F/59 页 −13 靶 −4 页＋offset-rotate 3→2F 复测差）：
attribute-changed-callback 9F、clip-path 8F（CLIP-URL 残面留核）、
get-auto-min-size 6F、column-rule-width 6F（unsupported 面需 provenance
区分，map.set round-trip 现为 Pass，文本背式翻转即 P2F）、width/height
各 6F（OM-6 STAY-FAIL 挂账）、opacity 4F（M3-A 欠账域）、
scroll-snap-align 4F（不可修族）、flex-basis 3F、余散布 1–2F。

## 验证面注记

改动为 JS shim 单文件（include_str 入 engine 二进制），fmt/clippy 不适
用；行为验证＝电池三 corpus（typed-om 363 页＋variables 56 页＋logical
86 页）＋探针链。既有单测 `test_get_computed_style_text_decoration_
longhands`（R2754）三例与 face C 臂同值恒（inline 单词形同引擎计算串；
双词形词表外回落引擎保规范序重组；无声明回落 'none'）——静态核对无
冲突，未重跑 debug lib 批（复用规则：无新影响证据不补昂贵测试）。

[控制面](../master.md)
