# M2 s471 — 他流刀入树归因轮：wptfyi-css 流 WAAPI 门刀 107bcd935（shim part03+part05，Element.prototype 挂 animate/getAnimations）html-semantics 面零位移清偿（P 集刀前刀后恒等 8306，唯一非族变动 dialog-overlay 键 solo 定谳瞬时 flake 立观察哨）+ 全桥绿窗 1643P/0F

日期：2026-11-15。承前：s470 下一步「守成巡查（WAAPI 刀落树则归
因轮）」。s470 双腿跑在刀前树 07312f8fd；s470 轮中 push 前 pull
携入 css 流源码刀 **`107bcd935` fix(engine) WAAPI 门刀落地**（同
窗口 eba29cc85 收口 docs 描述：shim part03 `Element.prototype`
挂 animate/getAnimations +34、part05 has 白名单补两名 +7，刀面自
证五组 A/B 净 +184P 零回归、engine lib 4032P/0F）→ s471 执行他
流刀归因轮，半径申报 engine shim part03/part05（WAAPI 门面），
html-semantics 面预期零位移，join 实测为准。

## 双腿 join（bc009e7a3 树 = 刀后组合态，make testharness-html-semantics ×2）

- leg1 **8513 行（8305P/200F/7T/1U）**，leg2 **8513 行（8306P/
  200F/6T/1U）**；共享 distinct 键 8511，腿键集完全一致
  （leg-only 0），**disagree = 2**：
  ① detailed 族整文件键（leg1 T ↔ leg2 F，既有族相位谱形）；
  ② **dialog-overlay same-origin 面 `dialog computed overlay
  initially 'none'` 键（leg1 F ↔ leg2 P，非族单腿翻红）**。
- select-home-end 族本轮形态：4 子测键双腿全部缺席，族主文件双
  腿只发整文件键 Timeout（既有相位谱形，s464 leg1 同形在册）。
- **stable 8511（8305P/199F/6T/1U，norm 配方口径，distinct
  8509）**。
- **vs s470 stable（刀前树）恰 3 GONE（族 End/Home 2 子测 F 双腿
  缺席 + dialog-overlay P 非族）+ 1 NEW（族整文件 T 双腿入）**；
  共享键零状态翻转（flips=0），余非族键逐键零差。
- **dialog-overlay 键 solo 定谳（s468 modulepreload 先例流程）**：
  该键 s468-s470 stable 全 Pass（跨刀前刀后树），本轮仅 leg1 单
  次 F；solo A/B（FILTER=dialog-overlay，同树同二进制 ×3）**三跑
  全 Pass rc=0** → 定谳全量 sweep 负载条件下瞬时 flake，非刀致。
  **立观察哨**（与 modulepreload 哨并列第二个非族观察哨）：后续
  轮次再翻红则升级调查；单次翻红不回头改记既往轮次口径。
- **归因定谳：WAAPI 门刀 107bcd935 html-semantics 面零位移清
  偿**——stable 内 animate/getAnimations/WAAPI 匹配键 2 个刀前刀
  后恒等，checkmark/picker-icon animation 键恒 F 无形变，族变动
  全数族相位，P 集破位键 solo 定谳非刀致；P 集账本口径 **8306 →
  8305 单轮破位**（dialog-overlay 键出 stable，solo 定谳瞬时
  flake，预期下轮回位）。
- **modulepreload 观察哨第 3 轮复核无再翻红**：referrerpolicy 面
  7 键双腿全 Pass（s450 起累计 42/43 腿），哨继续盯防。
- onblur 哨键组 10 键双腿一致 Pass 在 stable，红池 onblur 2 F 键
  照常在册——组成与 s470 逐行全同（哨解除后第十七轮）。
- stable T 键集 6 键 = 5 非族 + 族整文件 T（族相位回归 s469-s470
  间位形）。

## 全桥套件绿窗收口跑（刀后树旁证）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1643P/0F rc=0**（test 阶段 39.21s；基线 s470 1643P，+0，
filtered 2389 同值——刀直改 shim part03/part05 未触桥面，钉面零
破零新增，为刀面无回归旁证）。

## 结论

s471 收口（他流刀归因轮，本流零源码改动）：css 流 WAAPI 门刀
107bcd935 html-semantics 面零位移清偿——刀前后 stable 差集全数限
于 select-home-end 族相位（4 子测缺席 + 整文件 T 入）+ dialog-
overlay 非族键 solo 定谳瞬时 flake（已立观察哨），非族键逐键零
差，P 集口径 8305 单轮（预期下轮回位），全桥绿窗钉面零破。挂用
户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。hang 观察项静默哨维持（本轮双腿零 hang）。

工件：/tmp/s471/{s471-leg1,s471-leg2}.log、s471-{l1,l2}.norm、
l{1,2}.kstat、s471-stable.norm、st470.kstat、ks4{70,71}、
a4{70,71}、ob4{70,71}、solo-{1,2,3}.log、s471-bridge.log、
s471-bridge.rc。
