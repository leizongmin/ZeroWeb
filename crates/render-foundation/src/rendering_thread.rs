//! 持久渲染工作线程（#3 渲染线程化 RFC S2）。
//!
//! 工作线程独占 [`GlyphCache`]，主线程通过 channel 提交图元 job 并同步等待结果。
//! headless/reftest 与 `ZW_RENDER_THREAD=0` 时走 [`crate::cpu::render_full_scene`] 直连。

use std::sync::Arc;
use std::sync::mpsc::{self, Sender, SyncSender};
use std::thread::{self, JoinHandle};

use crate::cpu::{render_full_scene, render_full_scene_region};
use crate::font::{FontLoader, GlyphCache};
use crate::geometry::Rect;
use crate::gpu::renderer::GlyphDraw;
use crate::primitive::{FillPrimitive, RenderPrimitives, RoundedRectPrimitive};
use crate::surface::FrameBuffer;

/// 是否启用渲染线程（默认开；`ZW_RENDER_THREAD=0` 关闭）。
pub fn render_threading_enabled() -> bool {
    zero_runtime_config::enabled_unless_zero("ZW_RENDER_THREAD")
}

/// reftest/headless 专用：仅 `ZW_RENDER_THREAD=1` 时走线程路径（确定性默认单线程）。
pub fn render_threading_enabled_for_tests() -> bool {
    zero_runtime_config::enabled_when_true("ZW_RENDER_THREAD")
}

struct RenderJob {
    width: u32,
    height: u32,
    scale_factor: f32,
    primitives: RenderPrimitives,
    region: Option<Rect>,
    ui_glyphs: Vec<GlyphDraw>,
    overlay_fills: Vec<FillPrimitive>,
    overlay_glyphs: Vec<GlyphDraw>,
    overlay_rounded_rects: Vec<RoundedRectPrimitive>,
    reply: SyncSender<FrameBuffer>,
}

/// 持久 CPU 光栅化工作线程。
pub struct RenderingThread {
    job_tx: Option<Sender<RenderJob>>,
    join: Option<JoinHandle<()>>,
}

impl RenderingThread {
    /// 启动工作线程（`FontLoader` 共享到 worker；`GlyphCache` 在 worker 内创建）。
    pub fn spawn(font_loader: Arc<FontLoader>, glyph_cache_capacity: usize) -> Self {
        let (job_tx, job_rx) = mpsc::channel::<RenderJob>();
        let join = thread::Builder::new()
            .name("zero-rendering-thread".into())
            .spawn(move || {
                let loader = font_loader;
                let mut glyph_cache = GlyphCache::new(glyph_cache_capacity);
                while let Ok(job) = job_rx.recv() {
                    let fb = if let Some(region) = job.region {
                        render_full_scene_region(
                            job.width,
                            job.height,
                            job.scale_factor,
                            &job.primitives,
                            &loader,
                            &mut glyph_cache,
                            None,
                            &job.ui_glyphs,
                            &job.overlay_fills,
                            &job.overlay_glyphs,
                            &job.overlay_rounded_rects,
                            Some(region),
                        )
                    } else {
                        render_full_scene(
                            job.width,
                            job.height,
                            job.scale_factor,
                            &job.primitives,
                            &loader,
                            &mut glyph_cache,
                            None,
                            &job.ui_glyphs,
                            &job.overlay_fills,
                            &job.overlay_glyphs,
                            &job.overlay_rounded_rects,
                        )
                    };
                    let _ = job.reply.send(fb);
                }
            })
            .expect("rendering thread spawn");

        Self {
            job_tx: Some(job_tx),
            join: Some(join),
        }
    }

    /// 同步提交光栅化 job（调用方阻塞至 worker 返回帧缓冲）。
    #[allow(clippy::too_many_arguments)]
    pub fn rasterize_sync(
        &self,
        width: u32,
        height: u32,
        scale_factor: f32,
        primitives: &RenderPrimitives,
        ui_glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        overlay_glyphs: &[GlyphDraw],
        overlay_rounded_rects: &[RoundedRectPrimitive],
        region: Option<Rect>,
    ) -> FrameBuffer {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        let job = RenderJob {
            width,
            height,
            scale_factor,
            primitives: primitives.clone(),
            region,
            ui_glyphs: ui_glyphs.to_vec(),
            overlay_fills: overlay_fills.to_vec(),
            overlay_glyphs: overlay_glyphs.to_vec(),
            overlay_rounded_rects: overlay_rounded_rects.to_vec(),
            reply: reply_tx,
        };
        self.job_tx
            .as_ref()
            .expect("RenderingThread 已关闭")
            .send(job)
            .expect("rendering thread job send");
        reply_rx.recv().expect("rendering thread reply")
    }
}

impl Drop for RenderingThread {
    fn drop(&mut self) {
        // 先关闭 job 发送端，worker 从 recv 退出后再 join（否则 Drop 死锁）。
        self.job_tx.take();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::color::Color;
    use crate::font::{FontLoader, GlyphCache};
    use crate::geometry::Rect;
    use crate::primitive::{FillPrimitive, RenderPrimitives};

    use super::*;

    /// S2：持久 RenderingThread 与主线程直连光栅化逐像素一致。
    #[test]
    fn rendering_thread_matches_direct_full_scene() {
        let loader = Arc::new(FontLoader::new());
        let mut glyph_cache = GlyphCache::new(64);
        let mut primitives = RenderPrimitives::new();
        primitives.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 48.0, 32.0),
            color: Color::rgb(40, 80, 120),
        });

        let direct = render_full_scene(
            48,
            32,
            1.0,
            &primitives,
            &loader,
            &mut glyph_cache,
            None,
            &[],
            &[],
            &[],
            &[],
        );

        let rt = RenderingThread::spawn(Arc::clone(&loader), 64);
        let threaded = rt.rasterize_sync(48, 32, 1.0, &primitives, &[], &[], &[], &[], None);
        assert_eq!(direct.data, threaded.data);
    }

    /// Drop 须能在 worker 空闲时干净退出（回归：先 join 后关 channel 会死锁）。
    #[test]
    fn rendering_thread_drop_does_not_deadlock() {
        let loader = Arc::new(FontLoader::new());
        let rt = RenderingThread::spawn(loader, 64);
        drop(rt);
    }
}
