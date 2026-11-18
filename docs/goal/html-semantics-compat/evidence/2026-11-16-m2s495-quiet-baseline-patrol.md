# M2 s495 — 守成巡查轻覆盖轮第七轮（零入树同码态同二进制，连续第七轮）：**双腿完全一致**第七次（双 8516 = 双 8307P/202F/6T/1U，disagree 0 leg-only 0），s494 族键发射形相位带内往返——vs s494 stable GONE 0 + NEW 5（4 子测键 Fail 回子测形 + detailed 整文件键双腿 T 入 stable，s488/s490/s492 形恢复）+ flips 0 非族键逐键恒等 + T 集 5→6 恢复，onblur flaky 键 stable Pass 3 例窗口零新例，P 集 8307 持位第 17 轮 + picker 增益键第 18 轮保持 + 五观察哨零新翻红 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-16。承前：s494 下一步「守成巡查（对账基线换
s494 stable 8509；他流入树归因轮或轻覆盖）」。开工
`git pull --rebase`「已经是最新的」；码态核查
`git diff ecc820912..HEAD -- crates/ apps/` 零行——css 流
offset-position 刀（实例 2614542 实施中）与 ZW-3 后续刀均
未落树，无归因对象 → 轻覆盖轮连续第七轮。对端 R5678 补记
（eb867ac4f）口径确认 ecc820912 = ZW-3 三文件刀落 origin 同
commit（与 s494 归因的 input-events 刀为同一 commit 两面并
载：js_dom_shim +201 编辑面 + testharness.rs +50 harness
面），归因结论不受影响。

## 对端观察

起腿时对端零进程在飞，load 当前 1.13 带内（5/15 分钟档
7.32/7.70 为历史峰尾迹，非在飞负载）。

## 二进制 provenance

- leg1 起跑 make 重链（`Compiling` ×6 同内容重链形态）；
  leg2 零 `Compiling` 同二进制。runner size 126976568 恒
  （s494 换代值）。
- 码态 = ecc820912（HEAD 421913af6 docs-only）。

## 双腿 join（421913af6 树 = 码态 ecc820912，make testharness-html-semantics ×2）

- leg1/leg2 **均 8516 raw（双 8307P/202F/6T/1U）——双腿完
  全一致 disagree 0 + leg-only 0**（s479/s484/s486/s488/
  s490/s492 以来第七次全同形态），族 s488/s490/s492 形
  （optional 4 子测 F + detailed 整文件 T）。双腿零截断零
  真 hang（rc=2 惯例失败退出）。
- join：**stable 8514（8305P/202F/6T/1U）**——族键全数入
  stable。
- **vs s494 stable（8509 = 8305P/198F/5T/1U）：GONE 0 +
  NEW 5 + flips 0**——NEW 5 全族键：optional 4 子测键
  Fail（双腿恢复子测发射形）+ detailed 整文件键 Timeout
  （双腿 T 同相位入 stable）——**s494 归因轮判定的族带内
  发射形相位摆经一轮往返收案**（s494 整文件形 → s495 子
  测形 + detailed T 形，s488/s490/s492 形恢复；族相位谱
  s464 起在册维持观察）；**非族键逐键恒等**，T 键集 5→6
  恢复（新增即 detailed 整文件 T）。
- **P 集账本口径双腿双 8307P 持位第 17 轮**（stable 机械
  口径 8305P）。双发发射伪影 2 键照旧在册（8516 raw →
  8514 unique）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 **10 键 stable 8P/2F**（following-label 双键
  Pass 持位，parent-label 2F 红池照常）——flaky checkbox
  following-label 键 stable Pass，**3 例窗口零新例**（当前
  2 例：s489 leg1 / s493 leg2）。
- dialog-overlay 哨第 24 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 27 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 18 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 16.06s；filtered 2392 恒）。
测试清单 vs s494：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 结论

s495 收口（守成巡查轻覆盖轮第七轮，零源码改动）：零入树同
码态同二进制（ecc820912 持位，runner 126976568 恒），双腿
完全一致第七次，s494 族键发射形相位带内往返收案（GONE 0 +
NEW 5 族键回位 + flips 0 非族逐键恒等），族键全数入 stable
8514，P 集 8307 持位第 17 轮，flaky 键 3 例窗口零新例，
picker 增益键第 18 轮保持，五观察哨零新翻红，全桥绿窗钉面
1655 恒等零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s495 stable 8514；他流入树归
因轮或轻覆盖；onblur flaky 键 3 例窗口观察维持；css 流
offset-position 刀实施中 + ZW-3 后续刀信号持续，落树后按
他流刀归因轮）。

工件：/tmp/s495/{s495-leg1,s495-leg2}.log、s495-{l1,l2}.norm、
l{1,2}.canon、s495.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、flips（零行）、gone.txt（零行）、new.txt
（5 族键）、t494.keys/t495.keys（T 集恢复对账）、
s494.stable.{keys,canon}（基线复用）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s495-bridge.log、
b495.list、b494-b495.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
