# M8 切片：nonzero fill rule + fillRule 透传 + roundRect 镜像绕组（R56c）

**日期**: 2026-08-16
**Commit**: `bd027c98`（rebase over 并行流 `02eb652e` colorMatrix filter / `34df6510`）
**上一轮**: R56b（`b1446e9b`，见 [m8-slice-path-objects-regression-closure.md](m8-slice-path-objects-regression-closure.md)）
**证据**: [../evidence/2026-08-16-r56c-nonzero-fill-rule.json](../evidence/2026-08-16-r56c-nonzero-fill-rule.json)

## 切片目标

master.md 下轮候选 (a)：`fill.overlap` / `winding.add` 族——偶奇光栅对嵌套同向子路径挖假洞，需方向感知 nonzero fill rule。这是 R56/R56b 段式迭代铺好结构后的自然收口。

## 根因（两个）

1. **旧偶奇配对破裂**：`blit_path_to_pixels` / `blit_path_gradient` 排序交点后两两配对——嵌套同向子路径（绕组 ±2）中心被配对挖成假洞（winding.add），重叠子路径重叠区被挖空 + 半透明单次 fill 语义破坏（overlap）。
2. **WebIDL 可选前置参省略**：`ctx.fill("evenodd")` 的 rule 落在**第一参位**（spec `fill(Path2D?, CanvasFillRule)` 前置可选参省略）——shim 初版只查第二参 → rule 丢失回落 nonzero。

## 修复

| 层 | 变更 |
|---|---|
| `crates/canvas/src/context/raster.rs` | `fill_rule_spans(vertices, sy, rule)`：交点带方向（段向下 +1 / 向上 −1，屏幕 y 向下）；NonZero 按 x 排序累计绕组取非零区间，EvenOdd 保持奇偶配对。三个 fill 消费端统一经此。`FillRule` enum + `reverse_subpath`（roundRect 单轴镜像段序反转——真浏览器沿参数边方向环绕，roundrect.winding 实证） |
| `context_impl.rs` / `mod.rs` | `fill_with_rule` / `fill_path_with_rule` / `blit_*_rule` 族，默认封装 NonZero 保持兼容；`FillRule` pub 导出 |
| `js_dom_bridge/canvas.rs` | host op `fill`（rule=args[0]）/ `fillPath`（rule=args[1]，首参 path id）解析 `"evenodd"` |
| `js_dom_shim/part05.js` | `fill(path, fillRule)` 首参字符串嗅探（WebIDL 前置省略）→ 交换到 rule 位 |

## 验证

- **WPT path-objects**：166P/37F → **168P/35F**（fill.overlap / winding.add / evenodd.1 修复，零回归）
- **单测**：zero-canvas +6（nonzero 嵌套同向/反向、evenodd 双矩形/嵌套、overlap 单层 alpha、roundrect 镜像对消）；engine +1 e2e（四场景）
- **门禁**：canvas 788 / engine v8 2152 / engine quickjs 1416 全绿；clippy 双矩阵零警告；fmt 无 diff；跨目录 line-styles/drawing-rectangles/transformations/reset 0F；shadow 6F 基线既存（stash 验证）
- pre-commit-guard PASS

## 过程教训

1. **shim↔host 参数索引按实际 wire 形式核对**：同一 op 的有/无 path 两形式参数位不同。probe hook `__zw_canvas_op` 实参（`join('|')` 日志）是最快定位法。
2. **同区域反向矩形对在 nonzero（绕组对消）与 evenodd（计数偶）下都不填**——两规则的区别需用同向双矩形或嵌套对验证（初版单测期望错误，分析后修正）。
3. `fill.length=0` 是 dispatcher 转发器 `function(){}.apply(this, arguments)` 的正常 length，非参数丢失信号。

## 接手记录

上一轮 R56c 中途被 429 中断（工作树遗留 5 文件未提交）；本轮 stash → `git pull --rebase`（远端 +3 并行流提交）→ pop 恢复，补齐单测 + e2e + 定位 evenodd.1 arg 索引 bug，全部验证后 land。

## 剩余（下轮候选）

- isPointInPath 族（basic/edge/multi.path）现奇偶规则——可续接 `fill_rule_spans` 对齐 nonzero 默认
- arc/arcTo/bezier/quadratic 形状精度族 ~12
- clip.empty/intersect、roundrect.closed/end.3、stroke.prune/skew 族
