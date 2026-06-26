use std::cell::Cell;

use zero_engine::layout_estimate_char_width;
use zero_render_foundation::font::loader::FontLoader;

thread_local! {
    static MEASURE_CTX: Cell<Option<(*const FontLoader, u32)>> = const { Cell::new(None) };
}

pub fn measure_char(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        return font_size;
    }
    MEASURE_CTX.with(|cell| {
        if let Some((loader, font_id)) = cell.get() {
            unsafe { (*loader).measure_advance(font_id, ch, font_size) }
        } else {
            layout_estimate_char_width(ch, font_size, false)
        }
    })
}

pub fn with_measure_ctx_opt<R>(font_loader: &FontLoader, font_id: Option<u32>, f: impl FnOnce() -> R) -> R {
    match font_id {
        Some(id) => MEASURE_CTX.with(|cell| {
            cell.set(Some((font_loader as *const FontLoader, id)));
            let result = f();
            cell.set(None);
            result
        }),
        None => f(),
    }
}
