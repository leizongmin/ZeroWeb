//! C4 配对验收（media-runtime-isolation goal——M4 配对矩阵 + 资源审计，DESIGN §12）。
//!
//! 三态矩阵（§12.1）：内联基线（`media-process` 关）/ 宿主态（开，`ZW_MEDIA_SHM`
//! 缺省）/ 宿主态 + `ZW_MEDIA_SHM=0`（kill-switch 端到端）。容量/时钟/seek/漂移
//! 判据由 M1 harness（`media_m1_baseline.rs`）三态原样复跑产出；本文件补齐其余
//! 判据面：帧一致性摘要（§12.2 像素全等）、代际风暴（§12.3 旧 epoch 零泄漏）、
//! 能力真值三对照（§12.5 host 实报 vs 实解；页面可见面为 engine shim 静态表，
//! 见 ANALYSIS 对照表）与 30min 资源审计（§12.4）。
//!
//! 宿主态测试需 `zero-media-host` 二进制在位（`cargo build -p zero-media-host`），
//! 经 `ZW_MEDIA_HOST_BIN` 指路（`#[serial]` 互斥，契约测试先例）。
//!
//! 运行（先无限制编译，执行阶段 test-guard 包裹）：
//! `cargo build -p zero-media-host`
//! `cargo test -p zero-webview --test media_c4_paired --no-run`
//! `./target/test-guard --time-limit 600 --per-proc-mem 4 --total-mem 8 -- \
//!    target/debug/deps/media_c4_paired-* --ignored --nocapture --test-threads=1 \
//!    --skip c4_resource_audit_30min`
//! 审计单独跑（>30min，墙钟阈值放宽）：
//! `./target/test-guard --time-limit 2000 --per-proc-mem 4 --total-mem 8 -- \
//!    <同上二进制> --ignored --nocapture --test-threads=1 c4_resource_audit_30min --exact`

use std::hash::{DefaultHasher, Hash, Hasher};
#[cfg(feature = "media-process")]
use std::time::{Duration, Instant};

use zero_render_foundation::image_cache::ImageCache;
use zero_webview::video_registry::VideoPlayerRegistry;

#[cfg(feature = "media-process")]
use zero_protocol::media::MEDIA_SOURCE_MAX_BYTES;

const VP9_60S: &str = "file:///fixtures/long-webm-vp9-60s.webm";
const AV_60S: &str = "file:///fixtures/long-webm-vp9-vorbis-60s.webm";

fn fixture_bytes(name: &str) -> Vec<u8> {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // workspace root
    p.push("tests/fixtures/media");
    p.push(name);
    std::fs::read(p).unwrap_or_else(|e| panic!("fixture {name} 读取失败: {e}"))
}

/// 帧哈希序列折叠摘要（跨态比较面——同序全等 ⇔ 摘要相等且帧数相等）。
fn fold_digest(hashes: &[u64]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for h in hashes {
        h.hash(&mut hasher);
    }
    hasher.finish()
}

/// 参照摘要（DESIGN §12.2「C1 转换输出为参照点」）：顺序解码 fixture，逐帧走
/// 与呈现面同构的 `ImageData::from_rgba_variable` → `content_hash` 管线。
fn reference_video_hashes(bytes: &[u8]) -> Vec<u64> {
    let mut decoder = zero_media::VideoTrackDecoder::open_media(bytes).expect("fixture 可解码");
    let mut hashes = Vec::new();
    while let Ok(Some(frame)) = decoder.next_frame() {
        let data =
            zero_render_foundation::image_cache::ImageData::from_rgba_variable(frame.rgba, frame.width, frame.height)
                .expect("帧 RGBA 合法");
        hashes.push(data.content_hash);
    }
    hashes
}

/// 单源场景下取当前呈现帧哈希（cache 仅视频帧一入口——`ImageKey` 同键覆盖写）。
fn presented_hash(cache: &ImageCache) -> Option<u64> {
    let entries = cache.snapshot_entries();
    if entries.len() != 1 {
        return None;
    }
    entries.into_values().next().map(|d| d.content_hash)
}

/// 泵推进到流末并采集逐呈现帧哈希（M1 `pump_to_end` 同构 + 采集面）。
/// 时钟分叉与 M1 一致：内联合成加速钟（压满解码链），宿主真实节拍。
fn pump_collect_hashes(reg: &mut VideoPlayerRegistry, cache: &mut ImageCache, src: &str, max_ticks: u32) -> Vec<u64> {
    let mut hashes = Vec::new();
    // 宿主支首读前重写（真实节拍钟），内联支初值参与推进——按 feature 静音。
    #[cfg_attr(feature = "media-process", allow(unused_assignments))]
    let mut now = 0u64;
    #[cfg(feature = "media-process")]
    let started = Instant::now();
    for _ in 0..max_ticks {
        #[cfg(feature = "media-process")]
        {
            now = started.elapsed().as_millis() as u64;
        }
        if reg.tick_all(now, cache) {
            hashes.push(presented_hash(cache).expect("单源呈现帧在缓存"));
        }
        let _ = reg.audio_advance_all(now);
        #[cfg(not(feature = "media-process"))]
        {
            now += 16;
        }
        #[cfg(feature = "media-process")]
        {
            // 宿主态真实节拍：内核自走钟产出 24fps，16ms 泵排空追赶（事件队列
            // 吸收批到达，无丢帧面——丢帧在断言面暴露）。
            std::thread::sleep(Duration::from_millis(16));
        }
        if !reg.is_any_playing() {
            break;
        }
    }
    assert!(!reg.is_any_playing(), "{src} 未在 {max_ticks} tick 内到流末");
    hashes
}

/// 帧一致性（§12.2/§12.3 像素全等）：呈现序列逐帧哈希 vs 顺序解码参照，
/// 帧数 + 折叠摘要双断言（帧重复计入次序——摘要含序）。
fn assert_frame_sequence_matches_reference(reg: &mut VideoPlayerRegistry, src: &str, bytes: &[u8]) {
    let mut cache = ImageCache::default();
    reg.register_source(src, bytes.to_vec());
    assert!(reg.play(src, 0), "play 就绪");
    let presented = pump_collect_hashes(reg, &mut cache, src, 6000);
    let reference = reference_video_hashes(bytes);
    println!(
        "C4PAIR framehash src={src} presented={} reference={} digest_presented={:016x} digest_reference={:016x}",
        presented.len(),
        reference.len(),
        fold_digest(&presented),
        fold_digest(&reference),
    );
    assert_eq!(
        presented.len(),
        reference.len(),
        "呈现帧数 {} ≠ 参照帧数 {}（丢帧/重帧——传输面或消费面缺口）",
        presented.len(),
        reference.len()
    );
    assert_eq!(
        fold_digest(&presented),
        fold_digest(&reference),
        "帧哈希序列摘要不等（像素面跨态不一致）"
    );
}

/// T-P1：纯视频 60s 帧一致性（三态各跑同 fixture，摘要跨态比对归 ANALYSIS）。
#[test]
#[ignore = "C4 长跑（宿主态 60s 实时）——显式运行（evidence 协议）"]
fn c4_frame_hash_pure_video() {
    let bytes = fixture_bytes("long-webm-vp9-60s.webm");
    let mut reg = VideoPlayerRegistry::new();
    assert_frame_sequence_matches_reference(&mut reg, VP9_60S, &bytes);
}

/// T-P2：A/V 双轨 60s 帧一致性（视频帧序列；音频面由 M1 T5 漂移判据覆盖）。
#[test]
#[ignore = "C4 长跑（宿主态 60s 实时）——显式运行（evidence 协议）"]
fn c4_frame_hash_av_pair() {
    let bytes = fixture_bytes("long-webm-vp9-vorbis-60s.webm");
    let mut reg = VideoPlayerRegistry::new();
    assert_frame_sequence_matches_reference(&mut reg, AV_60S, &bytes);
}

/// 实解矩阵（§12.5 第三条腿）：逐 fixture 探针解码真值，能力主张对照归宿主态
/// 测试与 ANALYSIS。快速面（小 fixture）——常驻 `make test`。
#[test]
fn c4_capability_decode_matrix() {
    // 视频：容器路由 + codec feature 门控真值。
    for (name, expect_ok) in [
        ("sample-webm-vp9.webm", true),
        ("sample-webm-vp9-vorbis.webm", true),
        // decode-av1 / decode-h264 默认关（D-RFC-2/3/4）——不虚报：探针须失败。
        ("sample-webm-av1.webm", cfg!(feature = "decode-av1")),
        ("sample-mp4-h264.mp4", cfg!(feature = "decode-h264")),
    ] {
        let bytes = fixture_bytes(name);
        let ok = zero_media::VideoTrackDecoder::open_media(&bytes).is_ok();
        println!("C4PAIR decode-matrix fixture={name} ok={ok} expect_ok={expect_ok}");
        assert_eq!(ok, expect_ok, "实解真值 vs 预期（feature 门控）不符: {name}");
    }
    // 音频：通用探针（mp3 / ogg-vorbis / wav 走 symphonia；ogg-opus 走专用入口）。
    for name in ["sample-mp3.mp3", "sample-ogg-vorbis.oga", "sample-wav-pcm.wav"] {
        let bytes = fixture_bytes(name);
        let ok = zero_media::AudioDecoder::open(&bytes).is_ok();
        println!("C4PAIR decode-matrix fixture={name} ok={ok}");
        assert!(ok, "音频实解失败: {name}");
    }
    let ogg_opus = fixture_bytes("sample-ogg-opus.oga");
    let ok = zero_media::open_ogg_opus(&ogg_opus).is_ok();
    println!("C4PAIR decode-matrix fixture=sample-ogg-opus.oga ok={ok}");
    assert!(ok, "ogg-opus 实解失败");
}

#[cfg(feature = "media-process")]
mod hosted {
    use super::*;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};

    use zero_protocol::media::MediaReadyParams;
    use zero_protocol::message::{IpcMessage, IpcMessageKind};
    use zero_protocol::transport::PipeTransport;
    use zero_protocol::{IpcChannel, ProcessRole, child_process_args};
    use zero_webview::media_proxy::MediaHostClient;

    /// 解析 target 目录下的 zero-media-host（契约测试同构：current_exe 上两级）。
    fn built_media_host_bin() -> Option<PathBuf> {
        let exe = std::env::current_exe().ok()?;
        let debug_dir = exe.parent()?.parent()?;
        let candidate = debug_dir.join("zero-media-host");
        candidate.is_file().then_some(candidate)
    }

    /// `ZW_MEDIA_HOST_BIN` RAII（互斥由 `#[serial]` 保证）。
    struct HostBinGuard;
    impl HostBinGuard {
        fn set() -> Self {
            let Some(bin) = built_media_host_bin() else {
                panic!("zero-media-host 未构建——先 `cargo build -p zero-media-host`");
            };
            // SAFETY：测试进程单线程面对该变量（`#[serial]` 互斥所有宿主态测试），
            // 且无并发读方（契约测试 EnvGuard 同构）。
            unsafe { std::env::set_var("ZW_MEDIA_HOST_BIN", bin) };
            Self
        }
    }
    impl Drop for HostBinGuard {
        fn drop(&mut self) {
            // SAFETY：同 set——`#[serial]` 互斥窗口内执行。
            unsafe { std::env::remove_var("ZW_MEDIA_HOST_BIN") };
        }
    }

    /// 直连握手读 host 实报能力（客户端握手后不存 capabilities——真值对照
    /// 经管道直读，产品面零改动）。
    fn host_capabilities(bin: &PathBuf) -> MediaReadyParams {
        let mut child: Child = Command::new(bin)
            .args(child_process_args(ProcessRole::Media, 0))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("media-host 可 spawn");
        let mut channel = PipeTransport::new(child.stdout.take().unwrap(), child.stdin.take().unwrap());
        let ready = match channel.recv() {
            Ok(IpcMessage {
                kind: IpcMessageKind::MediaReady(params),
                ..
            }) => params,
            Ok(other) => panic!("media-host 首消息非 MediaReady: {other:?}"),
            Err(e) => panic!("media-host 握手读取失败: {e}"),
        };
        // stdin 关闭 → host 读线程 EOF → 进程退出（对齐骨架断开语义）。
        drop(channel);
        let _ = child.wait();
        ready
    }

    /// T-P3：能力真值三对照（§12.5）——host 实报 vs 实解（页面可见面为 engine
    /// shim 静态表 `canPlayType`，见 ANALYSIS 对照表）。附加 quota 面
    /// （register 超限 → QuotaExceeded，IF-002）。
    #[test]
    #[serial_test::serial]
    #[ignore = "C4 宿主态（spawn 真实二进制）——显式运行（evidence 协议）"]
    fn c4_capability_truth_host_vs_decode() {
        let _guard = HostBinGuard::set();
        let bin = built_media_host_bin().unwrap();
        let ready = host_capabilities(&bin);
        let caps = &ready.capabilities;
        println!(
            "C4PAIR host-capabilities protocol_v={} list={caps:?}",
            ready.protocol_version
        );
        assert!(!caps.is_empty(), "host 能力表非空");
        for (claim, fixture, probe_ok) in [
            ("video/webm; codecs=vp9", "sample-webm-vp9.webm", true),
            ("audio/webm; codecs=vorbis", "sample-webm-vp9-vorbis.webm", true),
            ("audio/mpeg", "sample-mp3.mp3", true),
            ("audio/ogg; codecs=opus", "sample-ogg-opus.oga", true),
            // D4 能力清单对齐（C4 ANALYSIS §四/§六 D4）：原欠申报两条目（ogg-vorbis/
            // wav，symphonia probe 面早已可解）转正式主张；aac 以 sample-mp4-h264.mp4
            // 的 AAC LC 音轨验证（feature 关时 open_media 报 NoVideoTrack 回落
            // AudioDecoder::open 的 symphonia isomp4+aac 路由——README 生成记录在案，
            // 不新增 fixture）。
            ("audio/mp4; codecs=aac", "sample-mp4-h264.mp4", true),
            ("audio/ogg; codecs=vorbis", "sample-ogg-vorbis.oga", true),
            ("audio/wav", "sample-wav-pcm.wav", true),
            // feature 门控主张（默认关）——不虚报面。
            (
                "video/webm; codecs=av1",
                "sample-webm-av1.webm",
                cfg!(feature = "decode-av1"),
            ),
            (
                "video/mp4; codecs=h264",
                "sample-mp4-h264.mp4",
                cfg!(feature = "decode-h264"),
            ),
        ] {
            let in_caps = caps.iter().any(|c| c == claim);
            assert_eq!(in_caps, probe_ok, "能力主张与 feature 门控不符: {claim}");
            if probe_ok {
                let bytes = fixture_bytes(fixture);
                let ok = zero_media::VideoTrackDecoder::open_media(&bytes).is_ok()
                    || zero_media::AudioDecoder::open(&bytes).is_ok()
                    || zero_media::open_ogg_opus(&bytes).is_ok();
                assert!(ok, "主张 {claim} 但 fixture {fixture} 实解失败");
                println!("C4PAIR capability-verified claim={claim} fixture={fixture} decode=ok");
            }
        }
        // audio/webm; codecs=opus：webm 容器 Opus 轨不在 symphonia probe 面，走
        // `open_webm_opus_audio_track` 直解（与 zero-media 常驻单测
        // `webm_av_opus_track_decode_chain` 同源资产 movie_5.webm——wpt-data fetch
        // 白名单按需拉取、不入库；VP9+Opus 双轨只验音频轨主张本身）。
        assert!(
            caps.iter().any(|c| c == "audio/webm; codecs=opus"),
            "主张缺失: audio/webm; codecs=opus"
        );
        let mut opus_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        opus_path.pop(); // crates/
        opus_path.pop(); // workspace root
        opus_path.push("tests/wpt-runner/wpt-data/media/movie_5.webm");
        let opus_bytes = std::fs::read(&opus_path).unwrap_or_else(|e| {
            panic!("movie_5.webm 读取失败（先跑 tests/wpt-runner/scripts/fetch-media-subset.sh）: {e}")
        });
        let mut opus_track = zero_media::open_webm_opus_audio_track(&opus_bytes)
            .expect("webm Opus 轨实解（audio/webm; codecs=opus 主张真值）");
        assert!(opus_track.next_batch().unwrap().is_some(), "Opus 轨应产出数据包");
        println!(
            "C4PAIR capability-verified claim=audio/webm; codecs=opus fixture=wpt-data/media/movie_5.webm decode=ok"
        );

        // quota 面（IF-002）：登记 total_len 超限 → QuotaExceeded（只声明不传字节）。
        let mut client = MediaHostClient::new();
        let err = client
            .register_source(1, MEDIA_SOURCE_MAX_BYTES + 1, "video/webm")
            .expect_err("超限登记须报错");
        println!("C4PAIR quota code={:?} message={}", err.code, err.message);
        assert_eq!(err.code, zero_protocol::media::MediaErrorCode::QuotaExceeded);
    }

    /// T-P4：代际风暴（§12.3 旧 epoch 帧/状态零泄漏）——交叉 seek 重建内核流后
    /// 收口到风暴未访问窗，采集后续 30 帧逐一对照参照序列局部窗（旧代际帧/
    /// 越窗帧即失败）。
    #[test]
    #[serial_test::serial]
    #[ignore = "C4 宿主态长跑——显式运行（evidence 协议）"]
    fn c4_epoch_storm_no_stale_frames() {
        let _guard = HostBinGuard::set();
        let bytes = fixture_bytes("long-webm-vp9-60s.webm");
        let reference = reference_video_hashes(&bytes);
        let mut reg = VideoPlayerRegistry::new();
        let mut cache = ImageCache::default();
        reg.register_source(VP9_60S, bytes);
        assert!(reg.play(VP9_60S, 0), "风暴前 play 就绪");
        // 风暴：交叉 seek（每次内核重建流、epoch+1）+ 回执窗消费。
        for round in 0..3 {
            for target_ms in [45_000u64, 15_000, 30_000] {
                assert!(reg.seek(VP9_60S, target_ms), "round {round} seek {target_ms} 失败");
                for now in 0..10u64 {
                    let _ = reg.tick_all(now * 16, &mut cache);
                }
            }
        }
        // 收口：跳风暴未访问窗（57.5s），真实节拍采集 30 帧。
        assert!(reg.seek(VP9_60S, 57_500), "收口 seek 失败");
        let mut captured = Vec::new();
        let started = Instant::now();
        let mut next_due = started;
        while captured.len() < 30 {
            let _ = reg.tick_all(started.elapsed().as_millis() as u64, &mut cache);
            let _ = reg.audio_advance_all(started.elapsed().as_millis() as u64);
            if let Some(h) = presented_hash(&cache) {
                if captured.last() != Some(&h) {
                    captured.push(h);
                }
            }
            next_due += Duration::from_millis(16);
            let now2 = Instant::now();
            if next_due > now2 {
                std::thread::sleep(next_due - now2);
            }
            assert!(
                started.elapsed() < Duration::from_secs(8),
                "8s 未采满 30 帧（呈现断流）"
            );
        }
        println!(
            "C4PAIR epoch-storm captured={} digest={:016x} ct_end={:.3} errored={}",
            captured.len(),
            fold_digest(&captured),
            reg.current_time(VP9_60S),
            reg.is_errored(VP9_60S),
        );
        assert!(!reg.is_errored(VP9_60S), "风暴后不落 error 面");
        // 局部窗比对：期望首帧 ≈ 57.5s×24fps = 1380。母本簇 ×30 使帧内容以 48 帧
        // 为周期重复——窗内同内容多索引，取**距期望最近**命中（最低命中会把周期
        // 等价帧误判为回跳）；|命中−期望| ≤ 48（±2s）容 pts 对齐抖动，同时仍能
        // 抓住风暴旧位（15s/30s/45s ⇒ 索引 360/720/1080，距收口窗 ≥ 300）陈旧帧。
        let mut expected = 1380usize;
        for (i, h) in captured.iter().enumerate() {
            let lo = expected.saturating_sub(96);
            let hi = (expected + 96).min(reference.len() - 1);
            let best = (lo..=hi)
                .filter(|&j| reference[j] == *h)
                .min_by_key(|&j| j.abs_diff(expected));
            assert!(
                best.is_some(),
                "第 {i} 帧哈希不在参照局部窗 [{lo},{hi}]（旧代际帧或越窗帧泄漏）"
            );
            let idx = best.unwrap();
            assert!(
                idx.abs_diff(expected) <= 48,
                "第 {i} 帧越出期望邻域（idx={idx} expected={expected}）——陈旧帧泄漏"
            );
            expected = (idx + 1).min(reference.len() - 1);
        }
    }

    /// 宿主进程 CPU 时间（utime+stime，ms；M1 `cpu_ms` 同构，/proc/<pid>/stat）。
    fn proc_cpu_ms(pid: u32) -> Option<f64> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        let after_paren = stat.rsplit(')').next()?;
        let fields: Vec<&str> = after_paren.split_whitespace().collect();
        let utime: f64 = fields.get(11)?.parse().ok()?;
        let stime: f64 = fields.get(12)?.parse().ok()?;
        Some((utime + stime) * 10.0)
    }

    /// 宿主 RSS（VmRSS kB）。
    fn proc_rss_kb(pid: u32) -> Option<u64> {
        let status = std::fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
        let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
        line.split_whitespace().nth(1)?.parse().ok()
    }

    /// 媒体 shm 段盘点（/dev/shm/zeroweb-media-*，IF-004 槽位面）。
    fn shm_segments() -> (usize, u64) {
        let mut count = 0usize;
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir("/dev/shm") {
            for e in entries.flatten() {
                if let Some(name) = e.file_name().to_str() {
                    if name.starts_with("zeroweb-media-") {
                        count += 1;
                        total += e.metadata().map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
        }
        (count, total)
    }

    /// T-P5：30min 资源审计（§12.4）——宿主态生产形态长播（AV 60s ×30 重播
    /// 风暴），采样进程树/RSS/CPU/shm；暂停态空闲成本 + 退出孤儿核对。
    /// 时钟面：跨轮单调递增（真实消费方性能钟不回拨——回拨基会让重播坍缩，
    /// 首跑已证），每轮偏移 61s（> 60s 流长，严格前向）。
    #[test]
    #[serial_test::serial]
    #[ignore = "C4 资源审计（>30min 实时）——显式单独运行（evidence 协议）"]
    fn c4_resource_audit_30min() {
        let _guard = HostBinGuard::set();
        let bytes = fixture_bytes("long-webm-vp9-vorbis-60s.webm");
        let mut reg = VideoPlayerRegistry::new();
        let mut cache = ImageCache::default();
        reg.register_source(AV_60S, bytes);

        let mut pids: Vec<u32> = Vec::new();
        let mut rss_peak_kb = 0u64;
        let mut rss_samples: Vec<u64> = Vec::new();
        let mut idle_cpu_ms = 0.0f64;
        let rounds = 30u32;
        let mut base_ms = 0u64;
        for round in 0..rounds {
            assert!(reg.play(AV_60S, base_ms), "round {round} play");
            // 暂停态空闲成本探针（§12.4「暂停态 host CPU ≈ 0」）：中段轮次抽一次；
            // 探完 play() 复播（paused 语义清除），本轮仍须完整重播。
            if round == rounds / 2 {
                reg.pause(AV_60S);
                if let Some(pid) = reg.media_host_pid() {
                    let cpu0 = proc_cpu_ms(pid).unwrap_or(0.0);
                    std::thread::sleep(Duration::from_secs(5));
                    idle_cpu_ms = proc_cpu_ms(pid).unwrap_or(0.0) - cpu0;
                    println!("C4AUDIT idle-probe round={round} cpu_delta_5s_ms={idle_cpu_ms:.0}");
                }
                assert!(reg.play(AV_60S, base_ms), "round {round} 复播");
            }
            let mut tick: u32 = 0;
            let mut sample_tick: u32 = 0;
            while reg.is_any_playing() {
                let now_ms = base_ms + u64::from(tick) * 16;
                let _ = reg.tick_all(now_ms, &mut cache);
                let _ = reg.audio_advance_all(now_ms);
                std::thread::sleep(Duration::from_millis(16));
                tick += 1;
                // 每 ~30s 采样一次（1875 tick ≈ 30s）。
                if tick - sample_tick >= 1875 {
                    sample_tick = tick;
                    if let Some(pid) = reg.media_host_pid() {
                        pids.push(pid);
                        let rss = proc_rss_kb(pid).unwrap_or(0);
                        rss_peak_kb = rss_peak_kb.max(rss);
                        rss_samples.push(rss);
                        let (shm_n, shm_bytes) = shm_segments();
                        println!(
                            "C4AUDIT sample round={round} tick={tick} pid={pid} rss_kb={rss} host_cpu_ms={:.0} shm_segments={shm_n} shm_bytes={shm_bytes}",
                            proc_cpu_ms(pid).unwrap_or(0.0),
                        );
                    }
                }
                assert!(tick < 6000, "round {round} 60s 未到流末（实时性缺口）");
            }
            // 完整重播核对：60s 流 @16ms tick ≈ 3750 tick 到流末——早退（重启
            // 后即 Ended）会让审计对「30 次长播」说谎，显式拒绝。阈值留 EOF
            // 检测抖动余量（音频游标到末的 tick 数轮间 ~1s 波动，实测 3690）。
            assert!(
                tick >= 3500,
                "round {round} 仅 {tick} tick 即退出——重播未走满 60s（宿主重启后 EOS 残留？）"
            );
            base_ms += 61_000;
        }
        let (shm_n, shm_bytes) = shm_segments();
        // IF-001：整个审计单 host（无按源/按轮 spawn）。
        let single_host = pids.iter().all(|&p| p == pids[0]);
        // 孤儿核对：registry Drop → shutdown → 宿主进程退出（≤5s）。
        let last_pid = reg.media_host_pid();
        drop(reg);
        let mut orphan = false;
        if let Some(pid) = last_pid {
            let deadline = Instant::now() + Duration::from_secs(5);
            while Instant::now() < deadline && std::path::Path::new(&format!("/proc/{pid}")).exists() {
                std::thread::sleep(Duration::from_millis(100));
            }
            orphan = std::path::Path::new(&format!("/proc/{pid}")).exists();
        }
        rss_samples.sort_unstable();
        let steady = rss_samples.get(rss_samples.len() / 2).copied().unwrap_or(0);
        println!(
            "C4AUDIT summary rounds={rounds} samples={} single_host={single_host} pid_stable={:?} rss_peak_kb={rss_peak_kb} rss_steady_kb={steady} host_cpu_total_ms={:.0} shm_segments={shm_n} shm_bytes={shm_bytes} idle_cpu_5s_ms={idle_cpu_ms:.0} orphan={orphan}",
            pids.len(),
            last_pid,
            pids.last().map(|&p| proc_cpu_ms(p).unwrap_or(0.0)).unwrap_or(0.0),
        );
        assert!(single_host, "IF-001 单宿主被破坏（出现多 pid）");
        assert!(!orphan, "宿主进程未随 shutdown 退出（孤儿）");
        assert!(shm_n == 0, "未预期的媒体 shm 段（当前传输面为内联 rgba）");
        assert!(
            idle_cpu_ms < 250.0,
            "暂停态 5s CPU {idle_cpu_ms}ms 越出空闲界（事件循环未停等）"
        );
        assert!(rss_peak_kb < 512 * 1024, "宿主 RSS 峰值 {rss_peak_kb}kB 越出 512MB 界");
    }
}
