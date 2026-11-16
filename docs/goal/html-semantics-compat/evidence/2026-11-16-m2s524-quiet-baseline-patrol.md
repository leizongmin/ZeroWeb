# M2 s524 — 守成巡查轻覆盖轮第十九轮：pull 零携入零码变，码态持位 ca4bff0e5，runner 127525992 恒（leg1 Compiling ×6 = 桥跑侧 churn 同形产物同码，leg2 Finished 0.16s 零重编）——join **stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0 全键双腿零分歧形回归**，vs s523 stable **NEW 5 + GONE 0 + flips 0 非族逐键恒等**：NEW 5 = 族 select-home-end-pagedown-pageup 两文件键全集双腿同相回归入 stable（**完全复位 s519/s522 形**——detailed 整文件键 Timeout/Timeout + 非 detailed 完成 4 子测 Fail/Fail，stable 与 s519/s522 stable 逐键恒等同数同形，stable T 形 5→6，回归形落 Fail/Timeout 带内不升级），P 集双腿双 8307P（持位第 46 轮）+ flaky 零新例第 13 轮 + 四观察哨零新翻红 + 全桥绿窗 1669P/0F 清单 set 恒等 + focus-dynamic 收窗后第 6 轮零再现

日期：2026-11-16。承前：s523 下一步「守成巡查（对账基
线换 s523 stable 8509——族键摆动观察维持……他流入树
归因轮或轻覆盖）」。本轮 pull --rebase 零携入（远端
tip = 本流 s523 收口 e66756485 docs-only）→ 按轻覆盖
轮。

## 双腿 join（HEAD e66756485 树 = 码态 ca4bff0e5 持位，
make testharness-html-semantics ×2）

- leg1 19:45:55–19:48:52 / leg2 19:48:52–19:51:14（零
  lull 背靠背），raw 8517/8517 → norm 8516/8516（双腿
  各丢 1 行 bare 发射 dup：exact-len + WINdow 伪影照
  旧）——双腿零截断零真 hang（rc=2 惯例失败退出，时
  长正常）。本轮管线全 LC_ALL=C 固化。
- leg1 `Compiling` ×6（product-version/net/engine/
  page-runtime/webview/wpt-runner 链）= s523 桥跑侧
  churn 同形——重编产物同码，runner 二进制指纹
  **127525992 恒**；leg2 `Finished 0.16s` 同二进制零
  重编。
- **P 集账本口径双腿双 8307P**（持位第 46 轮）。
- join：**stable 8514（8305P/202F/6T/1U）+ disagree 0
  + l1only 0 + l2only 0——全键双腿零分歧形回归**
  （s523 族键异相分歧后本轮全键同相）。
- **vs s523 stable（8509 = 8305P/198F/5T/1U）：NEW 5
  + GONE 0 + flips 0——非族逐键恒等**：
  - NEW 5 = 族 select-home-end-pagedown-pageup 两文
    件键全集（非 detailed 整文件键不占行 + 4 子测键
    Fail/Fail + detailed 整文件键 Timeout/Timeout）
    **双腿同相回归入 stable——完全复位 s519/s522 形**
    （stable 8514 与 s519/s522 stable 逐键恒等，同数
    同形；stable T 形 5→6 回位）。回归形落 Fail/
    Timeout 谱带内（该键从未 Pass），带内回归记账不
    升级。族键摆动谱更新：s519 T+4F→s520 F/T+T/F 异
    相→s521 出位混合→s522 T+4F 同相→s523 F/T+T/F 异
    相→s524 T+4F 同相回归。
  - 非族键零 NEW 零 flips——零语义位移。
- **focus-dynamic 121 键 stable 全 Pass：收窗后第 6
  轮零再现**（后续再现按带内间歇记账，无升级判定）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 13 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 53 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 56 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 47 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（码态持位旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 15.84s；
filtered 2405 持位）。测试清单 vs s523（b523.k）：
**set 恒等 1669，NEW 0 / GONE 0**。

## 对端观察

- 窗前外部负载（非本仓流程：mykernel aarch64 kernel
  构建 freebsd-personality 变体 rustc 起 175% CPU）
  **等待退出后起腿**（19:45:47 load 0.29 窗净判定，
  轮询等待 ~9 分钟）；窗内对端 editing/rendering 流
  零进程级交叠；窗内双腿时长正常零截断，族键形为
  s519/s522 已知形复位无新形。

## 结论

s524 收口（守成巡查轻覆盖轮第十九轮）：pull 零携入零
码变零换代（runner 127525992 恒，churn 重编产物同码），
**族键摆动谱 s523 异相出位后第 2 轮双腿同相回归复位
s519/s522 形**——stable 8514 逐键恒等（与 s519/s522
stable 同数同形），join 全键双腿零分歧形回归，非族零
位移；P 集双腿双 8307P 持位第 46 轮；flaky 键零新例
第 13 轮，picker 增益键第 47 轮保持，四观察哨零新翻
红；全桥 1669 钉面 set 恒等零破；focus-dynamic 收窗后
第 6 轮零再现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s524 stable 8514——族键
摆动观察维持：出位/回归/混合形按形谱记账、Pass 形首
现按 NEW 记账；flaky 键零新例时钟顺延——再现方触发
新 <4 轮升级判定；他流入树归因轮或轻覆盖；对端刀信
号持续：editing 流 B4-S2 残面刀（dwuf 八变体端点规范
化未接臂/during-and-after-dispatch 与克隆门互斥 1F/
残 9F 面余族刀——对端 ihe4 下一步 rawtext 4 键刀）+
rendering dialog 16 红块协同面 + css 文法刀后续刀 +
wptfyi-css 残面（calc 代数插值共享面/jsanim flip
Steps(Start)/path 命令级插值）+ ZW-3 querySelector 施
工刀，落树后按他流刀归因轮，join 实测为准）+ 候门决
项等待用户拍板（video P1+P2 立项 / 簇 D A2 立项 /
s137）；rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s524/{s524-leg1,s524-leg2}.log、
s524-leg{1,2}.rc、leg{1,2}.s524.{start,end}、
s524-{l1,l2}.norm、l524-{1,2}.canon、s524.new（5 族
键）、s524.gone（零行）、s524.flips.txt（零行）、
s524.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s523.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 canon）、s524-bridge.log、
s524-bridge.rc、bridge.s524.{start,end}、b523.k（复用）
+ b524.k（清单恒等对账）+ b524.{new,gone}（双零行）、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
