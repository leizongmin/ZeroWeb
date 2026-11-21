# 刀 OM-13：numeric-objects 构造器 percent-hint + normalize 三通道 + unparsed 序列化面 8 靶（2026-11-21 09:3x 收账）

## 池

OM-12 后 typed-om 残池 129F/66 页，本刀取三页 8F：
cssnumericvalue-multiply-two-types 1F（`new CSSMathProduct(Sum(px,%),
Sum(s,%))` 构造器不抛）、normalize-numeric 2F（calc 嵌套化简＋unitless
zero）、stylevalue-serialization/cssUnparsedValue.html 5F（lem/on/ade、
var-refs、mix、same-object、round-trip 注释面）。探针 zz-om13probe.json
12 事实定形（P1 构造器无门、P4/P5 两通道拒收 '0'、P6 数值通道 Sum
n=3 残积、P7 注释不分裂段、P9 平接无 /**/、P10 varref 逗号带空格、
P11 越界 set 不抛＝挂账、P8/P12 engine 转义域＝跨流挂账）。跑后探针
页已删，json 留 `.acceptance/wptfyi-css/`。

## face（patch v2 `96ab3aec`，单文件 part10-typed-om.js +142/−17）

六臂：

- **A. 乘积型构造器 percent-hint 门**（css-typed-om-1
  §multiplying-two-types：双方 hint 非 null 且异 → TypeError；文法面经
  `_zwOmCtorStrict` 旗标绕过，与加法型同门；单 hint/同 hint/无 hint
  放行——T2-T7 全不误伤，node 冒烟实证）。
- **B. calc 折叠补全**：`_zwOmFoldNumericSum` 双通道化——数值通道既有
  族宿主增量折不动（'calc(1px + 1in)' → Sum[px(97)] 绿面）；新增 number
  因子积成员吸收（calc(1px*2)→2px 并族，仅单 Unit×单 number 形）；
  **declared/get 通道族规范单位折**（css-values-4 §canonical-unit：
  deg/s/hz/dppx/px 基单位因子 1，仅 count>1 桶折、孤桶保位保单位）。
- **C. unitless zero**（css-values-4 §lengths 零可省单位；declared/get
  通道 `<length>`/`<length-percentage>` 面 width:'0' → CSS.px(0)；
  `<number>`/`<integer>` 面先行不受扰）。
- **D. unparsed 序列化**：相邻段 ident 交界插 `/**/`（css-typed-om-1
  §unparsedvalue-serialization 保 token 界——'lem'+'on'、'bar'+'var('
  插，'bar '+'var(' 与 ')'+v 不插）＋ varref 逗号后无空白（
  'var(name,fallback)' 形；fallback 自带空白保留）。
- **E. tokenizer 注释段界**（css-syntax-3 §comments：注释不产段内容且
  作段界，引号态感知、未闭合吞尾；'bar/*comment*/var(--foo)' →
  ['bar', varref]）。

## v1 电池 GATE FAIL 一面 P2F 定谳与回修（本刀主教训）

v1（patch `a635777e`，09:03 电池）8 靶全 F2P、双哨兵恒，唯
**offset-rotate.html P 33→32**（`Can set 'offset-rotate' to an angle:
calc(0rad + 0deg)`）：declared 通道折叠宿主取 firstIdx 单位（rad），
set→get 文本回环给出 Sum[rad(0)]，而语料 angle 例
`specifiedAlternateExpected = Sum[deg(0)]`（testsuite 注释自证＝css-values-4
规范主单位 deg；判读器 catch→alternate 通路合同）。回修＝declared
通道改族规范单位两遍折：桶和值即规范值（基单位因子 1），仅并族桶
动——percent/number/flex factorless（族表长度 1）入不了桶，
`'calc(0% + 0%)'` 平铺和式恒等面族（OM-12 −34 教训同域）零受扰；
时间例 Sum[s+ms]→Sum[s] ≡ alternate、px+em 孤桶恒、数值通道零漂移
（node 全合同面冒烟实证）。

## 电池 v2（knife-om13-battery.sh，log knife-om13-battery-v2-0919.log）

b0=`62c19474`（face 剥除＝OM-12 提交态 5d5476cfa 树）、b1=`484d9877`
（face 重放），M2 异 md5 ✓，FACE-MARKER OM-13 注释 0→10＋
_zwOmCanonFamUnit 0→2（两 token b0 侧 git grep 0 实证）。09:19:14
build 起、09:27:59 收账（8.7 分钟，全程 test-guard 包裹：build
per-proc 6/total 14/1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60）。
哨兵＝css-variables＋css-logical（tokenizer/toString 热路径）。

判读（knife-om13-judge.py）：TARGET-GAIN **8/8**（multiply-two-types
1/1、normalize-numeric 2/2、cssUnparsedValue.html 5/5——恰集翻转，三页
NAME-DIFF gone 合计恰 8 new=0）；STAY-FAIL 1/1（engine 转义域面按账
保 Fail）；typed-om P 12277→12285（**+8**）/F 129→121（−8）；双哨兵
SET-IDENTICAL（variables 56 页/logical 86 页）；**OM13-GATE PASS rc=0**。
v1 的 offset-rotate P2F 清偿（v2 无 PAGE-REGRESS）。

## 订正

OM-12 证据「电池脚本头注 normalize-numeric 2F 为过时 census（b0 即
绿，前刀已收）」系**我方 census path-filter 错误**：normalize-numeric
2F 实存于 stylevalue-normalization/ 目录（OM-12 探查按 /numeric-objects/
过滤漏计），b0 恒 Fail。本刀以 B/C 臂兑现该 2F，前笔表述作废。

## 残面

- **engine 自定义属性转义域（跨流挂账）**：'String serialization
  should roundtrip' 1F＋zzP8 var 名反斜杠截断（'--a\' 截为 '--a'）＋
  zzP12 gCS '--unparsed' = 'fail'——declaration/substitution 转义处理属
  engine/style-system 域（并行流 apply.rs 侧），shim 侧修不动终态
  （roundtrip 断言先 gCS 'pass' 后回写再 gCS），保 Fail。
- **OM-14 候选**：CSSUnparsedValue indexed setter 越界 RangeError
  （zzP11：len 2 set [3] 不抛——需 Proxy 结构改造，sentinel 热路径下
  风险高，单独立刀）；typed-om 残池 121F/64 页续分解。

## 验证面注记

JS shim 单文件（include_str 入 engine 二进制），fmt/clippy 不适用；
行为验证＝电池三 corpus（typed-om 359 页＋variables 56 页＋logical
86 页）＋node 合同面冒烟（三通道 normalize、set 回环恒等族、numeric
通道零漂移、T2-T7 不误伤）。

[控制面](../master.md)
