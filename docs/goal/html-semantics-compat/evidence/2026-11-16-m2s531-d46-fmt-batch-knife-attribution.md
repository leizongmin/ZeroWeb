# M2 s531 — 他流刀归因轮（双刀窗）：editing 流 d46 键入替换臂主刀 8a7c20773（part01.js +115/−9 唯一产品变更——非塌缩选区键入先删后插 `_zwCeSelectDelete` 第 4 参 typingData + gtr 端点直取空白门，s530 收口 push 前置 pull 携入，对端申报 input-events 1099P→1135P 净 +36）+ fmt 伴刀 2c8742499（crates/dom tests_9_query_coverage.rs +17/−6 + tests/wpt-runner testharness.rs +4/−2，零语义格式漂移修复）落树归因——html-semantics 面**零语义位移清偿，「申报=实测」谱系第二十八例（双刀窗）**；码态换新 2c8742499（runner 换代 127558768 → 127567632，+8,864 刀载荷，leg1 真重建 Compiling ×6，leg2 零重编）——vs s530 stable **NEW 1 + GONE 5 + flips 0**，位移全落**族键摆动带（NEW 1 + GONE 4 带内形变）+ mpreload 哨键单腿瞬态（GONE 1）**——非族非哨键零 NEW 零 GONE 零 flips 逐键恒等；join stable 8510（8304P/198F/7T/1U）disagree 1 + l1only 0 + l2only 0（disagree = mpreload 哨 unsafe-url 键 F/P 单腿瞬态）；P 集账本口径 l1 8306 / l2 8307（51 轮双 8307P 持位终止——差 1 全归哨键瞬态）+ flaky 零新例第 20 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨零翻红连轮第 62 轮终止（unsafe-url 键 leg1 单腿瞬态 Fail，同二进制 leg2 Pass = 环境瞬态非刀致，时钟重置）** + 全桥绿窗 1671P/0F rc=0 清单 set 恒等（NEW 0/GONE 0——part01.js shim 刀体零入桥清单，行为变更直过全桥 1671 断言零伤既有 DOM 桥语义）+ focus-dynamic 收窗后第 13 轮零再现

日期：2026-11-16。承前：s530 收口 push 前置 pull 携入对
端 editing 流双刀窗 **8a7c20773**（fix(editing): d46
TypingA 替换臂；committed 后随 2c8742499 fmt 伴刀）→
按他流刀归因轮（双刀窗合并归因，R5734 渲染流双刀批
先例同构）。主刀申报：input-events 套件 1099P→1135P
净 +36（TypingA 替换臂 69F 残面收口：`<p>{abc}</p>`
+'a' 期望 `<p>a</p>` 刀前 got `<p>aabc</p>`）。

## 二进制 provenance

- runner 换代 **127558768 → 127567632**（+8,864 双刀
  载荷；leg1 真重建 `Compiling` ×6（product-version/
  net/engine/page-runtime/webview/wpt-runner——主刀
  part01.js include_str 入 engine 链 + 伴刀
  testharness.rs 直触 runner 本体；css-parser/style-
  system 零变零重编，dom 测试面不在 runner 二进制链
  ），21:54:48 起腿；leg2 同二进制零重编）。
- 码态 = **2c8742499**（HEAD 8fd1cfb3f = 本流 s530 收
  口 docs-only 落基）；主刀刀体自身 = `git show
  8a7c20773` = 恰 3 文件（part01.js +115/−9 唯一产品
  变更 + 对端 docs evidence +106 + wptfyi-editing
  master +44/−32）= **与对端申报逐文件全等**；伴刀
  2c8742499 = 2 文件零语义 fmt（dom 测试 + runner
  harness 各自 fmt 漂移修复）。

## 双腿 join（HEAD 8fd1cfb3f 树 = 码态 2c8742499，
make testharness-html-semantics ×2）

- leg1 21:54:48–21:58:07 / leg2 21:58:07–22:00:49（零
  lull 背靠背），raw/norm 8513/8513（vs s530 8516 =
  −4 族 4 子测 F 行被整文件超时吞发射 +1 族整文件 T
  行首发射；双腿各 +2 行双发发射 dup：exact-len +
  WINdow 伪影照旧各 2 dup）——双腿零截断零真 hang
  （rc=2 惯例失败退出，时长正常）。本轮管线全
  LC_ALL=C 固化。
- join：**stable 8510（8304P/198F/7T/1U）+ disagree 1
  + l1only 0 + l2only 0**——disagree 1 = mpreload 哨
  unsafe-url 键（leg1 Fail / leg2 Pass，单腿瞬态），
  族键双腿同相零分歧。
- **vs s530 stable（8514）：NEW 1 + GONE 5 + flips 0
  ——位移全落带内**：
  - NEW 1 = 族 non-detailed 整文件键 T/T **首次双腿
    同相发射入 stable**（文件级超时吞全部子测发射形
    ——s525-leg2/s526 单腿极端形首次双腿同相化；
    Timeout 形零新 Pass 形，带内摆动记账不升级）。
  - GONE 4 = 族 4 子测键 Home/End/PageDown/PageUp
    （F/F→被整文件超时吞发射；带内形变非语义翻红翻
    绿）——族键摆动谱 s530 同相回归形（detailed T +
    4 子测 F + 整文件不发射）→ s531 双整文件 T 形
    （detailed T/T + non-detailed T/T + 零子测发射）
    ，摆动带内形谱续记。
  - GONE 1 = mpreload 哨 unsafe-url 键（出位，见哨组
    节）。
  - **非族非哨键零 NEW 零 GONE 零 flips——「申报=
    实测」谱系第二十八例（双刀窗）零语义位移清偿**
    （主刀 part01.js 键入替换臂与 html-semantics 键形
    零交点——键入臂域 html/semantics 键形实测无扰；
    fmt 伴刀零语义同证）。
- **P 集账本口径 l1 8306 / l2 8307**（s512 起 51 轮双
  8307P 持位终止——差 1 全归 mpreload 哨键 leg1 瞬态
  Fail；非族键 Pass 形逐键恒等零变）。
- **focus-dynamic 121 键 stable 全 Pass**（type-change
  -on-blur 80 + type-change 40 + time-focus-dynamic-
  value-change 1）：收窗后第 13 轮零再现。
- 双发发射伪影 2 键照旧在册（exact-len 双腿各 2 dup +
  WINdow 双腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨单腿瞬态翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 20 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 60 轮零翻红（双键双腿 Pass）。
- **modulepreload 哨：零翻红连轮第 62 轮终止**——
  unsafe-url 键 leg1 单腿瞬态 Fail（F/P；其余 14 键
  双腿全 Pass）。同二进制 leg2 Pass 定谳**环境瞬态非
  刀致**（主刀键入臂域与 module referrer 网络行为零
  交互路径），时钟重置，后续再现按新 <4 轮窗升级判定
  。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 54 轮
  保持）/ checkmark Fail（带形不变——d46 刀未翻动伪
  元素动画哨形）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（刀码同源旁证，清单 set 恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1671P/0F rc=0**（test 阶段 22.80s；
filtered 2405 持位；Compiling ×3 test profile 补
engine 链）。测试清单 vs s530（b530.k）：**1671 set
恒等，NEW 0 + GONE 0**——主刀 part01.js 为 shim 刀体
非测试面零入桥清单（s529 part125/s530 part03 钉面入
册形不同构），绿窗零破；shim 行为变更直过全桥 1671
断言 = **键入替换臂零伤既有 DOM 桥语义旁证**。

## 对端观察

- 起腿前窗净判定：21:47 起腿预检撞对端重腿 793% CPU
  （远 clone /lzcapp/document/work/ZeroWeb test-guard
  腿背靠背两连发）→ 按 s523/s527/s530 先例**等净窗
  21:54**（load 1.77 回落态，对端 corpus/rustc/cargo
  /zero-wpt-runner 零重腿）+ rally query 守护进程（CI
  轮询，非重负载）；窗内双腿背靠背零外部交叠零截断
  。

## 结论

s531 收口（他流刀归因轮，双刀窗：d46 主刀 + fmt 伴刀
）：码态换新 2c8742499（runner 换代 +8,864，leg1
Compiling ×6，leg2 零重编），html-semantics 面**零语
义位移清偿（申报=实测谱系第二十八例）**——vs s530
stable NEW 1 + GONE 5 + flips 0 位移全落族键摆动带
（带内形谱续记，零新 Pass 形）+ mpreload 哨单腿瞬态
，非族非哨键形逐键恒等；stable 8510（8304P/198F/7T/
1U）全键双腿零分歧（disagree 1 = 哨键瞬态）；P 集
l1 8306 / l2 8307（51 轮双 8307P 持位终止，差 1 归哨
键瞬态）；flaky 键零新例第 20 轮，picker 增益键第 54
轮保持，onblur/dialogov/waapi 三哨持位，mpreload 哨
翻红时钟重置；全桥 1671 set 恒等绿窗零破；
focus-dynamic 收窗后第 13 轮零再现。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。

下一步：守成巡查（对账基线换 s531 stable 8510——族
键基线形换 s531 双整文件 T 形，摆动观察维持：出位/
回归/混合形按形谱记账、Pass 形首现按 NEW 记账；全桥
清单基线 b531.k 1671 持位；flaky 键零新例时钟顺延；
**mpreload 哨翻红后新 <4 轮窗观察**——再现即升级判
定，安全收敛则时钟顺延；他流入树归因轮或轻覆盖；对
端刀信号持续：editing 流 B4-S2 残面刀（dwuf 八变体
端点规范化未接臂/during-and-after-dispatch 与克隆门
互斥 1F——d46 TypingA 替换臂已落残面收口）+ rendering
dialog 16 红块协同面 + css 文法刀后续刀（shapepos 残
面队列：位置串化规范刀 ~498F/round 列表完整建模/
inline inherit/path() 命令级插值/calc 代数插值）+
wptfyi-css 残面（jsanim flip Steps(Start) 等）+ ZW-3
querySelector 施工刀，落树后按他流刀归因轮，join 实
测为准）+ 候门决项等待用户拍板（video P1+P2 立项 /
簇 D A2 立项 / s137）；rendering 流 dialog 16 红块协
同面（归 rendering 流记账）。门开随批落预案不变：簇
D 开 → customizable-select-styles.css ×47 随批落。

工件：/tmp/s531/{s531-leg1,s531-leg2}.log、
s531-leg{1,2}.rc、leg{1,2}.s531.{start,end}、
s531-{l1,l2}.norm、l531-{1,2}.canon、s531.new（1）、
s531.gone（5）、s531.flips.txt（零行）、
s531.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s530.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 l531-{1,2}.canon）、s531-bridge
.log、s531-bridge.rc、b530.k（复用）+ b531.k（清单
set 恒等对账）+ b531.{new(0),gone(0)}、run-legs.sh、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
