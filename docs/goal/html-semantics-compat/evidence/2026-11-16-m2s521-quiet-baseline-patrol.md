# M2 s521 — 守成巡查轻覆盖轮第十七轮：pull 零携入零码变，码态持位 4f5e7a75d，runner 127521776 恒（leg1 Compiling ×6 = s520 桥跑侧 churn 同形产物同码，leg2 Finished 0.18s 零重编）——vs s520 stable NEW 0 + GONE 0 + flips 0 **stable 8509 逐键恒等持位**，join 分歧 6 行全族键（disagree 1 = 族 detailed 整文件键 T/F 异相 + l1only 4 = 族 4 子测键 + l2only 1 = 族非 detailed 整文件键）非族零分歧形，族键 GONE 5 出位回归窗第 1 轮保持出位未回归（形谱 leg1 回 s519 形/leg2 持 s520 形摆动，键从未 Pass 带内记账不升级），P 集双腿双 8307P（持位第 43 轮）+ flaky 零新例第 10 轮 + 四观察哨零新翻红 + 全桥绿窗 1669P/0F 清单 set 恒等 + focus-dynamic 收窗后第 3 轮零再现

日期：2026-11-16。承前：s520 下一步「守成巡查（对账基
线换 s520 stable 8509——族键 GONE 5 出位后回归窗观
察……他流入树归因轮或轻覆盖）」。本轮 pull --rebase
零携入（远端 tip = 本流 s520 收口 2faf1223c docs-only）
→ 按轻覆盖轮。

## 双腿 join（HEAD 2faf1223c 树 = 码态 4f5e7a75d 持位，
make testharness-html-semantics ×2）

- leg1 18:40:04–18:43:20 / leg2 18:43:20–18:46:07（零
  lull 背靠背），raw 8517/8514 → norm 8516/8513（双腿
  各丢 1 行 bare 发射 dup：exact-len + WINdow 伪影照
  旧）——双腿零截断零真 hang（rc=2 惯例失败退出，时
  长正常）。本轮管线全 LC_ALL=C 固化。
- leg1 `Compiling` ×6（product-version/net/engine/
  page-runtime/webview/wpt-runner 链）= s520 桥跑
  （cargo test -p zero-engine --lib dev profile）侧
  churn 同形——重编产物同码，runner 二进制指纹
  **127521776 恒**（18:40 时间戳重写、尺寸不变）；leg2
  `Finished 0.28s → 0.18s` 同二进制零重编。
- **P 集账本口径双腿双 8307P**（持位第 43 轮）。
- join：**stable 8509（8305P/198F/5T/1U）逐键恒等持
  位 + disagree 1 + l1only 4 + l2only 1**——分歧 6 行
  **全部为族 select-home-end-pagedown-pageup 两文件
  键，非族零分歧形**。
- **vs s520 stable（8509）：NEW 0 + GONE 0 + flips
  0——集合逐键恒等**（非族零位移）。
- **族键 GONE 5 出位回归窗第 1 轮：保持出位未回归**
  （s520 GONE 5 形持位）：形谱 leg1 = detailed 整文件
  键 Timeout + 非 detailed 完成 4 子测 Fail（**回
  s519 形**）/ leg2 = detailed Fail + 非 detailed 整
  文件 Timeout（**持 s520 形**）——s519 形 ↔ s520 形
  间双腿异相摆动；该键从未 Pass，带内形谱记账不升级
  （Pass 形首现方按 NEW 记账）。
- **focus-dynamic 121 键 stable 全 Pass：收窗后第 3
  轮零再现**（后续再现按带内间歇记账，无升级判定）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 10 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 50 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 53 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 44 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（码态持位旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 19.68s；
filtered 2405 持位）。测试清单 vs s520（b520.k）：
**set 恒等 1669，NEW 0 / GONE 0**。

## 对端观察

- 对端 editing 流 **d45 selection 单面腿窗内交叠**
  （18:30 起 `/tmp/zw-runner-d45 testharness-selection
  --json`，test-guard 包裹 time-limit 2700）——独立
  二进制拷贝**零构建级交叠**；16 核 load 2.57（对端单
  进程 118% CPU）余量起腿判定（等 7.5 分钟未退，非
  corpus 级全量负载，未干等）。窗内本流双腿时长正常
  零截断；若本轮出现 Timeout 形漂移，族键形谱判读需
  保守归因（本轮族键形恰为 s519/s520 已知形间摆动，
  无新形，不触发）。

## 结论

s521 收口（守成巡查轻覆盖轮第十七轮）：pull 零携入零
码变零换代（runner 127521776 恒，leg1 churn 重编产物
同码），stable 8509 逐键恒等持位（NEW 0/GONE 0/flips
0 非族零位移）；join 分歧 6 行全族键、非族零分歧形；
族键 GONE 5 回归窗第 1 轮保持出位（s519 形 ↔ s520 形
摆动带内）；P 集双腿双 8307P 持位第 43 轮；flaky 键零
新例第 10 轮，picker 增益键第 44 轮保持，四观察哨零
新翻红；全桥 1669 钉面 set 恒等零破；focus-dynamic 收
窗后第 3 轮零再现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s521 stable 8509——族键
GONE 5 出位回归窗观察维持：回归形按带内回归记账、
Pass 形首现按 NEW 记账；flaky 键零新例时钟顺延——再
现方触发新 <4 轮升级判定；他流入树归因轮或轻覆盖；
对端刀信号持续：editing 流 B4-S2 残面刀（左块尾随
br/嵌套深锚/dwuf gtr +22 名义红）/残 31F 面余族刀 +
rendering dialog 16 红块协同面 + css 文法刀后续刀 +
wptfyi-css 残面（calc 代数插值共享面/jsanim flip
Steps(Start)/path 命令级插值）+ ZW-3 querySelector 施
工刀，落树后按他流刀归因轮，join 实测为准）+ 候门决
项等待用户拍板（video P1+P2 立项 / 簇 D A2 立项 /
s137）；rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s521/{s521-leg1,s521-leg2}.log、
s521-leg{1,2}.rc、leg{1,2}.s521.{start,end}、
s521-{l1,l2}.norm、l521-{1,2}.canon、s521.new（零行）、
s521.gone（零行）、s521.flips.txt（零行）、
s521.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys(4
族子测),leg2only.keys(1 族整文件),disagree.keys(1 族
整文件),stable.keys,stable.canon}、
s520.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 canon）、s521-bridge.log、
s521-bridge.rc、bridge.s521.{start,end}、b520.k（复用）
+ b521.k（清单恒等对账）+ b521.{new,gone}（双零行）、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
