# WebCrypto控制面

**2026-11-10 · Active**。DC-1 收账中：语料污染修复 + 周跑切 v3 全语料已提交（3c4cd6772）；coverage 分片收录对账打通后，r1 账面 WebCryptoAPI 65,109 / 70,920（较发布快照 26,779 净增 38,330，已超 31,000 达标线；修复后语料终账预期 ≈67.8k，对齐收口锚 67,845）。\
**卡点** r4 终账与 CI 验证产物未落：r4（守卫硬顶 2026-11-10 00:49:22）由 systemd `zw-r4-finalize` 自动收账至 `.acceptance/wptfyi-webcrypto/r4-finalize.log`；CI coverage-only run [42653928283](https://github.com/leizongmin/ZeroWeb/actions/runs/42653928283) crypto 分片在跑。\
**下一步** 读 r4-finalize.log，按 [24MB 楔死指纹](../../learnings/platform/2026-11/2026-11-03-webcrypto-24mb-wedge-cascade-fingerprint.md) 核分母（262 案 / ≈69.8k keys，塌缩即作废串行复跑），补 evidence 补段并提交推送；CI 快照回写后做邻域无可归因回退核验。

[执行契约](../wptfyi-webcrypto.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | 收账中 | 修复+接入已提交（3c4cd6772 分片收录对账、74954f492 本地守卫 3600s）；evidence/2026-11-09 补段待 r4 终账 |
| DC-2 按簇收敛 | 已盘面 | 锚账 F 1,950 全部具名：Ed448 1,946 为 2026-11-02 拍板降 scope（MSRV 1.85 无可用签名实现；2026-11-10 生态复测无变化——ed448-goldilocks stable 0.9.0 无签名面、pre 线 2026-06 后未动），modern-algos 4 为 TurboSHAKE flake（learning 在案）；修复后语料口径余量 ~1.1k 为上游漂移与未覆盖路径，待 CI 快照逐路径归类 |
| DC-3 全量对账 | 待产物 | 分片收录口径净增 38,330 ≥ 31,000；待最新周跑有效产物 + 邻域核验 |

碰撞记录暂无。注意：`engine/src/js_dom_bridge/crypto_asym.rs` 在 zero-web 流写面内，DC-2 若涉及引擎改动先核对时序，避免同文件碰撞。
