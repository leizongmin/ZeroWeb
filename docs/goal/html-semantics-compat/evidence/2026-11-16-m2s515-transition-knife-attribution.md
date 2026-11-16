# M2 s515 — 他流刀归因轮（单刀）：wptfyi-css 流 transition 通道 gCS 同步读可见性刀 3fbaaf26d 落树归因（animation.rs +152 + transition_registry.rs 新文件 +476 含 8 单测 + callbacks.rs +36 + computed_style_cache.rs +20 + js_dom_bridge.rs +5 + js_anim_registry.rs pub 提级 ±1，6 文件 +690/-1）——html-semantics 面**零语义位移清偿，「申报=实测」谱系第十八例**；码态换新 3fbaaf26d（runner 换代 127490632 → 127514672 +24,040，换代发生于前会话 orphan leg1 11:29:42，fresh 双腿同二进制零重编 Compiling 0）——vs s514 stable NEW 14（focus-dynamic 12 键双腿双全发射回位预期兑现 + 族 optional 子测 End/Home 2 键首入 stable）+ GONE 0 + flips 0 非族逐键恒等，stable 8511，P 集双腿双 8307P（持位第 37 轮）+ 四观察哨零新翻红 + 全桥绿窗 1669P/0F（NEW 8 刀体自带单测）既有 1661 set 恒等

日期：2026-11-16。承前：s514 下一步「守成巡查（对账基
线换 s514 stable 8497……他流入树归因轮或轻覆盖）」。
s514 双腿树（HEAD 46f6f530a = 码态 8ed0e22bd）未及见
刀：s514 收口 push 前 pull --rebase 携入 wptfyi-css 流
**3fbaaf26d**（fix(engine): transition 通道 gCS 同步读
可见性刀——LAST_STYLE 前值捕获 + World::{Gen,View} 两
世界注册表 + entry_eased 本地求值 Steps 臂 StepPosition
修 + eased==1.0 echo 回落 + filter 量钳制两臂；committed
11:24:51）+ 其收口 docs 99d1019ca + rendering R5718
da3c22bd5 → HEAD 60aca76b7 树含双刀，而 `git diff
8ed0e22bd..HEAD -- crates/` 恰 6 文件 +690/-1 = 该刀码
为本轮唯一码变（8ed0e22bd 已由 s514 归因清偿）→ 按他
流刀归因轮（单刀）。对端刀申报：registry 单测 8/8；
engine lib 4,068P/0F；webview lib 852P/2F 逐名对号在
册环境形红零新增；A/B 七套件 corpus gained=322 lost=0
regressed=0（filter-effects 八页 +17~+40、motion 七页
+4~+25）——刀体为 CSS transition gCS 同步读通道，html
-semantics DOM 语义断言面此前零实测 → 本轮归因补测。

## 对端观察

- fresh 双腿窗（13:18:00–13:23:23）对端零构建级背景
  （起腿 load 0.88；腿尾 9.51 为本腿 harness 自身；收
  口 load 7.52 回落中；cargo/rustc 全程零在册）。
- 前会话 orphan leg1 窗（11:28:33–11:32:55）= 对端
  R5718「与本流 corpus 时窗全重叠 contention 疑点」期
  （da3c22bd5 双证并载）——本轮 fresh 双腿给出**独立
  第三证**（见下 orphan 旁证段）。

## 二进制 provenance

- runner 换代 **127490632 → 127514672**（+24,040 刀载
  荷）；换代发生于 orphan leg1 起腿后（11:29:42 落
  盘）——fresh 双腿（13:18 起）同二进制零重编
  `Compiling 0`；桥跑 `Compiling ×3` 桥侧 churn。
- 码态 = 3fbaaf26d 之后 HEAD 60aca76b7（docs-only 尾
  = 本流 s514 收口）；crates 面 diff 8ed0e22bd..HEAD
  逐文件对号 = 刀码唯一码变。

## 双腿 join（HEAD 60aca76b7 树 = 码态 8ed0e22bd +
3fbaaf26d，make testharness-html-semantics ×2）

- leg1 13:18:00–13:20:40 / leg2 13:20:47–13:23:23，
  raw 8516/8517 → norm 8515/8516（双腿各丢 1 行 bare
  发射 link 面同形）——双腿零截断零真 hang（rc=2 惯
  例失败退出）。本轮管线全 LC_ALL=C 固化。
- **P 集账本口径双腿双 8307P**（持位第 37 轮；leg2
  自 s514 瞬态 8295 回位——s507/s514 瞬态形谱第 3 例
  均单腿单轮回位）。
- join：**stable 8511（8305P/200F/5T/1U）+ disagree 1
  + l1only 1 + l2only 2**——全族键带内：
  - disagree 1 = 族 detailed 整文件键 leg1 Timeout/
    leg2 Fail（异相摆动持续——s513 F/T → s514 T/F →
    本轮 T/F）。
  - l1only 1 = 族 base optional 整文件键 leg1
    Timeout（leg2 零整文件发射）。
  - l2only 2 = 族 base optional 子测 PageDown/
    PageUp 键 leg2 Fail（leg1 四子测全发射 End/Home/
    PgDn/PgUp、leg2 仅 End/Home——**整文件/子测形互
    换两形异相持续**，s508/s513 同款谱内再现）。
- **vs s514 stable（8497 = 8293P/198F/5T/1U）：NEW 14
  + GONE 0 + flips 0——非族逐键恒等**：
  - **NEW 12 = focus-dynamic 12 子测键双腿双全发射回
    位**（s514 leg2 文件级 Timeout 截断形预期兑现
    「s515 双腿双全发射回位则 NEW 12 回位不升级」；
    整文件键双腿 Pass 同入 stable——focus-dynamic
    121 键 stable 全 Pass。**截断形观察窗 s515–s518
    第 1 轮零再现**，带内记账继续）。
  - NEW 2 = 族 base optional 子测 End/Home 键双腿双
    Fail 首入 stable（s514 为 l1only 形，本轮双腿双
    发射）。
  - 刀体（CSS transition gCS 同步读通道：LAST_STYLE
    前值捕获 + 两世界注册表 + entry_eased + 量钳制）
    与 html-semantics DOM 语义断言面零交点——对端申
    报 corpus +322 零回归、engine lib 4068P = s514
    时代 4060P + 8 新单测算术自洽，互证**「申报=实
    测」谱系第十八例（单刀）零语义位移清偿**。
- T 集 5 恒 + U 集 1 恒等。双发发射伪影 2 键照旧在册
  （exact-len 双腿各 2 dup + WINdow 双腿各 2 dup）。

## prev-session orphan leg1 旁证（不入正账）

前会话（API 限流 13:37 终止）遗留 orphan leg1
（11:28:33–11:32:55，rc=2，同码态同二进制——换代即该
腿 11:29:42 落盘）：P 8306，vs fresh leg1 仅 3 键差
——End/Home 零发射 + 族 optional 整文件键 Fail→
Timeout（均族摆动带内）+ **dialog-overlay 哨键
"dialog computed overlay initially 'none'" 瞬态红**
（fresh 双腿双 Pass 回绿）。orphan 窗正落对端 R5718
corpus 时窗（da3c22bd5 已双证并载「与本流 corpus 时窗
全重叠」）——负载相关瞬态红独立第三证。哨组计数以
fresh 双腿为准，orphan 仅作 contention 假说旁证归档。

## 观察哨复核（四组全零新翻红，以 fresh 双腿为准）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键双腿双 Pass：零新例第 4 轮
  （s512 时钟顺延；后续再现按新 <4 轮窗升级判定）。
- dialog-overlay 哨第 44 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 47 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 38 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断——
  s514 focus-dynamic leg2 文件级 Timeout 截断形本轮
  零再现，归观察窗记账非 hang）。

## 全桥套件绿窗收口跑（刀码同源旁证，既有清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1669P/0F rc=0**（test 阶段 24.34s；
filtered 2399 恒）。测试清单 vs s514（b514.k）：**NEW
8 = js_dom_bridge::transition_registry::tests::* 8 单
测**（刀体自带，对端申报「registry 单测 8/8」兑现）
+ **GONE 0，既有 1661 set 恒等**——刀对既有钉面零位
移，清单增长全可归因。

## 结论

s515 收口（他流刀归因轮，单刀 3fbaaf26d）：码态换新
3fbaaf26d（runner 换代 +24,040），html-semantics 面
**零语义位移清偿（申报=实测谱系第十八例）**——vs
s514 stable NEW 14（focus-dynamic 12 键回位预期兑现 +
族 End/Home 2 键首入 stable）+ GONE 0 + flips 0 非族
逐键恒等；族键摆动持续（detailed T/F 异相 + base
optional 整文件/子测形互换）；P 集双腿双 8307P 持位
第 37 轮；flaky 键零新例第 4 轮，picker 增益键第 38
轮保持，四观察哨零新翻红；全桥 1669P/0F 清单 NEW 8
刀体自带、既有 1661 恒等零破。截断形观察窗 s515–s518
第 1 轮零再现。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s515 stable 8511——
focus-dynamic 截断形观察窗 s515–s518 续记：s515 第 1
轮零再现，s516–s518 内再现按频率升级判定升 bug 排查，
窗外按带内间歇记账；族键发射形谱漂移观察维持——回位
/出位/混合形均按形谱记账不升级；flaky 键零新例时钟
顺延——再现方触发新 <4 轮升级判定；他流入树归因轮或
轻覆盖；对端刀信号持续：rendering dialog 16 红块协同
面 + css 文法刀后续刀 + editing generation-aware 挂
起刀 + ZW-3 querySelector 施工刀，落树后按他流刀归因
轮，join 实测为准）+ 候门决项等待用户拍板（video
P1+P2 立项 / 簇 D A2 立项 / s137）；rendering 流
dialog 16 红块协同面（归 rendering 流记账）。门开随
批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s515/{s515-leg1,s515-leg2}.log、
s515-leg{1,2}.rc、leg{1,2}.{start,end}、
s515-{l1,l2}.norm、l{1,2}.canon、flips.txt（零行）、
new.txt（14 键）、gone.txt（零行）、
s515.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s514.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 canon）、s515-bridge.log、
s515-bridge.rc、b514.k（复用）+ b515.k（清单对账）+
b515.{new,gone}（new 8 / gone 0）、
s515-leg1.prevsession.{log,rc} + leg1.prevsession.
{start,end}（orphan 旁证段工件）+ prev.norm/prev.
canon/prev.k（orphan 对账中间件）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
