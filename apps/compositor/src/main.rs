//! ZeroWeb 合成器进程（C2 骨架）— 接收渲染进程的图元帧，BackingStore 双缓冲管理。
//!
// Windows：GUI 子系统。compositor 由 browser 通过 stdin/stdout 管道 spawn，
// 不需要控制台；不加此项 Windows 会为子进程分配一个控制台窗口。
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]
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
use zero_protocol::transport::stdio_transport;
use zero_protocol::{IpcChannel, is_disconnected_channel_message};
use zero_render_foundation::backing_store::BackingStoreManager;
use zero_render_foundation::display_list::DisplayList;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::rendering_thread::{RenderingThread, render_threading_enabled};

mod convert;
mod gpu_raster;
mod present;
mod rasterize;
mod recovery;
mod sandbox;
mod scroll_transform;

#[cfg(test)]
mod rasterize_tests;

use std::collections::HashMap;
use std::io;
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
    image_cache: zero_render_foundation::image_cache::ImageCache,
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

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(io::stderr)
        .init();

    sandbox::apply_early_if_enabled();

    let mut transport = stdio_transport().unwrap_or_else(|e| panic!("compositor: stdio transport: {e}"));

    // 每个页面 surface 独立维护帧序列和双缓冲；字体/字形缓存由进程共享。
    let mut surfaces: HashMap<u64, SurfaceState> = HashMap::new();
    let mut ui_surfaces: HashMap<u64, UiSurfaceState> = HashMap::new();
    let mut window_surface: Option<zero_protocol::CompositorWindowSurfaceInfo> = None;
    let font_loader = Arc::new(load_compositor_fonts());
    sandbox::apply_landlock_after_init();
    let mut glyph_cache = GlyphCache::new(1024);
    let render_thread = render_threading_enabled().then(|| RenderingThread::spawn(Arc::clone(&font_loader), 1024));

    // C3 GPU 光栅化（Linux 默认开；`ZW_COMPOSITOR_GPU=0` 禁用）：headless wgpu 上下文在合成器
    // 进程内（对照 Ladybird GPU 隔离）；初始化失败/GPU 不可用 → 回退 CPU。
    // 现状：GPU 渲染器覆盖 fills/glyphs 图元子集（render_scene_ext）。
    let gpu_enabled = zero_protocol::compositor_gpu_enabled();
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
                let primitives = convert::to_render_primitives(&paint);
                let dirty_rects: Vec<(f32, f32, f32, f32)> = paint
                    .dirty_rects
                    .iter()
                    .map(|r| (r.x, r.y, r.width, r.height))
                    .collect();
                let is_partial = !DisplayList::new(primitives.clone(), dirty_rects.clone())
                    .is_full_viewport(paint.viewport_width.max(1) as f32, paint.viewport_height.max(1) as f32);

                let surface = surfaces.entry(surface_id).or_insert_with(|| SurfaceState {
                    navigation_epoch,
                    frame_id,
                    backing: BackingStoreManager::new(w, h),
                    scroll_x: 0.0,
                    scroll_y: 0.0,
                    image_cache: zero_render_foundation::image_cache::ImageCache::new(2048, 256 * 1024 * 1024),
                });
                if surface.navigation_epoch != navigation_epoch {
                    surface.image_cache.clear();
                    // R3254-M4：GPU 纹理缓存同样按导航 epoch 清理（进程级共享 renderer——
                    // clear 会误伤其他 surface 的纹理缓存，仅重传开销，无害）。
                    if let Some(renderer) = gpu_renderer.as_mut() {
                        renderer.clear_image_texture_cache();
                    }
                }
                for image in &paint.image_payloads {
                    let Ok(data) = zero_render_foundation::image_cache::ImageData::from_rgba(
                        image.rgba.clone(),
                        image.width,
                        image.height,
                    ) else {
                        tracing::warn!("compositor: image payload {} is invalid", image.image_key);
                        continue;
                    };
                    surface.image_cache.insert_with_key(
                        zero_render_foundation::image_cache::ImageKey::new(image.image_key),
                        data,
                    );
                }
                surface.backing.resize(w, h);

                if is_partial {
                    surface.backing.copy_front_to_back();
                }

                if gpu_enabled {
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
                    if !gpu_ok {
                        rasterize::rasterize_into_back(
                            &paint,
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
                        &paint,
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
                    match surfaces.get(&surface_id) {
                        Some(surface) => {
                            let front = surface.backing.front();
                            let mut scroll_x = surface.scroll_x;
                            let mut scroll_y = surface.scroll_y;
                            let pixel_data = if zero_protocol::compositor_scroll_transform_enabled()
                                && (scroll_x != 0.0 || scroll_y != 0.0)
                            {
                                scroll_x = 0.0;
                                scroll_y = 0.0;
                                scroll_transform::bake_scroll_into_rgba(
                                    &front.data,
                                    front.width,
                                    front.height,
                                    surface.scroll_x,
                                    surface.scroll_y,
                                )
                            } else {
                                front.data.clone()
                            };

                            #[cfg(target_os = "linux")]
                            let dma_frame: Option<DmaFrameTuple> = {
                                if zero_protocol::compositor_gpu_texture_export_enabled()
                                    && zero_protocol::compositor_gpu_image_enabled()
                                    && gpu_enabled
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
            IpcMessageKind::ReleaseCompositorSurface { surface_id } => {
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
}
