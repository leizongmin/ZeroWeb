# DC-2 deps evidence 编译预检——MSRV 1.85 实证 + 建议集修正 + KMAC 双案互证（2026-11-02）

> M1 deps evidence（2026-11-01）的 MSRV/共存结论全部为 crates.io metadata +
> 源码阅读级证据，未实际构建。本轮在 /tmp 隔离 scratch 用 rustc 1.85.0
> minimal 工具链（test-guard 包裹，`--per-proc-mem 8 --total-mem 16
> --time-limit 900`）对建议集做解析 + 编译 + 运行时冒烟，暴露三处 evidence
> 记载与事实不符，并在拍板前完成修正。仓库零触面（Cargo.toml/源码零改动），
> scratch 位于 `/tmp/zw-dc2-msrv-precheck` 与 `/tmp/zw-kmac-oracle`（一次性
> 诊断产物，不入库）。

## 结论（拍板前必须修正的三项 + 两项增强）

1. **PQC 建议值回落**：ml-kem 0.3.4 / ml-dsa 0.1.2 要求 rustc **1.87**
   （crates.io rust-version 字段，0.3.3/0.1.1 = 1.85）；evidence 汇总表
   「MSRV 全 1.85」对升替版失效。1.85 兼容集 = **ml-kem 0.3.3 + ml-dsa
   0.1.1**（+ module-lattice 0.2.3、ctutils 0.4.2 两处 `--precise` 降选），
   1.85.0 编译 + 运行时冒烟通过。
2. **X448/Ed448 pre 线 1.85 无解**（硬工具链阻塞，非仅 API 稳定性风险）：
   x448 pre.12 → ed448-goldilocks pre.15 → elliptic-curve ^0.14 →
   crypto-bigint ^0.7.5（≤1.85 的版本全 yanked，非 yanked 全 1.87）+
   ctutils ^0.4.6（非 yanked 全 1.87）。且任一 1.87 包入树即整 workspace
   `cargo build`@1.85 失败——**非按域豁免，是 workspace 级 MSRV 1.85→1.87**
   。拍板子问题②的选项空间改为：
   - (a) pre 线 + 接受 workspace MSRV 1.87（含全部能力：X448 derive +
     Ed448 sign/verify，pre.15 sign/ 模块在）；
   - (b) X448 走 stable **x448 0.6.0**（1.85 编译 + DH 冒烟通过；旧 API：
     `Secret::new`/`as_diffie_hellman`，RNG 走 rand_core 0.5），
     **Ed448 无 1.85 可用路径**——ed448-goldilocks 0.9.0 无 signing 面
     （src 仅 curve/decaf/field/ristretto，features 仅 fiat 后端）、
     ed448 0.5.0 仅 Signature 类型 + pkcs8（doc 示例即用户自建 signer），
     签名实现只存在于 pre.15 sign/ 模块；
   - (c) 域降 scope（弃 Ed448 或整体延后）。
3. **KMAC 证据增强（两案都有新事实）**：
   - option b（手写叠官方 cshake 0.2.1）**可行性实证**：~50 行框架
     （left/right_encode + encode_string + bytepad + KMAC 组块）与
     sha3-kmac 0.3.0 逐位相等 ×4 用例（空 data、空 custom、168B rate
     边界 key、300B 长输入）；cshake 0.2.1 海绵与 sha3 0.10 官方海绵同块
     互证相等。无短 key 限制、无双 Keccak 栈代价。
   - option a（sha3-kmac 0.3.0）**spec 合规缺口**：`MIN_KEY_SIZE =
     security/8`（KMAC128 = 16B），拒绝 spec 合法的短/空 key（KMAC spec
     允许任意长度 key）——语料面短 key 案将红；另有 sha3 0.10 + 0.12
     双 Keccak 入树 + 作者个人维护。
   - **关键约定事实**：`encode_string` 长度取 **BIT**（left_encode(len×8)
     ）——SP 800-185 §2.3.2 正文与 NIST 参考代码存在 byte/bit 二义，
     sample 向量与主流实现取 bit。本次互证以手写 byte-len 版分歧暴露后
     定谳；option b 落仓时框架代码必须以 NIST 向量单测交叉验证，不得从
     spec 正文重推。
4. XOF 五连（sha3 0.12 + shake 0.1.0 + cshake 0.2.1 + k12 0.5.1 +
   turboshake 0.7.1）、ocb3 0.2.0、chacha20poly1305 0.11.0、argon2 0.6.0
   全部 1.85 编译 + 冒烟通过，evidence 建议值维持。
5. Hybrid-KEM（无独立新依赖）、门禁面（无新依赖）不受影响。

## 1.85 兼容集终态（编译 + 运行时冒烟全过）

| 域 | 版本 | 验证内容 |
|---|---|---|
| ML-KEM | ml-kem 0.3.3（**features +getrandom**，default 只有 alloc） | `Kem::generate_keypair` / `encapsulate` / `decapsulate` 全链 + 共享密钥相等断言 |
| ML-DSA | ml-dsa 0.1.1 | `SigningKey::generate` / `sign` / `verifying_key` / `verify` 签验往返 |
| AES-OCB | ocb3 0.2.0 + **aes 0.9** | encrypt/decrypt roundtrip（Ocb3\<Aes128\>，NonceSize=U12 默认） |
| ChaCha20-Poly1305 | chacha20poly1305 0.11.0 | encrypt/decrypt roundtrip（nonce 12B） |
| Argon2 | argon2 0.6.0 | hash_password（无盐参）→ PHC str → verify_password 往返 |
| XOF digest | sha3 0.12 + shake 0.1.0 + cshake 0.2.1 + k12 0.5.1 + turboshake 0.7.1 | SHA3-256 digest + 四 XOF squeeze/read 冒烟 |
| X448（方案 b） | x448 0.6.0 | 双方 Secret::new → as_diffie_hellman 共享密钥相等断言 |

传递降选两处（ml-kem 0.3.3 腿）：`module-lattice 0.2.4 → 0.2.3`、
`ctutils 0.4.7 → 0.4.2`（后者在移除 ed448 pre 线依赖后可降；先前失败是
elliptic-curve 0.14.1 链的 crypto-bigint ^0.7.5 冲突）。

## 新栈 API 形状事实（域切片工程量输入，registry 源码核实）

RustCrypto 2025-2026 新一代栈与本仓既有面形状差异显著，切片实现按以下
事实写，不按旧 API 记忆：

- **crypto-common 0.2**：`Generate::generate()` **无 rng 参数**（OS 熵内
  取，infallible）；显式 RNG 路径 = rand_core **0.10** trait +
  getrandom **0.4** `SysRng` + `rand_core::UnwrapErr` 包装（rand_core 0.10
  本体已无 OsRng）。
- **ml-kem 0.3.3**：keygen/encap 走 `kem 0.3::Kem` trait
  （`generate_keypair` / `encapsulate` 无参默认面在 `getrandom` feature
  门控下，**ml-kem default 只有 alloc，需显式 +getrandom**）；带 rng 变体
  `generate_keypair_from_rng` / `encapsulate_with_rng`（rand_core 0.10
  `CryptoRng` bound）；`Decapsulate::decapsulate` infallible 直返
  SharedKey；`FromSeed` 面在（确定性测试向量路径）。
- **ml-dsa 0.1.1**：`SigningKey::<MlDsa65>::generate()` 无参；
  `verifying_key` 走 signature 3.0 `Keypair` trait（ SigningKey 同名
  字段私有）；`Signer::sign(&self, msg: &[u8])` / `Verifier::verify`
  收裸 slice（`&[u8; N]` 不自动 coerce）。
- **aead 0.6**（ocb3/chacha20poly1305）：`Key`/`Nonce` = crypto-common/
  hybrid-array `Array`；`Aead::encrypt`/`decrypt` 的 payload
  `impl Into<Payload>` **仅 `From<&[u8]>`，无 `From<Vec<u8>>`**——传
  `ct.as_slice()` 而非 `ct`。
- **argon2 0.6**：`PasswordHasher::hash_password(&self, password)` 无盐
  参（盐内部生成）；`verify_password` 为 `PasswordVerifier<str>`——PHC
  字符串 `&str`；`SaltString` 已移至 `phc` crate；定制盐路径 =
  `hash_password_with_salt(password, salt_bytes)`。
- **XOF 家族**：digest **0.11** 对齐（四 crate 同实例），trait 从
  `shake` root 一次 `use` 全家族通用；`CShake128::new_with_function_name
  (function_name, customization)` 双参（N/S 内部编码）；构造器
  `::default()`。
- **ocb3 0.2.0**：主类型 `Ocb3<Cipher, NonceSize=U12, TagSize=U16>`，
  `KeyInit::new(&Key)`；依赖 **aes 0.9**（cipher 0.5 线）。

## workspace 共存树分叉清单（采纳 1.85 兼容集的真实代价）

workspace lock 已在：digest 0.10.7 + **0.11.3（XOF 家族零新增）**、
getrandom 0.2.17 + 0.3.4 + **0.4.3（已在）**、rand_core 0.6.4 + 0.9.5 +
**0.10.1（已在，PQC 新栈 RNG 面零新增）**。真正新增分叉：

| 新增 | 来源 | 影响 |
|---|---|---|
| aes 0.9 | ocb3 0.2.0 | 与既有 aes 0.8.4（aes-gcm/cbc/ctr）双 major，代码体积代价，无正确性影响 |
| rand_core 0.5 | x448 0.6.0（方案 b） | 旧 RNG trait 面，与 0.6.4/0.9.5/0.10.1 四版本并存 |
| sha3 0.10 + 0.12 双 Keccak | 仅 KMAC option a | option b 无此项 |
| elliptic-curve 0.14 / crypto-bigint 0.7 / ctutils 0.4 | 仅 X448/Ed448 pre 线 | 同时推高 MSRV 至 1.87（见结论 2） |

## KMAC 双案互证细节（/tmp/zw-kmac-oracle）

- 四条腿：手写框架 + cshake 0.2.1 海绵；oracle = sha3-kmac 0.3.0
  （自带 NIST 向量单测，作为可信基）；第三腿 = 同一手写块喂 sha3 0.10.9
  `CShake128Core`（oracle 同底层，隔离海绵差异）；第四腿 =
  sha3-utils helper 组块（`bytepad_blocks` 前缀语义与手写 bytepad 不对齐
  ，仅诊断信息未对齐，不影响主线）。
- 结果：4 用例手写 ≡ oracle ≡ 官方海绵逐位相等；修正项 =
  `encode_string` 长度 byte→bit（见结论 3 关键约定事实）。
- 短/空 key（4B、空）：手写面跑通；sha3-kmac `new` 返回
  `InvalidLength`（MIN_KEY_SIZE=16B）。

## 复现

```bash
rustup toolchain install 1.85.0 --profile minimal
cd /tmp/zw-dc2-msrv-precheck   # scratch（Cargo.toml 见本账版本表）
rustup run 1.85.0 cargo build && rustup run 1.85.0 cargo run
cd /tmp/zw-kmac-oracle && rustup run 1.85.0 cargo run
```

crates.io API 版本核查（rust_version/yanked 字段）：ml-kem、ml-dsa、
crypto-bigint、module-lattice、ctutils、elliptic-curve、ed448、
ed448-goldilocks（GET /api/v1/crates/<name>/versions，2026-11-02 实测）。

## 对 D1 待决表的影响

- 子问题①（总体建议值）：PQC 两域建议值 0.3.4/0.1.2 → **0.3.3/0.1.1**
  （其余维持）。
- 子问题②（X448/Ed448 pre 甄别）：输入升级为三选（pre+MSRV 1.87 /
  X448 stable+Ed448 弃 / 域降 scope），pre 线不是「API 风险」而是
  「MSRV 硬阻塞」。
- 子问题③（KMAC 两案）：option b 从纸面变实证（含无短 key 缺口优势），
  option a 暴露短 key spec 缺口 + 双栈代价。
- 子问题④（ml-kem/ml-dsa 版本升替）：被 1.85 约束否决——升替即 1.87。
