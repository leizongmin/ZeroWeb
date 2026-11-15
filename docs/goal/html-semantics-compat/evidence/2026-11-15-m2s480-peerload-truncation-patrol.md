# M2 s480 — 守成巡查轻覆盖轮（零入树同码树）：leg1 他流 sweep 并跑窗口二族页截断 solo 全数回位定谳负载相位非树退化（s476 先例流程），leg2 静窗全数基带，P 集 8307 持位 + 三观察哨零翻红 + 全桥绿窗 1644P/0F

日期：2026-11-15。承前：s479 下一步「守成巡查（他流入树归因轮或
轻覆盖）」。开工 `git pull --rebase` 远端零新提交（HEAD = f6fd2f750
即 s479 收口 docs；码态 70cb4d2f5 与 s479 双腿树全同）→ 无归因
对象，轻覆盖轮。leg1 起跑时他流（ZeroWeb-3）snapshot 二进制
selection sweep 并跑（host load ~6.5，s476 截断带 ~10-13 之下
但仍在并跑窗内）。

## 双腿 join（f6fd2f750 同码树，make testharness-html-semantics ×2）

- leg1（并跑窗）**8516 行（8307P/201F/7T/1U，distinct 8514）**
  ——族二页截断相位：select-home-end optional 页整文件 T + 仅
  3/4 子测（PageDown 缺席）+ detailed 页整文件 T；leg2（静窗）
  **8516 行（8307P/202F/6T/1U，distinct 8514）零截断**（optional
  4 子测全 F + detailed 整文件 T，s478 形）。
- join：**stable 8513（8305P/201F/6T/1U），disagree 0，leg-only
  1/1**（l1only = optional 整文件 T 键、l2only = PageDown 子测键
  ——两侧皆 leg1 截断伪影）。
- **vs s479 stable：flips 1 = 族 detailed 整文件键 Fail→Timeout
  （族整文件键 T↔F 相位谱，s464 起在册）**；键集差 GONE 1
  （optional 整文件键）+ NEW 3（End/Home/PageUp 子测）全数族相
  位 + 截断伪影，非族键逐键零差。
- **P 集账本口径 8307 持位**（双腿双 8307P）；stable P 8305
  （机械口径）。

## 截断 solo 定谳（s476 先例流程）

- **optional 页 solo ×2**（FILTER=select-home-end-pagedown-pageup
  .optional.html，同树同二进制，静窗）：页跑通，**4 子测全 F
  （PageDown 在册）rc=2**——leg1 的整文件 T + PageDown 缺席 =
  并跑窗负载相位截断，非树退化。
- **detailed 页 solo ×2**：页跑通，4 子测全 F（Home-and-End 变
  体 ×3 + PageUp-and-PageDown，与 optional 页子测集不同的更重断
  言面）——sweep 只浮整文件键系页重 + sweep 负载（s476「solo
  逐文件回位」同型），F↔T 整文件键振荡为 sweep 相位谱内行为。
- 两页截断形态在族相位谱内连续在册（s476 二页整文件 T 截断 /
  s478 双腿 4 子测 F / s479 detailed F + optional T / 本轮 leg1
  二页 T + 部分子测）——**零非族位移**。

## 观察哨复核（全零新翻红）

- onblur 哨组：following-label 双键 stable Pass 持位（s478 第 3
  次翻红后连续第二轮回位）；红池 parent-label 双键照常 F（哨组
  stable 9 键 = 7P + 2F 在册）。
- dialog-overlay 观察哨第 9 轮零翻红（双键 stable Pass）；
  modulepreload 观察哨第 12 轮零翻红（15 键 stable 全 Pass）。
- picker-icon 刀增益键 stable Pass 持位（s478 增益第三轮保持）；
  checkmark 姊妹键恒 F 相位带形不变。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1644P/0F rc=0**（test 阶段 15.19s；filtered 2389 同值，测试
清单 vs s479 set 恒等 1644）。

## 结论

s480 收口（守成巡查轻覆盖轮，零源码改动）：同码树基带复测，leg
间族页相位差全数 solo 定谳负载相位非树退化，非族键逐键零差，P
集 8307 持位，picker 增益键保持，三观察哨零翻红，全桥绿窗钉面
零破。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+
s137（s445 已提示，不重复通知）。hang 观察项静默哨维持（双腿
零真 hang——族二页整文件 T 系在册相位非 hang）。

工件：/tmp/s480/{s480-leg1,s480-leg2}.log、s480-{l1,l2}.norm、
l{1,2}.canon、s480.stable.canon、s480.stable.keys、
gone/new-l1.keys、t480.pages、solo-{opt,det}-{1,2}.log、
solo-run.progress、s480-bridge.log、b480.s、
norm.sh/canon.sh/join.sh（s478 沉淀管线复用）。
