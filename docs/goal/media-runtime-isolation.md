# 平台媒体解码与媒体运行时隔离

**版本**：v1.0 / 2026-10-20
**状态**：Active（**D-RFC-1~4 已批复**——方案 C 批准，RFC v1.9 §10 批复记录；**A1–A4 + C1 + C2 切片 1/2 + C3 已落地**（feature 默认关），M3 IN_PROGRESS，下一步 C4/M4 终验）
**父目标**：[zero-web](zero-web.md) / **控制面**：[master](media-runtime-isolation/master.md)

## 0. 执行摘要与 Mission

测量现有媒体 CPU、帧复制、同步和权限成本，再评估平台硬件解码与独立媒体运行时。保持 HTMLMediaElement 可读状态、时钟、seek 与故障恢复语义。

依据：2026-10-20 用户要求按 [Ladybird 建议](../research/research-ladybird-progress-2026-10-20.md)立项；媒体进程是待证据支持的方案，不是本轮已确认架构。

## 基线与 Support Envelope

静态基线：`crates/media/src/lib.rs` 是解码/播放底座，`crates/webview/src/webview.rs` 与 renderer 承载页面语义；现有媒体 goal 已归档。新的硬解/进程隔离运行基线 **NOT_RUN**，不把历史 fixture 验收当长视频生产结果。

范围：现有格式真实能力、CPU/拷贝/A/V 同步、平台解码来源、状态代理/时钟与解码进程故障；sandbox 权限和 CPU 回退。

边界：复用 [media-playback](archive/media-playback/master.md)、[media-audio](archive/media-audio/master.md)、[media-elements](archive/media-elements/master.md)，不重做媒体元素语义。平台 SDK 能力、codec 许可与分发策略启动时核查；不自动改变许可证或引入 FFmpeg/新 codec。

碰撞：按 [run-rules §9–11](../rally/run-rules.md)协调 media/engine/webview/protocol/renderer/compositor 与 Android；协议改动同步消费者，不自动扩大平台支持承诺。

## Done Criteria / 验收场景（P2）

| ID | 条件与动作 | 期望与验证 |
|---|---|---|
| DC-1 / MEDIA-01 | 固定视频、播放条件、系统能力 | CPU/复制/掉帧/同步报告有版本与来源；能力查询不虚报。真实媒体采样与能力对照 |
| DC-2 / MEDIA-02 | play/pause、seek、换轨、导航并读时钟/状态 | 页面同步可读状态和事件符合现有契约，不返回旧代际数据。拟新增代理/共享时钟测试 |
| DC-3 / MEDIA-03 | 故障注入终止解码进程或模拟解码失败 | 变成可观察播放错误，renderer/浏览器可用，重载恢复；真实隔离实例故障验收 |
| DC-4 / MEDIA-04 | 平台无硬解、device lost 或权限不足 | 能力真实，已有软件路径或明确错误符合所选契约；权限减少有真实受限操作证据 |
| DC-5 | RFC 决策与最终支持范围审计 | 实现有常驻回归和适用门禁；若证据不支持架构投入，须用户认可延期结果后收口，不能自行豁免 DC-2~4 |

## 里程碑与实施交接

| 阶段 | 文件/职责与依赖 | 产物与验证 |
|---|---|---|
| M1 | media 现有解码 → renderer → compositor 帧消费 | 长视频资源基线、复制链和权限表；平台能力核查 |
| M2 | 本 goal RFC，protocol 与媒体 host 接口 | 比较现状、平台解码、进程隔离；许可证/SDK/分发与回滚方案评审 |
| M3 | 确认切片的 media/宿主/协议消费者 | 能力查询、状态/时钟、seek与错误路径；默认不大规模重构 |
| M4 | 生产媒体/故障/资源验收 | 同视频配对数据，崩溃恢复与所有消费者验证 |

实现来源：现有 Rust 解码器优先；平台 SDK（例如 macOS VideoToolbox）仅作为 M2 候选，核查 ABI/许可/可用性后定。状态代理在 webview/renderer 页面边界，解码留 media 底座；拟新增进程服务复用 protocol。建议分批基线/RFC、单平台解码、隔离与恢复；每批保留现有软件路径作为可验证回滚点。

## 执行与 Document Control

`./scripts/rally-goal-media-runtime-isolation.sh` 直接执行启动 Rally，加 `--dry-run` 仅预览；统一 [启动说明](ladybird-followups/README.md)。本轮不运行长视频、解码故障注入或 SDK 安装。

入口稳定，master/evidence 更新，archive 只追加；启动后 `CONTINUE` / `BLOCK`；全部 DC 或用户明确调整的 DC 有证才 `DONE`。
