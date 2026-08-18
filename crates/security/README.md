# ZeroWeb Security (`zero-security`)

> 浏览器安全模型实现：同源策略、CORS、CSP、沙箱、站点隔离、COOP/COEP、HSTS、混合内容

## 概述

`ZeroWeb Security` (`zero-security`) 为 ZeroWeb 提供核心安全策略基础设施：同源策略判断、跨源资源共享（CORS）检查、内容安全策略（CSP）解析与资源加载控制、iframe 沙箱、站点隔离（site-isolation，经 `zero-psl` 计算 eTLD+1）、COOP/COEP、HSTS 预加载与混合内容阻止/升级、权限模型。作为渲染引擎的安全边界层，它在网络请求、资源加载等环节拦截不安全的跨源访问，确保浏览器行为符合 Web 安全规范，并统一收敛到 `SecurityContext` 门面供页面加载路径调用。

## 主要功能

- **同源策略（Same-Origin Policy）** — 基于 scheme + host + port 的源解析与同源判断，支持默认端口归一化和安全上下文检测
- **CORS（跨源资源共享）** — 可配置的 CORS 策略检查，支持通配符源、白名单源、允许方法/请求头过滤、凭证模式、简单请求与 preflight 判断
- **CSP（内容安全策略）** — 从 HTTP 头解析 CSP 指令，检查资源加载权限，支持 `default-src` 回退、`'self'`、`'none'`、`*` 通配符、`*.domain` 通配域名、精确 URL 匹配、内联脚本/样式控制、nonce/hash、`upgrade-insecure-requests`、`strict-dynamic`、`report-only`
- **iframe 沙箱** — `sandbox` 属性 token 解析（ASCII 大小写不敏感）与导航/弹窗/表单/脚本能力限制
- **站点隔离** — `SiteIsolationManager`：site-per-process 模型，基于 PSL 的真实 eTLD+1 进程边界判定，跨站 DOM 访问阻止
- **COOP / COEP** — 跨源开放者策略与跨源嵌入者策略的响应头解析与检查
- **HSTS 预加载** — 内置 40+ 预加载域名，支持运行时注册与升级决策
- **混合内容阻止 / 升级** — 检测与分级（blockable/upgradable），主动升级可升级请求
- **权限模型** — `PermissionManager`：11 种权限类型、3 种状态、按 origin 隔离存储
- **统一门面** — `SecurityContext` 把上述检查整合为资源加载检查管线，供 webview / renderer 调用
- **统一错误类型** — 通过 `SecurityError` 枚举提供源解析、CORS、CSP 等错误的统一处理

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
