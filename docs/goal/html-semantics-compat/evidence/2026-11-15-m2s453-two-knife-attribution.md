# M2 s453 — 双刀入树全量归因轮：target expando + g50 零位移清偿 + onblur following-label 上下文敏感新观察族定谳（solo A/B）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s452 下一步「target expando 刀落树后按他流刀
归因轮全量双腿对账」。本轮开工 `git pull --rebase` 入双源刀组合树：
**8ee02114e（target expando 刀，engine shim part01/04/05 `target` 名
反射 tag 门限化 {a,area,form,link,base}）+ 7bc31687d（渲染流 g50 刀，
style-system/layout-engine/css-parser apply 期视口单源 + vertical-
align <length-percentage> 三层）**，另 ecaffa184（R5602 docs）→ 按
预案执行全量归因轮。

## 双腿 join（ecaffa184 树，make testharness-html-semantics ×2）

- leg1 **8513 键（8306P/201F/5T/1U）**，leg2 **8516 键（8305P/204F/
  6T/1U）**；disagree 集 8 键 = select-home-end 族 7 键（相位轮换）
  + onblur 键 1（见下）。
- **stable 8509（8305P/199F/5T/1U，norm 配方口径）**。
- **vs s452 stable（8514 = 8306P/202F/5T/1U）恰 4 键差集，零 GONE/
  零 NEW**：
  1. select-home-end 族 3 键（主文件 End/Home/PageUp 子测 Fail 相位
     本轮入双腿振荡出 stable；PageDown + detailed 文件级相位继续族内
     轮换）——族相位轮换记账口径不变；
  2. **`checkable-active-onblur-with-click.html` :: Checkboxes …
     following label（非族键，Pass→腿间振荡出 stable，P 集 norm 口径
     8306→8305 首次破缺，−1 全由该键构成）**。

## onblur 键定谳：pre-existing 上下文敏感（双刀零位移）

- post-knife solo ×3（FILTER 单文件）：**3/3 恒 4F/3P**——following
  label 对（checkbox+radio）恒 Fail，parent label 对（红池在册 onblur
  2）恒 Fail，press-it 对 + Wait for load 恒 Pass。
- **pre-knife 树 A/B**（checkout 26272fa25，重建 provenance 在案：
  solo1 日志含 zero-engine → zero-wpt-runner 全重编译链，include_str!
  陈旧二进制前科已排除）：solo ×3 **同型 4F/3P 恒定**。
- **定谳**：following label 对 solo 冷启动恒 Fail 为 **pre-existing
  上下文依赖**（sweep 暖上下文多 Pass），与双刀无因果——刀面核对：
  target expando 刀仅触 `target` 属性名（本测试无 target 语义路径），
  g50 刀 vertical-align/视口面与 :active/label 机制无交集。
- sweep 内表现：leg1 双子测 Pass / leg2 checkbox Fail radio Pass——
  暖上下文边界振荡新形态（s449→s452 四轮该键双腿恒 Pass 在 stable，
  本轮首次振荡）。**列为观察哨**：后续轮次跟踪是否回入 stable。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first）`cargo test -p zero-engine --lib
js_dom_bridge`：**1643P/0F rc=0**（test 阶段 18.24s；基线 s452 1643P，
+0）——target expando 刀直改 shim part01/04/05（js_dom_bridge.rs
include_str! 本体），桥窗零回归为其行为向后兼容的直接旁证。

## 结论

s453 收口（双刀入树全量归因轮）：**双刀在 html-semantics 面零位移**
——4 键差集全数归因于 select-home-end 族相位轮换（3）+ pre-existing
上下文敏感键振荡（1，solo A/B 定谳非刀因果）。P 集主口径记账：norm
8305（−1 onblur 键，观察哨维持），基带口径 **8305P/203F/6T/1U** 结构
不变。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137
（s445 已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零
hang）。

工件：/tmp/s453/{s453-leg1,s453-leg2}.log、s453-{l1,l2}.norm、
s453-stable.norm、s453-vs-s452.diff、s453-solo-{1,2,3}.log、
s453-solo-pre-{1,2,3}.log、s453-bridge.log、s453-bridge.rc。
