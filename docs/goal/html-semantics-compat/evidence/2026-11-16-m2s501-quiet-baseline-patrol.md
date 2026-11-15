# M2 s501 — 守成巡查轻覆盖轮第九轮（零入树同码态同二进制，连续第九轮）：**双腿完全一致**第九次（双 8517 raw = 双 8308P/202F/6T/1U，disagree 0 + leg-only 双零），族键双腿同相位回位入 stable——vs s500 stable GONE 0 + NEW 5（全族键：optional 4 子测 Fail + detailed 整文件 Timeout，s497 形恢复，族相位谱 1 轮周期往返摆 s500 出 → s501 入）+ flips 0 非族逐键恒等，onblur flaky 键双腿 Pass 3 例窗口零新例连续第七轮，P 集 8307 持位第 23 轮 + picker 增益键第 24 轮保持 + 四观察哨零新翻红 + 全桥绿窗 1659P/0F 清单恒等

日期：2026-11-16。承前：s500 下一步「守成巡查（对账基线
换 s500 stable 8509——逐键恒等形；他流入树归因轮或轻覆
盖；……对端 editing 流下一步 ihtml 轴一拆分插入刀信号持
续（对端申报待入账），落树后按他流刀归因轮）」。开工
`git pull --rebase`「已经是最新的」；码态核查 `git diff
cfd1a06af..HEAD -- crates/ apps/` 零行——ihtml 轴一拆分
插入刀（对端申报「下一步入账」）与其他对端刀均未落树，无
归因对象 → 轻覆盖轮连续第九轮。

## 对端观察

起腿前对端一笔 mykernel 域有界 clippy/check 探查在飞
（test-guard/systemd-run 包裹，python3 scripts/test_task.py
驱动），等待约 2.5 分钟干净窗（对端零构建进程、load 0.56
回落）起腿——负载干扰零。双克隆本轮静默零码刀。

## 二进制 provenance

- runner size **126999128** 恒（s500 换代值，刀码同二进制）。
- leg1 起跑 make 重链（`Compiling` ×6 同内容重链形态，s495/
  s497 leg1 先例同款）；leg2 零 `Compiling` 同二进制。
- 码态 = cfd1a06af（HEAD bb5cfc9f7 docs-only）。

## 双腿 join（bb5cfc9f7 树 = 码态 cfd1a06af，make testharness-html-semantics ×2）

- leg1/leg2 **均 8517 raw（双 8308P/202F/6T/1U）——双腿完
  全一致 disagree 0 + leg-only 双零**（s479/s484/s486/s488/
  s490/s492/s495/s499-骨架以来第九次全同形态——本轮族键
  亦双腿同相位，s499 后首次族键入一致形）。双腿零截断零
  真 hang（rc=2 惯例失败退出）。本轮管线全 LC_ALL=C 固化。
- join：**stable 8514（8305P/202F/6T/1U）**——族键全数入
  stable。
- **vs s500 stable（8509 = 8305P/198F/5T/1U）：GONE 0 +
  NEW 5 + flips 0——非族键逐键恒等**。NEW 5 全族键双腿同
  相位入 stable：optional 4 子测键 Fail（双腿子测发射形）
  + detailed 整文件键 Timeout（双腿 T 同相位）——**s497
  形恢复**（族相位谱在册维持观察：s494 出 → s495 入 →
  s496 出 → s497 入 → s498 整文件形双腿并现 → s499 整文
  件形第二连轮 → s500 两形 → s501 入，1 轮周期往返摆，
  不升级）。
- **P 集账本口径双腿双 8307P 持位第 23 轮**（stable 机械
  口径 8305P）。双发发射伪影 2 键照旧在册（8517 raw →
  8515 unique 口径内 dedup：wrapping-transformation.window.js
  exact-len + execution-timing/149.html WINdow）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 **10 键双腿双 8P/2F**（following-label 双键
  双腿 Pass 持位，parent-label 2F 红池照常）——flaky
  checkbox following-label 键双腿 Pass，**3 例窗口零新例
  连续第七轮**（当前 2 例：s489 leg1 / s493 leg2）。
- dialog-overlay 哨第 30 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 33 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 24 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1659P/0F rc=0**（test 阶段 16.91s；filtered 2394 恒）。
测试清单 vs s500：**set 恒等 1659，GONE 0**（码态未变零增
长预期兑现）。

## 结论

s501 收口（守成巡查轻覆盖轮第九轮，零源码改动）：零入树
同码态同二进制（cfd1a06af 持位，runner 126999128 恒），双
腿完全一致第九次（族键首次入一致形），s500 两形发射形相
位带内往返收案（GONE 0 + NEW 5 族键回位 + flips 0 非族逐
键恒等），族键全数入 stable 8514（s497 形恢复），P 集 8307
持位第 23 轮，flaky 键 3 例窗口零新例连续第七轮，picker
增益键第 24 轮保持，四观察哨零新翻红，全桥绿窗钉面 1659
恒等零破。挂用户门维持：GB-20261114-VIDEO + GB-20261031
（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s501 stable 8514——族键入
位形；他流入树归因轮或轻覆盖；onblur flaky 键 3 例窗口观
察维持；族键相位谱 1 轮周期往返摆观察维持不升级；对端
ihtml 轴一拆分插入刀信号持续 + 后续刀信号持续，落树后按
他流刀归因轮）+ 候门决项等待用户拍板（video P1+P2 立项 /
簇 D A2 立项 / s137）；rendering 流 dialog 16 红块协同面
（归 rendering 流记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s501/{s501-leg1,s501-leg2}.log、s501-{l1,l2}.norm、
l{1,2}.canon、flips.txt（零行）、gone.txt（零行）、new.txt
（5 族键）、s501.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s500.keys.s/s501.keys.s（跨轮对账）、s500.stable.{keys,
canon}（基线复用）+ s500canon.s/s501canon.s（flips 对账）、
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复用）、
s501-bridge.log、b500.sorted/b501.sorted（清单恒等对账）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全 LC_ALL=C）。
