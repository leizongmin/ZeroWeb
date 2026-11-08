# F1'-B typing wrap 放置/合并刀（2026-11-08）

## 根因链（三层证据）

1. **B 族 4 键病理**（f1p-mt-post.log 基线 got 签名，main :20072-20086 / ab :9973-9987 双页在场）：
   - [19]/:584 `<b>a[]</b>c` + bold-off + italic-on + insertText b：expected `<b>a</b><i>b</i>c`，got `<b>a</b>bc`（typed 段裸文本）。
   - [20]/:585 `<i>a[]</i>c` 同链镜像：expected `<i>a</i><b>b</b>c`，got `<i>a</i>bc`。
   - [21]/:586 `a[]<b>c</b>` + bold + italic + insertText b：expected `a<b><i>b</i>c</b>`，got `a<b><i>b</i></b><b>c</b>`（分裂形）。
   - [2]/:559 `<span style=font-weight:bold>{}<br></span></b>` + stylewithcss(true) + italic + insertText a：expected `<span style="font-weight:bold; font-style:italic">a</span>`，got `<span style="font-weight:bold"><i>a</i><br></span>`（tag 壳 + 占位 br 残）。
2. **引擎考古**：
   - D4 拆壳臂（2026-11-02 刀）innerTags 只收 off 壳内幸存链（`_zwD4Chain` 祖先链滤 off tag），显式 toggle-on 武装态（`__zwTypingTags`）不入 → off 拆出后 typed 段丢显式 tag → twD=null 裸文本（[19][20]）。
   - 前向并壳缺失：arm①/③ typed 壳落位后无同 tag 合并步 → 分裂形（[21]）。
   - swcss：`__zwStyleWithCSS` 仅存储（part06 styleWithCSS 命令置位），CSS 化 wrap 行为此前未接；`_zwSync` 清除序列不含它 → :559 置位跨案泄漏（recording 每案隔离 → Chrome 无观察面；语料无 qS(stylewithcss) 读数面 → 引擎零观察面，清除为生命周期卫生）。
   - **insertBefore(handle 元素 × 解析文本 ref) = part04 sel 域 wire 死区**：proxy trap（part04.js:9870 起 handle-newNode 分支链）走查——D3 臂 `refNode.nodeType === 1` 门（:10197）、R265 族 `handle &&` 父门（:10105/:10127，host 为 sel-only handle=null）、s165 `refNode.__zwHandle` 门（:10171）、R5058 `newNode.nodeType === 3` 文本子门（:10183）全 miss → 静默 no-op（:10236 注释自认「罕见」缺口）。dbg kids 实证：插入调用无错返回后 childNodes 恒 `B/linked|t:c`。arm① 不触坑：`sc.nextSibling` 恒 null 或元素兄弟（解析树文本节点不相邻）；D4 臂独有文本 ref 形（`offSplit.nextSibling`）。
3. **绿面对照**：:587 `a[]<i>c</i>` + bold + italic + insertText b 绿（expected `a<b><i>b</i></b><i>c</i>`）——外层 tag 异（typed b 壳 vs 后继 i 壳）即不并；同 tag 规则自然保护，无需特判。

## 刀体

- part01.js（+~75 行）：
  - `_zwCeSwcssTypingProps` 表（b/i/u/s → CSS prop 声明串）+ `_zwCeSwcssAbsorbSpan(sc, el)`：`__zwStyleWithCSS === true` ∧ tags 武装 ∧ caret 祖先链（至宿主）首个 span 的 style 无同名属性（冲突或冗余均不并）→ `{sp, add}`，否则 null（fallback 既有 tag 壳径；FIXME 嵌套 span 壳形未接）。
  - mutate core 预决策 `swAbs`/`swBr`（sole-child 占位 br 快照挂变异前）；D4 臂 / arm① 条件加 `&& !swAbs`（arm②/③ 元素 caret 形不触 swAbs 链）。
  - absorb 落笔块：style attr verbatim 追加 `'; ' + decl`（Chrome style attr 序列化恒 `; ` 分隔，part01d.js:348 先例）+ swBr 显式摘除（占位 br 消费）。
  - `_zwCeTypingForwardMerge(twEl, hasF2)`：typed 壳 tag ∈ {b,i,u,s} ∧ 紧随兄弟同 tag 壳 ∧ 无 f2（link 面独立吸收机制）→ 子显式摘除后头插进后继壳 + 空壳移除（zz-probe-d4x X4 隐式 detach 缺口先例——显式 removeChild 再插入）；arm① 尾段空时（尾段非空并壳会错位残段）+ arm③ 无条件调用。
  - D4 臂显式 tags 并入：链壳外→内序 + `__zwTypingTags` 垫内层（已含去重）。
  - D4 臂文本后继改锚插入：`offSplit.nextSibling` 为文本节点时——显式摘除文本（X4 教训）→ dEl append（null-ref 分支）→ 文本 `createTextNode` 尾追回填（ref = `dEl.nextSibling || null`）。FIXME: 文本后继尚有元素兄弟的位序保持未接（回填尾追落尾；语料普查无此形）。
- part06.js（+6 行）：`_zwSync` 清除序列 F1'-A override clear 块后挂 `globalThis.__zwStyleWithCSS = false`——选区变更即清（每案 setup removeAllRanges/addRange）；结构命令 presTt 同款保全未接（语料无 [swcss(true)+结构命令+typing] 链形，FIXME: 出现该链形时需 presSw 快照回写）。
- md5 锚（批硬绑定）：part01=`a82df7cfe839e61a45d6630e64c5724d`、part01d=`5ab2368ec895e74527c510847e5a9f83`（F1'-A 不变）、part06=`5e78d9a4ce56d0946eb051c6bfae42df`。

## solo 对照（zz-probe-f1b.html B1-B7 七案全绿）

| 案 | 形 | 判读 |
|---|---|---|
| B1 | :584 形 `<b>a[]</b>c` | 绿：`<b>a</b><i>b</i>c` 逐字节（刀前裸文本） |
| B2 | :585 形 `<i>a[]</i>c` | 绿：`<i>a</i><b>b</b>c`（刀前裸文本） |
| B3 | :586 形 `a[]<b>c</b>` | 绿：`a<b><i>b</i>c</b>`（刀前分裂形） |
| B4 | :587 护栏 `a[]<i>c</i>` | 绿：异 tag 不并逐字节 |
| B5 | :559 录音形 swcss+占位 br | 绿：`<span style="font-weight:bold; font-style:italic">a</span>` |
| B6 | swcss 清除护栏 | 绿：选区变更后 typed 回落 tag 壳（无 style 串） |
| B7 | 冲突护栏 | 绿：同名属性 span 保留 + fallback tag 壳 |

诊断三轮（埋点逐轮撤净，终态 `grep -c __zwDbgD4 = 0`）：

1. D4 臂内态 dump `{off:true, tags:["i"], toff:["b"], len:1, wrapped:true}` + 插入点后 innerHTML/kids 快照 `ins:<b>a</b>c kids=B/linked|t:c`——元素创建成功、insertBefore 无错返回后即刻不可见 → **wire 死区定谳**（D4 逻辑无误）。
2. 三步快照：rm=`<b>a</b>`（文本摘除成功）、ins=`<b>a</b><i>b</i> pn=ok`（dEl append 成功位序正确）、post=`<b>a</b>c<i>b</i>`（回填 ref=dEl 落 dEl **之前**——错位根因）。
3. 回填 ref 改 `dEl.nextSibling || null`（arm① 尾段 ref=null 先例——F2 语料 `foo[]bar`+link 绿面 `foo<a>a</a>bar` 同径）→ B1/B2 翻绿，七案全绿。

校验法：part01.js 为拼接 shim chunk 非独立可解析（HEAD 同样不可解析），node --check 不可用——Python brace-balance delta（HEAD vs 工作树 `{`/`(`/括号差相等）逐轮校验 BALANCED-DELTA-EQUAL。

## 机制预测

- mt 翻键 = 恰 4 键 ×双页 = 8（entry 级）；assertion 级 main 域 F2P = 4。
- 爆炸半径普查（语料 628 条 grep 定谳）：swcss(true) 全语料恰 1 处（:559）→ 吸收臂单点；caret-before-shell+2fmt+inserttext 唯 :584/:585 → D4 显式 tags 单域；caret-before-shell 并壳形唯 :586/:587 → 前向并壳单域（:587 异 tag 绿面由同 tag 规则保护）；文本后继改锚仅触 D4 臂文本 ref 形 = :584/:585；arm①/③ 前向并壳调用点对既有绿面零命中（绿面若存在同 tag 后继壳形，刀前即应红——无）。
- part06 swcss 清除对其它条目中性：CSS 化 wrap 未接 → swcss 态无其它消费者；语料无 qS(stylewithcss) 读数面（record {}）→ 滞后快照读数不受影响。
- 窗口：F2P ∈ [4,16]，翻键 ⊆ 四键签名白名单（:559 swcss span / :584 `<b>a[]</b>c` / :585 `<i>a[]</i>c` / :586 `a[]<b>c</b>`，均 insertText 锚）。

## 门禁批判读

主批 batch-f1p-b-typing-wrap-gate.sh 20:51:53 发射（head=5ca8c1fe7，锚三枚 md5 硬绑定 part01/part01d/part06 全过 + 特征串五门 + F1'-A 引用计数=2 + F3 两串），已过腿逐段 VERDICT：

- 基线自证：judge(f1p-mt-post.log vs 自身) 零漂移 OK（20126 rows）。
- probe-f1b **7/0**（B1-B7 刀 fixture）、probe-f2 **14/0**、probe-f3 **7/0**、probe **224/17** judge 零漂移（rows 229 恒一）。
- ip **7195 rows 恒一**（F 206 恒）、inserthtml **1565 恒一**（F 119 恒）、exec 三腿 **1806/1/6 rows 恒一**——P2F 0 全过。
- **mt judge：rows 10092→10092，P 10022→10026，F 70→66，F2P 4（assertion 级 main 域）＝机制预期恰值，P2F 0 / post-only 0 / lost 0 → VERDICT OK**。
- calibrate：entry 级翻键 **8**＝四键×双页逐行命中（ab 4 + main 4；flip 文件全量人工复核 + 残余批定长 grep 复验 8/8 中、域外 0）。
- **ABORT 披露（门脚本瑕疵，非刀体失败）**：主批 WL_PAT ERE 括号不平衡（`grep: Unmatched ( or \(`）→ 白名单计数空转触发 ABORT——mt 判据数据完好且全绿，判读由 flip 文件人工复核 + 残余批 batch-f1p-b-residual-gate.sh 定长匹配（grep -F，签名含 `()[]<>` 无需分组）双重落地，F1'-A v1.1 门瑕疵披露先例同型。
- ab 页 postF 5（post-only Fail）：4 行 delete/forwarddelete×inserttext 表格链族（knife10 已定谳 ab 变体页波动产物域）+ 1 行 forwarddelete>inserttext `<p>abc[]<br>` 形（同 delete 族非本刀键面）——本刀四键无 delete 链入路，全局 P2F 0，归 ab 截断/波动伪象族不追。
- 残余批（二进制与主批同构建，锚 md5 发射前重验）：iol/iul vs v5fix2 零漂移、enginelib 6 红名册、battery 七腿——结果见下节。

### 残余批判读

batch-f1p-b-residual-gate.sh 21:38:41 发射（log 头 pid=3838206 实证；初稿「21:53」系时间笔误，账本前置核账条同勘误）→ **21:54:23 ALL-DONE 全绿**：白名单定长复验 8/0（每签名=2）；iol **1761 rows 恒一**（F 2 伪影恒）VERDICT OK、iul **2065 rows 恒一**（F 4 恒）VERDICT OK；enginelib **3817P/6F** 六红名册逐名恒一（P +2 系基座新增测试，非本刀面）；battery 七腿 F=0 T=0（420/880/1104/552/3456/2607/1455）。

**F1'-B 收口 VERDICT OK**：刀体三 shim（part01 刀体 + part06 swcss 清除 + part01d 不变）+ 探针 zz-probe-f1b 七案 + 全门禁链（主批五段 + 残余批四段）。

## F1' 记账

- **B 族 4 键全翻清账**：[2][19][20][21] ×双页 entry 8 翻（mt judge F2P 4＝预测恰值零意外键）→ main 域 Fail 70→66。
- F1' 族（普查档 styled 壳重建 29 键）经 A/B 两刀收口：A 刀（override 清除）收 A 族污染层 25 键中的 fs5 系+白名单键（v1.2 F2P 3），B 刀收余下 4 键——**F1' 域内残键清零**；A 族 [3]-[18] 无位置派生缺陷面与 [1] br 残壳（挂 F5/F6）不属 F1' typed-wrap 域，维持原挂账。
- 设计稿偏差声明（收口确认）：[2] 采用 insert 时吸收（tags 不动）而非 command 时 carrier——qState 面零触碰（S5 恒绿，B5 逐字节达标）；前向并壳窄门未检 attributes（语料无 attr'd 同 tag 后继壳形，FIXME 已记刀体节）。
- 挂账移交：①文本后继+元素兄弟位序保持（R5058 beforeend 兜底落尾，语料无此形）；②嵌套 span 壳形吸收（语料单点 :559 为平壳）；③swcss(true)+结构命令+typing 链形的 presSw 快照回写（语料无此链形）；④swcss 载体仅在 shim 态存活、CSS 化 wrap 的 queryCommandState 读面未接（语料无读数面）——四项均语料零命中，出现新语料再议。
- mt 残 66F（main 域）下一刀：③ F6b sub/sup+fontsize 14 → ④ F4 ilb 10 / F7 misc 18 分治。
- 核验回执（17:33 前身链 rollover 实例，独立复核与上两节同判）：postF 5 逐条 grep f3-mt-post.log（F3 时代全覆盖 20184 rows）全部命中 Fail——比「波动伪象」更强的直接证据：F3 时代既红非刀致，仅 F1'-A 基线 ab 截断墙早 58 行未覆盖；WL_PAT v1.2 修正独立复验 8/0（ERE 与 -iF 双径同数）；evidence 门禁批判读/记账两节由 21:53 续轮先笔（与本执行体 21:42 认领注竞速，内容逐项核对无冲突），本执行体按回执接 goal 控制面与提交尾链。
