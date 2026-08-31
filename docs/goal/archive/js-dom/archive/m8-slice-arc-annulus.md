# M8 切片：弧自适应段数 + 真圆环带 + 用户空间 CTM 弧（R56g）

**日期**: 2026-08-16
**Commit**: `d0e93adc`
**上一轮**: R56f（`9ef44a8d`）
**证据**: [../evidence/2026-08-16-r56g-arc-annulus.json](../evidence/2026-08-16-r56g-arc-annulus.json)

## 三类独立缺陷与修法

| 缺陷 | 表现 | 修法 |
|---|---|---|
| 弦切向偏差翻正 butt 端面投影 | shape.1/3：距弧 40px 的画布区被首弦斜段矩形覆盖 | N = min(512, 64·lw/r) 自适应段数 |
| 折线伪节点覆盖洞 | shape.2：(1,1) 距弧 18.6 < half 50 却 miss（超短弦投影全落段外，Miter join 不画伪节点圆盘） | `blit_arc_annulus` 真圆环带后处理（\|dist−r\| ≤ half ∧ θ∈[span]） |
| CTM 半径未变换 | scale.2：scale(100,100) r=0.6 应成 r=60，旧路径画 0.6px 小点（arc() 只变换圆心） | 非恒等 CTM 逆变换 → 用户空间圆弧 → 逐点正变换（R56f 模式） |

## 过程中被 A/B 否决的方案

1. **interior-clamp**（段端点 t 放宽补偿覆盖洞）——被 selfintersect.1 回归否决：放宽折线首末段外区域 = 改 butt 端面语义。
2. **identity-CTM 判定笔误**（t.d==0 应为 ==1）——annulus 覆盖为零；debug trace 5 分钟定位。

## 验证

- **WPT path-objects**：186P/17F → **192P/11F**（arc.shape ×4 + arc.scale ×2；shape.2 过程回归两次同轮修复；零回归）
- canvas 788（+2 单测）/ engine v8 2153 / quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff；六跨目录 0F
