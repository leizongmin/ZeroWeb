# JS-in-Rust-string 测试断言：`{...}` 大括号与 `//` 行注释陷阱

**日期**：2026-08-09
**相关模块**：`crates/engine/src/js_dom_bridge_tests/part*.rs`（js_dom_bridge 测试，含大量内联 JS）
**踩坑轮次**：R3045、R3047、R3048（连续 3 次复现）

## 问题描述

`js_dom_bridge_tests` 模块的测试大量使用 Rust 字符串字面量内联 JS 代码（经 `\` 行续接拼成单行 JS），并在 `assert_eq!` 第三参写中文断言消息。两个高频陷阱导致测试**编译失败**或**运行逻辑被静默吞掉**，且报错信息不直观。

## 根因分析

### 陷阱 1：`assert_eq!` 消息字符串里的 `{...}` 被 Rust 当作格式占位符

```rust
assert_eq!(
    sandbox.execute("globalThis.__st3").unwrap().value,
    "80",
    "scrollTo({top:80}) → scrollTop=80（options 形式）"   // ❌ {top:80} 是格式占位符
);
```

`assert_eq!` 的第三参（失败消息）是 `format!` 格式串，`{...}` 被解析为命名/位置格式参数。`{top:80}` 触发 `error[E0425]: cannot find value 'top' in this scope`（或 `left`/`right`），报错指向断言消息而非真实逻辑，易误导。

**触发场景**：断言消息里写 JS 语法示例（`scrollTo({top:80})`、`new Request(url,{signal})`、`{block:'start'}` 等）。

### 陷阱 2：`\`-续接字符串里的 `//` 行注释吞掉后续整行

```rust
sandbox.execute(
    "...fetch(req).catch(...);\
     // abort reqOnly 的 signal → 不应触发\
     globalThis.__reqOnly.abort();\     // ❌ 被 // 注释掉
     globalThis.__afterReqAbort = globalThis.__override;\
     globalThis.__initOnly.abort();",
)?;
```

Rust 的 `\` 行续接把多行字面量拼成**单行 JS**。中间的 `// ...` 在 JS 里是行注释——但拼成单行后，`//` 注释掉**同一逻辑行后续全部语句**（`__reqOnly.abort()` / `__afterReqAbort = ...` / `__initOnly.abort()` 全部失效）。结果：变量未被赋值（读到 `undefined`），断言失败且看不出原因。

## 解决方案

**陷阱 1**：断言消息里的 `{` `}` 一律转义为 `{{` `}}`。
```rust
"scrollTo({{top:80}}) → scrollTop=80（options 形式）"   // ✅
```

**陷阱 2**：内联 JS 字符串中**禁用 `//` 行注释**。要加说明：
- 写在 Rust 侧 `//` 注释里（Rust 注释，不进 JS 串）；或
- 用 `/* ... */` 块注释（JS 块注释不吞行）；或
- 直接拆成多个 `sandbox.execute()` 调用，每个之间用 Rust 注释说明。

## 如何避免

1. **断言消息含 JS 示例时**：grep 检查 `assert_eq!` 第三参是否含未转义 `{`，统一 `{{`/`}}`。
2. **内联 JS 多语句**：绝不在 `\`-续接串里用 `//`；说明性文字放 Rust 注释。
3. 写完测试先 `cargo clippy -p zero-engine --all-targets` 快速捕编译错误（陷阱 1 立现），再跑 nextest（陷阱 2 表现为 `undefined` 断言失败）。
4. 报错 `cannot find value 'X' in this scope` 指向 `assert_eq!` 消息行 → 99% 是陷阱 1（转义大括号）。
5. 断言实际值为 `undefined`（预期非空）且 JS 含 `//` → 陷阱 2（删 `//`）。
