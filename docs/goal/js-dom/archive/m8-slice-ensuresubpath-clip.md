# M8 切片：ensuresubpath 族 + clip 相交/空 + ellipse 负半径（R56e）

**日期**: 2026-08-16
**Commit**: `eab37ea1`
**上一轮**: R56d（`16f0d885`，见 [m8-slice-ispointinpath-rule.md](m8-slice-ispointinpath-rule.md)）
**证据**: [../evidence/2026-08-16-r56e-ensuresubpath-clip.json](../evidence/2026-08-16-r56e-ensuresubpath-clip.json)

## 切片内容

| 族 | 语义修复 |
|---|---|
| ensuresubpath ×7 | lineTo 无子路径 = moveTo（无隐含 (0,0) 连线）；quadratic/bezier 无子路径 = **第一控制点成起点、曲线照画**（初版误读 no-op，WPT 期望图纠正）；arcTo 无子路径 = P1 加入、跳过 current→切点1 前导线（`flatten_arc_to` 加 `has_subpath` 参） |
| clip.empty / clip.intersect | `clip_path: Option<Path2D>` → `clip_paths: Vec<Path2D>`——spec 相交语义（`clip_applies` 全 AND；空路径 clip 交集空全裁）。旧实现空路径 early-return + 多次 clip 覆盖 |
| ellipse.basics | shim 负半径 → IndexSizeError（-0/0 合法） |

## 验证

- **WPT path-objects**：172P/31F → **181P/22F**（+9 修复零回归）
- canvas 793（+4 单测）/ engine v8 2153（+1 e2e）/ quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff
- 跨目录 10 个子目录 0F（clip 重构影响面广，全查）；shadow 6F 基线既存
- `test_clip_no_current_path_no_panic` 旧断言与 spec 相反（「空 clip 后正常绘制」），更新为全裁

## 教训

1. **语义拿不准先看 WPT 期望图**：「first control point is added」初读以为 no-op；ensuresubpath.2 期望图整片绿 = 退化直线必须画。
2. **守卫变量顺序**：`has_any_subpath = true` 若在 `flatten_arc_to(...)` 调用前执行，传入的已是 true——守卫恒失效（probe 17 段暴露）。
3. **clip 相交是 Vec+AND 最小正确模型**，save/restore 沿列表 clone 即可正确回滚。
