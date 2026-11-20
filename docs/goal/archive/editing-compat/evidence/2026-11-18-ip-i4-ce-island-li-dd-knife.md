# ip I4 族 CE-island host 本体 li/dd 拆分刀（R22 收口档）

日期：2026-11-18 ｜ 刀体：`b86912191`（发射时 `d35da58db`，push rebase 并入
上游窗 R5799-R5802 后重写）｜ part01.js `_zwCeParagraphMutate` ① 宿主级臂门
域局部扩域（⑩ 臂）＋sepTag 未设回退双面制＋part143 钉七形 ｜ 门禁批：ip1
四腿全绿 22:08 @bin `0e7cc7669596cbee5184a26950b823e8`，推后组合 spot check
ip 7185/10 failset 恒一 @`b86912191`

## 根因与刀面

语料 insertparagraph.js :1996/:2008（dps=div 显式）：
`<ol contenteditable=false><li contenteditable>ab[]cd</li></ol>` 及同形 dd。
Chrome 录音＝内容拆 div 块内嵌 li（`<li contenteditable=""><div>ab</div>
<div>cd</div></li>` 或尾 br 变体）；本引擎 got＝`ab<br>cd`（③ 行断退化）。
ip 腿 12F 恒挂此 2 键实红（余 10 键 recording 伪影挂账族）。

机制链：part06 host2 上溯最近 CE≠false 元素＝li/dd 自身（host 本体）；
ipara 预检 `_zwJusIsEditable(li)`＝false（本仓「editable＝严格位于编辑宿主
内」约定，host 本体返 false）→ `__zw_ce_ipara_wants()` false → legacy
`__zw_ce_paragraph_split_at(host2=li)` → `_zwCeParagraphMutate` 内 block=
host=li：① 臂 div 门不开（hostTag='li'）、② 臂要求 block≠host 不接 →
③ `_zwCeLineBreakAt` 插 br。Chrome 同标签兄弟拆分不成立——父 CE=false
不可编辑，`</li><li>` 越出编辑宿主；h1/dt 非段落内容容器 Chrome 退化行断
（corpus :1992/:2002 `ab<br>cd` 绿形）；li/dd 流内容容器可含 div → div
双块内嵌。

刀面（域局部）：

- ⑩ 臂＝① 宿主级臂门扩域：`hostTag ∈ {li,dd}` 且**折叠中点文本 caret**
  （boundaryOff>0 && eo<len 窄门）接 ① boundaryText 臂（头/尾双块嵌入
  host、空侧垫 br、caret 入新块首）——机制复用 div 宿主绿面同臂。h1/dt
  （行断退化维持）、span 行内宿主（block=li≠host）、普通可编辑列表
  （host=ul/block=li）均不入门；首/尾/元素 caret 与选区面无语料锚保守
  维持 ③ 现径。
- sepTag 未设回退 `sepTag=hostTag` → `(hostTag==='div')?'div':'p'`：⑩
  前该行仅 div 门消费（li/dd host 到不了 ① 消费面），语义恒等；扩展门面
  缺省落产线 'p'（li 嵌 li 非法形防护，双面制缺省语义）。⓪ void 臂自建
  newTag0、②b/②c 自建缺省式、③ br 不读 sepTag——消费面仅 ① 臂实证。
- part143 钉测 7 形：目标键 li/dd div 双块内嵌×2＋控制键 h1/dt/span 行断
  退化×3＋ul 宿主同标签兄弟拆分（:1438 绿形）＋dps 未设 'p' 面（产线
  缺省语义锚，自定回归基线非语料录音）。

## 门禁批（ip1 ALL-GATE-PASS）

判读锚＝od2-ip fresh A 腿（bba356b2d pre-knife，wall 974s）：
**7183P/12F**，Fail 集恰 I4 双键＋10 已知 recording 伪影（与 R20 终态恒
一）＋r21a-mt 实测锚 10090/2。

- L1 ip **7185P/10F/0T** wall 1084s——I4 双键翻绿恰整行集合（compare
  innerHTML 双 Pass 行）＋伪影集逐行 diff 恒一零新增红。
- L2 ilb **1691P/0F/0T** wall 38s——同函数 lineBreakOnly 面 SET-IDENTICAL
  （⑩ 门 !lineBreakOnly 互斥实证）。
- L3 lib **4189 passed / 0 failed** strict＋pin-in ip_island＋ok 行计数。
- L4 mt **10090P/2F/0T** wall 2211s SET-IDENTICAL 对 r21a-mt（Fail 集逐行
  diff 空＝X37 双键零暴露）。
- ksigs：二进制 arm1IslandLid=2；bin md5 全程恒一。全程见
  `.acceptance/dps/ip1-ledger.md`（未跟踪，收口后随档失效）。

## 推送与组合窗 spot check

批后 pull --rebase 并入上游窗（bba356b2d→8154da8e5 八笔：R5799/R5800/
R5802 rendering-compat css 面＋fv1/RP1 docs 活号＋s566 dialog key2——
part06.js 仅 dialog 面，不触本刀 paragraph-mutate 域）重写 d35da58db→
**b86912191** 推送。组合窗 spot check（重演 od2 同腿）：**7185P/10F**，
failset 与批 B 腿逐行 IDENTICAL（diff 空），I4 双键绿维持——上游窗零位移
实证。
