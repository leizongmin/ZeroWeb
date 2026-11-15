# M2 s468 — 守成巡查轻覆盖轮 + modulepreload 非族键单次翻红 solo 定谳（36/37 腿 Pass + solo ×3 全 Pass = 瞬时 flake 非刀致，立观察哨；P 集 8305 单轮，族 5 键反而全形态一致在册）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s467 下一步「守成巡查」。本轮开工
`git pull --rebase` 携入渲染流 docs 三笔（9ab2fa1cf R5632 /
53e400209 R5632 注记 / f2a6f3843 R5633，均 docs-only；R5633 预告
ZeroWeb-3 part05.js 刀 +98/−3 施工中未落树）→ 当前零源码入树，代
码树与 s467 双刀组合态树全同 → 轻覆盖轮。

## 双腿 join（d49921587 前身树即 10f81b307 码态，make testharness-html-semantics ×2）

- leg1 **8516 键（8305P/204F/6T/1U）**，leg2 **8516 键
  （8306P/203F/6T/1U）**；腿键集完全一致（leg-only 0/0）。
- **非族键分歧首现**：disagree = 1 = modulepreload-referrerpolicy
  文件 same-origin 子测（leg1 Fail ↔ leg2 Pass）——select-home-end
  族本轮反而全体一致（4 子测 F + detailed T 双腿一致，5 键全形态
  在册，整文件键双腿缺席）。
- **stable 8515（8305P/203F/6T/1U，norm 配方口径，distinct
  8513）**。
- **vs s467 stable 恰 1 GONE（modulepreload same-origin P 键——非
  族）+ 1 NEW（detailed T 键入，族）**；共享键零状态翻转，余非族
  键逐键零差。
- **P 集账本口径 8306 → 8305 单轮破位**（该键因腿间分歧出
  stable）。

## modulepreload 键 solo 定谳（s453 onblur 先例流程）

- 历史全查：该键 s450-s467 全部 **36 腿 Pass**（跨刀前刀后树；
  编辑流 069c6aed6 + ef429d0e9 双刀在树后已跑 s466/s467 共 4 腿全
  Pass），仅 s468 leg1 单次 Fail。
- solo A/B（FILTER=modulepreload-referrerpolicy，同树同二进制
  ×3）：**三跑全 Pass rc=0**。
- 定谳：全量 sweep 负载条件下瞬时 flake（fetch/referrer 时序相位
  带），非刀致、非确定性回归。**立观察哨**：该键后续轮次若再翻红
  则升级调查（sweep 资源竞争面或真边缘行为）；单次翻红不回头改记
  既往轮次口径。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 16.79s；基线 s467 1643P，+0，filtered
2389 同值）。onblur 哨键组 10 键双腿一致 Pass 在 stable（哨解除后
第十四轮）。

## 结论

s468 收口（守成巡查轻覆盖轮，零源码改动 + 一次非族键 solo 定
谳）：基带形态 **8305~8306P/203~204F/6T/1U**，族相位本轮全形态一
致在册，唯一变动为 modulepreload same-origin 键单次翻红（已定谳
瞬时 flake + 观察哨）。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang 观察
项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s468/{s468-leg1,s468-leg2}.log、s468-{l1,l2}.norm、
s468-stable.norm、s468-stable.keys、solo-{1,2,3}.log、
s468-bridge.log、s468-bridge.rc。
