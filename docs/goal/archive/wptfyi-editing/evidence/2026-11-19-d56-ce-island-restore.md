# ⑩ CE-island li/dd host 拆分刀恢复（rebase 误删回填）A/B 证据（2026-11-19）

> **收尾勘误（2026-11-19 push 时）**：editing-compat 流 e23 门禁批已独立捕获
> 同一误删并以 1cbcb7420 恢复（带恢复注注释，见 7db4d85fb/5ca52641c）——
> 本流恢复与该恢复**功能逐字节同**（rebase 冲突实况：两侧 `sepTag` 三元、
> `arm1IslandLid` 门完全一致，仅差恢复注注释行），冲突取远端侧收场。本文
> A/B 遂为恢复终态的**独立验证账**（纯净对/全量 battery/enginelib 均按
> 远端同款代码测得），非恢复本身。

## 性质与事故链

本提交不是新刀：并行流 2026-11-18 20:58 已把 ⑩ 落树（b86912191：part01.js
两 hunk ＋ `crates/engine/src/js_dom_bridge_tests/part143_ip_island_li_dd_pins.rs`
钉测）。本流在不知情下以 ⑩ 剥离树落 d55 提交（a763e74b1，剥离为有意——上轮
session 429 死前 23:21 留下的 ⑩ WIP hunks 未构建未 A/B，落树前剥离并记 d55
证据残留第 3 条），`git pull --rebase` 后该提交 diff 相对父代把 ⑩ 标为删除行，
rebase 重放到已含 b86912191 的 main 上即静默移除 ⑩ hunks（`git show -S
arm1IslandLid` 计数变化实证）；part143 钉测文件本流未触碰仍留 HEAD——钉测
断言 `<div>ab</div><div>cd</div>` 双块内嵌，仅 ⑩ hunks 能产（码面静态核），
恢复前 HEAD 上属孤儿钉测。本提交＝字节级恢复（下述逐字节比对），孤儿钉测
复活。

## 码面（part01.js 两 hunk，与 b86912191 树内版本逐字节一致）

1. hunk1 `_zwCeParagraphMutate` arm1 sepTag 缺省：`hostTag` →
   `(hostTag === 'div') ? 'div' : 'p'`——li/dd host 拆分若以 hostTag 作分隔
   标签会产 li-in-li/dd-in-dd 错形；非 div host 缺省回落 'p'。
2. hunk2 arm1 入口门扩：`arm1IslandLid` 15 行注释＋变量（host li/dd、直子
   文本中点 caret、collapsed、eo < 长度）＋门 `(hostTag === 'div' ||
   arm1IslandLid)`——CE-island host 本体直子文本中点 insertParagraph 拆
   `<div>ab</div><div>cd</div>` 内嵌（语料 insertparagraph.js :1996/:2008
   Chrome 录形）；dt/h1 host 无 div 语义走既有退化行断臂。
3. 恢复验证：两个 hunk 以 regex 抽取与 b86912191 版本逐字节比对 True×2
   （首轮残留一处换行形态差，二次 Edit 修正后恒等）。
4. 调用面核：`_zwCeParagraphMutate` 仅 insertParagraph（15454）/
   insertLineBreak（15477）两入口进入；indent/outdent 不路由该函数——⑩
   对 indent 语料零触达（下归因腿实证印证）。

## A/B 账

**⑩ 纯净对**（base＝/tmp/d56-head-runner＝M2+d55−⑩≡HEAD a763e74b1 原生
【stash roundtrip release 构建】，wip＝/tmp/d56-wip0-runner＝M2+d55+⑩，两
二进制唯一源差＝两 hunk）：

| 腿 | base | wip | 共键 F2P | P2F | 备注 |
|---|---|---|---|---|---|
| insertparagraph.html（纯净对） | 7183P/12F | 7185P/10F | 2 | 0 | 共键 7195；F2P＝恰 ol>li(:1996)/dl>dd(:2008) 两 island 面。双侧 ×2 轮：base 侧另轮（d55-final-runner，/tmp/d56-ipar-base.json）7195 键状态逐键恒等＝M1→M2 对本文件零漂移；wip 第二轮（/tmp/d56-ip-wip-r2.json）7195 键状态逐键恒等＝7185P/10F 复现。head-1 轮被 guard 2400s 击杀无产物 |

**全量 battery**（base＝/tmp/d55g-er-sh\*-wip.json＝d55-final-runner
（M1+d55−⑩，内嵌 part01.js 与 HEAD 逐字节一致），wip＝/tmp/d56-er-sh\*-wip.json
＝M2+d55+⑩；该对含 M1→M2 并行流增量混入，逐项归因见下）：

| 腿 | 共键 | F2P | P2F | 备注 |
|---|---|---|---|---|
| sh1（delete 族） | 7933 | 0 | 0 | gone=150 截断带伪影不入账 |
| sh2 | 8531 | 0 | 0 | gone=16 同上 |
| sh3 | 13824 | 0 | 0 | new=13 同上 |
| sh4（insertparagraph+inserttext） | 10204 | 2 | 0 | F2P＝恰两 island 面（⑩ 目标） |
| sh5 | 8980 | 0 | 0 | |
| sh6（justify/indent） | 14849 | 1 | 0 | F2P＝indent 面，归并行流（下归因腿） |
| sh7 | 16608 | 0 | 0 | |
| sh8 | 14906 | 0 | 0 | |
| input-events 全套件 | 5834 | 0 | 0 | gone=728/new=9 截断伪影 |
| editing-other | 14837 | 0 | 0 | 双侧本轮对 |
| selection 全语料 | 53363 | 0 | 0 | 对 base=d55-selection-wip8 raw 共键口径（初稿 53,312 系沿用 d55 旧账已勘正）；gone=1/new=5 stringifier 族带伪影（d54/d55 同谱） |

ER 八分片合计共键 95,935，F2P=3/P2F=0。sh4 两 F2P 面：

- `[["defaultparagraphseparator","div"],["insertparagraph",""]]
  "<ol contenteditable=false><li contenteditable>ab[]cd</li></ol>"`（compare
  innerHTML）——期望 li 内 `<div>ab</div><div>cd</div>`（:1996）
- 同形 `<dl ...><dd contenteditable>ab[]cd</dd></dl>`（:2008）

**indent 归因腿**（`[["indent",""]] "<ul><li>a<br>{<br>}</li><li>b</li></ul>"
compare innerHTML`，battery sh6 唯一非 ⑩ 翻面）：

| 二进制 | 代际 | ×2 结果 |
|---|---|---|
| d55-final-runner | M1+d55−⑩ | Fail ×2 |
| d56-head-runner | M2+d55−⑩（＝HEAD 原生） | Pass ×2 |
| d56-wip0-runner | M2+d55+⑩ | Pass ×2 |

⇒ +1 翻面在 d55-final 与 HEAD 之间已经发生、与 ⑩ 恢复无关（⑩ 零 indent
触达的调用面核印证）——归 M1→M2 并行流增量（367faeca9..0c8dd867d：
style-system/typed-om Rust 重构等），记并行流名下不入本刀账。d55-final
内嵌 part01.js 与 HEAD 逐字节一致 ⇒ 翻面为 shim 外（Rust 侧）行为差。

## enginelib

d56 轮 4205P/0F，`test js_dom_bridge::tests::test_ip_island_li_dd_pins ...
ok`（/tmp/d56-enginelib.log）。d55g 轮（00:18，⑩ 剥离前树含 WIP hunks）
4205P/0F 同绿——各 enginelib 轮次均未在孤儿钉测态运行。

## 记账

- ⑩ 刀账 ＋2（island 两面），P2F=0；indent ＋1 归并行流增量。
- 残留挂账不动：①(b) 跨元素边界 caret join 243F、①(c) `{<hr>}` 核拒收、
  ④ B3 text-control 期望矩阵、⑤ DC-3 周跑快照对账、③ generation-aware 域
  （d49/d55 两负样本同域）。

## 运行产物

- /tmp/d56-er-sh{1..8}-wip.json（battery 八分片）、/tmp/d56-suite-wip.json、
  /tmp/d56-eo-{base,wip}.json、/tmp/d56-selection-wip.json
- /tmp/d56-indent-{base,wip}-{1,2}.json（sh6 复核 ×2 侧）、
  /tmp/d56-indent-head-{1,2}.json（HEAD-runner 归因腿）、
  /tmp/d56-ipar-{base,wip}.json（纯净对先轮：d55-final×wip0，7195 键 F2P=2/P2F=0 同账）、
  /tmp/d56-ip-head-2.json（⑩ 纯净对 base 侧 head-runner 轮；head-1 被 guard
  2400s 击杀无产物）、/tmp/d56-ip-wip-r2.json（⑩ 纯净对 wip 第二轮）
- 二进制：/tmp/d56-head-runner（stash roundtrip release HEAD）、
  /tmp/d56-wip0-runner（release 恢复树，01:32 构建）、/tmp/d55-final-runner
  （M1+d55 基线）；日志 /tmp/d56-enginelib.log、/tmp/d56-headbuild.log
- 脚本：/tmp/d56-battery.sh、/tmp/d56-er-shard.sh、/tmp/d56-headbuild.sh、
  /tmp/d56-head-indent.sh、/tmp/d56-head-ip.sh
