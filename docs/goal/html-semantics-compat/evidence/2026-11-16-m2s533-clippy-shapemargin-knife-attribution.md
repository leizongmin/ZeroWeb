# M2 s533 — 他流刀归因轮（单刀·零语义伴修）：对端 clippy 自修复刀 7e2c25ff9（fix(engine): shape_margin_to_css 死绑定谓词改 is_some——CI clippy unused variable 修复，crates/engine/js_dom_bridge/computed_style.rs 单文件 +2/−2 构造性零语义改写：`if let Some(n) = <链>` n 未用绑定 → `if <链>.is_some()`，迭代链/过滤谓词/控制流/返回值逐 token 恒等）落树归因——html-semantics 面**实测零位移清偿，「申报=实测」谱系第二十九例（申报 = 构造性零语义主张，实测 = 双腿零位移确证）**——码态换新 7e2c25ff9（runner 换代 127567632 → 127567616，−16 字节缩量与零语义改写方向一致，leg1 真重建 Compiling ×6 engine 链，leg2 零重编）——vs s532 stable **NEW 1 + GONE 0 + flips 0 位移全落带内**：NEW 1 = 族 non-detailed 整文件键 T/T 同相回归入 stable（s531 双整文件 T 形回归，带内）+ disagree 1 = 族 detailed 键 F/T 异相（leg1 = detailed F + 整文件 T + 零子测发射恰 s527-leg1/s520-leg2 形；leg2 = detailed T + 双整文件 T + 零子测发射恰 s531 形——异相混合形带内摆动谱续记）——非族非哨键零 NEW 零 GONE 零 flips 逐键恒等；join stable 8510（8305P/198F/6T/1U）disagree 1 + l1only 0 + l2only 0；P 集双腿双 8307P（s532 回归后第 2 轮持位）+ flaky 零新例第 22 轮 + onblur/dialogov/waapi 三哨持位 + **mpreload 哨 15 键双腿全 Pass——unsafe-url 键 <4 轮窗 2/4 安全通过** + 全桥 1671P/0F rc=0 清单 set 恒等 + focus-dynamic 收窗后第 15 轮零再现

日期：2026-11-16。承前：s532 收口 push 前置 pull 携入对
端 **7e2c25ff9**（fix(engine): shape_margin_to_css 死
绑定谓词改 is_some——对端 CI clippy 红 self-fix，
2116a2a53 GB 巡检落账确认「11-16 clippy 红属流当日
自修复证」）+ 两笔 docs-only（b25985db4 rendering-
compat R5736 + 2116a2a53 zero-web 巡检落账）→ 按
他流刀归因轮（零语义伴修单刀）。

## 二进制 provenance

- 主刀面 = `git show 7e2c25ff9` = 恰 **1 文件 +2/−2**
  （crates/engine/src/js_dom_bridge/computed_style.rs
  shape_margin_to_css Calc 臂谓词改写）：`if let
  Some(n) = s.strip_suffix("px").and_then(parse).filter
  (finite).filter(<0)` n 未用 → 同链 `.is_some()`——
  **构造性零语义**（迭代链逐 token 恒等、绑定移除、
  控制流与 `"0px"` 早返回不变）。
- runner 换代 **127567632 → 127567616**（**−16 字节
  **——缩量方向与死绑定移除一致；leg1 22:34:03 起腿
  真重建 `Compiling` ×6（product-version/net/engine/
  page-runtime/webview/wpt-runner），leg2 同二进制零
  重编）。
- 码态 = **7e2c25ff9**（HEAD e07ae06d3 = 本流 s532
  收口 docs-only 落基，vs 2c8742499 零码面差恰该刀）。

## 双腿 join（HEAD e07ae06d3 树 = 码态 7e2c25ff9，
make testharness-html-semantics ×2）

- leg1 22:34:03–22:37:22 / leg2 22:37:22–22:40:10（零
  lull 背靠背），raw/norm 8513/8513（双腿各 +2 行双发
  发射 dup：exact-len + WINdow 伪影照旧）——双腿零截
  断零真 hang（rc=2 惯例失败退出，时长正常）。管线全
  LC_ALL=C 固化。
- join：**stable 8510（8305P/198F/6T/1U）+ disagree 1
  + l1only 0 + l2only 0**。
- **vs s532 stable（8509）：NEW 1 + GONE 0 + flips 0
  ——位移全落带内**：
  - NEW 1 = 族 non-detailed 整文件键 T/T 同相回归入
    stable（s531 双整文件 T 形腿位回归；Timeout 形零
    新 Pass 形，带内记账不升级）。
  - disagree 1 = 族 detailed 键 F/T 异相（leg1 =
    detailed **F** + non-detailed 整文件 T + 零子测
    发射恰 s527-leg1/s520-leg2 形；leg2 = detailed
    **T** + 双整文件 T + 零子测发射恰 s531 形）——异
    相混合形，带内摆动谱续记（零单腿出位零 flips）。
  - **非族非哨键零 NEW 零 GONE 零 flips——实测零位
    移清偿（申报=实测谱系第二十九例：申报 = 构造性
    零语义主张，实测 = 双腿 join 零位移确证；刀体
    shape_margin Calc 谓词与 html-semantics 键形零交
    点构造同证）**。
- **P 集账本口径双腿双 8307P**（s532 回归后第 2 轮持
  位）。
- **focus-dynamic 121 键 stable 全 Pass**：收窗后第
  15 轮零再现。
- 双发发射伪影 2 键照旧在册（exact-len + WINdow 双腿
  各 2 dup）。

## 观察哨复核（三哨持位 + mpreload 哨窗续过）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键（checkbox following-label）
  双腿双 Pass：零新例第 22 轮（s512 时钟顺延；后续再
  现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 62 轮零翻红（双键双腿 Pass）。
- **modulepreload 哨：15 键双腿全 Pass——unsafe-url
  键 <4 轮窗 2/4 安全通过**（s531 翻红后窗内零再现，
  过 3/4 即时钟重顺延零翻红起算）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 56 轮
  保持）/ checkmark Fail（带形不变——clippy 伴修刀
  未翻动伪元素动画哨形）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1671P/0F rc=0**（test 阶段 21.72s；
filtered 2405 持位；Compiling ×3 test profile 补
engine 链）。测试清单 vs s532（b532.k）：**1671 set
恒等，NEW 0 + GONE 0**——绿窗零破；刀体所在
shape_margin_to_css 直属 js_dom_bridge 模块，全桥
1671 断言含 shape_margin 三站钉全绿 = **构造性零语
义直证**。

## 对端观察

- 起腿前窗净判定（22:34 load 1.04 干净态，对端
  corpus/rustc/cargo/zero-wpt-runner 零重腿，仅余
  rally query 守护 CI 轮询）——零等窗直接起腿；窗内
  双腿背靠背零外部交叠零截断。

## 结论

s533 收口（他流刀归因轮，零语义伴修单刀 7e2c25ff9）
：码态换新 7e2c25ff9（runner 换代 −16 字节，leg1
Compiling ×6，leg2 零重编），html-semantics 面**实测
零位移清偿（申报=实测谱系第二十九例）**——vs s532
stable NEW 1 + GONE 0 + flips 0 位移全落带内（族键
整文件键 T/T 回归 + detailed 键 F/T 异相混合形摆动
带内），非族非哨键形逐键恒等；P 集双腿双 8307P 持位
第 2 轮；flaky 零新例第 22 轮，picker 增益键第 56 轮
保持，onblur/dialogov/waapi 三哨持位，mpreload 哨
<4 轮窗 2/4 安全通过；全桥 1671 set 恒等绿窗零破；
focus-dynamic 收窗后第 15 轮零再现。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。

下一步：守成巡查（对账基线换 s533 stable 8510——族
键摆动观察维持：出位/回归/混合形按形谱记账、Pass 形
首现按 NEW 记账；全桥清单基线 b533.k 1671 持位；
flaky 键零新例时钟顺延；**mpreload 哨 <4 轮窗第 3/4
轮观察**——再现即升级判定、过窗则时钟重顺延；他流
入树归因轮或轻覆盖；对端刀信号持续：editing 流 B4-S2
残面刀（dwuf 八变体端点规范化未接臂/during-and-
after-dispatch 与克隆门互斥 1F）+ rendering dialog 16
红块协同面 + css 文法刀后续刀（shapepos 残面队列：
位置串化规范刀 ~498F/round 列表完整建模/inline
inherit/path() 命令级插值/calc 代数插值）+ wptfyi-css
残面（jsanim flip Steps(Start) 等）+ ZW-3
querySelector 施工刀，落树后按他流刀归因轮，join 实
测为准）+ 候门决项等待用户拍板（video P1+P2 立项 /
簇 D A2 立项 / s137）；rendering 流 dialog 16 红块协
同面（归 rendering 流记账）。门开随批落预案不变：簇
D 开 → customizable-select-styles.css ×47 随批落。

工件：/tmp/s533/{s533-leg1,s533-leg2}.log、
s533-leg{1,2}.rc、leg{1,2}.s533.{start,end}、
s533-{l1,l2}.norm、l533-{1,2}.canon、s533.new（1）、
s533.gone（零行）、s533.flips.txt（零行）、
s533.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s532.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用）、s533-bridge.log、s533-bridge.rc、b532.k（复用）
+ b533.k（set 恒等对账）+ b533.{new(0),gone(0)}、
run-legs.sh、norm.sh/canon.sh/join.sh（s488 修复版管
线复用，全 LC_ALL=C）。
