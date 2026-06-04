# ZeroWebView Demo (`zero-webview-demo`)

> WebView 嵌入示例应用

## 概述

`ZeroWebViewDemo` 是 ZeroWeb WebView 嵌入 API 的演示应用，展示如何在自定义应用中嵌入 `zero-webview` 来加载和渲染网页内容。

## 运行

```bash
cargo run --bin zero-webview-demo
```

## 功能演示

- 创建 WebView 实例（通过 `WebViewBuilder` 配置）
- 加载 URL 并渲染 HTML 页面
- 执行 JavaScript 脚本
- 处理事件回调
- 展示 GPU 渲染输出到窗口

## 依赖

```
zero-webview-demo
├── zero-webview           — WebView 嵌入 API
├── zero-host-runtime      — 窗口和事件循环
└── zero-render-foundation — GPU 渲染基础设施
```

## 嵌入示例代码

```rust
use zero_webview::{WebViewBuilder, WebViewEvent};
use zero_host_runtime::WindowBuilder;

// 创建 WebView
let mut webview = WebViewBuilder::new()
    .viewport_size(800, 600)
    .build();

// 加载页面
webview.load_html("<html><body><h1>Hello ZeroWeb</h1></body></html>", "");

// 执行脚本
let result = webview.execute_script("1 + 1").unwrap();

// 注册事件回调
webview.on_event(|event| {
    match event {
        WebViewEvent::LoadComplete => println!("页面加载完成"),
        WebViewEvent::TitleChanged(title) => println!("标题: {}", title),
        _ => {}
    }
});
```
