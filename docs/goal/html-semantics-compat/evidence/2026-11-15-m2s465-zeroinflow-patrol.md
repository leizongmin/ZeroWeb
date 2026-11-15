# M2 s465 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（双腿完全一致 disagree=0，族 5 键全形态在册 4 子测 F + detailed T 双双入 stable，P 集恒等 8306 十二轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s464 下一步「守成巡查」。本轮开工
`git pull --rebase` 携入渲染流 docs 提交（R5629/R5630 三源核查，
docs-only；R5630 预告 ZeroWeb-3 侧 M3-B/C 九文件整体转 staged 落
地窗口临近，落地义务预挂——落树后按他流刀归因轮，半径全在 css
组，html-semantics 面预期零位移）→ 当前零源码零语料净变更，HEAD
= 5de13b648（代码树与 s461-s464 双腿树全同）→ 本轮仍按预案执行
纯轻覆盖轮。

## 双腿 join（5de13b648 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键
  （8306P/203F/6T/1U）**——腿键集完全一致（leg-only 0/0），
  **disagree = 0**，状态逐键全同。
- select-home-end 族 5 键全形态在册且双腿一致：4 子测 Fail +
  detailed 文件级 Timeout；族整文件 Timeout 键双腿缺席。
- **stable 8516（8306P/203F/6T/1U，norm 配方口径，distinct
  8514）**——族 5 键全部入 stable（4 子测 F + detailed T），为
  观察期内族键最全在册位形。
- **vs s464 stable（distinct 8510）零 GONE、恰 4 NEW（主文件 4 子
  测键）**；共享键零状态翻转，非族键逐键零差，零真位移。
- **P 集账本口径 8306 十二轮连续恒等**。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第十一
  轮）；红池 onblur 2 F 键照常在册。
- stable T 键集 = 5 非族键 + detailed T，非族 5 键与 s461-s464
  stable 同集。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 15.61s；基线 s464 1643P，+0——零入树
刀，无新钉预期）。

## 结论

s465 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/203F/6T/1U 双腿完全一致**，族 5 键全形
态在册（detailed 键本轮 T 相位双腿一致，一改 s461-s464 的 T↔F 腿
间对冲），P 集口径 8306 十二轮连续维持。**下轮注意**：ZeroWeb-3
侧 M3-B/C 九文件落地窗口预挂，落树后按他流刀归因轮处理（css 组半
径，html-semantics 面预期零位移，join 实测为准）。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，不重
复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s465/{s465-leg1,s465-leg2}.log、s465-{l1,l2}.norm、
s465-stable.norm、s465-stable.keys、s465-bridge.log、
s465-bridge.rc。
