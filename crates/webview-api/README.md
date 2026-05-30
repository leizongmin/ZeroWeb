# zero-webview-api

> 面向外部应用的稳定 WebView 嵌入接口

## 概述

`zero-webview-api` 是 ZeroBrowser 对外暴露的稳定 API 层，提供构建器模式创建 WebView、加载 HTML/URL、注入 CSS、渲染表面输出等核心能力。它封装了底层渲染管线（`zero-engine-core`、`zero-render-foundation`），为嵌入方提供简洁、类型安全的接口，是浏览器 shell 和第三方应用接入 ZeroBrowser 渲染能力的唯一入口。

## 主要功能

- **构建器模式** — 通过 `WebViewBuilder` 链式配置视口尺寸、透明背景、用户代理、初始 URL、开发者工具等参数
- **加载内容** — `load_html` 直接渲染 HTML 字符串（可附带 CSS），`load_url` 发起导航
- **渲染管线** — 自动完成 DOM 解析、样式计算、布局、渲染，返回图元和耗时统计
- **CSS 注入** — `inject_css` 向已加载页面追加样式并重新渲染
- **动态调整** — `resize` 运行时修改视口尺寸，自动重建渲染管线
- **状态查询** — 获取当前 URL、页面标题、加载状态、上次渲染结果
- **事件回调** — `WebViewEvent` 枚举覆盖加载开始/完成/失败、标题变更、URL 变更等事件

## 使用示例

```rust
use zero_webview_api::{WebViewBuilder, WebViewEvent};

fn main() {
    // 使用构建器创建 WebView
    let mut webview = WebViewBuilder::new()
        .width(1024)
        .height(768)
        .user_agent("MyApp/1.0")
        .devtools(true)
        .build();

    // 加载 HTML 并渲染
    let html = r#"<html><body><h1>Hello, ZeroBrowser!</h1></body></html>"#;
    let css = "h1 { color: blue; }";
    let result = webview.load_html(html, Some(css));
    println!("渲染耗时: {:.2}ms", result.timings.total_ms);

    // 注入额外 CSS 并重新渲染
    let result = webview.inject_css("h1 { font-size: 48px; }");
    println!("图元数量: {}", result.primitives.fills.len());

    // 通过 URL 导航
    webview.load_url("https://example.com");
    assert!(webview.is_loading());
    assert_eq!(webview.url(), Some("https://example.com"));

    // 调整视口尺寸
    webview.resize(1920, 1080);
}
```
