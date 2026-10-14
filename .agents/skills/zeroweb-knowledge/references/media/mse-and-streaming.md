# MSE、流式输入与 feature 覆盖

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

定位 MediaSource 能创建但 append 无帧、流尾提前 ended、续喂被冻结和双轨能力探测差异。SourceBuffer 将 ArrayBuffer/View 转 base64 交给注册表，独立 video/audio 增长缓冲供 symphonia fMP4 reader 和 H.264/AAC 解码；本主题不把最小流式面描述成完整 MSE。

### User Task Navigation
- feed 读侧及续喂测试：`crates/media/src/mse_feed.rs`；帧背压：`crates/media/src/player.rs`。
- 源登记、容量与音轨：`crates/webview/src/video_registry.rs`；JS 类型表和事件：`crates/engine/src/js_dom_shim/part06.js`；feature：`crates/media/Cargo.toml`、`crates/webview/Cargo.toml`。
- feed、registry 和 shim 的常驻断言只作来源，未运行 append、DASH、音频或产品验收。

## Gotchas
- MSE 模块与 WebView 桥都受 decode-h264 门控；shim 构造器存在不代表解码链存在，isTypeSupported 先检查 mseCreate 桥再判断 MIME/codec。 (`crates/media/src/lib.rs`, `crates/webview/src/video_registry.rs`, `crates/engine/src/js_dom_shim/part06.js`)
- feed 的 None 同时表示初始化段不足或已写边缘等待，只有 at_edge 且 ended 才是真 EOF；静态 decoder 的 is_exhausted 恒真不能套到 MSE。 (`crates/media/src/mse_feed.rs`, `crates/media/src/decode.rs`)
- append 会清共享 ended，但读侧 sticky eof 还必须在缓冲增长时解除；仅改 MediaSource readyState 或共享旗标不能恢复已闩住的 next_frame。 (`crates/media/src/mse_feed.rs`, `git:2b5095418fdc2ecf13a094ecb9e491e150c27029`)
- symphonia reader 的长度快照不会自动见到增长，在读边缘重建 reader 并按最后交付 PTS 跳过旧包；视频与音频有各自游标/跳过相，不能只修一个共享 ended 标志。 (`crates/media/src/mse_feed.rs`)
- 当前 isTypeSupported 音频声明限 AAC-LC 的 mp4a.40.2/aac，addSourceBuffer 的受理表更宽，mp4a.40.x 可创建但不保证可解码；HE-AAC 受理后静音降级不应被写成真实音频支持。 (`crates/engine/src/js_dom_shim/part06.js`)
- 能力表采用 MIME 前缀与 codec 正则，不是完整 MIME/RFC6381 验证器；普通 canPlayType 的 H.264 profile/level 判定不能推到 MSE 的 avc 前缀判断。 (`crates/engine/src/js_dom_shim/part06.js`)
- SourceBuffer 的轨章决定独立 feed 路由，audio 桥缺席时 append 失败而非把音频字节送入 video feed；两个缓冲的容量上限各自检查，不能当成整个页面共享总配额。 (`crates/engine/src/js_dom_shim/part06.js`, `crates/webview/src/video_registry.rs`)
- registry 用饱和加法检查增长总量后拒绝越限，feed 自身只是增长 Vec；容量保护依赖调用经过 registry，区间淘汰/SourceBuffer.remove 不是当前实现。 (`crates/webview/src/video_registry.rs`, `crates/media/src/mse_feed.rs`)

## Architecture
- 共享 MseFeedHandle 用 Arc/Mutex 保 append 与读视图一致，FeedSource 有独立读游标；moof/mdat 的语义解析委托 symphonia，不自行拼装 MP4 box 解析器。 (`crates/media/src/mse_feed.rs`)
- append 设置 updating 后以 task 执行桥写入，成功派 update/updateend，失败派 error/updateend；ended→open 只在成功追加后发生，不能由 updateend 单独推断解码成功。 (`crates/engine/src/js_dom_shim/part06.js`)
- 首次 play 惰性创建视频 feed，音频已有字节时创建伴音条目；MSE 音频仍 playing 时视频跟随其游标，音轨停止/失败时可退回视频墙钟，不能一律用存在音轨作为主时钟条件。 (`crates/webview/src/video_registry.rs`)
- append-after-end 的 feed 解闩与已 Ended player 的重建是两层恢复；续喂读侧能产帧不代表播放器状态已自动复活，须沿 registry 的 play/reset 分支核对。 (`crates/media/src/mse_feed.rs`, `crates/media/src/player.rs`, `crates/webview/src/video_registry.rs`)

## Decisions
- SourceBuffer.buffered 是最小区间投影而非精确分段缓存，remove/appendStream/changeType 和完整 detach/淘汰语义仍未齐全；不以 ABR 追加能播放推断完整 MSE API 已实现。 (`crates/engine/src/js_dom_shim/part06.js`, `crates/webview/src/video_registry.rs`)
