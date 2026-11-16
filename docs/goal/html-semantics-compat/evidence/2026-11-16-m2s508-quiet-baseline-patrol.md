# M2 s508 — 守成巡查轻覆盖轮第十三轮（零入树同码态同二进制，连续第十三轮；首跑遇对端 corpus 互踩杀腿重起）：**族键两形异相位出位**（leg1 子测形/leg2 整文件形，s505 构成同款）——vs s507 stable GONE 5（全族键出位）+ NEW 1（flaky 键双腿双 Pass 回位，非新例第 5 例判定时钟不变）+ flips 0 非族非哨逐键恒等，stable 8509，P 集双腿双 8307P（leg1 持位第 30 轮）+ 四观察哨零新翻红 + 全桥绿窗 1660P/0F 清单恒等

日期：2026-11-16。承前：s507 下一步「守成巡查（对账基线
换 s507 stable 8513——……他流入树归因轮或轻覆盖）」。
s507 收口 push 后 pull --rebase 携入对端 rendering-compat
R5702（1d87b4a30，docs-only）；码态核查
`git diff c6b73888a..HEAD -- crates/ apps/ engines/` 零
行——对端 ZW-3 querySelector 施工刀 / css 文法刀后续刀
/ editing 流 generation-aware 挂起刀均未落树，无归因对
象 → 轻覆盖轮连续第十三轮。

## 对端观察（含首跑互踩插曲）

- **首跑互踩**：s508 首次 leg1 起腿（08:40:57）恰逢对
  端渲染流 `make reftest-upstream` corpus 腿（zero-wpt
  -runner reftest-upstream 445% CPU）+ ZW-3 d43 独立
  target 构建风暴（rustc ×6，load 峰值 30.66）——重腿
  互踩场景（在册教训），立即杀首跑 leg1（relink 阶段，
  零产出污染），轮询至 09:02 双清场（peer procs 0 /
  load 1.86）重起。
- **leg1 09:02:26–09:05:13 干净窗零进程背景**；
  **leg2 09:05:44–09:08:53 期间对端 zw-runner-d43t
  input-events 单测探针（110% CPU）+ mykernel QEMU 单
  核在飞**——非构建风暴级；本轮族键异相位为发射形漂
  移（非失败新增）、flaky 键双 Pass 无瞬态，负载归因
  无对象，素材在案。
- 本轮对端源码刀零落树（双克隆静默）。

## 二进制 provenance

- runner size **127415952** 恒（s507 换代值持续，刀码同
  二进制）；leg1 make relink 同内容形态。
- 码态 = c6b73888a（HEAD 3cc6841c1 docs-only）。

## 双腿 join（HEAD 3cc6841c1 树 = 码态 c6b73888a，make testharness-html-semantics ×2）

- leg1 8517 raw（8308P raw）/ leg2 8517 raw（8308P
  raw）——双腿零截断零真 hang（rc=2 惯例失败退出）。
  本轮管线全 LC_ALL=C 固化。
- norm leg1 8516 / leg2 8513（leg2 -4 optional 子测发
  射 +1 optional 整文件发射，净 -3）；**P 集账本口径双
  腿双 8307P**（leg1 持位第 30 轮 / leg2 回位，raw
  8308 → norm 8307 两腿同形各丢 1 行 bare 发射）。
- join：**stable 8509（8305P/198F/5T/1U）+ disagree 1 +
  l1only 4 + l2only 1**——唯一 disagree = 族 detailed
  整文件键（leg1 Timeout / leg2 Fail）；l1only = 族
  optional 4 子测键（leg2 零发射）；l2only = 族
  optional 整文件键（leg1 零发射）。
- **vs s507 stable（8513 = 8304P/202F/6T/1U）：GONE 5 +
  NEW 1 + flips 0**：
  - **GONE 5 = 全族键两形异相位出位**：leg1 子测形
    （detailed 整文件 Timeout + optional 4 子测 Fail）
    vs leg2 整文件形（detailed 整文件 Fail + optional
    整文件 Timeout）——s505 构成同款（两腿完全异相位，
    stable 双零发射）。**「出位形重现再升观察」兑现**
    ：族相位谱 s505 半入位形 → s506 入 → s507 入 →
    **s508 出（两形异相位）**——发射形谱修订：子测形
    /整文件形/零发射形漂移无固定周期（回位两轮后出
    位，摆动假说降权维持），全落族带内发射形谱，观察
    维持不升级。
  - **NEW 1 = onblur flaky 键双腿双 Pass 回位入
    stable——非新例**（零 F 发射）：第 5 例判定时钟不
    变（上例 s507 leg2；s511 前再现 F 即间隔 <4 轮，
    按频率升级判定兑现升 bug 排查）。
  - 非族非哨键逐键恒等；码态零入树，**刀致语义位移
    零**。
- 双发发射伪影 2 键照旧在册（wrapping-transformation
  exact-len + execution-timing/149.html WINdow，双腿各
  2 dup）。T 集 5（stable 口径，族 detailed 出位后）；
  U 集 1 恒等。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键双腿双 Pass 回位（见上，非新
  例）。
- dialog-overlay 哨第 37 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 40 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 31 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1660P/0F rc=0**（test 阶段 33.94s；
filtered 2399 恒）。测试清单 vs s507：**set 恒等 1660，
GONE 0**（码态未变零增长预期兑现）。

## 结论

s508 收口（守成巡查轻覆盖轮第十三轮，零源码改动）：零
入树同码态同二进制（c6b73888a 持位，runner 127415952
恒），族键两形异相位出位（s505 构成同款，「出位形重现
再升观察」兑现，发射形谱漂移无固定周期——摆动假说降权
维持），flaky 键双腿双 Pass 回位非新例（第 5 例判定时
钟不变），P 集双腿双 8307（leg1 第 30 轮），picker 增
益键第 31 轮保持，四观察哨零新翻红，全桥绿窗钉面 1660
恒等零破。首跑互踩插曲按在册教训处置（杀腿重起，零产出
污染）。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s508 stable 8509——族键出
位形两形异相位；族键发射形谱漂移观察维持——子测形/整
文件形/零发射形无固定周期，回位或出位均按形谱记账，全
部落族带内不升级；flaky 键第 5 例判定时钟——s511 前再
现 F 即间隔 <4 轮升 bug 排查；他流入树归因轮或轻覆盖；
对端 ZW-3 querySelector 施工刀信号 + css 文法刀后续刀
信号 + editing 流 generation-aware 挂起刀信号持续，落
树后按他流刀归因轮，join 实测为准）+ 候门决项等待用户
拍板（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归 rendering 流记
账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s508/{s508-leg1,s508-leg2}.log、
s508-leg{1,2}.rc、leg{1,2}.{start,end}、
s508-{l1,l2}.norm、l{1,2}.canon、flips.txt（零行）、
gone.txt（5 族键）、new.txt（1 flaky 键）、
s508.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s507.stable.{keys,canon}（基线复用）、
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 canon）、s508-bridge.log、b507.k
（复用）+ b508.k（清单恒等对账）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
