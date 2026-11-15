# M2 s493 — 守成巡查轻覆盖轮第六轮（零入树同码态同二进制，连续第六轮）：onblur checkbox following-label 键**单腿瞬态再现**（leg2 F，s489 leg1 F 换腿第二例，本轮干净窗零负载背景——负载归因不成立，升级在册 flaky 观察项）致 stable 8513 vs s492 GONE 1 + NEW 0 + flips 0——非族键逐键恒等零稳定回归，族键全数持位 + P 集 leg1 8307P 持位第 15 轮 + picker 增益键第 16 轮保持 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-15。承前：s492 下一步「守成巡查（对账基线
换 s492 stable 8514；他流入树归因轮或轻覆盖）」。开工
`git pull --rebase`「已经是最新的」；码态核查
`git diff 6b84507fe..HEAD -- crates/ apps/` 零行——对端
R5677（a8143eef9 docs）记 ZW-3 三文件刀五文件 staged 收口
临近 + offset-position 刀实施中，均未落树，无归因对象 →
轻覆盖轮连续第六轮。

## 对端观察

起腿时对端零进程在飞，load 0.53 带内起腿——**干净窗**（此
点对下述瞬态归因关键）。

## 二进制 provenance

- leg1 起跑 make 重链（`Compiling` ×6 同内容重链形态）；
  leg2 零 `Compiling` 同二进制。runner size 126960144 恒。
- 码态 = 6b84507fe（HEAD 5584cf5db docs-only）。
- 工序注记：leg2 首跑在 /tmp/s493 目录内误起 make（无
  Makefile 目标，log 83 字节即败），删除垃圾产物后回仓库
  根目录重跑成功——管线脚本与 make 入口分目录操作时的路
  律：make 必须在仓库根。

## 双腿 join（5584cf5db 树 = 码态 6b84507fe，make testharness-html-semantics ×2）

- leg1 **8516（8307P/202F/6T/1U）** + leg2 **8516
  （8306P/203F/6T/1U）**，双零截断零真 hang（rc=2 惯例失
  败退出）。leg2 -1P/+1F 来自 onblur following-label 键单
  腿瞬态（见下）。
- join：**stable 8513（8304P/202F/6T/1U）disagree 1 +
  leg-only 0/0**——族键全数持位（optional 4 子测 F +
  detailed 整文件 T，s492 同款）。
- **vs s492 stable（8514 = 8305P/202F/6T/1U）：GONE 1 +
  NEW 0 + flips 0 + T 键集恒等 6/6——非族键逐键恒等零稳
  定回归**。GONE = onblur checkbox following-label 键
  （checkable-active-onblur-with-click :: Checkboxes
  should clear :active when the user tabs away from them
  while pressing the following label with a pointing
  device；leg1 P / leg2 瞬态 F）。
- **升级判定：在册 flaky 观察项**——s489 leg1 单腿瞬态 F
  （彼时归因对端 b2 负载窗）→ s490/s491/s492 三轮 stable
  P → s493 leg2 单腿瞬态 F。两例**换腿**、间隔 4 轮；本轮
  起腿为干净窗（对端零进程 load 0.53），**负载归因不成立
  ——瞬态源非仅对端负载，runner 自身 :active 清除时序抖
  动在册**（点击/tab 交互时序敏感型用例带内 flaky）。观
  察升级：后续哨组复核按 3 例窗口观察频率；stable 位置摇
  摆（P↔退）非单向迁移，不构成稳定回归，不升级 bug 排查。
- T 键集 6 恒等；**P 集账本口径 leg1 8307P 持位第 15 轮**
  （leg2 8306P 瞬态 -1 单键）；stable 机械口径 8304P。双发
  发射伪影 2 键照旧在册。

## 观察哨复核（零稳定翻红；一键单腿瞬态退 stable）

- onblur 哨组 **9 键 stable 7P/2F**（parent-label 双键 F 红
  池照常）+ checkbox following-label 键 leg2 瞬态 F 退
  stable（第二例换腿，升级在册 flaky 观察项）。
- dialog-overlay 哨第 22 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 25 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 16 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 15.66s；filtered 2392 恒）。
测试清单 vs s492：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 结论

s493 收口（守成巡查轻覆盖轮第六轮，零源码改动）：零入树同
码态同二进制（6b84507fe 持位），唯一位移 = onblur 哨组一
键单腿瞬态退 stable（GONE 1；**换腿第二例且干净窗——升级
在册 flaky 观察项，瞬态源在册为 :active 清除时序抖动非仅
负载**，stable 摇摆非单向迁移零稳定回归），族键全数持位，
P 集 leg1 8307P 持位第 15 轮，picker 增益键第 16 轮保持，
全桥绿窗钉面 1655 恒等零破。挂用户门维持：GB-20261114-VIDEO
+ GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s493 stable 8513；他流入树归
因轮或轻覆盖；onblur following-label 键按升级后 flaky 观
察项 3 例窗口观察；css 流 offset-position 刀实施中 + ZW-3
三文件刀五文件 staged 收口临近信号持续，落树后按他流刀归
因轮）。

工件：/tmp/s493/{s493-leg1,s493-leg2}.log、s493-{l1,l2}.norm、
l{1,2}.canon、s493.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、flips（零行）、gone.txt（1 瞬态键）、
new.txt（零行）、t492.keys/t493.keys（T 集恒等对账）、
s492.stable.{keys,canon}（基线复用）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s493-bridge.log、
b493.list、b492-b493.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
