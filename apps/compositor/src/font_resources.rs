//! One cache invalidation boundary for frame, scroll and surface-release paths.

use std::sync::Arc;
use zero_render_foundation::{
    font::{FontLoader, GlyphCache},
    gpu::renderer::GpuRenderer,
    rendering_thread::{RenderingThread, render_threading_enabled},
};

pub(crate) fn activate(
    next: Option<(u64, u64, u64)>,
    loader: &Arc<FontLoader>,
    current: &mut Option<(u64, u64, u64)>,
    active_loader: &mut Arc<FontLoader>,
    glyph_cache: &mut GlyphCache,
    gpu: &mut Option<GpuRenderer>,
    worker: &mut Option<RenderingThread>,
) {
    if *current == next {
        return;
    }
    *current = next;
    *active_loader = loader.clone();
    // Numeric font IDs can be reused by a different surface/document.
    glyph_cache.clear();
    if let Some(gpu) = gpu.as_mut() {
        gpu.clear_glyph_atlas();
    }
    *worker = render_threading_enabled().then(|| RenderingThread::spawn(loader.clone(), 1024));
}
