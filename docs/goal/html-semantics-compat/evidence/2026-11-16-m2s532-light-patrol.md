# M2 s532 — 守成巡查轻覆盖轮第二十二轮：pull 零携入零码变零换代（码态持位 2c8742499，runner 127567632 恒——leg1 Compiling ×6 = 桥跑侧 churn 同形产物同码，leg2 零重编），join stable 8509（8305P/198F/5T/1U）disagree 1 + l1only 1 + l2only 4；vs s531 stable **NEW 1 + GONE 2 + flips 0 位移全落带内**：NEW 1 = mpreload 哨 unsafe-url 键双腿同相回归入 stable（s531 单腿瞬态安全收敛）+ GONE 2 = 族双整文件键出位——**族键摆动谱 s527 异相镜像互换形第 2 次再现**（leg1 = detailed F + non-detailed 整文件 T + 零子测发射 恰 s527-leg1/s520-leg2 形；leg2 = detailed T + 4 子测 F + 整文件不发射 恰 s519-s530 稳态形），带内异相记账不升级；**非族非哨键零 NEW 零 GONE 零 flips 逐键恒等**；P 集双腿双 8307P 回归（s531 破轮后第 1 轮恢复）+ flaky 零新例第 21 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨 15 键双腿全 Pass 恢复（unsafe-url 键新 <4 轮窗 1/4 安全通过，时钟重顺延零翻红重新起算第 1 轮）** + 全桥 1671P/0F rc=0 清单 set 恒等 + focus-dynamic 收窗后第 14 轮零再现

日期：2026-11-16。承前：s531 收口 push 前置 pull 携入对
端两笔 docs-only（c44269d33 rendering-compat R5735
燃点轮 + 6ca9d9da7 editing-compat ihe6 收口，
`git diff 2c8742499..cf3db278f -- crates/ tests/` 零
码面）→ 本轮 pull 零携入，按守成巡查轻覆盖轮。

## 二进制 provenance

- 码态持位 **2c8742499**（HEAD cf3db278f = 本流 s531
  收口 docs-only 落基），runner **127567632 恒**（零
  换代；leg1 22:12:49 起腿 `Compiling` ×6 = 桥跑侧
  churn 同形产物同码，产物 size 与 mtime 代证——
  s526/s527 先例同形；leg2 零重编 22:16:15 背靠背）。

## 双腿 join（make testharness-html-semantics ×2）

- leg1 22:12:49–22:16:15 / leg2 22:16:15–22:18:49（零
  lull 背靠背），raw/norm 8513/8516（不对称 = 族键异
  相发射差：leg1 少 4 子测行、leg2 少整文件行，双腿
  各 +2 行双发发射 dup 照旧）——双腿零截断零真 hang
  （rc=2 惯例失败退出，时长正常）。管线全 LC_ALL=C
  固化。
- join：**stable 8509（8305P/198F/5T/1U）+ disagree 1
  + l1only 1 + l2only 4**：
  - disagree 1 = 族 detailed 整文件键（leg1 Fail /
    leg2 Timeout——异相槽位互换）。
  - l1only 1 = 族 non-detailed 整文件键（leg1 单腿
    Timeout 发射）；l2only 4 = 族 4 子测键（leg2 单腿
    Fail 发射）。
- **vs s531 stable（8510）：NEW 1 + GONE 2 + flips 0
  ——位移全落带内**：
  - NEW 1 = mpreload 哨 unsafe-url 键双腿同相回归入
    stable（P/P——s531 leg1 单腿瞬态 Fail 安全收敛，
    回归记账非 Pass 形首现）。
  - GONE 2 = 族双整文件键出位（detailed 异相 F/T 分
    歧 + non-detailed leg1 单腿 T 出位——s531 双整文
    件 T 形解体）。
  - **族键摆动谱：s527 异相镜像互换形第 2 次再现**
    ——leg1 = detailed **F** + non-detailed 整文件
    **T** + 零子测发射（恰 s527-leg1/s520-leg2 形）/
    leg2 = detailed **T** + 4 子测 **F** + 整文件不发
    射（恰 s519/s522/s524/s528-s530 稳态回归形）。
    带内异相形谱记账不升级（零新 Pass 形）。
  - **非族非哨键零 NEW 零 GONE 零 flips——轻覆盖轮
    守成零语义位移**（码态零变同构旁证）。
- **P 集账本口径双腿双 8307P 回归**（s531 l1 8306 破
  轮 1 轮后第 1 轮恢复；谱系累计与 s512 起持位带相连
  ）。
- **focus-dynamic 121 键 stable 全 Pass**：收窗后第
  14 轮零再现。
- 双发发射伪影 2 键照旧在册（exact-len + WINdow 双腿
  各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨收敛回归）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 21 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 61 轮零翻红（双键双腿 Pass）。
- **modulepreload 哨：15 键双腿全 Pass 恢复**——
  unsafe-url 键 P/P（s531 翻红后新 <4 轮窗 1/4 安全
  通过，时钟重顺延：零翻红重新起算第 1 轮；窗内再现
  即升级判定）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 55 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1671P/0F rc=0**（test 阶段 17.11s；
filtered 2405 持位）。测试清单 vs s531（b531.k）：
**1671 set 恒等，NEW 0 + GONE 0**——绿窗零破持位。

## 对端观察

- 起腿前窗净判定（22:12 load 2.00 回落态，对端
  corpus/rustc/cargo/zero-wpt-runner 零重腿，仅余
  rally query 守护 CI 轮询）——零等窗直接起腿；窗内
  双腿背靠背零外部交叠零截断。

## 结论

s532 收口（守成巡查轻覆盖轮第二十二轮）：pull 零携入
零码变零换代，码态持位 2c8742499（runner 127567632
恒，leg1 churn 同形产物同码，leg2 零重编）；vs s531
stable NEW 1 + GONE 2 + flips 0 位移全落带内——
mpreload 哨键收敛回归 + 族键 s527 异相镜像互换形第 2
次再现（带内形谱记账，零新 Pass 形），非族非哨键零
位移逐键恒等；P 集双腿双 8307P 回归；flaky 零新例第
21 轮，picker 增益键第 55 轮保持，onblur/dialogov/
waapi 三哨持位，mpreload 哨 15 键全 Pass 恢复时钟重
顺延；全桥 1671 set 恒等绿窗零破；focus-dynamic 收窗
后第 14 轮零再现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通
知）。

下一步：守成巡查（对账基线换 s532 stable 8509——族
键摆动观察维持：出位/回归/混合形按形谱记账、Pass 形
首现按 NEW 记账；全桥清单基线 b532.k 1671 持位；
flaky 键零新例时钟顺延；**mpreload 哨 <4 轮窗第 2/4
轮观察**——再现即升级判定；他流入树归因轮或轻覆盖
；对端刀信号持续：editing 流 B4-S2 残面刀（dwuf 八
变体端点规范化未接臂/during-and-after-dispatch 与克
隆门互斥 1F）+ rendering dialog 16 红块协同面 + css
文法刀后续刀（shapepos 残面队列：位置串化规范刀
~498F/round 列表完整建模/inline inherit/path() 命令
级插值/calc 代数插值）+ wptfyi-css 残面（jsanim flip
Steps(Start) 等）+ ZW-3 querySelector 施工刀，落树后
按他流刀归因轮，join 实测为准）+ 候门决项等待用户拍
板（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归 rendering 流记
账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s532/{s532-leg1,s532-leg2}.log、
s532-leg{1,2}.rc、leg{1,2}.s532.{start,end}、
s532-{l1,l2}.norm、l532-{1,2}.canon、s532.new（1）、
s532.gone（2）、s532.flips.txt（零行）、
s532.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s531.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用）、s532-bridge.log、s532-bridge.rc、b531.k（复用）
+ b532.k（set 恒等对账）+ b532.{new(0),gone(0)}、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
