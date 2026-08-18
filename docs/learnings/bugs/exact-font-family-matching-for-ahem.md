# Exact Font Family Matching For Ahem

日期：2026-08-18

相关模块：`crates/layout-engine/src/table_types.rs`

## 问题描述

vertical table 的 `compute_col_min_content` 需要在 Ahem 字体下用 `font-size` 作为每字宽度；非 Ahem 字体使用 `0.6×font-size` 估算。旧代码用 `trim_matches('"').contains("Ahem")` 判断字体 family，`NotAhem` 或 `MyAhemFallback` 会被误判为 Ahem，导致列 min-content 过宽。

## 根因分析

字体 family 是一个名称列表，匹配 Ahem 时应逐项精确比较。substring 匹配把 family 名称里的普通文本片段当成字体身份，破坏了 CSS family name 的边界。

## 解决方案

table 侧统一使用 `font_family_is_ahem`：

+ 遍历每个 family。
+ 去掉外层引号。
+ 使用 `eq_ignore_ascii_case("Ahem")` 精确匹配。

回归测试锁定两个分支：`NotAhem` 的 4 字符 word min-content 为 `24px`，带引号 `"Ahem"` 仍为 `40px`。

## 避免方式

处理 CSS family name 时不要用 substring 判断字体身份。优先按 family list 的单项边界做精确匹配；需要兼容序列化引号时只剥离外层引号，不改写名称内部内容。
