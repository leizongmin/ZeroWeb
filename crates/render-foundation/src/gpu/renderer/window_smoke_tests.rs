//! 窗口模式 GPU 渲染冒烟测试（P1-3）。
//!
//! `new_for_window` 此前全工作区零测试执行——surface format 选择、swapchain
//! 配置、present 生命周期都是「从未跑过一行」的代码（基线：
//! docs/learnings/bugs/cpu-gpu-path-divergence.md P1-3）。本测试在存在显示
//! 服务器时创建真实 winit 窗口，走完整窗口模式链路：
//! new_for_window → configure_surface → render_full_scene_gpu → present → drop。
//! 无 DISPLAY/WAYLAND_DISPLAY 环境时跳过（CI 无显示服务器时不失败）。

use super::*;
use serial_test::serial;
use std::sync::{Arc, OnceLock};

use crate::primitive::{FillPrimitive, FilterKind, FilterPrimitive, RenderPrimitives};

/// 模块级共享窗口（winit EventLoop 每进程只能创建一次，EventLoop 泄漏保持存活）。
static SHARED_WINDOW: OnceLock<Option<Arc<winit::window::Window>>> = OnceLock::new();

/// 获取共享窗口；无显示服务器时返回 None（测试跳过）。
fn shared_window() -> Option<Arc<winit::window::Window>> {
    SHARED_WINDOW
        .get_or_init(|| {
            if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
                return None;
            }
            // 测试线程不是主线程：winit 默认禁止非主线程建 EventLoop（panic），
            // 用 X11 backend 的 any_thread 绕过（Wayland-only 环境构建失败 → None 跳过）。
            let mut builder = winit::event_loop::EventLoop::builder();
            #[cfg(target_os = "linux")]
            {
                use winit::platform::x11::EventLoopBuilderExtX11;
                builder.with_x11().with_any_thread(true);
            }
            let event_loop = match builder.build() {
                Ok(el) => el,
                Err(e) => {
                    eprintln!("shared_window: EventLoop 构建失败: {e}");
                    return None;
                }
            };
            let attrs = winit::window::WindowAttributes::default();
            // create_window 在 0.30.13 标 deprecated（替代 ActiveEventLoop::create_window
            // 仅在 run_app 回调内可用，冒烟测试不进事件循环，故豁免）
            #[allow(deprecated)]
            match event_loop.create_window(attrs) {
                Ok(w) => {
                    // EventLoop 泄漏保持存活（进程生命周期内有效；winit 窗口依赖其存在）
                    std::mem::forget(event_loop);
                    Some(Arc::new(w))
                }
                Err(e) => {
                    eprintln!("shared_window: 窗口创建失败: {e}");
                    None
                }
            }
        })
        .clone()
}

/// 窗口模式全生命周期冒烟：建窗口 → new_for_window → format 选择 → configure →
/// 渲染一帧 → present → drop。
#[serial]
#[test]
fn window_mode_lifecycle_smoke() {
    let Some(window) = shared_window() else {
        eprintln!("window_mode_lifecycle_smoke: 无显示服务器，跳过（CI 环境）");
        return;
    };
    let mut renderer = match GpuRenderer::new_for_window(window) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("window_mode_lifecycle_smoke: 窗口 GPU 初始化失败（无 GPU 驱动/无表面支持），跳过: {e}");
            return;
        }
    };
    // 窗口模式 format 选择：优先非 sRGB（Bgra8Unorm/Rgba8Unorm）——与 headless
    // 固定 Rgba8UnormSrgb 不是同一条代码路径（P1-3 之前零覆盖）。
    let format = renderer.surface_format();
    assert!(
        matches!(
            format,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
        ),
        "窗口模式应选择非 sRGB 格式，得到 {format:?}"
    );

    let (w, h) = (320u32, 240u32);
    renderer.configure_surface(w, h);

    // 不透明场景（GPU 支持子集）→ 应返回 true 并 present
    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, w as f32, h as f32),
        color: Color::rgba(0, 0, 255, 255),
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered = renderer.render_full_scene_gpu(
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
        1.0,
    );
    assert!(rendered, "窗口模式不透明场景应渲染成功");
    // 窗口模式无 readback（swapchain 帧不可回读）
    assert!(renderer.read_pixels().is_none(), "窗口模式不应有 headless 回读");
    // drop：swapchain/device 释放路径
    drop(renderer);
}

/// 窗口模式 + filter → D/R3279 后应渲染成功（离屏纹理后处理 + blit 回 surface）。
#[serial]
#[test]
fn window_mode_filters_render_successfully() {
    let Some(window) = shared_window() else {
        eprintln!("window_mode_filters_return_false: 无显示服务器，跳过（CI 环境）");
        return;
    };
    let Ok(mut renderer) = GpuRenderer::new_for_window(window) else {
        eprintln!("window_mode_filters_return_false: 窗口 GPU 初始化失败，跳过");
        return;
    };
    renderer.configure_surface(64, 64);

    let mut primitives = RenderPrimitives::default();
    primitives.fills.push(FillPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        color: Color::rgba(255, 0, 0, 255),
    });
    primitives.filters.push(FilterPrimitive {
        rect: Rect::new(0.0, 0.0, 32.0, 32.0),
        filters: vec![FilterKind::Opacity(0.5)],
    });
    let font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(64);
    let rendered = renderer.render_full_scene_gpu(
        &primitives,
        &font_loader,
        &mut glyph_cache,
        None,
        &[],
        &[],
        &[],
        &[],
        1.0,
    );
    assert!(rendered, "窗口模式 + filter 应渲染成功（D/R3279 离屏后处理）");
}
