# M2 s456 — 他流刀归因轮：渲染流 b7ecf7864 fix(layout) flex vertical-align 泄漏修复刀 html-semantics 面零位移清偿 + 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s455 下一步「守成巡查（他流入树归因轮或轻
覆盖）」。本轮开工 `git pull --rebase` 入他流刀 **b7ecf7864
（fix(layout): flex item 块化盒 vertical-align 不再泄入内部根级文
本 run）**——渲染流 g50 泄漏面修复刀（fa909476e bisect 定谳 +
995F 真态 FIXED=1），仅触 `crates/layout-engine/src/inline/
collect_items.rs`（+70/−3）；另 fa909476e/8dbd093d1/10abba9ac 三
docs 提交。**归因窗口源码变更仅此一刀** → 按预案执行他流刀归因轮
全量双腿对账。注意：s455 双腿跑在 cc8499531 树（不含该刀，彼时该
刀尚未 rebase 入树），归因基线为 s455 stable。

## 双腿 join（a8eeeae09 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**，leg2 **8513 键（8306P/200F/
  6T/1U）**。
- 共享键状态分歧 disagree = 1：select-home-end detailed 文件级键
  T↔F 相位对冲（出 stable，与 s455 同键同型振荡）。
- 单腿缺席 5 键全在 select-home-end 族主文件：4 子测（End/Home/
  PageDown/PageUp）leg1-only + 文件级 Timeout 键 leg2-only——主文件
  子测粒度 ↔ 文件级粒度相位腿间切换（与 s454→s455 同族两形态轮换）。
- **stable 8511（8306P/199F/5T/1U，norm 配方口径，distinct 8509）**。
- **vs s455 stable（distinct 8513）恰 4 GONE（族主文件 4 子测 F，
  leg2 粒度相位单腿缺席出 stable）、零 NEW**，非族键逐键零差。
- onblur 哨键双腿 9/9 全在，following label 对双腿 Pass 在 stable
  （哨解除后第二轮回位确认）。

## 归因定谳

**b7ecf7864 在 html-semantics 面零位移**——stable 差集 4 键全数
select-home-end 族相位轮换（本谱系已知腿间粒度振荡，s454/s455 同型
在案），零 GONE 外移、零 NEW、P 集恒等 8306。刀面核对：collect_items.rs
flex item 块化盒 vertical-align 泄漏面为布局内联收集路径，与
html-semantics 面 DOM 语义 shim 机制无交集；与 s445/s446/s453 谱系
g47/g49/g50 渲染流刀零位移结论同型。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 18.33s；基线 s455
1643P，+0——零 engine shim 触面，无新钉预期）。

## 结论

s456 收口（他流刀归因轮）：渲染流 b7ecf7864 fix(layout) 刀
html-semantics 面零位移清偿，红池归账结构不变（簇 D 155 + 渲染域/
decoder 42 + onblur 2），基带口径 8306P/199~204F/5~6T/1U 全部变动
限于族相位。**P 集账本口径 8306 三轮连续**。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，不重
复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s456/{s456-leg1,s456-leg2}.log、s456-{l1,l2}.norm、
s456-stable.norm、s456-vs-s455.diff、s456-bridge.log、s456-bridge.rc。
