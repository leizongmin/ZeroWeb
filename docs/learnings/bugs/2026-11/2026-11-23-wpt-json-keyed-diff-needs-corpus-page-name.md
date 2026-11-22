---
date: 2026-11-23
modules: wpt-runner,engineering-process
---

# WPT JSON 键级差分必须 (corpus, page, name) 三元组键控

## 问题描述

KC-5-r5 电池（wptfyi-css，2026-11-23）判读收账时，键级 F2P/P2F 池差分用「断言名 name」单键做集合运算，得出 vs r4＝F2P new 234/lost 51、P2F resolved 429、vs r3＝+119/−195 的 delta。数字表面自洽，随后被双 fork 交叉验算揭穿：与官方池总账算术不相容（623−51+234=806≠832）。误判险些入库——r4 echo 判读证据里的池差分数字也是同型错误。

## 根因分析

testharness 断言名是插值 harness 生成的模板串（`CSS Animations: property <width> from [auto] to [calc-size(50px, size)] at (0.5) should be [...]`），**跨语料、跨页同名**：同一断言串会同时出现在 `css-sizing/animation/width-interpolation.html` 与 `css-values/calc-size-width-interpolation.html`。name-only 键控在两处塌缩：

1. 池构建时 dict 后页覆盖前页，同一 name 的多页状态只剩一份；
2. 集合差（`A − B`）把跨语料重复条目折叠成一条，new/lost 双向都系统性低估。

塌缩方向恰好部分对冲，错误总数与真值的偏差不刺眼，不做算术验算看不出。

## 解决方案

- 差分键一律用 `(corpus, page, name)` 三元组（最小充分键；page 内同名重复由 JSON 数组序自然保留）。
- 转录任何池 delta 前必跑算术恒等式双基线验算：`old − lost + new == new_total`，对 r3、r4 两个基线各自成立才可入库。两基线同时吻合时键控基本可信（单基线吻合可能是巧合对冲）。
- 交叉验证手段：另一 fork/会话独立重算后对数（本次勘误即由并行 fork 的锚定重算揭穿，仲裁 NOTES 落勘误附录后采彼修正）。

## 教训边界

页级 reconcile（gained/lost 页级净额）会稀释单案回归，case-name 键级差分是既有结论；本次补的是键级的**充分性**：键不唯一时差分静默错，恒等式验算是唯一廉价防线。
