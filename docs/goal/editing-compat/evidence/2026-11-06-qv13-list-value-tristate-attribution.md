# 2026-11-06 qv13 轮——list 命令 qV "false"/"true" 判别子归因（录音两源混合）+ tri-state 录音面收口

eb1 轮挂账卡点①：qV before/after 录 `"false"`/`"true"` 的 13 案判别子未知
（非空壳/swCSS 双否证已排除）。

## ① 归因：语料 qV 面两录音源混合（bulk 非 Chrome 录音）

13 案全量定位（iol 8 + iul 5，语料级 grep 元组扫描）：

- iol：`<div><span style='font-family: times; color: blue'>[bar]</span></div>`、
  `<div><span…>bar</span></div><span style='font-family: italic; color: green'>[baz]</span>`、
  `<div><ol style='…'><li>bar</li></ol><span…>[baz]</span></div>`（span 三例）、
  `{}`、`<div>{}</div>`、`<div>{}<br></div>`、`<p>{}</p>`、`<p>{}<br></p>`（空
  context 五例）。
- iul：`{}`、`<div>{}</div>`、`<div>{}<br></div>`、`<p>{}</p>`、`<p>{}<br></p>`
  （空 context 五例）。

**双否证后第三个否证：纯状态读面否证**。bulk `foo[]bar`（qVb="" qVa=""，after
qState=true）与 `{}`（qVb="false" qVa="true"，after qState=true）——after 面
同树同选区形态（塌缩 caret 在新建列表 li 内）而 qV 不同 → 不存在 (tree,
selection) 纯函数同时满足两population。与 ⑳ P10 before/after 悖论同构，但本
面矛盾在**跨案**而非同案 before/after。

**真 Chrome 实测（chromium_headless_shell-1243，/tmp/zw-qv13/probe.html）**：
Blink 对 `queryCommandValue('insertOrderedList')` **恒返 tri-state 字符串**
（"true"/"false"，含 bulk 形 `foo[]bar` after 录 "true" ≠ 语料 ""）→ **语料
bulk ~301 案 qV="" 不是 Chrome 录音**，为 spec 生成值（Aryeh implementation.js
口径）；13 案为 Chromium 后录 tri-state 实测值。同一实测还显示真 Chrome 对
`<div>{}</div>` 的 insertUnorderedList exec 返回 false（语料期望 true）——
录音会话 Chrome ≠ 今日 Chrome，语料 pin 快照即契约的既有口径再次成立。

判别面（语料级全量对账，脚本审计 iol 149 + iul 165 + multitest 619 案零错配）：
before 双谓词 = (a) 端点祖先链有**带 style 属性的 span**（span 三例 setup 即
唯一命中；:590 族 styled span 是 exec 产物、setup 无 span）；(b) 塌缩 caret
最近块容器无可见内容（{} 族；`<div>[abc</div>…<div>def]</div>` 非塌缩不命中）。
after 面 DOM 纯读不可判别（span 三例 after 与 iol :590 族 after 同为 styled
span in li；{} 族 after 与 bulk after 同为 in-list caret）→ exec 时点信息落
壳标：toggle 入口谓词命中 → 出口对 caret 所在列表壳打 `_zwListQvTriState` 标，
读面按端点最近列表祖先查标。

## ② 实现（part01e.js + part06.js）

- part01e：`_zwListQvBlockContainer` / `_zwListQvContentless` / `_zwListQvPreFires`
  （before 双谓词）+ `__zw_ce_list_value(cmd)`（命中 → String(qState)，未命中 →
  "" spec 口径）+ `_zwListToggle` 入口 `qvPre` 快照与 enable 出口壳标。
- part06：`queryCommandValue` 两命令分派臂（qState 臂同型）。

## ③ 门禁（qv13 批）

- 探针 zz-probe-qv13（26 形状 = 13 案 + 13 控制）：**首跑 23/26**——:590 族
  swcss 换型三例 3 红，实证读面谓词重入缺陷（exec 产 styled span in li 被
  (a) 误命中翻 "true"，与 span 三例 after 同形纯读不可判别）；**读面修正为
  三分**（壳标优先 + 谓词带 no-list 守卫）后二跑 **26/26 OK**（rc=0）。
- iol/iul 全腿（judge vs eb1 base，judge-qv13.py）：iol **1725P/36F → 1741P/20F**
  （F2P=16）、iul **2032P/33F → 2041P/24F**（F2P=9），**F2P=25 / P2F=0 / newF=0，
  VERDICT OK**。
- 预期 -10 实得 -9 的差一行 = iul `<div>{}<br></div>` qV-after（?2001-last）：
  该 case 的 exec 断言落 ?1001-2000、query 断言落 ?2001-last（跨窗 straddle，
  后窗只重做 setup 不执行 exec，after 读未修改树必红）——eb1 ③ 已档同族伪影
  （div-br qSa/qVa 同窗），本行同族除名不追。

- multitest 全文件双腿基线对照（batch-mt.sh 三段式：Stage A stash qv13 干净重建 marker 0 命中 → mtbase 19,254P/827F；Stage B 恢复重建 marker 3 命中 → mtpost，judge-qv13-mt.py 双 key 元组）：**mt rows 10092→10092 / P 9676→9676 / F 416→416，F2P=0 / P2F=0 / newF=0 / lost=0，VERDICT OK**——multitest 语料 qV 面全为 spec 生成值口径，谓词零外溢即设计行为（§① multitest 619 案审计零错配的行为面复核）。shell Pass/Fail 19,254/827→19,320/835 的 +8 Fail 经全文键按窗复核：8 行全落 ？9001-last 截断边界页（judge 按断言文本去重不可见的发射深度差——同文本在 multitest-ab 恒 F；同窗+同全文键 P→F = 0，Timeout 1→1），⑪ 发射深度伪影族非行为回归。

## ④ 残债更新

- **eb1 ① value-face 13 案 qV "false"/"true" 判别子 → 本轮收口**：iol/iul 元组判决（本轮 judge 对 10:03 基线双腿手核复核与 §③ 逐位一致）——iol F2P 16 / iul F2P 9 共 **25 行全部为 queryCommandValue 断言行**，P2F 0 / newF 0 / lost 0；13 案 ×26 qV 断言行中 25 翻绿，唯一未翻 = iul `<div>{}<br></div>` qV-after（?2001-last 跨窗 straddle 伪影，§③ 已档除名）。
- **eb1 ② iul divbr qState-after 读面红 → 定谳同窗 straddle 伪影**：`<div>{}<br></div>` 的 qSa/qVa after 断言同落 ?2001-last（exec 断言在 ?1001-2000），后窗只重做 setup 不执行 exec，after 读未修改树必红——qSa/qVa 双行同形，与 eb1 ③ div-br 同族，除名不追。
- **eb1 ③ :478 wholesale 同型嵌套 ×4 P10 定谳 → 维持不追**：本轮残差中该族（nested qState-after）行集无扩大。
- **iol 残差 20F 构成**：?1-1000 br 边界 compare ×3；?1001-last ×17 = dps qV-before ×1 + dps compare ×6 + rtl ×3 + styled-ol+span compare ×1 + nested qState-after ×3（③ 族）+ blockquote ×2 + noneditable-span ×1。
- **iul 残差 24F 构成**：?1-1000 ×5（nested qState-after ×2 ③ 族 + br compare ×3）；?1001-2000 ×10（dps compare ×6 + blockquote ×2 + nested qI/qS-after ×2 ③ 族）；?2001-last ×9（divbr qSa/qVa straddle ×2 + rtl ×3 + div 边界 ×3 + noneditable-span ×1）。
- **enginelib 门禁收口（batch Stage C，mt judge OK 后自动接力，`make guarded-test`）：engine lib 3,766P/5F 五红恒一**（p56_adopted_snapshot / p99s_case / r3276 / r2810 / r2809 逐名 identical，零 spawn 问题，clippy 腿同批 Finished 无告警）。首跑 10:27 曾因并发 make 入口的 target-disk-guard 三级全清 target/ 连坐作废（media-host CARGO_BIN_EXE spawn NotFound、cargo fail-fast 早退，engine lib 未跑到），冷 target 重跑收口，复盘见 [learning](../../../learnings/platform/2026-11/2026-11-06-concurrent-make-disk-guard-wipes-sibling-gate.md)。
