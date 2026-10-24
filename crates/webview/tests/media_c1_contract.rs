//! media-host C1 真实进程契约测试（`media-process` feature 门控 + `#[ignore]` 显式运行）。
//!
//! 与 M1 image-decoder 契约测试同模式：内存通道测试（`apps/media-host` 单测）验证不了
//! stdio 管道帧格式与真实子进程生命周期，须以真实 `zero-media-host` 二进制为对端
//! （zeroweb-knowledge：SharedMemoryChannel 不能替代真实对端验证）。
//!
//! 运行方式（需先构建 media-host 二进制，使 exe 上溯查找链能命中 target/debug）：
//! `./target/test-guard --compile-first ... -- cargo test -p zero-webview
//!  --features media-process --all-targets -- --ignored`

#![cfg(feature = "media-process")]

use std::path::PathBuf;

use zero_protocol::media::MediaErrorCode;
use zero_webview::media_proxy::MediaHostClient;

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

/// 真实二进制全生命周期契约：握手 → register/SourceReady → chunk/Ack →
/// play/seek 错误面（C2 前无解码内核，如实报 HostInternal）→ 配额拒绝后连接仍健康
/// （host MediaError 不置永久失败，IF-006）→ release → shutdown 进程退出。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
fn real_host_lifecycle_and_error_faces() {
    let Some(bin) = built_media_host_bin() else {
        panic!("zero-media-host 未构建：先 cargo build -p zero-media-host 再运行本测试");
    };

    // edition 2024：env 修改为 unsafe（串行运行无并发，安全）。
    let previous = std::env::var_os("ZW_MEDIA_HOST_BIN");
    unsafe {
        std::env::set_var("ZW_MEDIA_HOST_BIN", &bin);
    }
    let mut client = MediaHostClient::new();

    // 登记 + 分片回执（信封 id 配对由 client 断言，这里核语义值）。
    let ready = client.register_source(7, 128, "video/webm").expect("register_source");
    assert_eq!(ready.source_key, 7);
    assert_eq!(ready.duration_ms, None, "C1 骨架无 demux，元数据如实 None");

    let ack = client.send_data_chunk(7, 4, vec![0u8; 64]).expect("send_data_chunk");
    assert_eq!((ack.source_key, ack.last_seq), (7, 4));

    // C1 骨架播放控制如实报 HostInternal（不虚报状态）。
    let play_error = client.play(7, 0).expect_err("play 在 C1 必为错误面");
    assert_eq!(play_error.code, MediaErrorCode::HostInternal);
    assert!(
        play_error.message.contains("not wired"),
        "diagnostic: {}",
        play_error.message
    );
    let seek_error = client.seek(7, 1000).expect_err("seek 在 C1 必为错误面");
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

    // edition 2024：env 修改为 unsafe（串行运行无并发，安全）。
    match previous {
        Some(value) => unsafe { std::env::set_var("ZW_MEDIA_HOST_BIN", value) },
        None => unsafe { std::env::remove_var("ZW_MEDIA_HOST_BIN") },
    }
}

/// spawn 失败 → 永久错误面（IF-006：不重试、不内联回落）。
///
/// 指向不存在的二进制 → 首次调用失败；恢复合法路径后再次调用仍失败——证明失败
/// 被 latch（本 client 生命周期内不再尝试 spawn），而非每次重试。
#[test]
#[ignore = "需真实 zero-media-host 二进制（make 构建后显式运行）"]
#[serial_test::serial]
fn spawn_failure_latches_permanent_error_face() {
    let Some(bin) = built_media_host_bin() else {
        panic!("zero-media-host 未构建：先 cargo build -p zero-media-host 再运行本测试");
    };

    let previous = std::env::var_os("ZW_MEDIA_HOST_BIN");
    unsafe {
        // 指向必不存在的路径：spawn 失败 → 永久错误面。
        std::env::set_var(
            "ZW_MEDIA_HOST_BIN",
            bin.parent().expect("bin 必有 parent").join("zero-media-host-missing"),
        );
    }
    let mut client = MediaHostClient::new();
    let first = client
        .register_source(1, 8, "video/webm")
        .expect_err("坏路径 spawn 必失败");
    assert_eq!(first.code, MediaErrorCode::HostInternal);
    assert!(client.is_failed());

    // 恢复合法路径：仍失败 → 失败已 latch，不重试（IF-006）。
    unsafe {
        std::env::set_var("ZW_MEDIA_HOST_BIN", &bin);
    }
    let second = client
        .register_source(2, 8, "video/webm")
        .expect_err("永久错误面须短路");
    assert_eq!(second.code, MediaErrorCode::HostInternal);
    assert!(client.is_failed());

    match previous {
        Some(value) => unsafe { std::env::set_var("ZW_MEDIA_HOST_BIN", value) },
        None => unsafe { std::env::remove_var("ZW_MEDIA_HOST_BIN") },
    }
}
