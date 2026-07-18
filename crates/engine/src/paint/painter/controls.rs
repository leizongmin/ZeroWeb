//! 表单/替换控件的 UA 外观绘制（progress/meter value 填充条等）。
//!
//! 与 [`super::text::Painter::paint_input_value`]（R1660）/ `paint_img_element` 同属「元素特化
//! paint」。独立成文件因 `text.rs` 已超 2000 行（CLAUDE.md §5 文件大小控制）。

use zero_dom::{Document, NodeKind};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_style_system::ComputedStyle;

impl super::Painter {
    /// 绘制 `<progress>`/`<meter>` 的 value 填充条（R1671 paint 半，≡ R1660 `paint_input_value`）。
    ///
    /// progress/meter 是 inline-block 替换控件（R1670 sizing 半已给 track 盒 + 固有尺寸 +
    /// track bg/border）。chromium 把 value/max 比例填成一条彩色条覆盖在 track 上：
    /// - progress：value/max 比例，[`PROGRESS_VALUE`]（chrome-127 oracle 实测 (0,117,255)）。
    /// - meter：value/(max-min) 比例，颜色按 value vs low/high/optimum 三区域算法（HTML §4.10.16）。
    ///
    /// **调用时序**：须在 `paint_text` 之后（见 `mod.rs` 调用点）——bar 覆盖 fallback 文本。
    /// ZW 无「replaced 元素抑制子节点 layout」机制（select/option 同 latent gap），fallback 文本
    /// 仍会被 layout + paint；bar 后绘覆盖之，近似 chromium 不显示 fallback。indeterminate
    /// progress（无 value 属性）不绘条（chromium 绘条纹动画，超出本 slice scope）。
    pub(crate) fn paint_progress_meter_value(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        _style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };
        let elem = match doc.get(node_id) {
            Some(n) => match &n.kind {
                NodeKind::Element(e)
                    if e.local_name().eq_ignore_ascii_case("progress")
                        || e.local_name().eq_ignore_ascii_case("meter") =>
                {
                    e
                }
                _ => return,
            },
            None => return,
        };
        let is_progress = elem.local_name().eq_ignore_ascii_case("progress");

        // 解析数值属性（HTML 规范：缺失/非法回落默认值）。value 缺失 = indeterminate → 不绘条。
        let value = match elem
            .get_attribute("value")
            .and_then(|s| s.trim().parse::<f64>().ok())
            .filter(|v| v.is_finite())
        {
            Some(v) => v,
            None => return,
        };
        let parse = |name: &str| -> Option<f64> {
            elem.get_attribute(name)
                .and_then(|s| s.trim().parse::<f64>().ok())
                .filter(|v| v.is_finite())
        };

        // track content-box 几何（abs + border + padding）。R1670 已给固有尺寸 → content_w/h 非零。
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        let track_w = box_node.content_width;
        let track_h = box_node.content_height;
        if track_w <= 0.0 || track_h <= 0.0 {
            return;
        }

        // 计算 value 比例 + 颜色。
        let frac: f64 = if is_progress {
            let max = parse("max").filter(|m| *m > 0.0).unwrap_or(1.0);
            (value / max).clamp(0.0, 1.0)
        } else {
            let min = parse("min").unwrap_or(0.0);
            let max = parse("max").unwrap_or(1.0);
            if max <= min {
                return;
            }
            ((value - min) / (max - min)).clamp(0.0, 1.0)
        };
        let color = if is_progress {
            PROGRESS_VALUE
        } else {
            let min = parse("min").unwrap_or(0.0);
            let max = parse("max").unwrap_or(1.0);
            let low = parse("low").unwrap_or(min);
            let high = parse("high").unwrap_or(max);
            let optimum = parse("optimum").unwrap_or(min + (max - min) / 2.0);
            meter_color(value, low, high, optimum)
        };

        let bar_w = (frac * track_w as f64) as f32;
        if bar_w <= 0.5 {
            return;
        }
        self.primitives
            .add_fill(Rect::new(content_x, content_y, bar_w, track_h), color);
    }
}

/// chromium 默认 progress value 填充色（accent 蓝，chrome-127 oracle 实测 (0,117,255)）。
const PROGRESS_VALUE: Color = Color {
    r: 0,
    g: 117,
    b: 255,
    a: 255,
};

/// chromium meter 三区域颜色（green 为 chrome-127 oracle 实测 (16,124,16)；yellow/red 取
/// chromium UA 近似——精确系统色须 accent-color 支持，forward）。
const METER_GREEN: Color = Color {
    r: 16,
    g: 124,
    b: 16,
    a: 255,
};
const METER_YELLOW: Color = Color {
    r: 204,
    g: 153,
    b: 0,
    a: 255,
};
const METER_RED: Color = Color {
    r: 204,
    g: 0,
    b: 0,
    a: 255,
};

/// HTML `<meter>` 颜色算法（HTML 规范 §4.10.16.7）。
///
/// 把 [min, max] 按 low/high 切成三段：low-region（x<low）/ mid-region（[low,high]）/
/// high-region（x>high）。optimum 所在段为「目标段」，value 所在段与目标段的距离决定颜色：
/// 同段 → green、相邻段 → yellow、相隔一段（low↔high）→ red。low>high 时整段视为 mid。
fn meter_color(value: f64, low: f64, high: f64, optimum: f64) -> Color {
    let region = |x: f64| -> i8 {
        if low > high {
            1
        } else if x < low {
            0
        } else if x > high {
            2
        } else {
            1
        }
    };
    let g = region(optimum);
    let c = region(value);
    match (g - c).abs() {
        0 => METER_GREEN,
        1 => METER_YELLOW,
        _ => METER_RED,
    }
}
