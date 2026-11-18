# M2 s490 — 守成巡查轻覆盖轮第四轮（零入树同码树同二进制，连续第四轮）：**双腿完全一致**第五次（双 8307P/202F/6T/1U），s489 瞬态键双腿 Pass 回归 stable——vs s489 stable GONE 0 + NEW 1（即该键回收）+ flips 0 非族键逐键恒等，瞬态单轮单腿收案不升级，族键全数持位 + P 集 8307 持位 + picker 增益键第 13 轮保持 + 五观察哨零新翻红 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-15。承前：s489 下一步「守成巡查（他流入树归
因轮或轻覆盖）＋瞬态键下轮双腿复核」。开工 `git pull
--rebase`「已经是最新的」（本地已携 R5673/R5674 两笔
rendering docs，零代码）；码态核查 `git diff 492a0f2e0..HEAD
-- crates/ apps/` 零行——ZeroWeb-3 三文件刀 per R5673/R5674
施工面 185/16/50 持位未落树（testharness.rs +36→+50 连续第
二轮推进＝落地窗口临近信号持续），无归因对象 → 轻覆盖轮连
续第四轮。

## 对端观察

起腿时 ZW-3 clone b2-wip testharness-selection 长跑在飞
（guard 2400s 档，异树零干扰），load 1.79 带内起腿。

## 二进制 provenance

- leg1 起跑 make 重建（`Compiling` ×6，runner mtime
  23:16:11，size 126909176 与 s486-s489 逐字节同值——同内
  容重链现象持续）；leg2 零 `Compiling` 同二进制。
- 码态 = 492a0f2e0（HEAD 359d1268d docs-only）。

## 双腿 join（359d1268d 树 = 码态 492a0f2e0，make testharness-html-semantics ×2）

- leg1/leg2 **均 8516 行（双 8307P/202F/6T/1U）——双腿完全
  一致 disagree 0 + leg-only 0**（s479/s484/s486/s488 以来
  第五次全同形态）。双腿零截断零真 hang（rc=2 惯例失败退
  出）。
- join：**stable 8514（8305P/202F/6T/1U）**——族键全数持
  位（optional 4 子测 F + detailed 整文件 T，s488/s489 同
  款）。
- **vs s489 stable（8513 = 8304P/202F/6T/1U）：GONE 0 + NEW 1
  + flips 0**——NEW = onblur checkbox following-label 键回
  归 stable Pass（双腿 Pass，s489 leg1 单腿瞬态收案：单轮
  单腿带内瞬态确认，不升级观察项）；**非族键逐键恒等**。
- T 键集 6 = 非族 5 恒等 + detailed 整文件 T（s488/s489 同
  款）；**P 集账本口径 8307 持位**（双腿双 8307P 第 12 轮
  同形）；stable 机械口径 8305P。双发发射伪影 2 键照旧在
  册（8516 raw → 8514 unique）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 **10 键全回 stable 8P/2F**（following-label
  双键 Pass 持位恢复，parent-label 2F 红池照常）——s489 瞬
  态键回收闭环。
- dialog-overlay 哨第 19 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 22 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 13 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码树旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 18.85s；filtered 2389 同值）。
测试清单 vs s489：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 结论

s490 收口（守成巡查轻覆盖轮第四轮，零源码改动）：零入树同
码树同二进制（492a0f2e0 持位），双腿完全一致第五次，s489
瞬态键单轮单腿收案（双腿 Pass 回归 stable，GONE 0 + NEW 1
+ flips 0 非族逐键恒等），族键全数持位，P 集 8307 持位，
picker 增益键第 13 轮保持，五观察哨零新翻红，全桥绿窗钉面
1655 恒等零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

工件：/tmp/s490/{s490-leg1,s490-leg2}.log、s490-{l1,l2}.norm、
l{1,2}.canon、s490.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon,kts}、flips.txt、gone.txt、new.txt、
s489.stable.keys（s489.stable.canon 派生）、s477-{onblur,
dialogov,mpreload,waapi}.keys（哨键组复用）、s490-bridge.log、
b490.list、b489-b490.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
