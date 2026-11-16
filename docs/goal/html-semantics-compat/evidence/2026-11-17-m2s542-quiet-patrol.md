# M2 s542 — **守成巡查轻覆盖轮**：pull 零携入零码变零换代（码态 f4ca3f641 持位，runner 127779560 恒同码，leg1 Compiling ×6 churn 同形产物同码 s538 先例复现，leg2 零重编）——join **stable 8510（norm 双腿同相 8307P/202F/6T/1U）disagree 0 + l1only 1 + l2only 4**；vs s541 stable **NEW 0 + GONE 0 + flips 0 逐键恒等——全键含族零位移轮**：族单腿异相发射镜像形（s541 l1only 4/l2only 1 → s542 l1only 1/l2only 4 镜像互换，stable 集合零变），非族非哨键零位移逐键恒等

日期：2026-11-17（round start ~05:00，起腿
05:04:40）。承前：s541 收口后 pull 零携入（
ddfce0dbd R5747 docs-only 已在 s541 基线树）。

## 窗净判定与二进制 provenance

- 05:04:40 v3.1 守候器 clean-soaked（procs=0
  load<3 ×3 查 45s），HEAD=648fcb2ab 钉扎起
  腿；双腿 05:04:40–05:10:39 背靠背零 lull：
  leg1 3:23（Compiling ×6 churn）/ leg2 2:36
  常态，rc=2×2 惯例，零塌陷零交叠。
- 码态 = **f4ca3f641 持位**；runner **127779560
  恒**（pre=mid=post 同 size，md5 51fbb16b…
  同码零换代——churn 同形产物同码，s538 先例
  第二次复现）。

## 双腿 join（HEAD 648fcb2ab 树，make
testharness-html-semantics ×2）

- raw norm 双腿 l1 8513 / l2 8516（s541 l1
  8516/l2 8513 的镜像）。管线全 LC_ALL=C
  固化（s488 修复版复用）。
- join：**stable 8510 + disagree 0 + l1only 1
  + l2only 4**（norm 双腿同相 8307P/202F/6T
  /1U）。
- **vs s541 stable（8510）：NEW 0 + GONE 0 +
  flips 0 逐键恒等——全键含族零位移轮**：
  - l1only 1 = 族 non-detailed 整文件键（leg1
    发射 leg2 未发）。
  - l2only 4 = 族 non-detailed 4 子测键（
    Home/End/PageUp/PageDown，leg2 发射
    leg1 未发）。
  - 族单腿异相发射 s541→s542 **镜像互换**（
    l1only 4/l2only 1 → l1only 1/l2only 4）
    ——出位形谱续记，stable 集合两轮全等
    （族 detailed 整文件键 T/T + listbox 2F
    照旧）。
  - 非族非哨键零 NEW 零 GONE 零 flips 逐键恒
    等（leg-only 键零非族）；零新 Pass 形零
    新 Fail 形。
- **P 集账本口径双腿双 8307P 持位第 11 轮**。
- 双发发射伪影 2 键照旧在册（exact len +
  WINdow 双腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 常规计数第 4 轮）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-
  label 双键红池照常）——flaky 键双腿双
  Pass：零新例第 31 轮（s512 时钟顺延）。
- dialog-overlay 哨第 71 轮零翻红（双键双腿
  Pass）。
- **modulepreload 哨：15 键双腿全 Pass——转
  常规连轮计数第 4 轮零翻红**。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益
  第 65 轮保持）/ checkmark Fail（带形不变）。
  哨复核全程 live 状态 join。
- focus-dynamic 121 键双腿全 Pass：收窗后
  第 24 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行，
guard rc=0）：**1682P/0F**（test result 行 ok，
test 阶段 17.49s）。清单 vs s541（b541.k）：
**1682 set 恒等，NEW 0 + GONE 0（diff
SET-IDENTICAL 实证）**——绿窗零破，基线
b541.k→b542.k。

## 对端观察

- 本轮窗净判定期间对端零重腿交叠（05:00–
  05:11 对端无 corpus/测试腿）；对端刀信号
  持续：ZW-3 d47 刀体（querySelector
  generation-aware 重设计刀）+ editing 流
  B4-S2 残面刀 + rendering dialog 16 红块协
  同面 + css 文法刀后续刀（shapepos 残面队
  列余项）+ wptfyi-css 残面（B3 残面 348F
  余项、jsanim flip Steps(Start) 等）——均
  未入树。

## 结论

s542 收口（守成巡查轻覆盖轮）：pull 零携入零
码变零换代（码态 f4ca3f641 持位、runner
127779560 恒同码，leg1 churn ×6 s538 先例第
二次复现）；join stable 8510（8307P 双腿同相）
disagree 0 + l1only 1 + l2only 4——vs s541
NEW 0 + GONE 0 + flips 0 逐键恒等全键含族零
位移轮，族单腿异相发射镜像互换形谱续记；
P 集双腿双 8307P 持位第 11 轮；flaky 零新例
第 31 轮，picker 增益第 65 轮保持，onblur/
dialogov/waapi 三哨持位，mpreload 哨常规计数
第 4 轮零翻红；全桥 1682 set 恒等绿窗零破；
focus-dynamic 收窗后第 24 轮零再现。挂用户门
维持：GB-20261114-VIDEO + GB-20261031（簇 D
）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s542 stable 8510
——族键基线形维持「detailed 整文件 T 形持位
+ non-detailed 整文件/4 子测出位观察形（单腿
异相发射镜像互换）」：异相/出位按形谱记账、
Pass 形首现按 NEW 记账；全桥清单基线 b542.k
1682 持位；flaky 键零新例时钟顺延；**
mpreload 哨常规连轮计数第 5 轮**；他流入树归
因轮或轻覆盖；**起腿一律 v3.1 浸泡门**；对端
刀信号持续：ZW-3 d47 刀体落树后按他流刀归因
轮 + editing 流 B4-S2 残面刀 + rendering
dialog 16 红块协同面 + css 文法刀后续刀
（shapepos 残面队列余项：位置串化规范刀
~498F/round 列表完整建模/inline inherit/
path() 命令级插值/calc 代数插值）+ wptfyi-css
残面（B3 残面 348F 余项、jsanim flip
Steps(Start) 等），join 实测为准）+ 候门决项
等待用户拍板（video P1+P2 立项 / 簇 D A2 立项
/ s137）；rendering 流 dialog 16 红块协同面（
归 rendering 流记账）。门开随批落预案不变：
簇 D 开 → customizable-select-styles.css ×47
随批落。

工件：/tmp/s542/{s542-leg1,s542-leg2}.log、
s542-leg{1,2}.rc、leg{1,2}.s542.{start,end}、
legs.head（648fcb2ab）、runner.{pre,mid,post}
.{size,md5}（同码恒证）、window.s542.judge、
wait-and-run.sh（v3.1 版）、run-legs.sh、
s542-{l1,l2}.norm、l542-{1,2}.canon、
s542.new.keys/gone.keys（零行）、s542.flips.txt
（零行）、s542.{l1u,l2u,s1,s2,k1,k2,both.keys
,leg1only.keys,leg2only.keys,disagree.keys,
stable.keys,stable.canon}、s541.stable.{keys,
canon}（基线复用）+ s477-{onblur,dialogov,
mpreload,waapi}.keys + fd121.keys + b541.k（
复用）+ b542.k（1682）、s542.bridge.{new,gone}
（零行）、s542-bridge.log、s542-bridge.rc（
guard rc=0）+ norm.sh/canon.sh/join.sh（s488
修复版管线复用，全 LC_ALL=C）。
