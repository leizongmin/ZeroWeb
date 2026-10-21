# M3-S3 — 页面查找真实链路（P4 最小面）+ 演示流记账

**日期**: 2026-09-27
**goal**: [docs/goal/desktop-browser.md](../../desktop-browser.md)（M3 收口切片，DC-3 第三条）
**环境**: 同 M1/M2/M3（Linux x86_64，无人值守 `xvfb-run -a` 真实 X11 窗口；test-guard 包裹）。

## 1. 基线事实与最小 API 面决策（P4 评估结论）

**基线**：Ctrl+F 查找栏 UI、`FindState` 状态机（current/total/case/whole_word）、
`find_set_matches` 计数回写口全部就位，但**全仓无任何调用方**——键入查询后
`total_matches` 恒 0，无高亮无计数：查找是孤岛。

**最小面决策（入口文档执行协议：最小面自主做）**：零 engine/renderer crate 改动，
全部落在 `apps/browser`（§9 边界内）——**browser 侧已有页面 glyph 图元**
（`last_render.primitives.glyphs`，选区/命中测试同源），查找即对 glyph 序列做
匹配定位：

- `page_find.rs`（纯函数 + 5 单测）：逐 glyph 取源码点（`glyph_id` 保留源 Unicode
  码点，shaping 后不丢）组成文本流，滑窗匹配 `query`；大小写敏感/全字匹配选项
  直通 FindState 既有开关；返回 glyph 区间 + 文档坐标包围盒。
- **已知最小面局限（记账）**：连字（fi/ff 等）glyph 的多码点经 `source` cluster
  恢复才完整，本面按逐 glyph 码点匹配——ASCII 查询语义完整，连字语言完整匹配
  属后续切片（upstream 语料可挂 rendering-compat 流评估）。

## 2. 真实链路（apps/browser 内闭环）

| 环节 | 位置 | 行为 |
|------|------|------|
| 匹配计算 | `refresh_find_matches`（app.rs） | 查找栏键入/退格/开关切换/页面（重）加载后重算；匹配数写回 `find_set_matches`（计数 UI 数据源） |
| 逐项滚动 | `scroll_to_current_match` | Enter / F3 / Ctrl+G / Ctrl+N 后：匹配在视口外时滚至视口 25%/75% 参考线（上/下区分，保留上下文） |
| 高亮绘制 | `app_render.rs` 页面 glyph 前插 | 全部匹配淡黄底（与选区高亮同款几何：glyph.x × s + content 偏移，字宽 × 0.55）；当前匹配橙底 + 描边 |
| 失效 | `navigate_to_request` / `find_close` | 匹配缓存清空；查找栏开着时页面加载完成自动重算 |
| 配色 | `colors.rs` 三套主题 | `find_other_match_bg` / `find_current_match_bg` / `find_current_match_border` |

## 3. 演示流（可脚本重放）

```sh
bash scripts/desktop-browser-m3-smoke.sh        # 腿 3 find-flow（与 download/menu-zoom 同轮三 PASS）
```

- 驱动状态机：`apps/browser/src/find_smoke.rs`（`--find-smoke-base/-dir` CLI）
- fixture：`examples/m3-downloads/find.html`——"needle" 恰 5 处，含首屏内 2 处 +
  900px 下边距的首屏外 1 处 + 近底部 2 处
- smoke 只读面新增：`smoke_find_state`（active/query/current/total）、
  `smoke_scroll_y`（滚动定位断言面）

| 步骤 | 输入路径 | 断言 |
|------|----------|------|
| seed-load | `navigate_to` find.html | 快照 URL 落地 |
| find-count | **Ctrl+F + 逐字符键入 "needle"** | 查询落地 + `total==5`（fixture 精确值）；截图 01（计数 1/5 + 高亮） |
| find-next | **Enter** | current 1→2；首屏内匹配预期不滚动；截图 02 |
| find-advance | **Enter** | current 2→3 + **首屏外匹配必触发滚动**（scroll_y 0→636.3 = max_scroll） |
| find-closed | **Escape** | find_state 完全复位（query/current/total 归零）；截图 03 |

## 4. 结果（2026-09-27 实测）

- **find-flow 腿全绿**（与 download-flow/menu-zoom-flow 同轮三 PASS），复跑通过
- 截图证据：`.acceptance/desktop-browser-m3/find-flow/01…03.png`
- 滚动定位实测：匹配 3（top=1131.9，视口 676）推进后 `scroll_y=636.3`（max_scroll
  钳制，匹配进入视口下参考线）；首屏内匹配正确保持不滚动

## 5. DC-3 判定映射（收口）

| DC-3 条目 | 状态 | 证据 |
|-----------|------|------|
| 下载管理器演示流全绿 | ✅ | evidence/M3-download-manager.md（M3-S1） |
| 页面查找演示流全绿（engine 文本搜索 API 就位） | ✅ | 本文 §1-§3：P4 以零 engine/renderer 改动的最小面收口（browser 侧 glyph 匹配），engine 文本搜索 API 不再是本 goal 依赖项；连字完整匹配局限记账 §1 |
| 缩放演示流全绿 | ✅ | evidence/M3-context-menu-zoom.md（M3-S2） |
| 右键上下文菜单演示流全绿 | ✅ | evidence/M3-context-menu-zoom.md（M3-S2） |
