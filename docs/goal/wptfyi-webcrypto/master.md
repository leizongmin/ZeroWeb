# WebCrypto控制面

**2026-11-10 · Active**。DC-1 收账中：语料污染修复 + 周跑切 v3 全语料已提交（3c4cd6772）；coverage 分片收录对账打通后，r1 账面 WebCryptoAPI 65,109 / 70,920（较发布快照 26,779 净增 38,330，已超 31,000 达标线；修复后语料终账预期 ≈67.8k，对齐收口锚 67,845）。\
**卡点** 全语料本地单核满速需 >3600s：r4 于 2026-11-10 00:49:22 守卫杀树、空 JSON（`.acceptance/wptfyi-webcrypto/r4-finalize.log` 在案），7200 线已落 ci-shards（6a10ddaf8）与本地 Makefile 入口。终账载体已切换并在跑：本地 r5（7200s，systemd `zw-r5-run`，00:50 起，硬顶 02:50）+ CI coverage re-dispatch run [42659662102](https://github.com/leizongmin/ZeroWeb/actions/runs/42659662102)（7200 分片顶，concurrency 取代带 3600 旧顶的 run 42653928283）。\
**下一步** r5 或 CI 分片落账后读 `r5-finalize.log`，按 [24MB 楔死指纹](../../learnings/platform/2026-11/2026-11-03-webcrypto-24mb-wedge-cascade-fingerprint.md) 核分母（262 案 / ≈69.8k keys，塌缩即作废串行复跑），补 evidence 终账段并提交推送；CI 快照回写后做邻域无可归因回退核验。

[执行契约](../wptfyi-webcrypto.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 收账中 | 修复+接入已提交（3c4cd6772 分片收录对账、74954f492 / 6a10ddaf8 守卫 7200）；r4 空产出实证 3600 不足，r5（7200s）终账待落，evidence 终账段随账补 |
| DC-2 按簇收敛 | 已盘面 | 锚账 F 1,950 全部具名：Ed448 1,946 为 2026-11-02 拍板降 scope（MSRV 1.85 无可用签名实现；2026-11-10 生态复测无变化——ed448-goldilocks stable 0.9.0 无签名面、pre 线 2026-06 后未动），modern-algos 4 为 TurboSHAKE flake（learning 在案）；修复后语料口径余量 ~1.1k 为上游漂移与未覆盖路径，待 CI 快照逐路径归类 |
| DC-3 全量对账 | 待产物 | 分片收录口径净增 38,330 ≥ 31,000；CI re-dispatch run 42659662102（7200 分片顶）在跑，待有效产物 + 邻域核验 |

碰撞记录暂无。注意：`engine/src/js_dom_bridge/crypto_asym.rs` 在 zero-web 流写面内，DC-2 若涉及引擎改动先核对时序，避免同文件碰撞。
