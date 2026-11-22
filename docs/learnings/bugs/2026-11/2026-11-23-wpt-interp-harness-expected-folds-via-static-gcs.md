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
- **抑制不是万能回 b0**（2026-11-23 上午双流勘误）：b0 Pass 有两条互不同构的机制——①consult 真命中（HEAD canonicalize 宽松 accept→串化伪影）②None→级联 underlying。先定谳目标键属哪条再选刀：属②才可用对级抑制；属①时抑制会落到下游 **S1 used_size_echo**（callbacks.rs 独立于 consult 的静态 echo 层，`.target` block auto 形回 parent 200px），抑制后 got 仍是错值。方向应是**通道对称化**（canonicalize 产出收敛到 cascade 静态折叠形）而非更细抑制。多源归因前先核每层可达性（如 K-fold 需 consult_raw=Some 才可达；'200px ' 在 r5/r65 两代是 canonicalize used 解析与 S1 echo 两个不同源）。
- 抑制谓词可用的静态特征：`interpolate-size: allow-keywords` 逐元素 flag、calc-size basis 类型（KW-basis vs LP/any-basis）、通道（CSS Animations/WAAPI 离散翻转合法 vs CSS Transitions 无 allow-discrete 不启动）、composition op（pre-compose 车道只对 additive）。
- 探针页复刻 harness 双容器结构（actual/expected 两个 targetContainer）才能拿到与电池同构的 got 证据；单元素探针读不到折叠通道。
