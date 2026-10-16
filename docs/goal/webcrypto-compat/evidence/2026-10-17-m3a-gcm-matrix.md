# M3a AES-GCM tagLength/IV 矩阵（2026-10-17）

crypto.subtle AES-GCM 从「仅 128 位 tag + 96 位 IV」扩到 spec 合法矩阵：
tagLength ∈ {32,64,96,104,112,120,128}、IV 12/32 字节、密钥 128/192/256 位。
`aes_gcm` 与 `aes_gcm_256_iv` 两文件 **577+577 全绿**（此前 125+133），全量
**+900P**（9,421→10,321，28.8%→31.5%），全绿案 11→13。

## 改动

1. **host** js_dom_bridge/crypto.rs：`crypto_subtle_aes_gcm` 增 `tag_len_bits`
   参数——96-128 位走 crate `Aead`（TagSize 泛型 U12..U16）；32/64 位 crate 不
   密封（`TagSize` trait 只实现于 U12..U16），按 NIST SP 800-38D §5.2.1.2 截断
   语义手做 `truncated_gcm`：encrypt 走 `encrypt_in_place_detached` 取 128 位
   tag 前 N 字节；decrypt 用 detached encrypt 两次（CTR 对合变换：第一次 ct→pt
   弃 tag，第二次 pt→ct 的 tag 恰为期望全 128 位 tag）后**定长无早退**逐字节
   异或比对前缀（不引 `subtle` 依赖）。
2. **密钥 192 位解锁**：aes-gcm 0.10 的 `AesGcm<Aes192,…>` 泛型本可编译，此前
   R2957 仅写死 128/256——本片按密钥字节长三分派，importKey 校验同步简化
   （16/24/32 一致），`symmetric_importKey` +4P 级联。
3. **shim** part01b.js：`_zw_aesGcmCall` 校验 IV ∈ {12,32} 字节与 tagLength
   合法集（越界 → OperationError，spec 步骤「invalid tagLength」）；尾参传
   tagLength 给 host。part02.js encrypt/decrypt GCM 路由带 `tagLength`（缺省
   128）。
4. **测试**：`test_crypto_subtle_aes_cbc_ctr_m2b` 增截断路径回归（32/64 位 tag
   + 12 字节 IV round-trip——nonce 泛型按 IV 长度分派，急切构造曾 panic）+
   GCM 192 位密钥 256 位 IV 96 位 tag round-trip；三处过期断言修正（GCM 密文
   长度 20→22、192 位 key 反例、tagLength 96 反例→80）。

## 验证

- 定向：`aes_gcm.https.any.js` 125→**577 全绿**、`aes_gcm_256_iv.https.any.js`
  133→**577 全绿**（原 896F 簇全翻：876 IV 长度 + 20 非法 tagLength 分类）。
- 全量 123 案：**32,717 = Pass 10,321（31.5%）+ Fail 22,395 + Timeout 1**；
  M2b 9,421 → **+900**，全绿案 11→13，各域零回归（123 文件逐一对比对账）。
- `cargo test -p zero-engine` 3081/3081；fmt/clippy 零告警（修 6 个
  needless_borrow）。
- 证据：`-summary.json`、`-full.json.gz`。

## 剩余

- AES-KW（wrapKey/unwrapKey）、spki/pkcs8/jwk 导入格式 → M3b。
- RSA/EC 签名系 → M4。
- tentative 域（OCB/ChaCha20-Poly1305 305F）挂账不修。
