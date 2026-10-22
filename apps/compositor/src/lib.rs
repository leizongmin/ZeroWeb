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

mod animation_overrides;
mod font_resources;
mod frame_dump;
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
    /// Get 路径 CPU 重采样（滚动/动画）后 front 缓冲比最近一次 GPU 提交新——
    /// GPU 纹理仍是提交时基线，dma-buf 导出会以过期纹理冒充当前帧；置位期间
    /// 回落 Rgba/shm 交付，提交时清除。
    front_cpu_resampled: bool,
    /// M2-S1c：上一次 Get 采样判定的动画活跃标志。全部动画在两次 Get 之间结束时，
    /// 仍须再走一次基线重光栅（fill none 结束态 = 基线），之后停止。
    animations_active: bool,
    /// M3-S1c：front buffer 需重贴 worker patch（Update 到达或基线被重光栅/新帧
    /// 整体覆盖后置位；Get 响应前贴片并清除）。
    patches_dirty: bool,
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

/// M2-S1c：动画声明状态最新生效存储（协议契约：每 (surface, epoch) 以 generation
/// 单调替换；旧 epoch / 旧代际丢弃；surface 释放随清，epoch 推进在帧提交处清理）。
/// 独立于 `SurfaceState`：声明可能先于其首个帧到达（帧惰性建 surface），Get 时
/// 再按 `navigation_epoch` 与 surface 对齐。
fn store_animation_state(
    states: &mut HashMap<u64, zero_protocol::animation::CompositorAnimationStateParams>,
    params: zero_protocol::animation::CompositorAnimationStateParams,
) {
    match states.get(&params.surface_id) {
        Some(existing)
            if (existing.navigation_epoch, existing.generation) >= (params.navigation_epoch, params.generation) =>
        {
            tracing::debug!(
                "compositor: 丢弃 stale 动画声明 surface {} epoch/gen {}/{} ≤ 当前 {}/{}",
                params.surface_id,
                params.navigation_epoch,
                params.generation,
                existing.navigation_epoch,
                existing.generation
            );
        }
        _ => {
            // M2-S1e：接受路径记 debug——空集清理（declarations=0）会终止 Get 侧
            // 采样与浏览器 keep-alive，是负载窗断续归因的关键事件。
            tracing::debug!(
                "compositor: 接受动画声明 surface {} epoch/gen {}/{} declarations={}",
                params.surface_id,
                params.navigation_epoch,
                params.generation,
                params.declarations.len()
            );
            states.insert(params.surface_id, params);
        }
    }
}

/// M2-S1c：帧提交推进 epoch 后清理跨代声明（同 `surface.paint` 生命周期；新
/// epoch 的声明由 renderer 状态消息随后重发，Get 侧另有同 epoch 校验兜底）。
fn cleanup_stale_animation_state(
    states: &mut HashMap<u64, zero_protocol::animation::CompositorAnimationStateParams>,
    surface_id: u64,
    navigation_epoch: u64,
) {
    if states
        .get(&surface_id)
        .is_some_and(|state| state.navigation_epoch != navigation_epoch)
    {
        states.remove(&surface_id);
    }
}

/// M3-S1c：worker canvas patch 贴片段落（RFC `m3-slice-worker-canvas.md` §2.3）。
/// 独立于 `SurfaceState`（patch 可先于其首个帧到达——帧惰性建 surface，声明存储
/// 同款）；Get 期贴到冻结期基线帧之上的注册矩形。
struct PatchState {
    navigation_epoch: u64,
    /// 目标矩形（文档空间 CSS px，Register 锚定；冻结期布局恒定——M1 前提）。
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    /// 最新位图（RGBA8，Update 全量替换）。
    bitmap_w: u32,
    bitmap_h: u32,
    rgba: Vec<u8>,
    /// worker 已终止：定格最后位图、拒绝后续 Update（HTML spec canvas 保留位图
    /// 语义——Detach 后仍参与贴片，呈现不回退空白）。
    detached: bool,
}

/// patch 注册（stale epoch 丢弃；同 (surface, patch) 同 epoch 重注册覆盖矩形——
/// epoch 回退的重复 Register 不复活 detached 态）。
#[allow(clippy::too_many_arguments)] // 8 参 = surface/patch 双键 + epoch + 元素矩形四元，与 wire 字段一一对应
fn store_patch_register(
    patches: &mut HashMap<u64, HashMap<u64, PatchState>>,
    surface_id: u64,
    navigation_epoch: u64,
    patch_id: u64,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let entry = patches.entry(surface_id).or_default();
    if let Some(existing) = entry.get(&patch_id)
        && existing.navigation_epoch > navigation_epoch
    {
        tracing::debug!(
            "compositor: 丢弃 stale patch 注册 surface {surface_id} patch {patch_id} epoch {navigation_epoch} < {}",
            existing.navigation_epoch
        );
        return;
    }
    entry.insert(
        patch_id,
        PatchState {
            navigation_epoch,
            x,
            y,
            width,
            height,
            bitmap_w: 0,
            bitmap_h: 0,
            rgba: Vec::new(),
            detached: false,
        },
    );
}

/// patch 位图更新（仅已注册、同 epoch、未 detached 接受）。返回是否接受（调用方
/// 据此置 surface 重贴标志）。
fn store_patch_update(
    patches: &mut HashMap<u64, HashMap<u64, PatchState>>,
    surface_id: u64,
    navigation_epoch: u64,
    patch_id: u64,
    bitmap_w: u32,
    bitmap_h: u32,
    rgba: Vec<u8>,
) -> bool {
    let Some(patch) = patches.get_mut(&surface_id).and_then(|entry| entry.get_mut(&patch_id)) else {
        tracing::debug!("compositor: 丢弃未注册 patch 更新 surface {surface_id} patch {patch_id}");
        return false;
    };
    if patch.navigation_epoch != navigation_epoch || patch.detached {
        tracing::debug!("compositor: 丢弃 patch 更新 surface {surface_id} patch {patch_id}（epoch 不符或已 detached）");
        return false;
    }
    patch.bitmap_w = bitmap_w;
    patch.bitmap_h = bitmap_h;
    patch.rgba = rgba;
    true
}

/// patch 撤除（terminate/close）：转 detached 定格——位图保留参与贴片，Update 拒收。
fn store_patch_destroy(
    patches: &mut HashMap<u64, HashMap<u64, PatchState>>,
    surface_id: u64,
    navigation_epoch: u64,
    patch_id: u64,
) {
    if let Some(patch) = patches.get_mut(&surface_id).and_then(|entry| entry.get_mut(&patch_id))
        && patch.navigation_epoch == navigation_epoch
    {
        patch.detached = true;
    }
}

/// 帧提交推进 epoch 后清理跨代 patch（同 `cleanup_stale_animation_state` 语义；
/// 全部清空时移除 surface 条目）。
fn cleanup_stale_patches(patches: &mut HashMap<u64, HashMap<u64, PatchState>>, surface_id: u64, navigation_epoch: u64) {
    if let Some(entry) = patches.get_mut(&surface_id) {
        entry.retain(|_, patch| patch.navigation_epoch == navigation_epoch);
        if entry.is_empty() {
            patches.remove(&surface_id);
        }
    }
}

/// 把 patch 位图贴到 front buffer 注册矩形（src-over 混合——canvas 未绘制像素
/// 透明，不盖基线；矩形出视口裁剪；CSS px → 设备 px 按 scale，nearest 采样拉伸）。
fn paste_patches(
    patches: &HashMap<u64, PatchState>,
    front: &mut zero_render_foundation::surface::FrameBuffer,
    scale: f32,
    scroll_x: f32,
    scroll_y: f32,
) {
    for patch in patches.values() {
        if patch.rgba.is_empty() || patch.bitmap_w == 0 || patch.bitmap_h == 0 {
            continue;
        }
        let px = ((patch.x - scroll_x) * scale).round() as i64;
        let py = ((patch.y - scroll_y) * scale).round() as i64;
        // 注册矩形（CSS px × scale）即目标区域；位图拉伸填满（CSS 尺寸 ≠ 位图尺寸时
        // nearest 采样，同 canvas 元素位图缩放语义）。
        let pw = ((patch.width * scale).round() as i64).max(1);
        let ph = ((patch.height * scale).round() as i64).max(1);
        let (fw, fh) = (front.width as i64, front.height as i64);
        let x0 = px.max(0);
        let y0 = py.max(0);
        let x1 = (px + pw).min(fw);
        let y1 = (py + ph).min(fh);
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        for dy in y0..y1 {
            // 目标行 → 位图行（nearest；pw 已钳到位图宽时恒 1:1 直取）。
            let src_y = (((dy - py) * patch.bitmap_h as i64 / ph)
                .max(0)
                .min(patch.bitmap_h as i64 - 1)) as usize;
            let row_base = src_y * patch.bitmap_w as usize * 4;
            let dst_base = (dy as usize * fw as usize + x0 as usize) * 4;
            for dx in x0..x1 {
                let src_x = (((dx - px) * patch.bitmap_w as i64 / pw)
                    .max(0)
                    .min(patch.bitmap_w as i64 - 1)) as usize;
                let s = row_base + src_x * 4;
                let d = dst_base + (dx - x0) as usize * 4;
                let alpha = patch.rgba[s + 3] as u32;
                if alpha == 255 {
                    front.data[d..d + 4].copy_from_slice(&patch.rgba[s..s + 4]);
                } else if alpha > 0 {
                    // src-over：canvas 默认透明底，半透明像素与基线混合。
                    let inv = 255 - alpha;
                    for c in 0..3 {
                        front.data[d + c] =
                            ((patch.rgba[s + c] as u32 * alpha + front.data[d + c] as u32 * inv) / 255) as u8;
                    }
                }
            }
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
    // M2-S1c：surface 动画声明最新状态（renderer → browser → compositor，同一
    // FIFO 到达；最新生效全量替换，见 store_animation_state / 生命周期注释）。
    let mut animation_states: HashMap<u64, zero_protocol::animation::CompositorAnimationStateParams> = HashMap::new();
    // M3-S1c：worker canvas patch 贴片表（surface_id → patch_id → 态；独立于
    // SurfaceState——patch 可先于其首个帧到达）。
    let mut patch_surfaces: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
    let mut ui_surfaces: HashMap<u64, UiSurfaceState> = HashMap::new();
    let mut window_surface: Option<zero_protocol::CompositorWindowSurfaceInfo> = None;
    let base_fonts = Arc::new(load_compositor_fonts());
    let mut font_loader = base_fonts.clone();
    let mut font_namespace: Option<(u64, u64, u64)> = None;
    sandbox::apply_landlock_after_init();
    let mut glyph_cache = GlyphCache::new(1024);
    let mut render_thread = render_threading_enabled().then(|| RenderingThread::spawn(Arc::clone(&font_loader), 1024));
    // compositor-animation-continuity M1/M2：env 门控的已合成帧落盘（默认关闭），
    // 供真实窗口观察帧生产节奏（证据工具，失败自降级，见 frame_dump 模块文档）。
    let mut frame_dumper = frame_dump::FrameDumper::from_env();

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
                    front_cpu_resampled: false,
                    animations_active: false,
                    patches_dirty: false,
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
                // front 已被本次提交刷新（GPU 纹理同步渲染），Get 路径的 CPU 重采样
                // 过期标志随之解除。
                surface.front_cpu_resampled = false;
                // M2-S1c：epoch 变更清旧声明（同 surface.paint 生命周期；新 epoch 的
                // 声明由 renderer 状态消息随后重发）。Get 侧另有同 epoch 校验兜底。
                cleanup_stale_animation_state(&mut animation_states, surface_id, navigation_epoch);
                // M3-S1c：epoch 变更清旧 patch（同声明）；front 已被本次提交整体
                // 刷新——已有 patch 下次 Get 重贴（贴片不随帧提交，Get 期贴 front）。
                cleanup_stale_patches(&mut patch_surfaces, surface_id, navigation_epoch);
                surface.patches_dirty = true;
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
                if let Some(dumper) = frame_dumper.as_mut() {
                    dumper.dump(
                        surface_id,
                        navigation_epoch,
                        frame_id,
                        w,
                        h,
                        &surface.backing.front().data,
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
                // M2-S1c：本 surface 是否仍有未完成可合成动画（响应 animations_active，
                // 浏览器 S1d 据此保持 Get 轮询）。
                let mut response_animations_active = false;
                let (response_epoch, response_frame, width, height, rgba, shm_name, scroll_x, scroll_y, gpu_image) =
                    match surfaces.get_mut(&surface_id) {
                        Some(surface) => {
                            // M2-S1c：动画采样判定（flag 开 + 同 epoch 声明 + 未完成动画；
                            // 上次 Get 活跃也计入——全部动画在两次 Get 之间结束时，仍须
                            // 一次基线重光栅恢复 fill none 结束态，之后停止）。
                            let now_monotonic_ns = zero_protocol::monotonic_now_ns();
                            // RFC §2.4：flag 开启才采样（FR-3 flag 关零行为变化；renderer
                            // 导出侧同 flag 门控，正常无声明到达，此处兜底双保险）。
                            let animation_state = if zero_protocol::compositor_animations_enabled() {
                                animation_states.get(&surface_id).filter(|state| {
                                    state.navigation_epoch == surface.navigation_epoch && !state.declarations.is_empty()
                                })
                            } else {
                                None
                            };
                            let now_active = animation_state.is_some_and(|state| {
                                animation_overrides::has_active_animations(&state.declarations, now_monotonic_ns)
                            });
                            let anim_dirty = now_active || surface.animations_active;
                            // M2-S1e：采样判定 debug——now_active 翻 false 的时刻对齐
                            // renderer 声明清理发送（空集/换代），负载窗断续归因用。
                            tracing::debug!(
                                "compositor: Get surface {} 有声明={} now_active={} anim_dirty={} frame_id={}",
                                surface_id,
                                animation_state.is_some(),
                                now_active,
                                anim_dirty,
                                surface.frame_id
                            );
                            let scroll_dirty = zero_protocol::compositor_scroll_transform_enabled()
                                && (surface.rasterized_scroll_x != surface.scroll_x
                                    || surface.rasterized_scroll_y != surface.scroll_y);
                            if (scroll_dirty || anim_dirty)
                                && let Some(paint) = surface.paint.clone()
                            {
                                // RFC §2.4：先动画后滚动，均在同一 clone 上；采样只改
                                // clone，surface.paint 基线不动。
                                let mut viewport_paint = if anim_dirty && let Some(state) = animation_state {
                                    let mut sampled = paint;
                                    animation_overrides::apply(&mut sampled, &state.declarations, now_monotonic_ns);
                                    sampled
                                } else {
                                    paint
                                };
                                if scroll_dirty {
                                    let scale = rasterize::device_scale_factor(&viewport_paint);
                                    viewport_paint = scroll_transform::paint_for_viewport(
                                        &viewport_paint,
                                        surface.scroll_x / scale,
                                        surface.scroll_y / scale,
                                    );
                                }
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
                                // 重采样视区与旧 back buffer 没有可复用的坐标关系；全量重绘
                                //（动画/滚动 clone 均携带全视口脏区，走 is_full_viewport 全量）。
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
                                surface.front_cpu_resampled = true;
                                // M3-S1c：front 被重采样整体覆盖——patch 下次 Get 重贴。
                                surface.patches_dirty = true;
                                // M2-S1e：Get 采样路径同样落盘——动画重采样帧是
                                // DC-1「负载下帧连续」的直接证据（帧提交臂已有
                                // 同款 dump；序号命名见 frame_dump 模块文档）。
                                if let Some(dumper) = frame_dumper.as_mut() {
                                    dumper.dump(
                                        surface_id,
                                        surface.navigation_epoch,
                                        surface.frame_id,
                                        surface.backing.front().width,
                                        surface.backing.front().height,
                                        &surface.backing.front().data,
                                    );
                                }
                            }
                            surface.animations_active = now_active;
                            response_animations_active = now_active;
                            // M3-S1c：worker patch 贴片（Update 到达或 front 被重光栅/
                            // 新帧覆盖后重贴；detached patch 定格位图仍参与——呈现不
                            // 回退空白）。贴片不推进 frame_id（Get 拉 front 语义，完成
                            // 帧缓存按 (surface, epoch) 采纳）。
                            let has_live_patches = patch_surfaces
                                .get(&surface_id)
                                .is_some_and(|patches| !patches.is_empty());
                            if surface.patches_dirty && has_live_patches {
                                let scale = surface
                                    .paint
                                    .as_ref()
                                    .map(rasterize::device_scale_factor)
                                    .unwrap_or(1.0);
                                if let Some(patches) = patch_surfaces.get(&surface_id) {
                                    paste_patches(
                                        patches,
                                        surface.backing.front_mut(),
                                        scale,
                                        surface.rasterized_scroll_x,
                                        surface.rasterized_scroll_y,
                                    );
                                    // 贴片结果同落帧 dump（dc4 observe 的区域像素证据面）。
                                    if let Some(dumper) = frame_dumper.as_mut() {
                                        let front = surface.backing.front();
                                        dumper.dump(
                                            surface_id,
                                            surface.navigation_epoch,
                                            surface.frame_id,
                                            front.width,
                                            front.height,
                                            &front.data,
                                        );
                                    }
                                }
                            }
                            surface.patches_dirty = false;
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
                                    // M2-S1c：Get 路径 CPU 重采样（滚动/动画）后 front 比
                                    // GPU 纹理新（仍是提交时基线），同样不得冒充导出。
                                    && !surface.front_cpu_resampled
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
                        animations_active: response_animations_active,
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
            IpcMessageKind::CompositorAnimationState(params) => {
                // M2-S1c：动画声明状态（renderer → browser → compositor 中继）。
                // 静默消费、**不回执**——浏览器侧该命令 fire-and-forget、无响应
                // 匹配项，任何回执都会作为下一条命令的响应被判 id 错位致链路断连。
                store_animation_state(&mut animation_states, params);
            }
            IpcMessageKind::CompositorPatchRegister {
                surface_id,
                navigation_epoch,
                patch_id,
                x,
                y,
                width,
                height,
            } => {
                // M3-S1c：worker patch 注册（静默消费不回执——同声明中继语义）。
                store_patch_register(
                    &mut patch_surfaces,
                    surface_id,
                    navigation_epoch,
                    patch_id,
                    x,
                    y,
                    width,
                    height,
                );
            }
            IpcMessageKind::CompositorPatchUpdate {
                surface_id,
                navigation_epoch,
                patch_id,
                width,
                height,
                rgba,
            } => {
                if store_patch_update(
                    &mut patch_surfaces,
                    surface_id,
                    navigation_epoch,
                    patch_id,
                    width,
                    height,
                    rgba,
                ) && let Some(surface) = surfaces.get_mut(&surface_id)
                {
                    // 下次 Get 前贴片（front 须刷新）。
                    surface.patches_dirty = true;
                }
            }
            IpcMessageKind::CompositorPatchDestroy {
                surface_id,
                navigation_epoch,
                patch_id,
            } => {
                // terminate/close：patch 转 detached 定格（位图保留、Update 拒收）。
                store_patch_destroy(&mut patch_surfaces, surface_id, navigation_epoch, patch_id);
            }
            IpcMessageKind::ReleaseCompositorSurface { surface_id } => {
                // 挂起批随 surface 一起清理，避免 tab 关闭后像素滞留。
                pending_image_batches.remove(&surface_id);
                animation_states.remove(&surface_id);
                patch_surfaces.remove(&surface_id);
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
                        animations_active: false,
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
                        animations_active: false,
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
            front_cpu_resampled: false,
            animations_active: false,
            patches_dirty: false,
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

    fn animation_state(
        surface_id: u64,
        navigation_epoch: u64,
        generation: u64,
    ) -> zero_protocol::animation::CompositorAnimationStateParams {
        zero_protocol::animation::CompositorAnimationStateParams {
            surface_id,
            navigation_epoch,
            generation,
            declarations: Vec::new(),
            finished: Vec::new(),
        }
    }

    /// M2-S1c：声明最新生效——同 epoch 新代际替换、旧代际丢弃；跨 epoch 前进替换
    ///（导航换代的 generation 重置不构成回退）。
    #[test]
    fn animation_state_latest_wins_by_epoch_and_generation() {
        let mut states = HashMap::new();
        store_animation_state(&mut states, animation_state(7, 2, 5));
        // 同 epoch 旧代际：丢弃。
        store_animation_state(&mut states, animation_state(7, 2, 4));
        assert_eq!(states.get(&7).map(|s| s.generation), Some(5));
        // 同 epoch 新代际：替换。
        store_animation_state(&mut states, animation_state(7, 2, 6));
        assert_eq!(states.get(&7).map(|s| s.generation), Some(6));
        // 新 epoch（导航换代，generation 重置为 1）：替换。
        store_animation_state(&mut states, animation_state(7, 3, 1));
        let state = states.get(&7).expect("新 epoch 应替换");
        assert_eq!((state.navigation_epoch, state.generation), (3, 1));
        // 旧 epoch 迟到消息：丢弃。
        store_animation_state(&mut states, animation_state(7, 2, 99));
        assert_eq!(states.get(&7).map(|s| s.navigation_epoch), Some(3));
    }

    /// M2-S1c：帧提交推进 epoch 后跨代声明失效；同 epoch 帧重复提交不清理。
    #[test]
    fn animation_state_cleared_when_surface_epoch_advances() {
        let mut states = HashMap::new();
        store_animation_state(&mut states, animation_state(7, 2, 1));
        // 同 epoch 帧提交：声明保留。
        cleanup_stale_animation_state(&mut states, 7, 2);
        assert!(states.contains_key(&7));
        // epoch 推进：旧声明清除（新声明待 renderer 重发）。
        cleanup_stale_animation_state(&mut states, 7, 3);
        assert!(!states.contains_key(&7));
    }

    // ── M3-S1c：worker patch 存储 + 贴片 ──

    #[test]
    fn patch_register_latest_wins_and_stale_epoch_dropped() {
        let mut patches: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
        store_patch_register(&mut patches, 1, 2, 9, 10.0, 20.0, 100.0, 50.0);
        // stale epoch 注册丢弃（epoch 1 < 当前 2）。
        store_patch_register(&mut patches, 1, 1, 9, 0.0, 0.0, 1.0, 1.0);
        let patch = &patches[&1][&9];
        assert_eq!(patch.navigation_epoch, 2);
        assert_eq!((patch.x, patch.y, patch.width, patch.height), (10.0, 20.0, 100.0, 50.0));
        // 同 epoch 重注册覆盖矩形。
        store_patch_register(&mut patches, 1, 2, 9, 30.0, 40.0, 5.0, 5.0);
        assert_eq!(patches[&1][&9].x, 30.0);
    }

    #[test]
    fn patch_update_requires_registration_same_epoch_and_not_detached() {
        let mut patches: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
        // 未注册更新拒收。
        assert!(!store_patch_update(&mut patches, 1, 2, 9, 1, 1, vec![0, 0, 0, 255]));
        store_patch_register(&mut patches, 1, 2, 9, 0.0, 0.0, 10.0, 10.0);
        assert!(store_patch_update(
            &mut patches,
            1,
            2,
            9,
            2,
            2,
            vec![1, 2, 3, 255, 1, 2, 3, 255]
        ));
        assert_eq!(patches[&1][&9].rgba.len(), 8);
        // stale epoch 更新丢弃。
        assert!(!store_patch_update(&mut patches, 1, 1, 9, 1, 1, vec![9, 9, 9, 255]));
        assert_eq!(patches[&1][&9].rgba.len(), 8);
    }

    #[test]
    fn patch_destroy_detaches_and_stale_destroy_is_noop() {
        let mut patches: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
        store_patch_register(&mut patches, 1, 2, 9, 0.0, 0.0, 10.0, 10.0);
        store_patch_destroy(&mut patches, 1, 2, 9);
        assert!(patches[&1][&9].detached);
        // detached 后更新拒收（定格最后位图）。
        assert!(!store_patch_update(&mut patches, 1, 2, 9, 1, 1, vec![1, 1, 1, 255]));
        // 新 epoch 重注册复活（导航换代后同 patch_id 复用）。
        store_patch_register(&mut patches, 1, 3, 9, 0.0, 0.0, 10.0, 10.0);
        assert!(!patches[&1][&9].detached);
        // 旧 epoch destroy 不动新代 patch。
        store_patch_destroy(&mut patches, 1, 2, 9);
        assert!(!patches[&1][&9].detached);
    }

    #[test]
    fn patch_cleanup_removes_stale_epoch_and_empty_entries() {
        let mut patches: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
        store_patch_register(&mut patches, 1, 1, 9, 0.0, 0.0, 1.0, 1.0);
        store_patch_register(&mut patches, 1, 2, 9, 0.0, 0.0, 1.0, 1.0);
        store_patch_register(&mut patches, 1, 2, 10, 0.0, 0.0, 1.0, 1.0);
        cleanup_stale_patches(&mut patches, 1, 2);
        let entry = patches.get(&1).expect("epoch 2 patch 保留");
        assert!(entry.contains_key(&9) && entry.contains_key(&10));
        cleanup_stale_patches(&mut patches, 1, 3);
        assert!(!patches.contains_key(&1), "全清后条目移除");
    }

    /// 贴片像素面：矩形外基线不动；不透明直贴；半透明与基线 src-over 混合。
    #[test]
    fn paste_patches_blends_and_clips_to_rect() {
        let mut front = zero_render_foundation::surface::FrameBuffer::new(4, 4);
        front.data.fill(255);
        let mut patches: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
        store_patch_register(&mut patches, 1, 1, 9, 1.0, 1.0, 2.0, 2.0);
        // 位图 2×2：红（不透明）+ 绿（半透明）；未更新前贴片跳过（rgba 空）。
        paste_patches(patches.get(&1).unwrap(), &mut front, 1.0, 0.0, 0.0);
        assert_eq!(front.data[0], 255, "未更新位图不贴");
        // 位图 2×2：行 0 = 红/红（不透明），行 1 = 半透明绿/半透明绿。
        store_patch_update(
            &mut patches,
            1,
            1,
            9,
            2,
            2,
            vec![255, 0, 0, 255, 255, 0, 0, 255, 0, 255, 0, 128, 0, 255, 0, 128],
        );
        paste_patches(patches.get(&1).unwrap(), &mut front, 1.0, 0.0, 0.0);
        let px = |x: usize, y: usize, c: usize| front.data[(y * 4 + x) * 4 + c];
        assert_eq!((px(0, 0, 0), px(1, 0, 0)), (255, 255), "矩形外基线不动");
        assert_eq!((px(1, 1, 0), px(1, 1, 1), px(1, 1, 2)), (255, 0, 0), "不透明像素直贴");
        // 半透明绿(0,255,0,128) × 白基线：R 通道 0*128 + 255*127 ≈ 127（G 通道恒 255）。
        let r = px(2, 2, 0);
        assert!(r > 100 && r < 200, "半透明绿与白基线 src-over 混合，got R={r}");
        assert_eq!(px(2, 2, 1), 255, "G 通道绿+白恒 255");
        assert_eq!((px(2, 1, 0), px(2, 1, 1)), (255, 0), "行 0 不透明红直贴");
        assert_eq!(px(0, 3, 0), 255, "矩形外基线不动");
    }

    /// 贴片坐标面：位图拉伸填满注册矩形；CSS px → 设备 px 按 scale、滚动偏移相减。
    #[test]
    fn paste_patches_stretches_and_maps_viewport_space() {
        let mut front = zero_render_foundation::surface::FrameBuffer::new(8, 8);
        front.data.fill(255);
        let mut patches: HashMap<u64, HashMap<u64, PatchState>> = HashMap::new();
        // 文档 rect (2,0) 2×2，scroll_x=1，scale=2 → 设备 (2,0) 4×4；位图 1×1 红。
        store_patch_register(&mut patches, 1, 1, 9, 2.0, 0.0, 2.0, 2.0);
        store_patch_update(&mut patches, 1, 1, 9, 1, 1, vec![255, 0, 0, 255]);
        paste_patches(patches.get(&1).unwrap(), &mut front, 2.0, 1.0, 0.0);
        let px = |x: usize, y: usize, c: usize| front.data[(y * 8 + x) * 4 + c];
        assert_eq!((px(2, 0, 0), px(5, 3, 0)), (255, 255), "拉伸区全红");
        assert_eq!(px(1, 0, 0), 255, "区外基线");
        assert_eq!(px(6, 0, 0), 255, "区外基线");
        assert_eq!(px(2, 4, 0), 255, "区外基线");
    }
}
