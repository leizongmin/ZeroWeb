//! 浏览器 paint 阶段的真实字体 metrics 桥接。
//!
//! `zero-engine` 在绘制文本时调用全局 `measure_char`；本模块通过 thread-local
//! 在页面加载/重绘期间注入 `FontLoader::measure_advance`。

use std::cell::Cell;

use zero_engine::layout_estimate_char_width;
use zero_render_foundation::font::loader::FontLoader;

thread_local! {
    static MEASURE_CTX: Cell<Option<(*const FontLoader, u32)>> = const { Cell::new(None) };
    /// 跨帧/跨 painter 的 advance 测量缓存（key = (font_id, char, size_bits)）。
    ///
    /// painter 的 measure_cache 随每帧新建清空——CJK 文本页每帧对每个
    /// (字符, 字号) 重复测量（FreeType load_glyph ~µs 级）。字体集进程内
    /// 稳定（共享字体后 font_id 不变），font_id 进 key 后结果确定；thread_local
    /// 免锁（paint/worker 线程各自缓存，同线程反复测量命中）。
    static MEASURE_CACHE: std::cell::RefCell<std::collections::HashMap<(u32, u32, u32), f32>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// 全局 paint 测量回调，由 `BrowserApp::new` 注册到 `zero-engine`。
pub fn measure_char(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        return font_size;
    }
    MEASURE_CTX.with(|cell| {
        if let Some((loader, font_id)) = cell.get() {
            let key = (font_id, ch as u32, font_size.to_bits());
            MEASURE_CACHE.with(|cache| {
                if let Some(&w) = cache.borrow().get(&key) {
                    return w;
                }
                // SAFETY: 指针仅在 `with_measure_ctx` 闭包执行期间有效。
                let w = unsafe { (*loader).measure_advance(font_id, ch, font_size) };
                cache.borrow_mut().insert(key, w);
                w
            })
        } else {
            layout_estimate_char_width(ch, font_size, false)
        }
    })
}

/// 在闭包执行期间启用真实字体测量。
pub fn with_measure_ctx<R>(font_loader: &FontLoader, font_id: u32, f: impl FnOnce() -> R) -> R {
    MEASURE_CTX.with(|cell| {
        cell.set(Some((font_loader as *const FontLoader, font_id)));
        let result = f();
        cell.set(None);
        result
    })
}

/// 若已加载系统字体则在闭包内启用测量，否则直接执行。
pub fn with_measure_ctx_opt<R>(font_loader: &FontLoader, font_id: Option<u32>, f: impl FnOnce() -> R) -> R {
    match font_id {
        Some(id) => with_measure_ctx(font_loader, id, f),
        None => f(),
    }
}
