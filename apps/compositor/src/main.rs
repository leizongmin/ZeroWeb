//! ZeroWeb 合成器进程（C2 骨架）— 接收渲染进程的图元帧，BackingStore 双缓冲管理。
//!
//! 对照 Ladybird 2026-05 合成器独立进程（调研报告 §3.3/§3.4）：合成与
//! backing store 管理从渲染进程移出。本骨架实现：
//!   - stdio 管道 + bincode IPC（与 image-decoder 同款）
//!   - 接收 `CompositorFrame`（PaintSnapshotParams 图元快照）
//!   - BackingStoreManager 双缓冲：写 back → swap → 保留 front（供显示消费方读取）
//!   - 回复 `CompositorFrameResult`（帧已合成确认）
//!
//! 后续切片（RFC compositor-process-rfc C2/C3）：
//!   - renderer 帧传输接线（当前 renderer 仍直发 browser）
//!   - GPU 光栅化上下文迁移（C3：wgpu 在合成器进程内）
//!   - seccomp 沙箱

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::transport::stdio_transport;
use zero_protocol::{IpcChannel, is_disconnected_channel_message};
use zero_render_foundation::backing_store::BackingStoreManager;
use zero_render_foundation::cpu::render_full_scene_threaded;
use zero_render_foundation::font::{FontLoader, GlyphCache};

mod convert;

use std::io;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(io::stderr)
        .init();

    let mut transport = stdio_transport().unwrap_or_else(|e| panic!("compositor: stdio transport: {e}"));

    // 双缓冲：尺寸随首帧初始化；光栅化所需的字体/字形缓存（进程级单例）
    let mut backing: Option<BackingStoreManager> = None;
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(1024);
    let mut frame_count: u64 = 0;

    // C3 GPU 光栅化（env ZW_COMPOSITOR_GPU=1）：headless wgpu 上下文在合成器
    // 进程内（对照 Ladybird GPU 隔离）；初始化失败/GPU 不可用 → 回退 CPU。
    // 现状：GPU 渲染器覆盖 fills/glyphs 图元子集（render_scene_ext）。
    let gpu_enabled = std::env::var("ZW_COMPOSITOR_GPU").is_ok_and(|v| v == "1");
    let mut gpu_renderer: Option<zero_render_foundation::gpu::renderer::GpuRenderer> = None;

    tracing::info!(
        "zero-compositor 就绪（C2：IPC 图元 → 线程化光栅化 → 双缓冲；GPU={}）",
        gpu_enabled
    );

    loop {
        let msg: IpcMessage = match transport.recv() {
            Ok(m) => m,
            Err(e) => {
                if is_disconnected_channel_message(&e.to_string()) {
                    tracing::info!("compositor: 通道关闭，退出");
                    break;
                }
                tracing::warn!("compositor: 读取失败: {e}");
                continue;
            }
        };

        match msg.kind {
            IpcMessageKind::CompositorFrame(frame) => {
                frame_count += 1;
                let w = frame.viewport_width.max(1);
                let h = frame.viewport_height.max(1);
                let store = backing.get_or_insert_with(|| BackingStoreManager::new(w, h));
                store.resize(w, h);

                // IPC 图元 → 渲染图元 → 光栅化到 back buffer → swap
                let primitives = convert::to_render_primitives(&frame);
                let fb = if gpu_enabled {
                    // C3：GPU 光栅化（headless wgpu 上下文在本进程内；
                    // 初始化失败回退 CPU，fail-open）
                    if gpu_renderer.is_none() {
                        gpu_renderer = zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(w, h).ok();
                    }
                    match gpu_renderer.as_mut() {
                        Some(gpu) => {
                            gpu.render_scene_ext(&primitives.fills, &font_loader, &mut glyph_cache, &[], &[], &[]);
                            match gpu.read_pixels() {
                                Some(pixels) => {
                                    let mut fb = zero_render_foundation::surface::FrameBuffer::new(w, h);
                                    let len = fb.data.len().min(pixels.len());
                                    fb.data[..len].copy_from_slice(&pixels[..len]);
                                    fb
                                }
                                None => render_full_scene_threaded(
                                    w,
                                    h,
                                    1.0,
                                    &primitives,
                                    &font_loader,
                                    &mut glyph_cache,
                                    None,
                                    &[],
                                    &[],
                                    &[],
                                    &[],
                                ),
                            }
                        }
                        None => render_full_scene_threaded(
                            w,
                            h,
                            1.0,
                            &primitives,
                            &font_loader,
                            &mut glyph_cache,
                            None,
                            &[],
                            &[],
                            &[],
                            &[],
                        ),
                    }
                } else {
                    render_full_scene_threaded(
                        w,
                        h,
                        1.0,
                        &primitives,
                        &font_loader,
                        &mut glyph_cache,
                        None,
                        &[],
                        &[],
                        &[],
                        &[],
                    )
                };
                *store.back_mut() = fb;
                store.swap();
                tracing::info!(
                    "compositor: 帧 #{frame_count} 已光栅化并合成（{w}x{h}，fills={}）",
                    primitives.fills.len()
                );

                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameResult { frame_id: frame_count },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 响应失败: {e}");
                    break;
                }
            }
            // 显示消费方拉取最新已合成帧（front 缓冲像素）
            IpcMessageKind::GetCompositorFrame => {
                let (width, height, rgba) = match backing.as_ref() {
                    Some(store) => {
                        let front = store.front();
                        (front.width, front.height, front.data.clone())
                    }
                    None => (0, 0, Vec::new()),
                };
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameData {
                        frame_id: frame_count,
                        width,
                        height,
                        rgba,
                    },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 帧数据响应失败: {e}");
                    break;
                }
            }
            _ => {
                tracing::warn!("compositor: 忽略未知消息");
            }
        }
    }
}
