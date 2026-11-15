# M2 s477 — 守成巡查轻覆盖轮：零入树同码树基带复测，双腿完全一致（8513/8513 双 8306P/201F/5T/1U，disagree 0 leg-only 0）与 s476 leg1 逐键恒等，P 集 8306 持位 + 三观察哨零翻红 + 全桥绿窗 1644P/0F

日期：2026-11-15。承前：s476 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = be3648136 即 s476 收口
docs 提交；代码树与 s476 双腿树全同）→ 无归因对象，轻覆盖轮。

## 双腿 join（be3648136 同码树，make testharness-html-semantics ×2）

- leg1/leg2 **均 8513 行（双 8306P/201F/5T/1U）**，distinct 各
  8511，**disagree 0 + leg-only 0**——本系列最净一轮（双腿完全
  一致）；stable **8511（8306P/201F/5T/1U，distinct 8511）**。
- **vs s476 leg1 逐键恒等（GONE 0 + NEW 0）**；vs s475 stable
  恰 GONE 0 + NEW 1（select-home-end 族整文件键 Fail 形态，与
  s476 leg1 同形，族相位谱内连续第二轮持位）。
- **P 集账本口径 8306 持位**（第二轮连续）；s476 的 leg2 二页
  sweep 截断本轮未复现（双腿零整文件 Timeout 键新发）。

## 观察哨复核（三轮全零翻红）

- onblur 哨组（s476 起常规盯防）：following-label 双键 stable
  Pass；红池 parent-label 双键照常 F（哨组 stable 9 键 = 7P +
  2F 在册）。
- dialog-overlay 观察哨第 7 轮复核无再翻红（哨键 stable Pass）；
  modulepreload 观察哨第 9 轮无再翻红（15 键 stable 全 Pass，
  s450 起累计 54/55 腿）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1644P/0F rc=0**（test 阶段 32.38s；基线 s476 1644P，+0，
filtered 2389 同值，测试清单与 s476 逐项恒等）。

## 结论

s477 收口（守成巡查轻覆盖轮，零源码改动）：基带形态 **8306P/
201F/5T/1U** 全数持位，双腿完全一致零分歧，族形态与 s476 恒等，
P 集 8306 第二轮连续。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang 观
察项静默哨维持（双腿零 hang）。

工件：/tmp/s477/{s477-leg1,s477-leg2}.log、s477-{l1,l2}.norm、
l{1,2}.ks、stable.norm、s477-stable.norm、st477.u、
gone/new-vs475.lines、gone/new-vs476.lines、s477-bridge.log、
bridge477.names。
