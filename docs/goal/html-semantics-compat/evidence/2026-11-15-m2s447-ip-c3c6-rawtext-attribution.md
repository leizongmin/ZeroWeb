# M2 s447 — 守成巡查轮：ip C3/C6 非文本原子拆分边界刀零位移清偿（P 集恒等 8305 四轮连续）+ 全桥绿窗 1643P/0F（+6 新钉）

日期：2026-11-15。承前：s446 收口后 push 窗 rebase 入他流一刀
6c21880ac，本轮接续归因（同 session 守成巡查第二刀）。

## 归因面

6c21880ac（editing-compat ip 非文本原子拆分边界刀——C3 注释原子先摘
后挂 + C6 script rawtext M1 直构）：触 engine `js_dom_shim/part01f.js`
（**corpus runner shim 本体**——caret 拆分臂行为面）+
js_dom_bridge_tests.rs（part120 新钉 6 测）+ 新账本。C6 触 script 壳
克隆链，与语料 script 执行面潜在交叠 → 全量双腿归因成立。

## 双腿 join（6c21880ac 树，make testharness-html-semantics ×2）

- **stable 8515（8305P/203F/6T/1U），双腿逐键全一致（unstable = 0，
  本轮连族内相位振荡都未出现）**。
- **vs s446 stable（8510 = 8305P/199F/5T/1U）：恰 5 差集** = select-
  home-end 族 Fail 相位整体回入 stable（主文件 4 子测 F + detailed
  变体文件级 T 键）——与 s444「203F/5T」形态同型（s444 为 4F/5T，本轮
  detailed 文件级键另入 1T），族内相位轮换记账口径不变，非真位移。
- **P 集 8305 逐键恒等，s444→s447 四轮连续**；**C3/C6 刀零位移定谳**
  ——shim caret/script rawtext 改动面在本语料零键移动（语料 script
  执行族 execution-timing/149 等全数维持原态）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 17.12s；基线 s446 1637P，
**+6 = C3/C6 part120_ip_atomic_rawtext_pins 新钉**，与 commit 钉测数
一致）。

## 结论

s447 收口（守成巡查轮，零源码改动）：**in-domain 自由面维持实质清
零**，ip 系三刀（C1/C3/C6）全部零位移清偿；基带 **8305P/203F/6T/1U**
（族 Fail 相位在 stable；P 恒等为主口径）+ select-home-end 族振荡记
账不变。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137
（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang，
且双腿全一致零不稳定键）。

工件：/tmp/s445/{s447-leg1,s447-leg2}.log、s447-{l1,l2}.norm、
s447-bridge.log、s447-bridge.rc。
