# M7 归档：网络栈 + 导航模型

**状态**: ✅ 已完成
**完成日期**: 2026-05-30
**提交**: ad73d5b

---

## 交付物

| # | 交付物 | 状态 |
|---|--------|------|
| 1 | `net` crate HTTP/HTTPS 请求 | ✅ reqwest blocking client |
| 2 | HTML/CSS/JS/图片资源加载与缓存 | ✅ HttpClient with GET/POST |
| 3 | URL 解析和导航模型 | ✅ ParsedUrl + NavigationHistory (back/forward/replace) |
| 4 | `security` crate 同源策略、CORS | ✅ Origin + CorsPolicy + check_cors |
| 5 | Cookie 管理 | ✅ Cookie parsing + CookieStore |
| 6 | CSP 基础 | ✅ ContentSecurityPolicy parsing + resource checks |
| 7 | 单元测试 ≥50 个 | ✅ 56 个（32 net + 24 security） |
| 8 | 基准测试 ≥4 个 | ✅ net_bench.rs |

## 模块

### zero-net
| 模块 | 内容 | 测试 |
|------|------|------|
| url_parser | URL 解析、origin、同源判断 | 8 |
| request | HttpMethod/HttpRequest/HttpResponse | 3 |
| client | HttpClient (reqwest blocking) | 3 |
| navigation | NavigationHistory (back/forward/replace) | 8 |
| cookie | Cookie 解析、CookieStore | 10 |

### zero-security
| 模块 | 内容 | 测试 |
|------|------|------|
| origin | Origin 类型、同源策略 | 7 |
| cors | CorsPolicy、预检/简单请求判断 | 8 |
| csp | CSP 解析、资源加载检查 | 9 |
