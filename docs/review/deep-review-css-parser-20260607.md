# ZeroWeb 深度审查报告 — CSS 解析器 / WebView

> **摘要**
>
> **审查范围**：`crates/css-parser/`（Parser、Media Query、Values）、`crates/webview/`（WebView 主类型）
>
> **关键发现**：共发现 8 个问题（中 4 / 低 4）
>
> **最高优先级**：CSS `consume_declaration_block` 在最后声明无分号时吞噬 `}`，可能导致级联解析失败
>
> **验证状态**：已验证（2026-06-07）— 6 verified, 2 dismissed

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | css-parser/parser.rs、css-parser/media_query.rs、css-parser/values/、webview/webview.rs |
| **审查维度** | 实现缺陷、安全、性能 |
| **代码版本** | main 分支，commit f5eb85b |

---

## 问题清单

### 中优先级（Major）

#### CSS-01 [实现缺陷] consume_declaration_block 无分号结尾时吞噬 RBrace

- **位置**：`crates/css-parser/src/parser.rs:708-726`
- **置信度**：0.80
- **状态**：verified
- **描述**：`consume_declaration` 在 `RBrace` 处停止但不消耗它，随后外层循环的 `self.advance()` 无条件消耗该 `RBrace`。当下一个 token 是新规则开始时，循环再尝试 `consume_declaration` 失败后又 `advance()`，可能吞噬下一个规则的 token，导致级联解析失败。
- **触发条件**：CSS 规则 `div { color: red } p { ... }`——`p` 之前的空格和 `p` 可能被错误吞噬。
- **代码证据**：
  ```rust
  if let Some(decl) = self.consume_declaration() {
      declarations.push(decl);
  }
  self.advance(); // 无条件推进，可能吞噬 RBrace
  ```
- **建议修复**：仅当 `consume_declaration` 返回 `None` 且当前 token 非终止符时调用 `advance()`，或让 `consume_declaration` 消费终止分号。

---

#### CSS-02 [实现缺陷] parse_var 不处理 fallback 值中的嵌套括号

- **位置**：`crates/css-parser/src/values/parse_basic.rs:656-681`
- **置信度**：0.75
- **状态**：verified
- **描述**：`inner.find(',')` 定位第一个逗号，但该逗号可能在 fallback 值的嵌套括号内。如 `var(--color, rgb(255, 0, 0))` 会将 `name` 解析为 `--color`，但 `fallback` 截断为 `rgb(255`，丢弃 `0, 0))`。
- **触发条件**：CSS 属性 `color: var(--my-color, rgb(255, 128, 0))`。
- **建议修复**：使用括号平衡感知的逗号搜索，找到不在嵌套括号内的第一个逗号。

---

#### CSS-03 [实现缺陷] parse_size_condition 仅支持 <= 组合范围运算符

- **位置**：`crates/css-parser/src/parser.rs:1260-1270`
- **置信度**：0.70
- **状态**：dismissed
- **描述**：代码只检测 `<=` 模式，不支持 `<`、`>`、`>=` 等 CSS 规范定义的范围运算符。
- **dismiss 原因**：简单比较运算符（<, >, <=, >=）在 1305-1321 行另有处理。finding 仅查看了 1260-1270 行的范围语法部分，未看到完整的函数实现。如 `width >= 300px` 或 `200px < width < 500px` 无法正确解析。
- **建议修复**：扩展支持所有 CSS 范围运算符，或使用更结构化的解析方法。

---

#### CSS-04 [实现缺陷] media_query not/only 解析区分大小写

- **位置**：`crates/css-parser/src/media_query.rs:238-246`
- **置信度**：0.90
- **状态**：verified
- **描述**：`not`/`only` 前缀及媒体类型 `screen`/`print`/`all` 仅检查全小写和全大写，CSS 关键字应不区分大小写。`"Not "`、`"nOt "` 等变体无法识别。
- **建议修复**：将输入转为小写后再匹配。

---

### 低优先级（Minor）

#### CSS-05 [实现缺陷] 格式错误的 nth 表达式静默默认为 {a:0, b:0}

- **位置**：`crates/css-parser/src/parser.rs:493, 499`
- **置信度**：0.70
- **状态**：dismissed
- **描述**：`unwrap_or(0)` 将无效的 `a_part`/`b_part` 静默默认为 0，`nth-child(abc)` 被解析为 `{a:0, b:0}` 而非拒绝。
- **dismiss 原因**：对格式错误的输入默认 {a:0, b:0}（不匹配任何元素）是合理的 fail-safe 行为，与真实浏览器的容错处理一致。
- **建议修复**：返回 `Option` 或记录解析错误。

---

#### CSS-06 [实现缺陷] Quirks 模式颜色解析接受任意大 u32 未钳制

- **位置**：`crates/css-parser/src/values/color.rs:66-72`
- **置信度**：0.65
- **状态**：verified
- **描述**：`parse::<u32>()` 接受任意大值，浏览器 quirks 模式通常钳制到 `0xFFFFFF`。大值的高位被 `& 0xFF` 截断而非钳制。
- **建议修复**：提取 RGB 前钳制到 `0xFFFFFF`。

---

#### CSS-07 [实现缺陷] WebView remove_event_callback 使索引失效

- **位置**：`crates/webview/src/webview.rs:161-168`
- **置信度**：0.80
- **状态**：verified
- **描述**：`remove_event_callback` 使用 `Vec::remove(index)`，导致后续回调的索引偏移。若用户先注册 3 个回调得到索引 0/1/2，移除索引 0 后，原来的 1 变为 0、2 变为 1，但用户持有的索引仍指向错误回调。
- **建议修复**：使用 `Option` 槽位（`Vec<Option<...>>`）而非 `Vec::remove`，或返回稳定句柄。

---

#### CSS-08 [安全] WASM 错误消息直接插入 JavaScript 字符串（注入风险）

- **位置**：`crates/webview/src/webview.rs:609-613, 625-629`
- **置信度**：0.70
- **状态**：verified
- **描述**：WASM 编译/实例化错误消息直接通过 `format!(...)"{e}")` 插入 JS 代码字符串。若错误消息含引号或 `</script>`，可导致 JS 语法错误或 XSS。
- **触发条件**：恶意 WASM 字节码触发包含特殊字符的编译错误消息。
- **建议修复**：转义错误消息中的 JS 特殊字符（`'`、`\`、`</script>`）。

---

## 统计总览

| 维度 | 高 | 中 | 低 | 合计 |
|------|----|----|----|------|
| 实现缺陷 | 0 | 4 | 4 | 8 |
| **合计** | **0** | **4** | **4** | **8** |

## 修复建议优先级

| 优先级 | 问题 | 建议动作 | 预估改动量 |
|--------|------|---------|-----------|
| P1（本迭代） | CSS-01, CSS-02 | 修复声明块解析、var() 括号感知 | 各约 20-30 行 |
| P1（本迭代） | CSS-03, CSS-04 | 扩展范围运算符、小写化匹配 | 各约 15-30 行 |
| P2（后续跟进） | CSS-05~08 | 防御性编码、索引修复、转义 | 各约 5-20 行 |
