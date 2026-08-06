//! D1 集成测试：spawn zero-image-decoder 进程，经 stdin/stdout 管道解码 PNG，
//! 并与进程内解码（zero-render-foundation）结果逐像素对比。

use std::process::{Command, Stdio};

use zero_protocol::message::{ImageDecodeParams, ImageDecodeResultParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

/// 构造 2x2 测试 PNG（四色：红/绿/蓝/白）。
fn make_test_png() -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, 2, 2);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("png header");
        let data = [
            255u8, 0, 0, 255, // 红
            0, 255, 0, 255, // 绿
            0, 0, 255, 255, // 蓝
            255, 255, 255, 255, // 白
        ];
        writer.write_image_data(&data).expect("png data");
    }
    buf
}

#[test]
fn ipc_decode_matches_inline_decode() {
    let png_bytes = make_test_png();

    // 进程内解码（参考结果）
    let inline = zero_render_foundation::image_cache::decode_image_bytes(&png_bytes).expect("inline decode");
    assert_eq!((inline.width, inline.height), (2, 2));

    // spawn image-decoder 子进程
    let bin = env!("CARGO_BIN_EXE_zero-image-decoder");
    let mut child = Command::new(bin)
        .args(child_process_args(ProcessRole::ImageDecoder, 42))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn image-decoder");
    let stdout = child.stdout.take().expect("child stdout");
    let stdin = child.stdin.take().expect("child stdin");
    let mut transport = PipeTransport::new(stdout, stdin);

    // 发解码请求
    let request_id = 7u64;
    let msg = IpcMessage {
        id: request_id,
        kind: IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
            request_id,
            mime: "image/png".to_string(),
            bytes: png_bytes,
        }),
    };
    transport.send(msg).expect("send request");

    // 收响应并断言
    let resp: IpcMessage = transport.recv().expect("recv response");
    match resp.kind {
        IpcMessageKind::ImageDecodeResult(ImageDecodeResultParams {
            request_id: rid,
            width,
            height,
            rgba,
            error,
        }) => {
            assert_eq!(rid, request_id, "request id 回带一致");
            assert!(error.is_none(), "解码不应失败: {error:?}");
            assert_eq!((width, height), (2, 2));
            assert_eq!(rgba, inline.pixels, "IPC 解码像素与进程内解码逐像素一致");
        }
        other => panic!("意外消息类型: {other:?}"),
    }

    // 关闭 stdin → 进程应优雅退出
    drop(transport);
    let status = child.wait().expect("wait child");
    assert!(status.success(), "image-decoder 退出码应为 0: {status}");
}

#[test]
fn ipc_decode_reports_error_for_garbage_input() {
    let garbage = b"this is not an image".to_vec();
    let bin = env!("CARGO_BIN_EXE_zero-image-decoder");
    let mut child = Command::new(bin)
        .args(child_process_args(ProcessRole::ImageDecoder, 43))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn image-decoder");
    let stdout = child.stdout.take().expect("child stdout");
    let stdin = child.stdin.take().expect("child stdin");
    let mut transport = PipeTransport::new(stdout, stdin);

    let request_id = 8u64;
    transport
        .send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
                request_id,
                mime: "image/png".to_string(),
                bytes: garbage,
            }),
        })
        .expect("send request");

    let resp: IpcMessage = transport.recv().expect("recv response");
    match resp.kind {
        IpcMessageKind::ImageDecodeResult(ImageDecodeResultParams { error, .. }) => {
            assert!(error.is_some(), "畸形输入应返回错误");
        }
        other => panic!("意外消息类型: {other:?}"),
    }

    drop(transport);
    let _ = child.wait();
}
