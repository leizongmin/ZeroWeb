# Web Crypto 兼容 — crypto.getRandomValues / randomUUID / subtle

**版本**: v1.0
**日期**: 2026-10-08
**状态**: Active（已立项——无用户门控项；subtle 实现轮按切片自主推进）
**执行模式**: WPT 驱动（上游 WebCryptoAPI corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——加密面）

> **▶ 拆分动机（2026-10-08 html5test 缺口立项，用户「立项」）**：html5test 加密
> 节最大缺口；randomUUID/getRandomValues 已实现（js_dom_shim/part01b.js——含
> ≤65536 上下限与 TypedArray 类型限定语义），`crypto.subtle` 全系
> （digest/importKey/exportKey/encrypt/decrypt/wrapKey）缺失。
>
> **▶ 基线事实（2026-10-17 M1 盘点复核，修正立项时「subtle 面零」的过期判断）**：
> 随机源面已有（bridge crypto.rs + shim part01b）；subtle 面并非零——digest
> SHA-1/256/384/512（R2793）、HMAC sign/verify + importKey raw（R2955）、
> PBKDF2（R2956）、AES-GCM encrypt/decrypt（R2957）、HKDF（R2958）、exportKey raw
> + generateKey + deriveKey 均已落地（shim part01b + part02，宿主 callbacks 六个
> `__zw_crypto_subtle_*`）。**未实现**：RSA 家族（OAEP/PSS/RSASSA-PKCS1-v1_5）、
> EC 家族（ECDSA/ECDH/Ed25519/X25519）、AES-CBC/CTR/KW、jwk/pkcs8/spki 导入格式、
> wrapKey/unwrapKey。WPT corpus 基线尚未建立——已有实现未对齐 WPT 断言，
> 通过率未知；后续里程碑按 corpus 分域失败分类驱动，不是从零实现。
> `WebCryptoAPI/` corpus（pin 31597693）202 文件，入口 .any.js/.window.js 约 120，
> 子域 digest/sign_verify/import_export/generateKey/encrypt_decrypt/derive_bits_keys/
> serialization/wrapKey_unwrapKey/util 等。

## Mission

以 WPT WebCryptoAPI corpus 为验收标准，实现 subtle 全系并守住随机源既有语义。
排除：TLS/证书（zero-net 域）、secure-context 判定（security-hardening 已收口
遗产，消费之）、随机熵源质量（OS 熵源为前提，不重造）。

**实现纪律**：
1. **依赖选型先行**：Rust crypto 依赖盘点（既有依赖复用优先；新引入须 evidence/
   记录选型理由，参照 compression-compat 纪律）
2. **算子分片**：digest → importKey/exportKey → encrypt/decrypt（AES-GCM 系）→
   HMAC/RSASSA-PKCS1-v1_5/ECDSA 签名系，每片独立提交附 A/B
3. **安全边界不可让步**：密钥材料零落日志、extractable 语义严格执行（准则：
   安全措施不可简化）

## Done Criteria

- **DC-1** 导入 + 基线 + csv planned 行回填
- **DC-2** getRandomValues/randomUUID 既有语义守住（零回归）+ subtle 分片逐片
  对齐（digest 全格 → AES-GCM → HMAC/签名系），分级通过率不再下行两个连续轮次
- **DC-3** 回归测试账本纪律；make test 全绿、make reftest 零回归、product-smoke
  逐字节恒值

## 里程碑

M1 导入与基线（corpus + runner 通道 + csv 回填）→ M2 已实现面 WPT 对齐
（digest/HMAC/PBKDF2/HKDF/AES-GCM 逐域修失败；随机源零回归）→ M3 补面
（AES-CBC/CTR、wrapKey/unwrapKey、spki/pkcs8/jwk 格式）→ M4 签名系
（ECDSA/RSASSA-PKCS1-v1_5/RSA-PSS…）+ 收口归档。

> 原里程碑（M2 digest 从零实现 / M3 importKey+AES-GCM 从零实现）已被
> 2026-10-17 盘点修正——相应实现 R2793/R2955-R2958 立项前已存在，里程碑
> 改为「WPT 对齐 + 补面」。

## 依赖约束（run-rules §9）

与 security-hardening（编号 90，已收口）secure-contexts 遗产为消费关系；Cargo
依赖变更走 workspace 统一评审（perf-gate config_hash 敏感）。
