---
date: 2026-08-18
modules: crates/css-parser/src/values/parse_misc.rs, crates/css-parser/src/parser/at_rules.rs
---

# Font Style Oblique Token Boundary

## 问题描述

`font-style` 普通属性和 `@font-face font-style` 描述符用 `starts_with("oblique")` 识别 oblique。这样会把 `obliquex`、`oblique-angle` 这类非法 ident 当成 `oblique` 前缀处理，破坏 CSS token 边界。

## 根因分析

CSS keyword 不能用裸字符串前缀判断。`oblique` 可以单独出现，也可以后接空白角度或函数式角度写法；但后接普通 ident 字符或 `-` 时，整个 token 是另一个 ident，不应接受。

## 解决方案

收紧 oblique 判断边界：

+ `oblique` 后结束。
+ `oblique` 后接空白。
+ `oblique` 后接 `(`。

当前导出的 `parse_misc.rs`、旧 `parse_basic.rs` 副本和 `@font-face` 解析都使用同一边界规则。

## 避免方式

解析 CSS keyword 时先确认 token 边界，再处理可选尾部参数。不要用 `starts_with(keyword)` 直接接受带参数的语法。
