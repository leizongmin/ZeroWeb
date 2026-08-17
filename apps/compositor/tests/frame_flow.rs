//! C2 集成测试：spawn zero-compositor 进程，验证帧提交 → 双缓冲 → 确认回执。

use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::sync::Mutex;

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::paint_snapshot::{IpcColor, IpcFill, IpcImage, IpcImagePayload, IpcRect, PaintSnapshotParams};
use zero_protocol::transport::PipeTransport;
use zero_protocol::{IpcChannel, ProcessRole, child_process_args};

static COMPOSITOR_TEST_ENV: Mutex<()> = Mutex::new(());

/// RAII 包装：所有退出路径（含断言 panic）都 kill + wait 子进程；并持有 env 锁避免并行竞态。
struct CompositorProcess {
    child: std::process::Child,
    _env_lock: std::sync::MutexGuard<'static, ()>,
}

impl Drop for CompositorProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn make_frame(viewport_w: u32, viewport_h: u32, color: [u8; 4]) -> PaintSnapshotParams {
    make_frame_with_dirty(viewport_w, viewport_h, color, Vec::new())
}

fn make_frame_with_dirty(
    viewport_w: u32,
    viewport_h: u32,
    color: [u8; 4],
    dirty_rects: Vec<IpcRect>,
) -> PaintSnapshotParams {
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
        dirty_rects,
        ..Default::default()
    }
}

fn spawn_compositor() -> (PipeTransport<ChildStdout, ChildStdin>, CompositorProcess) {
    // 多数 IPC 用例经 get_frame CPU 读回；关闭 fd 导出避免与 inline/mailbox 路径竞态。
    spawn_compositor_with_env(&[("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0")])
}

/// 全默认 GPU 链路（含 dma-buf fd 导出）。
#[cfg(target_os = "linux")]
fn spawn_compositor_gpu_dmabuf() -> (PipeTransport<ChildStdout, ChildStdin>, CompositorProcess) {
    // `ZW_BROWSER_GPU_DMABUF_IMPORT` 经 `spawn_compositor_with_env` 在持锁内设为 "1"——
    // 该 env 同时驱动**测试进程**内 `resolve_compositor_frame_delivery_fenced` 的 Dmabuf
    // vs Rgba 分支（frame_shm.rs `browser_gpu_dmabuf_import_enabled()`），故须在
    // `COMPOSITOR_TEST_ENV` 锁内改进程 env，避免与本 crate 其它并行测 / frame_shm 测竞态
    // （曾因此 PoisonError 级联致 8 测级联失败——见 R3275 调查）。
    spawn_compositor_with_env(&[
        ("ZW_COMPOSITOR_GPU", "1"),
        ("ZW_COMPOSITOR_GPU_IMAGE", "1"),
        ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "1"),
        ("ZW_BROWSER_GPU_DMABUF_IMPORT", "1"),
        ("ZW_COMPOSITOR_SCROLL_TRANSFORM", "0"),
    ])
}

fn spawn_compositor_with_env(
    extra_env: &[(&str, &str)],
) -> (PipeTransport<ChildStdout, ChildStdin>, CompositorProcess) {
    let env_lock = COMPOSITOR_TEST_ENV.lock().unwrap_or_else(|error| error.into_inner());

    let mut env: Vec<(&str, &str)> = vec![
        ("ZW_COMPOSITOR_GPU", "0"),
        ("ZW_COMPOSITOR_GPU_IMAGE", "0"),
        ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0"),
        // get_frame 默认走 CPU RGBA 读回——测试进程（Browser 侧）禁用 dma-buf 导入解析，
        // 除非 gpu_dmabuf 专用 helper 经 extra_env 显式覆盖为 "1"。
        ("ZW_BROWSER_GPU_DMABUF_IMPORT", "0"),
    ];
    for (key, value) in extra_env {
        if let Some(slot) = env.iter_mut().find(|(k, _)| *k == *key) {
            *slot = (key, value);
        } else {
            env.push((key, value));
        }
    }

    // `ZW_BROWSER_GPU_DMABUF_IMPORT` 同时驱动**测试进程**（非 compositor 子进程）内
    // `resolve_compositor_frame_delivery_fenced` 的 Dmabuf vs Rgba 分支——它在锁内
    // 同步改进程 env，使子进程 spawn 与后续 get_frame_delivery 解析读到一致值
    // （避免与本 crate 其它并行测 / frame_shm.rs 的 serial 测竞态级联 PoisonError）。
    let import_value = env
        .iter()
        .find(|(k, _)| *k == "ZW_BROWSER_GPU_DMABUF_IMPORT")
        .map(|(_, v)| *v)
        .unwrap_or("0");
    // SAFETY: 测试进程 env，持 `COMPOSITOR_TEST_ENV` 锁期间无竞态。
    unsafe {
        std::env::set_var("ZW_BROWSER_GPU_DMABUF_IMPORT", import_value);
    }

    let bin = env!("CARGO_BIN_EXE_zero-compositor");
    #[allow(clippy::zombie_processes)]
    let mut cmd = Command::new(bin);
    cmd.args(child_process_args(ProcessRole::Compositor, 1))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for (k, v) in &env {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().expect("spawn compositor");
    let stdout = child.stdout.take().expect("stdout");
    let stdin = child.stdin.take().expect("stdin");
    (
        PipeTransport::new(stdout, stdin),
        CompositorProcess {
            child,
            _env_lock: env_lock,
        },
    )
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
    scroll_x: f32,
    scroll_y: f32,
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
            shm_name,
            scroll_x,
            scroll_y,
            gpu_image,
            ..
        } => {
            let rgba = zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, shm_name, gpu_image)
                .expect("resolve compositor frame rgba");
            FrameData {
                surface_id,
                navigation_epoch,
                frame_id,
                width,
                height,
                rgba,
                scroll_x,
                scroll_y,
            }
        }
        other => panic!("意外消息: {other:?}"),
    }
}

/// Browser dma-buf 导入路径：经 `resolve_compositor_frame_delivery_fenced` 解析。
#[cfg(target_os = "linux")]
fn get_frame_delivery(
    transport: &mut impl IpcChannel,
    request_id: u64,
    surface_id: u64,
    navigation_epoch: u64,
    frame_id: u64,
) -> (u32, u32, zero_protocol::CompositorResolvedFrame) {
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
            width,
            height,
            rgba,
            shm_name,
            gpu_image,
            ..
        } => {
            let resolved = zero_protocol::resolve_compositor_frame_delivery_fenced(
                width,
                height,
                rgba,
                shm_name,
                gpu_image,
                Some(frame_id),
            )
            .expect("resolve delivery");
            (width, height, resolved)
        }
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
fn compositor_retains_image_payload_across_frames() {
    assert_compositor_retains_image_payload(&[]);
}

#[test]
fn compositor_gpu_retains_image_payload_across_frames() {
    assert_compositor_retains_image_payload(&[("ZW_COMPOSITOR_GPU", "1"), ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0")]);
}

fn assert_compositor_retains_image_payload(extra_env: &[(&str, &str)]) {
    let (mut transport, _comp) = spawn_compositor_with_env(extra_env);
    let image = IpcImage {
        rect: IpcRect {
            x: 0.0,
            y: 0.0,
            width: 8.0,
            height: 8.0,
        },
        image_key: 44,
        clip: None,
    };
    let first = PaintSnapshotParams {
        viewport_width: 8,
        viewport_height: 8,
        document_height: 8.0,
        images: vec![image.clone()],
        image_payloads: vec![IpcImagePayload {
            image_key: 44,
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        }],
        ..Default::default()
    };
    submit_frame(&mut transport, 1, 88, 1, 1, first);
    let first_pixels = get_frame(&mut transport, 2, 88, 1, 1);
    assert_eq!(&first_pixels.rgba[..4], &[255, 0, 0, 255]);

    let second = PaintSnapshotParams {
        viewport_width: 8,
        viewport_height: 8,
        document_height: 8.0,
        images: vec![image],
        ..Default::default()
    };
    submit_frame(&mut transport, 3, 88, 1, 2, second);
    let second_pixels = get_frame(&mut transport, 4, 88, 1, 2);
    assert_eq!(&second_pixels.rgba[..4], &[255, 0, 0, 255]);
}

/// S3：部分 dirty 帧经 compositor 进程 copy_front + 区域重绘，区外像素保留。
#[test]
fn compositor_partial_dirty_preserves_pixels_outside_region() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[("ZW_COMPOSITOR_SCROLL_TRANSFORM", "0")]);
    let w = 100u32;
    let h = 80u32;

    assert_eq!(
        submit_frame(&mut transport, 1, 5, 1, 1, make_frame(w, h, [0, 0, 255, 255]),),
        (5, 1, 1)
    );

    assert_eq!(
        submit_frame(
            &mut transport,
            2,
            5,
            1,
            2,
            make_frame_with_dirty(
                w,
                h,
                [255, 0, 0, 255],
                vec![IpcRect {
                    x: 0.0,
                    y: 0.0,
                    width: 40.0,
                    height: 40.0,
                }],
            ),
        ),
        (5, 1, 2)
    );

    let frame = get_frame(&mut transport, 3, 5, 1, 2);
    assert_eq!(&frame.rgba[..4], &[255, 0, 0, 255], "dirty 内应为红");
    let outside = ((40 * w + 60) * 4) as usize;
    assert_eq!(frame.rgba[outside], 0, "dirty 外 R 应保留蓝");
    assert_eq!(frame.rgba[outside + 2], 255, "dirty 外 B 应保留蓝");
}

/// C3：`ZW_COMPOSITOR_GPU=1` 时 compositor 进程内 GPU 光栅化（不可用时与 CPU 同路径回退）。
#[test]
fn compositor_gpu_path_produces_expected_fill() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_GPU", "1"), ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0")]);
    assert_eq!(
        submit_frame(&mut transport, 1, 9, 1, 1, make_frame(64, 64, [255, 0, 0, 255]),),
        (9, 1, 1)
    );
    let frame = get_frame(&mut transport, 2, 9, 1, 1);
    assert_eq!(&frame.rgba[..4], &[255, 0, 0, 255]);
}

/// P2-8（真实合成器进程）：GPU 光栅模式下半透明 fill 由 GPU 直接渲染
/// （顶点 alpha 通道，shader 输出 color.a × 覆盖率）——像素为半透明混合结果
/// （128-alpha 红 over 黑底：0.5×红 → sRGB 编码 R<255，而非不透明 255）。
#[test]
fn compositor_gpu_semitransparent_renders_correctly() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_GPU", "1"), ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0")]);
    assert_eq!(
        submit_frame(&mut transport, 1, 9, 2, 1, make_frame(64, 64, [128, 0, 0, 128]),),
        (9, 2, 1)
    );
    let frame = get_frame(&mut transport, 2, 9, 2, 1);
    let center = ((32 * 64 + 32) * 4) as usize;
    assert!(
        (128..255).contains(&frame.rgba[center]),
        "半透明红应被 GPU 半混合渲染（R 在 128..255，非不透明 255），got {}",
        frame.rgba[center]
    );
}

/// P0-1（真实合成器进程）：GPU 光栅模式下带模糊阴影被拒绝 → 回退 CPU →
/// 灰 blur 阴影 over 白底（cpu::render_full_scene 的 framebuffer 为白底）：
/// 中心应为灰（≈128），rect 边缘外有模糊渐变、颜色回归白底（> 中心）。
#[test]
fn compositor_gpu_blur_shadow_falls_back_to_cpu() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_GPU", "1"), ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0")]);
    // 无背景 fill + 灰 blur=3 阴影（rect 8..24 × 8..24）
    let mut snapshot = PaintSnapshotParams {
        viewport_width: 64,
        viewport_height: 64,
        document_height: 64.0,
        ..Default::default()
    };
    snapshot.shadows = vec![zero_protocol::paint_snapshot::IpcShadow {
        rect: IpcRect {
            x: 8.0,
            y: 8.0,
            width: 16.0,
            height: 16.0,
        },
        color: IpcColor {
            r: 128,
            g: 128,
            b: 128,
            a: 255,
        },
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 3.0,
        spread_radius: 0.0,
    }];
    assert_eq!(submit_frame(&mut transport, 1, 9, 3, 1, snapshot), (9, 3, 1));
    let frame = get_frame(&mut transport, 2, 9, 3, 1);
    // 阴影中心 (16,16)：CPU blur 后灰（≈128 over 白底），完全没画则为白 255
    let center = ((16 * 64 + 16) * 4) as usize;
    assert!(
        (frame.rgba[center] as i32 - 128).abs() <= 15,
        "阴影中心应画灰（≈128），got {}",
        frame.rgba[center]
    );
    // rect 边缘外 2px (26,16)：blur 渐变衰减 → alpha 低 → 颜色回归白底（> 中心）
    let edge_out = ((16 * 64 + 26) * 4) as usize;
    assert!(
        frame.rgba[edge_out] > frame.rgba[center],
        "阴影边缘外应渐变回归白底（模糊衰减），center={} edge={}",
        frame.rgba[center],
        frame.rgba[edge_out]
    );
}

/// 4.3 S1：Linux 默认经 POSIX shm 传输帧像素（`ZW_COMPOSITOR_SHM=0` 回退内联；非 Linux 跳过）。
#[test]
#[cfg(target_os = "linux")]
fn compositor_shm_path_produces_expected_fill() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[("ZW_COMPOSITOR_SHM", "1")]);
    assert_eq!(
        submit_frame(&mut transport, 1, 10, 1, 1, make_frame(32, 32, [0, 128, 255, 255]),),
        (10, 1, 1)
    );
    let frame = get_frame(&mut transport, 2, 10, 1, 1);
    assert_eq!((frame.width, frame.height), (32, 32));
    assert_eq!(&frame.rgba[..4], &[0, 128, 255, 255]);
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

/// RFC 4.2：CompositorSetScroll 更新 surface 元数据并在 GetCompositorFrame 回读。
#[test]
fn compositor_scroll_metadata_round_trips() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[("ZW_COMPOSITOR_SCROLL_TRANSFORM", "0")]);
    let frame = make_frame(32, 24, [128, 64, 32, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 9, 1, 1, frame), (9, 1, 1));

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorSetScroll {
                surface_id: 9,
                scroll_x: 12.5,
                scroll_y: 48.0,
            },
        })
        .expect("set scroll");
    let ack: IpcMessage = transport.recv().expect("scroll ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    let frame = get_frame(&mut transport, 3, 9, 1, 1);
    assert!((frame.scroll_x - 12.5).abs() < f32::EPSILON);
    assert!((frame.scroll_y - 48.0).abs() < f32::EPSILON);
}

/// RFC 4.2-S2：scroll 变换以文档图元重光栅化，不能平移首屏位图。
#[test]
fn compositor_scroll_transform_rasterizes_content_beyond_initial_viewport() {
    // 默认路径必须保留 compositor 侧视口重绘；否则 Browser 会在滚动时退回
    // 本地图元/标题 URL 占位路径。
    let (mut transport, _comp) = spawn_compositor_with_env(&[]);
    let frame = PaintSnapshotParams {
        viewport_width: 2,
        viewport_height: 4,
        document_height: 8.0,
        fills: vec![
            IpcFill {
                rect: IpcRect {
                    x: 0.0,
                    y: 0.0,
                    width: 2.0,
                    height: 4.0,
                },
                color: IpcColor {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            },
            IpcFill {
                rect: IpcRect {
                    x: 0.0,
                    y: 4.0,
                    width: 2.0,
                    height: 4.0,
                },
                color: IpcColor {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            },
        ],
        ..Default::default()
    };
    assert_eq!(submit_frame(&mut transport, 1, 3, 1, 1, frame), (3, 1, 1));

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorSetScroll {
                surface_id: 3,
                scroll_x: 0.0,
                scroll_y: 4.0,
            },
        })
        .expect("set scroll");
    let _: IpcMessage = transport.recv().expect("scroll ack");

    let transformed = get_frame(&mut transport, 3, 3, 1, 1);
    assert!((transformed.scroll_x).abs() < f32::EPSILON);
    assert!((transformed.scroll_y - 4.0).abs() < f32::EPSILON);
    // scroll_y=4：首屏位图完全越界；仍须重光栅化出文档第 4~7 行的蓝色内容。
    assert_eq!(&transformed.rgba[..4], &[0, 0, 255, 255]);
    assert_eq!(&transformed.rgba[28..32], &[0, 0, 255, 255]);
}

/// RFC 4.3-S2：gpu_image mailbox 经 shm 后端传递像素。
#[cfg(target_os = "linux")]
#[test]
fn compositor_gpu_image_mailbox_round_trips() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_GPU_IMAGE", "1"), ("ZW_COMPOSITOR_GPU", "0")]);
    let frame = make_frame(2, 2, [42, 84, 126, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 5, 1, 1, frame), (5, 1, 1));

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::GetCompositorFrame {
                surface_id: 5,
                navigation_epoch: 1,
                frame_id: 1,
            },
        })
        .expect("get frame");
    let resp: IpcMessage = transport.recv().expect("frame data");
    match resp.kind {
        IpcMessageKind::CompositorFrameData {
            width,
            height,
            rgba,
            shm_name,
            gpu_image,
            ..
        } => {
            assert!(rgba.is_empty());
            assert!(shm_name.is_none());
            let desc = gpu_image.expect("gpu_image descriptor");
            assert_eq!(desc.width, width);
            assert_eq!(desc.height, height);
            let resolved =
                zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, None, Some(desc)).expect("resolve");
            assert_eq!(resolved.len(), (2 * 2 * 4) as usize);
            assert!(resolved.iter().all(|&b| b == 42 || b == 84 || b == 126 || b == 255));
        }
        other => panic!("unexpected {other:?}"),
    }
}

fn get_ui_frame(transport: &mut impl IpcChannel, request_id: u64, surface_id: u64) -> FrameData {
    transport
        .send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::GetCompositorUiFrame { surface_id },
        })
        .expect("get ui frame");
    let resp: IpcMessage = transport.recv().expect("ui frame data");
    match resp.kind {
        IpcMessageKind::CompositorFrameData {
            surface_id: sid,
            navigation_epoch,
            frame_id,
            width,
            height,
            rgba,
            shm_name,
            scroll_x,
            scroll_y,
            gpu_image,
            ..
        } => {
            assert_eq!(sid, surface_id);
            assert_eq!(navigation_epoch, 0);
            assert_eq!(frame_id, 0);
            assert!((scroll_x).abs() < f32::EPSILON);
            assert!((scroll_y).abs() < f32::EPSILON);
            let rgba = zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, shm_name, gpu_image)
                .expect("resolve ui frame rgba");
            FrameData {
                surface_id: sid,
                navigation_epoch,
                frame_id,
                width,
                height,
                rgba,
                scroll_x: 0.0,
                scroll_y: 0.0,
            }
        }
        other => panic!("unexpected {other:?}"),
    }
}

/// RFC 4.4-S2：UI 位图提交与回读。
#[test]
fn compositor_ui_frame_round_trips() {
    let (mut transport, _comp) = spawn_compositor();
    let ui_surface = u64::MAX;

    transport
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::CompositorRegisterUiSurface(zero_protocol::CompositorUiSurfaceInfo {
                surface_id: ui_surface,
                width: 2,
                height: 2,
            }),
        })
        .expect("register ui");
    let ack: IpcMessage = transport.recv().expect("register ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorUiFrame {
                surface_id: ui_surface,
                width: 2,
                height: 2,
                rgba: [255u8, 0, 0, 255].repeat(4),
                shm_name: None,
            },
        })
        .expect("ui frame");
    let ack: IpcMessage = transport.recv().expect("ui frame ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    let ui = get_ui_frame(&mut transport, 3, ui_surface);
    assert_eq!((ui.width, ui.height), (2, 2));
    assert_eq!(&ui.rgba[..4], &[255, 0, 0, 255]);
}

fn get_present_frame(
    transport: &mut impl IpcChannel,
    request_id: u64,
    width: u32,
    height: u32,
    page_surface_id: u64,
    ui_surface_id: u64,
) -> FrameData {
    transport
        .send(IpcMessage {
            id: request_id,
            kind: IpcMessageKind::GetCompositorPresentFrame {
                width,
                height,
                page_surface_id,
                ui_surface_id,
            },
        })
        .expect("get present frame");
    let resp: IpcMessage = transport.recv().expect("present frame data");
    match resp.kind {
        IpcMessageKind::CompositorFrameData {
            surface_id,
            navigation_epoch,
            frame_id,
            width,
            height,
            rgba,
            shm_name,
            scroll_x,
            scroll_y,
            gpu_image,
            ..
        } => {
            assert_eq!(surface_id, page_surface_id);
            assert_eq!(navigation_epoch, 0);
            assert_eq!(frame_id, 0);
            assert!((scroll_x).abs() < f32::EPSILON);
            assert!((scroll_y).abs() < f32::EPSILON);
            let rgba = zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, shm_name, gpu_image)
                .expect("resolve present rgba");
            FrameData {
                surface_id,
                navigation_epoch,
                frame_id,
                width,
                height,
                rgba,
                scroll_x: 0.0,
                scroll_y: 0.0,
            }
        }
        other => panic!("unexpected {other:?}"),
    }
}

/// RFC 4.4-S3：compositor 合成 page + UI present 帧。
#[test]
fn compositor_present_composites_page_and_ui() {
    let (mut transport, _comp) = spawn_compositor();
    let page_surface = 42u64;
    let ui_surface = u64::MAX;
    let w = 2u32;
    let h = 2u32;

    assert_eq!(
        submit_frame(
            &mut transport,
            1,
            page_surface,
            1,
            1,
            make_frame(w, h, [0, 0, 255, 255])
        ),
        (page_surface, 1, 1)
    );

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorRegisterUiSurface(zero_protocol::CompositorUiSurfaceInfo {
                surface_id: ui_surface,
                width: w,
                height: h,
            }),
        })
        .expect("register ui");
    let ack: IpcMessage = transport.recv().expect("register ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    transport
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::CompositorUiFrame {
                surface_id: ui_surface,
                width: w,
                height: h,
                rgba: [255u8, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
                shm_name: None,
            },
        })
        .expect("ui frame");
    let ack: IpcMessage = transport.recv().expect("ui frame ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    let present = get_present_frame(&mut transport, 4, w, h, page_surface, ui_surface);
    assert_eq!((present.width, present.height), (w, h));
    // 左上：蓝 page + 50% 红 UI → (128, 0, 127)
    assert_eq!(&present.rgba[..4], &[128, 0, 127, 255]);
    // 右上：纯蓝 page
    assert_eq!(&present.rgba[4..8], &[0, 0, 255, 255]);
}

/// 对照组：`ZW_COMPOSITOR_SCROLL_TRANSFORM=0` 时 present 帧不烘焙，页面像素保持原样。
#[test]
fn compositor_present_scroll_transform_disabled_keeps_pixels() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[("ZW_COMPOSITOR_SCROLL_TRANSFORM", "0")]);
    let page_surface = 44u64;
    let ui_surface = u64::MAX;
    let w = 2u32;
    let h = 2u32;

    let frame = PaintSnapshotParams {
        viewport_width: w,
        viewport_height: h,
        document_height: h as f32,
        fills: vec![IpcFill {
            rect: IpcRect {
                x: 0.0,
                y: 0.0,
                width: w as f32,
                height: h as f32,
            },
            color: IpcColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }],
        ..Default::default()
    };
    assert_eq!(
        submit_frame(&mut transport, 1, page_surface, 1, 1, frame),
        (page_surface, 1, 1)
    );

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorRegisterUiSurface(zero_protocol::CompositorUiSurfaceInfo {
                surface_id: ui_surface,
                width: w,
                height: h,
            }),
        })
        .expect("register ui");
    let ack: IpcMessage = transport.recv().expect("register ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    transport
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::CompositorUiFrame {
                surface_id: ui_surface,
                width: w,
                height: h,
                rgba: vec![0u8; (w * h * 4) as usize],
                shm_name: None,
            },
        })
        .expect("ui frame");
    let ack: IpcMessage = transport.recv().expect("ui frame ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    transport
        .send(IpcMessage {
            id: 4,
            kind: IpcMessageKind::CompositorSetScroll {
                surface_id: page_surface,
                scroll_x: 0.0,
                scroll_y: 1.0,
            },
        })
        .expect("set scroll");
    let _: IpcMessage = transport.recv().expect("scroll ack");

    let present = get_present_frame(&mut transport, 5, w, h, page_surface, ui_surface);
    // 未烘焙：全部像素保持提交时的纯红
    assert_eq!(&present.rgba[..4], &[255, 0, 0, 255]);
    assert_eq!(&present.rgba[4..8], &[255, 0, 0, 255]);
    assert_eq!(&present.rgba[8..12], &[255, 0, 0, 255]);
    assert_eq!(&present.rgba[12..16], &[255, 0, 0, 255]);
}

/// RFC 4.5-S2：seccomp 启用时 compositor 帧链路仍可用。
#[test]
fn compositor_seccomp_allows_frame_ipc() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_SANDBOX", "1"), ("ZW_COMPOSITOR_SECCOMP", "1")]);
    let frame = make_frame(4, 4, [0, 128, 255, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 9, 1, 1, frame), (9, 1, 1));
    let got = get_frame(&mut transport, 2, 9, 1, 1);
    assert_eq!((got.width, got.height), (4, 4));
    assert_eq!(&got.rgba[..4], &[0, 128, 255, 255]);
}

/// RFC 4.5-S3：landlock 启用时 compositor 帧链路仍可用。
#[test]
fn compositor_landlock_allows_frame_ipc() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_SANDBOX", "1"), ("ZW_COMPOSITOR_LANDLOCK", "1")]);
    let frame = make_frame(4, 4, [255, 0, 255, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 11, 1, 1, frame), (11, 1, 1));
    let got = get_frame(&mut transport, 2, 11, 1, 1);
    assert_eq!((got.width, got.height), (4, 4));
    assert_eq!(&got.rgba[..4], &[255, 0, 255, 255]);
}

/// RFC §五：模拟 GPU 设备丢失后 CPU 路径仍可出帧。
#[test]
fn compositor_gpu_simulated_loss_falls_back_to_cpu() {
    let (mut transport, _comp) =
        spawn_compositor_with_env(&[("ZW_COMPOSITOR_GPU", "1"), ("ZW_COMPOSITOR_GPU_SIMULATE_LOST", "1")]);
    let frame = make_frame(8, 8, [255, 128, 0, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 12, 1, 1, frame), (12, 1, 1));
    let got = get_frame(&mut transport, 2, 12, 1, 1);
    assert_eq!((got.width, got.height), (8, 8));
    assert_eq!(&got.rgba[..4], &[255, 128, 0, 255]);
}

/// RFC 4.4-S4：窗口 surface 登记与 present 权威标记。
#[test]
fn compositor_window_surface_registers_and_present_is_authoritative() {
    let (mut transport, _comp) = spawn_compositor();
    let window_surface = u64::MAX - 1;

    transport
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::CompositorRegisterWindowSurface(zero_protocol::CompositorWindowSurfaceInfo {
                surface_id: window_surface,
                width: 2,
                height: 2,
            }),
        })
        .expect("register window");
    let ack: IpcMessage = transport.recv().expect("register ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    let page_surface = 7u64;
    let ui_surface = u64::MAX;
    assert_eq!(
        submit_frame(
            &mut transport,
            2,
            page_surface,
            1,
            1,
            make_frame(2, 2, [0, 0, 255, 255])
        ),
        (page_surface, 1, 1)
    );
    transport
        .send(IpcMessage {
            id: 3,
            kind: IpcMessageKind::CompositorRegisterUiSurface(zero_protocol::CompositorUiSurfaceInfo {
                surface_id: ui_surface,
                width: 2,
                height: 2,
            }),
        })
        .expect("register ui");
    let ack: IpcMessage = transport.recv().expect("ui register ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));
    transport
        .send(IpcMessage {
            id: 4,
            kind: IpcMessageKind::CompositorUiFrame {
                surface_id: ui_surface,
                width: 2,
                height: 2,
                rgba: [255u8, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec(),
                shm_name: None,
            },
        })
        .expect("ui frame");
    let ack: IpcMessage = transport.recv().expect("ui frame ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    transport
        .send(IpcMessage {
            id: 5,
            kind: IpcMessageKind::GetCompositorPresentFrame {
                width: 2,
                height: 2,
                page_surface_id: page_surface,
                ui_surface_id: ui_surface,
            },
        })
        .expect("present");
    let resp: IpcMessage = transport.recv().expect("present data");
    match resp.kind {
        IpcMessageKind::CompositorFrameData {
            present_authoritative,
            width,
            height,
            rgba,
            shm_name,
            gpu_image,
            ..
        } => {
            assert!(present_authoritative);
            assert_eq!((width, height), (2, 2));
            let pixels = zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, shm_name, gpu_image)
                .expect("resolve");
            assert_eq!(pixels.len(), 16);
        }
        other => panic!("unexpected {other:?}"),
    }
}

/// RFC 4.5-S2：seccomp + GPU 默认链路共存时 compositor 帧链路仍可用。
#[test]
fn compositor_gpu_seccomp_allows_frame_ipc() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[
        ("ZW_COMPOSITOR_SANDBOX", "1"),
        ("ZW_COMPOSITOR_SECCOMP", "1"),
        ("ZW_COMPOSITOR_GPU", "1"),
    ]);
    let frame = make_frame(4, 4, [255, 64, 32, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 15, 1, 1, frame), (15, 1, 1));
    let got = get_frame(&mut transport, 2, 15, 1, 1);
    assert_eq!((got.width, got.height), (4, 4));
    assert_eq!(got.rgba[0], 255);
}

/// RFC 4.3-S5 + P0：Linux 默认 GPU 链路经 dma-buf 交付（Browser 导入路径）。
///
/// `#[serial]` 与 `zero-protocol::frame_shm` 测共享 serial 组——本测改进程全局
/// `ZW_BROWSER_GPU_DMABUF_IMPORT`（驱动本进程 `resolve_compositor_frame_delivery_fenced`），
/// 须与 frame_shm.rs 中改同一 env 的 serial 测互斥，避免跨 binary 并行竞态。
#[test]
#[cfg(target_os = "linux")]
#[serial_test::serial]
fn compositor_gpu_dmabuf_browser_import_round_trips() {
    if zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(1, 1).is_err() {
        return;
    }
    let (mut transport, _comp) = spawn_compositor_gpu_dmabuf();
    let frame = make_frame(4, 4, [255, 128, 0, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 14, 1, 1, frame), (14, 1, 1));
    let (width, height, resolved) = get_frame_delivery(&mut transport, 2, 14, 1, 1);
    assert_eq!((width, height), (4, 4));
    match resolved {
        zero_protocol::CompositorResolvedFrame::Dmabuf {
            fd,
            stride,
            drm_modifier,
            ..
        } => {
            use zero_render_foundation::gpu::{DRM_FORMAT_ABGR8888, ExportedGpuFrame, map_linear_rgba};
            let export = ExportedGpuFrame {
                fd,
                width,
                height,
                stride,
                drm_fourcc: DRM_FORMAT_ABGR8888,
                drm_modifier,
                sync_fd: None,
            };
            let rgba = map_linear_rgba(&export).expect("map rgba");
            assert_eq!(rgba.len(), 64);
            assert_eq!(rgba[0], 255, "R channel");
            assert!(rgba[1] >= 100, "expected orange G, got {}", rgba[1]);
        }
        zero_protocol::CompositorResolvedFrame::Rgba(_) => {
            panic!("expected dma-buf delivery when ZW_BROWSER_GPU_DMABUF_IMPORT=1");
        }
    }
}

/// RFC 4.3-S5：GPU 纹理 dma-buf fd 导出（memfd 回退）round-trip（CPU 读回路径）。
#[test]
fn compositor_gpu_texture_export_dma_buf_round_trips() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[
        ("ZW_COMPOSITOR_GPU", "1"),
        ("ZW_COMPOSITOR_GPU_IMAGE", "1"),
        ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "1"),
    ]);
    let frame = make_frame(4, 4, [255, 128, 0, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 13, 1, 1, frame), (13, 1, 1));
    let got = get_frame(&mut transport, 2, 13, 1, 1);
    assert_eq!((got.width, got.height), (4, 4));
    assert_eq!(got.rgba[0], 255, "R channel");
    assert!(got.rgba[1] >= 100, "expected orange G, got {}", got.rgba[1]);
    assert_eq!(got.rgba[3], 255, "alpha");
}

/// #6 整链契约：renderer 真实产出格式（FrameModel → PaintSnapshotParams 全字段：
/// viewport/document_height/fills/dirty/navigation_epoch）喂合成器 → 像素回传。
/// 渲染进程 publish 格式由 compositor_publish_tests/paint_export 覆盖，本测验证
/// 合成器消费「与 renderer 打包一致」的完整帧（含 draw_order 与 dirty）。
#[test]
fn compositor_consumes_full_renderer_style_frame() {
    let (mut transport, _comp) = spawn_compositor();
    let mut snapshot = make_frame_with_dirty(
        32,
        32,
        [0, 128, 255, 255],
        vec![IpcRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        }],
    );
    snapshot.document_height = 100.0;
    // renderer 打包会附带 draw_order（插入顺序）——全字段帧
    snapshot.draw_order = vec![zero_protocol::paint_snapshot::IpcDrawOp::Fill(0)];
    assert_eq!(submit_frame(&mut transport, 1, 77, 3, 1, snapshot), (77, 3, 1));
    let frame = get_frame(&mut transport, 2, 77, 3, 1);
    assert_eq!((frame.width, frame.height), (32, 32));
    assert_eq!(&frame.rgba[..4], &[0, 128, 255, 255], "全字段帧应正确光栅化");
}

/// R3285：合成器 GPU 初始光栅后，滚动重绘的 RGBA 回读须正确反映文档坐标。
#[test]
fn compositor_gpu_scroll_transform_rasterizes_scrolled_viewport() {
    let (mut transport, _comp) = spawn_compositor_with_env(&[
        ("ZW_COMPOSITOR_GPU", "1"),
        ("ZW_COMPOSITOR_GPU_TEXTURE_EXPORT", "0"),
        ("ZW_COMPOSITOR_SCROLL_TRANSFORM", "1"),
    ]);
    let frame = make_frame(32, 24, [128, 64, 32, 255]);
    assert_eq!(submit_frame(&mut transport, 1, 9, 4, 1, frame), (9, 4, 1));

    transport
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::CompositorSetScroll {
                surface_id: 9,
                scroll_x: 0.0,
                scroll_y: 8.0,
            },
        })
        .expect("set scroll");
    let ack: IpcMessage = transport.recv().expect("scroll ack");
    assert!(matches!(ack.kind, IpcMessageKind::Ok));

    let frame = get_frame(&mut transport, 3, 9, 4, 1);
    assert_eq!(frame.scroll_y, 8.0, "回读应标记图元已重绘到的文档偏移");
    // 文档第 8px 仍在原 fill 内，滚动后应位于视口顶端。
    let top = 8 * 4;
    assert_eq!(&frame.rgba[top..top + 3], &[128, 64, 32], "滚动后顶部应显示下方内容");
    // 文档末尾后的区域使用正常页面底色，不能再是位图平移留下的透明/黑洞。
    let bottom = (23 * 32 + 16) * 4;
    assert_eq!(
        &frame.rgba[bottom..bottom + 3],
        &[255, 255, 255],
        "底部露出应为页面底色"
    );
}
