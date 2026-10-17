//! Shared compositor role entry used by desktop helpers and Android Services.
//!
//! 对照 Ladybird 2026-05 合成器独立进程（调研报告 §3.3/§3.4）：合成与
//! backing store 管理从渲染进程移出。本骨架实现：
//!   - stdio 管道 + bincode IPC（与 image-decoder 同款）
//!   - 接收 `CompositorFrame`（PaintSnapshotParams 图元快照）
//!   - BackingStoreManager 双缓冲：写 back → swap → 保留 front（供显示消费方读取）
//!   - 回复 `CompositorFrameResult`（帧已合成确认）
//!
//! 详见 `docs/goal/archive/compositor-process-rfc-2026-08-07.md`（已实施归档）。

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::{IpcChannel, is_disconnected_channel_message};
use zero_render_foundation::backing_store::BackingStoreManager;
use zero_render_foundation::display_list::DisplayList;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::rendering_thread::{RenderingThread, render_threading_enabled};

mod font_resources;
mod gpu_raster;
mod present;
mod rasterize;
mod recovery;
mod sandbox;
mod scroll_transform;

#[cfg(test)]
mod font_resource_tests;
#[cfg(test)]
mod rasterize_tests;

use std::collections::HashMap;
use std::sync::Arc;

/// dma-buf 帧交付元组（RFC 4.3-S5）；非 Linux 上恒为 None。
type DmaFrameTuple = (
    u64,
    u64,
    u32,
    u32,
    Vec<u8>,
    Option<String>,
    f32,
    f32,
    Option<zero_protocol::GpuSharedImageDescriptor>,
);

struct SurfaceState {
    navigation_epoch: u64,
    frame_id: u64,
    backing: BackingStoreManager,
    scroll_x: f32,
    scroll_y: f32,
    rasterized_scroll_x: f32,
    rasterized_scroll_y: f32,
    paint: Option<zero_protocol::paint_snapshot::PaintSnapshotParams>,
    image_cache: zero_render_foundation::image_cache::ImageCache,
    fonts: zero_paint_convert::fonts::PaintFonts,
    /// 最新已合成帧是否由 GPU 光栅化（GPU 成功置位、CPU 回退清除）。Get 路径的
    /// dma-buf 导出仅在此时允许：预热窗口内的 CPU 帧之后，收编完成的 wgpu 上下文
    /// 纹理仍是未渲染的 1x1 空白，无此门控会以该帧 frame_id 冒充导出空白帧（t9 F1）。
    frame_gpu_rasterized: bool,
}

struct UiSurfaceState {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl SurfaceState {
    fn accepts(&self, navigation_epoch: u64, frame_id: u64) -> bool {
        navigation_epoch > self.navigation_epoch
            || (navigation_epoch == self.navigation_epoch && frame_id > self.frame_id)
    }
}

/// 按 Browser 字体 ID 顺序加载 compositor 光栅化所需的系统字体。
fn load_compositor_fonts() -> FontLoader {
    let platform = zero_render_foundation::font::system::load_platform_fonts();
    tracing::info!(
        "compositor: font fallback chain contains {} fonts",
        platform.loader.fallback_chain().len()
    );
    platform.loader
}

/// P100：图片像素按 key 注入 surface 缓存（帧内载荷与拆批消息共用；幂等）。
fn insert_image_payloads(surface: &mut SurfaceState, payloads: &[zero_protocol::paint_snapshot::IpcImagePayload]) {
    for image in payloads {
        let Ok(data) =
            zero_render_foundation::image_cache::ImageData::from_rgba(image.rgba.clone(), image.width, image.height)
        else {
            tracing::warn!("compositor: image payload {} is invalid", image.image_key);
            continue;
        };
        surface.image_cache.insert_with_key(
            zero_render_foundation::image_cache::ImageKey::new(image.image_key),
            data,
        );
    }
}

/// P100 返修（F1）：帧到达时注入先期挂起的拆批像素，epoch 不匹配的批随帧
/// 失效丢弃（与浏览器侧同规则）；处理后清空该 surface 全部挂起批。独立函数
/// 以便 cfg(test) 覆盖注入语义（run_role 消息循环依赖 transport）。
fn drain_pending_image_batches(
    pending_image_batches: &mut HashMap<u64, Vec<(u64, Vec<zero_protocol::paint_snapshot::IpcImagePayload>)>>,
    surface: &mut SurfaceState,
    surface_id: u64,
    navigation_epoch: u64,
) {
    let Some(pending) = pending_image_batches.get_mut(&surface_id) else {
        return;
    };
    for (batch_epoch, payloads) in pending.drain(..) {
        if batch_epoch == navigation_epoch {
            insert_image_payloads(surface, &payloads);
        } else {
            tracing::debug!(
                "compositor: 丢弃 stale image batch surface {surface_id} epoch {batch_epoch} != {navigation_epoch} n={}",
                payloads.len()
            );
        }
    }
}

/// Runs the compositor role until its IPC peer disconnects.
///
/// The platform bootstrap owns logging and selects a transport. Desktop uses
/// stdio; Android Services will supply a socket-backed transport.
pub fn run_role<C: IpcChannel>(transport: &mut C) {
    sandbox::apply_early_if_enabled();

    // 每个页面 surface 独立维护帧序列和双缓冲；字体/字形缓存由进程共享。
    let mut surfaces: HashMap<u64, SurfaceState> = HashMap::new();
    // P100 返修（F1）：批先于其帧到达且 surface 可能尚未由帧惰性创建，挂起
    // 队列独立于 SurfaceState 按 surface_id 收批；帧到达时按 epoch 注入后清空。
    let mut pending_image_batches: HashMap<u64, Vec<(u64, Vec<zero_protocol::paint_snapshot::IpcImagePayload>)>> =
        HashMap::new();
    let mut ui_surfaces: HashMap<u64, UiSurfaceState> = HashMap::new();
    let mut window_surface: Option<zero_protocol::CompositorWindowSurfaceInfo> = None;
    let base_fonts = Arc::new(load_compositor_fonts());
    let mut font_loader = base_fonts.clone();
    let mut font_namespace: Option<(u64, u64, u64)> = None;
    sandbox::apply_landlock_after_init();
    let mut glyph_cache = GlyphCache::new(1024);
    let mut render_thread = render_threading_enabled().then(|| RenderingThread::spawn(Arc::clone(&font_loader), 1024));

    // C3 GPU 光栅化（Linux 默认开；`ZW_COMPOSITOR_GPU=0` 禁用）：headless wgpu 上下文在合成器
    // 进程内（对照 Ladybird GPU 隔离）；初始化失败/GPU 不可用 → 回退 CPU。
    // 现状：GPU 渲染器覆盖 fills/glyphs 图元子集（render_scene_ext）。
    let gpu_enabled = zero_protocol::compositor_gpu_enabled();
    let mut gpu_renderer: Option<zero_render_foundation::gpu::renderer::GpuRenderer> = None;
    // t9 首帧预热：wgpu 适配器/设备枚举在 WSL2 等环境可达秒级，原先挂在首个页面帧的
    // 光栅化路径内惰性创建（gpu_raster::rasterize_gpu），会阻塞整个 compositor 消息循环
    // ——页面与窗口 chrome 一并停顿，浏览器侧只剩标题占位（首启近空白页 flake）。
    // 改为启动时后台线程预建上下文；就绪前的帧走下方 CPU 回退路径，循环不阻塞。
    // 预热失败不吞错：记 warn 后回到原帧内惰性重试语义。
    let mut gpu_warmup: Option<
        std::thread::JoinHandle<Result<zero_render_foundation::gpu::renderer::GpuRenderer, String>>,
    > = if gpu_enabled {
        match std::thread::Builder::new()
            .name("compositor-gpu-warmup".into())
            .spawn(|| zero_render_foundation::gpu::renderer::GpuRenderer::new_headless(1, 1))
        {
            Ok(handle) => Some(handle),
            Err(error) => {
                tracing::warn!("compositor: GPU 预热线程启动失败，回落帧内惰性初始化: {error}");
                None
            }
        }
    } else {
        None
    };

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

        // 收编已完成的预热结果（任何消息处理前检查，预热完成即对全部路径可见）。
        if gpu_warmup.as_ref().is_some_and(std::thread::JoinHandle::is_finished) {
            let handle = gpu_warmup.take().expect("warmup handle checked above");
            match handle.join() {
                Ok(Ok(renderer)) => gpu_renderer = Some(renderer),
                Ok(Err(error)) => {
                    tracing::warn!("compositor: GPU 上下文预热失败，回落帧内惰性初始化: {error}");
                }
                Err(_) => tracing::warn!("compositor: GPU 预热线程异常退出，回落帧内惰性初始化"),
            }
        }
        // 仍在预热：本帧直接走 CPU 回退，不在消息循环内阻塞等待 wgpu 初始化
        //（t9：阻塞即首帧近空白）。
        let gpu_warming = gpu_warmup.is_some();

        match msg.kind {
            IpcMessageKind::CompositorFrame {
                surface_id,
                navigation_epoch,
                frame_id,
                paint,
            } => {
                if let Some(surface) = surfaces.get(&surface_id)
                    && !surface.accepts(navigation_epoch, frame_id)
                {
                    tracing::warn!(
                        "compositor: 拒绝 surface {surface_id} 的旧帧 \
                         epoch={navigation_epoch}, frame={frame_id}；当前 \
                         epoch={}, frame={}",
                        surface.navigation_epoch,
                        surface.frame_id
                    );
                    let resp = IpcMessage {
                        id: msg.id,
                        kind: IpcMessageKind::CompositorFrameResult {
                            surface_id,
                            navigation_epoch: surface.navigation_epoch,
                            frame_id: surface.frame_id,
                        },
                    };
                    if let Err(e) = transport.send(resp) {
                        tracing::warn!("compositor: 拒绝帧响应失败: {e}");
                        break;
                    }
                    continue;
                }

                let (w, h) = rasterize::physical_viewport_size(&paint);

                // IPC 图元 → 渲染图元 → 光栅化到 back buffer → swap
                // T60：脏区在此归一（整数对齐 + 1px 墨迹外扩，见
                // rasterize::normalize_dirty_rects）——is_full_viewport 门控、GPU
                // scissor 路径与 CPU 回退消费同一像素集合；CPU 路径在
                // rasterize_into_back 内对 paint.dirty_rects 做同一归一。
                let dirty_rects: Vec<(f32, f32, f32, f32)> = rasterize::normalize_dirty_rects(
                    &paint
                        .dirty_rects
                        .iter()
                        .map(|r| (r.x, r.y, r.width, r.height))
                        .collect::<Vec<_>>(),
                    paint.viewport_width.max(1) as f32,
                    paint.viewport_height.max(1) as f32,
                );
                let surface = surfaces.entry(surface_id).or_insert_with(|| SurfaceState {
                    navigation_epoch,
                    frame_id,
                    backing: BackingStoreManager::new(w, h),
                    scroll_x: 0.0,
                    scroll_y: 0.0,
                    rasterized_scroll_x: 0.0,
                    rasterized_scroll_y: 0.0,
                    paint: None,
                    image_cache: zero_render_foundation::image_cache::ImageCache::new(2048, 256 * 1024 * 1024),
                    fonts: zero_paint_convert::fonts::PaintFonts::new(base_fonts.clone()),
                    frame_gpu_rasterized: false,
                });
                if surface.navigation_epoch != navigation_epoch {
                    surface.image_cache.clear();
                    surface.fonts = zero_paint_convert::fonts::PaintFonts::new(base_fonts.clone());
                    // R3254-M4：GPU 纹理缓存同样按导航 epoch 清理（进程级共享 renderer——
                    // clear 会误伤其他 surface 的纹理缓存，仅重传开销，无害）。
                    if let Some(renderer) = gpu_renderer.as_mut() {
                        renderer.clear_image_texture_cache();
                    }
                }
                if let Err(error) = surface.fonts.update(&paint.font_payloads) {
                    tracing::warn!(%error, surface_id, "compositor: rejected font resources");
                    if transport
                        .send(IpcMessage {
                            id: msg.id,
                            kind: IpcMessageKind::Error(error),
                        })
                        .is_err()
                    {
                        break;
                    }
                    continue;
                }
                let namespace = surface.fonts.has_downloaded_fonts().then_some((
                    surface_id,
                    navigation_epoch,
                    surface.fonts.revision,
                ));
                font_resources::activate(
                    namespace,
                    &surface.fonts.loader,
                    &mut font_namespace,
                    &mut font_loader,
                    &mut glyph_cache,
                    &mut gpu_renderer,
                    &mut render_thread,
                );
                insert_image_payloads(surface, &paint.image_payloads);
                // P100 返修（F1）：注入先于帧到达的拆批像素；epoch 不匹配的批
                // 随帧失效丢弃（与浏览器侧同规则），其余注入后清空。挂起队列
                // 独立于 surface（surface 由帧惰性创建，批可能先到）。
                drain_pending_image_batches(&mut pending_image_batches, surface, surface_id, navigation_epoch);
                surface.backing.resize(w, h);
                let mut raster_paint = if zero_protocol::compositor_scroll_transform_enabled() {
                    let scale = rasterize::device_scale_factor(&paint);
                    scroll_transform::paint_for_viewport(&paint, surface.scroll_x / scale, surface.scroll_y / scale)
                } else {
                    (*paint).clone()
                };
                surface.fonts.remap(&mut raster_paint);
                let primitives = zero_paint_convert::to_render_primitives(raster_paint.clone());
                let is_partial = !zero_protocol::compositor_scroll_transform_enabled()
                    && !DisplayList::new(primitives.clone(), dirty_rects.clone())
                        .is_full_viewport(paint.viewport_width.max(1) as f32, paint.viewport_height.max(1) as f32);

                if is_partial {
                    surface.backing.copy_front_to_back();
                }

                // 最新帧是否由 GPU 光栅化成功；失败或 CPU 回退保持 false，
                // 供 Get 路径 dma-buf 导出门控使用（见 SurfaceState::frame_gpu_rasterized）。
                let mut gpu_rasterized = false;
                if gpu_enabled && !gpu_warming {
                    let gpu_ok = if is_partial {
                        gpu_raster::try_rasterize_partial_into_back(
                            &mut gpu_renderer,
                            w,
                            h,
                            &primitives,
                            &font_loader,
                            &mut glyph_cache,
                            &mut surface.image_cache,
                            surface.backing.back_mut(),
                            &dirty_rects,
                            rasterize::device_scale_factor(&paint),
                        )
                    } else {
                        gpu_raster::try_rasterize_fills_into_back(
                            &mut gpu_renderer,
                            w,
                            h,
                            &primitives,
                            &font_loader,
                            &mut glyph_cache,
                            &mut surface.image_cache,
                            surface.backing.back_mut(),
                            rasterize::device_scale_factor(&paint),
                        )
                    };
                    gpu_rasterized = gpu_ok;
                    if !gpu_ok {
                        rasterize::rasterize_into_back(
                            &raster_paint,
                            &primitives,
                            &font_loader,
                            &mut glyph_cache,
                            render_thread.as_ref(),
                            &mut surface.image_cache,
                            surface.backing.back_mut(),
                            is_partial,
                        );
                    }
                } else {
                    rasterize::rasterize_into_back(
                        &raster_paint,
                        &primitives,
                        &font_loader,
                        &mut glyph_cache,
                        render_thread.as_ref(),
                        &mut surface.image_cache,
                        surface.backing.back_mut(),
                        is_partial,
                    );
                }

                surface.backing.swap();
                // R3254-M5：GPU/CPU 光栅化路径统一回收零引用图片（此前 GPU 路径从不 gc，
                // 2048 条目 / 256MB 上限形同虚设，长滚动懒加载页会无限累积解码位图）。
                surface.image_cache.gc();
                surface.navigation_epoch = navigation_epoch;
                surface.frame_id = frame_id;
                surface.frame_gpu_rasterized = gpu_rasterized;
                surface.rasterized_scroll_x = surface.scroll_x;
                surface.rasterized_scroll_y = surface.scroll_y;
                surface.paint = Some(*paint);
                tracing::info!(
                    "compositor: surface {surface_id} 帧 #{frame_id} 已光栅化并合成\
                     （epoch={navigation_epoch}，{w}x{h}，fills={}）",
                    primitives.fills.len()
                );
                if std::env::var("ZERO_BROWSER_PRODUCT_SMOKE").as_deref() == Ok("1") {
                    tracing::info!(
                        "SMOKE_EVENT component=zero-compositor event=frame_committed surface={surface_id} epoch={navigation_epoch} frame={frame_id} width={w} height={h}"
                    );
                }

                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameResult {
                        surface_id,
                        navigation_epoch,
                        frame_id,
                    },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 响应失败: {e}");
                    break;
                }
            }
            // 显示消费方拉取最新已合成帧（front 缓冲像素）
            IpcMessageKind::GetCompositorFrame { surface_id, .. } => {
                // dma-buf 导出（RFC 4.3-S5）仅 Linux；非 Linux 上无此通道
                #[cfg(target_os = "linux")]
                let mut fd_publish: Option<zero_protocol::CompositorFrameDelivery> = None;
                let (response_epoch, response_frame, width, height, rgba, shm_name, scroll_x, scroll_y, gpu_image) =
                    match surfaces.get_mut(&surface_id) {
                        Some(surface) => {
                            if zero_protocol::compositor_scroll_transform_enabled()
                                && (surface.rasterized_scroll_x != surface.scroll_x
                                    || surface.rasterized_scroll_y != surface.scroll_y)
                                && let Some(paint) = surface.paint.clone()
                            {
                                let scale = rasterize::device_scale_factor(&paint);
                                let mut viewport_paint = scroll_transform::paint_for_viewport(
                                    &paint,
                                    surface.scroll_x / scale,
                                    surface.scroll_y / scale,
                                );
                                let namespace = surface.fonts.has_downloaded_fonts().then_some((
                                    surface_id,
                                    surface.navigation_epoch,
                                    surface.fonts.revision,
                                ));
                                font_resources::activate(
                                    namespace,
                                    &surface.fonts.loader,
                                    &mut font_namespace,
                                    &mut font_loader,
                                    &mut glyph_cache,
                                    &mut gpu_renderer,
                                    &mut render_thread,
                                );
                                surface.fonts.remap(&mut viewport_paint);
                                let primitives = zero_paint_convert::to_render_primitives(viewport_paint.clone());
                                // 滚动后的可见区与旧 back buffer 没有可复用的坐标关系；全量重绘。
                                rasterize::rasterize_into_back(
                                    &viewport_paint,
                                    &primitives,
                                    &font_loader,
                                    &mut glyph_cache,
                                    render_thread.as_ref(),
                                    &mut surface.image_cache,
                                    surface.backing.back_mut(),
                                    false,
                                );
                                surface.backing.swap();
                                surface.rasterized_scroll_x = surface.scroll_x;
                                surface.rasterized_scroll_y = surface.scroll_y;
                            }
                            let front = surface.backing.front();
                            // 返回此位图已烘焙的文档偏移。Browser 用它计算仍在
                            // 飞行中的本地滚动差值，避免滚动条和页面像素脱节。
                            let scroll_x = surface.scroll_x;
                            let scroll_y = surface.scroll_y;
                            let pixel_data = front.data.clone();

                            #[cfg(target_os = "linux")]
                            let dma_frame: Option<DmaFrameTuple> = {
                                if !zero_protocol::compositor_scroll_transform_enabled()
                                    && zero_protocol::compositor_gpu_texture_export_enabled()
                                    && zero_protocol::compositor_gpu_image_enabled()
                                    && gpu_enabled
                                    // t9 F1：只导出确由 GPU 光栅化的最新帧。预热收编边界上
                                    // GPU 纹理可能仍是未渲染的 1x1 空白（最近一帧走了 CPU），
                                    // 此时回落 Rgba 交付，不得以旧 frame_id 冒充导出。
                                    && surface.frame_gpu_rasterized
                                    && let Some(gpu) = gpu_renderer.as_ref()
                                    && let Ok(exported) = zero_render_foundation::gpu::try_export_headless(gpu)
                                {
                                    use std::os::fd::IntoRawFd;
                                    let del = zero_protocol::build_compositor_dma_buf_delivery(
                                        surface_id,
                                        surface.frame_id,
                                        exported.width,
                                        exported.height,
                                        exported.stride,
                                        exported.drm_fourcc,
                                        exported.drm_modifier,
                                        surface.frame_id,
                                        exported.fd.into_raw_fd(),
                                    );
                                    let frame_tuple = (
                                        surface.navigation_epoch,
                                        surface.frame_id,
                                        front.width,
                                        front.height,
                                        del.rgba.clone(),
                                        del.shm_name.clone(),
                                        scroll_x,
                                        scroll_y,
                                        del.gpu_image.clone(),
                                    );
                                    fd_publish = Some(del);
                                    Some(frame_tuple)
                                } else {
                                    None
                                }
                            };
                            #[cfg(not(target_os = "linux"))]
                            let dma_frame: Option<DmaFrameTuple> = None;

                            if let Some(frame_tuple) = dma_frame {
                                frame_tuple
                            } else {
                                let delivery = deliver_pixels_or_inline(
                                    surface_id,
                                    surface.frame_id,
                                    &pixel_data,
                                    front.width,
                                    front.height,
                                );
                                (
                                    surface.navigation_epoch,
                                    surface.frame_id,
                                    front.width,
                                    front.height,
                                    delivery.rgba,
                                    delivery.shm_name,
                                    scroll_x,
                                    scroll_y,
                                    delivery.gpu_image,
                                )
                            }
                        }
                        None => (0, 0, 0, 0, Vec::new(), None, 0.0, 0.0, None),
                    };
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameData {
                        surface_id,
                        navigation_epoch: response_epoch,
                        frame_id: response_frame,
                        width,
                        height,
                        rgba,
                        shm_name,
                        scroll_x,
                        scroll_y,
                        gpu_image,
                        present_authoritative: false,
                    },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 帧数据响应失败: {e}");
                    break;
                }
                #[cfg(target_os = "linux")]
                if let Some(mut pending) = fd_publish
                    && let Err(error) = zero_protocol::publish_compositor_fd(&mut pending)
                {
                    tracing::warn!("compositor: dma-buf fd 发布失败: {error}");
                }
            }
            IpcMessageKind::ImagePayloadBatch {
                // P100 返修（F1）：compositor 发布模式帧载荷置空，像素经批先行
                // 到达；挂起至同 epoch 帧到达时注入（帧处理处），过期批随帧失效。
                // surface 尚未由帧创建也照常挂起。Legacy 批（None）不应到此进程。
                surface_id: Some(surface_id),
                navigation_epoch,
                payloads,
            } => {
                // 单 surface 挂起批上界：一个发布周期的批数（133MiB/48MiB ≈ 3）。
                // surface 释放后迟到的批无人排空，超界丢最旧并留痕，防像素滞留。
                const MAX_PENDING_BATCHES_PER_SURFACE: usize = 8;
                let entry = pending_image_batches.entry(surface_id).or_default();
                if entry.len() >= MAX_PENDING_BATCHES_PER_SURFACE {
                    entry.remove(0);
                    tracing::warn!("compositor: surface {surface_id} 挂起批超界，丢弃最旧批");
                }
                entry.push((navigation_epoch, payloads));
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::Ok,
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: image batch 响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::ImagePayloadBatch {
                surface_id: None,
                navigation_epoch,
                payloads,
            } => {
                tracing::debug!(
                    "compositor: 忽略无 surface 的 image batch epoch {navigation_epoch} n={}",
                    payloads.len()
                );
            }
            IpcMessageKind::ReleaseCompositorSurface { surface_id } => {
                // 挂起批随 surface 一起清理，避免 tab 关闭后像素滞留。
                pending_image_batches.remove(&surface_id);
                if font_namespace.is_some_and(|(id, _, _)| id == surface_id) {
                    font_resources::activate(
                        None,
                        &base_fonts,
                        &mut font_namespace,
                        &mut font_loader,
                        &mut glyph_cache,
                        &mut gpu_renderer,
                        &mut render_thread,
                    );
                }
                if surfaces.remove(&surface_id).is_some() {
                    tracing::info!("compositor: surface {surface_id} 已释放");
                }
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::Ok,
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: surface 释放响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::CompositorSetScroll {
                surface_id,
                scroll_x,
                scroll_y,
            } => {
                if let Some(surface) = surfaces.get_mut(&surface_id) {
                    surface.scroll_x = scroll_x;
                    surface.scroll_y = scroll_y;
                }
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::Ok,
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: scroll 更新响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::CompositorRegisterUiSurface(info) => {
                ui_surfaces.insert(
                    info.surface_id,
                    UiSurfaceState {
                        width: info.width,
                        height: info.height,
                        rgba: Vec::new(),
                    },
                );
                tracing::info!(
                    "compositor: UI surface {} 注册 {}x{}（4.4 切片）",
                    info.surface_id,
                    info.width,
                    info.height
                );
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::Ok,
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: UI surface 注册响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::CompositorRegisterWindowSurface(info) => {
                window_surface = Some(info.clone());
                tracing::info!(
                    "compositor: 窗口 surface {} 登记 {}x{}（4.4-S4 所有权）",
                    info.surface_id,
                    info.width,
                    info.height
                );
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::Ok,
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 窗口 surface 登记响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::CompositorUiFrame {
                surface_id,
                width,
                height,
                rgba,
                shm_name,
            } => {
                let stored = match zero_protocol::resolve_compositor_frame_rgba(width, height, rgba, shm_name, None) {
                    Ok(pixels) => pixels,
                    Err(e) => {
                        tracing::warn!("compositor: UI 帧像素无效: {e}");
                        let resp = IpcMessage {
                            id: msg.id,
                            kind: IpcMessageKind::Error(e.to_string()),
                        };
                        let _ = transport.send(resp);
                        continue;
                    }
                };
                if let Some(ui) = ui_surfaces.get_mut(&surface_id) {
                    ui.width = width;
                    ui.height = height;
                    ui.rgba = stored;
                    tracing::info!("compositor: UI surface {surface_id} 位图已更新 {width}x{height}");
                } else {
                    ui_surfaces.insert(
                        surface_id,
                        UiSurfaceState {
                            width,
                            height,
                            rgba: stored,
                        },
                    );
                }
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::Ok,
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: UI 帧响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::GetCompositorUiFrame { surface_id } => {
                let (width, height, rgba, shm_name, gpu_image) = match ui_surfaces.get(&surface_id) {
                    Some(ui) if !ui.rgba.is_empty() => {
                        let delivery = zero_protocol::deliver_compositor_frame_pixels(
                            surface_id, 0, &ui.rgba, ui.width, ui.height,
                        )
                        .unwrap_or_else(|e| {
                            tracing::warn!("compositor: UI 帧交付失败，回退内联 rgba: {e}");
                            zero_protocol::CompositorFrameDelivery {
                                rgba: ui.rgba.clone(),
                                shm_name: None,
                                gpu_image: None,
                                #[cfg(target_os = "linux")]
                                pending_fd: None,
                            }
                        });
                        (
                            ui.width,
                            ui.height,
                            delivery.rgba,
                            delivery.shm_name,
                            delivery.gpu_image,
                        )
                    }
                    _ => (0, 0, Vec::new(), None, None),
                };
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameData {
                        surface_id,
                        navigation_epoch: 0,
                        frame_id: 0,
                        width,
                        height,
                        rgba,
                        shm_name,
                        scroll_x: 0.0,
                        scroll_y: 0.0,
                        gpu_image,
                        present_authoritative: false,
                    },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: UI 帧读取响应失败: {e}");
                    break;
                }
            }
            IpcMessageKind::GetCompositorPresentFrame {
                width,
                height,
                page_surface_id,
                ui_surface_id,
            } => {
                let page_pixels = surfaces.get(&page_surface_id).map(|s| {
                    let front = s.backing.front();
                    (front.width, front.height, front.data.clone())
                });
                let ui_pixels = ui_surfaces
                    .get(&ui_surface_id)
                    .map(|ui| (ui.width, ui.height, ui.rgba.clone()));
                let (out_w, out_h, rgba, shm_name, gpu_image) = match (page_pixels, ui_pixels) {
                    (Some((pw, ph, page)), Some((uw, uh, ui))) if !page.is_empty() && !ui.is_empty() => {
                        let composed = present::composite_present_frame(width, height, &page, pw, ph, &ui, uw, uh);
                        let delivery = zero_protocol::deliver_compositor_frame_pixels(
                            page_surface_id,
                            0,
                            &composed,
                            width,
                            height,
                        )
                        .unwrap_or_else(|e| {
                            tracing::warn!("compositor: present 交付失败，回退内联 rgba: {e}");
                            zero_protocol::CompositorFrameDelivery {
                                rgba: composed,
                                shm_name: None,
                                gpu_image: None,
                                #[cfg(target_os = "linux")]
                                pending_fd: None,
                            }
                        });
                        (width, height, delivery.rgba, delivery.shm_name, delivery.gpu_image)
                    }
                    _ => (0, 0, Vec::new(), None, None),
                };
                let resp = IpcMessage {
                    id: msg.id,
                    kind: IpcMessageKind::CompositorFrameData {
                        surface_id: page_surface_id,
                        navigation_epoch: 0,
                        frame_id: 0,
                        width: out_w,
                        height: out_h,
                        rgba,
                        shm_name,
                        scroll_x: 0.0,
                        scroll_y: 0.0,
                        gpu_image,
                        present_authoritative: window_surface.is_some(),
                    },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: present 帧响应失败: {e}");
                    break;
                }
            }
            _ => {
                tracing::warn!("compositor: 忽略未知消息");
            }
        }
    }
}

fn deliver_pixels_or_inline(
    surface_id: u64,
    frame_id: u64,
    pixel_data: &[u8],
    width: u32,
    height: u32,
) -> zero_protocol::CompositorFrameDelivery {
    zero_protocol::deliver_compositor_frame_pixels(surface_id, frame_id, pixel_data, width, height).unwrap_or_else(
        |e| {
            tracing::warn!("compositor: 帧交付失败，回退内联 rgba: {e}");
            zero_protocol::CompositorFrameDelivery {
                rgba: pixel_data.to_vec(),
                shm_name: None,
                gpu_image: None,
                #[cfg(target_os = "linux")]
                pending_fd: None,
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compositor_font_loader_rasterizes_page_text() {
        let loader = load_compositor_fonts();
        let bitmap = loader
            .rasterize_glyph_with_fallback(0, 'A', 18.0)
            .expect("compositor must load a primary system font")
            .1;
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
        assert!(bitmap.data.iter().any(|alpha| *alpha != 0));
    }

    fn test_surface(navigation_epoch: u64) -> SurfaceState {
        SurfaceState {
            navigation_epoch,
            frame_id: 0,
            backing: BackingStoreManager::new(1, 1),
            scroll_x: 0.0,
            scroll_y: 0.0,
            rasterized_scroll_x: 0.0,
            rasterized_scroll_y: 0.0,
            paint: None,
            image_cache: zero_render_foundation::image_cache::ImageCache::new(16, 1024 * 1024),
            fonts: zero_paint_convert::fonts::PaintFonts::new(std::sync::Arc::new(load_compositor_fonts())),
            frame_gpu_rasterized: false,
        }
    }

    fn payload(key: u64) -> zero_protocol::paint_snapshot::IpcImagePayload {
        zero_protocol::paint_snapshot::IpcImagePayload {
            image_key: key,
            width: 1,
            height: 1,
            rgba: vec![255, 0, 0, 255],
        }
    }

    /// P100 返修（F1）：批先于帧到达（surface 可能尚未由帧创建）也照常挂起；
    /// 同 epoch 帧到达时按 key 注入。
    #[test]
    fn pending_batch_injects_when_matching_frame_arrives() {
        let mut pending: HashMap<u64, Vec<(u64, Vec<_>)>> = HashMap::new();
        // surface 尚不存在也先挂起（帧惰性建 surface 前像素已到）。
        pending.entry(5).or_default().push((3, vec![payload(42)]));
        let mut surface = test_surface(3);
        drain_pending_image_batches(&mut pending, &mut surface, 5, 3);
        assert!(
            surface
                .image_cache
                .get(&zero_render_foundation::image_cache::ImageKey::new(42))
                .is_some()
        );
        // 注入后挂起清空，重复帧不重复注入残留。
        assert!(pending.get(&5).is_none_or(|v| v.is_empty()));
    }

    /// P100 返修（F1）：过期 epoch 批随帧失效丢弃（不污染新导航画面）。
    #[test]
    fn stale_pending_batch_is_dropped_on_epoch_mismatch() {
        let mut pending: HashMap<u64, Vec<(u64, Vec<_>)>> = HashMap::new();
        pending.entry(5).or_default().push((2, vec![payload(42)]));
        let mut surface = test_surface(3);
        drain_pending_image_batches(&mut pending, &mut surface, 5, 3);
        assert!(
            surface
                .image_cache
                .get(&zero_render_foundation::image_cache::ImageKey::new(42))
                .is_none()
        );
    }
}
