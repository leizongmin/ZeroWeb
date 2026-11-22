---
date: 2026-11-22
modules: tests/wpt-runner
---

# 动画时序域页的 flake 与回归归因：单窗三 bin 探针无隔离力

## 问题描述

knife M4-b 电池（2026-11-22）在 `css-animations/Element-getAnimations.tentative.html`
出现 P2F=16（b0 Pass → b1 Fail，`getAnimations … expected 1 but got 0`／
`[object Undefined]`），同窗三 bin 判别探针（HEAD 系 17/21 ×4、face bin 1/21 ×2）
据此定谳 face 真机制回归并 fail-stop。后续五组快照 bin 零构建实验推翻该定谳：
P2F=16 为环境敏感 flake，face 语义无损（famd1b 时代同页 +16「真增益」勘误同根）。

## 根因分析

- 该页本征 wall-race：所有 bin 所有窗均 Timeout（`pending:2, tests:22` 恒在），
  pass 数＝墙钟前完成数，不是语义判定；1-form 的失败消息是动画时钟未前进时
  `getAnimations()` 返回空数组后的连锁读空，非值损坏。
- 1-form 由分钟级环境爆发产生、与 bin 无关：同款 HEAD bin 在另一窗自身出现
  1/21；face bin 在两个独立窗回到 17/21 满基线；同 bin 四连跑无交替；爆发窗
  边界约 10-25 秒。
- 「同窗三 bin 隔离」的探针若按分块顺序发射（b0 块→b1 块→f1b0 块），爆发恰罩
  中某块即伪造 bin 相关性——分块顺序设计对时窗变量无隔离力。臂 mtime 序列是
  识别顺序设计的法证入口。

## 解决方案

判别规则（动画时序域页一律适用）：

1. 同 bin 跨窗翻转即 flake 铁证（无须再找其他证据）。
2. 真机制回归按 bin 确定，face bin 跨窗回满基线是不可能形态——出现即排除回归。
3. 受控重发设计：b0/b1 腿背靠背相邻（同窗）；仅 b1 独有损失立真回归，单臂
   1-form 判窗污染不入 P2F。
4. 该族页单一读数不入任何流的 F2P/P2F 口径（同页先后产生过 +16 假增益与
   -16 假回归两个方向）。
