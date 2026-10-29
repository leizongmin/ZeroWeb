---
date: 2026-10-29
modules: engine
---

# rust-url 保留裸 `%` 不归一 `%25`——预编码输出必须源侧归一

## 问题

form-url-encoding-compat M2（`a.href` encoding-parse 预编码）首轮 href 通道
216,268P/211F，211 条失败全部集中在 `iso2022jp-encode-href.html`，断言消息
`Invalid code point NaN`。euc-jp/shift_jis/big5 等 43k 同构子测试全过，唯独
iso-2022-jp 挂。

## 根因

ISO-2022-JP 的 JIS 字节域是 0x21-0x7E，**字节 0x25（`%`）本身是合法编码产出**
（如 Ε U+0395 → `ESC $ B 0x26 0x25`）。special-query percent-encode set 不含
`%`，按 set 判定它应以字面量落 query——于是解析输入出现裸 `%`（`?%1B$B&%`）：

- Chrome 的 URL parser 对裸 `%` 归一为 `%25`（WPT 期望 `%1B%24%42%26%25` 的
  normalizeStr 桥依赖这一消费侧归一）；
- **rust-url 保留裸 `%` 原样**（validation error 但不重写）。下游 WPT
  `normalizeStr` 把尾部裸 `%` 解成 `parseInt("",16)=NaN` →
  `String.fromCodePoint(NaN)` 抛 `Invalid code point NaN`；
  若 `%` 在序列中间（如字节对 `0x25 0x41`）则 `%XA` 被误当转义整体解码，字节
  分布错位，同样判 FAIL。

euc-jp 等单/双字节编码字节日元（¥ → 0x5C）不产 `%` 字节，所以同构路径全绿——
失败面精确收敛于「编码产出含 0x25」的字符集（211 条）。

## 解决

预编码器侧把 `%`（0x25）并入 query set（`text_encoding.rs`
`encode_percent_bytes`）：任何编码产出中的 `%` 字节直接以 `%25` 落输出，下游
rust-url 看到的恒为合法转义序列。form set 本就编码 `%`（全量 set），行为不变。
修复后 iso2022jp-encode-href 211F→0，全量 encode 面 766,958P/0F/1T（唯一非 P
为 runner data: iframe 挂账案）。

## 模式与避免

- **预编码输出会被下游 URL parser 再消费**：set 判定不能只对着 spec 的
  percent-encode set 抄，还要核对自家 parser 的容错行为是否与 Chrome 一致。
  rust-url 与 Chrome 在裸 `%` 上分歧，WPT 期望又是按 Chrome 形态写的——以
  消费侧可观测输出对齐，而不是以 spec set 字面为准。
- 排查这类失败先看**断言消息形态**（`Invalid code point NaN` 直接指向
  normalizeStr 的 `%` 解码分支），再用同编码相邻码点（Γ 0x26 0x23 过、
  Ε 0x26 0x25 挂）圈定差异字节，避免盲目对照编码表。
