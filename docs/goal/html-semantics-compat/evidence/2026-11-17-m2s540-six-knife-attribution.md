# M2 s540 — **他流刀归因轮六刀窗**——上一 session（429 限流中断 03:17）四刀树腿 stale 作废全量覆写重跑（s537 先例第三度复用）：六刀全落树后 v3.1 浸泡门判净起腿——join **stable 8514（norm 双腿同相 8307P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0**；vs s539 stable **NEW 5 + GONE 1 + flips 0——位移全落族键摆动带（s539 形镜像回归＝s538 形）**：NEW 5 = 族 non-detailed 4 子测键（Home/End/PageUp/PageDown）F/F 双腿同相发射 + 族 detailed 整文件键 T/T 双腿同相发射，GONE 1 = 族 non-detailed 整文件键被吞——**s540 stable 与 s538 stable 逐键恒等（keyset comm 零差 + canon diff IDENTICAL 实证）**，零新 Pass 形零新 Fail 形，非族非哨键零位移逐键恒等；**「申报=实测」谱系第三十一例（六刀窗）**

日期：2026-11-17（round start 04:13，起腿
04:27:06）。承前：s539 收口后上一 session 02:04
pull 携入四刀（VISPAD-V2 3dd6eb716 + A壳尾 v1.1
57c7d2cd0 + gap-rules B ac516aaef + css-gaps B
engine 71b07c7d8）后 429 中断，其后台守候器
（session 死亡后存活）03:31:16–03:37:01 在**四
刀树**起腿一轮（stable 8514 disagree 0 形同本
轮）——判 **stale 作废**（缺 WSCANON 双刀
4e091e6f7/7356bf2aa，02:31/02:39 落树），工件
归档 stale-4knife/；本 session 04:13 pull 携入
WSCANON 双刀 + 他流 docs 六笔，六刀全数在树重
跑。

## 窗净判定（对端 r5745 corpus×3 期，waiter v3.1 首用）

- 对端 r5745 corpus×3（04:17–04:26，各 ~3min
  收尾 rc≠0 对端自账）——v3.1 守候器（v3 基础
  上：进程门扩 `reftest-upstream`/他流
  `wait-and-run` 计入 + deadline 3300s→7200s +
  judge 行钉 HEAD + legs 前后 runner size/md5
  provenance）在其收线后判净：04:27:06
  clean-soaked procs=0 load<3 ×3 查 45s，
  **HEAD=46c3df57a 钉扎起腿**。
- 双腿 04:27:06–04:32:37 背靠背零 lull：leg1
  3:04 / leg2 2:27＝常态（s539 常态 3:04/2:30）
  ，零塌陷零单腿出位，rc=2×2 惯例。

## 二进制 provenance

- 六刀（55c78a219→7356bf2aa，HEAD 46c3df57a
  docs-only）：editing ilb VISPAD-V2
  3dd6eb716（part01.js +76 + part127 伴钉）+
  A壳尾 v1.1 57c7d2cd0（part01.js +24）+ css
  gap-rules B ac516aaef（css-parser/style-system
  17 文件 +1449/−239）+ engine css-gaps B
  71b07c7d8（animation.rs +455 +
  css_style_declaration.rs +317 +
  computed_style.rs +278）+ WSCANON-V1
  4e091e6f7（part01.js +69 + part128 伴钉）+
  WSCANON v1.1 7356bf2aa（part01.js +8/−2）。
- runner 换代 **127651344→127719928（+68,584，
  md5 8f966007…）**，leg1 Compiling ×6（含
  css-parser/style-system/engine 全链），leg2
  零重编；legs 前后 size/md5 provenance 在册
  （pre=stale 四刀树二进制 127716096）。

## 双腿 join（HEAD 46c3df57a 树，make
testharness-html-semantics ×2）

- raw norm 双腿各 8516 行同相（s539 是
  8513/8513 族摆吞行态，本轮 4 子测行归位
  +4、non-detailed 整文件行 −1）。管线全
  LC_ALL=C 固化（s488 修复版复用）。
- join：**stable 8514 + disagree 0 + l1only 0 +
  l2only 0**（norm 双腿同相 8307P/202F/6T/1U）
  。
- **vs s539 stable（8510）：NEW 5 + GONE 1 +
  flips 0——位移全落族键摆动带，s539 形镜像
  回归（＝s538 形，s527→s538→s539→s540 摆动
  谱：A 形→B 形→A 形）**：
  - NEW 5 = 族 non-detailed 4 子测键
    （Home/End/PageUp/PageDown）**F/F 双腿
    同相发射** + 族 detailed 整文件键 **T/T
    双腿同相发射**（s539 的 detailed T/F 异相
    键本轮收敛同相）。
  - GONE 1 = 族 non-detailed 整文件键（s539
    T/T 形）被吞。
  - **s540 stable 与 s538 stable 逐键恒等**
    （keyset comm 零差 + canon diff IDENTICAL
    双实证）；零新 Pass 形零新 Fail 形（NEW
    键 F×4 + T×1）；非族非哨键零 NEW 零
    GONE 零 flips 逐键恒等。
- **P 集账本口径双腿双 8307P 持位第 9 轮**。
- 双发发射伪影 2 键照旧在册（exact len +
  WINdow 双腿各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 常规计数第 2 轮）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-
  label 双键红池照常）——flaky 键双腿双
  Pass：零新例第 29 轮（s512 时钟顺延）。
- dialog-overlay 哨第 69 轮零翻红（双键双腿
  Pass）。
- **modulepreload 哨：15 键双腿全 Pass——转
  常规连轮计数第 2 轮零翻红**。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益
  第 63 轮保持）/ checkmark Fail（带形不变）。
  哨复核全程 live 状态 join。
- focus-dynamic 121 键双腿全 Pass：收窗后
  第 22 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test -p
zero-engine --lib js_dom_bridge`（仓根执行）：
**1674P/0F**（test result 行 ok，test 阶段
35.20s；filtered 2405→2425 +20 非桥 src 侧新
测）。清单 vs s539（b539.k）：**NEW 2 =
part127 test_ilb_shell_tail_vispad_pins
（VISPAD-V2 伴钉）+ part128
test_ilb_wscanon_pins（WSCANON 伴钉）+ GONE
0**——绿窗零破，基线 b539.k→b540.k（1672→
1674，+2 全归 ilb 刀伴钉，与 s537 part126
先例同形）。

## 归因：「申报=实测」谱系第三十一例（六刀窗）

| 刀 | 流/申报 | 本面实测 |
|---|---|---|
| ilb VISPAD-V2 3dd6eb716 + A壳尾 v1.1 57c7d2cd0 | editing v1.125（8e66348e7）：i7d0 v2 ALL-DONE ilb 双腿 1669P/22F F2P-EXACT-5 零新增红 + enginelib 4078P/0F | 零位移（stable set 恒等域内）；bridge NEW 1 伴钉 |
| gap-rules B ac516aaef + css-gaps B engine 71b07c7d8 | wptfyi-css（c6f75ac9b）corpus 净 +265/0 lost 五页集合全等；rendering R5744（aa869032b）stable 996→995 GONE 1＝multicol-rule-percent-001 F→P | 零位移（column-rule/gap 域不入语义面）；bridge NEW 0 |
| WSCANON-V1 4e091e6f7 + v1.1 7356bf2aa | editing 申报 part128 钉测定向 1P/0F + ilb 面 ws 邻接形 7 键（Fail 5/Pass 2）判据对 2 现役绿键零扰动 | 零位移；bridge NEW 1 伴钉 |

六刀窗合计：html-semantics 面零语义位移清偿，
bridge 清单 +2 全归 ilb 刀伴钉——「申报=实测」
配对成立。

## 对端观察

- r5745 corpus×3 04:17–04:26 快速收尾 rc≠0（
  对端自账，不入本流账）；04:37+ ZW-3
  zeroseed decision_integration 测试腿再起——
  对端活跃复燃，**下轮起腿继续 v3.1 浸泡门**
  （进程门已扩他流 waiter/reftest-upstream 计
  入，deadline 7200s 防 corpus×3 长窗超时）。
- 对端刀信号持续：ZW-3 d47/d48 刀体
  （querySelector generation-aware 重设计刀）
  + editing 流 B4-S2 残面刀 + rendering dialog
  16 红块协同面 + css 文法刀后续刀（shapepos
  残面队列余项）+ wptfyi-css 残面——本轮未入
  树。

## 结论

s540 收口（他流刀归因轮六刀窗，上一 session
429 中断 stale 腿作废重跑）：码态
55c78a219→7356bf2aa、runner 127651344→
127719928 换代；join stable 8514（8307P 双腿
同相）disagree 0 + l1only 0 + l2only 0——vs
s539 NEW 5 + GONE 1 + flips 0 位移全落族键摆
动带，**s540 stable 与 s538 stable 逐键恒等
（canon IDENTICAL）**；P 集双腿双 8307P 持位
第 9 轮；flaky 零新例第 29 轮，picker 增益第
63 轮保持，onblur/dialogov/waapi 三哨持位，
mpreload 哨常规计数第 2 轮零翻红；全桥 1674P
/0F 清单 NEW 2（ilb 刀伴钉 ×2）绿窗零破；
focus-dynamic 收窗后第 22 轮零再现；「申报=
实测」谱系第三十一例（六刀窗）。waiter v3.1
首用零穿透。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重
复通知）。

下一步：守成巡查（对账基线换 s540 stable 8514
——族键基线形＝A 形回归「non-detailed 4 子测
F 形 + detailed 整文件 T 形」（s538 同型）：
s527→s540 摆动谱 A/B 两形交替在案，异相/出位
按形谱记账、Pass 形首现按 NEW 记账；全桥清单
基线 b540.k 1674 持位；flaky 键零新例时钟顺延
；**mpreload 哨常规连轮计数第 3 轮**；他流入树
归因轮或轻覆盖；**对端活跃复燃期起腿一律 v3.1
浸泡门**；对端刀信号持续：ZW-3 d47/d48 刀体落
树后按他流刀归因轮 + editing 流 B4-S2 残面刀
+ rendering dialog 16 红块协同面 + css 文法刀
后续刀（shapepos 残面队列余项：位置串化规范刀
~498F/round 列表完整建模/inline inherit/
path() 命令级插值/calc 代数插值）+ wptfyi-css
残面（jsanim flip Steps(Start) 等），join 实测
为准）+ 候门决项等待用户拍板（video P1+P2 立
项 / 簇 D A2 立项 / s137）；rendering 流
dialog 16 红块协同面（归 rendering 流记账）。
门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s540/{s540-leg1,s540-leg2}.log、
s540-leg{1,2}.rc、leg{1,2}.s540.{start,end}、
legs.head（46c3df57a）、runner.{pre,mid,post}
.{size,md5}、window.s540.judge（v3.1
clean-soaked）、wait-and-run.sh（v3.1 版）、
run-legs.sh、s540-{l1,l2}.norm、l540-{1,2}
.canon、s540.new.keys（NEW 5 族键）、
s540.gone.keys（GONE 1 族键）、s540.flips.txt
（零行）、s540.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,
stable.keys,stable.canon}、s538/s539.stable.
{keys,canon}（基线复用，s540≡s538 双实证）+
s477-{onblur,dialogov,mpreload,waapi}.keys +
fd121.keys + b539.k（复用）+ b540.k（1674）、
s540.bridge.{new,gone}（gone 零行）、
s540-bridge.log + stale-4knife/（上一 session
四刀树腿工件 + join 归档）+ norm.sh/canon.sh/
join.sh（s488 修复版管线复用，全 LC_ALL=C）。
