# M2 s487 — 守成巡查轻覆盖轮（零入树同码树，s486 归因后首静默轮）：双腿相位摆谱（leg1 8513 = 8307P/198F/7T/1U 族双整文件 T 形 + leg2 8516 = 8307P/202F/6T/1U 族混合形）零截断，join stable 8510（8305P/198F/6T/1U）vs s486 stable GONE 1 + NEW 0 + flips 1 全族整文件键相位谱——非族键逐键零差，P 集 8307 持位 + picker 增益键第 10 轮保持 + 五观察哨零新翻红 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-15。承前：s486 下一步「守成巡查（他流入树归
因轮或轻覆盖）」。开工 `git pull --rebase` 携入一笔 c7b7e9574
（rendering 流 R5664 docs，PR #305 义务销账账本，零代码）；
码态核查 `git diff 492a0f2e0..HEAD -- crates/ apps/` 零行——
ZeroWeb-3 三文件刀 per R5664 施工面增长（part01.js numstat
+146→+185）仍未落树，无归因对象 → 轻覆盖轮。

## 对端错峰观察（本流工序让位）

起腿前对端 rendering 流 R5665 easing 刀 corpus A/B 义务三跑
在飞（ZeroWeb clone，`reftest-upstream` ×3 串行，22:17-22:21
三跑收口 stable-3run 落盘）——本流双腿错峰等其收口后起跑，
避免向其 A/B 确定性窗口叠加负载（对端协议按瞬态交集口径核
算， solo 判据在册）。另察一次短促 `zw-runner-b2-wip
testharness-input-events`（1800s guard 包裹，分钟级收口，
非本树）后机器静默（load 1.03）方起腿。

## 二进制 provenance

- leg1 起跑 make 重建（`Compiling` ×6、`Finished release
  35.11s`，runner mtime 22:24:28，size 126909176 与 s486 逐字
  节同值——码态未变下的同内容重链，s486「重建现象」本轮带
  可见 Compiling 行变体）；leg2 零 `Compiling` 同二进制。
- 码态核查：`git diff 492a0f2e0..HEAD -- crates/ apps/` =
  零行，HEAD = 7cf312963（s486 收口 docs + 本轮无码）。

## 双腿 join（7cf312963 树 = 码态 492a0f2e0，make testharness-html-semantics ×2）

- leg1 **8513（8307P/198F/7T/1U）**：族 leg1 形本轮为双整文
  件 T（optional + detailed 整文件键均 Timeout，7T = 非族 5 +
  族 2——s486 的「optional T + detailed F」摆位新相）；
  leg2 **8516（8307P/202F/6T/1U）**：族混合形（detailed 整文
  件 T + optional 4 子测 F）。双腿零截断零真 hang（leg rc=2
  惯例失败退出，guard 未触发）。
- join：**stable 8510（8305P/198F/6T/1U）disagree 0 +
  leg-only 1/4**——leg-only 5 键全部族键（optional 整文件键
  leg1 独有 T；optional 4 子测键 leg2 独有 F，整文件 vs 子测
  发射形差伪影，s485 谱同款）。
- **vs s486 stable（8511 = 8305P/199F/6T/1U）：GONE 1 + NEW 0
  + flips 1**——GONE = optional 整文件键（T，双腿发射形差不
  入 stable）；flip = detailed 整文件键 Fail→Timeout（族整
  文件键 T↔F 相位谱 s464 起在册）；**非族键逐键零差**（非族
  T 键集 5/5 恒等 comm 复核）。
- **P 集账本口径 8307 持位**（双腿双 8307P，第 9 轮同形）；
  stable 机械口径 8305P 八轮同形。双发发射伪影 2 键照旧在册
  （textarea wrapping round-trips + script 149 for=' WINdow，
  sort -u 去重吸收）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 stable 10 键全在册 8P/2F（following-label 双键
  Pass 持位，红池 parent-label 2F 照常）。
- dialog-overlay 哨第 16 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 19 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 10 轮保
  持）/ checkmark Fail（双相位带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码树旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 18.27s；filtered 2389 同值）。
测试清单 vs s486：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 操作坑（本轮发现，已沉淀 learning）

- **locale collation 坑**：本会话 shell zh_CN.UTF-8，管线
  `LC_ALL=C cut | sort` 前缀只及首段，`comm` 按 zh_CN 规则判
  上一轮产物乱序拒绝对账；全链 `LC_ALL=C` 重跑（含对上一轮
  stable keys 先 C 下 sort -u 再规范化）后恢复。已沉淀
  docs/learnings/platform/2026-11/2026-11-15-patrol-join-lc-all-c-collation.md。
- **join.sh stable.canon 生成 bug**：脚本末行 `grep -Fxf
  stable.keys l1u` 整行匹配永空（key 行 vs key\tstatus 行）
  ——s486 轮该文件系事后单独生成，脚本从未产出过；本轮改
  awk key 集过滤生成，避免下轮复踩。

## 结论

s487 收口（守成巡查轻覆盖轮，零源码改动）：零入树同码树
（492a0f2e0 持位），双腿相位摆谱族带内（GONE 1 + NEW 0 +
flips 1 全族整文件键相位谱），非族键逐键零差，P 集 8307 持
位，picker 增益键第 10 轮保持，五观察哨零新翻红，全桥绿窗
钉面 1655 恒等零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

工件：/tmp/s487/{s487-leg1,s487-leg2}.log、s487-{l1,l2}.norm、
l{1,2}.canon、s487.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、s487.kts、s486.{stable.keys,stable.uk,kts2,T.keys}
（s486 产物 C 下再规范化副本）、flips.txt、gone.txt、new.txt、
s487.T.keys、s477-{onblur,dialogov,mpreload,waapi}.keys（哨
键组复用）、s487-bridge.log、b487.list、b486-b487.diff（空 =
清单恒等）、norm.sh/canon.sh/join.sh（s478 沉淀管线复用）。
