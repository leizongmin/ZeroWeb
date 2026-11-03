# D1 域切片第五轮：Hybrid PQ/T KEM 三组合（2026-11-03）

> D1 收官批（[d1s4](2026-11-03-d1s4-x448-kmac-slice.md) 后延续）：MLKEM768-P256
> / MLKEM768-X25519 / MLKEM1024-P384 全链路（draft-irtf-cfrg-concrete-hybrid-kems-03
> §4，CG 框架）。复用 d1s3 落树的 ml-kem 0.3.3 与已有 p256/p384/x25519 依赖，
> 零新增 crate。宿主实现 `crypto_hybrid.rs`（新文件）→ `crypto_asym.rs` 六
> mlkem 入口按名分流 → shim `_zw_mlKemIsFamily` 家族扩名（part02/callbacks
> 零改动）→ REG 三行 → 定向翻绿 → 锚对照零回归。对照基准
> [d1s4 锚](2026-11-03-d1s4-x448-kmac-full.json.gz)；本轮新基准
> [d1s5 锚](2026-11-03-d1s5-hybrid-kem-full.json.gz)。

## 宿主实现（`crypto_hybrid.rs`，新文件 ~600 行）

- **三组合参数表**（draft-03 §4，逐字段）：`HybridParams { ek_pq, ek_t,
  ct_pq, nseed_t, label, alg_id, ml1024 }` —— P256(1184/65/1088/128)、
  X25519(1184/32/1088/32)、P384(1568/97/1568/48)；seed 恒 32B、ss 恒 32B。
  label：`"MLKEM768-P256"` / `"\.//^\"`（X25519）/ `"MLKEM1024-P384"`。
- **ExpandSeed**（§3.2）：SHAKE256(seed, Nseed_T + 64)，拆分序
  seed_PQ(头 64B d‖z) ‖ seed_T(尾 Nseed_T)——keygen KAT 钉死。
- **曲线标量**（§3.1/§3.3）：P 系 rejection sampling（Nseed_T 128/48 池、
  `from_bytes` 拒 0 与 ≥n 后自愈续采样）；X25519 恒等（Nseed_T 32）。
- **Combiner（本轮核心修正，见下节）**：SHA3-256(ss_PQ ‖ ss_T ‖ ct_T ‖
  ek_T ‖ label)。
- 六入口 `hybrid_generate/public_from_seed/import/export_der/encapsulate/
  decapsulate`；encap 侧 ML-KEM 随机性走 ml-kem crate 内建 OS 源、传统侧
  ephemeral 走 getrandom；decap 侧 ct_PQ 篡改走 ML-KEM 隐式拒绝
  （FIPS 203 §7.3）。密钥材料零落 tracing（同前轮纪律）。

## 切片内修复：Combiner 公式（KAT 首轮 P256 不中的根因）

初版误写 X-Wing 形 `SHA3-256(ss_PQ‖ss_T‖ct_PQ‖ct_T‖label)`（含 ct_PQ、
缺 ek_T）——draft-03 §4 只声明「CG framework + SHA3-256」，combiner 精确
公式在 [HYBRID-KEMS]-09 §5.1.3/§5.5：

```
C2PRICombiner(ss_PQ, ss_T, ct_T, ek_T, label) = KDF(concat(...))
```

含**接收方公钥 ek_T**、**不含 ct_PQ**；Encaps/Decaps 两处同式。诊断链：
keygen/roundtrip 绿 + ss_T 经 python cryptography 独立 ECDH 证实 → 嫌疑收
敛到共享定义；15 种 combiner 变体 × 3 哈希离线枚举零命中 → draft 文本版
程序化解析（分页折行感知）逐字节 diff 三常量全 IDENTICAL（排除转录错）→
拉 HYBRID-KEMS-09 §5.1.3 得精确式，离线对 P256/X25519 两向量同时命中。
教训：**「identical to X-Wing」的正文表述与 CG combiner 定义冲突时以框架
定义与向量为准**；X-Wing 原式（含 ct_PQ 缺 ek_T）是 CK/CG 之外的特化。
已有沉淀 [kmac-newx-raw-x](../../../learnings/bugs/2026-11/2026-11-03-kmac-newx-raw-x-vs-tuplehash-encode-string.md)
同类（规范原文仲裁优于记忆）。

## shim 接线（三文件零新回调）

- `part01b.js`：`_zw_mlKemIsFamily` 三名并入（hybrid 与 ML-KEM 同
  usages/格式面，WICG modern-algos Hybrid KEM 域）。
- `part02a.js`：REG 三行（generateKey/importKey/encapsulate/decapsulate/
  exportKey + pub:1，格式同 ML-KEM 行）。
- `part02.js`/`callbacks.rs`：零改动——全部流量经既有 mlkem 入口按名分流
  （`crypto_asym.rs` 六入口开头 `is_hybrid(name)` 分流到 crypto_hybrid）。

## 语料对齐（wpt-data 本地陈旧副本 → pin rev 复位，非本地补丁）

本地 `WebCryptoAPI/util/helpers.js` 与 `generateKey/algorithm_registry.js`
系早期波次取数（hybrid 落上游前），缺三 hybrid 名——对上游锁定 rev
`cb6cc97a` 逐字节 diff 后整文件复位（上游两文件均已含三名单/三行；
helpers.js 唯一差异 = 三行名单，registry 同）。successes_Hybrid-KEM 靶面
Case 经 `assert_goodCryptoKey` 的 `registeredAlgorithmNames` 查名（helpers.js
:7 独立清单，与 registry 无关）——清单缺名即 expected undefined 全案红。
新 clone 取数走同一 pin，天然含对齐后文件，无本地漂移负担。

## 定向语料（d1s4 同款 test-guard 4/8/1800 直调 --json，filter 路径子串）

| filter | 账面 |
|---|---|
| Hybrid-KEM（generateKey 双族 + importKey） | **2736P / 0F**（exit=0） |
| encap_decap（bits + keys） | **774P / 0F**（exit=0） |
| hybridkem（serialization window） | **6P / 0F**（exit=0） |
| supports-modern | **482P / 0F**（exit=0） |
| getPublicKey | 152P / 8F——8F 全 Ed448 域外（拍板未注册，d1s3/d1s4 同款在案失败集；Hybrid 子测试全绿，d1s4 账面同位 20F 含 Hybrid，本轮消 12） |

host 单测（`cargo test -p zero-engine --lib hybrid_kem_tests`，5/5）：
keygen_matches_draft_vectors（P256/X25519/P384 ek 逐字节）/ decap_vectors_kat
（P256/X25519/P384 三组合 shared_secret 全账）/ encap_decap_roundtrip /
import_export_formats / import_validates_curve_point（P 系非曲线点拒绝、
X25519 任意 32B 收）。runner 单测 278P/0F/76I。

## 全量语料 + 锚对照（d1s4 → d1s5）

<!-- ANCHOR-SECTION -->

## 验证与门禁

<!-- GATES-SECTION -->
