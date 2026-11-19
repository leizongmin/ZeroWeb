# ihe4 判读档案——ihtml A 族断载荷/未闭合壳刀（2026-11-16）

刀体 4f5e7a75d（part06 `_zwExecCmdApplyInsertHtml` 三编辑：①去空载荷提前返回＝主臂 `if (frag === '') return;` 整命令 no-op 改为空载荷走宿主直子主路径 splice('') 自然完成删除＋②壳域门空载荷守卫 `frag !== ''` ＝空载荷无壳语义跳过 ShellResolve＋③内联臂拼接前 `_zwIhtmlCompleteFrag` 补全＝未闭合首标签先补尾闭标签再拼接；part124 +5 钉 ihb-a-*）。门禁批 ihe4（脚本 `.acceptance/dps/ihe4-tail.sh`，md5 302fc4628cf7cd66a58d16fae9f73d61，chmod 400 冻结）18:56:52 **ALL-DONE**，writer 全程在飞无代跑（15:3x 席发射未遂＝429 断链仅落 COORD 条目未落脚本/unit，本席 18:13 限额重置续轮接手续发射，COORD 18:2x 条目在案）。

## 批锚与四腿结果

- LAUNCH_HEAD=4f5e7a75d（＝刀体提交），bin=7fa4cc0e9eac20d852b62c077cd5f92e（四腿恒一，chk_bin 逐腿复查），ksig=2/2（unclosed-completion-arm＋_zwIhtmlCompleteFrag 特征串入二进制）。
- L1 iht A 腿：rc=2 rows=1565 **P=1556 F=9** wall=13s。
- L2 iht B 腿：rc=2 rows=1565 **P=1556 F=9** wall=11s。A≡B 红集 comm 恒等。
- L3 mt 腿：rc=2 **P=10090 F=2** T=0 wall=1948s，残红恰 subscript+formatblock X37 双键（与 ihe3 基线恒一），零暴露。
- L4 lib strict：rc=0 **4074 passed / 0 failed**（17.40s；组合态＝ihe3 时 4073＋渲染流 21872a345 Steps 钉 +1，5 钉入既有测试函数 +0）。
- 独立复核（原始日志，不依赖脚本回显）：A/B 红集 comm 0 行恒等；comm 对 ihe3 基线（1551P/14F）新红集空；5 键精确整行 fixed-string 双腿全数消失（ALL-5-GONE）；残 9F 逐案与族谱对账吻合。

## 语料 census：106（ihb3 前）→ 57（ihb3）→ 49（ihc1）→ 41（ihd1）→ 31（ihe1）→ 18（ihe2）→ 14（ihe3）→ **9F（清偿 A 族 5）**

F2P=5：A 族 5 键（`""` 空载荷 :103＋未闭合 `<b>`/`<b>abc` :118/:123 × swcss 双态，× foo[bar]baz）。红先绿后钉测证据（刀提交前实测，4f5e7a75d commit body）：套臂前 `IHBPIN MISMATCH x5`——ihb-a-empty got=`foobarbaz`、ihb-a-b-open-t/f got=`foo<b>baz</b>`、ihb-a-b-txt-t/f got=`foo<b>abcbaz</b>`，与 corpus 实录 got 逐字节同形＝钉忠实；套臂后 enginelib 全量 4074P/0F。`<li>abc`/`<p>abc` 块载荷走块臂已补全全绿＝爆炸半径恰为 A 族。

## 残 9F 族谱（iht A 腿实录，下一步刀界）

| 族 | 键数 | 键形 | 下一步 |
|---|---|---|---|
| rawtext 壳 | 4 | `<b>abc</b>` × script/xmp × swcss 双态 | **最大独立族**（既有拼接臂外溢形，独立刀域） |
| E 空行内壳 | 2 | `<span></span>`/`<span s1></span>b<span s2></span>` × `<div>a[]b</div>` | 攒批搭车候选（空壳整丢＝parser/materialization 域） |
| F CE-false 邻接 | 2 | `<i>Z</i>` × contenteditable=false span 邻接 | 攒批搭车候选（CE 域） |
| CE=false pre 链 | 1 | `<pre contenteditable=false><span contenteditable>[1234]</span></pre>` | 攒批搭车候选 |

## 与 ihe3 批对比

- ihe3（C 族占位块壳保留刀 770b38a87）：iht 1551P/14F。ihe4 净增 5P 清偿，零新增红，mt 恒一——A 族刀域收敛干净。
- 发射树组合态：21872a345（渲染流 Steps 位置语义钉）入发射基，iht/mt 严格计数门全过＝合流零语义位移 editing 面实证（lib 4074 与 15:2x 现树实测一致）。
- 发射接手归因：15:3x 席「门禁批脚本落盘发射」COORD 条目为发射未遂（429 断链，脚本未落盘 unit 未建），本席核证后续发射；单批契约由本席开立并收尾，全程无第二批。
