# M2 s474 — 守成巡查轻覆盖轮 + onblur 哨域键首现 sweep 单腿翻红 solo 对照定谳（solo ×3 全 F ≠ 瞬时 flake 同型：判 solo 相位带键 + sweep 单腿翻红，升级观察哨盯防；P 集 8305 单轮；族回 s472 形态全族内）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s473 下一步「守成巡查」。本轮开工
`git pull --rebase` 远端零新提交（HEAD = f61c4a582 即 s473 收口
docs 提交；代码树与 s473 双腿树全同，最后源码刀仍为 107bcd935
WAAPI 门刀已清偿组合态；ZeroWeb-3 d34 刀 R5639/R5640 连续停滞
未落树）→ 无归因对象，按预案执行轻覆盖轮 + 一次哨域 solo 对照。

## 双腿 join（f61c4a582 码态树，make testharness-html-semantics ×2）

- leg1 **8516 行（8305P/204F/6T/1U）**，leg2 **8516 行（8306P/
  203F/6T/1U）**；distinct 各 8514，leg-only 0，**disagree = 1**
  = **onblur 哨域键 sweep 单腿翻红**：`checkable-active-onblur-
  with-click.html :: Checkboxes … pressing the following label`
  键 leg1 Fail ↔ leg2 Pass（onblur 哨组解除后第 20 轮首现翻红；
  同文件 radios following-label 键双腿 Pass）。
- select-home-end 族本轮回 s472 形态且双腿一致：4 子测 Fail +
  detailed 整文件 Timeout（s473 的 detailed F 位形未固化——单轮
  观察判定维持）；族整文件键双腿缺席。
- **stable 8515（8305P/203F/6T/1U，norm 配方口径，distinct
  8513）**。
- **vs s473 stable 恰 3 GONE（onblur following-label P 非族哨域
  键 + 族 detailed F + 族整文件 T）+ 5 NEW（族 detailed T + 4
  子测 F）**；per-key 1 翻转 = detailed F→T（族键形态回摆），
  余共享键零翻转，非族非哨域键逐键零差。
- **P 集账本口径 8306 → 8305 单轮破位**（onblur 哨域键出 stable
  致；树零刀四轮、二进制与 s471-s473 腿恒等，非刀致）。

## onblur 哨域键 solo 对照定谳（s468/s471 先例流程执行，结论异型）

- solo A/B（FILTER=checkable-active-onblur-with-click，同树同二
  进制 ×3）：**三跑 rc=2，following-label 双键（Checkboxes +
  Radios）全 F**；"pressing it" 双键 + Wait for load 全 P，红池
  parent-label 双键照常 F。
- **判读异型**：s468（modulepreload）与 s471（dialog-overlay）先
  例中 solo ×3 全 Pass → 全量 sweep 瞬时 flake；本轮 solo 反为
  确定性 F，而 sweep 39 腿历史 38 P——**该键为 solo/sweep 双相
  位带键**：solo 相位带确定性 F（unload 时序相位），sweep 相位带
  38/39 腿 P 带 1 次边缘翻红。solo F 不证 sweep 真回归（l2 同树
  同二进制 P），sweep 单腿 F 亦不证确定性劣化——判 **solo 相位
  带键 + sweep 单腿翻红**，非瞬时 flake 简单同型，亦非刀致/真回
  归证据。
- **升级观察哨（onblur following-label 双键）**：后续轮次盯防
  sweep 翻红——2 轮内再现或转双腿一致 F → 升级真回归排查（引
 擎状态 bisect + solo/sweep 双相位对照归因）；单次翻红不回改既
  往轮次口径。onblur 哨组本轮 stable 9P（组内 1 键对冲出）+ 红
  池 2F 照常在册。
- dialog-overlay 观察哨第 4 轮复核无再翻红（哨键 Pass 在
  stable）；modulepreload 观察哨第 6 轮无再翻红（7 键 Pass，s450
  起累计 48/49 腿）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 15.22s；基线 s473 1643P，+0，
filtered 2389 同值——零入树刀，无新钉预期）。

## 结论

s474 收口（守成巡查轻覆盖轮，零源码改动 + 一次哨域 solo 对
照）：基带形态 **8305~8306P/203~204F/6T/1U**，族回 s472 形态全
族内（detailed F 位形单轮未固化），唯一非族变动为 onblur 哨域
键 sweep 单腿翻红（solo 对照判双相位带键 + 升级观察哨），P 集
8305 单轮预期回位。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。hang 观察
项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s474/{s474-leg1,s474-leg2}.log、s474-{l1,l2}.norm、
l{1,2}.kstat、s474-stable.norm、st473.kstat、ks47{3,4}、
solo-{1,2,3}.log、s474-bridge.log、s474-bridge.rc。
