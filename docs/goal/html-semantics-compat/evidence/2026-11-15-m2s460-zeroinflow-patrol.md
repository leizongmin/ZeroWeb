# M2 s460 — 守成巡查轻覆盖轮：零入树零刀同码树基带复测（detailed 文件级 F 双腿一致回入 stable，P 集恒等 8306 七轮连续）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s459 下一步「守成巡查（他流入树归因轮或轻
覆盖）+ wptfyi-css WAAPI 门刀落树对账预告」。本轮开工 `git pull
--rebase` 远端零新刀（HEAD = 6babd9aac，即 s459 收口 docs 提交，代
码树与 s459 双腿树全同；期间渲染流 R5609/R5610 均 docs-only）→ 无
归因对象，按预案执行纯轻覆盖轮（零源码零语料净变更，同码树基带复
测）。

## 双腿 join（6babd9aac 树，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/200F/6T/1U）**，leg2 **8516 键（8306P/204F/
  5T/1U）**；共享键状态分歧 disagree = 0。
- 单腿缺席 5 键全在 select-home-end 族主文件：整文件 Timeout 键
  leg1-only（leg1 整文件粒度）+ 4 子测 Fail 键 leg2-only（leg2 子测
  粒度）——主文件粒度相位腿间振荡延续；detailed 文件级本轮双腿一致
  **Fail** 相位。
- **stable 8512（8306P/200F/5T/1U，norm 配方口径，distinct 8510）**。
- **vs s459 stable（distinct 8509）零 GONE、恰 1 NEW**：
  detailed 文件级 Fail 键双腿一致回入 stable（s459 因腿间 T↔F 对冲
  出 stable）。族相位轮换记账口径不变，非族键逐键零差，零真位移。
- **P 集账本口径 8306 七轮连续恒等**。
- onblur 哨键双腿一致 Pass 在 stable（哨解除后第六轮 Pass 在册）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 19.41s；基线 s459
1643P，+0——零入树刀，无新钉预期）。

## 结论

s460 收口（守成巡查轻覆盖轮，零源码改动）：零入树零刀同码树基带复
现确认——基带形态 **8306P/200~204F/5~6T/1U，全部变动限于
select-home-end 族相位**（detailed F 回入 stable + 主文件粒度相位
腿间振荡延续），P 集口径 8306 七轮连续维持。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，不重
复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s460/{s460-leg1,s460-leg2}.log、s460-{l1,l2}.norm、
s460-stable.norm、s460-vs-s459.diff、s460-bridge.log、s460-bridge.rc。
