# M2 s528 — 他流刀归因轮（单刀）：rendering 流 shapepos shape-outside/circle-ellipse 位置文法刀 2f4fb299a（css-parser parse_extended_border.rs +194/−x + clip_path 钉测 +194 + engine animation.rs +4 + js_dom_bridge/computed_style.rs +35 + part03 钉 +29 + style-system apply.rs +54，s527 收口 push 时 pull 携入，对端申报 css-shapes corpus 净 +1223 零新回归）落树归因——html-semantics 面**零语义位移清偿，「申报=实测」谱系第二十五例（单刀，首个触碰 engine Rust 面 computed_style/animation 的对端刀）**；码态换新 2f4fb299a（runner 换代 127530232 → 127535432，+5,200 刀载荷，leg1 Compiling ×4 补 engine 链——首轮中断窗 css-parser/style-system 已编缓存，leg2 零重编）——vs s527 stable **NEW 4 + GONE 0 + flips 1 非族逐键恒等**：NEW 4 = 族 select-home-end-pagedown-pageup 4 子测键双腿同相回归入 stable（**完全复位 s519/s522/s524 形**）+ flips 1 = 族 detailed 整文件键 Fail→Timeout 回位（s527 首次双腿 F 后回 T/T 持位形）——stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0 全键双腿零分歧形回归，**stable 8514 与 s524 stable 逐键恒等（diff 全等验证）**，族键摆动谱 s527 异相后第 1 轮同相回归（带内回归不升级），P 集双腿双 8307P（持位第 50 轮）+ flaky 零新例第 17 轮 + 四观察哨零新翻红 + 全桥绿窗 1669P/0F 清单 set 恒等（part03 钉未入 js_dom_bridge 口径实测）+ focus-dynamic 收窗后第 10 轮零再现

日期：2026-11-16。承前：s527 收口 push 前置 pull
--rebase 携入对端 rendering/wptfyi-css 流刀
**2f4fb299a**（feat(engine,css-parser): shape-outside/
circle-ellipse 位置文法刀 shapepos 二十二轮；committed
20:42:10）→ 按他流刀归因轮（单刀）。对端刀申报：
css-parser parse_position_pair 严格文法 + engine
clip_path_to_css 缺省分量串化 + style-system
shape-outside 写面门（白名单 19→20）+ round 参量文法
四轮迭代——css-shapes corpus 净 +1223 零新回归；
css-parser 50/50 钉测 + style-system 1/1 + engine 2/2
+ MKT-RC=0（101 全对号在册 peer 红，刀域零新增）。
刀体 CSS shapes 解析/串化/写面域——html-semantics
DOM 语义断言面此前零实测 → 本轮归因补测定谳。

## 二进制 provenance

- runner 换代 **127530232 → 127535432**（+5,200 刀载
  荷；leg1 真重建 `Compiling` ×4（engine/page-runtime/
  webview/wpt-runner 链——首轮 20:48 中断起腿窗已编
  css-parser/style-system 入缓存，本轮补 engine 链），
  20:54:53 重起腿；leg2 同二进制零重编）。
- 码态 = **2f4fb299a**（HEAD 15e207325 = 本流 s527 收
  口 docs-only rebase 落基）；crates 面 diff
  8e9208695..2f4fb299a = 恰该刀 6 文件（css-parser
  parse_extended_border.rs +194 + clip_path tests
  +194 + engine animation.rs +4 + computed_style.rs
  +35 + part03 钉 +29 + style-system apply.rs +54）=
  唯一码变——**首个触碰 engine Rust 面
  （computed_style/animation）的对端刀**（此前
  part01/part06.js shim 刀均零 Rust 变更）。

## 起腿窗事件（对端燃点轮撞窗，kill 后窗净重起）

- 首次起腿 20:48:29 撞对端 rendering 流 R5732 燃点轮
  **corpus A/B ×3**（setsid 背靠背 `make
  reftest-upstream` 三连，785% CPU，load 峰 16.26）
  + editing 流 d46 selection 腿（112% CPU）窗内交叠
  ——按 s523 外部大负载先例 **kill 本流双腿等待退出**
  （本刀为首个 engine Rust 面刀，join 形谱归因基线须
  干净）。
- 对端 corpus ×3 于 20:54 前全部退出（all.done），
  20:54:45 复核 load 1.75 回落、reftest-upstream 零、
  仅余 d46 selection 轻腿（112%，s521/s522 先例同状
  态安全交叠）→ **窗净判定重起**（20:54:53）。
- 窗内双腿时长正常零截断，族键形为已知形谱带内回
  归无新形。

## 双腿 join（HEAD 15e207325 树 = 码态 2f4fb299a，
make testharness-html-semantics ×2）

- leg1 20:54:53–20:58:04 / leg2 20:58:04–21:00:36（零
  lull 背靠背），raw 8517/8517 → norm 8516/8516（双
  腿各丢 1 行 bare 发射 dup：exact-len + WINdow 伪影
  照旧）——双腿零截断零真 hang（rc=2 惯例失败退
  出，时长正常）。本轮管线全 LC_ALL=C 固化。
- **P 集账本口径双腿双 8307P**（持位第 50 轮）。
- join：**stable 8514（8305P/202F/6T/1U）+ disagree 0
  + l1only 0 + l2only 0——全键双腿零分歧形回归**。
- **vs s527 stable（8510 = 8305P/199F/5T/1U）：NEW 4
  + GONE 0 + flips 1——非族逐键恒等**：
  - NEW 4 = 族 select-home-end-pagedown-pageup 4 子
    测键（Home/End/PageDown/PageUp）**双腿同相回归
    入 stable（F/F）——完全复位 s519/s522/s524 形**
    （non-detailed 整文件键双腿均不发射照旧）。
  - flips 1 = 族 **detailed 整文件键 Fail→Timeout
    回位**（s527 首次双腿 F 后回 T/T 持位形；该键
    从未 Pass，Fail/Timeout 带内形谱记账不升级）。
    族键摆动谱 s519 T+4F→…→s524 T+4F 同相→s525
    leg1 s519 形/leg2 双 T 形→s526 leg1 s519 形/leg2
    双 T 零子测形→s527 双腿形镜像互换（detailed 槽
    首次双腿 F）→**s528 同相回归复位 s519/s522/s524
    形**；同码态跨轮异相/同相交替为摆动特征非确定
    性刀效应。
  - **stable 8514 与 s524 stable 逐键恒等（diff 全等
    验证 IDENTICAL）**。
  - **刀体（CSS shapes 位置文法/串化/写面门）与
    customizable-select 键盘导航测试零交点**；非族
    键零 NEW 零 flips——**「申报=实测」谱系第二十
    五例（单刀）零语义位移清偿**。
- **focus-dynamic 121 键 stable 全 Pass：收窗后第 10
  轮零再现**（后续再现按带内间歇记账，无升级判定）。
- 双发发射伪影 2 键照旧在册（exact-len 双腿各 2 dup +
  WINdow 双腿各 2 dup）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 17 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 57 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 60 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 51 轮
  保持）/ checkmark Fail（带形不变——shapepos 刀未
  翻动伪元素动画哨形）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（刀码同源旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 19.32s；
filtered 2405 持位）。测试清单 vs s527（b527.k）：
**set 恒等 1669，NEW 0 / GONE 0**——part03 钉 +29 未
入 js_dom_bridge 口径（实测为准；钉在 enginelib 全量
面由对端申报 engine 2/2 + css-parser 50/50 +
style-system 1/1 覆盖）。

## 结论

s528 收口（他流刀归因轮，单刀 2f4fb299a）：码态换新
2f4fb299a（runner 换代 +5,200，leg1 Compiling ×4，leg2
零重编），html-semantics 面**零语义位移清偿（申报=实
测谱系第二十五例，首个 engine Rust 面刀）**——vs
s527 stable NEW 4（族 4 子测键同相回归复位 s519/s522/
s524 形）+ GONE 0 + flips 1（detailed 键 Fail→Timeout
回位）非族逐键恒等；stable 8514 与 s524 stable 逐键
恒等；join 全键双腿零分歧形回归；P 集双腿双 8307P
持位第 50 轮；flaky 键零新例第 17 轮，picker 增益键
第 51 轮保持，四观察哨零新翻红；全桥 1669 钉面 set
恒等零破；focus-dynamic 收窗后第 10 轮零再现。对端
corpus ×3 燃点轮撞窗 kill 等待退出后窗净重起（形谱
归因基线干净）。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通
知）。

下一步：守成巡查（对账基线换 s528 stable 8514——族
键摆动观察维持：出位/回归/混合形按形谱记账、Pass 形
首现按 NEW 记账；flaky 键零新例时钟顺延——再现方触
发新 <4 轮升级判定；他流入树归因轮或轻覆盖；对端刀
信号持续：editing 流 B4-S2 残面刀（dwuf 八变体端点
规范化未接臂/during-and-after-dispatch 与克隆门互斥
1F/残 9F 面余族刀）+ rendering dialog 16 红块协同面 +
css 文法刀后续刀（shapepos 残面队列：位置串化规范刀
~498F/round 列表完整建模/inline inherit/path() 命令
级插值/calc 代数插值）+ wptfyi-css 残面（jsanim flip
Steps(Start) 等）+ ZW-3 querySelector 施工刀，落树后
按他流刀归因轮，join 实测为准）+ 候门决项等待用户拍
板（video P1+P2 立项 / 簇 D A2 立项 / s137）；
rendering 流 dialog 16 红块协同面（归 rendering 流记
账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s528/{s528-leg1,s528-leg2}.log、
s528-leg{1,2}.rc、leg{1,2}.s528.{start,end}、
s528-{l1,l2}.norm、l528-{1,2}.canon、s528.new（4 族
子测键）、s528.gone（零行）、s528.flips.txt（1 族
detailed 键 F→T 回位）、s528.{l1u,l2u,s1,s2,k1,k2,
both.keys,leg1only.keys,leg2only.keys,disagree.keys,
stable.keys,stable.canon}、s527.stable.{keys,canon} +
s524.stable.canon（逐键恒等核对复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 s1/s2）、s528-bridge.log、
s528-bridge.rc、b527.k（复用）+ b528.k（清单恒等对
账）+ b528.{new,gone}（双零行）、run-legs.sh、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
