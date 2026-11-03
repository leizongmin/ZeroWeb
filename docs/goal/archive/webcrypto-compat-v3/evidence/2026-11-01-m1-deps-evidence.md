# M1 依赖候选 deps evidence（2026-11-01，DC-2 前半）

> 逐域候选 crate 落证（版本/维护面/feature/license/MSRV），来源 crates.io API
> + static.crates.io / cargo registry 源码核对（2026-11-01 实测）。**引入一律
> 待用户拍板**（二期决策纪律延续）；本账只落证据与建议，不落引入。

## 复用基准（既有 workspace 依赖，零新引入）

`Cargo.toml` L135-172 既有 crypto 面：sha1/sha2 0.10（oid）、aes-gcm 0.10、
aes 0.8、cbc 0.1、ctr 0.9、rsa 0.9、p256/p384/p521 0.13（arithmetic/ecdsa/
ecdh/pkcs8）、ed25519-dalek 3.0、x25519-dalek 3.0（static_secrets）、
getrandom 0.3。全部 RustCrypto 系。

## 逐域证据与建议

### D-1 ML-KEM（候选 ml-kem）

- **ml-kem 0.3.3**（DC-2 甄别清单候选）：在册未 yanked；**0.3.4 stable 已于
  2026-10-31 发布**（同周补丁）。repo RustCrypto/KEMs；Apache-2.0 OR MIT；
  MSRV 1.85；downloads 13.55M / recent 9.70M；features：alloc/getrandom/
  hazmat/pem/pkcs8/zeroize。
- 建议：**0.3.4**（最新补丁，甄别期内发布的小版本）。

### D-2 ML-DSA（候选 ml-dsa）

- **ml-dsa 0.1.1**（候选）：在册未 yanked；**0.1.2 stable 已于 2026-10-31
  发布**。repo RustCrypto/signatures；Apache-2.0 OR MIT；MSRV 1.85；
  downloads 4.83M / recent 3.22M；features：alloc/getrandom/pkcs8/rand_core/
  zeroize。
- 建议：**0.1.2**。

### D-3 X448/Ed448（候选 x448 + ed448-goldilocks）

- **x448 0.14.0-pre.12**（候选）：pre 线最新（2026-06-24）；max stable 仅
  0.6.0（旧 trait 栈线）。repo RustCrypto/elliptic-curves；Apache-2.0 OR
  MIT；MSRV 1.85；downloads 3.10M / recent 1.02M；features：default/
  getrandom/serde/static_secrets。cargo fetch 实测传递依赖
  elliptic-curve 0.14.3 + sec1 0.8.1——**与 workspace p256/p384/p521 0.13
  （elliptic-curve 0.13 线）不同主版本，共存但栈分叉**。
- **ed448-goldilocks 0.14.0-pre.15**（候选）：pre 线最新（2026-06-24）；
  max stable 0.9.0。同 repo/license/MSRV；downloads 4.26M / recent 1.46M；
  features：alloc/getrandom/pkcs8/serde/signing/std；传递依赖 ed448 0.5.0。
- 甄别从严（入口文档既定）：pre 版无 stable 承诺、API 随 0.14 线迭代中；
  stable 替代（0.6.0/0.9.0）API 栈更旧、与既有 EC 栈同样不统一。
  **建议：pre.12/pre.15（pre 线即事实活跃线，下载量集中在 pre）——pre 甄别
  风险显式交用户拍板。**

### D-4 AES-OCB（候选 ocb3）

- **ocb3 0.2.0**：stable（2026-08-24）。**更正入口文档预判**：0.2.0 已迁入
  RustCrypto/AEADs 官方仓库（repository 字段 + AEADs 家族 trait 面），「非
  RustCrypto AEADs 主族」 concern 不再成立。Apache-2.0 OR MIT；MSRV 1.85；
  downloads 6.09M / recent 2.37M；features：alloc/arrayvec/bytes/getrandom/
  rand_core。
- 建议：**0.2.0**（RustCrypto AEADs 官方家族）。

### D-5 ChaCha20-Poly1305（候选 chacha20poly1305）

- **chacha20poly1305 0.11.0**：stable（2026-06-28）。repo RustCrypto/AEADs；
  Apache-2.0 OR MIT；MSRV 1.85；downloads 105.87M / recent 35.22M；features：
  alloc/arrayvec/bytes/getrandom/rand_core/reduced-round/zeroize。
- 建议：**0.11.0**（与 aes-gcm 同构建于 aead/cipher 家族）。

### D-6 KMAC（入口候选「sha3 0.12 KMAC feature」→ 证伪，改列两案）

- **sha3 0.12.0 实测不含 KMAC**：源码 lib.rs 仅 166 行，只导出 SHA3-224/
  256/384/512 固定输出变体（SHAKE 另在 `shake` crate，README 明示）；无
  KMAC/cSHAKE。入口候选的前提（sha3 自带 KMAC feature）不存在。
- RustCrypto 官方无 KMAC crate（crates.io 检索确认）。
- 候选 a：**sha3-kmac 0.3.0**（ericlagergren/sha3）：BSD-3-Clause；MSRV
  1.85；NIST SP 800-185 实现（~500 行含 NIST 向量单测）；downloads 27.6k
  （小众）；依赖 sha3 **0.10** + sha3-utils 0.5——**与 workspace sha3 0.12
  候选 semver 不兼容共存**（Keccak 双实现入树，代码体积代价、无正确性影响）。
  作者个人维护（非 RustCrypto org，lints 面完整）。
- 候选 b：**手写 KMAC 叠官方 cshake 0.2.1**：cshake 0.2.1 源码提供
  `new_with_function_name(b"KMAC", s)` 原语，KMAC = bytepad(encode_string(K))
  + encode_string(X) + right_encode(L) 框架（SP 800-185 §4.3，~40 行），
  框架 helper 可用 sha3-utils 0.5（同作者，114k recent）或手写；NIST 向量
  自测随切片落仓。
- **建议交用户拍板**（a：成熟实现但引入非官方 crate + 双 Keccak 栈；
  b：零新 KMAC 依赖但手写密码学框架代码）。

### D-7 Argon2（候选 argon2）

- **argon2 0.6.0**：stable（2026-08-27）。repo RustCrypto/password-hashes；
  MIT OR Apache-2.0；MSRV 1.85；downloads 69.98M / recent 27.53M；features：
  alloc/getrandom/kdf/parallel/password-hash/rand_core/zeroize。
- 建议：**0.6.0**。

### D-8 XOF digest（入口候选「sha3 0.12 + kangarootwelve 0.1.3」→ 修订）

语料算法面（modern-algos/digest 实测）：SHA3-256/384/512、SHAKE128/256、
cSHAKE128/256（含 customization 参数）、KangarooTwelve、TurboSHAKE128/256。

- **sha3 0.12.0**（SHA3-224/256/384/512）：stable（2026-05-15）；repo
  RustCrypto/hashes；MSRV 1.85；downloads 199.26M / recent 54.22M。✓
- **shake 0.1.0**（SHAKE128/256）：stable；repo **RustCrypto/XOFs（官方新
  仓库）**；MSRV 1.85；downloads 6.12M / recent 4.95M。
- **cshake 0.2.1**（cSHAKE128/256）：stable；同 repo；MSRV 1.85；downloads
  213k / recent 144k；提供 `new_with_function_name` 原语（D-6 候选 b 的地基）。
- **k12 0.5.1**（KangarooTwelve）：stable；同 repo；MSRV 1.85；downloads
  1.17M / recent 410k。**替代入口候选 kangarootwelve 0.1.3**：后者为个人
  维护（itzmeanjan，MIT），MSRV **1.86 > workspace MSRV 1.85**，downloads
  4.5k / recent 108，带 cuda/multi_threaded 非目标 features——甄别从严
  不建议，官方 k12 全面占优。
- **turboshake 0.7.1**（TurboSHAKE128/256）：stable；同 repo；MSRV 1.85；
  downloads 955k / recent 859k（入口文档未列候选，按语料面补齐）。
- 建议：**sha3 0.12 + shake 0.1.0 + cshake 0.2.1 + k12 0.5.1 + turboshake
  0.7.1**（5 crate，同一 RustCrypto hashes/XOFs 家族，Keccak 底座共享；
  kangarootwelve 不引入）。

### D-9 Hybrid-KEM（随 ML-KEM 选型，无独立新依赖）

- 语料组合面（draft-irtf-cfrg-concrete-hybrid-kems-03 vectors 实测）：
  MLKEM768-P256 / MLKEM768-X25519 / MLKEM1024-P384——曲线侧 P-256/X25519/
  P-384 **全在既有依赖内**（p256/p384 0.13、x25519-dalek 3.0），无 X448。
- 组合器 KDF 按草案 SHA3 系（sha3 0.12 候选覆盖）；组合逻辑（encap 组合 +
  secret 拼接 + KDF）在 engine 切片实现。
- 建议：**无独立新依赖**；随 D-1 拍板生效。

### D-10 门禁面（supports 门禁 + getPublicKey）

- 无新依赖。shim/engine 注册语义面切片（M2 无依赖面首切片，含
  `[SecureContext]` 门禁不回退核对）。

## 汇总（拍板清单建议值）

| 域 | 建议 | 新 crate 数 | 备注 |
|---|---|---|---|
| ML-KEM | ml-kem 0.3.4 | 1 | 0.3.3 候选已过时一日 |
| ML-DSA | ml-dsa 0.1.2 | 1 | 0.1.1 候选已过时一日 |
| X448/Ed448 | x448 0.14.0-pre.12 + ed448-goldilocks 0.14.0-pre.15 | 2 | pre 版甄别交拍板 |
| AES-OCB | ocb3 0.2.0 | 1 | 已入 RustCrypto AEADs（更正预判） |
| ChaCha20-Poly1305 | chacha20poly1305 0.11.0 | 1 | |
| KMAC | sha3-kmac 0.3.0 或 手写叠 cshake | 1–2 | 两案交拍板 |
| Argon2 | argon2 0.6.0 | 1 | |
| XOF digest | sha3 0.12 + shake 0.1.0 + cshake 0.2.1 + k12 0.5.1 + turboshake 0.7.1 | 5 | kangarootwelve 证伪替换 |
| Hybrid-KEM | 无新依赖 | 0 | 随 ML-KEM |
| 门禁面 | 无新依赖 | 0 | |

新 crate 合计建议值：**12–13 个**（全 RustCrypto 官方系除 KMAC 候选 a；
license 全 Apache-2.0 OR MIT / MIT / BSD-3-Clause；MSRV 全 1.85，除
kangarootwelve 1.86 已排除）。

## 无依赖面推进（拍板前，入口文档既定）

门禁面（D-10）与散新案（baseline 中 3 案已全绿；export-key-error-order
2P/4F、sharedarraybuffer 0P/8F 待归因）为拍板前可切片面；worker 腿接线
（runner/workers 通道）与 1T 销案（DOM 边界）无新依赖，穿插推进。
