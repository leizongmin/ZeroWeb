# M2 s449 — 守成巡查轻覆盖轮：他流入树仅 docs（零刀）基带复测（P 集恒等 8305 六轮连续，select-home-end 族双文件整 T 相位）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s448 下一步「守成巡查（他流入树归因轮或轻覆盖）」。
本轮开工 `git pull --rebase` 入他流 fb28e80d8（rendering 流 wptfyi-css
DC-3 证据补强）+ merge 919cbf953，diffstat 仅
`docs/goal/rendering-compat/evidence/2026-11-15-dc3-snapshot-refresh.md`
一文件（+60/−18）——**零源码零语料，代码树与 s448 双腿树（75a5f4451 =
af012f775）全同** → 无归因对象，按预案执行纯轻覆盖轮（零源码零语料净
变更）。

## 双腿 join（919cbf953 树，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/199F/7T/1U）**，leg2 **8516 键（8306P/204F/
  5T/1U）**；unstable 逐腿集 7 键全在 select-home-end 族，且本轮出现
  双腿间整文件相位对撞的新形态：**leg1 主文件 + detailed 文件同时整
  文件 Timeout（双文件级 T）vs leg2 主文件 4 子测 Fail + detailed 文件
  级 Fail**——仍为族内两形态（整文件 T ↔ 子测/文件级 F）的组合相位，
  与 s446/s448 记载的族内振荡同型，非真位移。
- **stable 8511（8306P/199F/5T/1U，norm 配方口径）**；族内键全数因双
  腿相位对撞出 stable（族 stable 面 = 空）。
- **vs s448 stable（8512 = 8306P/200F/5T/1U）恰 1 差集，全部 select-
  home-end 族**：s448 形态 {detailed 文件级 F 双腿一致入 stable} →
  本轮 detailed 文件级相位入双腿振荡 → 该键出 stable；主文件相位维持
  双腿间振荡不入 stable（与 s448 同）。零 GONE / 零 NEW 键，非族键全
  数恒等。
- **P 集 8306（账本口径 8305）逐键恒等，s444→s449 六轮连续**。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 15.16s；基线 s448 1643P，
+0——零入树刀，无新钉预期）。

## 结论

s449 收口（守成巡查轻覆盖轮，零源码改动）：他流入树仅 rendering 流
docs 证据，同码树基带复现确认——**基带 8305P/203F/6T/1U → 本轮 leg
形态 8305~8306P/199~204F/5~7T/1U，全部变动限于 select-home-end 族相
位（含双文件级 T 对撞新形态）**，P 集恒等口径维持。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，不重复通
知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s449/{s449-leg1,s449-leg2}.log、s449-{l1,l2}.norm、
s449-stable.norm、s449-vs-s448.diff、s449-bridge.log、s449-bridge.rc。
