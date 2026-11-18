# M2 s452 — 守成巡查轻覆盖轮：他流入树仅 docs 零刀基带复测（P 集恒等 8305 九轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s451 下一步「守成巡查 + 盯 target expando 刀
是否落树」。本轮开工 `git pull --rebase` 入他流 a2d86c22e（rendering
流 R5601 三源空转核查，diffstat 仅 docs/goal/rendering-compat.md 一
文件 +7 行）——**零源码零语料，代码树与 s451 双腿树（b7d5b14b9）全
同**；target expando 刀仍未落树（彼流记录仍为「§4 下轮执行」预告态）
→ 无归因对象，按预案执行纯轻覆盖轮（零源码零语料净变更）。

## 双腿 join（a2d86c22e 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8516 键（8306P/203F/
  6T/1U）**——双腿总量一致但族相位有腿间差异：disagree 集 4 键全在
  select-home-end 族（2 Fail + 2 Timeout 对冲：主文件 PageDown 子测
  单腿 F↔整 T 相位之一侧 + detailed 文件级 F↔T 相位之一侧）。
- **stable 8514（8306P/202F/5T/1U，norm 配方口径）**。
- **vs s451 stable（8516）恰 2 差集，全部 select-home-end 族**：主文
  件 PageDown 子测 Fail 键与 detailed 文件级 Timeout 键本轮入双腿振
  荡 → 双双出 stable（s451 该两键双腿一致在 stable）。族相位轮换记
  账口径不变，**非族键 comm 逐键零差**，零 GONE/零 NEW。
- **P 集 8306（账本口径 8305）逐键恒等，s444→s452 九轮连续**。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 19.66s；基线 s451 1643P，
+0——零入树刀，无新钉预期）。

## 结论

s452 收口（守成巡查轻覆盖轮，零源码改动）：他流入树仅 rendering 流
docs 巡查记录，同码树基带复现确认——基带 **8305P/203F/6T/1U → 本轮
leg 形态 8305~8306P/202~203F/5~6T/1U，全部变动限于 select-home-end
族相位**，P 集恒等口径维持。target expando 刀落树对账预告维持（彼流
仍备刀态；落树后按他流刀归因轮全量双腿对账，337 页半径全在 css 组，
html-semantics 面预期零位移，join 实测为准）。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，不重复
通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s452/{s452-leg1,s452-leg2}.log、s452-{l1,l2}.norm、
s452-stable.norm、s452-vs-s451.diff、s452-bridge.log、s452-bridge.rc。
