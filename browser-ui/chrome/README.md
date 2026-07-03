# zero-browser-chrome

浏览器专属 chrome 组件，由通用 `ui/widgets` + `ui/patterns` 组合实现。此 crate 是通用 UI SDK 与浏览器业务之间的**唯一允许耦合点**。

## 架构位置

```
browser-ui/chrome  ←── apps/browser
     │
     ├── ui/widgets（通用控件）
     ├── ui/patterns（通用组合模式）
     ├── ui/adapters/webview（WebViewWidget 包装）
     ├── zero-browser-shell（浏览器业务状态，UI-agnostic）
     └── ui/adapters/render-foundation（SDK chrome 渲染管线桥接）
```

## 提供的组件（spec §8.4.1A）

### 领域组件（12 个）

| 组件 | 实现方式 | 说明 |
|------|----------|------|
| `AddressBar` | `TextInputState` + `SuggestionList` + `SecurityBadge` | 地址栏 |
| `BrowserTabStrip` | `TabBar` | 标签页条 |
| `NavigationButtons` | `IconButton` × 3 | 后退/前进/刷新 |
| `SecurityBadge` | `Badge` + `Tooltip` | 安全状态徽章 |
| `BookmarksBar` | `Toolbar` | 书签栏 |
| `FindBar` | `TextInputState` + `StatusBubble` | 查找栏 |
| `PageLoadIndicator` | `ProgressIndicator` | 页面加载进度 |
| `BrowserMenu` | `Menu` + `ContextMenu` | 浏览器菜单 |
| `PermissionPrompt` | `DialogScaffold` + `Toggle` | 权限请求对话框 |
| `SiteInfoPanel` | `Popover` + `ListView` | 站点信息面板 |
| `DownloadPanel` | `Popover` + `ListView` + `ProgressIndicator` | 下载面板 |
| `DownloadItemView` | `ProgressIndicator` + `Badge` | 单个下载项 |
| `PageViewportFrame` | `WebViewWidget` | 页面视口容器（ExternalSurface marker） |

### Shell（3 种自适应）

| Shell | 适用形态 |
|-------|----------|
| `DesktopBrowserShell` | 桌面大屏 |
| `TabletBrowserShell` | 平板中屏 |
| `PhoneBrowserShell` | 手机窄屏（含 safe-area / 键盘避让） |

通过 `AdaptiveBrowserChrome` 按 `ViewportClass` / `PlatformClass` / `InputClass` 自动选择。

## 关键模块

| 模块 | 职责 |
|------|------|
| `shell` / `shell_demo` | Shell 声明树 build + demo fixture |
| `chrome_model` | `BrowserChromeModel::from_shell(&BrowserShell)` 状态投影桥 |
| `render` | `ChromePanel` 叶子控件 + `register_chrome_factories` |
| `sdk_render` | `render_chrome_via_sdk*` 渲染管线包装（含 viewport rect） |
| `i18n/` | 浏览器文案 `MessageCatalog`（ids + default_catalog + resolve） |
| `phone_demo` | 移动端 headless 集成 demo（Tap/Pinch/back 手势） |

## 依赖

- 通用 UI SDK crate：`zero-ui-core` / `zero-ui-widgets` / `zero-ui-patterns` / `zero-ui-render` / `zero-ui-runtime` / `zero-ui-i18n` / `zero-text-foundation`
- 适配器：`zero-ui-adapter-webview` / `zero-ui-adapter-winit` / `zero-ui-adapter-render-foundation`
- 浏览器桥接：`zero-browser-shell` / `zero-render-foundation`
- 手势/导航/平台：`zero-ui-gestures` / `zero-ui-navigation` / `zero-ui-platform`

## 语义色映射

所有 chrome 组件经 `chrome_color_themed(name, &SemanticTokens)` 从 semantic token 取色，零硬编码浏览器色值。映射表：secure → success / insecure → warning / dangerous → error 等。

## 测试

- `cargo test -p zero-browser-chrome` — 86 测
- 覆盖：shell 布局 / chrome_model 投影 / 12 组件 props+build / paint→Scene 快照 / adaptive shell 选择 / i18n 解析 / phone_demo 手势+back

## 文件大小

21 源文件 ~4153 行（单文件均 ≤2000 行）。
