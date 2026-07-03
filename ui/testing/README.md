# zero-ui-testing

通用 UI SDK 的测试工具集合。提供场景快照、语义快照、布局边界快照和 golden 比较等设施。浏览器无关。

## 模块

| 模块 | 说明 |
|------|------|
| `scene_snapshot` | `snapshot_scene(&Scene)` 产出一组字符串描述（Fill/Stroke/TextBlob/Image/ExternalSurface 各自变体，含 to_u8 clamp） |
| `semantics_snapshot` | `snapshot_semantics(&SemanticsNode)` 产出 a11y 树文本表示 |
| `layout_bounds` | `snapshot_layout_bounds(&[(WidgetId, Rect)])` 排版快照（DC-2/DC-9/DC-12 布局 golden） |
| `golden` | `golden::compare_snapshots(actual, expected) → Result<(), SnapshotDiff>` 逐行对比，首差异行报告 |

Crate 还提供 `FakeClock`（与 `ui/animation::FakeClock` 共享，测试动画用）。

## 依赖

- `zero-ui-core` / `zero-ui-render`
- dev-dep：`zero-text-foundation`（测试 TextBlob/ExternalSurface 快照变体）
- 零浏览器业务 crate 依赖

## 测试

- `cargo test -p zero-ui-testing` — 11 测
- 覆盖：Scene/semantics/layout_bounds snapshot 全变体 / golden comparison / FakeClock
- Coverage：golden 100% / layout_bounds 100% / scene_snapshot 85–96.93%
