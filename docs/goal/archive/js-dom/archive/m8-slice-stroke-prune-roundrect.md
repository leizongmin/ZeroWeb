# M8 切片：stroke 零长段剪除 + roundRect 闭合 + 环绕 join（R56i）

**日期**: 2026-08-16
**Commit**: `c218f196`
**上一轮**: R56h（`e332878f`）
**证据**: [../evidence/2026-08-16-r56i-stroke-prune-roundrect.json](../evidence/2026-08-16-r56i-stroke-prune-roundrect.json)

## 修复

1. **零长段剪除**：spec stroke 前移除零长线段——`moveTo(p)+lineTo(p)` 不画 round cap 圆盘；剪除在段列表构造处（prune.line/arc/curve 三用例）。
2. **roundRect 显式 ClosePath**：roundRect 是 spec 闭合子路径；`Path2D::round_rect` 追加 ClosePath + join 循环闭合环绕（`(i+1) % len`）——闭合环起点 join 缺失曾致 roundrect.closed 的 (50,25) 漏画。
3. **方案否决记录**：几何闭合探测（首尾相接）把「lineTo 显式回到起点」的开放子路径误判闭合 → 丢 square cap（`2d.line.cap.open` 跨目录 A/B 捕获）——**闭合判据在命令层不在几何层**。

## 验证

- **WPT path-objects**：196P/7F → **200P/3F**（prune ×3 + roundrect.closed，零回归）
- line-styles 2F 过程回归跨目录捕获同轮修（最终 0F）
- canvas 790（+1）/ engine v8 2153 / quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff；六跨目录 0F

## M8 剩余 3F（深项清单，非轻量切片）

| 用例 | 深项 |
|---|---|
| stroke.skew | 斜切 CTM 的平行四边形端面（端面在用户法向、经 CTM 斜切） |
| roundrect.end.3 | miter-limit 边界角部（11.5° 夹角 ratio=10 恰在 miterLimit） |
| isPointInStroke.scaleddashes | dash 相位命中测试（沿路径弧长参数化） |
