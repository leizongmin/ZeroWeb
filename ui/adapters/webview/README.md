# zero-ui-adapter-webview

WebView 高级自定义组件适配器。把 `zero-webview` 的输出（RenderPrimitives/Texture/SceneNode）合成为 UI scene 的 `ExternalSurface` marker。UI SDK **只计算 WebView 外部矩形，不把网页 DOM 节点映射为 UI widgets**（DC-3）。

## 架构位置

```
ui/adapters/webview  ←── browser-ui/chrome（PageViewportFrame）
     │
     ├── zero-ui-core（Widget trait / UiEvent）
     └── zero-webview（嵌入式浏览器 API）
```

## 核心类型

| 类型 | 说明 |
|------|------|
| `WebViewWidget` | 完整 `impl Widget`：layout 填充分配的外部矩形 / paint 记录 `ExternalSurface` marker / event 处理 Scroll 更新度量 |
| `WebViewLayoutInput` | 布局输入（viewport rect / scale / theme） |
| `WebViewPaintOutput` | 绘制输出（scene node / external surface id） |
| `WebviewBackend` trait | backends 将 webview 光栅纹理注册为 surface id |
| `apply_scroll_command(scroll)` | 把 `ScrollCommand` 应用到 WebView 的 scroll offset |

## 设计约束

1. UI SDK 不映射 DOM（DC-3 架构边界）
2. 页面内容尺寸/scroll offset 由 WebView 管理（DC-4 滚动语义）
3. WebViewWidget 外部矩形由 host 布局计算

## 依赖

- `zero-ui-core` / `zero-webview`
- dev-dep：`zero-ui-render` / `zero-ui-runtime` / `zero-ui-widgets`
- 此 crate 是**唯一允许**依赖 `zero-webview` 的通用 UI crate（DC-1）

## 测试

- `cargo test -p zero-ui-adapter-webview` — 8 测
- 覆盖：paint→ExternalSurface marker / scroll 事件钳制到 max / host 注册→layout→paint→Scene 含 ExternalSurface
