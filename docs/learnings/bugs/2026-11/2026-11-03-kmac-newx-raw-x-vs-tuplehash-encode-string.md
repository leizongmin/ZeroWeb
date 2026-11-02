---
date: 2026-11-03
modules: zero-engine
---

# KMAC newX 拼接是裸 X——把 TupleHash 的逐元素 encode_string 套到 KMAC 上产出全错 MAC

## 问题

`crypto_subtle_kmac`（SP 800-185 KMAC128/256）对全部 NIST 向量不中：实现侧组装
`newX = bytepad(encode_string(K), rate) || encode_string(X) || right_encode(L)`，
产出恒错；且 RustCrypto cshake 0.2.1 crate 与独立纯 Python Keccak-f[1600] 参考
两套实现互证一致地产出同一个错值——「双实现一致」无法发现**同源规范误读**。

## 根因

SP 800-185 §4.3 定义 `newX = bytepad(encode_string(K), 168/136) || X ||
right_encode(L)`——**X 直接拼接，不加 encode_string 包装**。逐元素
`encode_string` 是同文档 TupleHash（§4.4）/ParallelHash 的编组语义，记忆串线时
串到了 KMAC。两处易混点叠加放大排查成本：

- encode_string(X) 产出的错 MAC 依然是合法 Keccak 输出（无结构异常可查）；
- 修复期同时补上 bytepad 零填充（这个是对的）与 encode_string(X)（这个是错的）
  双改动，单变量对照被打乱，错误修复被正确修复掩护。

## 解决

- 组装回归 §4.3 原式：`left_encode(rate) || left_encode(|K|bits) || K || 补零至
  rate 整数倍 || X || right_encode(L)`；N="KMAC"、S=customization 走 cSHAKE
  domain 参数（pad 0x04，≠ SHAKE 0x1F）。
- **仲裁对外部正典**：NIST SP 800-185 原文 + KMAC_samples.pdf 期望值。本机无
  pip 时用纯 Python Keccak-f[1600]（先对 hashlib SHAKE128/256 逐字节验证，再切
  0x04 domain）独立复现，6/6 向量逐字节闭合。
- 期望值**程序化从语料文件解析**（`signature: new Uint8Array([...])` 正则抽数），
  不手抄长 hex——手抄串静默笔误曾制造 S4/S6「前 16 字节一致后发散」的假案。

## 避免方式

实现 NIST 派生函数（cSHAKE/KMAC/TupleHash/ParallelHash）时逐字对照原文步骤，
不凭记忆写组装式；尤其区分 KMAC（裸 X）与 TupleHash（逐元素 encode_string）。
密码学语料值一律程序化取数；两套自研实现互证不能替代对官方向量的仲裁。
