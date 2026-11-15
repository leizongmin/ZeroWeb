# M2 s476 — 守成巡查轻覆盖轮：leg2 二页 sweep 相位整文件 Timeout 截断（77 键缺席）solo 全数回位定谳环境相位非树退化，leg1 全数基带 P 集 8306 持位 + onblur 升级哨期满解除 + 全桥绿窗 1644P/0F

日期：2026-11-15。承前：s475 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = 03dc27b83 即 s475 收口
docs 提交；代码树与 s475 双腿树全同，最后源码刀仍为 d1625c26a
组合态）→ 无归因对象，按预案执行轻覆盖轮。

## 双腿 join（03dc27b83 同码树，make testharness-html-semantics ×2）

- leg1 **8513 行（8306P/201F/5T/1U）**——与 s475 leg2 状态直方
  图恒等；**vs s475 stable 恰 GONE 0 + NEW 1**（select-home-end
  族整文件键 Fail 形态回入，族相位谱内），非族非哨域键逐键零
  差，**全数基带**。
- leg2 **8438 行（8306P/201F/5T/1U）——首现二页 sweep 相位整文
  件 Timeout 截断**：`textfieldselection/select-event.html`
  （50 键缺席 + 整文件 Timeout 键）+
  `the-input-element/focus-dynamic-type-change-on-blur.html`
  （27 键缺席 + 整文件 Timeout 键）；截断点后子测全未发出。
  leg1/leg2 共享键 disagree 0。
- **P 集账本口径 8306 持位**（leg1 基带口径；stable 8434 因
  leg2 截断失真，不作为口径）。

## 二页截断 solo 对照定谳（s468/s471/s474 先例流程，结论同 s468 型）

- solo 单跑（FILTER 逐文件，同树同二进制）：
  select-event **270 行 rc=0 零整文件 Timeout**、
  focus-dynamic-type-change-on-blur **80 行 rc=0 零整文件
  Timeout**——与 leg1 全数一致，**sweep 截断键全数回位**。
- 判读：solo 全 Pass ≠ 全量 sweep 确定性劣化（l1 同树同二进制
  全数在案）；leg2 截断判 **sweep 相位带环境超时**——本轮全量
  期间 host load ~10~13（16 核，vmstat 零 wa）+ 编辑流 runner
  与 ZeroSeed test-all 编译并跑，页级 harness 超时预算被挤占
  （整文件 Timeout 键即页级超时标记，select-home-end 族 detailed
  T↔F 振荡为同现象轻症）。非刀致（零入树）、非真回归证据；
  单轮观察判定，不升级排查。
- 备注：`make testharness-html-semantics FILTER` 含 `|` 会穿
  Make 配方裸 shell 成管道（本轮实测 `/bin/sh: not found`），
  多文件 solo 需逐文件单跑或换 shell 安全分隔。

## 观察哨复核

- **onblur 升级观察哨第 2 轮**（s474 立，2 轮窗口）：following-
  label 双键 leg1 全 Pass，无再现翻红亦无双腿一致 F——**窗口期
  满解除**，降回常规 onblur 哨组盯防（红池 parent-label 双键照
  常 F，哨组 stable 10P + 红池 2F 在册）。
- dialog-overlay 观察哨第 6 轮复核无再翻红（哨键 leg1 Pass）；
  modulepreload 观察哨第 8 轮无再翻红（15 键 leg1 全 Pass，
  s450 起累计 52/53 腿）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1644P/0F rc=0**（test 阶段 31.14s；基线 s475 1644P，+0，
filtered 2389 同值，测试清单与 s475 逐项恒等——零入树刀，无新
钉预期兑现）。

## 结论

s476 收口（守成巡查轻覆盖轮，零源码改动）：基带形态 **8306P/
201F/5T/1U**（leg1 全数基带），P 集 8306 持位；leg2 二页截断判
sweep 相位环境超时（solo 全数回位，单轮事件）；onblur 升级哨期
满解除。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+
s137（s445 已提示，不重复通知）。hang 观察项静默哨维持（双腿零
runner hang；leg2 二页为页级 harness 超时非 runner hang）。

工件：/tmp/s476/{s476-leg1,s476-leg2}.log、s476-{l1,l2}.norm、
l{1,2}.ks、stable.norm、st_joined.u、solo-{se,fd}.log、
solo-1.log（FILTER 管道穿溃残证）、s476-bridge.log、
bridge476.names。
