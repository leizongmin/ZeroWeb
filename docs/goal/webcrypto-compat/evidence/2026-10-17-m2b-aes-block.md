# M2b AES-CBC/CTR 补面（2026-10-17）

crypto.subtle 新增 AES-CBC（PKCS#7）/AES-CTR（16 字节大端计数器）全链：
encrypt/decrypt/generateKey/importKey/exportKey/deriveKey 目标。AES 家族解锁
级联到 generateKey 与 derive 域——全量 **+2,617P**（本仓 webcrypto 单片最大增幅）。

## 改动

1. **依赖**：workspace 增 RustCrypto `aes` 0.8 + `cbc` 0.1 + `ctr` 0.9——与既有
   `aes-gcm` 0.10 同构建于 cipher 0.4.4，零版本分叉；选型与候选对比见
   [deps 选型](2026-10-17-m2b-deps.md)。
2. **host** js_dom_bridge/crypto.rs：`crypto_subtle_aes_block(kind, mode, key, iv,
   data)`——cbc::Encryptor/Decryptor + Pkcs7（填充校验在 crate 内，不经手写 unpad）、
   ctr::Ctr128BE（加解密同路径）；128/192/256 位密钥；错误返空串（shim 映射
   OperationError）。callbacks.rs 注册 `__zw_crypto_subtle_aes_cbc`/`_aes_ctr`。
3. **shim**：`_zw_normalizeImportAlgorithm` 扩 AES-CBC/CTR；importKey 密钥长度
   16/24/32（GCM 维持 16/32——aes-gcm crate 无 Aes192Gcm，同 R2957）；allowedUsages
   扩 AES 家族；encrypt/decrypt 三算法路由；generateKey AES 家族统一分支
   （length ∈ {128,192,256} 缺省 256，越界 OperationError——AES-GCM 原忽略 length
   的偏差一并修正）；`_zw_keyLengthBits` 尊重显式 length（deriveKey AES 目标）。
4. **spec 错误语义**（WPT aes.js 断言驱动）：iv/counter ≠16 → OperationError
   （非 DataError）；key/算法不匹配 → InvalidAccessError（原 NotSupportedError，
   AES-GCM 同享此修）；CTR length 成员（计数位宽）0/>128 → OperationError
   （位宽 <128 回绕递增未实现，挂 FIXME——WPT fixtures 未覆盖）；CTR 空明文
   短路空输出（WPT「transferred plaintext」组断言空结果）。
5. **测试**：引擎 `test_crypto_subtle_aes_cbc_ctr_m2b`——NIST SP 800-38A 向量锚定
   （CBC-AES128 F.2.1 / CBC-AES256 F.2.5 / CTR-AES128 F.5.1 首块逐字节比对）+ 192 位
   round-trip + 坏 padding/iv/counter/length/不匹配错误路径；R2955 反例 'AES-CBC'
   →'RSA-OAEP'（原算法已实现，前置失效）。

## 验证

- 定向：`aes_cbc.https.any.js` **61/61 全绿**、`aes_ctr.https.any.js` **52/52 全绿**；
  AES-GCM 两文件 97+105→125+133（InvalidAccessError 修复家族收益 +56）。
- 全量 123 案：**32,717 = Pass 9,421（28.8%）+ Fail 23,295 + Timeout 1**；
  M2a 6,804 → **+2,617**，全绿案 8→11，各域零回归。
- 分域：generateKey 1,132→**2,092**（12.5%→23.1%）、derive_bits_keys 5,179→
  **6,643**（41.9%→53.8%）、encrypt_decrypt 206→**373**、import_export 57→**81**、
  (top) 47→49；digest/sign_verify/serialization/wrapKey 持平。
- `cargo test -p zero-engine` 3081/3081；fmt/clippy 零告警（修掉 6 个
  unnecessary_mut_passed）。
- 证据：`-summary.json`、`-full.json.gz`。

## 剩余

- AES-GCM tagLength 仅 128、IV 仅 96（spec 支持 32-128 tag / 任意 IV 长度）——
  aes_gcm 两文件余 896F 中的主要可修簇，挂 M3 评估。
- AES-KW（wrapKey/unwrapKey）、RSA/EC 家族 → M3/M4。
- tentative 域（OCB/ChaCha20-Poly1305 305F）挂账不修。
