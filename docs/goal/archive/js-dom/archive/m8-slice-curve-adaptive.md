# M8 切片：曲线段数自适应 + 像素方形覆盖（R56h）

**日期**: 2026-08-16
**Commit**: `e332878f`
**上一轮**: R56g（`d0e93adc`）
**证据**: [../evidence/2026-08-16-r56h-curve-adaptive.json](../evidence/2026-08-16-r56h-curve-adaptive.json)

## 修复

1. **曲线段数自适应**：`N = clamp(控制点折线长/8, 8, 512)`（quadratic/bezier 双分支）。固定 8 段对巨坐标曲线（shape 用例折线 ~13000px、画布内 t 窗口仅 ~2.4 段）弦偏差 48px；自适应后画布内 ~154 段（~0.01px）。
2. **stroke 像素方形覆盖**：判定半径 `half + 0.5`（像素内切半径）。真实光栅是面积覆盖——中心距带 0.4px 外但方形与带相交的临界像素须着色（(1,1)：中心 27.9 / half 27.5 / 方形角 27.2 相交）。应用于 solid + gradient 双光栅化器。

## 验证

- **WPT path-objects**：192P/11F → **196P/7F**（bezier/quadratic shape/scaled 四用例，零回归）
- canvas 789（+1）/ engine v8 2153 / quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff；六跨目录 0F

## 教训

- 曲线与弧的段数问题同构但参数不同：弧按 lw/r（端面精度）、曲线按控制折线长（弦偏差）——各自独立推导。
- scaled 与 shape 同构（等比放大）——修一处自然双收。
