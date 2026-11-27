# s10k56 · 刀②：write-face TT 集成收敛 SVGScript-internal-slot（双根因）

日期：2026-11-27。刀号：②（s10k55 轮五探针考古设计的挂账刀）。提交 `87b57bb69`
（rebase 到位 `37a92b9c8` css 流 docs-only 无交叠；rebase 后 blob 恒等核证 part03
`6f0d7d16b`／part06 `352e39916`＝pre-commit guard 扫描绑定值）。

靶面：TT 池最后一键真实 F——`SVGScriptElement-internal-slot.html`（页形：document.write
流式写未闭合 `<svg><script>…first…` → manipulator 找 script[1] appendChild `/*byapi*/`
→ write2 闭合＋第二 script；期望 first 被 TT 阻断、second 执行）。上轮定性：tt12 锚
2280P/14F/20T，真实池 1F＝本页。

## 一、双根因定谳（五探针考古＋probe1 诊断）

考古（s10k55 五探针链）已定谳第一重根因：write-face RunFace（part06 9540-9860）直
eval 绕 TT 执行门＋未闭合 script 提前物化执行（SVG 形 first 在 manipulator 变异**前**
执行＝F 根因；HTML 形尾垃圾 SyntaxError 静默吞＝test1 假绿定性）。

刀② pending 记账实施后（part06 五处编辑，bin `b112aa16936f`→`69a6768532b4`）探针仍见
manipulator 断链，probe1（zz-s10k56-probe1.html，write1 后 dump DOM 状态）第三版定谳
第二重根因：

```
DIAG first=false FULL: H@w1 wm=0 done={} pend={"0":true} openIdx=0 bufLen=70
 | bodyKids=4 lastTag=SVG/http://www.w3.org/2000/svg lastKids=1 q=hit(SCRIPT)
 gEBTNsvg=0 gEBTNscript=1 | s1 ns=undefined textLen=-1
```

SVG script 元素在 DOM（lastTag=SVG、querySelector 命中）但
`getElementsByTagName('script')`/`('svg')` 均不命中——**「solo 悬挂族」真根因**：
判据页 manipulator 找不到 script[1] → `t.step_timeout(manipulator, 50)` 无限重试 →
页永不完成。

归因链：part05 `_zwFilterByTagNameNS` 非 HTML ns 分支走**原值精确比较**（R330，
18519 `qnCmp = String(qn)`）；元素 tagName 由 part03 `_zwMEl` 26394 无条件大写烘焙
（`_mUp = snap.preserveCase ? tag : tag.toUpperCase()`）→ SVG 元素 tagName='SCRIPT'
≠ 'script' 永不命中。spec（dom-node-tagname）：ASCII 大写化仅限「in the HTML
namespace and its node document is an HTML document」——foreign ns（SVG/MathML）
解析产物 qualifiedName 保持原值（HTML 解析 tree-construction 小写化＋adjust SVG
tag names camelCase 例外表）。host 侧干净（Rust 解析器 tokenizer
`crates/dom/src/parser.rs:790` 全小写；part04 handle 代理 R5000 分支读原值）——烘焙
只在 shim 本地解析视图 `_zwMEl`（write-face 回放物化面）。

## 二、修复刀面（part06.js＋part03.js，110 行）

### ① write-face 未闭合 script 延迟执行＋pending 槽位 TT 管道（part06）

- `__zwDocWriteOpenScriptIndex` helper（BuildCopy 前）：缓冲级闭合平衡扫描，返最后
  一个未闭合 `<script>` 开标签 0 基序号（全闭合 -1）。口径与 s353
  `FindBlockingClose` 同源（注释区跳过、标签名尾字检查、引号属性跳过、`</script`
  闭合定位；引号内 `<script` 字面等病态形状不作 token 级识别，FIXME 同 s353）。
- RunFace inline 分支：`_dwsOpen === _dwsWm` → `_dwsHeld = true; break`（水位线不推
  满：`if (!_dwsHeld) … = _dwsList.length`；排水续流条件加 `!_dwsHeld &&`）。
- `__zwDocWritePendingScriptSlots` global 槽位记账（write() 记账块落**首写 init 重置
  后**、s353 停车决策注释前；init 重置行同列 SlotCount/NewFrom。首版误置 init 前被
  抹账——探针 `pend={}` 定谳移位）。重放重建元素丢 appendChild 变异，记账必须走
  global 槽位号而非元素章。
- 曾 pending 槽位强制转换管道：`_dwsIsPend && __zwTtEnforced()` 时
  `__zwTtSinkValue('TrustedScript', text, sink)`（sink 按 namespaceURI 选
  'SVGScriptElement text'/'HTMLScriptElement text'）；TypeError → null →
  `_dwsDone[wm]=true; wm++; continue`（阻断）；空体 '' 静默臂；转换成功执行转换后
  文本（eval 点 `_dwsText` → `_dwsExecText`）。**不经盖章快照等值短路**——重放重建
  元素的章不可信（TT spec prepare-script-text 的 mid-parse 变异保护面，
  w3c/trusted-types #507）。非 pending 槽位免检直 eval 零变化（second 臂保绿）；
  非 enforcement 页 `_zwTtEnforced()` 首查即 false，零路径变化。

### ② `_zwMEl` foreign ns tagName 保持原值（part03）

```js
var _mUp = (snap.preserveCase
  || (snap.ns !== undefined && snap.ns !== 'http://www.w3.org/1999/xhtml'))
  ? tag : tag.toUpperCase();
```

深臂两个 builder（28440/28497）均已透传 `ns: entry.ns`；浅臂不发 ns 落 HTML ns 兜底
（仅 fallback 路径）。写面消费核对：RunFace 槽位收集与 10249 行均 `.toUpperCase()`
归一化比较，大小写改动安全；querySelector/序列化走 localName（探针实证序列化本就
小写）；gEBTN 非 HTML ns 分支原值精确比较在修后命中。

## 三、验证链

探针达标（bin `e0cd206f5f55`，probe1 第四版）：

```
lastTag=svg/http://www.w3.org/2000/svg … q=hit(SCRIPT) gEBTNsvg=1 gEBTNscript=2
 | s1 ns=http://www.w3.org/2000/svg textLen=57
```

真靶页 solo 直跑：`Pass trusted-types/SVGScriptElement-internal-slot.html :: Test TT
application when manipulating <script> elements during loading.` rc=0。

三臂电池（bin `e0cd206f5f55`，chain13 test-guard 包装 solo 窗，12:13-12:51Z，与兄弟
ZeroWeb-2 经 wait_idle 互斥）：

| 臂 | 结果 | 对锚 |
|---|---|---|
| tt13（testharness-trusted-types，12.4min） | **2282P/14F/19T** | tt12 锚 2280P/14F/20T（`/tmp/s10k55/ver-tt12.log`） |
| dom13（testharness-dom，16.7min） | **57,463P/361F/12T** | dom12 锚 57,449P/361F/12T |
| ref13（make reftest） | **725/725 全绿**（Layout 521/521＋Text 204/204） | 历史形逐字一致 |

判读（LC_ALL=C awk ＋ comm 对账，提取法与锚侧同源）：

- tt13 F anchor-only 恰一键：`SVGScriptElement-internal-slot.html :: Test TT
  application…` 离场翻 P＝**TT 真实池 1F→0F**；F new-in-13 仅本轮 DIAG 探针
  zz-s10k56-probe1（不入库噪声）。
- tt13 T 20→19：`zz-s10k55-slot-probe.html` T→P（悬挂族探针页修后完跑、探针子测
  Pass）＝gEBTN 根因修复直证；T new 空。P +2＝靶页 +1＋slot-probe 探针子测 +1。
- HTMLScriptElement-internal-slot 恒形：test1 P＋页级 T（face B tests2-8 未触碰）。
- dom13 F/T 四象限 comm **全空**＝361F＋12T 逐键恒等零回归（_zwMEl 改动影响全部解析
  foreign ns 视图节点的主回归臂）；页集 348=348；P +14 恰 Range-mutations 双页
  截断噪声族（dataChange 852→876、replaceData 896→886，±截断抖动已知带）。

## 四、残相与挂账

- 探针 `q=hit(SCRIPT)` 残相：body.querySelector('svg script') 命中元素 tagName 仍读
  大写——selector 包装面另一 representation（host apply 树回链），靶页与 gEBTN 消费
  面不触及，挂残相注记不入本刀。
- face B（HTMLScriptElement-internal-slot tests 2-8 页级 Timeout）维持挂账：门空体臂
  静默/置章区分＋sink setter 对已 prepare-空 script 触发再执行。
- zz-s10k55-*（8 件）＋zz-s10k56-probe1 探针页留语料目录（gitignored 不入库），终判
  清场删。
- census：+1 → **84,324**，距 ≥99,947＝15,623。
