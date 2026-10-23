---
date: 2026-10-23
modules: css-parser, engine
---

# CSS token 序列化必须转义 string 值——裸串化使含引号声明整条丢失

## 问题描述

`quotes:"\0022" "\0022" ...`（引号字形本身用转义写出的合法声明）在 ZeroWeb 中整条丢失：parse_quotes 返 None → quotes=Auto。影响所有值内含 `"` 或 `\` 的 string token（quotes、content、font-family 带引号形态等）。

## 根因

tokenizer 按 css-syntax-3 消费阶段解码 `\0022` → `"` 是正确的（css-syntax #consume-escaped-code-point）；但声明值重建走 `Token::String` 的 `Display`，实现为 `write!(f, "\"{}\"", s)` 裸拼原值——**消费态解码与生产态转义不对称**。解码后的 `"` 回填进字符串字面量边界，重建值成为 `""" """` 畸形串，下游属性解析器把 string 边界读劈，整条声明解析失败。css-syntax-3 #serialize-a-string 明确要求序列化 string token 时转义值内的 `"` 与 `\`；同文件 `Url` token 早有 `escape_css_string_for_url` 做同样的事，`String` 漏了对应实现。

## 解决方案

新增 `escape_css_string`（转义 `"`/`\`，前缀 `\`），`Token::String` Display 委托之；`escape_css_string_for_url` 同改委托，消除两份手写转义。判别特征：**属性值「解码后看起来畸形」（连续引号、意外截断）而源码合法时，先查 token 序列化路径是否漏转义，再查属性解析器**。修复落点（R5237）：`crates/css-parser/src/tokenizer.rs`。

## 避免方式

任何「token → 字符串 → 再解析」的往返（decl.value 重建、序列化对比、getComputedStyle 输出）都必须过 serialize-a-string 口径；新增 token 类型的 Display 时对照 css-syntax-3 序列化章节逐类型核对。
