# zero-ui-devtools

通用 UI SDK 的开发者工具。提供 widget inspector、布局边界/语义/绘制区域叠加显示、timeline 性能分析等调试能力。`dev` / `test` feature gate 隔离。浏览器无关。

## 核心类型

| 类型 | 说明 |
|------|------|
| `Inspector` | Widget 检查器（hovered 追踪 + dev overlay 开关：`show_layout_bounds` / `show_semantics` / `show_paint_regions` + `any_overlay()`） |
| `Timeline` | 帧性能分析器（`begin_frame` / `end_frame` → min/max/avg/percentile(p50/p99)/jank_count/fps + `with_jank_threshold`） |

## 使用场景

- 调试 UI 布局问题：打开 layout bounds overlay
- 检查 a11y 树：打开 semantics overlay
- 性能优化：Timeline 指标发现 jank

## 依赖

- `zero-ui-core`
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-devtools` — 5 测
- 覆盖：Inspector hovered + overlay 组合 / Timeline min/max/percentile/jank/fps
- Coverage 99.16%
