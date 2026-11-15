# M2 s459 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（族 5 键相位振荡出 stable，stable 回归 s456 同型形态，P 集恒等 8306 六轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s458 下一步「守成巡查（他流入树归因轮或轻
覆盖）+ wptfyi-css WAAPI 门刀落树对账预告」。本轮开工 `git pull
--rebase` 远端零新刀（HEAD = a7722cf21，即 s458 收口 docs 提交，代
码树与 s458 双腿树全同；期间渲染流 R5608 ×2 与 website 周报均
docs-only）→ 无归因对象，按预案执行纯轻覆盖轮（零源码零语料净变更，
同码树基带复测）。

## 双腿 join（a7722cf21 树，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/199F/7T/1U）**，leg2 **8516 键（8306P/204F/
  5T/1U）**。
- 族相位交错形态：leg1 detailed 文件级 T + 主文件整文件 T（双文件
  整文件粒度，T=7）；leg2 detailed 文件级 F + 主文件 4 子测 F（子测
  粒度，F=204）。disagree = 1（detailed 文件级 T↔F 对冲）+ 单腿缺
  席 5 键（主文件整文件 T 键 leg1-only + 4 子测 leg2-only）。
- **stable 8511（8306P/199F/5T/1U，norm 配方口径，distinct 8509）**
  ——族内零键，形态与 s456 同型（该轮族亦全振荡出 stable）。
- **vs s458 stable（distinct 8514）恰 5 GONE（select-home-end 族
  5 键：detailed 文件级 T + 主文件 4 子测 F，本轮相位振荡出
  stable）、零 NEW**，非族键逐键零差，零真位移。
- **P 集账本口径 8306 六轮连续恒等**。
- onblur 哨键双腿一致 Pass 在 stable（哨解除后第五轮 Pass 在册）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 18.11s；基线 s458
1643P，+0——零入树刀，无新钉预期）。

## 结论

s459 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/199~204F/5~7T/1U，全部变动限于
select-home-end 族相位**（族 5 键本轮全振荡出 stable，stable 回归
s456 同型 8306P/199F/5T/1U 形态），P 集口径 8306 六轮连续维持。挂
用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s459/{s459-leg1,s459-leg2}.log、s459-{l1,l2}.norm、
s459-stable.norm、s459-vs-s458.diff、s459-bridge.log、s459-bridge.rc。
