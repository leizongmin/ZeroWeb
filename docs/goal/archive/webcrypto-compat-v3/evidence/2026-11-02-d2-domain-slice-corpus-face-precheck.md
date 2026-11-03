# D1 前置：域切片语料面预检（2026-11-02，scratch 探针零仓库触面）

> [MSRV 编译预检](2026-11-02-dc2-msrv-compile-precheck.md) 的硬化补充：编译通过
> 只证明「教科书调用面」可编，不证明「语料实际参数面」可算。本轮对 D1 建议
> 值四个先行域（argon2 / AES-OCB / ChaCha20-Poly1305 / XOF）用 `/tmp` scratch
> 探针 crate 逐域复现 **WPT 语料真实向量**，全部在 rustc 1.85.0 下逐字节通过。
> 探针为一次性验证产物，不入库（源码位于 `/tmp/zw-dc2-corpus-face*`，Cargo.toml
> 钉版即本文结论）。

方法：从 `tests/wpt-runner/wpt-data/WebCryptoAPI/` 语料 JS 提取向量生成 Rust
探针（生成器 `/tmp/zw-gen-{ocb,chacha,xof}-probe.py`），`RUSTUP_TOOLCHAIN=
1.85.0-x86_64-unknown-linux-gnu` + test-guard（`--per-proc-mem 4 --total-mem 8
--time-limit 600`）编译运行。argon2 探针上一轮已过（RFC 9106 三向量），本轮
在 1.85.0 下复跑确认（缓存命中重放，编译为既遂 1.85 构建）。

## 逐域结论

### argon2 0.6.0（维持）

`Argon2::new_with_secret` + `ParamsBuilder`（m/t/p_cost、`.data(AssociatedData)`）
+ `hash_password_into` 在 1.85.0 逐字节复现语料 RFC 9106 Argon2d/i/id 三向量。
**建议值不变。**

### AES-OCB：ocb3 0.2.0 可用，**版本组合修正**

- **MSRV 坑（编译预检未暴露，本轮实锤）**：ocb3 0.2.0 自身 `rust-version = 1.85`
  合规，但依赖树踩两处传递升级——`aes` 解析到 **0.9.3（需 rustc 1.89）**、
  `ctutils` 解析到 **0.4.7（需 rustc 1.87）**（0.4.3–0.4.7 全线 1.87，
  **0.4.2 为 1.85 兼容顶**）。切片落地时需钉版：`aes = "=0.9.2"` +
  `ctutils = "=0.4.2"`（ctutils 为传递依赖，须在引入 crate 显式声明防
  `cargo update` 静默升版破 MSRV）。
- **语料面**：`Ocb3<Aes128, U15, TagSize>` 三档 tagLength（U8/U12/U16 ↔
  64/96/128 bit）encrypt/decrypt 与语料 `aes_ocb_vectors.js` 逐字节相等
  （keyLength=128、iv=15B、479B 明文、52B AAD）。
- **关键 API 事实（优于 GCM 先例）**：语料 iv=15B 需显式 `NonceSize = U15`
  （crate 默认 U12 不覆盖，`NonceSizes` 约束 6..=15）；**TagSize 是类型参数
  （1..=16B 原生表达截断 tag）**——`decrypt_inout_detached` 对 64-bit 截断
  tag 直接常时校验（crate 内 `ctutils::CtEq`），无需仓库 `truncated_gcm`
  的全 tag 重算 + 前缀比对模式；detached API（`encrypt_inout_detached` /
  `decrypt_inout_detached`，aead 0.6 `AeadInOut` 非 deprecated 面）在位。

### ChaCha20-Poly1305：chacha20poly1305 0.11.0 可用，**同一 ctutils 坑**

依赖树经 poly1305 链同样解析到 ctutils 0.4.7，需同钉 `ctutils = "=0.4.2"`。
语料 9 组（plaintext empty/short/long × additionalData empty/short/medium，
12B iv、128-bit 全 tag）encrypt/decrypt 逐字节通过；wrong AAD/IV →
`OperationError` 由 crate 校验失败路径天然承接。**默认面即语料面，建议值
0.11.0 维持（+钉版）。**

### XOF 五连：k12 0.5.1 + turboshake 0.7.1 + cshake 0.2.1 编译干净，**名称更正**

- **deps evidence 名称更正**：KangarooTwelve 官方 crate 名为 **`k12`**
  （`kangarootwelve` 另有其物，最高 0.1.3）——建议值 k12 0.5.1 指 `k12`
  crate，无误，但切片落地按 `k12 = "0.5.1"` 引入。
- 三 crate 1.85.0 编译干净（无 ctutils 型传递坑，`digest 0.11` 面）。
- **语料面 41 断言全过**：KT128 全向量集（17 条：ptn(17^n) 幂次输入至
  241,375B、8191/8192 块边界、`CustomKt128::new_customized` 定制串）+
  KT256 + TurboSHAKE128 全集（15 条：默认 DS=0x1f + `CTurboShake128::<DS>`
  const 泛型 DS=0x01/0x06/0x07/0x0b）+ cSHAKE128/256（outputLength
  256/384/512 及 **0 bit 空输出**）。
- **host 面设计事实**：现 `crypto_subtle_digest(algo, bytes_csv)`
  （`crates/engine/src/js_dom_bridge/crypto.rs:24`）定长签名不支持语料
  `{name, outputLength, customization/domainSeparation}` 调用形——XOF 切片
  需新增回调（如 `crypto_subtle_xof_digest(algo, bytes_csv, output_bits,
  param_csv)`），shim 侧先抽算法对象成员再回调；**TurboSHAKE DS 为 const
  泛型**（0x01..=0x7f），host 需运行时 byte → const 泛型小分派（macro 或
  match 臂），实现时评估实例化膨胀。SHA3/SHAKE 定长族无参，既有 digest
  面分派扩展即可，未另探（低风险）。

## 对 D1 的影响

| 域 | 原建议 | 预检后 |
|---|---|---|
| argon2 | 0.6.0 | **0.6.0 维持** |
| AES-OCB | ocb3 0.2.0 | ocb3 0.2.0 + **aes =0.9.2 + ctutils =0.4.2 钉版** |
| ChaCha20-Poly1305 | 0.11.0 | **0.11.0 维持 + ctutils =0.4.2 钉版** |
| XOF | k12 0.5.1 等 | **维持，crate 名 `k12` 更正**；host 回调扩参 + DS const 泛型分派 |

四个先行域（deps evidence 定序第一批）全部具备语料面实证，D1 拍板即可
开工无预检残留。ML-KEM/ML-DSA/X448-Ed448/KMAC 拍板子问题不变（见 master.md
D1 行）。
