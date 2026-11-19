# ihtml 轴二壳界选区刀门禁批收口（ihd1）

**日期**: 2026-11-16
**刀体**: c19712be9（part06 `_zwInsertHtmlShellResolve` 扩元素端点归一＋主臂入口门放行；part124 +8 钉）
**批**: ihd1 单批一次 ALL-DONE（发射 10:21:21，终态 10:57:30，writer=469684，unit zw-ihd1）
**census**: inserthtml 全页 1516P/**49F** → **1524P/41F**（F2P=8，零新增红）——ihc1 残 49 中轴二元素端点族 8 键清偿

## 根因与归因订正

ihb3 族谱将 span color 壳 × 行内载荷 8 键归「内联臂携带域」（预期 override carry 修色）；ihc1 判读后重取实录：4 形 got 均为**原串 no-op**（壳原样残留、载荷未插、选区未删）——非丢色，是删除+插入管线整臂旁路。订正归因：

语料 `{}` marker 落壳边界时（`elStartIdx==0` 形，include/tests.js setupDiv 同款 `startNode=parentNode`）产生**元素端点 range**（sc/ec 为元素节点、offset 为子节点索引），既有内联臂 `_zwExecCmdApplyInsertHtml` 入口门 `sc.nodeType===3` 直接 return——4 形全部命中此门（形① sc=ec=壳；形② sc=ec=host；形③ sc=host/ec=壳；形④ sc=壳/ec=host）。

## 刀体内容

`crates/engine/src/js_dom_shim/part06.js`：

- **主臂入口门**放行元素端点（`sc.nodeType===1` 入壳域解析）；非元素非文本维持 return。
- **ShellResolve sc 端点归一**：元素端点语义 = 子节点索引——sc===host 归一原点 0、so 前子文本累积；sc 为一层壳归一壳基 + 壳内 so 前子文本累积；更深旁路。`ws` 计算按 sc 节点类型分流（元素形 scBefore 已吸收 so，不再 +so）。
- **ec 端点归一**：元素同节点（形① eoEnd 前子文本累积）、元素另一壳（形③ 壳基 + eoEnd 前壳内累积）两新分支；ec===host 元素形落既有 spans 累积分支（子节点索引语义同构）。归一后 ws/we 与文本端点同构，拼接/溶解臂零改动。
- 钉测 `part124_ihtml_block_pins.rs` +8（`ihb-elm-in/around/cross-left/cross-right` × swcss true/false，期望值=语料 expected 串原样——溶解形期望不含色壳，与色 carry 无关）。

## ihd1 判读（ALL-DONE 10:57:30 @bin 96c1a096）

| 腿 | 结果 | 判定 |
|---|---|---|
| L1 iht A（乘数 2，wall 42s） | rows 1565 恒一，**1524P/41F/0T** | F2P=8 恰轴二元素端点族（style=color:#aBcDeF 壳 × ab<b>c</b>d 载荷 × 4 选区形 × swcss 双态 compare innerHTML 行，comm 对 ihc1-iht-a.log 基线双证）＋零新增红 |
| L2 iht B（wall 11s） | 同上恒一 | 双腿确定性成立 |
| L3 mt（wall 1927s） | **10090P/2F/0T** | 残红恰 subscript+formatblock X37 双键零暴露——mt 语料 `[]` 文本端点形，元素臂不触发 |
| L4 enginelib strict（16.4s） | **4060P/0F** | 组合态（ihc1 时点 4059＋渲染流 a7c81024d part03 +1，发射前现树实测定谳） |

腿 rc=2 为 make 底层对语料残余红的常态退出码，门=计数＋judge 双证（editing-gate-rc 纪律）。锚定 v2 全程抗性（LAUNCH_HEAD=c19712be9 域冻结＋bin 字节 96c1a096 三腿恒一＋docs 轮并行提交不误伤）。

钉测链：8 钉先红后绿（刀前 cargo test 红——got=原串 no-op 与语料实录逐字节同形；刀后全绿＋既有钉零位移）。

## 残 41 红族谱（49−8 对账闭合）

| 族 | 键数 | 处置 |
|---|---|---|
| `{}` 元素边界坍缩选区（占位 br 消费/注释壳） | 12 | 溶解臂全含门对 collapsed 选区恒 false——旁观 census 预判成立（ihd1 judge 零新增红+F2P 恰 8 键域兑现）；占位 br 消费属独立机制（ilb 刀先例），后续刀 |
| ws-pre 尾换行 caret 载荷丢失 | 10 | 文本端点形，元素臂不覆盖——ws 域后续刀 |
| span color 壳＋行内载荷 | 0 | **A 族 8 键本刀清偿** |
| rawtext 壳（xmp/script） | 4 | rawtext 域维持 |
| CE=false pre 链 / multi-root / display:none / 断载荷等 | 15 | 挂账维持（ihc1 census 档案全谱） |

## 下一步

- **D 族占位 br 消费刀**（12 键，`{}` 坍缩选区 × 占位 br/注释壳——ilb 占位 br 消费先例外推）；
- ws-pre 尾换行 10 键（文本端点 caret 位载荷丢失）；
- multi-root 载荷扩展 1 键搭车；assert_in_array 9 键收尾批量判读、exec1 474 域决策挂账维持。

## 批脚本与协调

`.acceptance/dps/ihd1-tail.sh`（md5 a330d4b371fb6958daca378ffd8288b5，chmod 400 冻结）。日志链 `.acceptance/dps/ihd1-{build,iht-a,iht-b,mt,lib,tail}.log`。残 49 全集普查见 [ihc1 残 49F census](2026-11-16-ihc1-residual-49f-census.md)（旁观轮，含本刀钉测 corpus 忠实性核验）。跨执行体协调全录 `.acceptance/dps/COORD.md` 2026-11-16 10:22/10:23 条目（含共享暂存区竞速拆分处置）。
