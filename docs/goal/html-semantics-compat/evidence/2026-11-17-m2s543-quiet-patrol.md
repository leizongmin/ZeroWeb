# M2 s543 — **守成巡查轻覆盖轮**：pull 零携入零码变零换代（码态 f4ca3f641 持位，runner 127779560 恒同码第三代三连，leg1 3:04 / leg2 2:40 常态零重编）——join **stable 8513（stable canon 8305P/202F/5T/1U）disagree 1 + l1only 0 + l2only 0**；vs s542 stable **NEW 4 + GONE 1 + flips 0——位移全落族键摆动带（混合回归形）**：NEW 4 = 族 non-detailed 4 子测键 F/F 双腿同相发射（s541 l1only 4 / s542 l2only 4 单腿异相 → 本轮同相回归），GONE 1 = 族 detailed 整文件键本轮 T/F 异相 disagree 出位（s541 形复现），族 non-detailed 整文件键双腿均被吞——**P 集 stable 口径 8305P 两轮恒等**，非族非哨键零位移逐键恒等

日期：2026-11-17（round start ~05:18，起腿
05:25:12）。承前：s542 收口后 pull 零携入（
远端无新提交）。

## 窗净判定与二进制 provenance

- 窗前门内负载两起：mykernel riscv release
  build（rustc 526% CPU）+ ZeroSeed FR028 验
  收腿（test-guard+xvfb）——v3.1 守候器进程
  门计入，二者收线后判净：05:25:12
  clean-soaked procs=0 load<3 ×3 查 45s，
  **HEAD=b7cfe9340 钉扎起腿**。
- 双腿 05:25:12–05:31 背靠背零 lull：leg1
  3:04 / leg2 2:40＝常态，rc=2×2 惯例，零塌
  陷零交叠。
- 码态 = **f4ca3f641 持位**；runner
  **127779560 恒**（pre=mid=post 同 size，md5
  51fbb16b… 同码零换代，s541 起第三代恒同
  码三连）。

## 双腿 join（HEAD b7cfe9340 树，make
testharness-html-semantics ×2）

- raw norm 双腿各 8516 行同形（leg1 全集
  8307P/202F/6T/1U ＝ s541/s542 口径同相；
  leg2 8307P/203F/5T/1U，+1F−1T 全落族
  detailed 键 T/F 异相）。管线全 LC_ALL=C
  前缀重跑（zh_CN 会话首跑踩序坑，二次实证
  后固化工法）。
- join：**stable 8513 + disagree 1 + l1only 0
  + l2only 0**（stable canon 8305P/202F/5T
  /1U）。
- **vs s542 stable（8510）：NEW 4 + GONE 1 +
  flips 0——位移全落族键摆动带（混合回归形
  ）**：
  - NEW 4 = 族 non-detailed 4 子测键
    （Home/End/PageUp/PageDown）**F/F 双腿
    同相发射**（s541 单腿异相 l1only 4 / s542
    镜像 l2only 4 → 本轮同相回归＝A 形成分）
    。
  - GONE 1 = 族 detailed 整文件键（s542
    T/T stable → 本轮 leg1 T / leg2 F 异相
    disagree 出位＝s541 形复现）。
  - 族 non-detailed 整文件键本轮双腿均被吞
    （零行；s541/s542 单腿异相形 → 本轮零发
    射）。
  - **P 集 stable 口径 8305P 两轮恒等**（U
    同 1 恒等；F 198→202 +4 = NEW 族 F 键、
    T 6→5 −1 = GONE detailed T 键，全数落
    族带）；非族非哨键零 NEW 零 GONE 零
    flips 逐键恒等（零新 Pass 形零新 Fail 形
    ，NEW 键全 F 形）。
- 双发发射伪影 2 键照旧在册（exact len +
  WINdow 双腿各 2 dup）。

## 观察哨复核（四哨持位 + mpreload 常规计数第 5 轮）

- onblur 哨组 10 键双腿 join 零 flip（8P/2F
  parent-label 红池照常）——flaky 键双腿双
  Pass：零新例第 32 轮（s512 时钟顺延）。
- dialog-overlay 哨第 72 轮零翻红（双键双腿
  零 flip）。
- **modulepreload 哨：15 键双腿零 flip——转
  常规连轮计数第 5 轮零翻红**。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益
  第 66 轮保持）/ checkmark Fail（带形不变）
  。
- focus-dynamic 121 键双腿全 Pass：收窗后
  第 25 轮零再现。
- 哨复核全程 awk hash join（免疫 join/comm
  排序序敏感性，本轮工法固化）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行，
guard rc=0）：**1682P/0F**（test result 行 ok，
test 阶段 17.45s）。清单 vs s542（b542.k）：
**1682 set 恒等，NEW 0 + GONE 0（diff
SET-IDENTICAL 实证）**——绿窗零破，基线
b542.k→b543.k。

## 对端观察

- 本轮窗净判定期间门内非本流负载（mykernel
  build + ZeroSeed 验收腿）收线后判净；对端
  ZeroWeb 流零重腿交叠。对端刀信号持续：
  ZW-3 d47 刀体（querySelector
  generation-aware 重设计刀）+ editing 流
  B4-S2 残面刀 + rendering dialog 16 红块协
  同面 + css 文法刀后续刀（shapepos 残面队
  列余项）+ wptfyi-css 残面（B3 残面 348F
  余项、jsanim flip Steps(Start) 等）——均
  未入树。

## 结论

s543 收口（守成巡查轻覆盖轮）：pull 零携入
零码变零换代（码态 f4ca3f641 持位、runner
127779560 恒同码第三代三连）；join stable
8513（8305P stable 口径两轮恒等）disagree 1
+ l1only 0 + l2only 0——vs s542 NEW 4 + GONE
1 + flips 0 位移全落族键摆动带（混合回归形
：4 子测 F/F 同相回归 + detailed T/F 异相出
位 + 整文件键吞形），非族非哨键逐键恒等；
flaky 零新例第 32 轮，picker 增益第 66 轮保
持，onblur/dialogov/waapi 三哨持位，mpreload
哨常规计数第 5 轮零翻红；全桥 1682 set 恒等
绿窗零破；focus-dynamic 收窗后第 25 轮零再
现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不
重复通知）。

下一步：守成巡查（对账基线换 s543 stable
8513——族键基线形换「detailed 整文件 T/F 异
相观察形（s541 形）+ non-detailed 4 子测 F/F
同相在册形 + non-detailed 整文件键吞形观察
」：异相/出位按形谱记账、Pass 形首现按 NEW
记账；全桥清单基线 b543.k 1682 持位；flaky
键零新例时钟顺延；**mpreload 哨常规连轮计数
第 6 轮**；他流入树归因轮或轻覆盖；**起腿一
律 v3.1 浸泡门**（本轮 mykernel/ZeroSeed 非
本流负载门内计入实证有效）；对端刀信号持续
：ZW-3 d47 刀体落树后按他流刀归因轮 +
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

工件：/tmp/s543/{s543-leg1,s543-leg2}.log、
s543-leg{1,2}.rc、leg{1,2}.s543.{start,end}、
legs.head（b7cfe9340）、runner.{pre,mid,post}
.{size,md5}（同码恒证）、window.s543.judge、
waiter.s543.log、wait-and-run.sh（v3.1 版）、
run-legs.sh、s543-{l1,l2}.norm、l543-{1,2}
.canon、s543.new.keys（NEW 4 族键）、
s543.gone.keys（GONE 1 族键）、s543.flips.txt
（零行）、s543.{l1u,l2u,s1,s2,k1,k2,both.keys
,leg1only.keys,leg2only.keys,disagree.keys,
stable.keys,stable.canon}、s542.stable.{keys,
canon}（基线复用）+ s477-{onblur,dialogov,
mpreload,waapi}.keys + fd121.keys + b542.k（
复用）+ b543.k（1682）、s543.bridge.{new,gone}
（零行）、s543-bridge.log、s543-bridge.rc（
guard rc=0）+ norm.sh/canon.sh/join.sh（
s488 修复版管线复用，本轮起调用侧 export
LC_ALL=C 前缀固化）。
