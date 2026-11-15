# M2 s464 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（族 4 子测出 stable + detailed T 键入，stable 回归 s459 同型 199F 形态，P 集恒等 8306 十一轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s463 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = 7bf978a84 即 s463 收口
docs 提交；期间渲染流 R5627/R5628 均 docs-only）→ 零源码零语料净
变更，代码树与 s461-s463 双腿树全同 → 无归因对象，按预案执行纯轻
覆盖轮。

## 双腿 join（7bf978a84 树，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/199F/7T/1U）**，leg2 **8516 键
  （8306P/203F/6T/1U）**；共享键 disagree = 0。
- 单腿缺席 5 键全在 select-home-end 族主文件：整文件 Timeout 键
  leg1-only（leg1 主文件整文件粒度挂起，族 2 个 T 键 = 整文件 +
  detailed 双双 Timeout）+ 4 子测 Fail 键 leg2-only（leg2 detailed
  T + 4 子测粒度 F=203）。
- **stable 8512（8306P/199F/6T/1U，norm 配方口径，distinct
  8510）**：detailed 文件级 T 键双腿一致入 stable，4 子测键全部
  不入（非共享键）。
- **vs s463 stable（distinct 8513）恰 4 GONE（主文件 4 子测键）+
  1 NEW（detailed T 键），全数族内**；共享键零状态翻转，非族键逐
  键零差，零真位移。
- **P 集账本口径 8306 十一轮连续恒等**；stable 形态回归 s459 同型
  199F 族近零键位形（差异仅 detailed T 在册）。
- onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后第十轮）；
  红池 onblur 2 F 键照常在册。
- stable T 键集 = 5 非族键 + detailed T，非族 5 键与 s461-s463
  stable 同集。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 15.68s；基线 s463 1643P，+0——零入树
刀，无新钉预期）。

## 结论

s464 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/199~203F/6~7T/1U，全部变动限于
select-home-end 族相位**（主文件整文件粒度挂起轮：4 子测出
stable、detailed T 双腿一致入），P 集口径 8306 十一轮连续维持。挂
用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang，
族整文件 Timeout 为该文件既有粒度相位形态非 hang 哨对象）。

工件：/tmp/s464/{s464-leg1,s464-leg2}.log、s464-{l1,l2}.norm、
s464-stable.norm、s464-stable.keys、s464-bridge.log、
s464-bridge.rc。
