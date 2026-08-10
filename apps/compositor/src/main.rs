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
use zero_render_foundation::display_list::DisplayList;
use zero_render_foundation::font::{FontLoader, GlyphCache};
use zero_render_foundation::rendering_thread::{RenderingThread, render_threading_enabled};

mod convert;
mod gpu_raster;
mod rasterize;
mod sandbox;

#[cfg(test)]
mod rasterize_tests;

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

struct SurfaceState {
    navigation_epoch: u64,
    frame_id: u64,
    backing: BackingStoreManager,
    scroll_x: f32,
    scroll_y: f32,
}

impl SurfaceState {
    fn accepts(&self, navigation_epoch: u64, frame_id: u64) -> bool {
        navigation_epoch > self.navigation_epoch
            || (navigation_epoch == self.navigation_epoch && frame_id > self.frame_id)
    }
}

fn primary_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/SFCompact.ttf",
            "/System/Library/Fonts/HelveticaNeue.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\segoeui.ttf", "C:\\Windows\\Fonts\\arial.ttf"]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/opentype/cantarell/Cantarell-VF.otf",
            "/usr/share/fonts/truetype/cantarell/Cantarell-Regular.otf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
        ]
    }
}

fn bold_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\arialbd.ttf", "C:\\Windows\\Fonts\\segoeuib.ttf"]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        ]
    }
}

fn fallback_font_paths() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/Apple Symbols.ttf",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &["C:\\Windows\\Fonts\\msyh.ttc", "C:\\Windows\\Fonts\\seguiemj.ttf"]
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/noto/NotoEmoji-Regular.ttf",
        ]
    }
}

/// 按 Browser 字体 ID 顺序加载 compositor 光栅化所需的系统字体。
fn load_compositor_fonts() -> FontLoader {
    let mut loader = FontLoader::new();
    let primary = primary_font_paths().iter().find_map(|path| {
        let data = std::fs::read(path).ok()?;
        let id = loader.load_font(&data).ok()?;
        tracing::info!("compositor: loaded primary font {path} (id={id})");
        Some(id)
    });
    let Some(primary) = primary else {
        tracing::warn!("compositor: no system font available; page text may be missing");
        return loader;
    };

    if let Some((id, path)) = bold_font_paths().iter().find_map(|path| {
        let data = std::fs::read(path).ok()?;
        let id = loader.load_font(&data).ok()?;
        Some((id, *path))
    }) {
        tracing::info!("compositor: loaded bold font {path} (id={id})");
    }

    let fallbacks = fallback_font_paths()
        .iter()
        .filter_map(|path| {
            let data = std::fs::read(path).ok()?;
            let id = loader.load_font(&data).ok()?;
            (id != primary).then_some(id)
        })
        .collect::<Vec<_>>();
    loader.set_fallback_chain(fallbacks);
    tracing::info!(
        "compositor: font fallback chain contains {} fonts",
        loader.fallback_chain().len()
    );
    loader
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(io::stderr)
        .init();

    sandbox::apply_if_enabled();

    let mut transport = stdio_transport().unwrap_or_else(|e| panic!("compositor: stdio transport: {e}"));

    // 每个页面 surface 独立维护帧序列和双缓冲；字体/字形缓存由进程共享。
    let mut surfaces: HashMap<u64, SurfaceState> = HashMap::new();
    let mut ui_surfaces: HashMap<u64, (u32, u32)> = HashMap::new();
    let font_loader = Arc::new(load_compositor_fonts());
    let mut glyph_cache = GlyphCache::new(1024);
    let render_thread = render_threading_enabled().then(|| RenderingThread::spawn(Arc::clone(&font_loader), 1024));

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

                let w = paint.viewport_width.max(1);
                let h = paint.viewport_height.max(1);

                // IPC 图元 → 渲染图元 → 光栅化到 back buffer → swap
                let primitives = convert::to_render_primitives(&paint);
                let dirty_rects: Vec<(f32, f32, f32, f32)> = paint
                    .dirty_rects
                    .iter()
                    .map(|r| (r.x, r.y, r.width, r.height))
                    .collect();
                let is_partial =
                    !DisplayList::new(primitives.clone(), dirty_rects.clone()).is_full_viewport(w as f32, h as f32);

                let surface = surfaces.entry(surface_id).or_insert_with(|| SurfaceState {
                    navigation_epoch,
                    frame_id,
                    backing: BackingStoreManager::new(w, h),
                    scroll_x: 0.0,
                    scroll_y: 0.0,
                });
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
                            surface.backing.back_mut(),
                            &dirty_rects,
                        )
                    } else {
                        gpu_raster::try_rasterize_fills_into_back(
                            &mut gpu_renderer,
                            w,
                            h,
                            &primitives,
                            &font_loader,
                            &mut glyph_cache,
                            surface.backing.back_mut(),
                        )
                    };
                    if !gpu_ok {
                        rasterize::rasterize_into_back(
                            &paint,
                            &primitives,
                            &font_loader,
                            &mut glyph_cache,
                            render_thread.as_ref(),
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
                        surface.backing.back_mut(),
                        is_partial,
                    );
                }

                surface.backing.swap();
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
                let (response_epoch, response_frame, width, height, rgba, shm_name, scroll_x, scroll_y) =
                    match surfaces.get(&surface_id) {
                        Some(surface) => {
                            let front = surface.backing.front();
                            let (rgba, shm_name) = if zero_protocol::compositor_shm_enabled() {
                                let name =
                                    zero_protocol::publish_compositor_frame(surface_id, surface.frame_id, &front.data)
                                        .unwrap_or_else(|e| {
                                            tracing::warn!("compositor: shm 发布失败，回退内联 rgba: {e}");
                                            String::new()
                                        });
                                if name.is_empty() {
                                    (front.data.clone(), None)
                                } else {
                                    (Vec::new(), Some(name))
                                }
                            } else {
                                (front.data.clone(), None)
                            };
                            (
                                surface.navigation_epoch,
                                surface.frame_id,
                                front.width,
                                front.height,
                                rgba,
                                shm_name,
                                surface.scroll_x,
                                surface.scroll_y,
                            )
                        }
                        None => (0, 0, 0, 0, Vec::new(), None, 0.0, 0.0),
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
                        gpu_image: None,
                    },
                };
                if let Err(e) = transport.send(resp) {
                    tracing::warn!("compositor: 帧数据响应失败: {e}");
                    break;
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
                ui_surfaces.insert(info.surface_id, (info.width, info.height));
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
            _ => {
                tracing::warn!("compositor: 忽略未知消息");
            }
        }
    }
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
