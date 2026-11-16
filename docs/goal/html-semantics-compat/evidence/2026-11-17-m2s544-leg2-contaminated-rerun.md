# M2 s544 — **守成巡查轻覆盖轮 + leg2 污染作废单腿重跑先例**：pull 零携入零码变零换代（码态 f4ca3f641 持位，runner 127779560 恒同码四轮连持，leg1 3:50 / leg2b 2:33 常态）——首轮 leg2（4:53 慢腿）在对端复燃交叠窗（load 5min 峰 20.05）劣化＝10T（+5 整文件 Timeout 键）+ 61 子测 Pass 键被吞，**甄别作废**；leg1 验干净保留（vs s543 leg1 位移全落族带），v3.1 浸泡门重跑 leg2b 干净腿——join **stable 8509（stable canon 8305P/198F/5T/1U）disagree 2 + l1only 0 + l2only 3**；vs s543 stable **NEW 0 + GONE 4 + flips 0——位移全落族键摆动带**：GONE 4 = 族 non-detailed 4 子测键本轮双腿异相出位（l1 全吞 / l2b 发射 3 键 F 形 + PageDown 双腿吞），两整文件键双腿 F/T 异相 disagree 出位——**P 集 stable 口径 8305P 两轮恒等**（全集口径双腿双 8307P 第 12 轮），非族非哨键零位移逐键恒等

日期：2026-11-17（round start ~06:20 计，首轮起腿
05:45:00，重跑起腿 06:33:54）。承前：s543 收口
后 pull 零携入（远端无新提交）。

## 腿级污染甄别与单腿重跑（本轮工法先例）

- 首轮双腿：05:45:00 v3.1 判净 HEAD=c3a170f53
  钉扎起腿。leg1 05:45:00–05:48:50（3:50，rc=
  2 惯例）；leg2 05:48:50–05:53:43（**4:53 慢
  腿**，比常态 2:40 慢 2 分钟）。
- **leg2 劣化甄别**：l2 全集 8456 行（比 l1 少
  57 行）＝10T（+5 整文件 Timeout 键：
  select-event/focus-dynamic×2/listbox-mouse
  /dialog-complex）+ 61 子测 Pass 键被吞 + 1
  Fail 键被吞；l2only 5 键全为其超时文件的整
  文件键形——**对端复燃交叠窗实锤**（leg2 窗
  内 load 5min 峰 20.05，渲染流 reftest-upstream
  重腿复燃两波：05:57+/06:14+）。
- **甄别保留 leg1**：vs s543 leg1 全集对账 NEW
  1 + GONE 4 位移全落族带（4 子测键吞形 +
  non-detailed 整文件键 F 形发射），非族键零
  位移——leg1 干净形可复用（树态恒 runner 同
  码前提，区别于 s540 树态变整轮重跑先例）。
- **单腿重跑 leg2b**：rerun 浸泡门（v3.1 同构
  单腿版，deadline 7200s）在对端两波重腿收线
  后 06:33:54 判净起腿，06:33:54–06:36:27（
  **2:33 常态时长**，rc=2）。污染腿工件保留
  在册（s544-leg2.log / s544-l2.norm 首版）供
  甄别复核。

## 稳定 Timeout 带定性（惰载 5 键）

- style-error-01 / audio-loading-lazy-source
  -deferred / sizes-auto / update-media /
  video-loading-lazy-source-poster-deferred 5
  键 s543/s544 两轮同为 Timeout——**历史常态
  带**（s543 T=6 = 惰载 5 + 族 detailed；s544
  leg1 T=5 = 惰载 5，leg2b T=7 = 惰载 5 + 族
  两整文件键 T 形）。历轮 T 计数 5-6 即此带，
  非新劣化。

## 双腿 join（HEAD c3a170f53 树，干净腿）

- raw norm：l1 8513 / l2b 8516（全集 l1
  8307P/200F/5T/1U、l2b 8307P/201F/7T/1U——
  **全集口径双腿双 8307P 持位第 12 轮**）。
  管线 export LC_ALL=C 前缀（s543 工法固化
  后首用零踩坑）。
- join：**stable 8509 + disagree 2 + l1only 0
  + l2only 3**（stable canon 8305P/198F/5T
  /1U）。
- **vs s543 stable（8513）：NEW 0 + GONE 4 +
  flips 0——位移全落族键摆动带**：
  - GONE 4 = 族 non-detailed 4 子测键（
    Home/End/PageUp/PageDown）本轮双腿异相
    出位：l1 全吞 / l2b 发射 3 键 F 形
    （l2only 3）+ PageDown 双腿吞。
  - disagree 2 = 族两整文件键双腿 F/T 异相
    出位（l1 F 形发射 / l2b T 形；s543 的
    detailed T/F 反向异相形本轮反向复现）。
  - **P 集 stable 口径 8305P 两轮恒等**（U
    同 1 恒等；F 202→198 −4 全落 GONE 族 F
    键、T 5→5 恒）；非族非哨键零 NEW 零
    GONE 零 flips 逐键恒等（零新 Pass 形零
    新 Fail 形，位移键全 F/T 形）。
- 双发发射伪影 2 键照旧在册（exact len +
  WINdow 双腿各 2 dup）。

## 观察哨复核（干净腿 join，四哨持位 + mpreload 常规计数第 6 轮）

- onblur 哨组 10 键双腿 join 零 flip（8P/2F
  parent-label 红池照常）——flaky 键双腿双
  Pass：零新例第 33 轮（s512 时钟顺延）。
- dialog-overlay 哨第 73 轮零翻红（双键双腿
  零 flip）。
- **modulepreload 哨：15 键双腿零 flip——转
  常规连轮计数第 6 轮零翻红**。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益
  第 67 轮保持）/ checkmark Fail（带形不变）。
- focus-dynamic 121 键双腿全 Pass：收窗后
  第 26 轮零再现（污染 leg2 中该域曾整文件
  超时，重跑干净腿后复验全 Pass——哨复核以
  干净腿为准）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行，
guard rc=0）：**1682P/0F**（test result 行 ok，
test 阶段 32.32s）。清单 vs s543（b543.k）：
**1682 set 恒等，NEW 0 + GONE 0（diff
SET-IDENTICAL 实证）**——绿窗零破，基线
b543.k→b544.k。

## 对端观察

- 对端渲染流本窗两波重腿复燃（05:57+ / 06:14+
  reftest-upstream，load 峰 20.05/16.12）——
  首轮 leg2 恰被第二波交叠污染；重跑窗 06:33
  收线后判净零交叠。
- 对端刀信号持续：ZW-3 d49 写面 WIP 在途未
  落树（R5750 实测：part01.js +260/−4 FD/BS
  可见单元删除核共用件、querySelector 写面
  未启动）+ ZW-3 d47 刀体 + editing 流 B4-S2
  残面刀 + rendering dialog 16 红块协同面 +
  css 文法刀后续刀（shapepos 残面队列余项）+
  wptfyi-css 残面（B3 残面 348F 余项、jsanim
  flip Steps(Start) 等）——均未入树。

## 结论

s544 收口（守成巡查轻覆盖轮 + leg2 污染作废
单腿重跑先例）：pull 零携入零码变零换代（码
态 f4ca3f641 持位、runner 127779560 四轮连
持）；首轮 leg2 对端交叠窗污染甄别作废（10T
+61 键被吞+整文件 T 键形甄别证据）、leg1 验
干净保留、leg2b 浸泡门重跑 2:33 常态；join
stable 8509（8305P stable 口径两轮恒等、全集
口径双 8307P 第 12 轮）disagree 2 + l1only 0
+ l2only 3——vs s543 NEW 0 + GONE 4 + flips 0
位移全落族键摆动带（4 子测键双腿异相出位 +
两整文件键 F/T 异相新形），非族非哨键逐键恒
等；惰载 5 键稳定 Timeout 带定性；flaky 零新
例第 33 轮，picker 增益第 67 轮保持，onblur/
dialogov/waapi 三哨持位，mpreload 哨常规计数
第 6 轮零翻红；全桥 1682 set 恒等绿窗零破；
focus-dynamic 收窗后第 26 轮零再现。挂用户门
维持：GB-20261114-VIDEO + GB-20261031（簇 D
）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s544 stable
8509——族键基线形换「4 子测键出位观察形（
本轮双腿异相，s541 形域）+ 两整文件键 F/T
异相出位形（本轮新形：l1 F / l2b T）」：异
相/出位按形谱记账、Pass 形首现按 NEW 记账；
全桥清单基线 b544.k 1682 持位；flaky 键零新
例时钟顺延；**mpreload 哨常规连轮计数第 7
轮**；他流入树归因轮或轻覆盖；**起腿一律
v3.1 浸泡门；慢腿/劣化腿按本轮甄别规程处置
（整文件 T 键形 + 子测键成批被吞 + 慢 2 分
钟三信号联判，验他腿干净则单腿重跑，树态变
则整轮重跑）**；对端刀信号持续：ZW-3 d49 写
面 WIP 落树后按他流刀归因轮 + d47 刀体 +
editing 流 B4-S2 残面刀 + rendering dialog
16 红块协同面 + css 文法刀后续刀（shapepos
残面队列余项：位置串化规范刀 ~498F/round 列
表完整建模/inline inherit/path() 命令级插值
/calc 代数插值）+ wptfyi-css 残面（B3 残面
348F 余项、jsanim flip Steps(Start) 等），
join 实测为准）+ 候门决项等待用户拍板
（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：簇
D 开 → customizable-select-styles.css ×47
随批落。

工件：/tmp/s544/{s544-leg1,s544-leg2,s544
-leg2b}.log（污染腿 log 在册）、s544-leg{1,2
,2b}.rc、leg{1,2,2b}.s544.{start,end}、legs
.head（c3a170f53）、runner.{pre,mid,post}
.{size,md5}（同码恒证）、window.s544.judge、
window.s544.rerun、rerun-leg2.sh（单腿浸泡
门先例脚本）、waiter.s544.log、wait-and-run
.sh（v3.1 版）、run-legs.sh、s544-{l1,l2,l2b}
.norm、l544-{1,2}.canon（2=leg2b 重生成）、
s544.new.keys（零行）、s544.gone.keys（GONE 4
族键）、s544.flips.txt（零行）、s544.{l1u
,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,
stable.canon}、s543.stable.{keys,canon}（基
线复用）+ s477-{onblur,dialogov,mpreload,
waapi}.keys + fd121.keys + b543.k（复用）+
b544.k（1682）、s544.bridge.{new,gone}（零行
）、s544-bridge.log、s544-bridge.rc（guard
rc=0）+ norm.sh/canon.sh/join.sh（s488 修复
版管线，export LC_ALL=C 前缀调用）。
