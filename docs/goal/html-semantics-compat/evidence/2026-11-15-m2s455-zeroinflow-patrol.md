# M2 s455 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（P 集恒等 8306 两轮连续，族主文件 4 子测 F 相位入 stable）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s454 下一步「守成巡查（他流入树归因轮或轻
覆盖）+ wptfyi-css WAAPI 门刀落树对账预告」。本轮开工 `git pull
--rebase` 远端零新刀（HEAD = cc8499531，即 s454 收口 docs 提交）；
自 s453 双腿树（1943457c5）至本树四提交全 docs-only（git diff
crates/ tests/ Makefile/ scripts/ 零触碰）→ 无归因对象，按预案执行
纯轻覆盖轮（零源码零语料净变更，同码树基带复测）。

## 双腿 join（cc8499531 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/204F/5T/1U）**，leg2 **8516 键（8306P/203F/
  6T/1U）**——双腿总量一致。
- **键集恒等（l1-only = 0，l2-only = 0）**；共享键状态分歧
  disagree = 1：select-home-end detailed 文件级键 F↔T 相位对冲出
  stable。
- **stable 8515（8306P/203F/5T/1U，norm 配方口径，distinct 8513）**。
- **vs s454 stable（distinct 8513）恰 2 GONE + 4 NEW，全数在
  select-home-end 族**：主文件文件级 Timeout 键两腿均未发出（出
  stable）→ 主文件 4 子测 Fail（End/Home/PageDown/PageUp）双腿一致
  入 stable；detailed 文件级 Fail 因腿间相位对冲出 stable。族相位轮
  换记账口径不变，**非族键逐键零差，零真位移**。
- **P 集账本口径 8306 两轮连续恒等**（s454 回归后保持）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 17.59s；基线 s454
1643P，+0——零入树刀，无新钉预期）。

## 结论

s455 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/203~204F/5~6T/1U，全部变动限于
select-home-end 族相位**（主文件子测粒度 ↔ 文件级粒度两形态再轮
换），P 集口径 8306 两轮连续维持，onblur 哨键本轮双腿一致 Pass 在
stable（哨解除后首轮回位确认）。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang 观察项
静默哨维持（本轮双腿零 hang；grep 命中均为测试名 "change"/"load
events" 字样假阳性）。

工件：/tmp/s455/{s455-leg1,s455-leg2}.log、s455-{l1,l2}.norm、
s455-stable.norm、s455-vs-s454.diff、s455-bridge.log、s455-bridge.rc。
