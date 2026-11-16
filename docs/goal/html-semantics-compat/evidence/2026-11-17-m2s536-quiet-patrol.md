# M2 s536 — 守成巡查轻覆盖轮第二十五轮：pull 零携入（round start 23:42 上游持位 9c6463d3a）零码变零换代——码态持位 7e2c25ff9，runner 127567616 恒（leg1 Compiling ×6 engine 链 churn 同形产物同码 = s532–s535 同形，leg2 零重编）——join **stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0**；vs s535 stable **NEW 0 + GONE 0 + flips 0 逐键恒等——全键含族零位移轮**（族键稳态形第 2 轮双腿同相持位：detailed 整文件键 T + non-detailed 4 子测 F 双腿 IDENTICAL diff 实证，s527→s535 摆动谱收敛确认）——P 集账本口径双腿双 8307P 持位第 5 轮（双腿 raw 全同相 202F/8307P/6T/1U）+ flaky 零新例第 25 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨 15 键双腿全 Pass——unsafe-url 键顺延后第 2 轮零翻红** + 全桥 1671P/0F rc=0 清单 set 恒等 + focus-dynamic 收窗后第 18 轮零再现

日期：2026-11-17（轮跨午夜：round start 2026-11-16
23:42，起腿 2026-11-17 00:13）。承前：s535 收口后
pull 零携入（对端 ZW-3 d47 刀体仍未写树；对端 d47
准备相探针链在飞）→ 按守成巡查轻覆盖轮（第二十五
轮）。

## 起腿前窗净判定（撞重腿等净窗先例复用）

- 23:42 预检 load 3.95：**撞对端 d47 准备相重腿**
  （/tmp/d46-tgt runner input-events 全套 23:39 起
  ，test-guard 包裹 time-limit 1500；随后链式续跑
  testharness-html-semantics + testharness-selection
  ，各 time-limit 2700）——按 s531 先例 bounded
  poll 30s×N 等净窗，不叠跑（族键摆动形对时序敏
  感，叠跑会污染形谱记账）。
- 对端探针链 00:12 收线（d46-tgt 进程零，load
  2.16 回落态）→ 00:13 零等窗直接起腿；窗内双腿
  00:13:08–00:18:47 背靠背零外部交叠零截断零真
  hang（rc=2 惯例失败退出，时长正常）。

## 二进制 provenance

- 主刀面 = 零（pull 零携入，零码变）。
- 码态 = **7e2c25ff9 持位**；runner 二进制字节数
  **127567616 恒**（与 s533–s535 同代零换代）。
- leg1 00:13:08 起腿 `Compiling` ×6（product-version
  /net/engine/page-runtime/webview/wpt-runner）=
  s532–s535 同形 churn，产物同码；leg2 同二进制零
  重编。

## 双腿 join（HEAD 9c6463d3a 树 = 码态 7e2c25ff9，
make testharness-html-semantics ×2）

- leg1 00:13:08–00:16:21 / leg2 00:16:21–00:18:47
  （零 lull 背靠背），raw/norm 8516/8516（双腿各
  +2 行双发发射 dup：exact-len + WINdow 伪影照旧）
  。管线全 LC_ALL=C 固化（s488 修复版复用）。
- join：**stable 8514（8305P/202F/6T/1U）+ disagree
  0 + l1only 0 + l2only 0**。
- **vs s535 stable（8514）：NEW 0 + GONE 0 + flips
  0——全键含族逐键恒等**：
  - 族键稳态形第 2 轮双腿同相持位：detailed 整文
    件键 T stable 持位 + non-detailed 整文件键双腿
    零发射 + 4 子测 F 双腿发射；族行双腿 diff 实证
    IDENTICAL——s527→s532→s534 互换摆动谱 s535 首轮
    同相收敛后第 2 轮持位，收敛确认观察点。
  - **零新 Pass 形零新 Fail 形**；非族非哨键零位
    移逐键恒等。
- **P 集账本口径双腿双 8307P**（raw Pass 行口径，
  持位第 5 轮）；双腿 raw 形全同相 202F/8307P/6T
  /1U（s535 首次双腿 raw 全同相后第 2 轮持位）。
- **focus-dynamic 121 键 stable 全 Pass**：收窗后
  第 18 轮零再现。
- 双发发射伪影 2 键照旧在册（exact-len + WINdow 双
  腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨顺延后第 2 轮）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label
  双键红池照常）——flaky 键（checkbox/radio
  following-label 双行）双腿双 Pass：零新例第 25 轮
  （s512 时钟顺延）。
- dialog-overlay 哨第 65 轮零翻红（双键双腿 Pass）。
- **modulepreload 哨：15 键双腿全 Pass——unsafe-url
  键时钟重顺延后第 2 轮零翻红**（窗内再现即升级判
  定）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 59
  轮保持）/ checkmark Fail（带形不变）。哨复核全
  程 live 状态 join。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行）：
**1671P/0F rc=0**（test 阶段 17.68s；filtered 2405
持位；Compiling ×3 test profile churn 同 s533/s535
形）。测试清单 vs s535（b535.k）：**1671 set 恒等
，NEW 0 + GONE 0**——绿窗零破。

## 对端观察

- 对端 d47 准备相活跃：23:39–00:12 三腿探针链
  （input-events 全套 → testharness-html-semantics
  → testharness-selection，全 test-guard 包裹
  time-limit 1500/2700，/tmp/d46-tgt 二进制）——d47
  刀体（querySelector 施工刀，4 文件 +95/−2）仍未
  commit 写树，R5739 首号预计燃点续挂（对端渲染流
  记账）。
- 对端刀信号持续：editing 流 B4-S2 残面刀 + rendering
  dialog 16 红块协同面 + css 文法刀后续刀（shapepos
  残面队列）+ wptfyi-css 残面——本轮均未入树。

## 结论

s536 收口（守成巡查轻覆盖轮第二十五轮，零携入零码
刀）：码态持位 7e2c25ff9、runner 127567616 恒；
join stable 8514（8305P/202F/6T/1U）disagree 0 +
l1only 0 + l2only 0——vs s535 NEW 0 + GONE 0 +
flips 0 全键含族逐键恒等（族键稳态形第 2 轮双腿
同相持位，族行双腿 diff IDENTICAL 实证，摆动谱收
敛确认）；P 集双腿双 8307P 持位第 5 轮；flaky 零
新例第 25 轮，picker 增益键第 59 轮保持，onblur/
dialogov/waapi 三哨持位，mpreload 哨顺延后第 2 轮
零翻红；全桥 1671 set 恒等绿窗零破；focus-dynamic
收窗后第 18 轮零再现。撞重腿等净窗先例复用（23:42
预检撞对端 d47 探针链，bounded poll 30 分钟等净窗
00:13 起腿零交叠）。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，
不重复通知）。

下一步：守成巡查（对账基线换 s536 stable 8514——
族键稳态形收敛观察维持：若再现异相/出位按形谱记
账、Pass 形首现按 NEW 记账；全桥清单基线 b536.k
1671 持位；flaky 键零新例时钟顺延；**mpreload 哨
顺延后窗内第 3 轮观察**——再现即升级判定；他流入
树归因轮或轻覆盖；对端刀信号持续：ZW-3 d47 刀体
（querySelector 施工刀）落树后按他流刀归因轮 +
editing 流 B4-S2 残面刀 + rendering dialog 16 红块
协同面 + css 文法刀后续刀（shapepos 残面队列）+
wptfyi-css 残面，join 实测为准）+ 候门决项等待用户
拍板（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归 rendering 流
记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s536/{s536-leg1,s536-leg2}.log、
s536-leg{1,2}.rc、leg{1,2}.s536.{start,end}、
window.s536.judge、s536-{l1,l2}.norm、l536-{1,2}
.canon、s536.new（零行）、s536.gone（零行）、
s536.flips.txt（零行）、s536.{l1u,l2u,s1,s2,k1,k2,
both.keys,leg1only.keys,leg2only.keys,disagree.keys
,stable.keys,stable.canon}、s535.stable.{keys,canon}
（基线复用）+ s477-{onblur,dialogov,mpreload,waapi}
.keys（哨键组复用）+ fd121.keys（focus-dynamic 组
复用）+ s536-bridge.log、s536-bridge.rc、b535.k
（复用）+ b536.k（set 恒等对账）、run-legs.sh、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
