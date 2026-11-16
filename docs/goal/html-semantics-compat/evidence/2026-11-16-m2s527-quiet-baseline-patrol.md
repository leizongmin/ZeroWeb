# M2 s527 — 守成巡查轻覆盖轮第二十一轮：pull 零携入零码变，码态持位 7668cf495，runner 127530232 恒（leg1 Compiling ×6 = 桥跑侧 churn 同形产物同码，leg2 零重编）——join **stable 8510（8305P/199F/5T/1U）disagree 0 + l1only 1 + l2only 4 全族键非族零分歧形**，vs s526 stable **NEW 0 + GONE 0 + flips 1 集合逐键恒等**：flips 1 = 族 select-home-end-pagedown-pageup **detailed 整文件键 Timeout→Fail**（s519–s526 八轮 T 持位后首次双腿同相 F——Fail/Timeout 带内键从未 Pass 带内记账不升级，stable T 形 6→5、F 形 198→199）——族键形本轮：leg1 = detailed F + non-detailed 整文件 T + 零子测发射（恰 s520-leg2 形）/ leg2 = detailed F + 4 子测 F（s519 形 detailed 槽 T→F 变体）——族键摆动谱 s525/s526 异相后第 2 轮持续异相（双腿形镜像互换），同码态双腿异相为摆动特征非确定性刀效应，P 集双腿双 8307P（持位第 49 轮）+ flaky 零新例第 16 轮 + 四观察哨零新翻红 + 全桥绿窗 1669P/0F 清单 set 恒等 + focus-dynamic 收窗后第 9 轮零再现

日期：2026-11-16。承前：s526 下一步「守成巡查（对账基
线换 s526 stable 8510——族键摆动观察维持……他流入树
归因轮或轻覆盖）」。本轮 pull --rebase 零携入（远端
tip = 本流 s526 收口 8e9208695 docs-only；上轮 push
前置 pull 携入 99afb7e4f + edef026b4 两笔 rendering 流
R5729/R5730 守成巡查轮 docs-only 零码刀）→ 按轻覆盖
轮。

## 双腿 join（HEAD 8e9208695 树 = 码态 7668cf495 持位，
make testharness-html-semantics ×2）

- leg1 20:30:12–20:33:26 / leg2 20:33:26–20:36:05（零
  lull 背靠背），raw 8514/8517 → norm 8513/8516（双腿
  各丢 1 行 bare 发射 dup：exact-len + WINdow 伪影照
  旧）——双腿零截断零真 hang（rc=2 惯例失败退出，时
  长正常）。本轮管线全 LC_ALL=C 固化。
- leg1 `Compiling` ×6（product-version/net/engine/
  page-runtime/webview/wpt-runner 链）= s526 桥跑侧
  churn 同形——重编产物同码，runner 二进制指纹
  **127530232 恒**；leg2 零重编。
- **P 集账本口径双腿双 8307P**（持位第 49 轮）。
- join：**stable 8510（8305P/199F/5T/1U）+ disagree 0
  + l1only 1 + l2only 4**——单腿出位 5 行**全部为族
  select-home-end-pagedown-pageup 两文件键（1 非
  detailed 整文件 + 4 子测），非族零分歧形**。
- **vs s526 stable（8510 = 8305P/198F/6T/1U）：NEW 0
  + GONE 0 + flips 1——集合逐键恒等**：
  - flips 1 = 族 **detailed 整文件键 Timeout→Fail**
    （s519–s526 八轮 T 槽持位后首次双腿同相 F）。
    族键形本轮：leg1 = detailed F + non-detailed 整
    文件 T + 零子测发射（**恰 s520-leg2 形**）/
    leg2 = detailed F + 4 子测 F（**s519 形 detailed
    槽 T→F 变体**）——族键摆动谱 s519 T+4F→…→
    s524 T+4F 同相→s525 leg1 s519 形/leg2 双 T 形→
    s526 leg1 s519 形/leg2 双 T 零子测形→s527 双腿
    形镜像互换（leg1 持双 T 形变体/leg2 持 s519 形
    变体）——全部落 Fail/Timeout 谱带内，键从未
    Pass，带内形谱记账不升级（Pass 形首现方按 NEW
    记账）；同码态双腿异相为摆动特征非确定性刀效
    应；stable T 形 6→5、F 形 198→199（detailed 槽
    T→F 位移）。
  - 非族键零 NEW 零 GONE 零 flips——零语义位移
    （码态持位旁证）。
- **focus-dynamic 121 键 stable 全 Pass：收窗后第 9
  轮零再现**（后续再现按带内间歇记账，无升级判定）。
- 双发发射伪影 2 键照旧在册（exact-len 双腿各 2 dup +
  WINdow 双腿各 2 dup）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 16 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 56 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 59 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 50 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（码态持位旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 25.74s；
filtered 2405 持位）。测试清单 vs s526（b526.k）：
**set 恒等 1669，NEW 0 / GONE 0**。

## 对端观察

- 起腿前窗净判定（20:30:07 pull 零携入核对 + 进程检
  查零对端/外部 runner/cargo/rustc/mksquashfs 交叠，
  20:30:12 起腿）；窗内双腿时长正常零截断，族键形为
  已知形谱带内组合无新 Pass 形。

## 结论

s527 收口（守成巡查轻覆盖轮第二十一轮）：pull 零携入
零码变零换代（runner 127530232 恒，churn 重编产物同
码），**族键摆动谱 s525/s526 异相后第 2 轮持续异相
（双腿形镜像互换，detailed 槽八轮 T 持位后首次双腿
F，flips 1 全族键带内不升级）**——stable 8510 集合逐
键恒等（NEW 0/GONE 0），非族零位移零翻转；P 集双腿
双 8307P 持位第 49 轮；flaky 键零新例第 16 轮，picker
增益键第 50 轮保持，四观察哨零新翻红；全桥 1669 钉面
set 恒等零破；focus-dynamic 收窗后第 9 轮零再现。挂
用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+
s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s527 stable 8510——族键
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

工件：/tmp/s527/{s527-leg1,s527-leg2}.log、
s527-leg{1,2}.rc、leg{1,2}.s527.{start,end}、
s527-{l1,l2}.norm、l527-{1,2}.canon、s527.new（零行）、
s527.gone（零行）、s527.flips.txt（1 族 detailed 键
T→F）、s527.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only
.keys(1 族整文件),leg2only.keys(4 族子测),disagree
.keys,stable.keys,stable.canon}、s526.stable.{keys,
canon}（基线复用）+ s477-{onblur,dialogov,mpreload,
waapi}.keys（哨键组复用，须剥状态后缀直查 s1/s2）、
s527-bridge.log、s527-bridge.rc、b526.k（复用）+
b527.k（清单恒等对账）+ b527.{new,gone}（双零行）、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
