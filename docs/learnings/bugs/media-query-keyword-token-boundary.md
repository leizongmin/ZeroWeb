# Media Query Keyword Token Boundary

日期：2026-08-18

相关模块：`crates/css-parser/src/media_query.rs`

## 问题描述

media query parser 用 `starts_with("screen"|"print"|"all")` 识别 media type，并用 `starts_with("and")` 识别连接词。`screenand (...)`、`screen andfoo (...)`、`printand (...)`、`alland (...)` 会被误当成合法 media type + `and`。

## 根因分析

CSS keyword 必须按 token 边界匹配。裸前缀匹配会把一个更长 ident 拆成关键字前缀和剩余内容，导致非法 media query 被接受。

## 解决方案

新增两个边界 helper：

+ media type 后只能结束或接空白。
+ `and` 后只能接空白或 `(`。

`screen and(min-width: 600px)` 继续保持兼容，因为现有 parser 支持 `and(` 这种无空白写法。

## 避免方式

解析 CSS keyword 时不要只用 `starts_with(keyword)`。先确认关键字后面的 token 边界，再消费后续参数或组合条件。
