# ZeroWeb Security (`zero-security`)

> 浏览器安全模型实现：CORS、CSP、同源策略

## 概述

`ZeroWeb Security` (`zero-security`) 为 ZeroWeb 提供核心安全策略基础设施，涵盖跨源资源共享（CORS）检查、内容安全策略（CSP）解析与资源加载控制、以及同源策略判断。作为渲染引擎的安全边界层，它在网络请求、资源加载等环节拦截不安全的跨源访问，确保浏览器行为符合 Web 安全规范。

## 主要功能

- **同源策略（Same-Origin Policy）** — 基于 scheme + host + port 的源解析与同源判断，支持默认端口归一化和安全上下文检测
- **CORS（跨源资源共享）** — 可配置的 CORS 策略检查，支持通配符源、白名单源、允许方法/请求头过滤、凭证模式、简单请求判断
- **CSP（内容安全策略）** — 从 HTTP 头解析 CSP 指令，检查资源加载权限，支持 `default-src` 回退、`'self'`、`'none'`、`*` 通配符、`*.domain` 通配域名、精确 URL 匹配、内联脚本/样式控制
- **统一错误类型** — 通过 `SecurityError` 枚举提供源解析、CORS、CSP 三类错误的统一处理

## 使用示例

```rust
use zero_security::{Origin, CorsPolicy, check_cors, ContentSecurityPolicy};

// 同源策略：解析并比较两个源
let origin_a = Origin::parse("https://example.com/page1").unwrap();
let origin_b = Origin::parse("https://example.com:443/page2").unwrap();
assert!(origin_a.is_same_origin(&origin_b));

// CORS：配置策略并检查跨源请求
let policy = CorsPolicy {
    allow_origins: vec!["https://trusted.com".to_string()],
    allow_methods: vec!["GET".to_string(), "POST".to_string()],
    allow_headers: vec!["X-Custom".to_string()],
    allow_credentials: true,
    max_age: Some(3600),
};
let request_origin = Origin::parse("https://trusted.com").unwrap();
let result = check_cors(&policy, &request_origin, "GET", &[]);
assert!(result.allowed);

// CSP：解析策略并检查资源加载权限
let csp = ContentSecurityPolicy::parse(
    "default-src 'self'; script-src https://cdn.example.com; style-src 'unsafe-inline'"
);
assert!(csp.is_resource_allowed("script", "https://cdn.example.com/app.js", None));
assert!(!csp.is_resource_allowed("script", "https://evil.com/bad.js", None));
assert!(csp.is_inline_style_allowed(None, None));
```
