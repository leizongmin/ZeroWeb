# M2 s458 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（双腿完全一致 + stable 与 s457 逐键全同，P 集恒等 8306 五轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s457 下一步「守成巡查（他流入树归因轮或轻
覆盖）+ wptfyi-css WAAPI 门刀落树对账预告」。本轮开工 `git pull
--rebase` 远端零新刀（HEAD = b5903fa8f，即 s457 收口 docs 提交，
代码树与 s457 双腿树全同；期间渲染流 R5606/R5607 均 docs-only）→
无归因对象，按预案执行纯轻覆盖轮（零源码零语料净变更，同码树基带
复测）。

## 双腿 join（b5903fa8f 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键（8306P/203F/
  6T/1U）**——**双腿完全一致形态**（disagree = 0、单腿缺席 = 0），
  与 s457 同型连续第二轮。
- **stable 8516（8306P/203F/6T/1U，distinct 8514）**。
- **vs s457 stable 零 GONE、零 NEW——stable 逐键全同**（distinct
  8514 恒等）：族相位形态稳定（detailed 文件级 T + 主文件 4 子测 F
  双轮一致），非族键逐键零差。
- **P 集账本口径 8306 五轮连续恒等**（s454 回归后保持）。
- onblur 哨键组 7 键全在 stable（哨解除后第四轮 Pass 在册）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 15.69s；基线 s457
1643P，+0——零入树刀，无新钉预期）。

## 结论

s458 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——本轮双腿完全一致且 **stable 与 s457 逐键全同**（基带形态
8306P/203F/6T/1U 含族相位全稳），P 集口径 8306 五轮连续维持。挂用
户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提
示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s458/{s458-leg1,s458-leg2}.log、s458-{l1,l2}.norm、
s458-stable.norm、s458-vs-s457.diff、s458-bridge.log、s458-bridge.rc。
