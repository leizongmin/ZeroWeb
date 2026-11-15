# M2 s494 — 他流刀归因轮：zero-web 流 input-events 刀 ecc820912 落树（getTargetRanges 快照面 + Actions 编辑键分流，js_dom_shim part01 +185/part05 +16/testharness.rs +50，申报 223P→289P 净 +66 零回归），html-semantics 面**零稳定位移清偿**——「申报=实测」谱系第六例：vs s493 stable GONE 5 + NEW 1 + flips 0，GONE 全族键发射形相位带内（pagedown-pageup 族双整文件键 leg1 T/leg2 F + 4 子测未发射），NEW 1 = onblur flaky 键双腿 Pass 回归 stable（第 2 例复核收案）——编辑面刀与语义面非族键逐键恒等，刀半径零 HTML 语义面稳定影响

日期：2026-11-16。承前：s493 下一步「守成巡查（对账基线换
s493 stable 8513；他流入树归因轮或轻覆盖）」。开工
`git pull --rebase`「已经是最新的」；码态核查
`git diff 6b84507fe..HEAD -- crates/ apps/` **+201 非零**——
**ecc820912**（zero-web 流 input-events 刀：js_dom_shim
part01.js +185 / part05.js +16 / testharness.rs +50 /
wptfyi-editing docs；申报 input-events 测试面 223P→289P 净
+66 零回归）→ 他流刀归因轮。

## 刀时序注记（s493 台账准确性复核）

刀 author date 11-15 23:48:35（对端 clone 本地提交），s493
双腿窗口 23:52-00:01 内 origin 未有（s493 开工 pull「已经
是最新的」+ 码态核查零行双证）——**s493 双腿确实跑在
6b84507fe 旧码上，其台账记录准确**；s493 收口 00:02 pull
携入 ecc820912 + 2b79ab8d5（R5678 docs）。本轮码态 =
**ecc820912**（HEAD 16f6d57a4 docs-only）。

## 对端观察

起腿时对端零进程在飞，load 0.29 带内起腿。

## 二进制 provenance

- leg1 起跑 make 重建（`Compiling` ×6——js_dom_shim part 文
  件变化触发 engine 域重构建 + runner 重链）；leg2 零
  `Compiling` 同二进制。
- runner size **126976568**（s493 恒 126960144 → 本轮
  **+16424 换代**——input-events 刀码入二进制）；码态 =
  ecc820912（HEAD 16f6d57a4 docs-only）。

## 双腿 join（16f6d57a4 树 = 码态 ecc820912，make testharness-html-semantics ×2）

- leg1 **8513 raw（8307P/198F/7T/1U）** + leg2 **8513 raw
  （8307P/200F/5T/1U）**，双零截断零真 hang（rc=2 惯例失
  败退出）。**disagree 2 + leg-only 0/0**。
- join：**stable 8509（8305P/198F/5T/1U）**。
- **vs s493 stable（8513 = 8304P/202F/6T/1U）：GONE 5 +
  NEW 1 + flips 0——非族键逐键恒等零稳定位移**。
  - disagree 2 全族键：select-home-end-pagedown-pageup
    **optional 与 detailed 两整文件键 leg1 Timeout/leg2
    Fail**（族相位 T↔F 摆，s486/s487 先例同款）。
  - GONE 5 全族键：上述 detailed 整文件键（disagree 不入
    stable）+ optional 4 子测键（本轮双腿均未发射子测形——
    族发射形转整文件形，s464 相位谱带内、s487/s491 先例）。
  - **NEW 1 = onblur checkbox following-label 键双腿 Pass
    回归 stable**——s493 leg2 单腿瞬态 F 的**第 2 例复核
    收案**：两例（s489 leg1 F / s493 leg2 F）均单轮单腿、
    stable 摇摆 P↔退↔P 非单向迁移，带内 flaky 形态确认，
    维持 3 例窗口观察（当前 2 例）。
- T 键集 5 = 非族 5 恒等（detailed 整文件键退 出 T 集，
  族带内）；**P 集账本口径双腿双 8307P 持位第 16 轮**；双
  发发射伪影 2 键照旧在册。

## input-events 刀半径归因

- 语义交叠核查：getTargetRanges 快照面 + Actions 编辑键分
  流为 input-events 编辑面（beforeinput/input 事件与编辑命
  令），html-semantics 腿唯一位移全族键带内 + flaky 键带内
  回归，**非族键 flips 0**——刀零 HTML 语义面稳定影响。
  js_dom_shim 改动在 bridge 域但桥面单测清单零增零破（见
  下）——「申报=实测」谱系第六例（前五例：s478 gCS 通道 /
  s485 iframe navigator 平面 / s486 easing 感知离散 /
  s491 offset 簇 / s470s 期内另例）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 **10 键全回 stable 8P/2F**（following-label
  双键 Pass 持位恢复含 flaky 键，parent-label 2F 红池照
  常）——flaky 第 2 例复核收案闭环。
- dialog-overlay 哨第 23 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 26 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 17 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（码态换代旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 31.61s；filtered 2392 恒）。
测试清单 vs s493：**set 恒等 1655，GONE 0**——刀碰
js_dom_shim（bridge 域）但桥面单测零增零破（刀 +66 测试在
wptfyi-editing harness 面，非桥面单测），预期兑现。

## 结论

s494 收口（他流刀归因轮，零源码改动）：input-events 编辑刀
落树码态换新（6b84507fe → ecc820912，runner 换代 +16424），
html-semantics 面**零稳定位移清偿**（GONE 5 全族键发射形
相位带内 + NEW 1 flaky 键带内回归 + flips 0 非族逐键恒
等），族键维持相位谱观察不升级，onblur flaky 键第 2 例复
核收案（3 例窗口观察维持），P 集 8307 持位第 16 轮，picker
增益键第 17 轮保持，五观察哨零新翻红，全桥绿窗钉面 1655 恒
等零破。挂用户门维持：GB-20261114-VIDEO + GB-20261031
（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s494 stable 8509；他流入树归
因轮或轻覆盖；onblur flaky 键 3 例窗口观察维持；css 流
offset-position 刀实施中 + ZW-3 三文件刀 origin 落地触发器
预挂信号持续，落树后按他流刀归因轮）。

工件：/tmp/s494/{s494-leg1,s494-leg2}.log、s494-{l1,l2}.norm、
l{1,2}.canon、s494.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、flips（零行）、gone.txt（5 族键）、new.txt
（1 flaky 键）、t493.keys/t494.keys（T 集差 1 族键带内）、
s493.stable.{keys,canon}（基线复用）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s494-bridge.log、
b494.list、b493-b494.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
