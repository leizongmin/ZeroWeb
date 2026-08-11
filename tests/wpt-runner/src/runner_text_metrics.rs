//! Runner paint 阶段真实字体 metrics 桥接（镜像 `apps/browser/src/text_metrics.rs`）。
//!
//! **R1765 发现**：wpt-runner 此前**未注册** `set_char_measure_fn`，导致 reftest /
//! product-smoke 的 paint 路径回退到 `estimate_char_width`（0.55×fs 启发式）而非
//! fontdue 真实 advance。实测探针：ZW 渲染 'm' = 0.584×fs（≈estimate）vs chromium
//! 0.797×fs vs fontdue Liberation 0.833×fs → 「font-wall」部分是**测量 artifact**
//! （runner 用 estimate paint，browser 用 fontdue）。本模块把 browser 的
//! `with_measure_ctx` 模式复刻到 runner，使测量与真实 browser 一致。
use std::cell::Cell;

use zero_engine::layout_estimate_char_width;
use zero_render_foundation::font::{OpenTypeFeature, ShapedGlyph, TextDirection, loader::FontLoader};

thread_local! {
    static MEASURE_CTX: Cell<Option<(*const FontLoader, u32)>> = const { Cell::new(None) };
}

/// 全局 paint 测量回调（注册到 `zero-engine`）。读 thread-local `MEASURE_CTX`，
/// 有则用 `FontLoader::measure_advance`（真实 fontdue advance），无则回退 estimate。
pub fn measure_char(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        return font_size;
    }
    MEASURE_CTX.with(|cell| {
        if let Some((loader, font_id)) = cell.get() {
            // SAFETY: 指针仅在 `with_measure_ctx` 闭包执行期间有效（runner 单线程渲染）。
            unsafe { (*loader).measure_advance(font_id, ch, font_size) }
        } else {
            layout_estimate_char_width(ch, font_size, false)
        }
    })
}

/// 在当前 WPT 字体上下文中按指定 face 整形文本。
pub fn shape_text(
    font_id: u32,
    text: &str,
    font_size: f32,
    direction: TextDirection,
    features: &[OpenTypeFeature],
) -> Option<Vec<ShapedGlyph>> {
    MEASURE_CTX.with(|cell| {
        let (loader, _) = cell.get()?;
        // SAFETY: 指针仅在 `with_measure_ctx` 闭包执行期间有效。
        let loader = unsafe { &*loader };
        loader.shape_text_cached_with_features(font_id, text, font_size, direction, features)
    })
}

/// 在闭包执行期间启用真实字体测量（镜像 browser `with_measure_ctx`）。
pub fn with_measure_ctx<R>(font_loader: &FontLoader, font_id: u32, f: impl FnOnce() -> R) -> R {
    MEASURE_CTX.with(|cell| {
        cell.set(Some((font_loader as *const FontLoader, font_id)));
        let result = f();
        cell.set(None);
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_text_requires_context_and_uses_requested_face() {
        assert!(shape_text(0, "AV", 16.0, TextDirection::LeftToRight, &[]).is_none());

        const LATO_TTF: &[u8] = include_bytes!("../fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("load bundled Lato");
        let glyphs = with_measure_ctx(&loader, font_id, || {
            shape_text(font_id, "AV", 16.0, TextDirection::LeftToRight, &[]).expect("shape in active font context")
        });

        assert_eq!(glyphs.len(), 2);
        assert!(glyphs.iter().all(|glyph| glyph.glyph_id > 0));
        assert!(glyphs.iter().all(|glyph| glyph.advance_x > 0.0));

        let rtl = with_measure_ctx(&loader, font_id, || {
            shape_text(font_id, "ABC", 16.0, TextDirection::RightToLeft, &[]).expect("RTL shape")
        });
        assert_eq!(rtl.iter().map(|glyph| glyph.cluster).collect::<Vec<_>>(), vec![2, 1, 0]);

        let ligatures = with_measure_ctx(&loader, font_id, || {
            let enabled = shape_text(
                font_id,
                "fi",
                16.0,
                TextDirection::LeftToRight,
                &[OpenTypeFeature::new(*b"liga", 1)],
            )
            .expect("liga enabled shape");
            let disabled = shape_text(
                font_id,
                "fi",
                16.0,
                TextDirection::LeftToRight,
                &[OpenTypeFeature::new(*b"liga", 0)],
            )
            .expect("liga disabled shape");
            (enabled, disabled)
        });
        assert_eq!(ligatures.0.len(), 1);
        assert_eq!(ligatures.1.len(), 2);
    }
}
