//! 真实 `zero-image-decoder` 二进制在环的 stdio 解码往返（goal ③A-image-decoder 验收）。
//!
//! 子进程 main 自装沙箱（默认开启：seccomp 网络/exec 黑名单 + landlock 空
//! allowed-set），本测试证明默认沙箱不破坏 stdio IPC 与解码主路径——隔离语义与
//! 生产 renderer→decoder 链路同构（`child_process_args(ProcessRole::ImageDecoder)` +
//! `PipeTransport`），非 mock 通道。

use std::process::{Command, Stdio};

use zero_protocol::message::{ImageDecodeParams, ImageDecodeResultParams, IpcMessage, IpcMessageKind};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

/// 1x1 红色 PNG（RGBA，alpha=127；与 crates/webview/src/image_decoder.rs
/// `ONE_PX_RED_PNG_B64` 同源，此处展开为字节常量避免测试目标引 b64 依赖）。
const ONE_PX_RED_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196,
    137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 218, 99, 252, 207, 192, 80, 15, 0, 4, 133, 1, 128, 132, 169, 140, 33, 0, 0,
    0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[test]
fn real_decoder_binary_decodes_with_default_sandbox() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zero-image-decoder"));
    for arg in child_process_args(ProcessRole::ImageDecoder, 0) {
        cmd.arg(arg);
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn zero-image-decoder");
    let stdout = child.stdout.take().expect("child stdout piped");
    let stdin = child.stdin.take().expect("child stdin piped");
    let response: IpcMessage = {
        // 作用域限定 transport：recv 后 drop 即关闭子进程 stdin——run_role
        // 以 EOF 为退出信号（is_disconnected_channel_message），否则子进程
        // 阻塞在下一轮 recv，wait() 永不返回。
        let mut transport = PipeTransport::new(stdout, stdin);

        transport
            .send(IpcMessage {
                id: 1,
                kind: IpcMessageKind::ImageDecodeRequest(ImageDecodeParams {
                    request_id: 11,
                    mime: "image/png".to_string(),
                    bytes: ONE_PX_RED_PNG.to_vec(),
                }),
            })
            .expect("send decode request");

        transport.recv().expect("recv decode result")
    };
    let _ = child.wait();

    match response.kind {
        IpcMessageKind::ImageDecodeResult(ImageDecodeResultParams {
            request_id,
            width,
            height,
            rgba,
            error,
        }) => {
            assert_eq!(request_id, 11);
            assert_eq!(width, 1);
            assert_eq!(height, 1);
            assert_eq!(rgba, vec![255, 0, 0, 127], "1x1 红色像素（直通 alpha）");
            assert!(error.is_none(), "沙箱在环时解码不应报错: {error:?}");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}
