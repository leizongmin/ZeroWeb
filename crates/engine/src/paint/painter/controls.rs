//! 表单/替换控件的 UA 外观绘制（progress/meter value 填充条等）。
//!
//! 与 [`super::text::Painter::paint_input_value`]（R1660）/ `paint_img_element` 同属「元素特化
//! paint」。独立成文件因 `text.rs` 已超 2000 行（CLAUDE.md §5 文件大小控制）。

use zero_css_parser::values::{LengthValue, ListStyleTypeValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::GlyphPrimitive;
use zero_style_system::{AppearanceComputedValue, ComputedStyle, DirectionValue};

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
        style: &ComputedStyle,
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

        // css-ui-4 §appearance-switching：value 填充条属原生 widget 外观——作者
        // `appearance: none` 关闭（appearance 初值 auto，cascade 无声明即 auto；
        // [`Self::effective_appearance`] 的原生映射恒非 None，故此处 effective 判定
        // 与原始 computed 一致）。track chrome 的对应门在
        // `widget_track_chrome_suppressed`。driving：R1670
        // meter-appearance-none-rendering 族 mismatch 四页。
        if matches!(
            super::effective_appearance(style, box_node, Some(doc)),
            AppearanceComputedValue::None
        ) {
            return;
        }

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

    /// 绘制 `<input type=range>` 的 datalist tick 刻度（R1687，≡ R1671 progress/meter value
    /// 条 paint 谱系）。
    ///
    /// HTML range state：控件带 `list` 属性且引用到 `<datalist>` 时，UA 须在滑轨下渲染
    /// tick 刻度行——每个 step 对齐的 option value 一枚。chrome-127 行为（driving 五页
    /// +八页 match 域）：
    /// - disabled option 不产生刻度（range-tick-marks-04 match 域）；
    /// - step 不对齐的 option value 不产生刻度（range-tick-marks-05 + repaint 八页，全
    ///   `step=3 min=-5`，仅 {-2,4} 对齐）；
    /// - fraction = (value-min)/(max-min) 截到 [0,1]（range-tick-marks-03 mismatch：min/max
    ///   不同 → 刻度像素位置可分）；
    /// - `direction: rtl`（`dir` 属性经 presentational hint 映射）水平镜像（range-tick-marks-02
    ///   match 域：test 侧 dir=rtl vs ref 侧 `transform: scaleX(-1)` 翻转，两侧须同镜像才一致）。
    ///
    /// list→datalist 每次 paint 时按 id 解析（repaint 族八页经 JS 改 list/id/option 后
    /// 重发布，静态快照不行）。刻度 = 2px 宽竖线绘于 content-box 下半带（mismatch 判据
    /// 超过 0.01% 即 48px/480000：4 刻度 × 2px × 8px = 64px 过阈）。作者 `appearance: none`
    /// 关闭原生 widget（≡ s313 meter 门，css-ui-4 §appearance-switching）。规范：
    /// https://html.spec.whatwg.org/multipage/input.html#range-state-(type=range)
    pub(crate) fn paint_range_ticks(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let Some(node_id) = box_node.node_id else {
            return;
        };
        let Some(node) = doc.get(node_id) else {
            return;
        };
        let NodeKind::Element(elem) = &node.kind else {
            return;
        };
        if !elem.local_name().eq_ignore_ascii_case("input")
            || !elem
                .get_attribute("type")
                .unwrap_or_default()
                .eq_ignore_ascii_case("range")
        {
            return;
        }
        if matches!(
            super::effective_appearance(style, box_node, Some(doc)),
            AppearanceComputedValue::None
        ) {
            return;
        }
        let Some(list_value) = elem
            .get_attribute("list")
            .map(|s| s.trim().to_string())
            .filter(|v| !v.is_empty())
        else {
            return;
        };
        // list→datalist 逐 paint 解析（repaint 族 JS 改 list/id 后无重挂载静态快照失效）。
        let Some(datalist_id) = doc.get_element_by_id(&list_value) else {
            return;
        };
        let Some(dl) = doc.get(datalist_id) else {
            return;
        };
        if !matches!(&dl.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("datalist")) {
            return;
        }

        // 数值属性（HTML：缺失/非法回落 min=0 / max=100 / step=1；step ≤ 0 无效回落 1）。
        let num_attr = |name: &str, default: f64| -> f64 {
            elem.get_attribute(name)
                .and_then(|s| s.trim().parse::<f64>().ok())
                .filter(|v| v.is_finite())
                .unwrap_or(default)
        };
        let min = num_attr("min", 0.0);
        let max = num_attr("max", 100.0);
        let step = {
            let s = num_attr("step", 1.0);
            if s > 0.0 { s } else { 1.0 }
        };
        if max <= min {
            return;
        }

        // 收集 step 对齐 option 刻度位置。option 语义值 = `value` 属性（否则文本内容，
        // HTML option value 模型）；disabled 排除；step 不对齐排除（浮点容差按商取整距）。
        let mut fractions: Vec<f32> = Vec::new();
        for child in doc.child_nodes(datalist_id) {
            let Some(cn) = doc.get(child) else { continue };
            let NodeKind::Element(opt) = &cn.kind else { continue };
            if !opt.local_name().eq_ignore_ascii_case("option") {
                continue;
            }
            if opt.get_attribute("disabled").is_some() {
                continue;
            }
            let raw = opt
                .get_attribute("value")
                .unwrap_or_else(|| doc.text_content(child).unwrap_or_default());
            let Some(v) = raw.trim().parse::<f64>().ok().filter(|v| v.is_finite()) else {
                continue;
            };
            let q = (v - min) / step;
            if (q - q.round()).abs() > 1e-6 {
                continue;
            }
            fractions.push((((v - min) / (max - min)) as f32).clamp(0.0, 1.0));
        }
        if fractions.is_empty() {
            return;
        }

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        let track_w = box_node.content_width;
        let track_h = box_node.content_height;
        if track_w <= 0.0 || track_h <= 0.0 {
            return;
        }
        let tick_h = track_h * 0.5;
        let tick_y = content_y + track_h - tick_h;
        let rtl = matches!(style.direction, DirectionValue::Rtl);
        for frac in fractions {
            let f = if rtl { 1.0 - frac } else { frac };
            // 2px 宽竖线居中于刻度位（镜像与 transform scaleX(-1) 翻转的像素域一致性：
            // [c+d-1, c+d+1] ↔ 翻转 [2c-(c+d+1), 2c-(c+d-1)]，同一浮点矩形）。
            let x = content_x + f * track_w - 1.0;
            self.primitives.add_fill(Rect::new(x, tick_y, 2.0, tick_h), RANGE_TICK);
        }
    }

    /// 绘制 `<select>` 的 selected option 标签文本（R1679，≡ R1660 `paint_input_value` 谱系）。
    ///
    /// ZW 无 select popup shadow tree，option/optgroup 经 UA `display:none` 抑制（R1679）不生成
    /// 盒 → select 按钮内无文本（`has_direct_paintable_text` 对 select 返回 false，paint_text 跳过），
    /// 须显式绘 selected option 标签。selected option = 首个带 `selected` 属性的 option（含
    /// optgroup 内），否则 DOM 首个 option（HTML 默认选中首项）。标签 = option `label` 属性（非空）
    /// 否则文本内容。左对齐于内容盒，baseline = content_y + font_size（≡ paint_input_value 几何）。
    /// kill-switch `ZW_SELECT_SUPPRESS_OPTIONS=0` 关闭（与 option 抑制 + select 宽同 gate，default-on）。
    pub(crate) fn paint_select_value(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        if std::env::var("ZW_SELECT_SUPPRESS_OPTIONS").as_deref() == Ok("0") {
            return;
        }
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };
        match doc.get(node_id) {
            Some(n) => match &n.kind {
                NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("select") => {}
                _ => return,
            },
            None => return,
        }

        let label = match select_selected_option_label(doc, node_id) {
            Some(l) if !l.is_empty() => l,
            _ => return,
        };

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 {
            return;
        }
        let color = super::super::color::color_value_to_render(&style.color);
        let default_font_id = self.resolve_style_font_id(&style.font_family, style).0;
        let variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&variations);

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        let baseline_y = content_y + font_size;

        let mut char_x = content_x;
        for ch in label.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: baseline_y,
                font_size,
                color,
                glyph_id: ch as u32,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            });
            char_x += self.measure_char_cached(default_font_id.0, ch, font_size, false);
        }

        // R1680：下拉箭头（小灰色向下三角），填补 R1679 select 固有宽预留的 chrome 空间。
        // chromium 在 select 右侧绘向下箭头指示可展开；固定中性灰（非 style.color，≡ chromium
        // 系统箭头色），垂直居中于内容盒。select 太窄（<16px）时跳过避免溢出。
        let cw = box_node.content_width;
        if cw >= 16.0 {
            let ch = if box_node.content_height > 0.0 {
                box_node.content_height
            } else {
                font_size
            };
            let cy = content_y + ch * 0.5;
            let ax = content_x + cw - 10.0;
            self.primitives
                .add_path_fill(vec![ax - 4.0, cy - 3.0, ax + 4.0, cy - 3.0, ax, cy + 3.0], SELECT_ARROW);
        }
    }

    /// 绘制 `<summary>` 的 disclosure 标记（R1686，≡ R1680 select 箭头 paint 谱系）。
    ///
    /// chromium 在 summary 文本前绘一个小三角指示 disclosure 状态：闭合态 ▶（右指）、
    /// 开启态 ▼（下指）。标记用 currentColor（≡ chrome-127 oracle 实测黑字页 (0,0,0)），
    /// 绘在 summary 的 UA `padding-left` 区（text 让位）。无 `padding-left`（作者覆盖为 0）时
    /// 跳过避免压字。父非 `<details>` 或无 open 属性判定时按闭合态处理（≡ R1684 is_closed_details）。
    pub(crate) fn paint_summary_marker(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let Some(node_id) = box_node.node_id else {
            return;
        };
        let Some(node) = doc.get(node_id) else {
            return;
        };
        if !matches!(&node.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("summary")) {
            return;
        }
        // css-lists-3 §list-style-type（https://www.w3.org/TR/css-lists-3/#list-style-type-property）：
        // `list-style-type: none` 抑制标记，与 ul/ol marker 臂同判据（text_list.rs 的
        // ListStyleTypeValue::None 抑制先例）。站点对 summary 声明 none（如 github
        // Primer `summary{list-style:none}`）并以 ::after 自绘 caret 时，无条件原生
        // disclosure 三角即双画（zinc B-1 heal 臂定谳）——以绘制源真理收口。
        if style.list_style_type == ListStyleTypeValue::None {
            return;
        }
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => 16.0,
        };
        if font_size <= 0.0 {
            return;
        }
        // 父 <details> 的 open 态：open 属性存在 = 开启（▼），否则闭合（▶）。
        let is_open =
            doc.parent_node(node_id).and_then(|pid| doc.get(pid)).is_some_and(
                |p| matches!(&p.kind, NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("details")),
            ) && doc
                .parent_node(node_id)
                .is_some_and(|pid| doc.get_attribute(pid, "open").is_some());

        // 标记绘在 padding-left 区（chromium 标记 ~0.4em 宽 + gap，text 让位 ≈1.2em）。
        // padding_left 不足（作者覆盖为 0）时跳过避免压字。
        let pad_l = box_node.padding_left;
        if pad_l < 6.0 {
            return;
        }
        let ms_w = font_size * 0.4;
        let ms_h = font_size * 0.5;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        // 标记垂直居中于首行 x-height（≈ content_y + font_size*0.45）。
        let cy = content_y + font_size * 0.45;
        // 标记贴 padding 区起点（chromium 标记在 content 起 x=8，text 让位到 x≈26）。
        let mx = abs_x + box_node.border_left + 2.0;
        let color = super::super::color::color_value_to_render(&style.color);
        let verts = if is_open {
            // ▼ 下指三角：上边两角 + 下尖。
            vec![
                mx,
                cy - ms_h * 0.5,
                mx + ms_w,
                cy - ms_h * 0.5,
                mx + ms_w * 0.5,
                cy + ms_h * 0.5,
            ]
        } else {
            // ▶ 右指三角：左边两角 + 右尖。
            vec![mx, cy - ms_h * 0.5, mx, cy + ms_h * 0.5, mx + ms_w, cy]
        };
        self.primitives.add_path_fill(verts, color);
    }
}

/// R1679：返回 `<select>` 的 selected option 标签文本。
///
/// 选中项 = 首个带 `selected` 属性的 option（含 optgroup 内，按 DOM 顺序）；无则首个 option
///（HTML：select 单选默认选中首个 option）。标签 = option `label` 属性（非空）否则文本内容。
fn select_selected_option_label(doc: &Document, select: NodeId) -> Option<String> {
    let mut first: Option<String> = None;
    for child in doc.child_nodes(select) {
        let Some(node) = doc.get(child) else { continue };
        let NodeKind::Element(e) = &node.kind else { continue };
        let name = e.local_name();
        if name.eq_ignore_ascii_case("option") {
            let label = option_label_text(doc, child);
            if doc.get_attribute(child, "selected").is_some() {
                return Some(label);
            }
            first.get_or_insert(label);
        } else if name.eq_ignore_ascii_case("optgroup") {
            for gc in doc.child_nodes(child) {
                let Some(gn) = doc.get(gc) else { continue };
                let NodeKind::Element(ge) = &gn.kind else { continue };
                if !ge.local_name().eq_ignore_ascii_case("option") {
                    continue;
                }
                let label = option_label_text(doc, gc);
                if doc.get_attribute(gc, "selected").is_some() {
                    return Some(label);
                }
                first.get_or_insert(label);
            }
        }
    }
    first
}

/// R1679：返回 `<option>` 标签文本（`label` 属性非空优先，否则 text content）。
fn option_label_text(doc: &Document, id: NodeId) -> String {
    if let Some(l) = doc.get_attribute(id, "label") {
        let trimmed = l.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    doc.text_content(id).unwrap_or_default().trim().to_string()
}

/// chromium 默认 progress value 填充色（accent 蓝，chrome-127 oracle 实测 (0,117,255)）。
const PROGRESS_VALUE: Color = Color {
    r: 0,
    g: 117,
    b: 255,
    a: 255,
};

/// chromium range datalist tick 刻度色（中性灰，chrome-127 近似——与 UA 轨道边框同灰阶）。
const RANGE_TICK: Color = Color {
    r: 118,
    g: 118,
    b: 118,
    a: 255,
};

/// chromium `<select>` 下拉箭头色（中性灰，chrome-127 oracle 实测 (90,90,90) 近似）。
const SELECT_ARROW: Color = Color {
    r: 90,
    g: 90,
    b: 90,
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
