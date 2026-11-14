# M2 s445 — 基带巡查轮：ip C1 + g47 三刀入树归因清偿（零位移，P 集恒等 8305）+ 全桥绿窗 1637P/0F

日期：2026-11-15。承前：s444 下一步「基带巡查轮（全量 sweep 或轻量覆盖，
他流入树一并归因）+ 候门决项提示用户（video P1+P2 / 簇 D A2 / s137）」。

## 归因面：s444 收口后入树三刀

| 刀 | 流 | 触面 |
|---|---|---|
| 607018f9a（ip C1 容器尾拆分序反转刀） | editing-compat | engine 编辑域 |
| 616fb3b57（g47 object-position canonical + conic calc 色标槽型别门） | 渲染流 | css-parser + engine computed_style.rs |
| a3b89b387（g47 <position> 分面定谳，bg 宽容/严格分治 + 逻辑关键字） | 渲染流 | css-parser + style-system + engine computed_style.rs + painter/effects.rs |

## 上一 session 429 断点遗产复用（ip C1 归因，零新跑）

上轮 session 在 05:56 429 断点前已跑毕 e519f75d1 树（= s444 树 + ip C1，
g47 未入）双腿（/tmp/s442/s445-leg{1,2}.log 完整），join 分析未及落账。
本轮以 sed 归一化管道重建（管道先对 s444-leg{1,2} 复算逐字节 MATCH
验真）：

- **stable 8510（8304P/199F/6T/1U）**；vs s444 stable（8305P/203F/5T/1U）
  仅 6 差集，全数在册振荡族：select-home-end 主文件 4 子测 Fail 相位出
  stable（文件级 Timeout 形态入）+ detailed 变体单键 + modulepreload
  strict-origin 单键（s440 在册，同窗 Phase 轮换）。
- **ip C1 刀零位移定谳**，s444 挂账（e519f75d1）清偿。

## 本轮新跑双腿（g47 归因，a3b89b387 树）

a3b89b387 = prev 腿树 + 616fb3b57 + a3b89b387 两刀（eee336928 仅 docs），
链上无其他源码刀——**g47 增量隔离归因成立**。`make
testharness-html-semantics` 双腿（release runner 增量重建 51s；各 ~3-8min）：

- **stable 8511（8305P/199F/6T/1U）**。
- **vs prev stable：仅 1 差集 = modulepreload strict-origin 回位（P+1，
  在册负载振荡回绿，s440 同型）→ g47 两刀零波及定谳**（s443 时代 g47
  首刀 88dd6b3ef 已由 s444 双腿清偿，本刀为剩余两刀）。
- **vs s444：P 集逐键恒等（8305 = 8305，零 P 位移）**；F/T 差集 5 全数
  select-home-end 族相位轮换（4 子测 Fail 相位出 + 主文件文件级 T 键入
  + detailed 变体单键 leg1 T/leg2 F）——与 s444「族内相位轮换记账口径」
  同型，非真位移。
- **新基带：8305P/199F/6T/1U（stable 口径）+ select-home-end 族 5 键振
  荡记账**（4 子测相位轮换 + detailed 1 键 T↔F + 文件级 T 形态轮换）。

## 红池全数归账（199F 逐族）

select（the-select-element 143，含 listbox 变体 2）+ combobox 12 = **簇 D
155**；dialog 16 / img 13 / tabular 4 / video 4（VIDEO 门）/ embed 2 /
details 1 / textarea 1 / audio 1 = 渲染域/decoder 耦合面 **42**；input 2 =
onblur 在册族。**合计 199，无新增未归因红。**

## 全桥套件绿窗收口跑

guard 包裹（--compile-first，total-mem 8，time-limit 2400）`cargo test
-p zero-engine --lib js_dom_bridge`：**1637P/0F rc=0**（test 阶段 19.05s；
基线 s444 1625P，**+12 = 组合树他流新钉 bridge 面**）——覆盖 g47 触面
（engine computed_style.rs / painter/effects.rs）的桥级验证。

## 结论

s445 收口（基带巡查轮，零源码改动）：**in-domain 自由面维持实质清零**，
三刀入树（ip C1 + g47×2）全部零位移清偿归因，P 集 8305 恒等基带保持。
挂用户门维持：GB-20261114-VIDEO（video P1+P2 立项决策）+ GB-20261031
（簇 D customizable select/combobox 立项征询，含 combobox 12 并入 +
styles.css ×47 defer）+ s137（form-nameditem）——候门决项已按 s444 下一
步提示用户（飞书）。hang 观察项静默哨维持（本轮双腿零 hang 复现）。

工件：/tmp/s442/{s445-l1,s445-l2,s445-stable}.norm（prev 腿重建 join）+
/tmp/s445/{fresh-leg1,fresh-leg2}.log、fresh-{l1,l2}.norm、bridge.log、
bridge.rc。清场：零 CPU burner 残留，语料零新增资产。
