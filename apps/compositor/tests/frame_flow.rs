//! C2 集成测试：spawn zero-compositor 进程，验证帧提交 → 双缓冲 → 确认回执。

use std::process::{Command, Stdio};

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::paint_snapshot::{IpcColor, IpcFill, IpcRect, PaintSnapshotParams};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

/// RAII 包装：所有退出路径（含断言 panic）都 kill + wait 子进程。
struct CompositorProcess(std::process::Child);

impl Drop for CompositorProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn make_frame(viewport_w: u32, viewport_h: u32) -> PaintSnapshotParams {
    PaintSnapshotParams {
        viewport_width: viewport_w,
        viewport_height: viewport_h,
        document_height: viewport_h as f32,
        fills: vec![IpcFill {
            rect: IpcRect {
                x: 0.0,
                y: 0.0,
                width: viewport_w as f32,
                height: viewport_h as f32,
            },
            color: IpcColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }],
        ..Default::default()
    }
}

#[test]
fn compositor_accepts_frames_and_confirms() {
    // spawn compositor
    let bin = env!("CARGO_BIN_EXE_zero-compositor");
    // lint 不追踪 RAII Drop（kill+wait）——见 CompositorProcess
    #[allow(clippy::zombie_processes)]
    let mut child = Command::new(bin)
        .args(child_process_args(ProcessRole::ImageDecoder, 1)) // 角色名仅作参数占位
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn compositor");
    let stdout = child.stdout.take().expect("stdout");
    let stdin = child.stdin.take().expect("stdin");
    let mut transport = PipeTransport::new(stdout, stdin);
    let _comp = CompositorProcess(child);

    // 发两帧，验证双缓冲连续确认
    for i in 1..=2u64 {
        let frame = make_frame(640, 480);
        transport
            .send(IpcMessage {
                id: i,
                kind: IpcMessageKind::CompositorFrame(Box::new(frame)),
            })
            .expect("send frame");

        let resp: IpcMessage = transport.recv().expect("recv result");
        match resp.kind {
            IpcMessageKind::CompositorFrameResult { frame_id } => {
                assert_eq!(frame_id, i, "帧序号应递增确认");
                assert_eq!(resp.id, i, "回执 id 应与请求一致");
            }
            other => panic!("意外消息: {other:?}"),
        }
    }

    // 拉取最新帧：纯红 fill 帧 → front 像素应为红色
    transport
        .send(IpcMessage {
            id: 100,
            kind: IpcMessageKind::GetCompositorFrame,
        })
        .expect("send get frame");
    let resp: IpcMessage = transport.recv().expect("recv frame data");
    match resp.kind {
        IpcMessageKind::CompositorFrameData {
            frame_id,
            width,
            height,
            rgba,
        } => {
            assert_eq!(frame_id, 2, "应是最新帧（第 2 帧）");
            assert_eq!((width, height), (640, 480));
            assert_eq!(rgba.len(), (640 * 480 * 4) as usize, "帧像素完整");
            // 纯红 fill 覆盖全视口 → 所有像素应为 (255,0,0,255)
            assert_eq!(&rgba[..4], &[255, 0, 0, 255], "首像素应为纯红");
            let last = &rgba[rgba.len() - 4..];
            assert_eq!(last, &[255, 0, 0, 255], "末像素应为纯红");
        }
        other => panic!("意外消息: {other:?}"),
    }
}
