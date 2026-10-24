//! M1 媒体基线测量（media-runtime-isolation goal——`#[ignore]` 长跑，显式运行）。
//!
//! 测量生产帧消费链：`VideoPlayerRegistry::tick_all` → `ImageCache`（renderer
//! runtime.rs 泵同路径，16ms 节拍）。产出 `M1BASELINE key=value` 行供 evidence
//! 记录解析。长 fixture 由 `scripts/make-long-video-fixture.py` 生成（来源 =
//! 仓内 2s fixture 簇重复，见 tests/fixtures/media/README.md）。
//!
//! 运行（先无限制编译，执行阶段走 test-guard 包裹）：
//! `cargo test -p zero-webview --test media_m1_baseline --no-run`
//! `./target/test-guard --time-limit 900 --per-proc-mem 4 -- <测试二进制> --ignored --nocapture --test-threads=1`

use std::time::{Duration, Instant};

use zero_media::VideoClock;
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::video_registry::VideoPlayerRegistry;

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

/// 进程 CPU 时间（utime+stime，毫秒；Linux /proc/self/stat，CLK_TCK=100 假设）。
fn cpu_ms() -> f64 {
    let stat = std::fs::read_to_string("/proc/self/stat").expect("Linux /proc/self/stat");
    let after_paren = stat.rsplit(')').next().expect("stat 形态");
    let fields: Vec<&str> = after_paren.split_whitespace().collect();
    // rsplit 从右起：fields[11]=utime、fields[12]=stime（相对全字段 14/15）。
    let utime: f64 = fields[11].parse().unwrap_or(0.0);
    let stime: f64 = fields[12].parse().unwrap_or(0.0);
    (utime + stime) * 10.0 // 100Hz tick → ms
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn report(stats: &[u64], label: &str) {
    let mut v: Vec<f64> = stats.iter().map(|&x| x as f64).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    let sum: f64 = v.iter().sum();
    println!(
        "M1BASELINE tick_us label={label} n={n} p50={:.1} p95={:.1} p99={:.1} max={:.1} mean_us={:.1}",
        percentile(&v, 0.50),
        percentile(&v, 0.95),
        percentile(&v, 0.99),
        v.last().copied().unwrap_or(0.0),
        if n > 0 { sum / n as f64 / 1000.0 } else { 0.0 },
    );
}

/// 泵推进到流末（16ms 模拟节拍；生产 runtime.rs 同步长）。
/// 返回 (逐 tick 耗时 ns, 帧更新 tick 数)。
fn pump_to_end(reg: &mut VideoPlayerRegistry, cache: &mut ImageCache, src: &str, max_ticks: u32) -> (Vec<u64>, u32) {
    let mut per_tick_ns = Vec::new();
    let mut present_ticks = 0u32;
    let mut now = 0u64;
    for _ in 0..max_ticks {
        let t0 = Instant::now();
        if reg.tick_all(now, cache) {
            present_ticks += 1;
        }
        let _ = reg.audio_advance_all(now);
        per_tick_ns.push(t0.elapsed().as_nanos() as u64);
        now += 16;
        if !reg.is_any_playing() {
            break;
        }
    }
    assert!(
        !reg.is_any_playing(),
        "{src} 在 {max_ticks} tick（{now} ms 媒体时间）内未到流末"
    );
    (per_tick_ns, present_ticks)
}

/// T1：纯视频 60s 容量基线（解码 + 帧注入 ImageCache 全链）。
#[test]
#[ignore = "M1 长跑基线——显式运行（evidence 协议）"]
fn m1_capacity_pure_video_60s() {
    let bytes = fixture_bytes("long-webm-vp9-60s.webm");
    // fixture 校验面：open_media 声明时长/首帧尺寸真值。
    let mut probe = zero_media::VideoTrackDecoder::open_media(&bytes).expect("长 fixture 可解码");
    assert_eq!(probe.duration_ms(), Some(60000), "容器声明 60s");
    let f0 = probe.next_frame().unwrap().expect("首帧可解");
    assert_eq!((f0.width, f0.height), (320, 240));

    let mut reg = VideoPlayerRegistry::new();
    let mut cache = ImageCache::default(); // 生产预算：256 entries / 64MB
    reg.register_source(VP9_60S, bytes);
    assert!(reg.play(VP9_60S, 0), "60s VP9 play 建播放器");
    assert_eq!(reg.duration(VP9_60S), Some(60.0));

    let cpu0 = cpu_ms();
    let wall0 = Instant::now();
    let (ticks, present_ticks) = pump_to_end(&mut reg, &mut cache, VP9_60S, 6000);
    let wall = wall0.elapsed();
    let cpu = cpu_ms() - cpu0;

    let media_ms = reg.current_time(VP9_60S);
    println!(
        "M1BASELINE scenario=pure-video-60s wall_ms={} cpu_ms={:.0} cores={:.3} ticks={} present_ticks={present_ticks} media_end_s={media_ms:.3} cpu_per_present_ms={:.3}",
        wall.as_millis(),
        cpu,
        cpu / wall.as_millis() as f64,
        ticks.len(),
        cpu / present_ticks.max(1) as f64
    );
    report(&ticks, "pure-video-60s");
}

/// T2：A/V 双轨 60s 容量基线（音频主时钟 + 视频对齐 + 音频泵写 sink）。
#[test]
#[ignore = "M1 长跑基线——显式运行（evidence 协议）"]
fn m1_capacity_av_pair_60s() {
    let bytes = fixture_bytes("long-webm-vp9-vorbis-60s.webm");
    let mut reg = VideoPlayerRegistry::new();
    let mut cache = ImageCache::default();
    reg.register_source(AV_60S, bytes);
    assert!(reg.play(AV_60S, 0), "60s A/V pair play");
    assert_eq!(reg.duration(AV_60S), Some(60.09), "双轨容器声明 60.09s");

    let cpu0 = cpu_ms();
    let wall0 = Instant::now();
    let (ticks, present_ticks) = pump_to_end(&mut reg, &mut cache, AV_60S, 6000);
    let wall = wall0.elapsed();
    let cpu = cpu_ms() - cpu0;

    let media_ms = reg.current_time(AV_60S);
    println!(
        "M1BASELINE scenario=av-pair-60s wall_ms={} cpu_ms={:.0} cores={:.3} ticks={} present_ticks={present_ticks} media_end_s={media_ms:.3}",
        wall.as_millis(),
        cpu,
        cpu / wall.as_millis() as f64,
        ticks.len()
    );
    report(&ticks, "av-pair-60s");
}

/// T3：真实节拍时钟保真（15s 段，16ms 实际 sleep 泵）——media clock 速率 ≈1。
#[test]
#[ignore = "M1 长跑基线——显式运行（evidence 协议）"]
fn m1_clock_fidelity_realtime_15s() {
    let bytes = fixture_bytes("long-webm-vp9-vorbis-60s.webm");
    let mut reg = VideoPlayerRegistry::new();
    let mut cache = ImageCache::default();
    reg.register_source(AV_60S, bytes);
    assert!(reg.play(AV_60S, 0));

    let start = Instant::now();
    let mut samples: Vec<(u64, f64)> = Vec::new(); // (墙钟 ms, media s)
    let mut tick_us: Vec<u64> = Vec::new();
    let mut next_due = start;
    for _ in 0..=938 {
        // 938 × 16ms ≈ 15.008s
        let now = start.elapsed().as_millis() as u64;
        let t0 = Instant::now();
        let _ = reg.tick_all(now, &mut cache);
        let _ = reg.audio_advance_all(now);
        tick_us.push(t0.elapsed().as_nanos() as u64);
        samples.push((now, reg.current_time(AV_60S)));
        next_due += Duration::from_millis(16);
        let now2 = Instant::now();
        if next_due > now2 {
            std::thread::sleep(next_due - now2);
        }
    }
    let (wall_last, media_last) = *samples.last().expect("采样非空");
    let rate = media_last / (wall_last as f64 / 1000.0);
    let mut offsets: Vec<f64> = samples.iter().map(|(w, m)| (m - *w as f64 / 1000.0).abs()).collect();
    offsets.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    println!(
        "M1BASELINE scenario=clock-fidelity-rt-15s rate={:.5} max_offset_ms={:.2} p95_offset_ms={:.2} end_media_s={media_last:.3}",
        rate,
        offsets.last().copied().unwrap_or(0.0) * 1000.0,
        percentile(&offsets, 0.95) * 1000.0,
    );
    report(&tick_us, "clock-fidelity-rt");
    // A1 修复后收带（2026-10-24 RFC D-RFC-2 批复，DESIGN §2.3）：样本级队列
    // （BatchSampleQueue）按 tick 目标帧数精确消费，rate ≡ 1.0 由构造保证；带宽
    // 留 EOF/睡眠抖动裕度。修复前批粒度整批推进实测 rate≈1.45（M1 FINDING）。
    println!("M1BASELINE a1=clock-rate-aligned rate={rate:.5}");
    assert!(
        (0.9..=1.1).contains(&rate),
        "media clock 速率 {rate} 越出 A1 对齐带 [0.9, 1.1]"
    );
}

/// T4：cue 路径 seek 基线（重建 Cues 的长 fixture——cue 二分命中 keyframe）
/// + seek 后续播到流末的参考链完整性 + A4 窄化断言（mid-GOP 冷 seek 有界且
///   不随 target 线性增长——cue 锚定前向解码 O(GOP)，DESIGN §3.4）。
#[test]
#[ignore = "M1 长跑基线——显式运行（evidence 协议）"]
fn m1_seek_cue_path() {
    let bytes = fixture_bytes("long-webm-vp9-60s.webm");
    let mut reg = VideoPlayerRegistry::new();
    let mut cache = ImageCache::default();
    reg.register_source(VP9_60S, bytes);
    let mut mid_gop_seek_us: Vec<(f64, u128)> = Vec::new();
    for target_s in [15.0f64, 30.0, 45.0] {
        let target_ms = (target_s * 1000.0) as u64;
        // 每轮先 play（首轮建播放器；后续轮 Ended→play 重头语义，解码器重建）。
        assert!(reg.play(VP9_60S, 0), "seek 前 play 就绪");
        let t0 = Instant::now();
        assert!(reg.seek(VP9_60S, target_ms), "seek {target_s}s 命中");
        let seek_us = t0.elapsed().as_micros();
        let ct = reg.current_time(VP9_60S);
        println!("M1BASELINE scenario=seek-cue target_s={target_s} seek_us={seek_us} after_ct_s={ct:.3}");
        if target_s != 30.0 {
            mid_gop_seek_us.push((target_s, seek_us));
        }
        assert!(
            ct >= target_s - 0.05 && ct <= target_s + 1.0,
            "seek 后位置 {ct} 应落在 {target_s}s 附近"
        );
        // seek 后续播到流末（seek 不改 paused——play 恢复播放态）。
        assert!(reg.play(VP9_60S, 0), "seek 后恢复播放");
        let (ticks, _) = pump_to_end(&mut reg, &mut cache, VP9_60S, 6000);
        println!(
            "M1BASELINE scenario=seek-resume target_s={target_s} resume_ticks={} end_s={:.3}",
            ticks.len(),
            reg.current_time(VP9_60S)
        );
    }
    // A4 断言（DESIGN §3.4）：mid-GOP 冷 seek ≤ GOP 前向上界（48 帧 GOP × 单帧
    // 解码 ~0.8ms × 2.5 安全系数 ≈ 100ms），且 15s vs 45s 差 < GOP 解码耗时上限
    //（50ms）——全量前向形态下 45s ≈ 3× 15s（基线 302ms/1000ms），窄化后两者同量级。
    for &(s, us) in &mid_gop_seek_us {
        assert!(us <= 100_000, "A4 mid-GOP seek({s}s) {us}us 越出 GOP 前向上界 100ms");
    }
    let s15 = mid_gop_seek_us.iter().find(|&&(s, _)| s == 15.0).unwrap().1;
    let s45 = mid_gop_seek_us.iter().find(|&&(s, _)| s == 45.0).unwrap().1;
    assert!(
        s45.abs_diff(s15) < 50_000,
        "A4 seek 不应随 target 线性增长（15s {s15}us vs 45s {s45}us 差 ≥ 50ms）"
    );
    println!("M1BASELINE a4=seek-narrowed mid_gop_seek_us={s15}/{s45}");
}

/// T5：media 层 A/V 漂移真值（audio cursor 主时钟 vs 视频呈现 pts，VideoPlayer
/// `presented_pts_ms`/sync 路径直读——registry 面不透出 pts，此测补真值）。
#[test]
#[ignore = "M1 长跑基线——显式运行（evidence 协议）"]
fn m1_av_drift_media_layer() {
    let bytes = fixture_bytes("long-webm-vp9-vorbis-60s.webm");
    let video = zero_media::VideoTrackDecoder::open_media(&bytes).expect("视频轨可开");
    let mut track = zero_media::open_webm_audio_track(&bytes).expect("音频轨可开");
    let mut player = zero_media::VideoPlayer::new(video);
    player.play(0);

    let mut cursor_ms = 0u64;
    let mut drifts: Vec<f64> = Vec::new();
    let mut now = 0u64;
    let mut presented = 0u32;
    while player.is_playing() && now < 70_000 {
        now += 16;
        // 音频主时钟先行：解码一批计游标（一拍 ≈16ms 媒体时间，registry 同序）。
        while cursor_ms < now {
            match track.next_batch() {
                Ok(Some(batch)) => {
                    cursor_ms = batch.pts_ms
                        + (batch.samples.len() as u64 * 1000)
                            / (u64::from(batch.sample_rate) * u64::from(batch.channels).max(1));
                }
                _ => break,
            }
        }
        if let Ok(Some(frame)) = player.sync_to_media_time(cursor_ms as f64) {
            presented += 1;
            // 漂移 = 主时钟游标 − 已呈现帧 pts（呈现延迟；正值为视频落后音频）。
            drifts.push((cursor_ms as f64 - frame.pts_ms as f64).max(0.0));
        }
    }
    drifts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = drifts.len();
    let sum: f64 = drifts.iter().sum();
    println!(
        "M1BASELINE scenario=av-drift-media presented_frames={presented} drift_ms p50={:.1} p95={:.1} max={:.1} mean={:.1} audio_cursor_end_ms={cursor_ms}",
        percentile(&drifts, 0.50),
        percentile(&drifts, 0.95),
        drifts.last().copied().unwrap_or(0.0),
        if n > 0 { sum / n as f64 } else { 0.0 },
    );
    assert!(presented > 1000, "60s@24fps 应呈现 >1000 帧（got {presented}）");
}
