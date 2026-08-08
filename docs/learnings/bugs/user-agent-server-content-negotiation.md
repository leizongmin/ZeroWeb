# User-Agent 导致服务端返回降级页面

日期：2026-08-09

相关模块：`zero-net`、`zero-engine`、浏览器导航

## 问题描述

ZeroBrowser 使用 `ZeroWeb/1.0` 访问 `https://www.baidu.com/` 时显示空白，但访问 HTTP 地址可以正常显示。Chrome 访问 HTTP 地址则会先跳转到 HTTPS，再显示完整页面。

## 根因分析

服务端会根据 `User-Agent` 返回不同内容：

+ `ZeroWeb/1.0` 请求 HTTPS 时只得到一个 227 字节的降级页面。页面正文为空，仅通过 `location.replace()` 尝试跳回 HTTP。
+ Chrome 兼容 `User-Agent` 请求 HTTP 时得到 302 HTTPS 重定向，请求 HTTPS 时得到完整首页。
+ ZeroWeb 的 `location.replace()` 当前只更新进程内 URL 和 history，不触发宿主重新抓取文档，因此降级页面无法完成跳转并表现为空白。

HTTP 状态码为 200 不代表拿到了真实页面。排查站点兼容问题时，应同时比较响应体、响应头和不同 `User-Agent` 下的行为。

## 解决方案

+ 默认 HTTP 客户端使用带平台信息的 Chrome 兼容 `User-Agent`，避免被常见站点识别为不支持 HTTPS 的非浏览器客户端；末尾保留 `ZeroWeb/<version>` 产品标识，其中版本通过 `env!("CARGO_PKG_VERSION")` 读取 `Cargo.toml`。
+ 使用本地 TCP 服务端回归测试实际请求头，防止后续退回产品自定义短 UA。
+ 后续仍需把 `location.assign()`、`location.replace()` 和 URL setter 接到宿主真实导航；兼容 UA 只能解决服务端错误分流，不能替代完整导航语义。
