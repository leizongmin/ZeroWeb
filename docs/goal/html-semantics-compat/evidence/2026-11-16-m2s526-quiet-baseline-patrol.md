# M2 s526 — 守成巡查轻覆盖轮第二十轮：pull 零携入零码变，码态持位 7668cf495，runner 127530232 恒（leg1 Compiling ×6 = 桥跑侧 churn 同形产物同码，leg2 零重编）——join **stable 8510（8305P/198F/6T/1U）disagree 0 + l1only 4 + l2only 1 全族键非族零分歧形**，vs s525 stable **NEW 0 + GONE 2 + flips 0 非族逐键恒等**：GONE 2 = 族 select-home-end-pagedown-pageup Home/End 两子测键双腿异相出位（leg1 = detailed T + 4 子测全发射 F 恰 s519/s522/s524/s525-leg1 形；leg2 = detailed T + non-detailed 整文件 T + 零子测发射——文件级超时吞全部子测发射，s525-leg2 双 T 形的极端形，带内无新 Pass 形，同码态双腿异相为摆动特征非确定性刀效应，键从未 Pass 带内记账不升级，stable T 形 6→6 持位、F 形 200→198），P 集双腿双 8307P（持位第 48 轮）+ flaky 零新例第 15 轮 + 四观察哨零新翻红 + 全桥绿窗 1669P/0F 清单 set 恒等 + focus-dynamic 收窗后第 8 轮零再现

日期：2026-11-16。承前：s525 下一步「守成巡查（对账基
线换 s525 stable 8512——族键摆动观察维持……他流入树
归因轮或轻覆盖）」。本轮 pull --rebase 零携入（远端
tip = 本流 s525 收口 7dae773ff docs-only；上轮 push
前置 pull 携入 fd6228df0 rendering 流 R5728 燃点轮
docs-only 零码刀）→ 按轻覆盖轮。

## 双腿 join（HEAD 7dae773ff 树 = 码态 7668cf495 持位，
make testharness-html-semantics ×2）

- leg1 20:16:23–20:19:19 / leg2 20:19:19–20:22:03（零
  lull 背靠背），raw 8517/8514 → norm 8516/8513（双腿
  各丢 1 行 bare 发射 dup：exact-len + WINdow 伪影照
  旧）——双腿零截断零真 hang（rc=2 惯例失败退出，时
  长正常）。本轮管线全 LC_ALL=C 固化。
- leg1 `Compiling` ×6（product-version/net/engine/
  page-runtime/webview/wpt-runner 链）= s525 桥跑侧
  churn 同形——重编产物同码，runner 二进制指纹
  **127530232 恒**；leg2 零重编。
- **P 集账本口径双腿双 8307P**（持位第 48 轮）。
- join：**stable 8510（8305P/198F/6T/1U）+ disagree 0
  + l1only 4 + l2only 1**——单腿出位 5 行**全部为族
  select-home-end-pagedown-pageup 两文件键（4 子测 +
  non-detailed 整文件），非族零分歧形**。
- **vs s525 stable（8512 = 8305P/200F/6T/1U）：NEW 0
  + GONE 2 + flips 0——非族逐键恒等**：
  - GONE 2 = 族 Home/End 两子测键双腿异相出位：
    leg1 = detailed T/T + 4 子测全发射 F/F（恰
    s519/s522/s524/s525-leg1 形）/ leg2 = detailed T
    + non-detailed 整文件 T + **零子测发射**（文件级
    超时吞全部子测发射——s525-leg2 双 T 形的极端
    形：上轮 Home/End 尚发射 2 子测，本轮零发射）。
    全部落 Fail/Timeout 谱带内，键从未 Pass，带内形
    谱记账不升级（Pass 形首现方按 NEW 记账）；同码
    态双腿异相为摆动特征非确定性刀效应；stable T
    形 6→6 持位、F 形 200→198。族键摆动谱 s519
    T+4F→…→s524 T+4F 同相→s525 leg1 s519 形/leg2
    双 T 形异相→s526 leg1 s519 形/leg2 双 T 零子测
    发射形异相。
  - 非族键零 NEW 零 flips——零语义位移（码态持位
    旁证）。
- **focus-dynamic 121 键 stable 全 Pass：收窗后第 8
  轮零再现**（后续再现按带内间歇记账，无升级判定）。
- 双发发射伪影 2 键照旧在册（exact-len 双腿各 2 dup +
  WINdow 双腿各 2 dup）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 15 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 55 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 58 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 49 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（码态持位旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 17.47s；
filtered 2405 持位）。测试清单 vs s525（b525.k）：
**set 恒等 1669，NEW 0 / GONE 0**。

## 对端观察

- 起腿前窗净判定（20:16:09 pull 零携入核对时 load
  3.69 回落态，对端/外部零 runner/cargo/rustc/
  mksquashfs 进程级交叠）；窗内双腿时长正常零截断，
  族键形为已知形谱带内组合无新 Pass 形。

## 结论

s526 收口（守成巡查轻覆盖轮第二十轮）：pull 零携入零
码变零换代（runner 127530232 恒，churn 重编产物同
码），**族键摆动谱 s525 异相后第 1 轮持续异相（leg1
s519 形持位/leg2 双 T 零子测发射极端形）**——stable
8510，非族零位移零翻转；GONE 2 全族键带内出位不升
级；P 集双腿双 8307P 持位第 48 轮；flaky 键零新例第
15 轮，picker 增益键第 49 轮保持，四观察哨零新翻红；
全桥 1669 钉面 set 恒等零破；focus-dynamic 收窗后第
8 轮零再现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s526 stable 8510——族键
摆动观察维持：出位/回归/混合形按形谱记账、Pass 形首
现按 NEW 记账；flaky 键零新例时钟顺延——再现方触发
新 <4 轮升级判定；他流入树归因轮或轻覆盖；对端刀信
号持续：editing 流 B4-S2 残面刀（dwuf 八变体端点规范
化未接臂/during-and-after-dispatch 与克隆门互斥 1F/
残 9F 面余族刀——ihe5 rawtext 4 键刀已落 7668cf495）
+ rendering dialog 16 红块协同面 + css 文法刀后续刀 +
wptfyi-css 残面（calc 代数插值共享面/jsanim flip
Steps(Start)/path 命令级插值）+ ZW-3 querySelector 施
工刀，落树后按他流刀归因轮，join 实测为准）+ 候门决
项等待用户拍板（video P1+P2 立项 / 簇 D A2 立项 /
s137）；rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s526/{s526-leg1,s526-leg2}.log、
s526-leg{1,2}.rc、leg{1,2}.s526.{start,end}、
s526-{l1,l2}.norm、l526-{1,2}.canon、s526.new（零行）、
s526.gone（2 族子测键）、s526.flips.txt（零行）、
s526.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys(4
族子测),leg2only.keys(1 族整文件),disagree.keys,
stable.keys,stable.canon}、s525.stable.{keys,canon}
（基线复用）+ s477-{onblur,dialogov,mpreload,waapi}
.keys（哨键组复用，须剥状态后缀直查 s1/s2）、
s526-bridge.log、s526-bridge.rc、b525.k（复用）+
b526.k（清单恒等对账）+ b526.{new,gone}（双零行）、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
