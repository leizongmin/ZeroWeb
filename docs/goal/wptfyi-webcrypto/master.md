# WebCrypto控制面

**2026-11-10 · Active**。DC-2 收账完成：generateKey 数值成员 [EnforceRange] 修复（AES `length` unsigned short / HMAC `length`、RSA `modulusLength` unsigned long 三处入口归一，与 deriveBits 同根因类；part53.rs 契约块 ±2^32 → TypeError）+ 全量 r7 终账 **70,627 keys = P 68,677 + F 1,950 + T 0**（墙钟 ~4 分钟量级，与 editing 流探针并发，不作精确账面）；对 r6 锚对照 **REGRESSED 0 / FIXED 恰 800 / FLIPPED 0 / DRIFT 0**，余 F 1,950 全为 Ed448 降 scope 面（NotSupportedError）。coverage 两 basis 净增 **41,898**（68,677 − 发布快照 26,779）≥ 31,000 达标线。\
**卡点** DC-3 待 weekly run [42667268541](https://github.com/leizongmin/ZeroWeb/actions/runs/42667268541) 收尾（crypto 分片已回并核验：产物非空，**与本地 r6 逐 key 一致** 70,627 keys 0 漂移）；该 run 为 deriveBits 修复 HEAD（4482feb56），未含本批 generateKey 修复。\
**下一步** run 收尾后 DC-3 全量对账（快照回写、邻域无可归因回退核验；如需含 generateKey 修复的 CI 账可再 dispatch 一次）；挂账：HMAC generateKey 显式 `length: 0` 现为块大小密钥、spec 为 OperationError（corpus 未覆盖，待补契约后另批处理，见 [evidence 2026-11-10 第五节](evidence/2026-11-10-dc2-generatekey-enforcerange.md)）。

[执行契约](../wptfyi-webcrypto.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 收账完成 | r6 终账（修复后语料 + EnforceRange 二进制）70,627 keys / T 0 / 3m58s；锚对照 vs r1、11-03 零回归；楔死取证链与修复见 [evidence 2026-11-09 第五节](evidence/2026-11-09-dc1-corpus-repair-and-baseline.md) |
| DC-2 按簇收敛 | 收账完成 | F 全部具名并处理：generateKey failures 面 800 已修（[EnforceRange] 三处入口归一，r7 转 P）+ Ed448 面 1,950 降 scope（2026-11-02 拍板，MSRV 1.85 无可用签名实现，2026-11-10 生态复测无变化）；r7 对 r6 零回归见 [evidence 2026-11-10](evidence/2026-11-10-dc2-generatekey-enforcerange.md) |
| DC-3 全量对账 | 待产物 | 分片收录口径净增 41,898 ≥ 31,000（coverage-r7 两 basis 试跑）；CI run 42667268541 在跑（crypto 分片已回核验），待全 run 收尾 + 快照回写 + 邻域核验 |

碰撞记录（2026-11-10 02:14–02:31）：quota 重置触发同目标 re-dispatch，双 webcrypto session 并发（f2fcd740 / 010f426b），r7 三链并发同写一账产出逐位一致；02:31:38 commit 4cba77f7e 为两 session 工作合并账（shim×3 + part53 契约 + evidence + 本控制面），已推 origin。冗余链已清理，coverage 双跑目录已清场重跑收口。注意：`engine/src/js_dom_bridge/crypto_asym.rs` 在 zero-web 流写面内，DC-2 若涉及引擎改动先核对时序，避免同文件碰撞。
