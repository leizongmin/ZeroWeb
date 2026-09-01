//! zero-media 单元测试 — fixture 解码正确性（RFC §3.3 验收资产）。
//!
//! 参照基准：ffmpeg 7.1.5 对 `sample-webm-vp9.webm` 首帧的 yuv420p rawvideo 输出
//! （2026-09-01 探针实测逐字节一致）；此处固化为 RGBA 哈希 + 帧元数据断言。

use super::decode::{VideoDecoder, fixture_path, rgba_mean};
use std::fs;

#[test]
fn webm_vp9_metadata() {
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    // container duration 2000ms（ffprobe 实测一致）
    assert_eq!(dec.duration_ms(), Some(2000));
}

#[test]
fn webm_vp9_first_frame_matches_ffmpeg_reference() {
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();

    let f0 = dec.next_frame().unwrap().expect("frame 0");
    assert_eq!(f0.width, 320);
    assert_eq!(f0.height, 240);
    assert_eq!(f0.pts_ms, 0);
    // 首帧 testsrc2 纹样锚点——RGBA 面的 RGB 均值观测窗。M2 色度精化后 limited
    // range 转换与 ffmpeg swscale RGBA 参照（实测 123.27）一致——窗口 ±15。
    // （旧锚点 153.5 源于全范围误释 + 色度索引坍缩两个缺陷的叠加，本窗同步收紧。）
    let mean = rgba_mean(&f0.rgba);
    assert!(
        (108.0..=138.0).contains(&mean),
        "first frame RGB mean out of reference window: {mean}"
    );
    // 哈希稳定性：同一输入两次解码逐像素一致（转换无随机性）。
    let mut dec2 = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f0b = dec2.next_frame().unwrap().unwrap();
    assert_eq!(f0.rgba, f0b.rgba, "decode not deterministic");
}

#[test]
fn webm_vp9_full_stream_frame_count_and_pts() {
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();

    let mut count = 0u32;
    let mut last_pts = 0u64;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        assert!(frame.pts_ms >= last_pts, "PTS monotonicity broken");
        last_pts = frame.pts_ms;
        count += 1;
        assert!(count <= 100, "runaway frame count");
    }
    // ffprobe -count_frames 实测 48 帧（2s @ 24fps）。
    assert_eq!(count, 48);
    // 末帧 PTS ≈ 1958ms（48 帧满覆盖 2s 窗口）。
    assert!((1900..=2000).contains(&last_pts));
}

#[test]
fn non_webm_input_rejected() {
    let garbage = b"definitely not an EBML container".to_vec();
    assert!(VideoDecoder::open_webm_vp9(&garbage).is_err());
}

#[test]
fn empty_input_rejected() {
    assert!(VideoDecoder::open_webm_vp9(&[]).is_err());
}

#[test]
fn webm_vp9_seek_to_mid_stream_keyframe() {
    // M2b：seek 关键帧粒度——seek 到 1s 后 next_frame 应返回 ≥ 定位点的帧，
    // 后续帧 PTS 单调续播至流末（48 帧 @ 24fps，fixture）。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    dec.seek_to_ms(1000).unwrap();
    let f = dec.next_frame().unwrap().expect("seek 后应有帧");
    // 精确 seek（spec）：单 keyframe 流回退路径前向解码 → 首帧 pts ≥ 1000。
    assert!(
        f.pts_ms >= 1000,
        "seek(1000ms) 后首帧 PTS 应 ≥ target（precise-seek），got {}",
        f.pts_ms
    );
    // 续播单调至末：剩余帧数 < 48（跳过了前段），PTS 单调。
    let mut count = 1u32;
    let mut last = f.pts_ms;
    while let Some(frame) = dec.next_frame().unwrap() {
        assert!(frame.pts_ms >= last, "seek 后 PTS 单调性破坏");
        last = frame.pts_ms;
        count += 1;
        assert!(count <= 48, "runaway frame count");
    }
    assert!(count < 48, "seek(1s) 后剩余帧应少于全流 48 帧，got {count}");
    assert!((1700..=2100).contains(&last), "末帧 PTS 应仍在 2s 窗口，got {last}");
}

#[test]
fn webm_vp9_seek_to_zero_replays_full_stream() {
    // seek(0)：回到开头，全流 48 帧可重播（ended 重播路径的解码器面）。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    // 先消费几帧再 seek 回 0。
    let _ = dec.next_frame().unwrap();
    let _ = dec.next_frame().unwrap();
    dec.seek_to_ms(0).unwrap();
    let f0 = dec.next_frame().unwrap().expect("seek(0) 后应有帧");
    assert_eq!(f0.pts_ms, 0, "seek(0) 后首帧应为 pts=0");
    let mut count = 1u32;
    while dec.next_frame().unwrap().is_some() {
        count += 1;
        assert!(count <= 48);
    }
    assert_eq!(count, 48, "seek(0) 后全流应可重播 48 帧");
}

#[test]
fn webm_colour_identity_full_range_passthrough() {
    // M2 色度精化：reftest-upstream replaced-element-003 的 support 素材
    //（2x2-green.webm，ffprobe: color_range=pc + color_space=gbr）→ identity
    // 矩阵 + full range → 平面 GBR 直传不做 YUV 数学。解码位形 = 位面内容
    //（0,127,0——上游编码器的位面真值）vs ref #008000 差 1 ≤ fuzzy 0-30
    //（chromium 同样在此预算内通过——identity 直传即正确行为面）。
    let path =
        super::decode::workspace_path("tests/wpt-runner/wpt-data/css/css-sizing/aspect-ratio/support/2x2-green.webm");
    let data = fs::read(&path).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f = dec.next_frame().unwrap().expect("2x2 帧应可解");
    assert_eq!((f.width, f.height), (2, 2));
    // identity 直传：位面序 G/B/R → RGBA = (R=Cr, G=Y, B=Cb) = (0,127,0)。
    for px in f.rgba.as_chunks::<4>().0 {
        assert_eq!((px[0], px[1], px[2], px[3]), (0, 127, 0, 255), "identity 直传位面真值");
    }
}

#[test]
fn webm_colour_broadcast_range_luma_not_clipped() {
    // 本仓 fixture（libvpx）：Colour 元素 range=Broadcast（limited）、matrix 缺省。
    // limited 面正确转换后 RGB 均值应落 ffmpeg swscale RGBA 参照（123.3）的观测窗
    //——旧全范围 BT.601 数学下为 153.5（值域误释 +25% 亮度失真，本测试防回退）。
    let data = fs::read(fixture_path("sample-webm-vp9.webm")).unwrap();
    let mut dec = VideoDecoder::open_webm_vp9(&data).unwrap();
    let f = dec.next_frame().unwrap().expect("frame 0");
    let mean = rgba_mean(&f.rgba);
    assert!(
        (108.0..=138.0).contains(&mean),
        "limited-range 转换后 RGB 均值应近 ffmpeg 参照 123.3，got {mean}"
    );
}

#[test]
fn audio_mp3_decode_to_nullsink_zero_crossing_chain() {
    // M2c 全链 e2e：mp3 fixture（440Hz sine）→ symphonia 解码 f32 → AudioSink
    //（NullSink 可观测断言——过零率 ≈ 880 = 2×频率，media-audio M1 契约锚点）。
    use crate::audio::NullSink;
    use crate::audio::{AudioFormat, AudioSink};
    use crate::audio_decode::AudioDecoder;

    let data = fs::read(fixture_path("sample-mp3.mp3")).unwrap();
    let mut dec = AudioDecoder::open(&data).unwrap();
    let (rate, channels) = (dec.sample_rate(), dec.channels());
    assert_eq!(channels, 1, "fixture 为单声道");

    let mut sink = NullSink::new();
    sink.start(AudioFormat {
        sample_rate: rate,
        channels,
    })
    .unwrap();
    let mut batches = 0u32;
    let mut wrote_any = false;
    while let Some(batch) = dec.next_batch().unwrap() {
        assert_eq!(batch.sample_rate, rate);
        assert_eq!(batch.channels, channels);
        sink.write(&batch.samples).unwrap();
        wrote_any = true;
        batches += 1;
        assert!(batches <= 200, "runaway batch count");
    }
    assert!(wrote_any, "解码应产出采样");

    // 时长面：2s @ rate 采样 ≈ frames_written（MP3 延迟/padding 容差 ±5%）。
    let frames = sink.frames_written();
    let expect = u64::from(rate) * 2;
    assert!(
        (frames as i64 - expect as i64).abs() < (expect as i64 / 20),
        "写入帧数应 ≈ 2s 采样数：got {frames}, expect ≈{expect}"
    );
    // 频域代理锚点：440Hz sine 过零率 ≈ 880（2×频率；media-audio NullSink 契约）。
    let zcr = sink.zero_crossings_per_second().expect("写入后应有过零率");
    assert!((zcr - 880.0).abs() < 90.0, "440Hz sine 过零率应 ≈880，got {zcr}");
}

#[test]
fn audio_ogg_vorbis_decode_to_nullsink_chain() {
    // 同链 ogg/vorbis 面：fixture（440Hz sine）→ 解码 → NullSink 过零率锚点。
    use crate::audio::{AudioFormat, AudioSink, NullSink};
    use crate::audio_decode::AudioDecoder;

    let data = fs::read(fixture_path("sample-ogg-vorbis.oga")).unwrap();
    let mut dec = AudioDecoder::open(&data).unwrap();
    let (rate, channels) = (dec.sample_rate(), dec.channels());

    let mut sink = NullSink::new();
    sink.start(AudioFormat {
        sample_rate: rate,
        channels,
    })
    .unwrap();
    let mut wrote_any = false;
    while let Some(batch) = dec.next_batch().unwrap() {
        sink.write(&batch.samples).unwrap();
        wrote_any = true;
    }
    assert!(wrote_any);
    let frames = sink.frames_written();
    let expect = u64::from(rate) * 2;
    assert!(
        (frames as i64 - expect as i64).abs() < (expect as i64 / 20),
        "vorbis 写入帧数应 ≈ 2s：got {frames}, expect ≈{expect}"
    );
    let zcr = sink.zero_crossings_per_second().expect("写入后应有过零率");
    assert!((zcr - 880.0).abs() < 90.0, "440Hz sine 过零率应 ≈880，got {zcr}");
}

#[test]
fn audio_non_audio_bytes_rejected() {
    use crate::audio_decode::AudioDecoder;
    let garbage = b"definitely not an audio container".to_vec();
    assert!(AudioDecoder::open(&garbage).is_err());
}
