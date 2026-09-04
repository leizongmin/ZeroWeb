# M3 fixture-mounted runner 播放面切片与 M1/M2 早期切片过程记录归档（只追加不修改）

**入口文档**: [../media-playback.md](../media-playback.md) | **控制面**: [../master.md](../master.md)
**归档日期**: 2026-09-04（治理切片——master.md 下一步计划的切片 1~11 落地明细与当前状态块 M1a~M2b 历史过程段移入本档；头链保留切片 5~12 摘要（最新态）；每片证据/单测在各自 commit 与 evidence/ 不动；DC 审计/缺口清单/决策表不动）

---

## 当前状态块历史过程段（M1a ~ M2b，2026-09-01，原文）

**M1a 已落地**（2026-09-01）：`crates/media`（`zero-media`）解码管线全通——
`VideoDecoder::open_webm_vp9` → 逐帧 `next_frame()` → `DecodedVideoFrame`（RGBA +
pts_ms）；fixture 48 帧全解、PTS 单调、首帧与 ffmpeg 7.1.5 rawvideo 参照**逐字节一致**
（探针实测），`VideoClock` trait（M2a 对接点）已定义。
**M1b 已落地**（同日）：首帧上屏通路全链打通——painter `paint_video_element`
（src 哈希 → ImagePrimitive，仅当解码像素已注入时发图元，占位行为零回归）+
pipeline `build_img_intrinsic_all` video 段（解码尺寸 → NodeId 固有尺寸表）+
layout-engine replaced sizing 白名单 +video + wpt-runner `load_video_first_frames`
（真实 fixture 首帧解码注入 ImageCache）。e2e 双测常驻（正例：真实帧像素上屏、
RGB 均值锚点；负例：不可解码 src 保持占位）。
**M2a 已落地**（同日）：`zero-media::player::VideoPlayer`——`VideoClock` 的帧率
驱动实现（PlayerState Ready/Playing/Ended 状态机 + play(now)/pause + tick(now)
位置推进与帧调度（pts ≤ position 呈现最新帧）+ playback_rate clamp + reset 重播）；
调用方注入单调时钟（rAF event loop P1a 挂点），单测 6 件常驻。
**M2a 切片 2 已落地**（同日）：duration 真值注入链全通——webview async_load 对
video 资源 fetch 成功后经 `probe_video_media_meta`（容器时长 + 首帧固有尺寸）真值化
`ResourceElementEvent`（新增 `media_duration_ms`，webview → zero-media 依赖）→
`script_commit_resource_element_state` 增第 6 参 → shim `_zwSettleResourceKey
.durationMs` → `_zwMediaLoadSequence` 以真值（ms→spec 秒）设 `duration`；无真值
（非 webm-VP9/headless 路径）回落定值 600——**testharness-media 372P/0F/0T/41PF
维持（零回归）**，单测 `test_media_duration_truth_injection_m2a` 3 断言组常驻。
**M2a 切片 3 已落地**（同日）：`videoWidth`/`videoHeight` IDL getter（part04 get
trap，VIDEO-gated 读 `_resourceStates.width/height`——切片 2 探针真值；未 settle 恒 0
per spec 元数据未就绪）+ has 白名单 tag-gated 分支（part05，`'videoWidth' in audio`
恒 false——接口成员归属面）。单测 `test_media_video_width_height_truth_m2a` 5 断言组
常驻；testharness-media 372P/0F/0T/41PF 维持（零回归）。**M2a 切片 4 已落地**（同日）：生产侧帧注入——`probe_video_media_meta` 扩为返回
首帧 RGBA（时长/尺寸/像素三真值一体），settle 时经 `ImageData::from_rgba` 注入
`webview.image_cache`（`ImageKey(image_resource_key(abs_url))`——与 painter
`image_resource_key(src, document_url)` 解析后同键，img/canvas 同款两段式）；非 webm
负例不注入（渲染占位零回归）。e2e 双测常驻（`video_settle_injects_first_frame_and_
truth_m2a` 真实 webm fixture 驱动 + `video_settle_non_webm_stays_headless_and_
placeholder` 负例）；webview 662 全绿。**生产侧首帧出图闭环达成。
**M2a 切片 5a 已落地**（同日）：`VideoPlayerRegistry`（webview `video_registry`
新模块）——`register_source`（settle 登记）/`play`（懒建 player，源未登记 no-op）/
`pause`/`current_time`/`duration`/`is_playing` 真值查询/`tick_all(now, ImageCache)`
（渲染泵推帧 + painter 同键注入 + changed 返回）/`release`（导航/元素移除资源释放）。
WebView 持 `video_players()` Arc 句柄；async_load settle 自动登记源字节（e2e 扩断言：
settle 后 `play` 即成功）。单测 4 件常驻；webview 666 全绿。**M2a 切片 5b 已落地**（同日）：三段接线全通——
① webview `register_video_bridge_callbacks`：五回调族（play/pause/current_time/
duration/is_playing，`Fn(&[String]) -> String` 契约）+ `__zwVideoBridge` JS 门面
（shim feature-detect 单点）；
② tab_js_worker `SetVideoPlayers` 命令（SetFetchHandler 同款 late-injection 模式）
+ `TabJsWorkerHandle::set_video_players`；tab_worker WebView 构建后注入
`wv.video_players()` Arc（settle 写入与桥读取同一实例）；
③ 帧泵：tab_worker 1ms 事件循环节拍——`is_any_playing` 快速门（无播放零开销）→
`tick_all(pump_epoch.elapsed(), image_cache)` 注帧 → changed 时增量重渲染 + snapshot。
shim 侧（part03/part04）：`play()` 桥接（bridgeSrc = IDL src getter 同源解析绝对 URL，
bridgeOn 标记）/`pause()` 桥停/`currentTime`/`duration` getter 桥真值优先——无桥环境
回落 headless（零回归）。测试：webview 桥 e2e（V8 sandbox + 真实 fixture roundtrip）
+ engine shim 契约测试（JS stub 桥：play 传绝对 URL / currentTime 1.25 / duration 2 /
pause 记录）；engine 2539 / webview 667 / browser 411 全绿；testharness-media 372
基线维持。**M2a 全部切片收口**。
**M2 切片 C 色度精化已落地**（2026-09-01，commit `f05493a07`）：`zero-media` 解码层
色彩面全对齐——
① **WebM Colour 解析**：`ColorSpace`（matrix/full_range）开流时经
`TrackEntry.Video.Colour` 解析一次（matroska-demuxer `colour()` API）；identity（GBR）
通道直传、BT.709/BT.601 矩阵选择、limited [16,235] 值域归一（ITU-R BT.601-7 §2.5/
BT.709-6 §2.5 标准形）；缺省：matrix None→BT.709 声明语义、range 非 Full→limited。
② **色度采样索引坍缩根因修复**：定点索引除数误用 chroma 维（应为 luma 维）——4:2:0
全行/列只采样前两个色度样点；旧锚点 153.5 为「全范围误释 + 该缺陷」叠加伪值，修正后
RGB 均值 123.3 与 ffmpeg swscale RGBA 参照对齐。
③ **值域误释修复**：声明 limited 的源此前按 full-range 数学转换（亮度 +25% 失真）。
④ **reftest-upstream 实证**：replaced-element-003（2x2-green.webm = identity+full，
位面真值 (0,127,0) vs ref #008000 差 1 ≤ fuzzy 0-30）✓ PASS——**R3881 以来唯一净
delta 的 false-pass unmask 案收口**；13950→13981/16730 = 83.6%（+31 净涨零回归）；
product-smoke 23.37% 逐字节同值（welcome 无 video，非回归）+ struct PASS。
⑤ 测试锚点同步：zero-media 新增 2 件（identity 直传位面真值 + limited 不削顶），
138-168 窗口更正 108-138（media 单测 + M1b e2e）。

**M2c 后续切片 A+B 已落地**（同日）：播放管线接 AudioSink 的生产侧全通——
① **settle 登记接通**（核心缺口修复）：`async_load::poll_element_resources` 对
video 源 `register_source` + `register_audio_source` 双登记、audio 源音频登记——
切片 5a 提交说明声称「async_load settle 自动登记源字节」但代码中从未存在生产调用方
（`register_source` 仅测试可达），宿主桥 play 在真实浏览器路径恒返 false 回落
headless；本片补全后 settle → 桥 play → 帧泵/音频泵全链生产可达（e2e
`media_settle_registers_playback_registry_m2c` 三面断言：webm video play 即成功 /
mp3 audio 音频 play 成功 / oga-opus 不登记回落）。
② **资源生命周期（DC-4）**：`prepare_document_state` 清空注册表（导航离开释放
player/解码器/源字节）+ `register_audio_source` 留存源字节（seek 重建解码器所需）+
registry `clear()` 单测。
③ **seek 追赶区静默**：`AudioEntry.skip_until_ms` 丢弃线——音频解码器单向流，seek
重建后前向解码至 target，target 前采样不入 sink（spec precise-seek；断言见
audio chain 单测）。
④ **增益联动全接**：shim play() 起播同步 `setGain`（play 前设置的 volume/muted 生效）
+ volume/muted IDL setter 桥推（切片 A 未提交工作树内容收编）；tab_worker 音频泵
（audio_advance_all）写入不计入重渲染触发（音频不改帧）。
⑤ **renderer 多进程路径对齐**（master.md 下一步 #2 兑现）：renderer js_worker 增
`SetVideoPlayers` 命令 + `set_video_players` 句柄 + runtime WebView 初始化后注入
`webview.video_players()`——镜像 tab_js_worker 切片 5b 同款，`__zwVideoBridge`
真值面两路径一致。
测试：webview 674（+6）/ engine 2540 / make test 18636 全绿；testharness-media
372P/0F/41PF 基线维持；clippy 零警告。
**M2c 音频解码面已落地**（同日）：`zero-media::audio_decode::AudioDecoder`——
symphonia probe 自动识别容器/编码 → f32 交错 PCM 逐包输出（`copy_to_vec_interleaved`
跨格式直转，值域 [-1,1] 对齐 AudioSink 契约）；损坏包跳过（DecodeError 面继续）；
依赖 `symphonia = { 0.6, default-features = false, features = [mp3, ogg, vorbis, pcm] }`
（纯 Rust 零 C 依赖，路线 C 约束保持）。**opus 选型注记**：symphonia 0.6 无 opus
解码器（libopus 为 C 依赖）——`sample-ogg-opus.oga` 不在 M2c 面，留待后续选型评估
（新增 vorbis fixture 补齐 ogg 容器面）。全链 e2e 双件常驻：`audio_mp3_decode_to_
nullsink_zero_crossing_chain` / `audio_ogg_vorbis_decode_to_nullsink_chain`——fixture
（440Hz sine）→ 解码 → NullSink 过零率 ≈880（2×频率锚点）+ 采样数 ≈2s 容差面；
media 30 全绿。**余：A/V 同步（media-audio M2，audio clock 主时钟）+ 播放管线接
AudioSink（video play 时同步音频面——M2c 后续切片）**。
**M2 切片 D+E 已落地**（同日，A/V 同步——media-audio M2 契约兑现）：
① **切片 D（webm 双轨伴生音频面）**：`zero-media::av_decode::open_webm_audio_track`
——Matroska demux A_VORBIS 轨 + CodecPrivate 三段 Xiph 头拆解 → OGG 页重封装
（RFC 3533 纯字节操作，零 C 依赖，路线 C 保持）→ symphonia ogg/vorbis 解码 f32 PCM
（与独立音频面同契约）。granule 语义注记：symphonia ogg reader 按页 granule 对包做
端部裁剪，granule 小于包累计时长会把采样整段裁空（实测 0 样本缺陷）——本轨无 seek，
每页取 (i+1)×16384 保守高估使裁剪恒零；数据页不设 EOS 位（symphonia 以 EOF 判流末）。
fixture `sample-webm-vp9-vorbis.webm`（VP9+Vorbis 双轨 2s）。registry 增
`WebmAudioEntry` 伴生条目：video play 懒建、同锚起播、泵推进写 NullSink、增益联动、
pause 冻结；纯视频 webm 无音频轨静默。
② **切片 E（audio clock 主时钟）**：`VideoPlayer::sync_to_media_time`——视频帧调度
对齐外部主时钟游标（位置每 tick 派生自主时钟——drift 构造校正不积累墙钟差；位置只
前进不回退；帧调度核抽 `present_pending` 供 tick/sync 共用，墙钟路径零回归）。
`tick_all` 主时钟先行序（伴音频轨先推进 → A/V pair 视频 sync 追随游标 → 纯视频回落
墙钟 tick）。currentTime 组合时钟：A/V pair 优先报音频游标。seek 双轨对齐：
`av_sources` 字节留存 + 伴生轨重建（游标/静默线对齐 target）——master clock 面不脱轨。
`WebmAudioEntry` 补 `skip_until_ms` 追赶区静默（AudioEntry 同面）。
③ 测试：zero-media 36+1（av_decode 3 件：解码链 44.1kHz 2s≈88200 帧 + 440Hz 过零率
880 锚点 / 双轨下 VP9 视频轨 48 帧零回归 / 无音频轨 NoTrack feature-detect；player
sync 4 面断言）；webview 676（registry AV pair master-clock e2e + seek 对齐续进）；
engine 2542 全绿。

**M2b 已落地**（同日）：精确 seek + 变速桥接全链——
① `VideoDecoder::seek_to_ms`：两阶段精确 seek——phase ① demuxer Cues 定位 +
  **keyframe 落点验证**（`block.is_keyframe` gate：cue 点即 keyframe；非 keyframe
  落点——无 Cues 流的线性搜索——参考链断裂不可靠）→ phase ② 全量回退（seek 0 +
  解码器重建 + 前向解码至 ≥ target 首帧，帧存 `pending` 不丢——spec precise-seek）。
  实测锚点：fixture 单 keyframe（testsrc2 无 GOP 内刷新，ffprobe 实证全流唯一 K 帧
  pts=0）→ seek(1000ms) 回退路径命中 pts≥1000 首帧、续播 PTS 单调至末；seek(0)
  全流 48 帧完整重播；
② `VideoPlayer::seek_to_ms`：位置 clamp [0,duration]、播放中 seek 保持播放（时钟
  锚点重置防 Δt 跳变）、暂停中 seek 保持暂停（spec）、pending 帧先弹出；
③ registry/桥：`seek`（未 play 的已登记源自动建 player 且置暂停——spec「seek 不改
  paused」）+ `set_playback_rate`；桥回调 `__zw_video_seek`/`__zw_video_set_rate`
  + JS 门面 `seek`/`setRate`；
④ shim：`currentTime=` setter 桥推（seeking/seeked 事件序列不变——headless 断言面
  保持）+ `playbackRate=` setter 桥推（ratechange 不变）+ `play()` 起播时同步既有
  速率。
测试：zero-media seek 2 件 + player seek 2 件；registry seek 1 件 + 桥 e2e seek 断言；
engine 2539 / media 27 / webview 668 全绿；testharness-media 372P/0F/41PF 维持。

## 下一步计划块：切片 1 ~ 11 落地明细（2026-09-02 ~ 2026-09-04，原文）

   **切片 1 落地（2026-09-02，DC-4 WPT 子集导入前置）**：① webm A_OPUS 解码
   （`WebmOpusAudioTrack`——Matroska demux + CodecPrivate=OpusHead 解析 +
   opus-decoder 逐包直解，无 OGG 重封装；movie_5.webm 实测 5.01s/pts 单调/
   48kHz）+ registry 伴生轨 codec 泛化（`WebmAudioTrackKind`，Vorbis/Opus 双形态
   Box 装箱）+ canPlayType video/webm opus 扩表（'vp9, opus' → probably——WPT
   common/media.js getVideoURI 判定串解锁，此前该判定 '' 使 URI 落不存在的
   .mp4）；② wpt-runner 播放面：webview `install_playback_bridge()`（同进程嵌入方
   桥注册入口）+ 页面脚本后 media src 提取登记（extract_media_resources →
   wpt-data 字节 → registry）+ probe 循环播放泵（tick_all + audio_advance_all，
   playback_clock_origin 单调时钟与桥 play(0) 对齐——tab_worker pump_epoch 同
   契约）+ shim `_zwMediaTimeMarchesOn` cue 调度钩子（桥真值钟推进 enter/exit/
   pauseOnExit——media-elements track-cues-* 播放推进族解锁面）。探针实证
   play→playing→promise resolve→currentTime 真值推进（0.53s@500ms）全链通；
   media-elements 529P/0F/24PF 零回归。**余**：WPT 播放推进用例导入
   （track-cues-enter-exit / missed / seeking 等——cue 时钟推进面已就绪，
   seek 语义面待切片 2）。
   **切片 2 落地（2026-09-03，track-cues-* 播放推进族解锁——media-elements
   M3 扩批 XVI 兑现）**：① runner 播放桥**前置**页面脚本（execute_script 预热
   ensure_sandbox/ensure_js_shim 后 install_playback_bridge——canplaythrough 内
   同步 play 可达桥）；② probe 循环**逐 tick 动态源登记**（JS 快照 media 现值
   src → wpt-data 字节 → registry；contains_source 幂等 + Rc 字节缓存；settle
   提交随首次登记）；③ shim play() 桥 src 改 latest-wins 读 + `_hit` 单次调用 +
   未命中退避重试（setTimeout 0 × 5000 上限）+ pending seek 补推（seek-before-
   play 落位）；④ registry play 未命中**不消费**源字节（sources.get 克隆、命中
   才 remove——旧形态一次失败即丢字节，单测
   `play_miss_retains_source_bytes_for_retry`）+ `contains_source` + `is_ended`
   桥面（`__zw_video_is_ended`/`isEnded`）；⑤ march 钩子**区间捕获 + 事件时间
   序派发**（跨 (lastMs, nowMs] 收集 enter@start/exit@end 按时间排序——1ms cue
   不再整体跳过；seek 判据改「时钟回退 ∨ seeking 标志」）+ **ended 面**（active
   cue 倒序 exit flush + timeupdate + ended）。WPT 实证：media-elements 531P/
   0F/24PF（+2 净涨——track-cues-enter-seeking + track-cues-missed 稳定全绿；
   enter-exit 因注册竞态 1/4 flake 暂排除，随泵节拍精化复评）。**余**：WPT 播放
   推进用例续批（enter-exit 复评 + seeking/sorted/pause-on-exit 等随基础设施
   增量逐件）。
   **切片 3 落地（2026-09-03 续，解码器 EOF 排空缺陷修复——播放推进族余件
   解锁）**：宿主插桩实证（console→tracing + 泵侧 Rust 快照）定位两类
   zero-media 解码层缺陷：① `VideoDecoder::next_frame` 的 demux 耗尽分支 flush
   后仅 pull 一帧即置 `eof`——rusty_vp9 的 hidden/alt-ref 帧（show_frame=0）
   解码后返 `Again` 不产出，每次消耗一个 pull 机会但其 pts 帧晚一个 demux 块
   浮现，形成 ~15 帧（≈0.5s）输出滞后；积压帧被提前置位的 eof 吞掉，
   `present_pending` 在 position < duration 处遇 `Ok(None)` 即转 Ended
   （test.webm 实测 Ended@媒体时间 3.57s / 流长 6.035s；WPT 形态 =
   track-cues-enter-exit 的 cue@4-5s 永不触发）。修复：`draining` 中间态 +
   `drain_frame`（Again=隐藏帧继续拉、Eof=队列真空才停）；seek 双分支同步
   重置。② `present_pending` 遇 `pts > position` 的未来帧把它返回给调用方
   （渲染后丢弃）——时间槽永久丢失，粗 tick 背压下逐 tick 丢未来帧使解码器
   再次提前耗尽。修复：`VideoDecoder::un_read` 队首退回（pending 槽复用）——
   spec ended「currentTime 到达流末」，帧调度不得超越时钟消费时间线。修复后
   模拟 181 帧全呈现、position=6.0 才 Ended；WPT：media-elements 534P/0F/24PF
   （+3 净涨——enter-exit / pause-on-exit 解除排除，4 连跑稳定）。单测
   `webm_sequential_decode_drains_hidden_tail_frames_r3936`。**余**：WPT 播放
   推进用例余面（track-cues-seeking——seek 事件真值化复评；其余 B 组随基础
   设施增量逐件）。
   **切片 6 落地（2026-09-03 续，media load invoke 重置面——B 组排除件全清）**：
   扩批 XXII 后 track-active-cues 复评仍 Timeout，dormant 插桩
   （`__zwPauseWatch` + `ZW_MEDIA_SEEK_DEBUG` runner 门控，验证后移除）定位双
   根因：① invoke 入口 `_resourceStates[key]` 未重置——settle 幂等门吞掉
   `src=''` 二次调度的 error 提交；② settle 的 load/error 派发传 handle=null——
   handle-only（createElement）元素 listener 键恒失配，on\* expando 兜底
   （`_zwMediaFire`）未接。修复四处（shim part06）：invoke 重置 settle 面 +
   invoke 步 6 位置重置（currentTime=0 / `_zwMediaTimeKnown` 失效）+ invoke 重置
   track 子产物 cue（addTextTrack 产物排除）+ settle 的 track/audio/video
   load|error 派发改 `_zwMediaFire`。track-active-cues 导入（540P/0F/24PF，
   +1 净涨零回归；单测 `test_media_load_invoke_reset_face_m3xxiii`）。
   **B 组排除件至此全清**（XX disabled/no-cuechange/remove-active-cue + XXIII
   track-active-cues）。**余**：WPT 播放推进用例余面（playing-the-media-resource
   的 play-in-detached-document / loop-from-ended / fragmented-mp4-end——依赖
   detached 文档播放钟与 loop 面）。
   **切片 7 落地（2026-09-03 续，扩批 XXIV——loop 真面 + played TimeRanges）**：
   registry `set_loop`（音频 entry 流末回卷：`restart()` 解码器重建 + 游标归零 +
   skip 线清零 + 播放态保持——解码器单向流 fixture 级小源可接受；`WebmAudioEntry`
   伴生轨同面；`reached_end` 标志——loop=false 流末置位 / play·seek·restart 清除，
   补齐音频面 `isEnded` 桥驱动源）+ `registry_key` 规范化（strip query/fragment，
   全查询面统一——WPT cache-buster `?...Math.random()` 与 shim/runner URL 编码
   差异此前两侧键失配；bridge play 的 `audio_guess` 同面 strip，`.oga?...` 此前恒
   miss）+ shim loop IDL setter/getter（`_mediaState.loop` 镜像 + 桥推送 +
   play 起播/retry 路径同步）+ march Ended 面 loop 分叉（seek(0)+play 重头 +
   seeking/seeked 异步派发非 ended；非 loop 分支补 `ms.ended = true` IDL 置位）+
   played TimeRanges（march 逐拍采样 `_zwPlayedRanges` 区间合并 250ms 容差 →
   getter `length/start/end` + IndexSizeError；seek 回退不合并；loop 回卷前尾段
   `[lastMs, duration]` 按桥真值计入）。**缺陷修复**：`__zw_video_set_loop` 回调
   读 `args.get(2)` 而门面 `setLoop(src, on)` 只传 2 参——`on` 恒 false，loop 真
   面从未生效（本片单测实证回卷后修正）。media-elements 542P/0F/24PF（+2 净涨
   零回归——played-loop / audio_loop_seek_to_eos 导入；loop-from-ended 暂不导入，
   动态 src settle 竞态注记于 testharness.rs）。单测
   `registry_audio_loop_restarts_at_stream_end`（回卷 ≥2 次 + 播放态保持 +
   loop=false 对照停）。
   **切片 8 落地（2026-09-03 续，扩批 XXV——loop-from-ended 导入 + Ended→play
   全链收口）**：dormant 探针链（8 轮 probe，验证后全移除）定位四处缺陷：
   ① registry `play` 的 player Ended 态直接置 Playing——解码器耗尽下一拍即再
   Ended（seek(0) 也无效）；经 sources/av_sources 留存字节 `reset()` 重建 +
   伴生轨 `cursor/skip/last_tick` 归零（current_time 优先读 av 游标——不归零则
   Ended→play 后桥钟读数恒在流末）；② registry `seek` 的 av 游标未 clamp——
   语义层以 headless duration 600 算的 seek 目标（599.5s）超真实流长（5.008s），
   audio clock 主时钟游标超界把视频位置拉出流末（ended 面 currentTime 读数
   失真）；clamp 到 player clamp 后位置；③ **泵时钟原点错位**——shim 桥 play
   恒传 nowMs=0 而泵 tick 用宿主单调钟 elapsed，首拍 delta=泵全程使位置瞬间跳
   到流末；`register_video_bridge_callbacks` 增 `clock: Option<Arc<AtomicU64>>`
   参数 + webview `install_playback_bridge_with_clock`，runner 泵时钟注入——
   play 锚与 tick 同源（tab_worker pump_epoch 语义同构，后续可同法注入）；
   ④ shim `play()` 无「ended 态 play」语义——spec「ended playback」步 6.4 在
   play 入口即生效（seek 回最早位置 + 派 seeking/seeked 非 ended），以 march
   非 loop 分支的 `_zwEndedDispatched` 标记为判定面（loop setter 已把 ms.ended
   翻回 false——spec「looping 媒体不能是 ended」，IDL 态不可靠）+ loop=true
   setter 同步翻 ended=false。loop-from-ended.tentative 导入
   （543P/0F/24PF，+1 净涨零回归）。
   **切片 9 落地（2026-09-04 续，扩批 XXVI——seekable/buffered TimeRanges +
   seek clamp）**：① `__zwMediaSeekableRanges`（part06 共享 helper，part04
   get trap 的 seekable/buffered 统一入口）：readyState>=1 后 [0,duration]
   单区间、HAVE_NOTHING 空集合、越界 IndexSizeError；duration 解析序 桥真值 →
   _mediaState.duration → settle durationMs → headless 600；has-trap 白名单补
   seekable/buffered（part05）。**DC-3 的「buffered/seekable 未实施——上游无
   断言用例」注记失效**（seeking/ 目录即 seekable 断言面）——headless 近似面
   收口，流式真值化随播放面背压优化另评。② currentTime setter seek 语义补全：
   clamp 到 seekable 范围（spec seek 步 5——镜像写 clamp 后值，duration 未知只
   clamp 下限 0）+ seeking/timeupdate/seeked 同一排队任务序派发（seeking 异步
   化——后挂 onseeking 可达；seeking 翻 false 先于 timeupdate——Chromium 可
   观察语义；事件序 [seeking, timeupdate, seeked]）。seeking/ 三件
   （seek-to-currentTime/max-value/negative-time）+ volume_nonfinite 导入
   （552P/0F/24PF，+9 净涨零回归）。
   **切片 10 落地（2026-09-04 续，扩批 XXVII——media fragment + headless 播放
   时钟）**：① media fragment #t= 起点解析（`_zwMediaLoadSequence` 内：settle
   url hash → & 分隔 k=v 对取 t= → percent-decode → `npt:` 前缀剥离 →
   start,end 取 start → HH:MM:SS.ms / MM:SS.ms / SS 解析 → 成功置
   ms.currentTime 起始位置——spec media-frags「seek to the fragment start」，
   media_fragment_seek 的 5 形态断言面）。② **headless 播放时钟推进**（march
   内非 bridgeOn 且 playing：performance.now 墙钟差 × playbackRate 推进
   ms.currentTime，`_zwHeadlessClockOrigin` 记 play 基点只前进不回退——此前
   headless 播放无推进面，autoplay 驱动的播放 currentTime 恒 0，
   autoplay-with-broken-track 的 currentTime>0 断言面）。③ **周期 timeupdate**
   （march 内 nowMs>lastMs 时 ≥250ms 节流派发——spec time updates；此前播放
   推进期页面无 timeupdate 可收）。dormant 探针实证 headless 时钟 0→0.12s
   推进 + expando handler 可达后移除。media_fragment_seek +
   autoplay-with-broken-track 导入（556P/0F/24PF，+4 净涨零回归）；no-autoplay-
   audio-history-back 不导入（iframe+history+postMessage 导航深结构）。
   **切片 11 落地（2026-09-04 续，扩批 XXVIII——同文档移动 currentTime 面）**：
   currentTime-move-within-document 导入（offsets-into-the-media-resource/ 末件
   ——seek(10) 后 appendChild 移动 paused=false + currentTime>=10 保持；零改动
   导入，headless 时钟推进面（切片 10）现成）；fixture 增 movie_300.webm（VP9
   300s）。media-elements 557P/0F/24PF（+1 净涨零回归）。**余**：
   playing-the-media-resource 余面（play-in-detached-document /
