# M2 s492 — 守成巡查轻覆盖轮第五轮（零入树同码态同二进制，连续第五轮）：**双腿完全一致**第六次（双 8516 = 双 8307P/202F/6T/1U，disagree 0 leg-only 0），s491 族键发射形差带内往返收案——vs s491 stable GONE 0 + NEW 4（即 s491 GONE 同 4 族键 Fail 回归 stable，子测形恢复 s488/s490 形）+ flips 0 非族键逐键恒等 + T 集恒等，P 集 8307 持位第 14 轮 + picker 增益键第 15 轮保持 + 五观察哨零新翻红 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-15。承前：s491 下一步「守成巡查（对账基线换
s491 stable 8510；他流入树归因轮或轻覆盖）」。开工
`git pull --rebase`「已经是最新的」；码态核查
`git diff 6b84507fe..HEAD -- crates/ apps/` 零行——css 流
offset-position 刀（实例 2614542 实施中信号，wptfyi-css
master.md 7acc4b056 在记）与 ZeroWeb-3 三文件刀均未落树，
无归因对象 → 轻覆盖轮连续第五轮。

## 对端观察

起腿时对端零进程在飞，load 0.64 带内起腿——负载干扰零。

## 二进制 provenance

- leg1 起跑 make 重链（`Compiling` ×6——码态未变同内容重
  链形态自 s491 换代后恢复）；leg2 零 `Compiling` 同二进制。
- runner size **126960144**（s491 换代值恒——offset 簇刀
  代码已在二进制内，本轮零码变）；码态 = 6b84507fe（HEAD
  97179e7a8 docs-only）。

## 双腿 join（97179e7a8 树 = 码态 6b84507fe，make testharness-html-semantics ×2）

- leg1/leg2 **均 8516 raw（双 8307P/202F/6T/1U）——双腿完
  全一致 disagree 0 + leg-only 0**（s479/s484/s486/s488/
  s490 以来第六次全同形态），族 s488/s490 形（optional 4
  子测 F + detailed 整文件 T）。双腿零截断零真 hang（rc=2
  惯例失败退出）。
- join：**stable 8514（8305P/202F/6T/1U）**——族键全数入
  stable。
- **vs s491 stable（8510 = 8305P/198F/6T/1U）：GONE 0 +
  NEW 4 + flips 0 + T 键集恒等 6/6**——NEW 4 = s491 GONE
  同 4 族键（select-home-end-pagedown-pageup.optional.html
  End/Home/PageDown/PageUp 4 子测键）双腿 Fail 回归 stable
  ：s491 leg2 整文件键 Timeout 发射形未再现，s492 双腿回
  子测形——**s491 归因轮判定的族带内发射形差经一轮带内往
  返收案**（族相位谱 s464 起在册，维持观察不升级）；**非
  族键逐键恒等**。
- T 键集 6 恒等（非族 5 + detailed 整文件 T）；**P 集账本
  口径 8307 持位第 14 轮**（双腿双 8307P 同形）；stable 机
  械口径 8305P。双发发射伪影 2 键照旧在册（8516 raw →
  8514 unique）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 **10 键 stable 8P/2F**（following-label 双键
  Pass 持位，parent-label 2F 红池照常）——瞬态收案后第三
  轮全带形。
- dialog-overlay 哨第 21 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 24 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 15 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 15.39s；filtered 2392 恒）。
测试清单 vs s491：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 结论

s492 收口（守成巡查轻覆盖轮第五轮，零源码改动）：零入树同
码态同二进制（6b84507fe 持位，runner 126960144 恒），双腿
完全一致第六次，s491 族键发射形差带内往返收案（GONE 0 +
NEW 4 族键回位 + flips 0 非族逐键恒等），族键全数入 stable
8514，P 集 8307 持位第 14 轮，picker 增益键第 15 轮保持，
五观察哨零新翻红，全桥绿窗钉面 1655 恒等零破。挂用户门维
持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已
提示，不重复通知）。

下一步：守成巡查（对账基线换 s492 stable 8514；他流入树归
因轮或轻覆盖；css 流 offset-position 刀实例 2614542 实施
中 + ZeroWeb-3 三文件刀 testharness.rs +50 落地窗口临近信
号持续，落树后按他流刀归因轮）。

工件：/tmp/s492/{s492-leg1,s492-leg2}.log、s492-{l1,l2}.norm、
l{1,2}.canon、s492.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、flips（零行）、gone.txt（零行）、new.txt
（4 族键）、t491.keys/t492.keys（T 集恒等对账）、
s491.stable.{keys,canon}（基线复用）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s492-bridge.log、
b492.list、b491-b492.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
