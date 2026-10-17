# Web Crypto 兼容 — crypto.getRandomValues / randomUUID / subtle

**版本**: v1.4
**日期**: 2026-10-17
**状态**: Active——M4b 收口（EC 家族 ECDSA/ECDH P-256/384/521 +3,159P，
2026-10-17）；下一步 M5（OKP 签名系：Ed25519/X25519，含 cfrg_curves 域）
**执行模式**: WPT 驱动（上游 WebCryptoAPI corpus 为验收标尺）+ 实现与语义修齐
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——加密面）

> **▶ M4b EC 家族（2026-10-17）**：ECDSA/ECDH P-256/P-384/P-521 全链——
> importKey（spki/pkcs8/jwk/raw 含 SEC1 压缩点）、generateKey CryptoKeyPair
> （publicKey 恒 extractable + usages 类型拆分）、exportKey 四格式（raw=SEC1
> 未压缩点、jwk x/y/d 定宽零填充）、ECDSA sign/verify（RFC 6979 确定性 +
> IEEE P1363 r||s）、ECDH deriveBits/deriveKey（x 坐标 + length 位截断）。
> 依赖 p256/p384/p521 0.13（选型见 deps evidence）。ec_importKey/ecdh_bits/
> ecdh_keys/sign_verify ecdsa/generateKey EC 四文件/serialization ×2/
> derive_key_and_encrypt 全绿；derive 共享路径修齐（wrong-key
> InvalidAccessError、HKDF/PBKDF2 目标 deriveKey length null）。全量
> **33,028 = Pass 20,900（63.3%）+ Fail 12,127 + Timeout 1**（+3,159，零
> 回归）；全绿案 41→53。明细见 [M4b evidence](webcrypto-compat/evidence/2026-10-17-m4b-ec.md)。

> **▶ M4a RSA 家族（2026-10-17）**：RSA-OAEP/RSASSA-PKCS1-v1_5/RSA-PSS 全链——
> importKey spki/pkcs8/jwk、generateKey CryptoKeyPair（publicKey 恒 extractable +
> usages 类型拆分）、exportKey 三格式、OAEP encrypt/decrypt（label、空明文空返）、
> RSASSA/PSS sign/verify（确定性锚定、verify 坏签名 resolve false）、wrapKey/
> unwrapKey RSA-OAEP wrapper + spki/pkcs8 回导。依赖 rsa 0.9（RustCrypto，选型见
> deps evidence）。rsa_importKey 1,056 全绿、generateKey RSA 六文件全绿、
> rsa_oaep/rsa_pss/rsa_pkcs 全绿、wrapKey_unwrapKey 86→183 全绿。全量 **32,899 =
> Pass 17,741（53.9%）+ Fail 15,157 + Timeout 1**（+2,351，零回归）；全绿案 27→41。
> 明细见 [M4a evidence](webcrypto-compat/evidence/2026-10-17-m4a-rsa.md)。

> **▶ M3b wrapKey/unwrapKey AES-KW + jwk 格式（2026-10-17）**：对称密钥管理面
> 补齐——wrapKey/unwrapKey（AES-KW RFC 3394 + AES-GCM/CBC/CTR 封装路径）、
> jwk 导入/导出（AES 家族/HMAC，kty/use/key_ops/ext/alg 成员校验）、spki/pkcs8
> 对称 NotSupportedError、importKey/generateKey length 成员与 usages 错误次序
> （HmacKeyAlgorithm.hash 为对象）。wrapKey_unwrapKey 1→86P/25F（25F 全
> RSA-OAEP wrapper，M4 域）、symmetric_importKey 84→606 **全绿**、generateKey
> AES/HMAC 六文件 + serialization aes*/hmac 全绿；deriveKey 目标 length 透传
> 级联 pbkdf2 +1,836 / hkdf +768。全量 **32,827 = Pass 15,390（46.9%）+
> Fail 17,436 + Timeout 1**（M3a 10,321 → +5,069，零回归）；全绿案 13→27。
> 明细见 [M3b evidence](webcrypto-compat/evidence/2026-10-17-m3b-kw-jwk.md)。

> **▶ M3a AES-GCM tagLength/IV 矩阵（2026-10-17）**：tagLength ∈
> {32,64,96,104,112,120,128}、IV 12/32 字节、密钥 128/192/256 位全矩阵——
> 32/64 位 tag 走 detached encrypt 截断语义（SP 800-38D §5.2.1.2，decrypt 两次
> detached 调用重算全 tag 常时前缀比对）。aes_gcm 125→577、aes_gcm_256_iv
> 133→577 **双双全绿**（原 896F 簇全翻）；192 位密钥解锁级联
> symmetric_importKey +4。全量 **32,717 = Pass 10,321（31.5%）+ Fail 22,395 +
> Timeout 1**（M2b 9,421 → +900，零回归）；全绿案 11→13。明细见
> [M3a evidence](webcrypto-compat/evidence/2026-10-17-m3a-gcm-matrix.md)。

> **▶ M2b AES-CBC/CTR 补面（2026-10-17）**：RustCrypto cbc/ctr/aes（与 aes-gcm 同
> cipher 0.4 族）接入 encrypt/decrypt/generateKey/importKey/deriveKey；aes_cbc
> 61/61、aes_ctr 52/52 全绿；AES 家族解锁级联 generateKey +960 / derive +1,464。
> 全量 **32,717 = Pass 9,421（28.8%）+ Fail 23,295 + Timeout 1**（M2a 6,804 →
> +2,617，零回归）；全绿案 8→11。spec 错误语义修齐（iv ≠16 OperationError、
> key/算法不匹配 InvalidAccessError、CTR length 位宽校验）。明细见
> [M2b evidence](webcrypto-compat/evidence/2026-10-17-m2b-aes-block.md)。

> **▶ M2a 语义修齐（2026-10-17）**：M1 失败簇 1-3 全修（随机源 ×12、normalize ×4、
> digest ×16）+ 错误实例身份修复（subtle reject 经全局 DOMException——R384 同族，
> encrypt_decrypt +36 / sign_verify +8 额外收益）；原生 DOMException 补
> TypeMismatchError code 17。全量 32,717 = Pass 6,804（20.8%）+ Fail 25,912 +
> Timeout 1（M1 6,728 → +76，零回归）；全绿案 5→8。分域与改动明细见
> [M2a evidence](webcrypto-compat/evidence/2026-10-17-m2a-semantics.md)。

> **▶ M1 基线（2026-10-17）**：subtests 32,717 = Pass 6,728（20.6%）+ Fail 25,988 +
> Timeout 1；123 案执行（idlharness ×2 + secure-context sub.html 记账排除），5 案全绿。
> 分域与失败簇见 [M1 基线 evidence](webcrypto-compat/evidence/2026-10-17-m1-baseline.md)。
> M2 可执行清单：getRandomValues 大长度/Float/DataView 拒绝 ×12（DC-2 守护面）、
> normalize-algorithm-name U+212A ASCII 折叠 ×4、digest 空 algorithm/转移 buffer ×16；
> tentative XOF（cSHAKE/SHA3/K12）419F 挂账不修。

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
（RSA 三算法 M4a ✓ + EC 家族 M4b ✓）→ M5 OKP（Ed25519/X25519，cfrg_curves
域）→ 残留收口（pbkdf2/hkdf 0 length 细分、tentative 域挂账清点）+ 归档。

> 原里程碑（M2 digest 从零实现 / M3 importKey+AES-GCM 从零实现）已被
> 2026-10-17 盘点修正——相应实现 R2793/R2955-R2958 立项前已存在，里程碑
> 改为「WPT 对齐 + 补面」。

## 依赖约束（run-rules §9）

与 security-hardening（编号 90，已收口）secure-contexts 遗产为消费关系；Cargo
依赖变更走 workspace 统一评审（perf-gate config_hash 敏感）。
