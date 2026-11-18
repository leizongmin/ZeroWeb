# M2 s451 — 守成巡查轻覆盖轮：零入树刀基带复测（P 集恒等 8305 八轮连续，双腿全一致 unstable=0，stable 形态回归基带）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s450 下一步「守成巡查（他流入树归因轮或轻覆
盖）+ 盯 wptfyi-css target expando 刀是否落树」。本轮开工
`git pull --rebase` 远端零新刀（HEAD = 6a49b5648，即 s450 收口 docs
提交，代码树与 s450 双腿树全同），target expando 刀未落树（彼流仍
docs 备刀态）→ 无归因对象，按预案执行纯轻覆盖轮（零源码零语料净变
更）。

## 双腿 join（6a49b5648 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键（8306P/203F/
  6T/1U）**——**双腿逐键全一致，unstable = 0**（本轮 select-home-end
  族相位双腿一致：主文件 4 子测 Fail + detailed 文件级 T，与 s450
  leg1 形态同型）。
- **stable 8516（8306P/203F/6T/1U，norm 配方口径）**——stable 形态
  与基带口径（8305P/203F/6T/1U 账本 norm 推导）完全同型。
- **vs s450 stable（8510）恰 4 差集，全部 select-home-end 族**：主文
  件 4 子测 Fail 相位本轮双腿一致入 stable（s450 该 4 键因主文件整 T
  相位双腿振荡出 stable）；detailed 文件级 T 维持双腿一致入 stable。
  族相位轮换记账口径不变，**非族键 comm 逐键零差**，零 GONE/零 NEW。
- **P 集 8306（账本口径 8305）逐键恒等，s444→s451 八轮连续**。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 18.50s；基线 s450 1643P，
+0——零入树刀，无新钉预期）。

## 结论

s451 收口（守成巡查轻覆盖轮，零源码改动）：零入树刀同码树基带复现
确认——本轮双腿全一致且 stable 形态回归基带 **8305P/203F/6T/1U**，
全部变动仍限于 select-home-end 族相位，P 集恒等口径维持。target
expando 刀落树对账预告维持（落树后按他流刀归因轮全量双腿对账，
337 页半径全在 css 组，html-semantics 面预期零位移，join 实测为准）。
挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s451/{s451-leg1,s451-leg2}.log、s451-{l1,l2}.norm、
s451-stable.norm、s451-vs-s450.diff、s451-bridge.log、s451-bridge.rc。
