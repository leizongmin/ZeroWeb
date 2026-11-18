# M2 s463 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（PageDown 子测回入 stable，detailed 键 T↔F 对冲延续，P 集恒等 8306 十轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s462 下一步「守成巡查」。本轮开工
`git pull --rebase` 携入渲染流 docs 提交（f2be8a1da R5625 /
f155a8621 R5626 三源空转核查，均 docs-only）→ 零源码零语料净变
更，HEAD = 5d10fbf49（代码树与 s461/s462 双腿树全同）→ 无归因对
象，按预案执行纯轻覆盖轮。

## 双腿 join（5d10fbf49 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键
  （8306P/204F/5T/1U）**；腿键集完全一致（leg-only 0/0）。
- disagree = 1：detailed 文件级整文件键（leg1 Timeout ↔ leg2
  Fail，与 s461 同相位）；族主文件 4 子测 Fail 双腿一致（本轮含
  PageDown 回归双腿化），族整文件 Timeout 键双腿缺席。
- **stable 8515（8306P/203F/5T/1U，norm 配方口径，distinct
  8513）**。
- **vs s462 stable（distinct 8512）恰 1 NEW（PageDown 子测键回
  入）、零 GONE**；共享键零状态翻转，非族键逐键零差，零真位移。
- **P 集账本口径 8306 十轮连续恒等**。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第九轮）；
  红池 onblur 2 F 键照常在册。
- stable T 键集 5 键全非族，与 s461/s462 stable 同集。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 15.79s；基线 s462 1643P，+0——零入树
刀，无新钉预期）。

## 结论

s463 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/203~204F/5~6T/1U，全部变动限于
select-home-end 族相位**（PageDown 子测单腿化一轮后回入双腿一致，
detailed 文件级 T↔F 腿间对冲为族内唯一持续振荡点），P 集口径 8306
十轮连续维持。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇
D）+ s137（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮
双腿零 hang）。

工件：/tmp/s463/{s463-leg1,s463-leg2}.log、s463-{l1,l2}.norm、
s463-stable.norm、s463-stable.keys、s463-bridge.log、
s463-bridge.rc。
