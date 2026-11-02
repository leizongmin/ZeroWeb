//! 真实 `zero-media-host` 二进制 + 默认沙箱下的媒体消息族往返（goal
//! ③A-media-host 验收「功能不回退」；镜像 image-decoder 切片真实进程验收）。
//!
//! 无 env 覆盖直接 spawn 生产二进制（默认双沙箱自装 + fail-closed）：stdio
//! 收发 register → data → play 消息族，须产出 `MediaStateUpdate(Playing)`——
//! 沙箱安装不破坏解码内核与事件循环；安装失败会即时 exit(1) 使 recv 命中
//! EOF 而非假绿。子进程健康路径数毫秒内回包；若回归成卡死形态，由测试守卫
//! 墙钟兜底终止。

use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use zero_protocol::IpcChannel;
use zero_protocol::media::{MediaDataChunkParams, MediaPlayParams, MediaRegisterSourceParams};
use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;

fn fixture_bytes(name: &str) -> Vec<u8> {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // apps/
    p.pop(); // workspace root
    p.push("tests/fixtures/media");
    p.push(name);
    std::fs::read(p).expect("media fixture present")
}

#[test]
fn real_binary_serves_media_family_under_default_sandbox() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zero-media-host"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn production zero-media-host binary");
    let mut stdin = PipeTransport::new(io::empty(), child.stdin.take().expect("piped stdin"));
    let mut stdout = PipeTransport::new(child.stdout.take().expect("piped stdout"), io::sink());
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");

    let bytes = fixture_bytes("sample-webm-vp9.webm");
    let total = bytes.len() as u64;
    let script = [
        IpcMessage {
            id: 1,
            kind: IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                source_key: 7,
                total_len: total,
                mime: "video/webm".into(),
            }),
        },
        IpcMessage {
            id: 2,
            kind: IpcMessageKind::MediaDataChunk(MediaDataChunkParams {
                source_key: 7,
                seq: 0,
                data: bytes,
            }),
        },
        IpcMessage {
            id: 3,
            kind: IpcMessageKind::MediaPlay(MediaPlayParams {
                source_key: 7,
                position_ms: 0,
            }),
        },
    ];
    for message in &script {
        stdin.send(message.clone()).expect("send media message");
    }

    // 首条出站必为 MediaReady 握手（IF-001）；随后消费回包直至 Playing。
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut saw_ready = false;
    let mut saw_playing = false;
    for _ in 0..16 {
        assert!(Instant::now() < deadline, "子进程回包超时（沙箱误伤事件循环？）");
        let message = stdout.recv().expect("recv media reply——EOF 即 fail-closed 退出");
        match message.kind {
            IpcMessageKind::MediaReady(_) => saw_ready = true,
            IpcMessageKind::MediaStateUpdate(s) => {
                if matches!(s.state, zero_protocol::media::MediaPlaybackState::Playing { .. }) {
                    saw_playing = true;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(saw_ready, "未收到 MediaReady 握手");
    assert!(saw_playing, "未收到 MediaStateUpdate(Playing)——沙箱下解码/起播链断裂");

    // 作用域内显式关闭 stdin（EOF → 读线程终结 → 角色循环退出），再回收退出码。
    drop(stdin);
    drop(stdout);
    let status = child.wait().expect("wait media-host child");
    assert!(status.success(), "沙箱下子进程异常退出: {status}");
    // 真实进程安装日志（验收形态）：默认档 seccomp/landlock 安装成功可见。
    let mut stderr_text = String::new();
    stderr_pipe.read_to_string(&mut stderr_text).expect("read stderr");
    assert!(
        stderr_text.contains("seccomp 网络/exec 过滤已启用") && stderr_text.contains("landlock 已启用"),
        "缺沙箱安装成功日志，stderr={stderr_text:?}"
    );
    assert!(!stderr_text.contains("未沙箱运行"), "默认档不应出现未沙箱警告");
}

/// 操作员豁免面契约：`ZW_MEDIA_HOST_SANDBOX=0` 时进程照常服务，且 stderr 有
/// 显式「未沙箱运行」警告（降级可见，非静默裸奔）。
#[test]
fn sandbox_opt_out_serves_with_explicit_warning() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zero-media-host"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("ZW_MEDIA_HOST_SANDBOX", "0")
        .spawn()
        .expect("spawn zero-media-host with sandbox opt-out");
    let mut stdin = PipeTransport::new(io::empty(), child.stdin.take().expect("piped stdin"));
    let mut stdout = PipeTransport::new(child.stdout.take().expect("piped stdout"), io::sink());
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");

    stdin
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::MediaRegisterSource(MediaRegisterSourceParams {
                source_key: 1,
                total_len: 4,
                mime: "video/webm".into(),
            }),
        })
        .expect("send register message");
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut saw_ready = false;
    for _ in 0..4 {
        assert!(Instant::now() < deadline, "豁免面子进程回包超时");
        let message = stdout.recv().expect("recv media reply");
        if matches!(message.kind, IpcMessageKind::MediaReady(_)) {
            saw_ready = true;
            break;
        }
    }
    assert!(saw_ready, "ZW_MEDIA_HOST_SANDBOX=0 豁免面应照常服务");

    drop(stdin);
    drop(stdout);
    let status = child.wait().expect("wait media-host child");
    assert!(status.success(), "豁免面子进程异常退出: {status}");
    // 子进程已退出：stderr 管道读到 EOF，无阻塞面。
    let mut stderr_text = String::new();
    stderr_pipe.read_to_string(&mut stderr_text).expect("read stderr");
    assert!(
        stderr_text.contains("未沙箱运行"),
        "豁免面缺少显式未沙箱警告，stderr={stderr_text:?}"
    );
}
