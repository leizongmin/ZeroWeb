# M2 s539 — 守成巡查轮第二十七轮：**双污染窗作废 + 第三窗静默浸泡判净起腿**——pull 零携入（round start 01:22 上游持位 ad3221ab0，4bcce5f4a docs-only 已在 s538 基线树）零码变零换代（码态持位 55c78a219，runner 127651344 恒）——窗三 join **stable 8510（leg1 8307P/198F/7T/1U + leg2 8307P/199F/6T/1U norm）disagree 1 + l1only 0 + l2only 0**；vs s538 stable **NEW 1 + GONE 5 + flips 0——位移全落族键摆动带（s531 同型带内记账）**：NEW 1 = 族 non-detailed 整文件键 **T/T 双腿同相发射**（s531/s533 已入谱形）+ GONE 4 = 族 non-detailed 4 子测键被吞 + disagree 1 = 族 detailed 整文件键 T/F 单键异相（s533 形 T/F 镜像方向）——**非族非哨键零 NEW 零 GONE 零 flips 逐键恒等**，零新 Pass 形（NEW 键为 T 形）——P 集双腿双 8307P 持位第 8 轮 + flaky 零新例第 28 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨转常规连轮计数第 1 轮零翻红（15 键双腿全 Pass）** + 全桥 1672P/0F rc=0 清单 set 恒等（b538.k→b539.k）+ focus-dynamic 收窗后第 21 轮零再现

日期：2026-11-17（round start 01:22，窗三起腿
01:51:32）。承前：s538 收口后 pull 零携入 → 按
守成巡查轻覆盖轮（第二十七轮）。

## 窗净判定三幕（对端 d48 测量腿/build 循环期）

- **窗一 01:24:16–01:31:18（作废）**：守候 v1
  （进程门 zero-wpt-runner/test-guard 20s 双查，
  s538 先例脚本）判净起腿——但对端 d48 测量腿+
  build 循环于腿窗内叠跑：leg1/leg2 时长拉长
  （3:49/3:13 vs 常态 3:04/2:30）、leg2 塌 67 键
  （l1only 67，textfieldselection select-event
  负载敏感族整组未发）、族键 detailed T→F 双腿
  形 + non-detailed 整文件键首发射 NEW 1——
  **污染形谱作废不入账**（s536 先例：族键摆动形
  对时序敏感，叠跑污染形谱记账）；v1 judge 取证
  文件缺失成因未定，作废证据以腿时序+负载曲线
  （01:32 起 load 13.95 对端 d46-tgt 全量重建
  rustc ×3@300%）+塌陷形谱为准；18 工件归档
  /tmp/s539/polluted/。
- **窗二 01:35:25–01:42:22（作废）**：守候 v2
  （进程门+load<3 门 20s 双查）再判净起腿仍被叠
  跑——leg2 再塌 69 键（l1only 71）+ disagree 2
  ，leg1 亦短 3 行，stable 8438 形同污染——作废
  归档 polluted2/（17 工件）。
- **窗三 01:51:32–01:57:07（入账）**：守候 v3
  （进程门+load<3 门 ×**3 查 45s 静默浸泡**）判
  净起腿——双腿时长恢复常态（2:52/2:43）、norm
  双腿 8513 完全同相、零单腿出位——入账窗。
  01:54 对端 python 驱动再起但双腿未现塌陷形
  （窗一/二污染签名=单腿塌 67+/71 键，窗三零塌
  陷），作带内形判读。

## 二进制 provenance

- 主刀面 = 零（pull 零携入，零码变）。
- 码态 = **55c78a219 持位**；runner 二进制字节
  数 **127651344 恒**（与 s538 同代零换代）。
- 窗三 leg1 零 Compiling（同产物零重编）；leg2
  同二进制零重编。

## 双腿 join（窗三，HEAD ad3221ab0 树，make
testharness-html-semantics ×2）

- leg1 01:51:32–01:54:24 / leg2 01:54:24–01:57:07
  （零 lull 背靠背），rc=2 惯例；raw 双腿各 8514
  行同相（norm 8513/8513——常态 8516 的 −3 全落
  族形互换：4 子测行被吞 −4 + 整文件行 +1；空子
  测整文件 Pass 行 link-stylesheet tentative 双腿
  各 1 照旧）。管线全 LC_ALL=C 固化（s488 修复版
  复用）。
- join：**stable 8510 + disagree 1 + l1only 0 +
  l2only 0**（norm 双腿同相 P 集 8307P/202F→198F
  +199F/6T→7T 差 1 全落 detailed 异相键）。
- **vs s538 stable（8514）：NEW 1 + GONE 5 + flips
  0——位移全落族键摆动带（s531 同型带内记账，
  谱系形谱不升级）**：
  - NEW 1 = 族 non-detailed 整文件键 **T/T 双腿
    同相发射**（s531「族 non-detailed 整文件键
    T/T 首双腿同相发射」形复现）。
  - GONE 4 = 族 non-detailed 4 子测键被吞
    （Home/End/PageUp/PageDown）。
  - disagree 1 = 族 detailed 整文件键 **T/F 单键
    异相**（leg1 T + leg2 F——s533 detailed F/T
    异相形的镜像方向，s527→s539 摆动谱续记）。
  - **非族非哨键零 NEW 零 GONE 零 flips 逐键恒
    等**；零新 Pass 形零新 Fail 形（NEW 键 T 形，
    F 形零新——leg2 detailed F 为出位带内形）。
- **P 集账本口径双腿双 8307P 持位第 8 轮**（raw
  双腿双 8308P 同相）。
- **focus-dynamic 121 键 stable 全 Pass**：收窗后
  第 21 轮零再现。
- 双发发射伪影 2 键照旧在册（exact-len + WINdow
  双腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨常规计数首轮）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label
  双键红池照常）——flaky 键双腿双 Pass：零新例
  第 28 轮（s512 时钟顺延）。
- dialog-overlay 哨第 68 轮零翻红（双键双腿
  Pass）。
- **modulepreload 哨：15 键双腿全 Pass——转常规
  连轮计数第 1 轮零翻红**（顺延窗 s535–s538 四轮
  收口后续计）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第
  62 轮保持）/ checkmark Fail（带形不变）。哨复
  核全程 live 状态 join。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行）：
**1672P/0F rc=0**（test 阶段 26.21s——对端 load
16.34 叠跑下时长拉长但零失败；filtered 2405 持
位）。测试清单 vs s538（b538.k）：**1672 set 恒
等，NEW 0 + GONE 0**——绿窗零破，基线 b538.k→
b539.k。

## 对端观察

- 对端 d48 测量腿/build/clippy 循环持续：01:21
  input-events 探针（time-limit 600）→ 01:32+
  01:59 d46-tgt 全量重建（rustc ×3@300%）→
  01:54 python 驱动 → 01:59 clippy
  local_session_create_e2e——对端活跃期巡逻腿窗
  （~7 min）碰撞率高，**守候 v3 静默浸泡门
  （3×45s）为本轮可用先例**（v1 进程门单查、v2
  20s 双查均被叠跑穿透）。
- 对端刀信号持续：ZW-3 d47/d48 刀体未写树（
  querySelector generation-aware 重设计刀为对端
  R5742 首号预计燃点）+ editing 流 B4-S2 残面刀
  + rendering dialog 16 红块协同面 + shapepos 残
  面队列余项 + wptfyi-css 残面——本轮均未入树。

## 结论

s539 收口（守成巡查轮第二十七轮，双污染窗作废+
窗三带内形轮）：码态持位 55c78a219、runner
127651344 恒；窗三 join stable 8510（8307P 双腿
同相）disagree 1 + l1only 0 + l2only 0——vs s538
NEW 1 + GONE 5 + flips 0 **位移全落族键摆动带**
（non-detailed 整文件键 T/T 同相发射 + 4 子测键
被吞 + detailed T/F 异相单键，s531/s533 已入谱
形续记），非族非哨键逐键恒等；P 集双腿双 8307P
持位第 8 轮；flaky 零新例第 28 轮，picker 增益键
第 62 轮保持，onblur/dialogov/waapi 三哨持位，
mpreload 哨常规计数第 1 轮零翻红；全桥 1672 set
恒等绿窗零破；focus-dynamic 收窗后第 21 轮零再
现。双污染窗（窗一 v1 进程门/窗二 v2 20s 双查被
对端 d48 测量腿+build 循环叠跑穿透，塌 67+/71
键形谱作废归档）→ 守候 v3 静默浸泡门（3×45s）
判净起腿先例落地。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137（s445 已提示
，不重复通知）。

下一步：守成巡查（对账基线换 s539 stable 8510
——族键基线形换「non-detailed 整文件 T 形 +
detailed 出位观察形」：s527→s539 摆动谱续记，异
相/出位按形谱记账、Pass 形首现按 NEW 记账；全桥
清单基线 b539.k 1672 持位；flaky 键零新例时钟顺
延；**mpreload 哨常规连轮计数第 2 轮**；他流入
树归因轮或轻覆盖；**对端 d48 测量腿/build 循环
期起腿一律守候 v3 浸泡门**（3×45s 静默浸泡先例
，v1/v2 门已被穿透勿复用）；对端刀信号持续：
ZW-3 d47/d48 刀体落树后按他流刀归因轮 + editing
流 B4-S2 残面刀 + rendering dialog 16 红块协同面
+ css 文法刀后续刀（shapepos 残面队列余项）+
wptfyi-css 残面，join 实测为准）+ 候门决项等待
用户拍板（video P1+P2 立项 / 簇 D A2 立项 /
s137）；rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：簇 D 开
→ customizable-select-styles.css ×47 随批落。

工件：/tmp/s539/{s539-leg1,s539-leg2}.log、
s539-leg{1,2}.rc、leg{1,2}.s539.{start,end}（窗
三）、window.s539.judge（v3 clean-soaked 判词）、
wait-and-run.sh（v3 浸泡门版）、run-legs.sh、
s539-{l1,l2}.norm、l539-{1,2}.canon、s539.new
（NEW 1 族键）、s539.gone（GONE 5 族键）、
s539.flips.txt（零行）、s539.{l1u,l2u,s1,s2,k1,
k2,both.keys,leg1only.keys,leg2only.keys,
disagree.keys,stable.keys,stable.canon}、
s538.stable.{keys,canon}（基线复用）+ s477-{onblur
,dialogov,mpreload,waapi}.keys + fd121.keys +
b538.k（复用）+ b539.k、s539.bridge.{new,gone}
（零行）、s539-bridge.log、s539-bridge.rc +
polluted/（窗一 18 工件）+ polluted2/（窗二 17
工件）+ norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
