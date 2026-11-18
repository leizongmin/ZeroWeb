---
date: 2026-11-18
modules: crates/engine
---

# V8Sandbox 钉测 JSON.stringify 快照断言的 Debug 双层转义

## 问题描述

`js_dom_bridge_tests/part1xx_*_pins.rs` 钉测的通用判读形是
`sandbox.execute("JSON.stringify(globalThis.__mg)")` 后
`format!("{:?}", snap.value)` 再 `snap_str.contains(...)` 断言。当被断言
的 innerHTML 含属性引号（如 `contenteditable=""`）时，转义层级判断反复
出错：`\"` 与 `\\\"` 两形来回试错浪费编译轮（本次 ip I4 钉测首跑即因此
假红）。

## 根因分析

转义发生两次，对象不同：

1. **JSON.stringify**（JS 侧）：字符串值内部的 `"` 产生 `\"`（两字符：
   反斜杠＋引号），这是 JSON 字符串语法的一部分。
2. **Rust `{:?}` Debug**（Rust 侧）：把第 1 步结果整串当 Rust 字符串转义
   ——JSON 的普通 `"` 变 `\"`，JSON 的 `\` 变 `\\`。

两层叠加后，HTML 属性引号在 snap_str 中的最终形态是**三层**：
`contenteditable=\\\"false\\\"`（Debug 后实际字节为 `\\"` 反斜杠＋反斜杠
＋引号——JSON 转义出的 `\` 被 Debug 转成 `\\`，JSON 未转义的 `"` 被
Debug 转成 `\"`）。而 JSON 结构性引号（`"tag":` 的定界符）只有一层：
`\"tag\":`。

## 解决方案

- 断言模式一律用 raw 字符串 `r#"..."#` 书写，转义层次可见：
  - JSON 结构性引号（键名/字符串定界）→ `\"`（一层）。
  - HTML 属性引号（被 stringify 转义过一次的内容引号）→ `\\\"`（三层）。
- 首跑失败时不要凭直觉改转义层数：断言消息会把 snap_str 全文打出来，
  直接从失败输出复制期望串的精确字节（本次即靠 p-li 失败输出一次性修定
  全部 7 个断言）。
- 规避路径（可选）：钉测断言只用不含属性引号的 innerHTML 子串（如
  `<div>ab</div><div>cd</div>`），或断言值面板改走 `JSON.stringify`
  后在 JS 侧再做一次 base64，Rust 侧只比对 base64 串——零转义歧义，
  但可读性下降，仅在属性引号密集时考虑。
