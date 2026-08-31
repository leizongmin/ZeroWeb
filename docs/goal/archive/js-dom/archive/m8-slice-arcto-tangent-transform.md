# M8 切片：arcTo 切线弧中心修正 + 各向异性 CTM（R56f）

**日期**: 2026-08-16
**Commit**: `9ef44a8d`
**上一轮**: R56e（`eab37ea1`）
**证据**: [../evidence/2026-08-16-r56f-arcto-tangent-transform.json](../evidence/2026-08-16-r56f-arcto-tangent-transform.json)

## 切片内容

1. **双实现统一**：`arc_to_tangent_segments` 提取为 path.rs 共享函数。并行流 `76655cc4` 只修了 path.rs（Path2D hit-test 路径），ctx.stroke/fill 仍走 raster.rs 旧线段近似——flatten 双实现漂移。
2. **圆心计算修正**：候选 `T1 ± rot90(u1)·r` 取与 T2 距离 = r 者（规范恒等）。旧「u2−u1 平分」不垂直 u1 → 弧张角偏差（probe：弧止于 (114,6) 而非 T2(150,0)）。
3. **各向异性 CTM**：弧在用户空间是圆、经 CTM 变换为设备空间椭圆——逆变换控制点 → 用户空间构造 → 输出逐点正变换（`arcTo.scale` 的 scale(0.1,1)）。
4. **shim 守卫补齐**：`76655cc4` message 声称的 arcTo 负半径 IndexSizeError hunk 未随提交进入（合并态 arcTo.negative 仍 fail），补 ctx + Path2D 两处。
5. **死代码清理**：`flatten_arc_to` + `compute_arc_to_geometry` + 10 个旧单测删除。
6. **R56e 单测断言修正**：no-subpath arcTo 什么都不画（真浏览器语义；R56e 版「弧仍画」是误读）。

## 验证

- **WPT path-objects**：181P/22F → **186P/17F**（arcTo 五用例，零回归）
- canvas 786 / engine v8 2153 / quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff；六跨目录 0F

## 教训

1. **并行流提交可能丢 hunk**——合并态先跑 fail 集核对 message 声称的修复是否真在。
2. **flatten 双实现漂移是结构性风险**——共享函数根治。
3. **圆心几何用候选+距离恒等验证**比平分向量推导可靠。
