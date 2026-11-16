# M2 s535 — 守成巡查轻覆盖轮第二十四轮：pull 零携入（上游无新提交）零码变零换代——码态持位 7e2c25ff9，runner 127567616 恒（leg1 Compiling ×6 engine 链 churn 同形产物同码 = s532–s534 同形，leg2 Finished 0.18s 零重编）——join **stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0 双腿完全同相**；vs s534 stable **NEW 4 + GONE 0 + flips 0 位移全落族内**：NEW 4 = 族 non-detailed 4 子测键（Home/End/PageUp/PageDown）F/F 双腿同相入 stable——**族键双腿双稳态形首次同相再现**（detailed 整文件键 T stable 持位 + non-detailed 整文件键双腿零发射、4 子测 F 双腿发射 = s519–s530 稳态形；s527→s534 摆动谱收敛观察点），Fail 形零新 Pass 形带内记账不升级——非族非哨键零 NEW 零 GONE 零 flips 逐键恒等；P 集账本口径双腿双 8307P 持位第 4 轮 + flaky 零新例第 24 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨 15 键双腿全 Pass——unsafe-url 键时钟重顺延后第 1 轮零翻红** + 全桥 1671P/0F rc=0 清单 set 恒等 + focus-dynamic 收窗后第 17 轮零再现

日期：2026-11-16。承前：s534 收口 push 后本轮 pull
**零携入**（`git pull --rebase` = 已经是最新的，对端
ZW-3 d47 刀体仍未写树——对端 R5738 在账 4 文件
+95/−2 未 commit）→ 按守成巡查轻覆盖轮（第二十四
轮）。

## 二进制 provenance

- 主刀面 = 零（pull 零携入，零码变）。
- 码态 = **7e2c25ff9 持位**；runner 二进制字节数
  **127567616 恒**（与 s533/s534 同代零换代）。
- leg1 23:32:33 起腿 `Compiling` ×6（product-version
  /net/engine/page-runtime/webview/wpt-runner）=
  s532–s534 同形 churn，产物同码（Finished release
  37.08s 后 runner 字节数不变）；leg2 同二进制
  `Finished in 0.18s` 零重编。

## 双腿 join（HEAD cf2d10ac9 树 = 码态 7e2c25ff9，
make testharness-html-semantics ×2）

- leg1 23:32:33–23:35:41 / leg2 23:35:41–23:38:09（
  零 lull 背靠背），raw/norm 8516/8516（双腿各 +2 行
  双发发射 dup：exact-len + WINdow 伪影照旧）——双
  腿零截断零真 hang（rc=2 惯例失败退出，时长正常）
  。管线全 LC_ALL=C 固化（s488 修复版复用）。
- join：**stable 8514（8305P/202F/6T/1U）+ disagree
  0 + l1only 0 + l2only 0——双腿完全同相**（摆动谱
  s519 轮以来首次零异相零单腿出位）。
- **vs s534 stable（8510）：NEW 4 + GONE 0 + flips
  0——位移全落族内**：
  - NEW 4 = 族 non-detailed 4 子测键
    （select-home-end-pagedown-pageup.optional.html
    :: Behavior of Home/End/PageDown/PageUp for
    customizable-\<select\>）**F/F 双腿同相入
    stable**（stable F 计数 198→202 持平对账）。
  - 族形 = **双腿双稳态形**：detailed 整文件键 T
    stable 持位（6T 之一）+ non-detailed 整文件键双
    腿零发射 + 4 子测 F 双腿发射——恰 s519–s530 稳
    态形；s527→s532→s534 互换摆动谱本轮收敛同相。
  - Fail 形零新 Pass 形，带内记账不升级；**非族非
    哨键零 NEW 零 GONE 零 flips——逐键恒等**。
- **P 集账本口径双腿双 8307P**（raw Pass 行口径，
  持位第 4 轮）；双腿 raw 形同相 202F/8307P/6T/1U。
- **focus-dynamic 121 键 stable 全 Pass**：收窗后
  第 17 轮零再现。
- 双发发射伪影 2 键照旧在册（exact-len + WINdow 双
  腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨顺延后零翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label
  双键红池照常）——flaky 键（checkbox/radio
  following-label 双行）双腿双 Pass：零新例第 24 轮
  （s512 时钟顺延）。
- dialog-overlay 哨第 64 轮零翻红（双键双腿 Pass）。
- **modulepreload 哨：15 键双腿全 Pass——unsafe-url
  键时钟重顺延后第 1 轮零翻红**（s534 过窗重起算后
  窗内零再现）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 58
  轮保持）/ checkmark Fail（带形不变）。哨复核全程
  live 状态 join。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行）：
**1671P/0F rc=0**（test 阶段 21.38s；filtered 2405
持位；Compiling ×3 test profile churn 同 s533 形）。
测试清单 vs s534（b534.k）：**1671 set 恒等，NEW 0
+ GONE 0**——绿窗零破。

## 对端观察

- 起腿前窗净判定（23:32 load 1.53 回落态，对端
  corpus/rustc/cargo/zero-wpt-runner 零重腿零等窗直
  接起腿）；窗内双腿背靠背零外部交叠零截断。
- 对端刀信号持续：ZW-3 d47 写面未 commit（4 文件
  +95/−2）；editing 流 B4-S2 残面刀 + rendering
  dialog 16 红块协同面 + css 文法刀后续刀（shapepos
  残面队列）+ wptfyi-css 残面——本轮均未入树。

## 结论

s535 收口（守成巡查轻覆盖轮第二十四轮，零携入零码
刀）：码态持位 7e2c25ff9、runner 127567616 恒；
join stable 8514（8305P/202F/6T/1U）disagree 0 +
l1only 0 + l2only 0 双腿完全同相——vs s534 NEW 4 +
GONE 0 + flips 0 位移全落族内（族 non-detailed 4
子测键 F/F 入 stable，族形双腿双稳态形 = s519–s530
形收敛，Fail 形零新 Pass 形带内），非族非哨键逐键
恒等；P 集双腿双 8307P 持位第 4 轮；flaky 零新例
第 24 轮，picker 增益键第 58 轮保持，onblur/
dialogov/waapi 三哨持位，mpreload 哨顺延后第 1 轮
零翻红；全桥 1671 set 恒等绿窗零破；focus-dynamic
收窗后第 17 轮零再现。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，
不重复通知）。

下一步：守成巡查（对账基线换 s535 stable 8514——
族键稳态形收敛观察维持：若再现异相/出位按形谱记
账、Pass 形首现按 NEW 记账；全桥清单基线 b535.k
1671 持位；flaky 键零新例时钟顺延；**mpreload 哨
顺延后窗内第 2 轮观察**——再现即升级判定；他流入
树归因轮或轻覆盖；对端刀信号持续：ZW-3 d47 刀体
（querySelector 施工刀）落树后按他流刀归因轮 +
editing 流 B4-S2 残面刀 + rendering dialog 16 红块
协同面 + css 文法刀后续刀（shapepos 残面队列）+
wptfyi-css 残面，join 实测为准）+ 候门决项等待用户
拍板（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归 rendering 流
记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s535/{s535-leg1,s535-leg2}.log、
s535-leg{1,2}.rc、leg{1,2}.s535.{start,end}、
s535-{l1,l2}.norm、l535-{1,2}.canon、s535.new（4）、
s535.gone（零行）、s535.flips.txt（零行）、
s535.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}
、s534.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组
复用）+ fd121.keys（focus-dynamic 组复用）+
s535-bridge.log、s535-bridge.rc、b534.k（复用）+
b535.k（set 恒等对账）、run-legs.sh、norm.sh/canon
.sh/join.sh（s488 修复版管线复用，全 LC_ALL=C）。
