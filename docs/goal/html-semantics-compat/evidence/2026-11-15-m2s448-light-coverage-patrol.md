# M2 s448 — 守成巡查轻覆盖轮：零入树刀基带复测（P 集恒等 8305 五轮连续，select-home-end 族相位再轮换）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s447 下一步「守成巡查（他流入树归因轮或轻覆盖）」。
本轮开工 `git pull --rebase` 确认远端无新刀（最新 = af012f775，即 s447
收口 docs 提交，代码树与 s447 双腿树 6c21880ac 全同）→ 无归因对象，
按预案执行纯轻覆盖轮（零源码零语料净变更）。

## 双腿 join（af012f775 树，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/200F/6T/1U）**，leg2 **8516 键（8306P/204F/5T/
  1U）**；unstable 集 5 键全在 select-home-end 族（主文件
  select-home-end-pagedown-pageup.optional.html：leg1 整文件 Timeout
  相位 vs leg2 4 子测 Fail 相位——与 s446 记载的族内两形态同型）。
- **stable 8512（8306P/200F/5T/1U，本账本 norm 配方口径；s447 同配方
  复推导 = 8516 = 8306P/203F/6T/1U，配方含双空格子测行 +1P 去重差异，
  双侧同配方对齐后比较不受影响）**。
- **vs s447 stable 恰 6 差集，全部 select-home-end 族**：s447 形态
  {主文件 4 子测 F + detailed 文件级 T} → 本轮形态 {detailed 文件级 F
  （双腿一致入 stable）+ 主文件相位在双腿间振荡（T↔4F，不入 stable）}
  ——族内相位轮换记账口径不变，非真位移。
- **P 集 8306（账本口径 8305）逐键恒等，s444→s448 五轮连续**；零
  GONE / 零 NEW 键，非族键全数恒等。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 15.13s；基线 s447 1643P，
+0——零入树刀，无新钉预期）。

## 结论

s448 收口（守成巡查轻覆盖轮，零源码改动）：同码树基带复现确认——
**基带 8305P/203F/6T/1U → 本轮 leg 形态 8305P/200~204F/5~6T/1U，全部
变动限于 select-home-end 族相位**，P 集恒等口径维持。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，不重复通
知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s448/{s448-leg1,s448-leg2}.log、s448-{l1,l2}.norm、
s448-stable.norm、s447-{base,base2,stable}.norm（同配方复推导基带）、
s448-bridge.log、s448-bridge.rc。
