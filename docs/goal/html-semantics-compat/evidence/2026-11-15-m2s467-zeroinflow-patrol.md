# M2 s467 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（detailed T 键振荡出 stable + 4 子测回入，P 集恒等 8306 十四轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s466 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = 10f81b307 即 s466 收口
docs 提交；代码树与 s466 双刀归因轮双腿树 0dd607f21 全同，含
ZeroWeb-3 编辑流 069c6aed6 + ef429d0e9 双刀已清偿组合态）→ 无归
因对象，按预案执行纯轻覆盖轮。

## 双腿 join（10f81b307 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/204F/5T/1U）**，leg2 **8516 键
  （8306P/203F/6T/1U）**；腿键集完全一致（leg-only 0/0）。
- disagree = 1：detailed 文件级整文件键（leg1 Fail ↔ leg2
  Timeout）；族主文件 4 子测 Fail 双腿一致；族整文件键本轮双腿缺
  席——s466 首现的整文件键 Fail 形态未复现，单次观察判定维持（不
  固化、不入哨）。
- **stable 8515（8306P/203F/5T/1U，norm 配方口径，distinct
  8513）**。
- **vs s466 stable（distinct 8510）恰 1 GONE（detailed T 键振荡
  出）、4 NEW（主文件 4 子测键回入）**，全数族内；共享键零状态翻
  转，非族键逐键零差，零真位移。
- **P 集账本口径 8306 十四轮连续恒等**。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第十三轮）；
  红池 onblur 2 F 键照常在册。
- stable T 键集 5 键全非族，与 s461-s466 stable 同集。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 24.38s；基线 s466 1643P，+0，filtered
2389 同值——零入树刀，无新钉预期）。

## 结论

s467 收口（守成巡查轻覆盖轮，零源码改动）：刀后组合态基带复现确
认——基带形态 **8306P/203~204F/5~6T/1U，全部变动限于
select-home-end 族相位**（detailed 键 T↔F 腿间对冲延续、4 子测双
腿一致在册），P 集口径 8306 十四轮连续维持，编辑流双刀组合态下基
带无漂移。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+
s137（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿
零 hang）。

工件：/tmp/s467/{s467-leg1,s467-leg2}.log、s467-{l1,l2}.norm、
s467-stable.norm、s467-stable.keys、s467-bridge.log、
s467-bridge.rc。
