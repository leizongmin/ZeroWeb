# D1 前置：PQC/KMAC/X448 语料面预检（2026-11-02，scratch 探针零仓库触面）

> [先行四域语料面预检](2026-11-02-d2-domain-slice-corpus-face-precheck.md) 的
> 补全：D1 拍板子问题①（ML-KEM/ML-DSA）③（KMAC 两案）②（X448 option b）
> 的语料真实向量面在 rustc 1.85.0 下逐字节复现。方法同先行四域：从
> `tests/wpt-runner/wpt-data/WebCryptoAPI/` 语料 JS 提取向量生成 scratch
> 探针（生成器 `/tmp/zw-gen-pqc-kmac-corpus-probe.py`，crate 位于
> `/tmp/zw-dc2-corpus-face-{mlkem,mldsa,kmac,x448}`，一次性验证产物不入库），
> `RUSTUP_TOOLCHAIN=1.85.0` + test-guard（`--per-proc-mem 4 --total-mem 8
> --time-limit 600`）编译运行。至此 **1.85 可探域的语料面预检全部闭环**
> （Ed448 无 1.85 可用路径，无可探对象，见 [MSRV 编译预检](2026-11-02-dc2-msrv-compile-precheck.md) 结论 2）。

## 逐域结论

### ML-KEM 0.3.3：3/3 全过（seed 确定性 + decap KAT）

语料 `encap_decap/ml_kem_vectors.js` 三参数集 KAT，经 `DecapsulationKey::
from_seed`（`Seed = Array<u8, U64>`）确定性展开：

- **keygen 确定性**：`dk.encapsulation_key().to_bytes()` 与语料 publicKey
  逐字节相等（ML-KEM-512/768/1024 三集全过）——语料 raw-secret 导入的
  64B seed 即 FIPS 203 d‖z，展开结果与上游生成侧一致。
- **decap KAT**：`decapsulate(sampleCiphertext)` 与 expectedSharedSecret
  逐字节相等（768/1088/1568B ciphertext 三集全过）。
- 语料驱动为 `encap_decap_bits.https.any.js`（raw-secret 导入 seed →
  decapsulateBits），与探针路径一致。
- 钉版维持 MSRV 预检结论：`ctutils =0.4.2` + `module-lattice =0.2.3`
  （探针 lock 已按此钉，1.85.0 编译干净）。

### ML-DSA 0.1.1：3/3 全过（verify KAT + roundtrip；det_sign 差异已归因）

语料 `sign_verify/mldsa_vectors.js` 三变体（pkcs8 seed-only DER 尾部 32B
seed + spki raw ek + 固定 48B 消息 + 固定签名）：

- **verify KAT**：`SigningKey::from_seed` → `VerifyingKey::decode(spki raw)`
  → corpus 固定签名 verify 通过（ML-DSA-44/65/87 全过）——语料签名为
  确定性锚点。
- **det_sign == corpus？否**：ml-dsa 0.1.1 `Signer` 为 FIPS 204 确定性变体
  （源码注释明确 empty context、rnd 空），产出与 corpus 签名**不同**。
  归因：corpus 签名非确定性变体生成（hedged/rnd 非 0），而 WPT 驱动
  `signature.js` 对 ML-DSA **未开 `katFirst`**——sign 只要求生成签名可验
  （roundtrip），固定签名只用于 verify KAT。**非缺口**，但域切片时 sign
  面不得断言字节等于语料签名。
- **负例**：5B 非法签名在 `Signature::try_from` 长度校验即拒——host shim
  的 import/verify 面须以解码失败承接（verify 返回 false）。
- 钉版同上两处。

### KMAC 双案：6/6 NIST 采样双案逐字节全过；**option a 语料面缺口实锤**

语料 `sign_verify/kmac_vectors.js` = NIST SP 800-185 KMAC samples 全 6 条
（KMAC128 L=256 ×2、KMAC256 L=512 ×4；customization 空/21B、data 4B/200B）：

- **option b（手写叠官方 cshake 0.2.1）**：KMAC128 rate=168 + **KMAC256
  rate=136** 两路径全过——上一轮互证框架直接复用，KMAC256 扩展一次通过，
  与 oracle 逐位相等再证。
- **option a（sha3-kmac 0.3.0）**：6/6 同样全过（`Kmac128::new(k, s)` /
  `Kmac256::new(k, s)`，`update` + `finalize_into`）。
- **新语料面事实（对③有决定性）**：`import_export/symmetric_importKey.js`
  的 rawKeyData 为 **16B/24B/32B 三档**，KMAC128/KMAC256 全跑（无 ChaCha
  式尺寸跳过）。sha3-kmac `MIN_KEY_SIZE = security/8` → **KMAC256 拒收
  16B/24B key（MIN=32B）**，语料正例 import 必红；KMAC128 的 16B 恰在
  边界（MIN=16B）通过。option b 无 key 长度限制，全绿。结合 MSRV 预检
  已发现的 sha3 双栈代价，**option b 证据面进一步强化**。

### X448（D1 子问题② option b 加固）：KAT 全过 + 小阶点拒绝路径成立

语料 `derive_bits_keys/cfrg_curves_bits_fixtures.js`（`cfrg_curves.js`
驱动，X448 分支）：

- **deriveBits KAT**：x448 0.6.0 `Secret::as_diffie_hellman` 56B 共享密钥
  与语料逐字节相等（pkcs8 尾 56B scalar × spki 尾 56B public）。
- **5 条小阶点负例全可拒**：语料驱动要求 **importKey 成功** + deriveBits
  抛 `OperationError`（测试名即 "all-zero value result"）。x448 0.6.0 实测
  分两段承接：
  - order 0/1/2 三点：crate `is_low_order` 在 `as_diffie_hellman` 内拒绝
    （返回 None）；
  - order 4/1 两点（p、p+1）：crate 返回 `Some` 且共享密钥**全零**——host
    shim 须对输出做全零后检查后抛 OperationError（同 RFC 7748 §6 提供
    all-zero 判据，实现为普通 shim 规则）。
  - **host 侧义务两条**：① import 必须用无贡献性检查的解码（crate
    `from_bytes` 会在导入期拒小阶点，与语料 importKey 必须成功冲突；用
    `from_bytes_unchecked` 承接）；② derive 全零后检查。
- **option (b) 语料面成立**；option (a)/(c) 的取舍仍由 MSRV 硬约束主导
  （pre 线 1.87、Ed448 无 1.85 签名路径），本轮不改变该格局。

## 对 D1 待决表的影响

| 子问题 | 预检后证据面 |
|---|---|
| ① 总体选型（ml-kem 0.3.3 / ml-dsa 0.1.1） | **两域语料 KAT 全过**，钉版组合实证，拍板即可开工 |
| ② X448/Ed448 三选 | option (b) X448 语料面成立（+host 两条义务）；option (a) 维持 MSRV 1.87 代价；Ed448 无 1.85 路径不变 |
| ③ KMAC 两案 | 双案 NIST 6/6 全过；**option a 的 MIN_KEY_SIZE 缺口从理论变语料实锤**（KMAC256 16B/24B import 必红），option b 全绿无缺口 |
| ④ ml-kem/ml-dsa 升替 | 已被 1.85 否决，不变 |

## 复现

```bash
python3 /tmp/zw-gen-pqc-kmac-corpus-probe.py   # 生成三个探针 crate（x448 探针内联生成）
cd /tmp/zw-dc2-corpus-face-mlkem
env RUSTUP_TOOLCHAIN=1.85.0 cargo update ctutils@0.4.7 --precise 0.4.2
env RUSTUP_TOOLCHAIN=1.85.0 cargo update module-lattice@0.2.4 --precise 0.2.3
env RUSTUP_TOOLCHAIN=1.85.0 cargo build && env RUSTUP_TOOLCHAIN=1.85.0 cargo run
# mldsa 同（同两处 --precise）；kmac/x448 无钉版直接 build+run
# 全程 test-guard 包裹：--per-proc-mem 4 --total-mem 8 --time-limit 600
```

输出：ML-KEM 3/3 PASS、ML-DSA 3/3 PASS、KMAC 6/6 both candidates、
X448 KAT + 5/5 small-order PASS。
