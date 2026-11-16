# ihe3 判读档案——ihtml C 族占位块壳保留刀（2026-11-16）

刀体 770b38a87（part06 `_zwInsertHtmlShellResolve` 增 C 族 blockshell-keep-arm 块壳保留臂；part124 +4 钉 ihb-c-*）。门禁批 ihe3（脚本 `.acceptance/dps/ihe3-tail.sh`，md5 1efbd99e9c86eaa6afc979398bfcbc21，chmod 400 冻结，exec 前 md5 断言过）15:12:51 **ALL-DONE**，writer 全程在飞无代跑。

## 批锚与四腿结果

- LAUNCH_HEAD=770b38a87（＝刀体提交），bin=fef43652e9209af01a52331cfd9b1ed8（四腿恒一，chk_bin 逐腿复查），ksig=2/2（blockshell-keep-arm＋cOkCmt 特征串入二进制）。
- L1 iht A 腿：rc=2 rows=1565 **P=1551 F=14** wall=11s。
- L2 iht B 腿：rc=2 rows=1565 **P=1551 F=14** wall=11s。A≡B 红集 diff 恒等。
- L3 mt 腿：rc=2 **P=10090 F=2** T=0 wall=1654s，残红恰 subscript+formatblock X37 双键（与 ihe2 基线恒一），零暴露。
- L4 lib strict：rc=0 **4073 passed / 0 failed**（17.15s；发射前现树组合态＝ihe2 时 4068＋渲染流 58b3721ad applier 臂 +5，4 钉入既有测试函数 +0）。
- 独立复核（原始日志，不依赖脚本回显）：comm 对 ihe2 基线（1547P/18F）F2P=4 键零残留＋新红集空；4 键精确清单逐键 fixed-string 核对在册基线全数消失；A≡B 恒等。

## 语料 census：31F（ihe1 后）→ 18F（ihe2）→ 14F（清偿 C 族 4）

F2P=4：C 族 4 键（`<p>[foo]</p>` 全段选区 × {space, span display:none swcss 双态, comment}，corpus inserthtml :463-481）。红先绿后钉测证据：套臂前 `IHBPIN MISMATCH x4`——ihb-c-space got=` `、ihb-c-sspan-t/f got=`<span style="display:none"></span>`、ihb-c-cmt got=`<!--abc-->`，与 corpus 实录 got 逐字节同形＝钉忠实；套臂后 enginelib 全量 4073P/0F。

## 残 14F 族谱（iht A 腿实录，下一步刀界）

| 族 | 键数 | 键形 | 下一步 |
|---|---|---|---|
| A 族断载荷/未闭合壳 | 5 | `""`/`<b>`/`<b>abc` × foo[bar]baz × swcss | **下一刀**（staging 草稿已落 ihe4-a-knife.*：①去空载荷提前返回＝主臂 `if (frag === '') return;` 整命令 no-op 未删选区，got `foobarbaz`；②内联臂 ：3402 拼接前 `_zwIhtmlCompleteFrag` 补全——原串拼接开壳吞并后文，got `foo<b>baz</b>`/`foo<b>abcbaz</b>`，`<li>abc`/`<p>abc` 块载荷走块臂已补全全绿证爆炸半径恰为 A 族） |
| rawtext 壳 | 4 | `<b>abc</b>` × script/xmp × swcss 双态 | 维持（既有拼接臂外溢形，独立刀域） |
| E 空行内壳 | 2 | `<span></span>`/`<span s1></span>b<span s2></span>` × `<div>a[]b</div>` | 攒批搭车候选（空壳整丢＋case36 载荷文本连丢＝parser/materialization 域） |
| F CE-false 邻接 | 2 | `<i>Z</i>` × contenteditable=false span 邻接 | 攒批搭车候选（CE 域） |
| CE=false pre 链 | 1 | `<pre contenteditable=false><span contenteditable>[1234]</span></pre>` | 攒批搭车候选 |

## 与 ihe2 批对比

- ihe2（D 族占位 br 消费刀 36f3731b0）：iht 1547P/18F。ihe3 净增 4P 清偿，零新增红，mt/lib 恒一——C 族刀域收敛干净。
- 发射树含并行流合流：58b3721ad（clip-path/shape-outside 动画 applier，crates/engine +5 lib 测试）入发射基，iht/mt 严格计数门全过＝合流零语义位移的 editing 面实证；发射前 rebase 引入 808aa31a7/bf8276b39 均 docs-only（crates/engine 域 diff 空核证）。
