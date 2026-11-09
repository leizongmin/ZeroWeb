# WPT 95% · 11 媒体与实时通信

**状态** Active\
**立项** 2026-11-09，按用户要求扩展到 95% 冲刺，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-media/master.md)\
**共同计分与运行规则** [95% 冲刺计划](../compat/wptfyi-95-plan.md)

## 基线与目标

11 月 7 日 [全量快照](../compat/wptfyi/coverage-2026-11-07.md) 已通过 1,402 / 16,045 项，缺口 14,643 项。`webaudio` 缺 4,485 项，`webcodecs` 缺 3,916 项，`webrtc` 缺 2,414 项。复用 [媒体播放归档](archive/media-playback.md)、[Web Audio 归档](archive/media-audio.md) 与 [媒体验收判据](../../.agents/skills/zeroweb-verify/features/media.md)。

同口径净增 **10,000** 项，达到至少 **11,402** 项通过。固定 revision 的净增与当前周跑的上游漂移分别报告。

## 范围与协作

计分目录为 `webaudio/`、`webcodecs/`、`webrtc/`、`webrtc-stats/`、`webrtc-encoded-transform/`、`media-source/`、`mediacapture-streams/`、`mediacapture-record/`、`mediacapture-image/`、`media-capabilities/`、`encrypted-media/`、`audio-output/`、`video-rvfc/`、`webvtt/`。

仅计本入口所列目录。HTMLMediaElement 位于 `html/` 的测试由 05 记账。媒体解码、播放和页面语义按既有分层处理。安全上下文复用 07，自动化和权限测试复用 10。硬件与编解码器缺失单列环境结果。 共享文件按 [Rally 运行规则](../rally/run-rules.md) 分配归属。等待共享修复时，继续独立语料导入、对账和失败分类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。用守卫跑 `testharness-webaudio` 和媒体基线，再从 WebCodecs、MSE、WebRTC 中各选代表样本。盘点可用的实际解码器、音频输出与可重复的虚拟采集环境，核对播放、帧输出、事件和资源回收。 归档固定 revision、当前 commit、预期与实际执行路径，以及按路径和变体的差额。
- **DC-2 · 按簇收敛**。先收敛可稳定复跑的 Web Audio、解码和 MSE，再补录制、采集、WebRTC 协商和统计。对帧序、时间戳、轨道状态、错误、权限与结束释放分别保留断言。用例允许的虚拟设备仍须经过产品采集路径。 修复随附常驻 WPT 或等价回归。
- **DC-3 · 全量对账**。接入周跑并验收净增至少 10,000 项，邻域无可归因回退。完成守卫下的构建、测试与 clippy，保存有效分片和全量结果，再归档。

每轮更新控制面与证据，最后一行按 [共同计划](../compat/wptfyi-95-plan.md#无人值守运行) 输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。
