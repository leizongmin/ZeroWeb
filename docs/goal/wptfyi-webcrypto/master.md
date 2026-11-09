# WebCrypto控制面

**2026-11-10 · Active**。DC-1 收账完成：语料修复 + derived_bits_length 楔死定位与修复（deriveBits `length` 缺 [EnforceRange] 入口归一——2\*\*32+8 曾穿透到 host KDF 致 PBKDF2 512MB dkLen 不可终止；part02.js 入口归一 + ECDH/X25519/KDF/Argon2 分支收敛 + part53.rs M6 契约同步 −8→TypeError）+ 全量重跑 r6 终账 **70,627 keys = P 67,877 + F 2,750 + T 0**（3m58s，exit 0）；对 r1 与 11-03 收口锚均**零回归**（drift 恰为 failures.js 修复显形 800 F + enforceRange 32 P，分母逐位对平）。coverage 试跑（ce5ca39646 / 792c7512e3 两 basis，guard 过）净增 **41,098**（67,877 − 发布快照 26,779）≥ 31,000 达标线。\
**卡点** CI coverage 已于修复后 HEAD re-dispatch：run [42667268541](https://github.com/leizongmin/ZeroWeb/actions/runs/42667268541)（4482feb56，concurrency 取代旧二进制 run 42659662102）；DC-2 新显形 800 F（generateKey failures 面）待逐路径归类。\
**下一步** 跟踪 run 42667268541 产物并核验 crypto 分片非空；DC-2 余量归类（failures 面 800 + Ed448 面 1,950 = secure-curves 1,946 + getPublicKey 4，均 NotSupportedError；r1 的 turboSHAKE 4 T 已在 r6 清零）；CI 快照回写后做 DC-3 邻域无可归因回退核验；分片/本地守卫线已随根因修复回落 3600（r6 实测 3m58s，~15 倍余量，本轮落账）。

[执行契约](../wptfyi-webcrypto.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 收账完成 | r6 终账（修复后语料 + EnforceRange 二进制）70,627 keys / T 0 / 3m58s；锚对照 vs r1、11-03 零回归；楔死取证链与修复见 [evidence 2026-11-09 第五节](evidence/2026-11-09-dc1-corpus-repair-and-baseline.md) |
| DC-2 按簇收敛 | 已盘面 | F 2,750 全部具名：Ed448 面 1,950（secure-curves Ed448 1,946 + getPublicKey 4，均 NotSupportedError；2026-11-02 拍板降 scope，MSRV 1.85 无可用签名实现，2026-11-10 生态复测无变化）+ generateKey failures 面新显形 800（failures.js v3 内容修复后枚举恢复，与 deriveBits 同根因，待修） |
| DC-3 全量对账 | 待产物 | 分片收录口径净增 41,098 ≥ 31,000（coverage-r6 两 basis 试跑）；CI run 42667268541（修复后 HEAD）在跑，待有效产物 + 邻域核验 |

碰撞记录暂无。注意：`engine/src/js_dom_bridge/crypto_asym.rs` 在 zero-web 流写面内，DC-2 若涉及引擎改动先核对时序，避免同文件碰撞。
