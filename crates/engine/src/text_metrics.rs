//! 可选的 paint 阶段字符宽度测量回调。
//!
//! 浏览器在渲染帧前通过 thread-local 注入真实字体 metrics；
//! 未注入时回退到 layout 的 `estimate_char_width`。

use std::sync::OnceLock;

use zero_layout_engine::inline::estimate_char_width;

static CHAR_MEASURE: OnceLock<fn(char, f32, bool) -> f32> = OnceLock::new();

/// 注册全局字符宽度测量函数（浏览器启动时调用一次）。
pub fn set_char_measure_fn(f: fn(char, f32, bool) -> f32) {
    let _ = CHAR_MEASURE.set(f);
}

/// Paint 阶段测量单个字符 advance；Ahem 字体固定为 1em 方框宽。
pub fn measure_char_for_paint(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        return font_size;
    }
    CHAR_MEASURE
        .get()
        .copied()
        .map(|measure| measure(ch, font_size, is_ahem))
        .unwrap_or_else(|| estimate_char_width(ch, font_size, is_ahem))
}

/// 与 layout IFC 一致的字符宽度估计（无真实字体回调时使用）。
pub fn layout_estimate_char_width(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    estimate_char_width(ch, font_size, is_ahem)
}
