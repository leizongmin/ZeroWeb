# M2 s522 — 守成巡查轻覆盖轮第十八轮：pull 零携入零码变，码态持位 4f5e7a75d，runner 127521776 恒（leg1 Compiling ×6 = 桥跑侧 churn 同形产物同码，leg2 零重编）——join **stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0 全键双腿零分歧形回归**，vs s521 stable **NEW 5 + GONE 0 + flips 0 非族逐键恒等**：NEW 5 = 族 select-home-end-pagedown-pageup 两文件键全集双腿同相回归入 stable（**回归窗第 2 轮兑现回归形：完全复位 s519 形**——detailed 整文件键 Timeout/Timeout + 非 detailed 完成 4 子测 Fail/Fail，stable 8514 与 s519 stable 逐键恒等同数同形，stable T 形 5→6），P 集双腿双 8307P（持位第 44 轮）+ flaky 零新例第 11 轮 + 四观察哨零新翻红 + 全桥绿窗 1669P/0F 清单 set 恒等 + focus-dynamic 收窗后第 4 轮零再现

日期：2026-11-16。承前：s521 下一步「守成巡查（对账基
线换 s521 stable 8509——族键 GONE 5 出位回归窗观察维
持……他流入树归因轮或轻覆盖）」。本轮 pull --rebase
零携入（远端 tip = 本流 s521 收口 ca4b224ee docs-only）
→ 按轻覆盖轮。

## 双腿 join（HEAD ca4b224ee 树 = 码态 4f5e7a75d 持位，
make testharness-html-semantics ×2）

- leg1 18:53:19–18:56:31 / leg2 18:56:31–18:58:58（零
  lull 背靠背），raw 8517/8517 → norm 8516/8516（双腿
  各丢 1 行 bare 发射 dup：exact-len + WINdow 伪影照
  旧）——双腿零截断零真 hang（rc=2 惯例失败退出，时
  长正常）。本轮管线全 LC_ALL=C 固化。
- leg1 `Compiling` ×6（product-version/net/engine/
  page-runtime/webview/wpt-runner 链）= s521 桥跑侧
  churn 同形——重编产物同码，runner 二进制指纹
  **127521776 恒**（18:54 时间戳重写、尺寸不变）；
  leg2 零重编。
- **P 集账本口径双腿双 8307P**（持位第 44 轮）。
- join：**stable 8514（8305P/202F/6T/1U）+ disagree 0
  + l1only 0 + l2only 0——全键双腿零分歧形回归**
  （s520/s521 族键异相分歧后本轮全键同相）。
- **vs s521 stable（8509 = 8305P/198F/5T/1U）：NEW 5
  + GONE 0 + flips 0——非族逐键恒等**：
  - NEW 5 = 族 select-home-end-pagedown-pageup 两文
    件键全集（非 detailed 整文件键不占行 + 4 子测键
    Fail/Fail + detailed 整文件键 Timeout/Timeout）
    **双腿同相回归入 stable——族键 GONE 5 出位回归窗
    第 2 轮兑现回归形：完全复位 s519 形**（stable
    8514 与 s519 stable 逐键恒等，同数同形；stable T
    形 5→6 回 s519 位）。回归形落 Fail/Timeout 谱带
    内（该键从未 Pass），带内回归记账不升级。
  - 非族键零 NEW 零 flips——零语义位移。
- **focus-dynamic 121 键 stable 全 Pass：收窗后第 4
  轮零再现**（后续再现按带内间歇记账，无升级判定）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 11 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 51 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 54 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 45 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（码态持位旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 18.18s；
filtered 2405 持位）。测试清单 vs s521（b521.k）：
**set 恒等 1669，NEW 0 / GONE 0**。

## 对端观察

- 对端 editing 流 **d45 selection 单面腿持续在跑**
  （18:30 起窗内 >28 分钟，`/tmp/zw-runner-d45
  testharness-selection --json` 112% CPU，test-guard
  包裹 time-limit 2700）——独立二进制拷贝**零构建级
  交叠**；s521 同状态双腿安全先例（零形漂零截断）+
  16 核 load 回落 1.42 起腿判定。窗内本流双腿时长正
  常零截断，族键形为已知 s519 形复位无新形。

## 结论

s522 收口（守成巡查轻覆盖轮第十八轮）：pull 零携入零
码变零换代（runner 127521776 恒，churn 重编产物同码），
**族键 GONE 5 出位回归窗第 2 轮双腿同相回归复位 s519
形**——stable 8514 逐键恒等（与 s519 stable 同数同
形），join 全键双腿零分歧形回归，非族零位移；P 集双
腿双 8307P 持位第 44 轮；flaky 键零新例第 11 轮，
picker 增益键第 45 轮保持，四观察哨零新翻红；全桥
1669 钉面 set 恒等零破；focus-dynamic 收窗后第 4 轮零
再现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s522 stable 8514——族键
回归形后摆动观察维持：出位/混合形按形谱记账、Pass 形
首现按 NEW 记账；flaky 键零新例时钟顺延——再现方触
发新 <4 轮升级判定；他流入树归因轮或轻覆盖；对端刀
信号持续：editing 流 B4-S2 残面刀（左块尾随 br/嵌套
深锚/dwuf gtr +22 名义红）/残 31F 面余族刀 + rendering
dialog 16 红块协同面 + css 文法刀后续刀 + wptfyi-css
残面（calc 代数插值共享面/jsanim flip Steps(Start)/
path 命令级插值）+ ZW-3 querySelector 施工刀，落树后
按他流刀归因轮，join 实测为准）+ 候门决项等待用户拍
板（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归 rendering 流记
账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s522/{s522-leg1,s522-leg2}.log、
s522-leg{1,2}.rc、leg{1,2}.s522.{start,end}、
s522-{l1,l2}.norm、l522-{1,2}.canon、s522.new（5 族
键）、s522.gone（零行）、s522.flips.txt（零行）、
s522.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s521.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 canon）、s522-bridge.log、
s522-bridge.rc、bridge.s522.{start,end}、b521.k（复用）
+ b522.k（清单恒等对账）+ b522.{new,gone}（双零行）、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
