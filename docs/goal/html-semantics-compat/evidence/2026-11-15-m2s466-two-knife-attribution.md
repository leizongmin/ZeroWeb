# M2 s466 — 他流双刀入树归因轮：ZeroWeb-3 编辑流 B1 切片 3（069c6aed6 fix(engine) shim part01/part06 + bridge）+ 编辑语料接入（ef429d0e9）html-semantics 面零位移清偿（P 集恒等 8306 十三轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s465 下一步「守成巡查 + R5630 预告的
M3-B/C 落地对账」。本轮开工 `git pull --rebase` 携入 ZeroWeb-3 编辑
流两笔源码刀（在 s465 push 前的 pull 中落树，位于本流 s465 docs
提交之下）：①`069c6aed6` fix(engine) B1 切片 3 键入绕链接边界臂与
unlink 实应用——触共享面 engine 本体：`js_dom_bridge.rs` +170、
`js_dom_bridge/callbacks.rs` +21、`js_dom_bridge/script_gen.rs` +12、
`js_dom_shim/part01.js` +242、`js_dom_shim/part06.js` +413（含
`docs/goal/wptfyi-editing/` 账本与 probe 工件）；②`ef429d0e9`
test(wpt-runner) editing whitespaces/plaintext-only/event 语料接入
selection 通道——触 `tests/wpt-runner/src/testharness.rs` +17 与
`tests/wpt-data/editing/run/` 语料。归因窗口源码变更仅此两刀 → 执
行他流刀归因轮（半径申报在 editing/link 边界面，html-semantics 面
预期零位移，join 实测为准）。

## 双腿 join（0dd607f21 树 = 双刀组合态，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/200F/6T/1U）**，leg2 **8516 键
  （8306P/203F/6T/1U）**；共享键 disagree = 0。
- 单腿缺席 5 键全在 select-home-end 族主文件：整文件键 leg1-only
  且**本轮以 Fail 形态出现**（文件级聚合完成形态，既往各轮出现均
  为 Timeout 形态——族粒度相位谱新增一位形，观察备注在案）+
  4 子测 Fail 键 leg2-only；detailed 文件级 Timeout 键双腿一致共
  享在册。
- **stable 8512（8306P/199F/6T/1U，norm 配方口径，distinct
  8510）**：族内仅 detailed T 入 stable。
- **vs s465 stable（刀前树，distinct 8514）恰 4 GONE（主文件 4 子
  测键）、零 NEW**——差集全数族内；共享 stable 键零状态翻转，非
  族键逐键零差。
- **归因定谳：双刀 html-semantics 面零位移**（与 g47/g49/g50/b7ecf7864
  渲染流刀及 s453 双刀结论同型）；P 集账本口径 **8306 十三轮连续
  恒等**（刀前刀后同值，编辑语料接入未向 html-semantics sweep 键
  集增长任何键——双腿总量仍处 8513~8516 观察带内）。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第十二轮）；
  红池 onblur 2 F 键照常在册。
- stable T 键集 = 5 非族键 + detailed T，非族 5 键与 s461-s465
  stable 同集。

## 全桥套件绿窗收口跑（刀后树旁证）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 15.60s；基线 s465 1643P，+0，filtered
2389 同值——刀直改 js_dom_bridge.rs 与 shim part01/part06 本体，
钉面零破零新增，为刀面无回归旁证）。

## 观察备注（非哨升级）

族主文件整文件键本轮首次以 Fail（而非 Timeout）形态出现：与该文
件子测粒度完成性（4 子测均可完成并 Fail）相符的聚合相位，单次观
察不入哨；若后续连续多轮固化为 Fail 形态或出现族外键形变，再升观
察哨。该键不在 stable 在册集内，不影响 P 集与位移判定。

## 结论

s466 收口（他流双刀归因轮，本流零源码改动）：ZeroWeb-3 编辑流
069c6aed6 + ef429d0e9 双刀 html-semantics 面零位移清偿——刀前后
stable 差集全数限于 select-home-end 族相位（4 子测出 stable +
detailed T 双腿一致保持），非族键逐键零差，P 集口径 8306 十三轮连
续维持，全桥绿窗钉面零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang 观察
项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s466/{s466-leg1,s466-leg2}.log、s466-{l1,l2}.norm、
s466-stable.norm、s466-stable.keys、s466-bridge.log、
s466-bridge.rc。
