# M2 s438 — rebase 组合树例常位移检查（零代码变更轮）：s437 修复零位移维持、渲染流新刀零波及、dialog 红相持续待复验

日期：2026-11-14。前情：s437 收口（evidence/2026-11-14-m2s437-strut-arrival-
regression-attribution-fix.md）push 前 pull --rebase 到远端 main（窗口
88edca67f..65f88ed13，10 刀：渲染流 s424a/s424b 实验记账、**b0e5ac2da
anchor-size() used 值布局期求值**（style-system apply/computed +257、
layout-engine anchor_pos +256）、**9d587f932 ip C4 刀**（块边界拆分块级
兄弟止步 + bare-br 行界双空块物化）、R5564/R5565 记账、site-opt PR #303
merge）。两刀均触 engine 共享面 → 本轮在组合树上复跑例常位移检查，同窗
复验 s437 遗留 dialog phase-flip。

## 执行

rebase 后组合树（HEAD=a80d3884b）双跑 `make testharness-html-semantics`
（无 FILTER，默认 TIME_LIMIT，release runner 增量重建 2m10s；宿主
loadavg 受 zeroseed D 态风暴拖累——1min 12.5 但 15min 峰值 114.68，R5565
在册同型）。join vs 双基线（s433 终稿 + s437 fix）：

| 基线 | stable-P | stable-F | stable-T | s438 |
|---|---|---|---|---|
| s433 终稿 | 8309 | 202 | 3 | — |
| s437 fix | 8308 | 203 | 2 | — |
| **s438（本轮）** | — | — | — | **8308 / 199 / 2** |

**vs s437 fix（直接前驱）：P/F/T new=gone=0**——s437 修复在组合树零位移，
渲染流 anchor-size + ip C4 对 html/semantics testharness 语料稳定集零波及
（anchor-size 语料面在 css-anchor-position，ip C4 块边界 14 键在 reftest
corpus，两 corpus 键集与本语料不交叠，实测证实）。

## 差异逐项归因（vs s433，全部非代码）

- **dialog P→F（1 键）**：`focus-after-close.html :: Focus should be
  moved to the previously focused element (Complex dialog usage)`——s437
  已分诊 phase-flip（14:29 同树绿窗 vs 今日全红含父提交 solo×3）。本轮
  双腿 F 与 fix437 双腿 F 一致 ⇒ **红相自 ~20:00 持续 ≥2h 未翻回**。
  重载风暴时序假设在案（14:29 绿窗早于 zeroseed 风暴）；非代码定性维持
  （父提交 solo×3 红排除本刀因果），待下轮位移检查复验翻回。
- **customizable-select 4 键 F「gone」非真改善**：
  `select-home-end-pagedown-pageup.optional.html` 4 子测试在 leg1 F、
  leg2 **文件级 Timeout**（`loaded:false pending:4`——页面未完成 load，
  重载时序形态），子测试态被文件级 T 置换 ⇒ 双腿不齐退出 stable-F。
  键面真态 = 红池在册（customizable-select 面），本轮无变化。
- **detailed.optional F→T**（沾染 1）：同文件级 load 时序族，同上。
- **149.html T gone（1）**：`for='  WINdow` 提取工件，在册振荡。

## 结论

s438 收口（零代码变更轮）：组合树位移检查证 s437 修复与渲染流新刀在
本语料互不干扰，键面回到 s437 fix 等值带（stable 8308/199→203 差集全数
归因文件级 load 时序族与在册工件）。dialog 红相持续待复验（挂账维持）。
跨流衔接：R5565 已预告其 R5566 对 s437 落地面核共享面——本轮组合树
位移数据可直接引用（vs fix 集合差 0/0/0）。

工件：/tmp/s438/{sweep-1,sweep-2}.log、sweep-{1,2}.rc（rc=2，键数
8517/8514 与 8515 基线带同带）、join.sh、s438.{1,2}.ks、s438.s{P,F,T}、
vs433/vs437 差集、s438.taint。
