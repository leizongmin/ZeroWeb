---
date: 2026-08-09
modules: zero-product-version, zero-net, zero-engine, zero-browser, 打包脚本
---

# 使用构建日期生成产品版本

## 问题描述

Cargo crate 版本用于依赖解析和 `Cargo.lock`，不适合在每次构建时动态修改；但浏览器产品需要让 User-Agent、运行时协议、About 页面和安装包元数据自动反映构建日期。

## 解决方案

+ 保持 workspace crate 版本稳定，单独生成格式为 `YY.M.D` 的产品构建版本。
+ 默认使用构建机本地日期，使用户看到的“今天”与本地日历一致。
+ 设置 `SOURCE_DATE_EPOCH` 时改用该时间戳的 UTC 日期，保证发布产物可复现。
+ 网络和 JavaScript User-Agent、UA Client Hints、Headless/WebDriver、About/Settings 页面、Windows PE resource 及各平台安装包统一读取同一版本。
+ Cargo 构建脚本在 crate 或显式版本环境变化时重新计算日期；普通增量构建会复用已生成的日期版本。

## 注意事项

产品构建版本不能替代 crate 的语义化版本。前者标识产物日期，后者仍负责 Rust 依赖兼容性和发布约束。

跨午夜继续使用同一 `target` 时，Cargo 不会因系统日期变化自动重跑 build script，内嵌版本可能仍是前一天。若 `embedded_version_uses_short_date_format` 出现昨日/今日不一致，先执行 `cargo clean -p zero-product-version`，再重跑测试；只清理该 crate 的生成产物，不修改源码或放宽断言。
