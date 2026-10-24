//! media-host 真实进程契约测试（`media-process` feature 门控 + `#[ignore]` 显式运行）。
//!
//! 与 M1 image-decoder 契约测试同模式：内存通道测试（`apps/media-host` 单测）验证不了
//! stdio 管道帧格式与真实子进程生命周期，须以真实 `zero-media-host` 二进制为对端
//! （zeroweb-knowledge：SharedMemoryChannel 不能替代真实对端验证）。
//!
//! 覆盖面：控制面契约（登记/喂入/字节未齐错误面/配额/降级纪律）+ C2 解码内核真实
//! 管线（play 起播 → 元数据/状态/帧推送 → seek 代际 → 暂停冻结 → 纯音频循环回卷
//! 代际；夹具均为 2s，tests/fixtures/media）。
//!
//! 运行方式（需先构建 media-host 二进制，使 exe 上溯查找链能命中 target/debug）：
//! `./target/test-guard --compile-first ... -- cargo test -p zero-webview
//!  --features media-process --all-targets -- --ignored`

#![cfg(feature = "media-process")]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use zero_protocol::media::{MediaErrorCode, MediaPlaybackState};
use zero_webview::media_proxy::{MediaHostClient, MediaHostEvent};

/// 提取 target/debug 级 zero-media-host（测试二进制在 target/debug/deps/，上溯一级）。
fn built_media_host_bin() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let debug_dir = exe.parent()?.parent()?;
    let candidate = debug_dir.join(if cfg!(windows) {
        "zero-media-host.exe"
    } else {
        "zero-media-host"
    });
    candidate.is_file().then_some(candidate)
}

/// `ZW_MEDIA_HOST_BIN` 指向真实二进制（RAII 恢复原值；互斥由 `#[serial]` 保证，
/// edition 2024：env 修改为 unsafe）。
struct HostBinGuard;

impl HostBinGuard {
    fn set() -> Self {
        let Some(bin) = built_media_host_bin() else {
            panic!("zero-media-host 未构建：先 cargo build -p zero-media-host 再运行本测试");
        };
        unsafe {
            std::env::set_var("ZW_MEDIA_HOST_BIN", bin);
        }
        Self
    }
}

impl Drop for HostBinGuard {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var("ZW_MEDIA_HOST_BIN");
        }
    }
}

/// 读取工作区媒体夹具（CARGO_MANIFEST_DIR = crates/webview → 上溯至工作区根）。
fn fixture_bytes(name: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.pop();
    path.push("tests/fixtures/media");
    path.push(name);
    std::fs::read(path).expect("media fixture present")
}

/// 登记 + 全量喂入（register → 单 chunk 全量 → 累积 Ack 回执）。
fn feed_fixture(client: &mut MediaHostClient, source_key: u64, name: &str, mime: &str) {
    let bytes = fixture_bytes(name);
    let ready = client
        .register_source(source_key, bytes.len() as u64, mime)
        .expect("register_source");
    assert_eq!(ready.source_key, source_key);
    assert_eq!(ready.duration_ms, None, "喂入前 demux 不可行，元数据如实 None");
    let ack = client.send_data_chunk(source_key, 0, bytes).expect("send_data_chunk");
    assert_eq!((ack.source_key, ack.last_seq), (source_key, 0));
}

/// 阻塞等待首个匹配事件（真实进程推送为异步面；5ms 轮询 + 超时返回 None）。
fn wait_event(
    client: &mut MediaHostClient,
    timeout: Duration,
    mut take: impl FnMut(&MediaHostEvent) -> bool,
) -> Option<MediaHostEvent> {
    let deadline = Instant::now() + timeout;
    loop {
        match client.try_recv_event() {
            Ok(Some(event)) if take(&event) => return Some(event),
            Ok(_) => {}
            Err(error) => panic!("事件面失效: {error}"),
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// 排空当前全部事件。
fn drain_events(client: &mut MediaHostClient) -> Vec<MediaHostEvent> {
    let mut events = Vec::new();
    while let Ok(Some(event)) = client.try_recv_event() {
        events.push(event);
    }
    events
}

/// 真实二进制控制面契约：握手 → register/SourceReady(None 元数据) → chunk/Ack →
/// 字节未齐 play/seek 如实报 HostInternal（不虚起半流）→ 配额拒绝后连接仍健康
/// （host MediaError 不置永久失败，IF-006）→ release → shutdown 进程退出。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
#[serial_test::serial]
fn real_host_lifecycle_and_error_faces() {
    let _guard = HostBinGuard::set();
    let mut client = MediaHostClient::new();

    // 登记 + 分片回执（信封 id 配对由 client 断言，这里核语义值）。
    let ready = client.register_source(7, 128, "video/webm").expect("register_source");
    assert_eq!(ready.source_key, 7);
    assert_eq!(ready.duration_ms, None, "字节未齐 demux 不可行，元数据如实 None");

    let ack = client.send_data_chunk(7, 4, vec![0u8; 64]).expect("send_data_chunk");
    assert_eq!((ack.source_key, ack.last_seq), (7, 4));

    // 字节未齐（64/128）：play/seek 如实报错误面（内核不虚起半流）。
    let play_error = client.play(7, 0).expect_err("字节未齐 play 必为错误面");
    assert_eq!(play_error.code, MediaErrorCode::HostInternal);
    assert!(
        play_error.message.contains("incomplete"),
        "diagnostic: {}",
        play_error.message
    );
    let seek_error = client.seek(7, 1000).expect_err("字节未齐 seek 必为错误面");
    assert_eq!(seek_error.code, MediaErrorCode::HostInternal);

    // host MediaError（配额）不置永久失败：超限登记被拒后，连接仍可服务后续请求。
    let quota_error = client
        .register_source(8, zero_protocol::media::MEDIA_SOURCE_MAX_BYTES + 1, "video/webm")
        .expect_err("登记超限必拒");
    assert_eq!(quota_error.code, MediaErrorCode::QuotaExceeded);
    assert!(!client.is_failed(), "host 错误面 ≠ 传输失败，client 不置永久失败");
    let ready2 = client
        .register_source(9, 16, "audio/ogg")
        .expect("配额拒绝后连接仍健康");
    assert_eq!(ready2.source_key, 9);

    // fire-and-forget 释放 + 优雅关闭（写半 EOF → host 自行退出）。
    client.release_source(9).expect("release_source");
    client.shutdown().expect("shutdown");
    drop(client);
}

/// C2 视频管线契约：play 起播 → SourceReady 元数据推送（真实尺寸/时长）→ Playing
/// 状态推送 → 帧推送（epoch 0，pts 单调、rgba 自洽）→ seek 解析真值 + 代际 +1 →
/// pause 回执 + 帧流冻结（Paused 回执先于断言，wire FIFO 保证无迟到帧）。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
#[serial_test::serial]
fn real_host_video_play_frames_seek_and_pause() {
    let _guard = HostBinGuard::set();
    let mut client = MediaHostClient::new();
    feed_fixture(&mut client, 7, "sample-webm-vp9.webm", "video/webm");

    client.play(7, 0).expect("play");

    // 元数据推送（play 建解码器后的真值；id 0 自发面）。
    let event = wait_event(&mut client, Duration::from_secs(5), |e| {
        matches!(e, MediaHostEvent::SourceReady(_))
    })
    .expect("SourceReady 推送超时");
    let MediaHostEvent::SourceReady(ready) = event else {
        unreachable!("wait_event 谓词已过滤")
    };
    assert_eq!(ready.source_key, 7);
    assert_eq!((ready.width, ready.height), (Some(320), Some(240)), "夹具尺寸真值");
    assert_eq!(ready.duration_ms, Some(2000), "2s 夹具时长真值");

    // 帧推送：pts 单调不减（present 语义 = 最新可呈现帧，中间帧跳过——registry
    // 同构），首帧受真实泵启动延迟约束（< 250ms），rgba 与声明尺寸自洽。
    let mut first_pts: Option<u64> = None;
    let mut last_pts: Option<u64> = None;
    for _ in 0..3 {
        let event = wait_event(&mut client, Duration::from_secs(5), |e| {
            matches!(e, MediaHostEvent::Frame(_))
        })
        .expect("帧推送超时");
        let MediaHostEvent::Frame(frame) = event else {
            unreachable!("wait_event 谓词已过滤")
        };
        assert_eq!((frame.source_key, frame.epoch), (7, 0));
        assert_eq!(frame.rgba.len(), (frame.width * frame.height * 4) as usize);
        if let Some(previous) = last_pts {
            assert!(frame.pts_ms >= previous, "pts 单调不减: {} < {previous}", frame.pts_ms);
        }
        if first_pts.is_none() {
            first_pts = Some(frame.pts_ms);
        }
        last_pts = Some(frame.pts_ms);
    }
    assert!(first_pts.is_some_and(|pts| pts < 250), "首帧 pts 过迟: {first_pts:?}");

    // seek：解析真值（精确 seek）+ 代际 +1（后续帧 epoch 1）；回执配对由
    // client.seek 按信封 id 匹配成功即证。
    let result = client.seek(7, 1000).expect("seek");
    assert_eq!(result.resolved_ms, 1000);
    let event = wait_event(
        &mut client,
        Duration::from_secs(5),
        |e| matches!(e, MediaHostEvent::Frame(f) if f.epoch == 1),
    )
    .expect("seek 后新代纪帧超时");
    let MediaHostEvent::Frame(frame) = event else {
        unreachable!("wait_event 谓词已过滤")
    };
    assert_eq!(frame.source_key, 7);

    // pause：回执后帧流冻结（Paused 回执被 client.pause 消费；内核停摆无后续帧）。
    client.pause(7).expect("pause");
    std::thread::sleep(Duration::from_millis(400));
    let frozen = drain_events(&mut client);
    assert!(
        !frozen.iter().any(|e| matches!(e, MediaHostEvent::Frame(_))),
        "暂停后不得再出帧: {frozen:?}"
    );

    client.shutdown().expect("shutdown");
}

/// C2 A/V 管线契约：伴生音轨同构起播（音频主钟 sync_to_media_time），元数据/帧
/// 推送与纯视频同面。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
#[serial_test::serial]
fn real_host_av_play_pushes_metadata_and_frames() {
    let _guard = HostBinGuard::set();
    let mut client = MediaHostClient::new();
    feed_fixture(&mut client, 7, "sample-webm-vp9-vorbis.webm", "video/webm");

    client.play(7, 0).expect("play");

    let event = wait_event(&mut client, Duration::from_secs(5), |e| {
        matches!(e, MediaHostEvent::SourceReady(_))
    })
    .expect("SourceReady 推送超时");
    let MediaHostEvent::SourceReady(ready) = event else {
        unreachable!("wait_event 谓词已过滤")
    };
    assert_eq!((ready.width, ready.height), (Some(320), Some(240)));
    // demux 时长真值：容器封装取末包边界，2s 夹具实报 2003ms。
    let duration = ready.duration_ms.expect("A/V 时长真值");
    assert!((1900..=2100).contains(&duration), "时长 2s 量级: {duration}");

    for _ in 0..2 {
        let event = wait_event(&mut client, Duration::from_secs(5), |e| {
            matches!(e, MediaHostEvent::Frame(_))
        })
        .expect("帧推送超时");
        let MediaHostEvent::Frame(frame) = event else {
            unreachable!("wait_event 谓词已过滤")
        };
        assert_eq!((frame.source_key, frame.epoch), (7, 0));
        assert_eq!(frame.rgba.len(), (frame.width * frame.height * 4) as usize);
    }

    client.shutdown().expect("shutdown");
}

/// C2 纯音频循环契约：set_loop 后流末回卷（内核 wrap 检测 → 重建解码链 → 代际
/// +1 → 新代纪 Playing 推送；registry `AudioEntry.wrap_pending` 同构）。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
#[serial_test::serial]
fn real_host_audio_loop_wraps_with_epoch_increment() {
    let _guard = HostBinGuard::set();
    let mut client = MediaHostClient::new();
    feed_fixture(&mut client, 7, "sample-ogg-vorbis.oga", "audio/ogg");

    client.set_loop(7, true).expect("set_loop");
    client.play(7, 0).expect("play");

    // 首代 Playing。
    let event = wait_event(
        &mut client,
        Duration::from_secs(5),
        |e| matches!(e, MediaHostEvent::State(s) if s.epoch == 0),
    )
    .expect("首代 Playing 推送超时");
    let MediaHostEvent::State(state) = event else {
        unreachable!("wait_event 谓词已过滤")
    };
    assert!(matches!(state.state, MediaPlaybackState::Playing { .. }));

    // 回卷：2s 夹具播完 → wrap → 代际 +1，新代纪光标回到头部。
    let event = wait_event(
        &mut client,
        Duration::from_secs(8),
        |e| matches!(e, MediaHostEvent::State(s) if s.epoch == 1),
    )
    .expect("回卷代际推送超时（2s 夹具 + 心跳节奏）");
    let MediaHostEvent::State(state) = event else {
        unreachable!("wait_event 谓词已过滤")
    };
    match state.state {
        MediaPlaybackState::Playing { media_time_ms } => {
            assert!(media_time_ms < 500, "回卷后光标回头部: {media_time_ms}");
        }
        other => panic!("回卷后应为 Playing: {other:?}"),
    }

    client.shutdown().expect("shutdown");
}

/// C2 切片 2 端到端：`VideoPlayerRegistry` 经 `HostMediaFacade` 全宿主面——
/// register 镜像建档（不落内联字节）→ play 回执载荷即时 Playing → tick_all 排空
/// SourceReady 升级元数据 + 帧注入 `ImageCache` → current_time 镜像真值 → seek
/// resolved 真值 → pause 冻结 → 纯视频 loop Ended 回卷重头（renderer 侧 play(0)，
/// 内核 replay 链代际 +1——§10.7-2）。真实二进制驱动，DC-2/M1 验收面。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
#[serial_test::serial]
fn hosted_registry_end_to_end() {
    use zero_render_foundation::image_cache::ImageCache;
    use zero_webview::video_registry::VideoPlayerRegistry;

    let _guard = HostBinGuard::set();
    let registry = std::sync::Arc::new(std::sync::Mutex::new(VideoPlayerRegistry::new()));
    let src = "https://example.com/media/e2e-hosted.webm";
    fn lock(
        r: &std::sync::Arc<std::sync::Mutex<VideoPlayerRegistry>>,
    ) -> std::sync::MutexGuard<'_, VideoPlayerRegistry> {
        r.lock().unwrap_or_else(|e| e.into_inner())
    }

    // 登记：宿主建档镜像；contains_source 宿主面命中（幂等登记守卫同面）。
    lock(&registry).register_source(src, fixture_bytes("sample-webm-vp9.webm"));
    assert!(lock(&registry).contains_source(src));

    // 未 play：镜像初值——时钟 0、元数据未就绪（SourceReady 未到，§10.7 偏差 9）。
    assert_eq!(lock(&registry).current_time(src), 0.0);
    assert_eq!(lock(&registry).duration(src), None);
    assert_eq!(lock(&registry).probe_dimensions(src), (0, 0));
    assert!(!lock(&registry).is_any_playing());

    // play：回执载荷即时入镜像（无心跳盲区）。
    assert!(lock(&registry).play(src, 0));
    assert!(lock(&registry).is_playing(src));
    assert!(lock(&registry).is_any_playing());

    // tick 排空：SourceReady 升级元数据 + 帧注入 ImageCache（painter 同键）。
    let mut cache = ImageCache::default();
    let started = Instant::now();
    loop {
        if lock(&registry).tick_all(started.elapsed().as_millis() as u64, &mut cache) {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "帧注入超时（SourceReady/帧推送未达）"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(cache.len() > 0, "帧须入 ImageCache");
    assert_eq!(lock(&registry).duration(src), Some(2.0), "SourceReady 升级时长真值");
    assert_eq!(lock(&registry).probe_dimensions(src), (320, 240));
    assert!(lock(&registry).source_media_meta(src).is_some());

    // seek：resolved 真值即时入镜像（回执配对面）。
    assert!(lock(&registry).seek(src, 1000));
    let seeked = lock(&registry).current_time(src);
    assert!((seeked - 1.0).abs() < 0.05, "seek 后镜像时钟 = resolved: {seeked}");

    // pause：冻结（暂停后内核无帧/无时钟推进，镜像时钟不再变化）。
    lock(&registry).pause(src);
    assert!(!lock(&registry).is_playing(src));
    let frozen_at = lock(&registry).current_time(src);
    std::thread::sleep(Duration::from_millis(200));
    lock(&registry).tick_all(u64::MAX / 2, &mut cache);
    assert_eq!(lock(&registry).current_time(src), frozen_at, "暂停时钟冻结");

    // 纯视频 loop：内核不自动回卷（无音频轨 wrap 检测）——Ended 推送后 registry
    // 侧 flush_replays play(0) 走内核 replay 链（代际 +1）。观测面：时钟先走远
    //（>1.5s）再回小值（<0.5s）且保持播放态。
    lock(&registry).set_loop(src, true);
    assert!(lock(&registry).play(src, 0));
    let loop_started = Instant::now();
    let (mut saw_far, mut saw_wrapped) = (false, false);
    while loop_started.elapsed() < Duration::from_secs(8) {
        lock(&registry).tick_all(loop_started.elapsed().as_millis() as u64, &mut cache);
        let t = lock(&registry).current_time(src);
        if t > 1.5 {
            saw_far = true;
        }
        if saw_far && t < 0.5 {
            saw_wrapped = true;
            assert!(lock(&registry).is_playing(src), "回卷后保持播放态");
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        saw_far && saw_wrapped,
        "纯视频 loop 回卷未观测到: far={saw_far} wrapped={saw_wrapped}"
    );

    lock(&registry).clear();
    assert!(!lock(&registry).contains_source(src), "clear 清宿主镜像");
    assert!(!lock(&registry).is_any_playing());
}
