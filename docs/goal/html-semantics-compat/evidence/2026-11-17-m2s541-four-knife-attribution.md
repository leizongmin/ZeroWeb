# M2 s541 — **他流刀归因轮四刀窗**（d48 join 壳深化 + ILB-FLEXBR v1/v1.1 + trans-all/WAAPI hardening）：v3.1 浸泡门判净起腿——join **stable 8510（norm 双腿同相 8307P/202F/6T/1U）disagree 0 + l1only 4 + l2only 1**；vs s540 stable **NEW 0 + GONE 4 + flips 0——位移全落族键摆动带（单腿异相发射/被吞形）**：GONE 4 = 族 non-detailed 4 子测键（l1only 4＝leg1 发射 leg2 未发）+ l2only 1 = 族 non-detailed 整文件键（leg2 发射 leg1 未发）——**非族非哨键零 NEW 零 GONE 零 flips 逐键恒等**；**「申报=实测」谱系第三十二例（四刀窗）**

日期：2026-11-17（round start ~04:43，起腿
04:52:31）。承前：s540 收口后 pull 携入四刀全
数落树（70b18f81e d48 join 壳深化 A+B1+B2 +
5dc363b35 ILB-FLEXBR + b6e2ddc05 FLEXBR v1.1 +
f4ca3f641 trans-all/computed 值域钉/WAAPI
hardening）+ 他流 docs 数笔（R5745/R5746/
R5747 + wptfyi-css trans-all 收账）。

## 窗净判定（对端 r5747 corpus×3 期）

- 对端 r5747 守候器 NET WINDOW 04:43:25 起
  corpus×3（04:43–04:52 快速收尾，rc 对端自账
  ）——v3.1 守候器（deadline 9000s）在其收线
  后判净：04:52:31 clean-soaked procs=0
  load<3 ×3 查 45s，**HEAD=1ec39ee46 钉扎起腿
  **。
- 双腿 04:52:31–04:58:22 背靠背零 lull：leg1
  3:08 / leg2 2:43＝常态，零塌陷零单腿塌陷形
  ，rc=2×2 惯例。

## 二进制 provenance

- 四刀（7356bf2aa→f4ca3f641，legs 时 HEAD
  1ec39ee46 docs-only）：70b18f81e（part01.js
  join 壳 A/B1/B2）+ 5dc363b35/b6e2ddc05
  （part01.js 垫节点三分 + part129 伴钉）+
  f4ca3f641（transition_registry +237、
  computed_style +119、js_anim_registry +58）。
- runner 换代 **127719928→127779560（+59,632
  ，md5 51fbb16b…）**，leg1 Compiling ×6，
  leg2 零重编；legs 前后 size/md5 provenance
  在册。

## 双腿 join（HEAD 1ec39ee46 树，make
testharness-html-semantics ×2）

- raw norm 双腿 l1 8516 / l2 8513（−3 全落族
  形单腿异相：4 子测行 + 整文件行互换）。管线
  全 LC_ALL=C 固化（s488 修复版复用）。
- join：**stable 8510 + disagree 0 + l1only 4
  + l2only 1**（norm 双腿同相 8307P/202F/6T
  /1U）。
- **vs s540 stable（8514）：NEW 0 + GONE 4 +
  flips 0——位移全落族键摆动带（单腿异相形，
  s527→s541 摆动谱新变体：发射/被吞单腿异相
  ）**：
  - GONE 4 = 族 non-detailed 4 子测键
    （Home/End/PageUp/PageDown）＝l1only 4
    （leg1 发射、leg2 未发）。
  - l2only 1 = 族 non-detailed 整文件键
    （leg2 发射、leg1 未发；s540 stable 本无
    此键，故不入 GONE）。
  - 族 detailed 整文件键 T/T 双腿同相持位（
    stable 在册）；listbox 2 键 F 照旧。
  - **非族非哨键零 NEW 零 GONE 零 flips 逐键
    恒等**；零新 Pass 形零新 Fail 形（族键全
    F/T 出位形）。
- **P 集账本口径双腿双 8307P 持位第 10 轮**。
- 双发发射伪影 2 键照旧在册（exact len +
  WINdow 双腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 常规计数第 3 轮）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-
  label 双键红池照常）——flaky 键双腿双
  Pass：零新例第 30 轮（s512 时钟顺延）。
- dialog-overlay 哨第 70 轮零翻红（双键双腿
  Pass）。
- **modulepreload 哨：15 键双腿全 Pass——转
  常规连轮计数第 3 轮零翻红**。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益
  第 64 轮保持）/ checkmark Fail（带形不变）。
  哨复核全程 live 状态 join。
- focus-dynamic 121 键双腿全 Pass：收窗后
  第 23 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行，
guard rc=0）：**1682P/0F**（test result 行 ok，
test 阶段 17.60s）。清单 vs s540（b540.k）：
**NEW 8 + GONE 0**——7× f4ca3f641 伴钉
（computed_style rule_list_computed_tests ×4 +
transition_registry tests ×2 + js_anim_registry
keyframes 校验钉 ×1）+ 1× part129
test_ilb_flexbr_pins（FLEXBR 伴钉）——+8 全数
对号入座四刀窗伴钉，绿窗零破，基线
b540.k→b541.k（1674→1682）。

## 归因：「申报=实测」谱系第三十二例（四刀窗）

| 刀 | 流/申报 | 本面实测 |
|---|---|---|
| d48 join 壳深化 70b18f81e | editing：input-events F2P=223/P2F=0；哨兵 html-semantics P 恒等 5378（1 键 T→F 瞬态）+ selection 49,006P 逐键恒等 | 零位移（non-family 恒等）；对端哨 T→F 瞬态在本轮双腿零复现（disagree 0 + flips 0）判对端环境窗瞬态 |
| ILB-FLEXBR 5dc363b35 + v1.1 b6e2ddc05 | editing：corpus ④族 4 键 F→P + 现役绿 8 键零扰动 + ilb 钉测全家 4P/0F | 零位移；bridge NEW 1 伴钉（part129） |
| trans-all＋值域钉＋WAAPI 校验 f4ca3f641 | wptfyi-css：五 corpus 净 +474 零回归；rendering r5746 batch（d48+flexbr）stable 995 SET-IDENTICAL | 零位移；bridge NEW 7 伴钉 |

四刀窗合计：html-semantics 面零语义位移清偿，
bridge 清单 +8 全对号四刀伴钉——「申报=实测」
配对成立。

## 对端观察

- r5747 corpus×3 04:43–04:52 快速收尾（rc 对
  端自账不入本流账）；本轮 waiter v3.1 在其
  收线后判净起腿零交叠。
- 对端刀信号持续：ZW-3 d47 刀体
  （querySelector generation-aware 重设计刀）
  + editing 流 B4-S2 残面刀 + rendering dialog
  16 红块协同面 + css 文法刀后续刀（shapepos
  残面队列余项）+ wptfyi-css 残面（B3 残面
  715→348F 后余项、jsanim flip Steps(Start)
  等）——本轮均未入树。

## 结论

s541 收口（他流刀归因轮四刀窗）：码态
7356bf2aa→f4ca3f641、runner 127719928→
127779560 换代；join stable 8510（8307P 双腿
同相）disagree 0 + l1only 4 + l2only 1——vs
s540 NEW 0 + GONE 4 + flips 0 位移全落族键摆
动带（单腿异相发射/被吞新变体），非族非哨键
逐键恒等；P 集双腿双 8307P 持位第 10 轮；
flaky 零新例第 30 轮，picker 增益第 64 轮保持
，onblur/dialogov/waapi 三哨持位，mpreload 哨
常规计数第 3 轮零翻红；全桥 1682P/0F 清单
NEW 8（四刀伴钉 7+1）绿窗零破；focus-dynamic
收窗后第 23 轮零再现；「申报=实测」谱系第三
十二例（四刀窗）。挂用户门维持：GB-20261114-
VIDEO + GB-20261031（簇 D）+ s137（s445 已提
示，不重复通知）。

下一步：守成巡查（对账基线换 s541 stable 8510
——族键基线形换「detailed 整文件 T 形持位 +
non-detailed 整文件/4 子测出位观察形（单腿异
相发射，s541 新变体）」：异相/出位按形谱记账
、Pass 形首现按 NEW 记账；全桥清单基线
b541.k 1682 持位；flaky 键零新例时钟顺延；
**mpreload 哨常规连轮计数第 4 轮**；他流入树
归因轮或轻覆盖；**起腿一律 v3.1 浸泡门**；对
端刀信号持续：ZW-3 d47 刀体落树后按他流刀归
因轮 + editing 流 B4-S2 残面刀 + rendering
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

工件：/tmp/s541/{s541-leg1,s541-leg2}.log、
s541-leg{1,2}.rc、leg{1,2}.s541.{start,end}、
legs.head（1ec39ee46）、runner.{pre,mid,post}
.{size,md5}、window.s541.judge（v3.1
clean-soaked）、wait-and-run.sh（v3.1 版
9000s）、run-legs.sh、s541-{l1,l2}.norm、
l541-{1,2}.canon、s541.new.keys（零行）、
s541.gone.keys（GONE 4 族键）、s541.flips.txt
（零行）、s541.{l1u,l2u,s1,s2,k1,k2,both.keys
,leg1only.keys,leg2only.keys,disagree.keys,
stable.keys,stable.canon}、s540.stable.{keys,
canon}（基线复用）+ s477-{onblur,dialogov,
mpreload,waapi}.keys + fd121.keys + b540.k（
复用）+ b541.k（1682）、s541.bridge.{new,gone}
（gone 零行）、s541-bridge.log、s541-bridge.rc
（guard rc=0）+ norm.sh/canon.sh/join.sh（
s488 修复版管线复用，全 LC_ALL=C）。
