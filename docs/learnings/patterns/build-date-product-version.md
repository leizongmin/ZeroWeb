# 使用构建日期生成产品版本

日期：2026-08-09

相关模块：`zero-product-version`、`zero-net`、`zero-engine`、`zero-browser`、打包脚本

## 问题描述

Cargo crate 版本用于依赖解析和 `Cargo.lock`，不适合在每次构建时动态修改；但浏览器产品需要让 User-Agent、运行时协议、About 页面和安装包元数据自动反映构建日期。

## 解决方案

+ 保持 workspace crate 版本稳定，单独生成格式为 `YY.M.D` 的产品构建版本。
+ 默认使用构建机本地日期，使用户看到的“今天”与本地日历一致。
+ 设置 `SOURCE_DATE_EPOCH` 时改用该时间戳的 UTC 日期，保证发布产物可复现。
+ 网络和 JavaScript User-Agent、UA Client Hints、Headless/WebDriver、About/Settings 页面、Windows PE resource 及各平台安装包统一读取同一版本。
+ Cargo 构建脚本监控一个不存在的标记路径，使日期在后续构建中重新计算，而不是永久复用首次构建缓存。

## 注意事项

产品构建版本不能替代 crate 的语义化版本。前者标识产物日期，后者仍负责 Rust 依赖兼容性和发布约束。
