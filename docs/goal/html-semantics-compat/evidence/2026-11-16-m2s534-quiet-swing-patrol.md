# M2 s534 — 守成巡查轻覆盖轮第二十三轮：pull 零码刀零码变零换代（携入 909f4a8dc rendering-compat R5737 对账 docs-only +7 行，零 `.rs`/`.toml` 差）——码态持位 7e2c25ff9，runner 127567616 恒（leg1 Compiling ×6 engine 链 churn 同形产物同码 = s532/s533 同形，leg2 Finished 0.17s 零重编）——join **stable 8510（8305P/198F/6T/1U）disagree 0 + l1only 1 + l2only 4**；vs s533 stable **NEW 1 + GONE 1 + flips 0 位移全落族内**：NEW 1 = 族 detailed 整文件键 T/T 双腿同相入 stable（s533 disagree F/T 异相位收敛，Timeout 形零新 Pass 形带内记账不升级）+ GONE 1 = 族 non-detailed 整文件键出位（s533 NEW 键摆出）——**族键 stable 位互换形第 3 次**（s527→s532→s534；leg1 = 双整文件 T 形 = s531/s533-leg2 形，leg2 = detailed T + non-detailed 4 子测 F 发射、整文件键不发射 = s519–s530 稳态形），带内异相记账不升级——非族非哨键零 NEW 零 GONE 零 flips 逐键恒等；P 集账本口径双腿双 8307P 持位第 3 轮 + flaky 零新例第 23 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨 15 键双腿全 Pass——unsafe-url 键 <4 轮窗 3/4 安全通过过窗时钟重顺延** + 全桥 1671P/0F rc=0 清单 set 恒等 + focus-dynamic 收窗后第 16 轮零再现

日期：2026-11-16。承前：s533 收口 push 后本轮 pull 携入
对端 **909f4a8dc**（docs(goal): rendering-compat R5737
守成巡查轮，`docs/goal/rendering-compat.md` 单文件
+7 行，`git diff e07ae06d3..909f4a8dc -- '*.rs'
'*.toml' '*.kt' '*.js'` 空 = 零码刀）→ 按守成巡查轻
覆盖轮（第二十三轮）。

## 二进制 provenance

- 主刀面 = 零（pull 携入 docs-only，零码变）。
- 码态 = **7e2c25ff9 持位**；runner 二进制字节数
  **127567616 恒**（`ls -l` 实测，与 s533 同代零换
  代）。
- leg1 23:16:39 起腿 `Compiling` ×6（product-version
  /net/engine/page-runtime/webview/wpt-runner）=
  s532/s533 同形 churn，产物同码（ Finished release
  36.88s 后 runner 字节数不变）；leg2 同二进制
  `Finished in 0.17s` 零重编。

## 双腿 join（HEAD 4eca63c46 树 = 码态 7e2c25ff9，
make testharness-html-semantics ×2）

- leg1 23:16:39–23:19:58 / leg2 23:19:58–23:22:28（
  零 lull 背靠背），raw/norm 8513/8516（双腿各 +2 行
  双发发射 dup：exact-len + WINdow 伪影照旧；l1 uniq
  8511 = stable 8510 + l1only 1，l2 uniq 8514 =
  stable 8510 + l2only 4，账实相符）——双腿零截断零
  真 hang（rc=2 惯例失败退出，时长正常）。管线全
  LC_ALL=C 固化（s488 修复版 norm/canon/join 复用）。
- join：**stable 8510（8305P/198F/6T/1U）+ disagree
  0 + l1only 1 + l2only 4**。
- **vs s533 stable（8510 同数异形）：NEW 1 + GONE 1
  + flips 0——位移全落族内**：
  - NEW 1 = 族 detailed 整文件键
    （select-home-end-pagedown-pageup-detailed
    .optional.html）T/T 双腿同相入 stable——s533
    disagree（F/T）槽位收敛；Timeout 形零新 Pass
    形，带内记账不升级。
  - GONE 1 = 族 non-detailed 整文件键（select-
    home-end-pagedown-pageup.optional.html）出位
    （s533 NEW 键摆出）：leg1 = 整文件 T + 零子测
    发射；leg2 = 4 子测 F 发射（Home/End/PageUp/
    PageDown）、整文件键不发射——双腿签名异相，
    l1only（整文件键）+ l2only（4 子测键）。
  - **stable 位互换形第 3 次**（s527→s532→s534）：
    detailed 入、non-detailed 出；leg1 形 = 双整文
    件 T（s531/s533-leg2 形），leg2 形 = detailed
    T + 4 子测 F 稳态形（s519–s530 形）——同一
    二进制双腿异相 = 环境调度摆动带内，非刀致。
  - **非族非哨键零 NEW 零 GONE 零 flips——逐键恒
    等**。
- **P 集账本口径双腿双 8307P**（raw Pass 行口径，
  持位第 3 轮）；leg1 raw 形 198F/8307P/7T/1U、leg2
  raw 形 202F/8307P/6T/1U（+4F = l2only 子测键，
  +1T/−1T = l1only 整文件键，全对账相符）。
- **focus-dynamic 121 键 stable 全 Pass**：收窗后
  第 16 轮零再现（组 = focus-dynamic 文件名 121 键：
  type-change-on-blur 80 + type-change 40 +
  time-focus-dynamic-value-change 1）。
- 双发发射伪影 2 键照旧在册（exact-len + WINdow 双
  腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨过窗顺延）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label
  双键红池照常）——flaky 键（checkbox following-
  label）双腿双 Pass：零新例第 23 轮（s512 时钟顺
  延）。
- dialog-overlay 哨第 63 轮零翻红（双键双腿 Pass）。
- **modulepreload 哨：15 键双腿全 Pass——unsafe-url
  键 <4 轮窗 3/4 安全通过，过窗时钟重顺延**（s531
  翻红后第 3 轮窗内零再现，零翻红重新起算；窗内再
  现即升级判定）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 57
  轮保持）/ checkmark Fail（带形不变）。哨复核全程
  用 live 状态 join（哨键文件状态列不复读）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行）：
**1671P/0F rc=0**（test 阶段 18.33s；filtered 2405
持位；Compiling 0 = test profile 零重编）。测试清单
vs s533（b533.k）：**1671 set 恒等，NEW 0 + GONE 0**
——绿窗零破；shape_margin 钉测试在列全绿。

## 对端观察

- 起腿前窗净判定（23:16 load 1.18，对端 corpus/
  rustc/cargo/zero-wpt-runner 零重腿零等窗直接起腿
  ）；窗内双腿背靠背零外部交叠零截断。
- 对端刀信号持续：editing 流 B4-S2 残面刀（dwuf 八
  变体端点规范化未接臂/during-and-after-dispatch
  与克隆门互斥 1F）+ rendering dialog 16 红块协同面
  + css 文法刀后续刀（shapepos 残面队列：位置串化
  规范刀 ~498F/round 列表完整建模/inline inherit/
  path() 命令级插值/calc 代数插值）+ wptfyi-css 残
  面（jsanim flip Steps(Start) 等）+ ZW-3
  querySelector 施工刀——本轮均未入树。

## 结论

s534 收口（守成巡查轻覆盖轮第二十三轮，零码刀）：
码态持位 7e2c25ff9、runner 127567616 恒（leg1
churn ×6 同形产物同码，leg2 零重编）；join stable
8510（8305P/198F/6T/1U）disagree 0 + l1only 1 +
l2only 4——vs s533 NEW 1 + GONE 1 + flips 0 位移全
落族内（族键 stable 位互换形第 3 次：detailed T/T
入、non-detailed 出，leg1 双整文件 T 形 / leg2 稳
态形，同一二进制双腿异相 = 环境调度带内摆动），
非族非哨键逐键恒等；P 集双腿双 8307P 持位第 3 轮；
flaky 零新例第 23 轮，picker 增益键第 57 轮保持，
onblur/dialogov/waapi 三哨持位，mpreload 哨过窗时
钟重顺延；全桥 1671 set 恒等绿窗零破；focus-dynamic
收窗后第 16 轮零再现。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，
不重复通知）。

下一步：守成巡查（对账基线换 s534 stable 8510——族
键摆动观察维持：出位/回归/混合形按形谱记账、Pass
形首现按 NEW 记账；全桥清单基线 b534.k 1671 持位；
flaky 键零新例时钟顺延——再现方触发新 <4 轮升级判
定；**mpreload 哨时钟重顺延后第 1 轮观察**——窗内
再现即升级判定、连续零翻红照常记账；他流入树归因
轮或轻覆盖；对端刀信号持续同上，落树后按他流刀归
因轮，join 实测为准）+ 候门决项等待用户拍板（video
P1+P2 立项 / 簇 D A2 立项 / s137）；rendering 流
dialog 16 红块协同面（归 rendering 流记账）。门开随
批落预案不变：簇 D 开 → customizable-select-styles
.css ×47 随批落。

工件：/tmp/s534/{s534-leg1,s534-leg2}.log、
s534-leg{1,2}.rc、leg{1,2}.s534.{start,end}、
s534-{l1,l2}.norm、l534-{1,2}.canon、s534.new（1）、
s534.gone（1）、s534.flips.txt（零行）、
s534.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}
、s533.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组
复用）+ fd121.keys（focus-dynamic 组）+ p1/p2.keys
（P 集）+ s534-bridge.log、s534-bridge.rc、b533.k
（复用）+ b534.k（set 恒等对账）、run-legs.sh、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
