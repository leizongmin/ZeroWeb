# ZeroWeb Media Host (`zero-media-host`)

> 媒体宿主进程（media-runtime-isolation 方案 C）— 独立进程承载解码与媒体时钟（C2 解码内核 + 自走时钟）

## 概述

`ZeroWeb Media Host` (`zero-media-host`) 是 ZeroWeb 的独立媒体宿主进程：把媒体解码与权威媒体时钟从渲染进程拆出到子进程。由渲染进程内 webview 的 `MediaHostClient`（`media_proxy`，`media-process` feature 门控）在首个媒体源登记时懒惰 spawn（image_decoder 先例，`--type=media`），每 webview 至多一个 host。动机：编解码器处理不可信媒体字节，独立进程把解码器漏洞与解码/时钟负载隔离在子进程地址空间内，不波及渲染主进程。

wire format 契约 = `zero-protocol` 的 `media` 模块（17 变体消息族，RFC §9 IF-001~006）：`IpcMessage` 信封 + bincode over stdin/stdout 管道（`PipeTransport`，4B LE 长度前缀，单帧受 64MiB 上限约束），请求/响应以信封 `id` 配对，fire-and-forget 仅限无回执面（`MediaReleaseSource` / `MediaShutdown`）。

## 主要功能

- **解码内核**（`kernel::MediaKernel`）— 源表 + 播放控制 + 时钟推进的纯函数面：`handle` / `advance` 注入单调毫秒时钟、出站消息追加到调用方向量，不做任何 IO——事件循环与 stdio 传输分立，单测以虚拟时钟直接驱动；播放语义与 registry（`zero-webview/video_registry`）同构，同构代码引用 `zero-media::pipeline` 的 A1 时钟单一真值
- **解码路由与配额准入** — play 时懒建解码器（`VideoTrackDecoder::open_media` 容器嗅探 + 独立探针读时长/首帧尺寸；伴生音轨链 A_VORBIS → A_OPUS → mp4/AAC；视频失败回落纯音频 symphonia → ogg opus）；单源 256MB 配额（`MEDIA_SOURCE_MAX_BYTES`）对登记 `total_len` 与实收字节双面封顶——实收超限拒绝并摘除源，不静默截断；A/V pair 走 audio clock 主时钟、纯视频走墙钟 tick、纯音频走游标；seek/replay/loop 轨重建链与 registry 同序（源字节恒留存，轨重建随时要读）
- **自走时钟事件循环**（`run_role_feed` / `RoleIo` / `FeedTick`）— 无播放中源时无限期阻塞等消息；播放中按内核唤醒截止（音频样本队列排空时刻 / 纯视频 16ms 泵量子）等待，超时 Idle 推进时钟，不做固定节拍轮询；状态迁移即时推 `MediaStateUpdate`，播放中以 250ms 心跳补充（shim timeupdate 节流同款）；帧内联 RGBA 推 `MediaFrame`（shm 传输面归后续切片）
- **回执 id 配对**（IF-002）— 请求响应以信封 `id` 配对：play 的即时 Playing 推送兼作回执，seek 回 `MediaSeekResult{request_id, resolved_ms}`；后续心跳/终态为自发推送面（id 0）；每源持单调 epoch（seek/replay 递增），旧代际推送由 renderer 侧丢弃
- **握手与能力真值**（IF-001 / DC-4）— 连接后首发 `MediaReady{protocol_version, capabilities}`，capabilities 报内核实际路由真值，随 `decode-av1` / `decode-h264` feature 门控增删，不虚报
- **stdio 双半传输**（`main.rs`）— stdin 读线程阻塞 recv → mpsc，主线程 `recv_timeout` 等消息（超时即 Idle——`IpcChannel` 的阻塞 recv 无超时面，是拆分根因）；stdout 写半承载回写；日志走 stderr（tracing，WARN 起）
- **feature gate** — `decode-av1` / `decode-h264` 转发 `zero-media`（默认关，D-RFC-2/3 获批面），headless/CI 走默认面

## 使用示例

`zero-media-host` 由 webview（`media_proxy`）自动 spawn，不面向终端用户直接运行：

```bash
# 手动启动仅用于调试（正常由 webview 经 stdin/stdout 管道 spawn；spawn 参数
# --type=media --instance-id=0 由 zero-protocol child_process_args 生成，本二进制
# 不解析命令行参数——启动后即进入事件循环并首发 MediaReady 握手）
cargo run --bin zero-media-host
```

codec 能力面与 `zero-media` 同 feature 门控（默认关）：

```bash
cargo build -p zero-media-host --features decode-av1,decode-h264
```

内存通道/测试侧经 lib 入口直接驱动同一角色循环：

```rust
use zero_media_host::run_role;

// 实现了 zero_protocol::IpcChannel 的通道（内存通道/既有通道面）：
// 握手首发 MediaReady → 命令处理/时钟推进 → 对端断开或 MediaShutdown 退出。
// 时钟取进程启动起的单调毫秒；生产 stdio IO 面见 src/main.rs。
run_role(&mut channel);
```

## 部署要求

由 `zero-webview` 的 `MediaHostClient` 固定使用；`media-process` feature 默认关——关闭时 registry 走 `zero-media` 进程内直调，行为零变化（启用时机归 C4/M4）。

webview 按以下顺序定位本二进制（与 image-decoder 的发现模式一致）：

1. 环境变量 `ZW_MEDIA_HOST_BIN`
2. `CARGO_BIN_EXE_zero-media-host`（cargo test 注入的集成测试二进制路径）
3. current_exe 同目录（测试二进制 `target/debug/deps/` 上溯 `target/debug/`）
4. `PATH` 兜底

接入纪律（IF-006，webview 侧状态机）：初始 spawn/握手失败（含协议版本不符）→ 永久错误面，本生命周期不重试、不内联回落；运行期连接失效 → 退避重启（1s/2s/4s，n≤3，握手成功重置计数），耗尽转永久错误面；host 回复的 `MediaError` 为播放级错误面，不置进程失败；显式 shutdown/teardown 回 Idle（reload 后全新 host）。

## 相关文档

- 协议契约：`crates/protocol/src/media.rs`（RFC `docs/specs/media-runtime-isolation-spec-rfc.md` §9）
- 客户端接入面：`crates/webview/src/media_proxy.rs`
- 目标与执行记录：`docs/goal/archive/media-runtime-isolation.md`（2026-10-25 收口归档）
