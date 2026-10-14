# 播放时钟、HTMLMediaElement 与帧注入

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 play 已调用但时间冻结、视频帧不更新、seek/ended 重播异常和首帧元数据丢失。VideoClock 是媒体时间读接口，VideoPlayer 推进解码与帧调度；WebView 注册表持源、播放器和音轨，engine shim 管理 HTMLMediaElement 状态与事件，Painter 只读取同 key 的像素资源。

### User Task Navigation
- 时间与背压：`crates/media/src/clock.rs`、`player.rs`；生产注册与泵：`crates/webview/src/video_registry.rs`。
- settle：`crates/webview/src/async_load.rs`；页面状态机：`crates/engine/src/js_dom_shim/part06.js`；上屏：`crates/engine/src/paint/painter/text.rs` 的 paint_video_element。
- player/registry 的细节拍、seek、ended 与 A/V 测试是常驻判据，源码存在不代表当前产品路径已运行验证。

## Gotchas
- tick 用注入墙钟的饱和差乘 playbackRate 推进；暂停不推进，落后多帧只返回最新可展示帧，不能把 tick 次数当视频解码帧数。 (`crates/media/src/player.rs`)
- 未来帧必须 un_read 回队首且时间继续前进；只有非 EOF 的真实缺帧等待才回持 presented_pts，混合两个 None 分支会在 16ms 泵下永久钉住首帧。 (`crates/media/src/player.rs`, `git:ddd51e1d2cb64cfe73d675dc030106ac350e7cf8`)
- VideoPlayer::play 在 Ended 后只重置位置/状态，不重建已耗尽 decoder；注册表的重播分支必须提供新 decoder/reset，不能仅把 state 改 Playing。 (`crates/media/src/player.rs`, `crates/webview/src/video_registry.rs`)
- seek 在已知 duration 内钳目标，清时钟锚与已呈帧但保持播放/暂停状态；播放器 cue 路径可能呈现实际关键帧落点，不能只用设置的 currentTime 证明精确呈现位置。 (`crates/media/src/player.rs`, `crates/media/src/decode.rs`)
- 播放速率只接受有限正值并封顶 16，零、负数和 NaN 忽略；VideoClock 注释不能作为零速、反向播放或 preservesPitch 已真实实现的证明。 (`crates/media/src/player.rs`, `crates/media/src/clock.rs`)
- registry_key 先剥 query/fragment 再调用 image_resource_key，而 settle/Painter 保留原URL；带 query 的源可能首帧与持续帧落不同键。同归一化源的多个元素共享条目，不是天然独立播放器。 (`crates/webview/src/video_registry.rs`, `crates/webview/src/async_load.rs`, `crates/engine/src/paint/painter/text.rs`)
- play 解码失败不消费源字节，成功才移出 sources 并保留所需伴音/元数据；缺源 provider 的取字节必须在 registry 锁外，避免同步网络取源阻塞持锁泵。 (`crates/webview/src/video_registry.rs`, `git:803c912fcb8342ebf5b0191d8ac0a1b2e0a68444`)

## Architecture
- settle 先登记视频/音频源再探 duration、尺寸和首帧，并将 RGBA 以绝对 URL key 插入 ImageCache；资源事件还须经过排序与宿主派发，首帧像素到齐不等于 HTML load 事件已经结束。 (`crates/webview/src/async_load.rs`)
- A/V tick 先推进伴音轨，再让 VideoPlayer::sync_to_media_time 对齐音频解码游标；该方法只前进不回退，纯视频用墙钟，音频解码时钟不是硬件已消费时钟。 (`crates/webview/src/video_registry.rs`, `crates/media/src/player.rs`, `crates/media/src/audio.rs`)
- 图片上屏在 paint_video_element：content:url 替换优先，其次有缓存帧的 src，否则 poster；持续帧按注册表归一化键注入，调试只看到poster或首帧时需核对该键与Painter的完整URL键是否一致。 (`crates/engine/src/paint/painter/text.rs`, `crates/webview/src/video_registry.rs`, `crates/webview/src/async_load.rs`)
- registry clear 同时清源、播放器、音轨、元数据与 feature 内 MSE 状态，但不在该函数清 ImageCache；不能把释放播放状态等同于全部像素资源已释放。 (`crates/webview/src/video_registry.rs`)

## Decisions
- HTMLMediaElement 事件归 engine shim，VideoClock/VideoPlayer 不直接派事件；无桥或不可解码源可回落近似驱动，页面 IDL 的时间前进本身不能证明真实解码/上屏链已接通。 (`crates/media/src/clock.rs`, `crates/media/src/player.rs`, `crates/engine/src/js_dom_shim/part06.js`)
