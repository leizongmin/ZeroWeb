# M2 s450 — 守成巡查轻覆盖轮：零入树刀基带复测（P 集恒等 8305 七轮连续，detailed 文件级 T 相位入 stable）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s449 下一步「守成巡查（他流入树归因轮或轻覆
盖）」。本轮开工 `git pull --rebase` 远端零新刀（HEAD = 1879d53a5，即
s449 收口 docs 提交，代码树与 s449 双腿树全同——自 6c21880ac 谱系至
本树全程 docs-only）→ 无归因对象，按预案执行纯轻覆盖轮（零源码零语
料净变更）。

## 双腿 join（1879d53a5 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8513 键（8306P/199F/
  7T/1U）**；unstable 逐腿集 5 键全在 select-home-end 族：leg1 主文件
  4 子测 Fail + detailed 文件级 T vs leg2 主文件整文件 T + detailed
  文件级 T——族内两形态（子测 F ↔ 整文件 T）再轮换，非真位移。
- **stable 8510（8306P/199F/6T/1U，norm 配方口径）**，族内仅 detailed
  文件级 T 双腿一致入 stable（主文件相位维持双腿间振荡）。
- **vs s449 stable（8511）恰 1 差集**：detailed 文件级 T 相位本轮入
  stable（s449 该键因双腿振荡出 stable）——select-home-end 族相位轮
  换记账口径不变。**非族键 comm 逐键零差**（键集恒等），零 GONE/零
  NEW。
- **P 集 8306（账本口径 8305）逐键恒等，s444→s450 七轮连续**。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 16.43s；基线 s449 1643P，
+0——零入树刀，无新钉预期）。

## 他流在途对账预告（watch item）

wptfyi-css 流 `target` expando 刀已定谳（cee144a8d，engine shim 元素
属性 set 陷阱对已知属性名 `target` 的对象值走 setAttribute 串化单根
因，爆炸半径 337 页/56,365 上游子测全在 css 组插值 harness），其证据
档 §4 刀规格标注「下轮执行」归彼流施工。**落树后本面按他流刀归因轮
全量双腿对账**：修面仅改对象值 expando 存储（原始值 setAttribute 路
径不变），html-semantics 面预期零位移，join 实测为准。

## 结论

s450 收口（守成巡查轻覆盖轮，零源码改动）：零入树刀同码树基带复现
确认——基带 **8305P/203F/6T/1U → 本轮 leg 形态 8305~8306P/199~203F/
6~7T/1U，全部变动限于 select-home-end 族相位**，P 集恒等口径维持。
挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s450/{s450-leg1,s450-leg2}.log、s450-{l1,l2}.norm、
s450-stable.norm、s450-vs-s449.diff、s450-bridge.log、s450-bridge.rc。
