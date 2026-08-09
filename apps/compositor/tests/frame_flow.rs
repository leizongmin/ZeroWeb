//! C2 集成测试：spawn zero-compositor 进程，验证帧提交 → 双缓冲 → 确认回执。

use std::process::{ChildStdin, ChildStdout, Command, Stdio};

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

fn make_frame(viewport_w: u32, viewport_h: u32, color: [u8; 4]) -> PaintSnapshotParams {
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
                r: color[0],
                g: color[1],
                b: color[2],
                a: color[3],
            },
        }],
        ..Default::default()
    }
}

fn spawn_compositor() -> (PipeTransport<ChildStdout, ChildStdin>, CompositorProcess) {
    let bin = env!("CARGO_BIN_EXE_zero-compositor");
    // lint 不追踪 RAII Drop（kill+wait）——见 CompositorProcess
    #[allow(clippy::zombie_processes)]
    let mut child = Command::new(bin)
        .args(child_process_args(ProcessRole::Compositor, 1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn compositor");
    let stdout = child.stdout.take().expect("stdout");
    let stdin = child.stdin.take().expect("stdin");
    (PipeTransport::new(stdout, stdin), CompositorProcess(child))
}

fn submit_frame(
    transport: &mut impl IpcChannel,
    request_id: u64,
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
    frame: PaintSnapshotParams,
) -> (u64, u64, u64) {
    transport
        .send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::CompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
                paint: Box::new(frame),
            },
        })
        .expect("send frame");

    let resp: IpcMessage = transport.recv().expect("recv result");
    assert_eq!(resp.id, request_id, "回执 id 应与请求一致");
    match resp.kind {
        IpcMessageKind::CompositorFrameResult {
            surface_id,
            navigation_epoch,
            frame_id,
        } => (surface_id, navigation_epoch, frame_id),
        other => panic!("意外消息: {other:?}"),
    }
}

struct FrameData {
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn get_frame(
    transport: &mut impl IpcChannel,
    request_id: u64,
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
) -> FrameData {
    transport
        .send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::GetCompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
            },
        })
        .expect("send get frame");
    let resp: IpcMessage = transport.recv().expect("recv frame data");
    assert_eq!(resp.id, request_id, "帧数据 id 应与请求一致");
    match resp.kind {
        IpcMessageKind::CompositorFrameData {
            surface_id,
            navigation_epoch,
            frame_id,
            width,
            height,
            rgba,
        } => FrameData {
            surface_id,
            navigation_epoch,
            frame_id,
            width,
            height,
            rgba,
        },
        other => panic!("意外消息: {other:?}"),
    }
}

#[test]
fn compositor_accepts_frames_and_confirms() {
    let (mut transport, _comp) = spawn_compositor();

    // 发两帧，验证双缓冲连续确认
    for i in 1..=2u64 {
        let frame = make_frame(640, 480, [255, 0, 0, 255]);
        assert_eq!(submit_frame(&mut transport, i, 7, 3, i, frame), (7, 3, i));
    }

    // 拉取最新帧：纯红 fill 帧 → front 像素应为红色
    let frame = get_frame(&mut transport, 100, 7, 3, 2);
    assert_eq!(frame.surface_id, 7);
    assert_eq!(frame.navigation_epoch, 3);
    assert_eq!(frame.frame_id, 2, "应是最新帧（第 2 帧）");
    assert_eq!((frame.width, frame.height), (640, 480));
    assert_eq!(frame.rgba.len(), (640 * 480 * 4) as usize, "帧像素完整");
    assert_eq!(&frame.rgba[..4], &[255, 0, 0, 255], "首像素应为纯红");
    let last = &frame.rgba[frame.rgba.len() - 4..];
    assert_eq!(last, &[255, 0, 0, 255], "末像素应为纯红");
}

#[test]
fn compositor_isolates_surfaces_rejects_old_frames_resizes_and_releases() {
    let (mut transport, _comp) = spawn_compositor();

    assert_eq!(
        submit_frame(&mut transport, 1, 11, 2, 10, make_frame(2, 2, [255, 0, 0, 255])),
        (11, 2, 10)
    );
    assert_eq!(
        submit_frame(&mut transport, 2, 22, 4, 7, make_frame(3, 1, [0, 255, 0, 255])),
        (22, 4, 7)
    );

    // 新导航允许 frame id 从较小值重新开始，并独立调整该 surface 尺寸。
    assert_eq!(
        submit_frame(&mut transport, 3, 11, 3, 1, make_frame(4, 2, [0, 0, 255, 255])),
        (11, 3, 1)
    );

    // 旧 epoch 和同 epoch 倒序帧均返回当前 front 标识，不覆盖蓝色帧。
    assert_eq!(
        submit_frame(&mut transport, 4, 11, 2, 99, make_frame(1, 1, [255, 255, 0, 255])),
        (11, 3, 1)
    );
    assert_eq!(
        submit_frame(&mut transport, 5, 11, 3, 0, make_frame(1, 1, [255, 255, 0, 255])),
        (11, 3, 1)
    );

    let first = get_frame(&mut transport, 6, 11, 3, 1);
    assert_eq!((first.width, first.height), (4, 2));
    assert_eq!((first.navigation_epoch, first.frame_id), (3, 1));
    assert_eq!(&first.rgba[..4], &[0, 0, 255, 255]);

    let second = get_frame(&mut transport, 7, 22, 4, 7);
    assert_eq!((second.width, second.height), (3, 1));
    assert_eq!((second.navigation_epoch, second.frame_id), (4, 7));
    assert_eq!(&second.rgba[..4], &[0, 255, 0, 255]);

    transport
        .send(IpcMessage {
            id: 8,
            kind: IpcMessageKind::ReleaseCompositorSurface { surface_id: 11 },
        })
        .expect("release surface");
    let release: IpcMessage = transport.recv().expect("recv release result");
    assert_eq!(release.id, 8);
    assert!(matches!(release.kind, IpcMessageKind::Ok));

    let released = get_frame(&mut transport, 9, 11, 3, 1);
    assert_eq!((released.navigation_epoch, released.frame_id), (0, 0));
    assert_eq!((released.width, released.height), (0, 0));
    assert!(released.rgba.is_empty());

    let remaining = get_frame(&mut transport, 10, 22, 4, 7);
    assert_eq!((remaining.width, remaining.height), (3, 1));
    assert_eq!(&remaining.rgba[..4], &[0, 255, 0, 255]);
}
