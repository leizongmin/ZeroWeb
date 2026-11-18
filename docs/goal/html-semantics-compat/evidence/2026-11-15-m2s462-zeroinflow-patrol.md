# M2 s462 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（族相位再翻转：PageDown 子测出 stable，detailed 键腿间 F↔T 对冲延续，P 集恒等 8306 九轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s461 下一步「守成巡查」。本轮开工
`git pull --rebase` 携入渲染流 docs 提交（ee09fe5d7 R5622 /
970286045 R5623 三源空转核查，均 docs-only）→ 零源码零语料净变
更，HEAD = 0018b7a1c（代码树与 s461 双腿树 d37cb3a99 全同）→ 无
归因对象，按预案执行纯轻覆盖轮。

## 双腿 join（0018b7a1c 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键
  （8306P/203F/6T/1U）**——状态总量双腿完全一致。
- 单腿缺席 2 键全在 select-home-end 族主文件：整文件 Timeout 键
  leg1-only 回归 + PageDown 子测键 leg2-only；disagree = 1 =
  detailed 文件级键（leg1 Fail ↔ leg2 Timeout，与 s461 相位相
  反）。
- **stable 8514（8306P/202F/5T/1U，norm 配方口径，distinct
  8512）**：族主文件 Home/PageUp/End 3 子测 Fail 双腿一致在册，
  PageDown 子测出 stable，detailed 文件级键腿间 F↔T 对冲延续出
  stable，族整文件 Timeout 键 leg2 缺席不入。
- **vs s461 stable（distinct 8513）恰 1 GONE（PageDown 子测键）、
  零 NEW**；共享键零状态翻转，非族键逐键零差，零真位移。
- **P 集账本口径 8306 九轮连续恒等**。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第八轮）；
  红池 onblur 2 F 键照常在册。
- stable T 键集 5 键全非族，与 s461 stable 同集。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 16.57s；基线 s461 1643P，+0——零入树
刀，无新钉预期）。

## 结论

s462 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/202~203F/5~6T/1U，全部变动限于
select-home-end 族相位**（族键自振荡：4 子测中 PageDown 本轮单腿
化出 stable、余 3 子测双腿一致在册，detailed 文件级腿间相位再翻
转），P 集口径 8306 九轮连续维持。挂用户门维持：GB-20261114-VIDEO
+ GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang 观察
项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s462/{s462-leg1,s462-leg2}.log、s462-{l1,l2}.norm、
s462-stable.norm、s462-stable.keys、s462-bridge.log、
s462-bridge.rc。
