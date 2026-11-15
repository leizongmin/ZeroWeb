# M2 s503 — 守成巡查轻覆盖轮第十一轮（零入树同码态同二进制，连续第十一轮）：onblur following-label flaky 键**第三例**（leg2 瞬态 F，s489 leg1 → s493 leg2 → s503 leg2，**3 例窗口满**）+ 族键双腿同相位回位（上轮预期兑现）——vs s502 stable GONE 1（flaky 键）+ NEW 5（全族键）+ flips 0 非族非哨逐键恒等，stable 8513 = s501 stable − flaky 键，P 集 leg1 8307 持位第 25 轮/leg2 8306P 瞬态 -1 单键 + 四观察哨零新翻红 + 全桥绿窗 1659P/0F 清单恒等

日期：2026-11-16。承前：s502 下一步「守成巡查（对账基线
换 s502 stable 8509——族键出位形、s500 构成同款；族键相
位谱 1 轮周期往返摆观察维持——下轮预期回位入位形；……
对端 ihtml 轴一拆分插入刀信号持续 + 对端 ZW-3
querySelector 施工刀信号持续，落树后按他流刀归因轮）」。
开工 `git pull --rebase`「已经是最新的」（HEAD 25bc5b01b
= s502 收口提交；s502 收口 pull 携入对端 fe23e88b9 亦
docs-only）；码态核查 `git diff cfd1a06af..HEAD --
crates/ apps/` 零行——ihtml 轴一拆分插入刀与对端 ZW-3
querySelector 施工面均未落树，无归因对象 → 轻覆盖轮连续
第十一轮。

## 对端观察

起腿前对端一笔 ZW-3 d42 input-events 探查在飞（04:31–
04:41 完成）+ mykernel 租户 `_guarded-check` 构建
（test_task 驱动，rustc 140% CPU）并行——轮询至
04:46:55 双清场（对端零进程、load 回落 2.00）起 leg1；
leg2 起腿时 peer procs = 0。**双腿均为干净窗零进程背
景**（load 2.00–2.7 为 host 侧残余，容器视角；负载归因
素材在案）。本轮对端源码刀零落树（双克隆静默）。

## 二进制 provenance

- runner size **126999128** 恒（s500 换代值持续，刀码同
  二进制）；leg1/leg2 make 重链同内容形态。
- 码态 = cfd1a06af（HEAD 25bc5b01b docs-only）。

## 双腿 join（HEAD 25bc5b01b 树 = 码态 cfd1a06af，make testharness-html-semantics ×2）

- leg1 8517 raw（8308P/202F/6T/0U/1U）/ leg2 8517 raw
  （8307P/203F/6T/0U/1U）——双腿零截断零真 hang（rc=2 惯
  例失败退出）。本轮管线全 LC_ALL=C 固化。
- join：**stable 8513（8304P/202F/6T/1U）+ disagree 1 +
  leg-only 双零**——唯一 disagree = onblur checkbox
  following-label 键（leg1 Pass / leg2 Fail）。
- **vs s502 stable（8509 = 8305P/198F/5T/1U）：GONE 1 +
  NEW 5 + flips 0**：
  - **GONE 1 = onblur checkbox following-label 键 leg2 瞬
    态 F 退 stable——第三例**（换腿序列 s489 leg1 →
    s493 leg2 → s503 leg2，**3 例窗口满**）。本轮双腿干
    净窗零进程背景，负载归因不成立（s493 第二例同判定）；
    摇摆维持 P↔F 交替非单向迁移——按在册政策维持观察不
    升级 bug 排查（升级后政策：3 例窗口观察频率；窗口满
    后新例即第 4 例，频率升级判定点临近）。
  - **NEW 5 = 全族键双腿同相位回位**（optional 4 子测
    Fail + detailed 整文件 Timeout，s501 形恢复）——上轮
    「下轮预期回位入位形」兑现；族相位谱：s500 出 →
    s501 入 → s502 出 → s503 入，1 轮周期往返摆维持，不
    升级。
- stable 8513 与 s501 stable（8514）comm 恰 1 行差 = 该
  flaky 键——非族非哨键逐键恒等零位移。
- **P 集账本口径 leg1 8307P 持位第 25 轮 / leg2 8306P**
  （瞬态 -1 单键，s493 同款形）；stable 机械口径 8304P。
  双发发射伪影 2 键照旧在册（wrapping-transformation
  exact-len + execution-timing/149.html WINdow，双腿各 2
  dup）。T 集 6 恒等。

## 观察哨复核

- onblur 哨组 10 键：**leg1 8P/2F / leg2 7P/3F**（
  parent-label 双键双腿 F 红池照常；following-label
  checkbox 键 leg2 瞬态 F 第三例，见上）。
- dialog-overlay 哨第 32 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 35 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 26 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1659P/0F rc=0**（test 阶段 18.45s；
filtered 2394 恒）。测试清单 vs s502：**set 恒等 1659，
GONE 0**（码态未变零增长预期兑现）。

## 结论

s503 收口（守成巡查轻覆盖轮第十一轮，零源码改动）：零入
树同码态同二进制（cfd1a06af 持位，runner 126999128 恒），
flaky 键第三例入窗窗口满（干净窗负载归因不成立，摇摆非
单向维持观察不升级），族键双腿同相位回位（s501 形恢复，
上轮预期兑现，1 轮周期往返摆维持），stable 8513 = s501
stable − flaky 键零其他位移，P 集 leg1 8307 持位第 25
轮，picker 增益键第 26 轮保持，四观察哨零新翻红，全桥绿
窗钉面 1659 恒等零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s503 stable 8513——族键入
位形 + flaky 键出位形；flaky 键 3 例窗口已满——后续新例
即第 4 例按频率升级判定，摇摆形态（单腿瞬态 P↔F 非单向
vs 双腿同 F/单向迁移）届时定性；族键相位谱 1 轮周期往返
摆观察维持——下轮预期出位形；他流入树归因轮或轻覆盖；对
端 ihtml 轴一拆分插入刀信号 + 对端 ZW-3 querySelector 施
工刀信号持续，落树后按他流刀归因轮）+ 候门决项等待用户
拍板（video P1+P2 立项 / 簇 D A2 立项 / s137）；rendering
流 dialog 16 红块协同面（归 rendering 流记账）。门开随批
落预案不变：簇 D 开 → customizable-select-styles.css ×47
随批落。

工件：/tmp/s503/{s503-leg1,s503-leg2}.log、
s503-{l1,l2}.norm、l{1,2}.canon、flips.txt（零行）、
gone.txt（1 flaky 键）、new.txt（5 族键）、s503.{l1u,l2u,
s1,s2,k1,k2,both.keys,leg1only.keys,leg2only.keys,
disagree.keys,stable.keys,stable.canon}、s502.stable.{keys,
canon}（基线复用）+ s501.stable.keys（s501 形对账）、
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复用；
onblur 键组含 s477 时代 \tPass/\tFail 状态后缀，复核须剥
后缀直查 canon）、s503-bridge.log、b502.k（复用）+
b503.k（清单恒等对账）、norm.sh/canon.sh/join.sh（s488
修复版管线复用，全 LC_ALL=C）。
