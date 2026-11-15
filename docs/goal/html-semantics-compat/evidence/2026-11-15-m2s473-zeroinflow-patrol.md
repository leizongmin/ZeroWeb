# M2 s473 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（双腿完全一致 disagree=0，P 集 8306 第二轮持位，detailed 族键首次以 Fail 形态入 stable——族全变动限相位谱，双观察哨零翻红）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s472 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = 46551ee73 即 s472 收口
docs 提交；代码树与 s472 双腿树全同，ZeroWeb-3 T 切片 d34 刀
part03.js +22＋part05.js +45 仍 unstaged 未落树）→ 无归因对象，
按预案执行纯轻覆盖轮。

## 双腿 join（46551ee73 码态树，make testharness-html-semantics ×2）

- leg1 **8513 行（8306P/200F/6T/1U）**，leg2 **8513 行（8306P/
  200F/6T/1U）**——腿键集完全一致（distinct 各 8511，leg-only
  0），**disagree = 0**，状态逐键全同。
- select-home-end 族本轮形态：4 子测键双腿全部缺席；**detailed
  整文件键双腿一致以 Fail 形态出现**（该键 stable 首现 F 形态—
  —既往入册均为 T 形态；s466 曾单腿 F 未入册，本轮双腿一致 F 入
  stable，族相位谱新增一员在册位形）；族主文件整文件键双腿一致
  Timeout。族 3 键（detailed F + 整文件 T + 子测缺席）全数相位
  谱内。
- **stable 8513（8306P/200F/6T/1U，norm 配方口径，distinct
  8511）**。
- **vs s472 stable 恰 4 GONE（detailed T + 4 子测 F）+ 2 NEW
  （detailed F 形态 + 族整文件 T）**——行差集全数族内；per-key
  计 1 翻转 = detailed 键 T→F 形态翻转（族键），共享键余者零状
  态翻转，非族键逐键零差。
- **P 集账本口径 8306 第二轮连续持位**（s471 单轮破位、s472 回
  位后本轮持位；刀后三轮 P 集无真位移）。
- **dialog-overlay 观察哨第 3 轮复核无再翻红**：该文件 2 键双腿
  全 Pass，哨键在 stable，单次翻红判定维持，哨继续盯防。
- **modulepreload 观察哨第 5 轮复核无再翻红**：referrerpolicy
  面 7 键双腿全 Pass（s450 起累计 46/47 腿），哨继续盯防。
- onblur 哨键组 10 键双腿一致 Pass 在 stable，红池 onblur 2 F
  键照常在册——组成与 s472 逐行全同（哨解除后第十九轮）。
- stable T 键集 6 键 = 5 非族 + 族整文件 T（与 s467/s471 stable
  同集形态）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 26.75s；基线 s472 1643P，+0，
filtered 2389 同值——零入树刀，无新钉预期）。

## 结论

s473 收口（守成巡查轻覆盖轮，零源码改动）：基带复现确认——基带
形态 **8306P/200F/6T/1U 双腿完全一致**，全部变动限于
select-home-end 族相位谱（detailed 键 stable 首现 F 形态 + 整文
件 T 回入 + 4 子测缺席），非族键逐键零差，双观察哨（moduleprel
oad + dialog-overlay）零翻红，P 集口径 8306 第二轮连续。**下轮
注意**：ZeroWeb-3 T 切片 d34 iframe 子文档 head 段三臂刀（R5637
预告 part03.js +22＋part05.js +45 unstaged）落树后按他流刀归因
轮。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+
s137（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮双
腿零 hang）。

工件：/tmp/s473/{s473-leg1,s473-leg2}.log、s473-{l1,l2}.norm、
l{1,2}.kstat、s473-stable.norm、st472.kstat、ks47{2,3}、ob47{2,3}、
s473-bridge.log、s473-bridge.rc。
