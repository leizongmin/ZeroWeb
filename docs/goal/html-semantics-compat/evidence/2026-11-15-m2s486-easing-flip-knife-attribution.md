# M2 s486 — 他流刀入树归因轮：css 流 easing 感知离散翻转采纳刀 492a0f2e0（js_anim_registry flip_ok 形态门 eased 一致性采纳 + shim 零行为注释标记，corpus +21 零回归）html-semantics 面零位移清偿（双腿完全一致 stable 8511，非族逐键零差 + flips 1 全族相位谱），动画面零位移（picker 增益键第 9 轮保持），五观察哨零新翻红 + 全桥绿窗 1655P/0F（刀内嵌七自测合法入桥 1648→1655 纯增长零破）

日期：2026-11-15。承前：s485 下一步「守成巡查（他流入树归
因轮或轻覆盖）」。开工 `git pull --rebase` 携入 **492a0f2e0
feat(engine) easing 感知离散翻转采纳刀**（css 流 wptfyi-css，
19:53:03）：jsanim v2 fallback 检测器放宽——flip_ok 形态门
（恰 2 帧、端点 offset 0/1、direction Normal）eased 一致性
采纳（输出≡to 且 eased≥0.5 / ≡from 且 eased<0.5 / 恒值对无
条件），侧别失配/NaN 维持回落；落树面 js_dom_bridge/
js_anim_registry.rs +214（含内嵌 #[cfg(test)] 七自测）+
part03.js +2（`zw-easing-flip-adopt-2026-11-15` 刀标记注释，
Rust const 不引用不落二进制三试教训）。半径申报 engine 动画
面（jsanim 检测器），html-semantics 面预期零位移——s478
WAAPI 刀同动画面曾破预期出 picker 增益，本轮 join 实测为准
并重点盯 waapi 哨键——**实测兑现预期（申报=实测谱系第四例）**。

## 二进制 provenance

- leg1 起跑 make 重建（零 `Compiling` 行但 runner mtime
  19:57:34 晚于刀 19:53:03，size 126890752→126909176 +18KB
  ——「零 Compiling 行但重建」系统性现象持续在册）；leg2 零
  重建同二进制。刀后钉：`strings target/release/zero-wpt-runner
  | grep -c zw-easing-flip-adopt-2026-11-15` = 1 命中（+
  `_zwMakeIframeWin` = 6 持位）。桥跑同二进制。
- 码态核查：`git diff a520bc2f9..HEAD -- crates/` = 285 行
  （PR305 277 + 本刀 8），HEAD = e31ec6454 docs（码态
  492a0f2e0）。

## 双腿 join（e31ec6454 树 = 码态 492a0f2e0，make testharness-html-semantics ×2）

- leg1/leg2 **均 8513 行（双 8307P/199F/6T/1U）——双腿完全
  一致 disagree 0 + leg-only 0**（s479/s484 以来第三次全同
  形态），族 s479 形（optional 整文件 T + detailed F）。
  双腿零截断，零重建。
- join：**stable 8511（8305P/199F/6T/1U）**——本轮双腿同相
  位，族两整文件键（optional T + detailed F）均入 stable。
- **vs s485 stable（8510）：GONE 0 + NEW 1 + flips 1**——
  NEW = optional 整文件键（T）；flip = detailed 整文件键
  Timeout→Fail（族整文件键 T↔F 相位谱 s464 起在册，本轮双
  腿同 F 新摆位）；**非族键逐键零差**。
- T 键集 6 = 非族 5 恒等 + optional 整文件 T；**P 集账本口
  径 8307 持位**（双腿双 8307P；stable 机械口径 8305P 七轮
  同形）。双发发射伪影 2 键照旧在册（sort -u 去重吸收）。
- **动画面零位移旁证**：waapi 哨 2 键 stable picker-icon
  Pass（s478 刀增益第 9 轮保持）/ checkmark Fail——easing
  刀改 jsanim 检测器但 html-semantics 语料动画键零位移，与
  s478 WAAPI gCS 刀（首现 picker 增益）半径分野对号（gCS 值
  通道 vs fallback 检测通道）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 stable 10 键全在册 8P/2F（following-label 双键
  Pass 持位，红池 parent-label 2F 照常）。
- dialog-overlay 哨第 15 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 18 轮零翻红（15 键 stable 全 Pass）。

## 全桥套件绿窗收口跑（刀后树旁证，清单合法增长）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 22.37s；filtered 2389 同值）。
测试清单 vs s485：**1648→1655 纯增长 +7，GONE 0**——新增
7 测试全数 `js_anim_registry::tests::` 刀内嵌自测
（boundary_half_to_side / constant_pair_always_adopts /
ease_in_midpoint / foreign_value_rejects / linear_flip_sides /
marker_present / steps_clamp），与刀落树面（js_anim_registry.rs
+214 含 #[cfg(test)]）对号，钉面合法增长零破。

## 结论

s486 收口（他流刀归因轮，本流零源码改动）：easing 翻转采纳
刀 html-semantics 面零位移清偿——双腿完全一致（第三次全同
形态），stable 差集仅族相位键（GONE 0 + NEW 1 + flips 1 全
族整文件键相位谱），非族键逐键零差，「申报=实测」谱系第四
例（s479 d35 / s481 双刀 / s485 T72 / 本轮 easing）；动画面
零位移（与 s478 gCS 通道分野对号），P 集 8307 持位，picker
增益键第 9 轮保持，五观察哨零新翻红，全桥绿窗钉面 1655 零
破（刀内嵌七自测合法入桥）。挂用户门维持：GB-20261114-VIDEO
+ GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。
hang 观察项静默哨维持（双腿零真 hang，零截断）。

工件：/tmp/s486/{s486-leg1,s486-leg2}.log、s486-{l1,l2}.norm、
l{1,2}.canon、s486.{stable.keys,stable.canon}、flips.txt、
st486.sig、b486.list（vs s485 b485.list +7 GONE 0 对账）、
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复用）、
s486-bridge.log、norm.sh/canon.sh/join.sh（s478 沉淀管线复
用）。
