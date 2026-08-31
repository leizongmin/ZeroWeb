# M8 切片：isPointInPath 族 nonzero 对齐（R56d）

**日期**: 2026-08-16
**Commit**: `16f0d885`
**上一轮**: R56c（`bd027c98`，见 [m8-slice-nonzero-fill-rule.md](m8-slice-nonzero-fill-rule.md)）
**证据**: [../evidence/2026-08-16-r56d-ispointinpath-rule.json](../evidence/2026-08-16-r56d-ispointinpath-rule.json)

## 切片目标

master.md 下轮候选 (a)：isPointInPath 族（basic/winding/edge/invalid/multi.path 五用例）——旧实现用 `point_in_polygon` 奇偶 ray-casting，spec 默认 nonzero 绕组。

## 修复面

| 面 | 变更 |
|---|---|
| `context_impl.rs` | `is_point_in_path_rule` / `is_point_in_path_for_rule`（复用 R56c `fill_rule_spans`）；路径上点算 inside（span 闭区间 + on-segment 兜底，零长度退化段排除）；±Inf 顶点收缩 ±MAX/4 |
| `raster.rs` | 无 MoveTo 直接 arc 的整圆 = 自包含子路径（subpath_start 重置到弧首点——杂散闭合对角线修复） |
| `js_dom_bridge/canvas.rs` | `isPointInPath` op 加 fillRule（args[2]）；新 `isPointInPathPath` op（Path2D 形式）；`scale` op ±Inf 钳 ±f32::MAX |
| `part05.js` | WebIDL union 首参校验（TypeError）+ CanvasFillRule 枚举校验 + path 形式参数位修正 |

## 验证

- **WPT path-objects**：168P/35F → **172P/31F**（isPointInPath.basic/edge/invalid/multi.path 修复；bigarc 过程回归同轮捕获修复；零回归）
- canvas 789 / engine v8 2152 / quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff
- 跨目录 line-styles/drawing-rectangles/transformations/reset/fillrect/compositing 全 0F；shadow 6F 基线既存

## 关键根因记录

1. **矩阵合成 0×Inf=NaN**：`scale(Number.MAX_VALUE)` 经 wire 解析为 inf → `transform.multiply` 恒等基底 `0×inf=NaN` 毒化 b/f 分量 → 全部顶点 NaN。单测 `scale(f32::MAX)`（有限）不触发——单测与 e2e 差异曾误导定位。
2. **flatten 子路径起点初值 (0,0)**：无 MoveTo 直接 arc 时 closepath-on-fill 补出弧末→(0,0) 杂散对角线（bigarc 第 17 段）。与 R56 roundRect 修复同模式：**自闭合命令须重置 subpath_start 到自身起点**。
3. **path 形式参数位**：`isPointInPath(path, x, y, fillRule)` —— shim 初版把 JS 形参 `fillRule` 位（第 3 位）当 y 坐标。
