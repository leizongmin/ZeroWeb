# DC-7 browser-ui/chrome §8.4.1A 组件批次（2026-07-01）

> M2 DC-7 进展证据：browser-ui/chrome 8/12 §8.4.1A 组件已落地（M1 既有 NavigationButtons/PageViewportFrame + 本轮 6 个）。
> 均遵循 props + build_* + on_activate 模式，由通用 widgets/patterns 组合，输出进统一 UI scene（不绕过 ui/render）。
> 本轮 SDK-only，未触碰 apps/browser（无 product-smoke 风险）。

## 本轮新增 6 组件

| 组件 | spec §8.4.1A 组合 | 输出 action | 测试锚点 |
|------|-------------------|-------------|----------|
| `PageLoadIndicator` | `ProgressIndicator` | （展示型，无 action） | `page_load_indicator::tests::*`（idle/loading/determinate+clamp） |
| `SecurityBadge` | `Badge` + `Tooltip` | （展示型；点击打开 SiteInfoPanel 由 shell overlay 处理） | `security_badge::tests::*`（4 态→tone/label；badge+tooltip 承载 label） |
| `BookmarksBar` | `Toolbar`（+文件夹 `Menu` 由 shell 注入子项） | URL→`Navigate`；文件夹→`OpenBookmark` | `bookmarks_bar::tests::*`（toolbar 条目；URL/folder/unknown 映射） |
| `BrowserTabStrip` | `TabBar`（+favicon/loading 由 shell 叠加） | `ActivateTab`/`CloseTab`/`OpenTab`/`ReorderTab` | `browser_tab_strip::tests::*`（tab_bar 构造+激活；action 映射；越界/no-op） |
| `FindBar` | `TextInputState` + `StatusBubble` | `FindNext`/`FindPrev`/`FindClose` | `find_bar::tests::*`（query 同步；计数 StatusBubble；action 门控） |
| `AddressBar` | `TextInputState` + `SuggestionList` + `SecurityBadge` | `Navigate`/`Search`（+建议→`Navigate`） | `address_bar::tests::*`（组合；URL vs 搜索分类；submit；建议激活） |

## 模式说明

每个组件 = 浏览器领域 props 结构（从 `zero-browser-shell` 状态投影）+ `build_*`（组合通用 widgets/patterns）+ `on_activate/on_submit/...`（action id → `BrowserAction`）。组件**不直接绘制**，也不绕过 ui/render；绘制发生在通用 widgets 的 paint 阶段。

## 覆盖率（evidence/coverage-20260701-030502.txt）

新 6 组件 line coverage：address_bar 100% · bookmarks_bar 100% · browser_tab_strip 100% · find_bar 98.48% · page_load_indicator 100% · security_badge 100%。browser-ui/chrome crate ~99%。

## 未完成（剩余 DC-7）

- 剩余 §8.4.1A 组件：`SiteInfoPanel`（Popover+DialogScaffold+ListView+Toggle+Button）、`PermissionPrompt`（DialogScaffold+Popover+Button+Toggle）、`DownloadPanel`/`DownloadItemView`（Popover+ListView+ProgressIndicator+Button+Menu）、`BrowserMenu`（Menu+ContextMenu+Separator+IconButton）——overlay/dialog 重，随 `ui/overlay` 接入 + shell 注入子项模型落地。
- chrome 组件 paint → scene snapshot：需通用 widgets 的 Widget::paint 管线接通后，把 chrome 组件渲染到 Scene 并 golden snapshot（DC-7 完整验收项）。
