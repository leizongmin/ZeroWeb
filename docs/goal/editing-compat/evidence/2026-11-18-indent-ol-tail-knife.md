# indent C 族 ol 域尾区刀（R19）收口与双 ABORT 归因订正（2026-11-18）

刀体 **8e6bccebe**（已推 origin/main）：part01g.js 双刀面＋part140 钉测 4 形
（`test_indent_ol_tail_pins`）。

## 刀面（r18b-indent.log 3F 中本刀 1 键）

- C 族 ol 域尾键：`<ol><li>foo<ol><li>bar</ol>[baz]</ol>` compare innerHTML——
  got `<ol><li>foo<ol><li>bar</li></ol>baz</li></ol>`（baz 留内层尾区）、
  exp `<ol><li>foo</li><li>bar</li></ol><li>baz</li></ol>` 形族（Chrome 录形
  尾区整体出内层域）。

## 根因与修复（详见 commit 8e6bccebe 正文）

1. items 循环 normalize 尾区门：对每个 item li 取最后一个 ol/ul 直接子
   （lastList）；选区两端均严格位于其之后（`_zwJusBpCompare > 0`）→ 跳过
   `_zwListNormalizeSublists(itLi)`。尾区内容不属任何内层子列表，normalize
   把裸文本拽入内层 li 是 got 形的直接来源。
   spec：[normalize sublists](https://w3c.github.io/editing/execCommand.html#normalize-sublists)。
2. indent apply li 单项锚定收口：`_zwJusBlockExtend` 块扩展后若选区两端爬升
   同锚最近 li（liS===liE），把 range 收口到该 li 的列表父界
   `(parentNode, liIdx)`–`(parentNode, liIdx+1)`——单项形块扩展塌缩进 li 内
   侧导致新壳包住内层 ol 的残形收口。
   spec：[indent](https://w3c.github.io/editing/execCommand.html#indent)。

钉测 4 形（尾区 baz／前导 foo／跨内层选区／内层首 bar）＋blockquote 负断言，
期望串逐形经临时探针实证后落钉；pin 局部 1P/0F；clippy `-D warnings` 干净；
fmt 零漂。

## 批链：zw-r19 L6 ABORT → 归因订正 → r19b L6 PASS/L7 ABORT → r19c ALL-DONE

### 首射 zw-r19（07:03:46，pid 1462915，head=8e6bccebe bin 241b8441a9951115a96f6cc230eedd58 ksig lastList=6）

L0-L5 全绿（indent 1327/2 翻绿恰 1 ol 域尾键零新增红；fb/fs/img/event
SET-IDENTICAL）；**L6 ABORT：ie 1552/1776/3 vs 锚 1476/1852/3**。

### L6 归因＝窗内 d53 刀（初判「刀致位移」证伪订正）

- 翻绿集恰 **76 键 F2P、零新增红、Timeout 集恒一**：72 键
  `input-events-get-target-ranges-deleting-in-list-items.tentative.html`
  ＋4 键 `…non-collapsed-selection.tentative.html`（全 ol/ul 域）。
- v1 账本因果链第④证「唯一引擎变量＝刀体提交」**为假**：git log
  2b21adf7f..HEAD 复勘得 **22 个窗内提交**——r18b 锚捕于窗底 2b21adf7f，
  未含其后落窗的 d53/A1c/刀 S/shape-outside 等。窗底账按 legs HEAD 界定。
- 定谳＝窗内 d53 刀 **1d5044761**（html-semantics dil 归一刀）：72 dil 键恰合
  其账本自述「净 +72」；余 4 non-collapsed 键归同刀 `_zwCeSelectDelete`
  `_sdLiKid` 臂＋整节点摘除清空块占位 br——样本键对拍正合：
  `<ol><li>[list-item1]</li></ol>` Backspace r18b got `<ol><li></li></ol>`
  （空 li 缺占位 br）→ 现 Pass `<ol><li><br></li></ol>`＝其换代记账直接产物
  （A1c 为 transition 域与 ie 零交汇；本刀 ie 可达面＝exec-command 子测两次
  空爬升零写副作用）。
- 域确定性佐证：ie failset r13→r18b 六轮字节恒一＋同 head 同 bin solo 复跑
  与批 L6 failset 字节恒一（/tmp/r19-ie-repro.log）＋语料 mtime 全域
  Nov 17 12:21。

### 打捞批 zw-r19b（07:38:20，pid 1507756）

L6 重锚门（1552/1776/3 strict＋resolved 恰 76 键整行集合白名单 diff＋零新增
红＋T 恒一）**PASS**；L1-L5 SET-IDENTICAL 全 PASS。**L7 ABORT**：lib
4167 passed / **2 failed**＝`test_allow_discrete_registers_discrete_pair`＋
`test_allow_discrete_keyword_endpoints_flip`（transition_registry.rs:820-884，
`--exact` 隔离 0.00s 确定性红非串扰）。

### L7 归因＝A1c 窗刀自带测红（bisect 定谳）

- 两测均 A1c 窗刀 **ab7821b24** 新增（2b21adf7f 时点不存在）；本刀零 Rust
  逻辑，diff 无 Rust 面。
- bisect：git worktree @ab7821b24 全新构建（82 crate，lib 总测 4169＝4170−
  part140 ✓）逐测复跑——**双 FAILED 0.00s**（/tmp/bisect-a1c3.log、
  /tmp/bisect-a1c4.log）＝**A1c 自带测红**，其门禁未含 zero-engine lib。
  worktree 用后已删。
- 处置沿 2d3e71d19 font 外域红先例：skip＋留痕＋挂账 transitions 流。

### 三射 zw-r19c ALL-DONE（09:15:48）

八腿全绿 @8e6bccebe bin 241b8441 全程恒一：indent 1327/2 翻绿恰 1 ol 域尾键
整行集合＋fb 5043/3＋fs 2419/3＋img 962/0＋event 176/4＋ie 1552/1776 重锚
76 键 T 恒一＋**lib 4167/0 pin 9/9**（skip 3：font 外域红 2d3e71d19＋A1c 红
两测）＋**mt 10090/2 X37 multiplier2 恒一**（窗刀 d53 未位移 multitest，
wall 2004s）。脚本 r19c-ol-tail.sh（md5 d97ea12f0d5996fbfb6c080b7ba9e32e
冻结＋发射器内嵌 md5 断言）；逐腿墙钟 19s/—/—/12s/3s/159s/17s/2004s。

## 教训入账

1. **跨窗 SET-IDENTICAL 锚失效**：多刀窗内，窗底捕获的锚对被窗内刀触及的腿
   无效；ABORT 归因先 `git log 锚点..HEAD` 全列核实再定因果（r19-ledger.md
   v1 误归本刀已订正；「六轮 failset 恒一＋solo 复跑」只能证域确定性，
   不能证因果归属本刀）。
2. **外域新测红先归因再 skip**：lib 红先查测试是否存在锚点时点（git log
   -S），bisect 到引入提交头复测区分「自带测红」vs「窗内后续刀回归」；
   自带红＝引入流门禁缺口，挂账其流。
3. 派生批脚本 `sed` 改名须带 `/g` 并全局审计旧名引用（r19b 首派残留
   `"$D/r19-` 旧日志名险读首射旧 log）。

## 残 2F（indent 域，非本刀域）

| 族 | 键 | 状态 |
|----|----|------|
| E-表 | `<table><tbody data-start=0 data-end=1>…<p>extra` | 升级臂扩展待做（tbody 全覆盖判据，R17 排除域遗留） |
| F li 尾 br | `<ul><li>a<br>{<br>}</li><li>b</li></ul>` | 占位 br＋选中 br 双存待做 |
