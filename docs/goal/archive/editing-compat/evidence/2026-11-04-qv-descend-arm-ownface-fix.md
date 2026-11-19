# ⑦ 值命令 descend 臂自面读修复笔（2026-11-04，乙续三）

## 背景（qv-fix2 轮回归定谳）

qv-fix2 笔（`_zwVaSetStyleProp` minified joiner + descend 臂
`_zwVaSpecifiedValue !== null` 门）probe 三案绿，但五语料判读
**VERDICT REGRESSION**：fontsize 84→78（6 F→P 达成）同时 fontname
59→67——`foo<tt/code/kbd/samp>[bar]</…>baz` fontname ×8（双 swc 模式）
由 P 转 F，got 出现双层套
`foo<span style="font-family:sans-serif"><tt><span style="font-family:sans-serif">bar</span></tt></span>baz`
（期望 `foo<tt><span style="font-family:sans-serif">bar</span></tt>baz`，
data/fontname.js :218/:223 原行）。

归因：门只覆盖 author-specified（style/font 属性），漏 **UA stylesheet
元素规则面**。tt/code/kbd/samp 自带 `font-family:monospace` UA 规则
（HTML 渲染节 15.3.1.1），真 Chrome move 后 computed 自身面起算
（UA 声明占优）= monospace ≠ sans-serif → descend 臂合法走（包裹收进
tt 内层 = 期望形）；我方 specified-null 门误判「无自面」跳臂 → tt 滞留
外层 wrapper 成双层套。span1（无任何自面声明）与 tt（UA 规则自面）在
specified 读数下不可区分，门在判别面上不成立。

## 修复（本笔，1 处）

`crates/engine/src/js_dom_shim/part01d.js` `_zwVaForceValue` descend 臂：
读值源从 `_zwValEffectiveValue`（host 通道，sel 注册 M 形元素看不见
plain 新建 styled wrapper——qv-fix2 之前双层套误触发的根因）改读
`__zwPlainComputed(node, property)` **自面走查**：plain 通道同时看见
wrapper（style attr / font 表现提示）与 UA 表（`_zwPlainUaMono` mono 族、
a:link 链色）——

- plain span（无自面）：走查爬升命中 wrapper 新值 → 等价 → 跳臂
  （单层外裹 = Chrome 形）；
- tt/code/kbd/samp：首步命中 UA mono 规则 ≠ 新值 → 臂照走（包裹收进
  tt 内层 = Chrome 形）；
- 自面无源（''）回落 host effective 保 UA 初值口径。

比较器 `_zwVaEquivalentValues` → `_zwVaLooselyEquivalent`（自面
fontsize 返 px 形 `18px`，sizeMap 桥接关键字 `large`；`_zwPlainFontKwPx`
与 `_zwVaSizeMapData` 同为 host 整数标度 10/13/16/18/24/32/48px）。
子过滤臂（`_zwVaSpecifiedValue` kid 冲突跳过）为 vendored step 8.2
原文，不动。minified joiner 保持。

## 探针（zz-probe-vawrap，5 案全 PASS）

常驻探针页新增 D1/D2 判别案（data/fontname.js :218/:223 原行，[]
剥除）：D1 swc=true 期望 `foo<tt><span style="font-family:sans-serif">bar</span></tt>baz`、
D2 swc=false 期望 `foo<tt><font face="sans-serif">bar</font></tt>baz`；
原 A1/A2/C1（span 对双层套 + multicol 三案）保持绿。
运行：zw-qvfix3-probe2（05:58，probe-vawrap3.log）。

## 验证（qv-fix3 轮，05:58–06:01，zw-qvfix3-corpus，五语料全量）

**6 F→P / 0 P→F / 0 newF / VERDICT OK**（judge-qvfix3.py，对 qv-fix1
行级元组差；分母与 qv-fix1 逐语料恒等 2,422/2,225/2,327/1,026/1,218）：

| 语料 | 行数 | F | F→P | P→F |
|---|---|---|---|---|
| fontsize | 2,422 | 84→78 | 6 | 0 |
| fontname | 2,225 | 59→59 | 0 | 0 |
| forecolor | 2,327 | 39→39 | 0 | 0 |
| backcolor | 1,026 | 12→12 | 0 | 0 |
| hilitecolor | 1,218 | 25→25 | 0 | 0 |

- fontsize 6 案 = qv-fix2 所收原样保持：`<span>[foo</span> <span>bar]</span>`
  compare ×2 swc + 同形 indeterm ×2 + `<p>[foo</p><p> <span>bar</span> </p>
  <p>baz]</p>` compare ×2 swc。
- fontname 失败集与 qv-fix1 逐行恒等（0 newF）——qv-fix2 的 8 例 tt 族
  回归消除，59 例其余族（qV-after 读面等）不动。
- 色三命令零位移符合预期（本轮未触其残差族）。
- 残差总量：**219F → 213F**（qv-fix2 曾短暂 +2 至 221，本笔归位并净收 6）。

## 门禁

make test（zw-qvfix3-maketest，`make test` test-guard 包裹）——结果与
既有 5 红（`js_dom_bridge::tests` stylesheet/rule-style/adopted 面，
HEAD 287aed4a5 既有，并行席 s271 提交体同账）对账后落 commit body。
本笔改动面（shim 值命令 descend 臂 + 手工合并 joiner）不触上述五案面，
语料判决 6 F→P/0 P→F 独立成立。

## 残差与后续

compare 面 exec 包裹形状 ~120 行（骨架 `exp /font+/span+font+span vs got
font` 单族最大）仍是 ⑦ 主攻；qV-after fontname `exp="sans-serif" got=""`
×16 读面族、S5 选区/走查族 ~25 行不变。色三命令本轮零位移，其残差在
push-down/包裹面与 S5 下游。
