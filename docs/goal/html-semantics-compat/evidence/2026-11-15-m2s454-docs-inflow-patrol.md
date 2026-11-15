# M2 s454 — 守成巡查轻覆盖轮：他流入树仅 wptfyi-css 流 docs 零刀基带复测（共享键零分歧 disagree=0，P 集口径 8306 回归，onblur 观察哨解除回入 stable）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s453 下一步「守成巡查（他流入树归因轮或轻
覆盖）+ onblur following-label 键观察哨」。本轮开工 `git pull --rebase`
入他流 cf644d842（wptfyi-css 逐组转化率实测，diffstat 仅
docs/goal/wptfyi-css/ 两文件 +163 行）——**零源码零语料，代码树与
s453 双腿树（ecaffa184）全同** → 无归因对象，按预案执行纯轻覆盖轮
（零源码零语料净变更）。

## 断点遗产复用（上 session 429 终端）

上 session 09:42 提交 s453 docs（1943457c5）后即开跑 s454 leg1
（/tmp/s454 工件在案，log 09:42–09:46 落盘），随后 429 限流中断。
**provenance 核对**：reflog 证实 leg1 运行树 = 1943457c5，与
ecaffa184 及当前树 cf644d842 三者代码全同（git diff 仅 docs 两文件；
runner 二进制同码树缓存复用）→ leg1 直接复用，leg2 本轮新跑。

## 双腿 join（1943457c5 树，make testharness-html-semantics ×2）

- leg1 **8516 键（8306P/203F/6T/1U）**（遗产），leg2 **8513 键
  （8306P/200F/6T/1U）**（新跑）。
- **共享键状态分歧 disagree = 0**（join 逐键零互异——s453 谱系首次
  双腿全一致形态；s451 曾达成一次）。
- 单腿缺席键 3 全在 leg1 侧、全在 select-home-end 族主文件
  （End/Home/PageUp 子测 Fail；leg2 该文件整文件 Timeout 相位，
  子测未发出）→ 族相位轮换记账口径不变，零真位移。
- **stable 8513（8306P/200F/6T/1U，norm 配方口径，distinct 8511）**。
- **vs s453 stable（distinct 8508）零 GONE、恰 3 NEW，全数在册**：
  1. **`checkable-active-onblur-with-click.html` :: Checkboxes …
     following label（Pass，双腿一致回入 stable——s453 观察哨解除
     回合确认**；solo 冷启动恒 Fail 的 pre-existing 上下文敏感定谳
     不变，本轮为 warm sweep 双腿稳定 Pass 形态）；
  2. select-home-end 主文件文件级 Timeout 键（族相位入 stable）；
  3. select-home-end detailed 文件级 Fail 键（族相位入 stable；
     s453 该两文件级键因双腿振荡出 stable，本轮双腿一致）。
- **P 集账本口径 8306 回归**（s444→s452 九轮 8306 → s453 onblur 振
  荡破缺 8305 一轮 → 本轮 onblur 键回位 8306）。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 31.00s；基线 s453
1643P，+0——零入树刀，无新钉预期）。

## 结论

s454 收口（守成巡查轻覆盖轮，零源码改动）：他流入树仅 wptfyi-css
流 docs 巡查记录，同码树基带复现确认——基带 **8305P/203F/6T/1U →
本轮 leg 形态 8306P/200~203F/6T/1U，全部变动限于 select-home-end
族相位（3 键单腿缺席 + 2 文件级键回入 stable）与 onblur 哨键回位**，
P 集口径 8306 回归维持。onblur following-label 观察哨**解除**（回入
stable 双腿一致确认；若后续轮再振荡出 stable 可重开哨）。挂用户门
维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，
不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang，日志假阳性
命中均为测试名内 "load events" 字样）。

工件：/tmp/s454/{s454-leg1,s454-leg2}.log、s454-{l1,l2}.norm、
s454-stable.norm、s454-vs-s453.diff、s454-bridge.log、s454-bridge.rc。
