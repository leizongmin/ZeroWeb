# M2 s472 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（双腿完全一致 disagree=0，P 集 8306 回位兑现，dialog-overlay 哨第 2 轮无再翻红，族 5 键回全形态在册）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s471 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = 0818d416a 即 s471 收口
docs 提交；代码树与 s471 归因轮双腿树 bc009e7a3 全同，含 css 流
WAAPI 门刀 107bcd935 已清偿组合态）→ 无归因对象，按预案执行纯轻
覆盖轮。

## 双腿 join（0818d416a 码态树，make testharness-html-semantics ×2）

- leg1 **8516 行（8306P/203F/6T/1U）**，leg2 **8516 行（8306P/
  203F/6T/1U）**——腿键集完全一致（distinct 各 8514，leg-only
  0），**disagree = 0**，状态逐键全同（含 2 已知重复键）。
- select-home-end 族本轮 5 键全形态在册且双腿一致：4 子测 Fail +
  detailed 整文件 Timeout；族主文件整文件键双腿缺席。
- **stable 8516（8306P/203F/6T/1U，norm 配方口径，distinct
  8514）**。
- **vs s471 stable 恰 1 GONE（族整文件 T 键，双腿缺席）+ 5 NEW
  （detailed T + 4 子测 F 回入，全数族内 + dialog-overlay P 哨键
  回位）**；共享键零状态翻转（flips=0），非族键逐键零差。
- **P 集账本口径 8306 回位兑现**（s471 单轮破位 8305 后按 solo
  定谳预期回位；刀后两轮合计 P 集无真位移）。
- **dialog-overlay 观察哨第 2 轮复核无再翻红**：该文件 2 键双腿
  全 Pass，`dialog computed overlay initially 'none'` 键回入
  stable，单次翻红瞬时 flake 判定维持，哨继续盯防。
- **modulepreload 观察哨第 4 轮复核无再翻红**：referrerpolicy 面
  7 键双腿全 Pass（s450 起累计 44/45 腿），哨继续盯防。
- onblur 哨键组 10 键双腿一致 Pass 在 stable，红池 onblur 2 F 键
  照常在册——组成与 s471 逐行全同（哨解除后第十八轮）。
- stable T 键集 6 键 = 5 非族 + detailed T（与 s461-s466/s469
  stable 同集形态）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 18.90s；基线 s471 1643P，+0，
filtered 2389 同值——零入树刀，无新钉预期）。

## 结论

s472 收口（守成巡查轻覆盖轮，零源码改动）：WAAPI 门刀后组合态基
带复现确认——基带形态 **8306P/203F/6T/1U 双腿完全一致**，全部变
动限于 select-home-end 族相位（整文件键出 + detailed T/4 子测回
入）+ dialog-overlay 哨键预期回位，P 集口径 8306 恢复，双观察哨
（modulepreload + dialog-overlay）均无再翻红，刀后基带零漂移。挂
用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s472/{s472-leg1,s472-leg2}.log、s472-{l1,l2}.norm、
l{1,2}.kstat、s472-stable.norm、st471.kstat、ks47{1,2}、ob47{1,2}、
s472-bridge.log、s472-bridge.rc。
