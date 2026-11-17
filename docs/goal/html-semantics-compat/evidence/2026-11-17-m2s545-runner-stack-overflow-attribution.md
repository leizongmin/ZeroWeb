# M2 s545 — **他流刀归因轮 + runner 爆栈归因修复轮（先例）**：上 session 429 断线夜 waiter 自动起腿双双 runner 栈溢出 fatal abort（0 结果行），三级诊断链归因到他流 css 刀 D rule-inset（ef6f2cf5a）加深 style/gCS 帧 × runner 并行道 2MiB 栈欠账——`parallel_map` rayon 池补 `.stack_size(256MB)` 与主道同规格（先例 main.rs:406 同族类），RUST_MIN_STACK=256MB 实证全语料恢复后固化入码——修复后 v3.1 浸泡门重跑双腿常态全绿：join **stable 8510（stable canon 8305P/198F/6T/1U）disagree 1 + l1only 3 + l2only 0**；vs s544 stable **NEW 1 + GONE 0 + flips 0——位移全落族键摆动带**（NEW 1 = 族 non-detailed 整文件键 T/T 同相入 stable，Timeout 形 NEW 记账；disagree 1 = 族 detailed 整文件键 T/F 异相，s544 反向形复现；l1only 3 = 族 3 子测键 F 形 l1 单腿发射 + PageDown 双腿吞持续），非族非哨键零位移逐键恒等

日期：2026-11-17（崩溃腿 ~08:07 断线夜自动起腿，
诊断 09:14–09:47，正腿起 09:51:35）。承前：s544
收口后本 session 起 pull 携入六刀窗（d087044b7
→1baacb05e）。

## 崩溃腿（上 session 断线夜，工件改名 crash-* 在册）

- 断线前 session 已置 v3.1 waiter（/tmp/s545），
  429 限流窗口内 08:07:16 自动判净起腿（HEAD=
  d087044b7 钉扎——彼树已含 rule-inset 刀 D
  + ILB-SELDEL + T77，未含本 session 携入的
  后六刀）。
- leg1 08:07:16–08:08:54（1:38）/ leg2 08:08:
  54–08:10:20（1:26）——双双 rc=2 非惯例形：
  `thread '<unknown>' has overflowed its stack
  / fatal runtime error: stack overflow,
  aborting`，**0 结果行**（runner 结果行末统
  一输出，崩点无法从日志定位）；runner 换代
  51fbb16b→5d1d4e1e（+202,392，刀 D+ILB-
  SELDEL+T77 入码）。
- 甄别：fatal abort 形一票作废（rc=2 惯例形
  与 rc=2+fatal 形靠日志 grep 区分）——工件
  crash-* 保留在册，双腿重跑。

## 诊断链（09:14–09:47）

1. **HEAD 复现**：pull 后 HEAD=1baacb05e 诊断
   腿同形 fatal abort（0 结果行）——六刀未消
   除，肇事窗锁定更早。
2. **FILTER 小域探针**：forms/the-output-
   element（5 案）rc=0 干净跑通——非起动通
   用崩，触发面在语料深部。
3. **git bisect** f4ca3f641（good，s544 四轮
   干净）→ d087044b7（bad）：c3a170f53 干净
   / 033e2f51c（ILB-SELDEL 码在树）干净 /
   efdb312e6（rule-inset 首档）崩——**肇事 =
   ef6f2cf5a 刀 D rule-inset 全链＋inline 站
   （20 文件 1919 行，wptfyi-css/rendering
   流）**。bisect 工法注记：f3b8480e0 假 good
   作废（外层超时 SIGTERM 杀死 probe 于编译
   期，脚本 grep 无爆栈仍退出 0）——**探针
   good 判据必须绑定正向完成证据（结果行数
   >0），仅「无崩溃」不构成 good**。
4. **根因**：tests/wpt-runner `parallel_map`
   rayon 池（8 worker）默认 2MiB 栈 × 刀 D
   加深的 style/gCS 每层帧——深树页 style 递
   归（compute_styles_recursive 族）爆栈。
   runner 主道 256MB 大栈先例（main.rs:406，
   注释点名 shadow-dom 999 层嵌套 case + 「
   style 递归迭代化」挂账）未覆盖并行道——
   `parallel_map` 为五调用点共用（testharness
   /reftest/oracle/struct-sweep）。
5. **实证**：RUST_MIN_STACK=268435456 全语
   料跑通（0 爆栈 rc=2 惯例）。
6. **修复**：rayon::ThreadPoolBuilder 补
   `.stack_size(256 * 1024 * 1024)`（与主道
   同规格，注释引用先例与本例）——HEAD 全语
   料 8517 结果行 0 爆栈 rc=2。

## 归因定性

- 刀 D 语义面对端申报成立（五 corpus 净
  +1545 零回归零丢页 + engine lib 4116/0F
  + census 22 页，css 面实测绿）——本流爆栈
  系**栈深预算交互**（runner 并行道欠账 × 刀
  加深帧），非语义回归。
- 修复归 runner 通道（本流携带，刀窗归因与
  先例引用均注释在码面）；对端 reftest/oracle
  道同一 helper 同险敞口，修复随 main 下轮
  对其生效。
- 「申报=实测」谱系**第三十三例（六刀窗零语
  义位移清偿）**：对端申报（刀 D +1545 / 刀
  E CSSA +3260 / 刀 F neutral +47 / d49
  input-events +75 / ILB-SELTBL 十族全清 /
  T78 内存仪拍）与本流 join 实测互证——非族
  非哨键零位移。

## 双腿 join（HEAD 1baacb05e 树，修复后）

- v3.1 判净 09:51:35 HEAD=1baacb05e 钉扎起
  腿。leg1 09:51:35–09:54:07（2:32）/ leg2
  09:54:07–09:56:49（2:42）常态，rc=2 惯例，
  0 爆栈。runner 2524d01f（128,042,560）恒
  同码双腿零重编（leg1 Finished 0.18s）。
- raw canon：l1 8307P/201F/7T/1U、l2 8307P
  /199F/6T/1U——**全集口径双腿双 8307P 第
  13 轮持位**。
- join：**stable 8510 + disagree 1 + l1only
  3 + l2only 0**（stable canon 8305P/198F
  /6T/1U）。
- **vs s544 stable（8509）：NEW 1 + GONE 0 +
  flips 0——位移全落族键摆动带**：
  - NEW 1 = 族 non-detailed 整文件键 T/T
    同相入 stable（**Timeout 形 NEW 记账**
    ——Pass 形首现才升级形谱）；s544 该键
    双腿 F 形出位，本轮同相 T 形回归。
  - disagree 1 = 族 detailed 整文件键 T/F
    异相（l1 T / l2 F——s544 F/T 反向形复
    现，方向翻转）。
  - l1only 3 = 族 3 子测键 End/Home/
    PageUp F 形 l1 单腿发射（s544 l2b 同形
    镜像），PageDown 双腿吞持续。
  - **P 集 stable 口径 8305P 第三轮恒等**；
    惰载 5 键 stable T 带持位（stable T
    5→6 = +1 族键 T 形，非惰载新劣化）。
  - 非族非哨键零 NEW 零 GONE 零 flips 逐
    键恒等（零新 Pass 形零新 Fail 形）。
- 双发发射伪影 2 键照旧在册（exact len +
  WINdow）。

## 观察哨复核（干净腿 join，四哨持位 + mpreload 常规计数第 7 轮）

- onblur 哨组 10 键双腿 join 零 flip（8P/2F
  parent-label 红池照常）——flaky 键（
  checkbox/radio following-label 双行）双腿
  双 Pass：零新例第 34 轮（s512 时钟顺延）。
- dialog-overlay 哨第 74 轮零翻红（双键双腿
  零 flip）。
- **modulepreload 哨：15 键双腿零 flip——常
  规连轮计数第 7 轮零翻红**（s544 排程兑现）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增
  益第 68 轮保持）/ checkmark Fail（带形不变）。
- focus-dynamic 121 键双腿全 Pass：收窗后
  第 27 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test
-p zero-engine --lib js_dom_bridge`（仓根执
行，guard rc=0）：**1690P/0F**（test 阶段
21.62s）。清单 vs s544（b544.k 1682）：
**1690，NEW 8 + GONE 0——NEW 8 全归属刀窗**
：6× 刀 D rule-inset（rule_inset_gcs_
serialization_tests ×4 + kd_rule_inset_
inline_pins + eased_one_rule_inset_calc_
mix）+ 1× ILB-SELDEL（test_ilb_seldel_pins）
+ 1× ILB-SELTBL（test_ilb_seltbl_pins =
part131）。绿窗零破，基线 b544.k→b545.k。

## 对端观察

- 六刀窗全数在树：刀 D rule-inset（肇事刀，
  语义面绿、栈深交互触发本流崩溃——修复已
  固化）/ ILB-SELDEL / ILB-SELTBL / 刀 E
  CSSA / 刀 F neutral / d49 可见单元删除核
  / T77-T78 renderer 系列（PR #308/#309）。
- 对端 reftest/oracle/struct-sweep 三道与
  本流 testharness 共用 `parallel_map`——
  修复随 main 下轮对其生效；若他流再遇
  fatal stack overflow，按本轮诊断链处置
  （HEAD 复现 → 小域探针定位触发面 → bisect
  窗 → RUST_MIN_STACK 实证 → 池级固化）。

## 结论

s545 收口（他流刀归因轮 + runner 爆栈归因修
复轮先例）：六刀窗 pull 携入（码态
f4ca3f641→1baacb05e，runner 三代 51fbb16b→
5d1d4e1e→2524d01f）；断线夜自动起腿双双爆
栈作废（fatal abort 形甄别先例）；三级诊断
链归因刀 D 栈深交互、rayon 池 256MB 固化修
复、实证恢复；修复后浸泡门双腿常态全绿——
join stable 8510（8305P stable 口径第三轮恒
等、全集口径双 8307P 第 13 轮）disagree 1 +
l1only 3 + l2only 0，vs s544 NEW 1 + GONE 0
+ flips 0 位移全落族键摆动带（族键 T 形同相
回归 + detailed 异相反向 + 3 子测键单腿发射
带），非族非哨键逐键恒等；惰载 5 键 T 带持
位；flaky 零新例第 34 轮，onblur/dialogov/
waapi 三哨持位（picker 增益第 68 轮），
mpreload 哨常规计数第 7 轮零翻红；全桥 1690
P/0F 清单 NEW 8 全归属刀窗（b545.k）；fd121
第 27 轮零再现。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137（s445 已
提示，不重复通知）。

下一步：守成巡查（对账基线换 s545 stable
8510——族键基线形换「non-detailed 整文件键
T 持位形 + detailed 整文件键异相观察形（本
轮 T/F 向）+ 3 子测键单腿发射带 + PageDown
双腿吞带」：异相/出位按形谱记账、Pass 形首
现按 NEW 记账；全桥清单基线 b545.k 1690 持
位；flaky 键零新例时钟顺延；**mpreload 哨
常规连轮计数第 8 轮**；他流入树归因轮或轻
覆盖；起腿一律 v3.1 浸泡门；慢腿/劣化腿按
s544 甄别规程处置、**fatal abort 形按本轮
一票作废处置**；对端刀信号持续：css 文法刀
后续（shapepos 残面队列余项：位置串化规范刀
~498F/round 列表完整建模/inline inherit/
path() 命令级插值/calc 代数插值）+ wptfyi-css
残面（mask-border 家族支持门定谳、B3 残面
348F 余项、jsanim flip Steps(Start)）+
wptfyi-editing d49 后续 + ZW-3 d47 刀体 +
editing 流 B4-S2 残面 + rendering dialog 16
红块协同面，join 实测为准）+ 候门决项等待
用户拍板（video P1+P2 立项 / 簇 D A2 立项 /
s137）；rendering 流 dialog 16 红块协同面（
归 rendering 流记账）。门开随批落预案不变：
簇 D 开 → customizable-select-styles.css
×47 随批落。

工件：/tmp/s545/{crash-s545-leg1,crash-s545
-leg2}.log（爆栈腿在册）+ crash-{leg1,leg2,
legs}.{start,end,done,rc} + crash-window.
judge + diag-head.{log,rc,start,end}（HEAD
复现）+ diag-filter.log（小域干净）+
bisect.log（肇事档探针）+ diag-rms.{log,rc,
start,end}（RUST_MIN_STACK 实证）+ diag-fix.
{log,rc,start,end}（修复后全语料 8517 行）+
bisect-probe.sh（探针脚本，good 判据教训在
册）+ s545-leg{1,2}.log/.rc、leg{1,2}.s545.
{start,end}、legs.head（1baacb05e）、runner.
{pre,mid,post}.{size,md5}（2524d01f 恒同码）
+ window.s545.judge（09:51:35 clean-soaked）
+ waiter.s545b.log + run-legs.sh/wait-and-
run.sh（v3.1 复用）+ s545-{l1,l2}.norm、
l545-{1,2}.canon、s545.{l1u,l2u,s1,s2,k1,k2
,both.keys,leg1only.keys,leg2only.keys,
disagree.keys,stable.keys,stable.canon}、
s545.new.keys（NEW 1 族键）、s545.gone.keys
（零行）+ x.l{1,2}.{add,del}（全集口径对账
）+ s544.stable.{keys,canon} + s544.{l1u,
l2u} + l544-{1,2}.canon（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys
+ fd121.keys + b544.k（复用）+ b545.k（1690
）、s545.bridge.{new,gone}（NEW 8/GONE 0）
+ s545-bridge.log、s545-bridge.rc（guard
rc=0）+ norm.sh/canon.sh/join.sh（s488 修复
版管线，export LC_ALL=C 前缀调用）。
