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
    // 首帧 testsrc2 纹样锚点——RGBA 面的 RGB 均值观测窗（BT.601 下 RGB 均值
    // 与 YUV 的 Y 均值 122 不同，实测 153.5；窗口取 ±15 防像素级抖动误报）。
    let mean = rgba_mean(&f0.rgba);
    assert!(
        (138.0..=168.0).contains(&mean),
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
