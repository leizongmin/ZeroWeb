# ilb H+J 归一域刀收口（WSCANON-V1，2026-11-17）

刀体 4e091e6f7＋v1.1 保形门归因修复 7356bf2aa（`fix(editing): ilb H+J 归一域
刀——行断面空白归一三则（WSCANON-V1）` ＋ `UA 空白保形上下文整体不入归一臂`）
＋门禁批 i7e2 ALL-DONE（head=7356bf2aa，bin
a25ed41494bdd3eeb9f66780b8ace76b，2026-11-17 03:22 完走）。census 刀序 ③
兑现，F2P 恰 5 键零新增红，ilb 残 17F。

## 基线

刀③基线＝i7d0 v2 门禁双腿（1669P/22F head=57c7d2cd0，02:06 完走）——刀②
fresh 即现树面态，协议免重跑基线腿。

## 根因与修复面

H 族 4 键（`<div>abc [] </div>` / `abc[]  ` / ` [] abc` / `  []abc`）＝br 型
（normal-ws 域）拆点空白不归一：got caret 邻折叠 ws 原样保留
（H1 got `<div>abc <br><br> </div>` / exp `<div>abc<br><br></div>`；H3 got
`<div> <br> abc</div>` / exp 数组接受 `<div><br>abc</div>` 与
`<div><br> abc</div>`，仅删左段尾随 ws 即落接受变体）。J 键（CE-false 壳邻接
` <span contenteditable=false>A</span>[] ; …`）＝caret 后 ` ; ` 未 nbsp 化
（exp 唯一变体 `<br>&nbsp;;`）。

v1（WSCANON-V1）拆点三则归一，全部只挂 br 型（wsSig \n 型 pre 域不入臂）：

- (a) 左段尾随普通空格 run 删——spec implementation.js
  canonicalizeWhitespace :5407-5418 行尾空格 deleteData 形作形态参考；
- (b) 右段全折叠空白整删（H1/H2 右段 ws-only 行；删后既有 pad v2 扫描补第二
  br）；
- (c) 右段首部 [空格|nbsp] run：首 nbsp 翻、run 内其余 nbsp 翻回空格
  （ip _zwCeSplitBoundaryNormalize 行首同形）——仅头侧结构内容放行
  （`_zwCeIlbHeadStructural` 回扫：元素或非折叠空白文本即结构，host 为界）。
  J CE-false 壳邻接放行 nbsp 化；退化头 `[] abc` 形保形绝不 nbsp（H3 接受
  数组不含 `<br>&nbsp;abc` 形，无条件套用会打翻 H3——ip :13009 同判据）。

规范锚：insertlinebreak 规范臂（implementation.js :7383-7469）**无
canonicalizeWhitespace 调用点**——H/J 期望是 Chrome 实录行为（语料 ：1000 注
"expectations are based on Chrome's behavior"），三则锚语料期望，非规范步骤
移植。

## v1.1 归因修复（i7e0 v1 ABORT）

v1 门禁 ilb-a 1672P/19F ABORT（期望 1674P/17F，恰 2 P2F）：bare-pre 键
`<pre>foo[]&#10;</pre>` / `<pre>foo&#10;[]&#10;</pre>`（idx 253/258，刀②
v1.1 前科同款）无 pre 声明 → wsSig=false 走 br 型，规则 (b) 的
_zwCeIsCollapsibleWsOnly 把保形 `\n` 尾判折叠空白整删——UA 空白保形上下文
里保形 \n 是可见内容。修复：`wsp3 = wsSig || _zwCeIlbWsPreserved(boundaryText)`
（pre/listing/plaintext/textarea 标签链或声明 pre 系祖先，刀② v1.1 同款
判定），三则归一统一挂 `!wsp3` 门，normal 域折叠语义不变。ilb 钉测全家
3 passed/0 failed（part126＋part127 含 pre P2F 回归钉＋part128）。

## 钉测（part128，同提交常驻）

`part128_ilb_wscanon_pins.rs` 7 钉：H×4 语料形＋J＋退化头控制＋nbsp-first
翻转恒等控制。定向 1 passed/0 failed。

## 门禁批链（i7e0 → i7e2）

- **i7e0 v1 ABORT**（02:33 head=4e091e6f7 bin=017d06b97…）：ilb-a
  1672P/19F vs 期望 1674P/17F，恰 2 P2F bare-pre 键（红线清单在 tail log 与
  v1.1 commit message 在档）→ v1.1 修复 7356bf2aa。
- **i7e1 judge ABORT**（02:40 head=7356bf2aa bin=a25ed41494…）：双腿
  1674P/17F 计数过，L3 F2P-SET 失配——diff 显示翻绿集与白名单**同键集不同
  sort 序**（前导空格键 `  []abc` / ` [] abc` 排序位次不同），内容恒等；
  judge exp 侧 sort 对账修复后重射。
- **i7e2 ALL-DONE**（02:45:47 pid=1230288 → 03:22:17）：

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 | bin=a25ed414… ksig=2 head=7356bf2aa |
| L1/L2 ilb 双腿 | 1674P/17F/0T | 1674P/17F/0T 双腿同数（49s/41s） |
| L3 键集 judge | 零新增红＋F2P 恰 5 H+J 刀键 | F2P-EXACT-5（H1-H4＋J，白名单同配方提取逐字节恒一） |
| L4 enginelib strict | 0F | **4099P/0F**（含 part128 新钉＋并行流 css-gaps lib 新增，组合态基线） |
| L5 mt 腿 | 残红恰 X37 双键恒一 | **10090P/2F/0T** vs ihe6-mt 基线逐字节恒一（wall 2072s） |

批 02:45 发射后脱会话存活（429 断会话后 detached 完走），批工件
`.acceptance/dps/i7e0-*`（v1 ABORT）／`i7e1-*`（judge ABORT）／`i7e2-*`
（终判）。

## 尾巴

ilb 残 17F＝B 元素端点选区删除旁路 6＋G 块级壳全含 1＋E table 选区/落位 4＋
I flex/grid br 元素面 4＋D rawtext no-op 2。census 刀序 ④**I flex/grid 4**
（探针已确认：`<div style=display:flex;white-space:pre><span>abc[]</span></div>`
got `<span>abc\n\n</span>` / exp `<span>abc\n<br></span>`——wsSig \n 型臂插
`\n` 后缺 br 元素）→ ⑤B+G+E 选区删除域 8（+3 E1 等待归域）→ ⑥D rawtext 2。
