# F 面具名归因扫查（2026-11-02，M2 收口后 DC-5③ 前置）

对 [m2s4 锚](2026-11-02-m2s4-full.json.gz) 全部非 P 子测试做逐案族谱归因，
验证「每个非 P 都有明确去向、无散在 shim 缺陷红面」——DC-5③ 谱系零移交
的预闭合，同时排查是否存在**无依赖可修**的意外红簇（结果：无）。

## 归因闭环（81,851 keys = P 52,781 + F 29,066 + T 4）

| 族 | 案数 | F | T | 去向 |
|---|---:|---:|---:|---|
| stable-root（既有 stable 案，157 案） | 157 | **0** | 0 | 全绿——零回归底线面 |
| 旧 `.tentative.` 路径（上游已删） | 98（96 有 F） | 14,150 | 0 | 收口重定基退役（DC-5②），无回归义务 |
| modern-algos（新族） | 79 | 11,252 | 4 | D1 域门控（见下表） |
| secure-curves（新族） | 26 | 3,664 | 0 | D1 域门控（X448/Ed448 pre 版随拍板） |

**F 总数 29,066 = 14,150 + 11,252 + 3,664 逐位闭合；stable-root 零红。**
T 4 = Hybrid-KEM generateKey ×4（window/worker 双腿 × failures/successes，
modern-algos 族，域门控随 ML-KEM/curve448 拍板）。

## 新族 F 逐算法分布（D1 定序的预期 P 产出参考）

| 算法族 | F | 对应 deps 候选（[M1 deps evidence](2026-11-01-m1-deps-evidence.md)） |
|---|---:|---|
| ML-KEM | 2,952 | ml-kem 0.3.4 建议版 |
| AES-OCB | 2,514 | ocb3 0.2.0 |
| Ed448 | 1,908 | ed448-goldilocks 0.14.0-pre.15（pre 版随拍板） |
| X448/curve448 | 1,756 | x448 0.14.0-pre.12（pre 版随拍板） |
| KMAC | 1,416 | sha3-kmac 0.3.0 vs 手写叠 cshake（拍板两案） |
| ML-DSA | 1,080 | ml-dsa 0.1.2 建议版 |
| ChaCha20-Poly1305 | 936 | chacha20poly1305 0.11.0 |
| Hybrid-KEM | 432 + T 4 | 随 ML-KEM/curve448 |
| XOF（k12/turboshake/cshake/sha3/shake） | 1,074 | 官方 XOFs 家族（MSRV 1.85 全齐） |
| supports-modern | 140 | 域门控扩展面（随各域注册扩表） |
| argon2 / encap_decap / 其余 | 684 | argon2 0.6.0 / 组合域 |

## 消息签名扫查（异常排查）

域族 F 消息签名 147 个，除一类外全部为 `NotSupportedError: Unsupported or
missing algorithm / Unsupported hash algorithm` 及其 **assert_unreached/
级联** 形态（域未注册的可预期级联）——与 D1 域门控归因一致，无散在
shim 缺陷红面。

**唯一潜伏项（记录不修）**：cshake 50 F（`digest/transferred buffer` 族
子测试）表现为 `TypeError: Cannot perform ArrayBuffer.prototype.transfer
on a detached ArrayBuffer` 而非 NotSupportedError——归因 = 失败路径
（算法未注册 → 构造错误）存在对 `algorithm.name` 的**二次读**（name
getter 副作用回调二次触发 `buffer.transfer()`）。成功路径单次读已被
stable-root 全绿佐证（digest sha 族同型 transfer-during-call 子测试
全 Pass）。无修复动作：① 当前失败面预期必红（cshake 未注册，改错误
类型仍 F）；② D1 后成功路径接管，这些子测试走单次读路径自然翻绿；
③ 现库无任何「失败路径 + name getter」语料面（failures_* 族均纯对象）。
留作 M 末域切片回归观察点：若某域切片后出现该 TypeError 形态新红，
按失败路径单点收敛。

## 结论

- DC-5③ 归因闭环**预闭合成立**：81,851 keys 内每个非 P 具名归因到
  旧路径退役（14,150）或 D1 域门控（14,912 + T 4），无外挂挂账、
  无无依赖可修红面。
- M2 无依赖面收口结论经本扫查复核维持：余下工作全部待 D1。
