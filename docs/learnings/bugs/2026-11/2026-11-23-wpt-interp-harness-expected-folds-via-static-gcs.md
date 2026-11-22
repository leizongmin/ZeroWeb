---
date: 2026-11-23
modules: engine, style-system
---

# WPT interpolation harness 的 expected 串经参考元素静态 gCS 折叠

## 问题描述

KC-5 sizing 动画插值电池（`css/*/{animation,}/*-interpolation.html`）出现一批「怎么修都修不对」的 GT 键：b0（无 face）全部 Pass，任何 face 迭代一碰 consult/插值臂就把它们打成 Fail。这些键的 expected 串形如 `calc-size(fit-content, size)`、`calc-size(auto, -6px + size * 1.3)`——引擎插值臂无论如何都产不出这种串，按串面做插值/翻转/合成全部落空。典型样本：`width from [auto] to [calc-size(fit-content, size)] at (0.5) should be [calc-size(fit-content, size)]`，b0 got='0px '，face 态 got='200px '。

## 根因分析

assert 消息里的 `should be [...]` 是**折叠前的原串**，不是比较值。`tests/wpt-runner/wpt-data/css/support/interpolation-testcommon.js`（:360-430）的真实比较流程：

1. expected 串被 `setProperty` 到**参考元素**（`expectedTargetContainer.target`）上，再 `getComputedStyle` 读回作 expectedValue；
2. 静态 apply 拒收的串（calc-size 组合形、带 `size` 表达式等）在参考元素上落 underlying → expected = **underlying 的静态串化值**（带尾随空格的 `'0px '`、`'none '`、`'10px '` 形）；
3. 断言 = `normalizeValue(gCS(actual.target))` vs `normalizeValue(expectedValue)`——两侧都是折叠伪影对折叠伪影。

因此 b0 Pass 的机制常常是**巧合对称**：b0 的宽松 parse_px 对垃圾串写 0 → width 槽 Px(0) → gCS `'0px '`，恰好等于 harness 参考元素拒收伪影 `'0px '`。引擎侧任何让 apply 变严格、让 consult 返回真插值串的改动都会打破这个对称（got 变成 `'200px '`＝布局 used px 折叠），键反而变红。另一层不对称：引擎静态 gCS 对 sizing 关键字回 computed 槽串（`width:min-content` → `'0px '`），而动画侧 K-fold 走全量布局回 used px——同一个 `auto` 两侧折叠结果不同（`'0px '` vs `'200px '`）。

## 解决方案

- 判读时先算 **fold(expected)**：把 expected 串经「静态 apply → gCS 串化」同一链路折叠，got 与 fold(exp) 串等才是真 Pass；`should be` 原串只是消息文本。
- 修 face 的正确姿势是**对 consult 做对级抑制**（consult 返 None → 级联 underlying），让 got 逐位回到 b0 路径——等价于复现 b0 的巧合对称，构造上零回归。抑制谓词用静态可得特征分流：`interpolate-size: allow-keywords` 逐元素 flag、calc-size basis 类型（KW-basis vs LP/any-basis）、通道（CSS Animations/WAAPI 离散翻转合法 vs CSS Transitions 无 allow-discrete 不启动）、composition op（pre-compose 车道只对 additive）。
- 探针页复刻 harness 双容器结构（actual/expected 两个 targetContainer）才能拿到与电池同构的 got 证据；单元素探针读不到折叠通道。
