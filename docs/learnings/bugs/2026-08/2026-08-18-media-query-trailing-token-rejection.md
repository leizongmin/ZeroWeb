---
date: 2026-08-18
modules: crates/css-parser/src/media_query.rs
---

# Media Query Trailing Token Rejection

## 问题描述

media query parser 解析完括号条件后，如果后面还有非空 token，会静默返回已解析条件。例如 `(min-width: 600px) garbage` 会被当成 `(min-width: 600px)`，`(min-width: 600px) andfoo (...)` 会被截断成第一个条件。

## 根因分析

条件循环只在 `remaining.starts_with('(')` 时继续解析；遇到不是 `(` 的剩余内容时直接退出循环并返回查询。连接词也使用裸 `starts_with("and")`，会把 `andfoo` 当成 `and` 前缀消费。

## 解决方案

每个条件后：

+ 空剩余表示查询结束。
+ 合法 `and` 继续解析下一条件。
+ 其他非空剩余直接返回 no-match。

循环结束后再次检查剩余内容，非空即拒绝。`and(` 无空白连接继续保留。

## 避免方式

解析分段语法时不要在局部成功后忽略尾部输入。消费循环结束时必须确认剩余 token 为空，除非调用方明确支持错误恢复。
