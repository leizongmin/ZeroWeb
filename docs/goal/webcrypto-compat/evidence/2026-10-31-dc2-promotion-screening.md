# DC-2 转正甄别清单（2026-10-31，M1）

## 口径与上游事实

- 语料 pin（一期沿用）：`315976933870b34d6ea30e3f6643403edae678ba`，
  WebCryptoAPI 子树 `ad14e36c81d741f265d7d4a80271c370a82569f2`（git trees，
  202 blob，126 test 入口，其中 `.tentative.` 50）。
- 上游 master（2026-10-31 查证）：`cb6cc97ada2772b2afe668b9fd5cb39a576a5347`，
  WebCryptoAPI 子树 `084495e78396bf8e3c4d797a8e761afecdc0a8f5`（216 blob，
  136 test 入口，`.tentative.` **0**）。
- 转正判定不靠文件名单查：HEAD 的 `WebCryptoAPI/modern-algos/META.yml` 与
  `secure-curves/META.yml` 只声明 `spec:`（wicg/webcrypto-modern-algos /
  wicg/webcrypto-secure-curves），**无 `tentative:` 声明**。
- 分母基准：wpt.fyi aligned stable（coverage-2026-10-31 快照同源，rev
  ceec05f81f）三家并集——WebCryptoAPI 334 文件 / 70,756 子测试，其中
  modern-algos + secure-curves 两族 **109 文件 / 18,052 子测试**。

## 结论：一期挂账域全部转正

pin→HEAD diff 显示一期 7,320F 挂账的 `.tentative.` 域（50 文件）在上游已
**全部离开 tentative**：文件从旧位置删除、以非 tentative 名迁入两个新目录族
（`modern-algos/`、`secure-curves/`），无任何域维持挂账状态。此外 HEAD 新增
pin 时不存在的 Hybrid-KEM 域与若干 stable 新案。

## 逐域甄别与立项切片建议（D1 待用户决策）

分母为 universe（三家并集）子测试数；依赖列为候选（**本 goal 不引入**，
选型须用户拍板 + deps evidence 先行）：

| 域 | pin 位置（tentative） | HEAD 位置（转正） | 分母 | 建议范围 | 依赖候选 |
|---|---|---|---:|---|---|
| ML-KEM | generateKey/import_export/serialization/encap_decap ×.tentative. | modern-algos/ 同族目录 | 12 案 / 3,480 | encap/decap + generateKey + import/export raw 全链 | ml-kem 0.3.3（RustCrypto） |
| ML-DSA | 同上 ×.tentative. | modern-algos/ 同族目录 | 10 / 1,206 | sign/verify + generateKey + import/export | ml-dsa 0.1.1（RustCrypto） |
| X448/Ed448 | derive_bits_keys/generateKey/import_export/serialization/sign_verify ×.tentative. | secure-curves/ | 26 / 3,700 | deriveBits/deriveKey + generateKey + import/export + sign/verify + serialization | x448 0.14.0-pre.12 / ed448-goldilocks 0.14.0-pre.15（pre 版，选型注意） |
| AES-OCB | encrypt_decrypt 等 ×.tentative. | modern-algos/encrypt_decrypt 等 | 10 / 2,652 | encrypt/decrypt + generateKey + import/export + serialization | ocb3 0.2.0（非 RustCrypto AEADs 主族，deps evidence 先行） |
| ChaCha20-Poly1305 | 同上 ×.tentative. | modern-algos/ 同族目录 | 10 / 1,044 | 全链 | chacha20poly1305 0.11.0（RustCrypto AEADs） |
| KMAC | 同上 ×.tentative. | modern-algos/sign_verify 等 | 10 / 1,418 | sign/verify + import/export | sha3 0.12（KMAC 面需核实 feature） |
| Argon2 | derive_bits_keys/argon2.tentative. 等 | modern-algos/derive_bits_keys 等 | 4 / 92 | deriveBits + importKey | argon2 0.6.0（RustCrypto password-hashes） |
| XOF digest（cSHAKE/K12/SHA3/TurboSHAKE） | digest/ ×.tentative. | modern-algos/digest/ | 8 / 838 | digest 扩展算法注册面 | sha3 0.12 + kangarootwelve 0.1.3 |
| Hybrid-KEM（pin 无此域） | —（HEAD 新增） | modern-algos/encap_decap 等 | 7 / 2,742 | encap/decap 组合域（ML-KEM+curve 系） | 随 ML-KEM/curve448 选型 |
| modern-algos 门禁面（supports/getPublicKey/raw 别名/idlharness） | supports*.tentative. 等 | modern-algos/ 顶层 | 12 / 880 | supports 门禁语义 + getPublicKey | 无新依赖 |

合计 109 案 / 18,052 子测试（universe 口径）。

## 对本 goal 边界的影响

- 本 goal 不执行收敛（入口文档 Mission 2，2026-10-31 用户决策）：上表全部
  转入 master.md「待用户决策」D1，按域立项时点由用户定。
- corpus 仍在 pin：转正后的新目录族文件 pin 里不存在，语料补全（DC-1）以
  pin 全量为准——一期 corpus 即 pin 全量（198 文件全在位，见
  [M1 语料与基线](2026-10-31-m1-corpus-and-baseline.md)）。
- coverage 未覆盖面（57,509）的主体即上述转正族 + 稳定案 window/worker
  变体双计，归因与清零路径见
  [DC-4 对账](2026-10-31-dc4-coverage-attribution.md)。

## 查证方法备注

- pin/HEAD 清单：GitHub git trees API（`?recursive=1`），逐 blob diff。
- META.yml：raw.githubusercontent（HEAD，2026-10-31）。
- 依赖候选：crates.io API（2026-10-31 查证，名称/版本/更新时间见上表）。
