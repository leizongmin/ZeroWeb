---
date: 2026-11-03
modules: engine
---

# Hybrid PQ/T KEM combiner 用 CG 框架 C2PRI 定义（含 ek_T、不含 ct_PQ），不是 X-Wing 原式

## 问题描述

实现 draft-irtf-cfrg-concrete-hybrid-kems-03 三组合（MLKEM768-P256 /
MLKEM768-X25519 / MLKEM1024-P384）时，decapsulation KAT 对 P256 全不中：
keygen KAT（ek 逐字节）与 encap/decap roundtrip 均绿，唯独
`shared_secret` 与向量期望不一致。

初版 combiner 写成 X-Wing 形：
`SHA3-256(ss_PQ ‖ ss_T ‖ ct_PQ ‖ ct_T ‖ label)`——含 ct_PQ、缺 ek_T。

## 根因分析

三层证据链各自为真，导致误排方向：

1. **正文表述误导**：draft-03 §4 对 MLKEM768-X25519 写「identical to the
   X-Wing construction in [XWING-SPEC]」——X-Wing 原式确实是
   `SHA3-256(ss_MLKEM ‖ ss_T ‖ ct_MLKEM ‖ ct_T ‖ "\.//^\")`。
2. **但框架归属是 CG**：同一节开头声明三组合全部「using the CG
   framework」。combiner 精确定义不在本文，而在
   draft-irtf-cfrg-hybrid-kems-09 §5.1.3：

   ```
   C2PRICombiner(ss_PQ, ss_T, ct_T, ek_T, label) = KDF(concat(...))
   ```

   CG 框架（§5.5）Encaps/Decaps 均调它——**含接收方公钥 ek_T、不含
   ct_PQ**。X-Wing 原式与 C2PRI 是不同公式；「identical」指构造概念层
   （同组件同 KDF），不是组合器逐字节相同。
3. **互证陷阱**：15 种输入序变体 × 3 种哈希的离线枚举零命中，且
   ss_T、ct 切分、三组 hex 常量全部被独立通道证实正确——错误的公式
   形状根本不在枚举空间里（缺 ek_T 一段，任何换序都补不回来）。

修复用程序化解析（分页折行感知）把 HYBRID-KEMS-09 §5.1.3/§5.5 原式
取出，离线对 P256/X25519 两向量同时命中后改实现，三组合 KAT 全绿。

## 解决方案

- 框架名（QSF/CG/UG/UK…）优先于正文类比句：combiner 公式只从框架定义
  章节取，正文「identical to X」当线索不当依据。
- 组合器形状性错误（少一段/多一段）对换序枚举免疫：候选枚举零命中时，
  下一步应核对输入段集合本身（对照框架定义逐段点名），而不是扩大哈希
  或序变体空间。
- 规范跨文档引用（draft → framework draft）必须拉被引文档原文，凭记忆
  写跨文档公式与凭记忆写常量同险（同因复现
  [kmac-newx-raw-x](../../../learnings/bugs/2026-11/2026-11-03-kmac-newx-raw-x-vs-tuplehash-encode-string.md)）。
