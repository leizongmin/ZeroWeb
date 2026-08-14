# ZeroWeb Network (`zero-net`)

> 基于 reqwest 的 HTTP/HTTPS 网络栈，提供 URL 解析、HTTP 客户端、导航历史和 Cookie 管理功能。

## 概述

`ZeroWeb Network` (`zero-net`) 是 ZeroWeb 的网络层，封装了 HTTP/HTTPS 请求的完整生命周期。它负责 URL 解析与同源判断、同步 HTTP 请求发送（支持重定向和超时控制）、浏览器风格的导航历史栈（前进/后退/替换），以及 Cookie 的解析、存储和按域名/路径匹配。

## 协议支持

- **HTTP/1.1** — 基础传输，始终可用。
- **HTTP/2** — 默认启用（经 ALPN 协商）；设环境变量 `ZERO_HTTP2=0`（或 `false`）可退回 HTTP/1.1。
- **HTTP/3（QUIC）** — **明确不支持**。不实现、不引入 QUIC/HTTP-3 依赖，也不支持 HTTP/3 优先级帧（RFC 9218）；后续如需支持须另立设计（见 `docs/specs/network-loading-p2-http2-rfc.md`）。

## 主要功能

- **URL 解析** — 基于 `url` crate，解析 scheme、host、port、path、query、fragment、认证信息，支持同源判断和安全性检查
- **HTTP 客户端** — 封装 reqwest blocking 客户端，支持 GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS 方法，可配置超时时间和最大重定向次数
- **请求与响应类型** — `HttpRequest` 支持链式添加请求头，`HttpResponse` 提供状态码判断、Content-Type 获取和 UTF-8 文本解码
- **导航历史** — 浏览器风格的双向历史栈，支持 navigate/go_back/go_forward/replace_current，超出容量时自动淘汰最早条目
- **Cookie 管理** — 解析 Set-Cookie 头，支持 Secure、HttpOnly、SameSite、Domain、Path、Expires/Max-Age 属性，按域名（含子域名）和路径匹配

## 使用示例

```rust
use zero_net::{HttpClient, parse_url, CookieStore, NavigationHistory};

// 发送 HTTP GET 请求
let client = HttpClient::new();
let response = client.get("https://example.com")?;
println!("状态码: {}", response.status_code);
println!("响应体: {}", response.text()?);

// 解析 URL 并判断同源
let url_a = parse_url("https://example.com/page1")?;
let url_b = parse_url("https://example.com/page2")?;
assert!(url_a.is_same_origin(&url_b));

// Cookie 解析与匹配
let mut store = CookieStore::new();
let cookie = CookieStore::parse_set_cookie("session=abc123; Domain=example.com; Path=/; HttpOnly")?;
store.add(cookie);
let header = store.cookie_header(&parse_url("https://example.com/")?);
assert_eq!(header, "session=abc123");

// 导航历史
let mut nav = NavigationHistory::new(50);
nav.navigate("https://example.com", Some("首页".into()));
nav.navigate("https://example.com/about", Some("关于".into()));
nav.go_back();
assert_eq!(nav.current().unwrap().url, "https://example.com");
```
