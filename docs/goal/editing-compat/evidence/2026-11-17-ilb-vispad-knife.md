# ilb A 壳尾计数族刀收口（ILB-VISPAD-V2，2026-11-17）

刀体 3dd6eb716＋v1.1 归因修复 57c7d2cd0（`fix(editing): ilb A 壳尾计数族刀
——行断可见性补断 v2 文档序块界扫描` ＋ `pre 域 ws-only 文本保形可见性判定`）
＋门禁批 i7d0 v2 ALL-DONE（head=57c7d2cd0，bin
f4cb9383ba1ff2e4956fc0b87540261d，2026-11-17 02:06 完走）。census 刀序 ②
兑现，F2P 恰 5 键零新增红，ilb 残 22F。

## 基线

刀②基线＝i7c0-ilb-a.log（1664P/27F head=3312189d7）——刀①门禁双腿 fresh
（01:09 完走）即现树面态，协议免重跑基线腿。

## 根因与修复面

`_zwCeLineBreakAt` 可见性补断旧规则 `!brk3.nextSibling` 只看同父直接兄弟，
双向失真：inline 壳尾（`<b>foo[]</b>bar` 形）块内可见后续被误双垫（A 族 4
键：idx 458/468/518/538），注释邻接（`<p>foo[]<!--bar-->` 形，idx 528）有
兄弟（注释不可见）漏垫。

v2：br 型（normal-ws 域）按 `_zwCeBrVisibleAfter` 文档序前向扫描至最近块界
定补断——非折叠空白文本可见、注释/其它节点型不可见、元素含可见后代可见、
空子树仅替换型元素可见（img/br/hr/input/…）；inline 父链上爬白名单（显式
inline 或无声明 UA-inline 标签；窄表块标签＋dt/dd/pre/显式非 inline/宿主即
块界）。spec implementation.js :795 isCollapsedLineBreak（br 后插 zwsp 看最
近非 inline 祖先高度）shim 无渲染域结构近似；:7459 collapsed line break 补
第二条。wsSig（\n 型）维持旧规则（H+J 归一域刀 ③ 另案）。

## v1.1 归因修复（i7d0 v1 ABORT）

v1 门禁 ilb-a 1667P/24F ABORT：5 A 键全翻绿但两现役绿键 P2F——
`<pre>foo[]&#10;</pre>` / `<pre>foo&#10;[]&#10;</pre>`（idx 253/258）。归因：
`_zwCeIlbNodeVisible` 折叠空白判据（_zwCeIsCollapsibleWsOnly）把 pre 域
ws-only 文本误判不可见→误补断；该面 keys 走 br 型（引擎 wsSig 世界模型读
声明链，bare-pre 无声明→normal→br 形）。修复：ws-only 文本经
`_zwCeIlbWsPreserved` 祖先链判定（pre/listing/plaintext/textarea 标签或声明
pre 系 white-space/break-spaces 即保形可见），normal 域折叠语义不变（A5
判据不回退）。part127 增补两 P2F 回归钉。

## 钉测（part127，同提交常驻）

`test_ilb_shell_tail_vispad_pins` 9 钉：A 族 5 语料形（idx 458/468/518/528/
538，A4 选区形 style attr 锁引擎原始 hex 形——语料 rgb(171, 205, 239) 归一
在 harness 比对面）＋控制键 2（idx 203 块尾双垫现役绿形＋idx 523 注释邻接
块起点现役绿形）＋P2F 回归钉 2（idx 253/258 pre LF 保形）。定向通过。

## 门禁批 i7d0 v2（刀后 ALL-DONE）

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 | bin=f4cb9383… ksig=2 head=57c7d2cd0 |
| L1/L2 ilb 双腿 | 1669P/22F/0T | 1669P/22F/0T 双腿同数（38s/38s） |
| L3 键集 judge | 零新增红＋F2P 恰 5 A 族刀键 | F2P-EXACT-5（白名单同配方提取逐字节恒一） |
| L4 enginelib strict | 0F | **4078P/0F**（含 part127 新钉） |
| L5 mt 腿 | 残红恰 X37 双键恒一 | **10090P/2F/0T** vs ihe6-mt 基线逐字节恒一（wall 2112s） |

v1 批工件（ABORT 归因存证）：`.acceptance/dps/i7d0-ilb-a.log` 被 v2 重写，
P2F 两键 Fail 行全文在 v1.1 commit message 与本节归因段在档。

## 尾巴

ilb 残 22F＝census 刀序 ③H+J 归一域 5（ws 坍缩 4＋nbsp 1）→ ④I flex/grid
4 → ⑤B+G+E 选区删除域 8 → ⑥D rawtext 2（+3 E1 等待归域）。批工件
`.acceptance/dps/i7c0-*` / `i7d0-*`。
