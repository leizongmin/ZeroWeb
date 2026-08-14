//! Canvas 2D 类型定义 — 枚举、结构体、辅助函数。

use std::sync::{Arc, Mutex};

use zero_render_foundation::color::Color;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::path::Path2D;

/// 字体粗细。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontWeight {
    /// 正常。
    Normal,
    /// 粗体。
    Bold,
}

/// 字体样式。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontStyle {
    /// 正常。
    Normal,
    /// 斜体。
    Italic,
}

/// 字体描述符。
#[derive(Debug, Clone)]
pub struct FontDescriptor {
    /// 字体族。
    pub family: String,
    /// 字体大小。
    pub size: f32,
    /// 字体粗细。
    pub weight: FontWeight,
    /// 字体样式。
    pub style: FontStyle,
    /// R34xx：font-variant small-caps 标记（getter 重建 'small-caps'——font.parse.complex）。
    pub small_caps: bool,
    /// R34xx：数值 weight（100-900——getter 重建 '300' 等——font.parse.weight）。
    pub weight_value: Option<u16>,
    /// R34xx：letterSpacing 原始 CSS 长度串（spec CanvasTextDrawingStyles——每字符簇附加
    /// 间距；em/% 等相对单位**随字号重解析**——2d.text.drawing.style.letterSpacing.change.font）。
    pub letter_spacing: String,
    /// R34xx：wordSpacing 原始 CSS 长度串（每个词后附加间距）。
    pub word_spacing: String,
}

impl Default for FontDescriptor {
    fn default() -> Self {
        Self {
            family: "sans-serif".to_string(),
            size: 10.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            small_caps: false,
            weight_value: None,
            letter_spacing: "0px".to_string(),
            word_spacing: "0px".to_string(),
        }
    }
}

impl FontDescriptor {
    /// 解析 CSS font 简写字符串（HTML Canvas `ctx.font`，https://drafts.csswg.org/css-fonts/#font-shorthand）。
    ///
    /// 格式：`[style] [variant] [weight] <size>[/<line-height>] <family>`——style/variant/weight 为
    /// 可选关键字（序无关，size 之前），size 与 family 必需。R3304：canvas 文本状态面暴露 `ctx.font`
    /// 需把页面设的 CSS font 串解析为 `FontDescriptor`。
    ///
    /// **诚实范围**（headless canvas 无真实字体管线）：① 仅解析 size 的 px/em/rem/pt（em/rem 同字号
    /// 上下文近似作 px，pt 按 96/72）；size 关键字（`small`/`large`）→ 默认 size（无法精确映射）。
    /// ② variant（`small-caps`）/line-height/stretch 解析后丢弃（canvas FontDescriptor 不建模这些）。
    /// ③ weight 数字（100-900）/`bolder`/`lighter` 仅区分 Bold（≥600 或 `bold`/`bolder`）vs Normal。
    /// ④ 解析失败返 `None`（real browser 忽略非法 font 串保持原值，本解析器同语义，调用方决定回退）。
    /// family 保留原串（含逗号多族 / 引号），交字体解析流后续精确解析。
    pub fn parse_css(s: &str) -> Option<Self> {
        Self::parse_css_with_current(s, 10.0)
    }

    /// R34xx：带当前字号解析（em/rem 相对当前 font size——'1em' 在默认 10px 下 → 10px）。
    pub fn parse_css_with_current(s: &str, current_size: f32) -> Option<Self> {
        // 简单分词：按空白切，但 family（size 之后全部）整体保留（含逗号/引号）。
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        if tokens.len() < 2 {
            return None; // 至少 size + family
        }

        let mut style = FontStyle::Normal;
        let mut weight = FontWeight::Normal;
        let mut weight_value = None;
        let mut small_caps = false;

        // size 之前的关键字（style/variant/weight），序无关——遇 size token 停。
        let mut i = 0;
        while i < tokens.len() {
            let t = tokens[i];
            let lower = t.to_ascii_lowercase();
            if lower == "italic" || lower == "oblique" {
                style = FontStyle::Italic;
                i += 1;
                continue;
            }
            if lower == "normal" {
                // normal 可表 style/variant/weight/stretch 任一 → 忽略（默认即 normal）
                i += 1;
                continue;
            }
            if lower == "bold" || lower == "bolder" {
                weight = FontWeight::Bold;
                i += 1;
                continue;
            }
            if lower == "lighter" {
                weight = FontWeight::Normal;
                i += 1;
                continue;
            }
            if lower == "small-caps" {
                small_caps = true;
                i += 1;
                continue;
            }
            // 数字 weight（100-900）
            if let Ok(n) = lower.parse::<u32>()
                && (100..=900).contains(&n)
            {
                weight = if n >= 600 { FontWeight::Bold } else { FontWeight::Normal };
                weight_value = Some(n as u16);
                i += 1;
                continue;
            }
            // 否则视作 size 起点（数字 + 单位 / 关键字）→ 停
            break;
        }
        if i >= tokens.len() {
            return None; // 缺 size
        }

        // size：解析 tokens[i]，可能含 `/line-height`。
        let size_tok = tokens[i];
        let size_part = size_tok.split('/').next().unwrap_or(size_tok);
        // size 关键字（`medium`/`small`/`large`...）→ parse_font_size 返 None（canvas headless 无关键字→px
        // 映射表）。real browser 这些关键字合法；此处保守：若 size 未解析出，返 None（调用方保持原 font）。
        let size = parse_font_size_with_current(size_part, current_size)?;
        let family_start = i + 1; // family 从 size 之后开始
        if family_start >= tokens.len() {
            return None; // 缺 family
        }

        // family：size 之后全部原串保留（重新从 trimmed 切，避分词丢逗号/引号内空白）。
        // 找到 family 起点 = 第 (family_start) 个空白分隔 token 在 trimmed 中的偏移。
        let family = extract_family(trimmed, family_start);
        if family.is_empty() {
            return None;
        }
        // R34xx：family 段标识符校验——每段须为引号串或合法 CSS 标识符（无 { } / 等
        // 非法字符——2d.text.font.parse.invalid 的 '{bogus}' 应整体拒绝保持原 font）。
        let mut family_valid = true;
        let mut in_quote = false;
        let mut fam_chars = family.chars().peekable();
        while let Some(ch) = fam_chars.next() {
            match ch {
                '\\' if in_quote => {
                    // 引号内反斜杠转义（\" 是字面引号非闭合）。
                    fam_chars.next();
                }
                '"' => in_quote = !in_quote,
                ',' if !in_quote => {}
                '{' | '}' | '/' | '\\' | '(' | ')' | ';' if !in_quote => {
                    family_valid = false;
                    break;
                }
                _ => {}
            }
        }
        // CSS-wide 关键字作族名 → 无效（'10px initial' 等——font.parse.invalid）。
        let fam_lower = family.trim_matches('"').to_ascii_lowercase();
        if matches!(
            fam_lower.as_str(),
            "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "default"
        ) {
            return None;
        }
        if !family_valid || in_quote {
            return None;
        }

        Some(Self {
            family,
            size,
            weight,
            style,
            small_caps,
            weight_value,
            letter_spacing: "0px".to_string(),
            word_spacing: "0px".to_string(),
        })
    }
}

/// 解析 font-size 单位→px。支持 px/em/rem/pt（em/rem 无上下文近似作绝对 px）。
/// 失败（无单位 / 未知单位 / 非数字）返 None。
/// R34xx：em 相对当前字号（canvas ctx.font——'1em' 默认 10px → 10px）；
/// rem 恒相对 root（近似 16px）。
fn parse_font_size_with_current(s: &str, current_size: f32) -> Option<f32> {
    let lower = s.to_ascii_lowercase();
    let (num_str, mul) = lower
        .strip_suffix("px")
        .map(|st| (st, 1.0))
        .or_else(|| lower.strip_suffix("pt").map(|st| (st, 96.0 / 72.0)))
        .or_else(|| lower.strip_suffix("rem").map(|st| (st, 16.0))) // root em 近似 16px
        .or_else(|| lower.strip_suffix("em").map(|st| (st, current_size)))?; // em 相对当前字号
    let n = num_str.trim().parse::<f32>().ok()?;
    if !n.is_finite() || n <= 0.0 {
        return None;
    }
    Some(n * mul)
}

/// R34xx：解析 CSS 长度到 px（letterSpacing/wordSpacing——spec CanvasTextDrawingStyles）。
/// 支持 px/em/rem/ex/pt/pc/cm/mm/in（em/rem/ex 按字号近似；% 按字号百分比；无单位 → px）。
/// 非有限/负值 → None（调用方保持旧值，spec 非法忽略）。
pub fn parse_length_px(s: &str, font_size: f32) -> Option<f32> {
    let lower = s.trim().to_ascii_lowercase();
    let (num_str, mul) = lower
        .strip_suffix("px")
        .map(|st| (st, 1.0))
        .or_else(|| lower.strip_suffix("em").map(|st| (st, font_size)))
        .or_else(|| lower.strip_suffix("rem").map(|st| (st, 16.0)))
        .or_else(|| lower.strip_suffix("ex").map(|st| (st, font_size * 0.5)))
        .or_else(|| lower.strip_suffix("ch").map(|st| (st, font_size * 0.5)))
        .or_else(|| lower.strip_suffix("ic").map(|st| (st, font_size)))
        .or_else(|| lower.strip_suffix("cap").map(|st| (st, font_size * 0.5)))
        .or_else(|| lower.strip_suffix("pt").map(|st| (st, 96.0 / 72.0)))
        .or_else(|| lower.strip_suffix("pc").map(|st| (st, 16.0)))
        .or_else(|| lower.strip_suffix("cm").map(|st| (st, 96.0 / 2.54)))
        .or_else(|| lower.strip_suffix("mm").map(|st| (st, 96.0 / 25.4)))
        .or_else(|| lower.strip_suffix("in").map(|st| (st, 96.0)))
        .or_else(|| lower.strip_suffix('%').map(|st| (st, font_size / 100.0)))
        .or(Some((lower.as_str(), 1.0)))?;
    let n = num_str.trim().parse::<f32>().ok()?;
    if !n.is_finite() {
        return None;
    }
    Some(n * mul)
}

/// 从 CSS font 串中提取第 `token_index` 个空白分隔 token 起的子串作为 family（保留逗号/引号）。
fn extract_family(s: &str, token_index: usize) -> String {
    // 扫描字节，找第 token_index 个非空白 token 的起点，返回该起点到串尾的 trim 子串。
    let bytes = s.as_bytes();
    let mut idx = 0usize; // 已跳过的 token 数
    let mut in_token = false;
    for (i, &b) in bytes.iter().enumerate() {
        let is_ws = b == b' ' || b == b'\t' || b == b'\n' || b == b'\r';
        if !is_ws {
            if !in_token {
                // token 起点
                if idx == token_index {
                    return s[i..].trim().to_string();
                }
                idx += 1;
                in_token = true;
            }
        } else {
            in_token = false;
        }
    }
    String::new()
}

/// 2D 仿射变换矩阵。
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    /// 矩阵元素 a (scale X / cos rotate)。
    pub a: f32,
    /// 矩阵元素 b (skew Y / sin rotate)。
    pub b: f32,
    /// 矩阵元素 c (skew X / -sin rotate)。
    pub c: f32,
    /// 矩阵元素 d (scale Y / cos rotate)。
    pub d: f32,
    /// 平移 X。
    pub e: f32,
    /// 平移 Y。
    pub f: f32,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

impl Transform2D {
    /// 单位矩阵。
    pub fn identity() -> Self {
        Self::default()
    }

    /// 平移变换。
    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            e: tx,
            f: ty,
            ..Self::default()
        }
    }

    /// 缩放变换。
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            d: sy,
            ..Self::default()
        }
    }

    /// 旋转变换（弧度）。
    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// 矩阵乘法：self * other。
    pub fn multiply(&self, other: &Transform2D) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// 变换点。
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }

    /// 逆变换（2×3 仿射矩阵求逆）。det≈0（奇异）时返恒等（调用方容错）。
    pub fn inverse(&self) -> Transform2D {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < f32::EPSILON {
            return Transform2D::identity();
        }
        let ia = self.d / det;
        let ib = -self.b / det;
        let ic = -self.c / det;
        let id = self.a / det;
        Transform2D {
            a: ia,
            b: ib,
            c: ic,
            d: id,
            e: -(ia * self.e + ic * self.f),
            f: -(ib * self.e + id * self.f),
        }
    }
}

/// 文本对齐。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    /// 起始对齐。
    Start,
    /// 末尾对齐。
    End,
    /// 左对齐。
    Left,
    /// 右对齐。
    Right,
    /// 居中对齐。
    Center,
}

/// 文本基线。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBaseline {
    /// 顶部。
    Top,
    /// 中部。
    Middle,
    /// 字母基线。
    Alphabetic,
    /// 底部。
    Bottom,
    /// 悬挂基线（R34xx：2d.text.draw.baseline.hanging——0.5em 近似）。
    Hanging,
    /// 表意基线（R34xx：2d.text.draw.baseline.ideographic——em 盒底 = +descent）。
    Ideographic,
}

/// 文本方向。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextDirection {
    /// 从左到右。
    Ltr,
    /// 从右到左。
    Rtl,
    /// 继承（默认）。
    #[default]
    Inherit,
}

/// 图像平滑质量（HTML Canvas `imageSmoothingQuality`，
/// https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-imagesmoothingquality）。
///
/// R3305：drawImage 缩放的重采样质量。canvas crate 无真实重采样后端（drawImage 逐像素采样，
/// 无低/中/高差异化算法），故仅存储 + 反射（headless 简化）；真实重采样质量须接渲染流图像管线
/// 作 follow-up。即便如此，完整属性表面使依赖库（图像编辑/游戏像素艺术）feature-detect 与读值不抛错。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ImageSmoothingQuality {
    /// 低质量。
    Low,
    /// 中等质量。
    Medium,
    /// 高质量（默认，real browser 默认 low；headless 取 high 为保守近似，调用方读值不依赖默认）。
    #[default]
    High,
}

/// 线段连接样式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineJoin {
    /// 默认：尖角连接。
    #[default]
    Miter,
    /// 圆角连接。
    Round,
    /// 斜角连接。
    Bevel,
}

/// 线段端点样式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LineCap {
    /// 默认：平头端点。
    #[default]
    Butt,
    /// 圆头端点。
    Round,
    /// 方头端点（延伸半个线宽）。
    Square,
}

/// 合成操作模式 — 控制 Canvas 绘制时新图元与已有内容的混合方式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CompositeOperation {
    /// 默认：新图元绘制在已有内容之上。
    #[default]
    SourceOver,
    /// 新图元只绘制在透明区域。
    DestinationOver,
    /// 清除新图元与已有内容重叠的区域。
    DestinationOut,
    /// 新图元与已有内容重叠的部分保留已有内容。
    DestinationAtop,
    /// 新图元与已有内容的重叠区域显示已有内容。
    DestinationIn,
    /// 新图元与已有内容重叠区域显示新图元，其余清除。
    SourceIn,
    /// R34xx：新图元只绘制在透明区域（Porter-Duff source-out——此前缺失，composite
    /// 光栅的 source-out 语义从未实现）。
    SourceOut,
    /// 新图元与已有内容重叠区域显示新图元。
    SourceAtop,
    /// 新图元和已有内容取较亮值。
    Lighter,
    /// 新图元复制到输出，忽略已有内容。
    Copy,
    /// 新图元和已有内容取异或。
    Xor,
    /// R34xx：清除画布（Porter-Duff clear：Fa=0, Fb=0——2d.composite.operation.clear）。
    Clear,
    /// 新图元乘以已有内容（变暗）。
    Multiply,
    /// 新图元与已有内容取屏幕混合（变亮）。
    Screen,
    /// 新图元与已有内容叠加混合。
    Overlay,
    /// 新图层变暗模式。
    Darken,
    /// 新图层变亮模式。
    Lighten,
    /// 新图层颜色减淡。
    ColorDodge,
    /// 新图层颜色加深。
    ColorBurn,
    /// 新图层强光模式。
    HardLight,
    /// 新图层柔光模式。
    SoftLight,
    /// 新图层差值模式。
    Difference,
    /// 新图层排除模式。
    Exclusion,
    /// 新图层色相模式。
    Hue,
    /// 新图层饱和度模式。
    Saturation,
    /// 新图层颜色模式。
    Color,
    /// 新图层亮度模式。
    Luminosity,
}

/// 渐变停止点。
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// 偏移量 [0.0, 1.0]。
    pub offset: f32,
    /// 颜色。
    pub color: Color,
}

/// 线性渐变 — 从起点到终点的颜色过渡。
#[derive(Debug, Clone)]
pub struct LinearGradient {
    /// 起点 X。
    pub x0: f32,
    /// 起点 Y。
    pub y0: f32,
    /// 终点 X。
    pub x1: f32,
    /// 终点 Y。
    pub y1: f32,
    /// 颜色停止点列表。
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// 创建线性渐变。
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self {
            x0,
            y0,
            x1,
            y1,
            stops: Vec::new(),
        }
    }

    /// 添加颜色停止点。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop { offset, color });
    }

    /// 在指定偏移量处采样颜色（线性插值）。
    pub fn sample_color(&self, offset: f32) -> Color {
        sample_gradient_stops(&self.stops, offset)
    }
}

/// 径向渐变 — 从内圆到外圆的颜色过渡。
#[derive(Debug, Clone)]
pub struct RadialGradient {
    /// 内圆圆心 X。
    pub x0: f32,
    /// 内圆圆心 Y。
    pub y0: f32,
    /// 内圆半径。
    pub r0: f32,
    /// 外圆圆心 X。
    pub x1: f32,
    /// 外圆圆心 Y。
    pub y1: f32,
    /// 外圆半径。
    pub r1: f32,
    /// 颜色停止点列表。
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    /// 创建径向渐变。
    pub fn new(x0: f32, y0: f32, r0: f32, x1: f32, y1: f32, r1: f32) -> Self {
        Self {
            x0,
            y0,
            r0,
            x1,
            y1,
            r1,
            stops: Vec::new(),
        }
    }

    /// 添加颜色停止点。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop { offset, color });
    }

    /// 在指定偏移量处采样颜色（线性插值）。
    pub fn sample_color(&self, offset: f32) -> Color {
        sample_gradient_stops(&self.stops, offset)
    }
}

/// 锥形渐变 — 围绕中心点按角度过渡颜色。
#[derive(Debug, Clone)]
pub struct ConicGradient {
    /// 起始角度（弧度）。
    pub start_angle: f32,
    /// 中心 X 坐标。
    pub cx: f32,
    /// 中心 Y 坐标。
    pub cy: f32,
    /// 颜色停止点列表。
    pub stops: Vec<GradientStop>,
}

impl ConicGradient {
    /// 创建锥形渐变。
    pub fn new(start_angle: f32, cx: f32, cy: f32) -> Self {
        Self {
            start_angle,
            cx,
            cy,
            stops: Vec::new(),
        }
    }

    /// 添加颜色停止点。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        self.stops.push(GradientStop { offset, color });
    }

    /// 在指定偏移量处采样颜色（线性插值）。
    pub fn sample_color(&self, offset: f32) -> Color {
        sample_gradient_stops(&self.stops, offset)
    }
}

/// 图案重复模式。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PatternRepetition {
    /// 水平和垂直方向都重复。
    #[default]
    Repeat,
    /// 只在水平方向重复。
    RepeatX,
    /// 只在垂直方向重复。
    RepeatY,
    /// 不重复。
    NoRepeat,
}

/// 图案 — 从 ImageData 创建的平铺图案。
#[derive(Debug, Clone)]
pub struct CanvasPattern {
    /// 图案源图像数据。
    pub image_data: ImageData,
    /// 重复模式。
    pub repetition: PatternRepetition,
    /// R34xx：pattern 变换（CanvasPattern.setTransform——恒等生效；非 identity 的
    /// 采样变换为已知缺口）。
    pub transform: Transform2D,
    /// R34xx：平铺锚定变换 = fill 时 CTM 的逆（spec：pattern 网格锚定在 fill 坐标空间——
    /// 2d.pattern.paint.repeat.coord1 的 translate(-128,-78) 后 device (1,1) 映射到 fill
    /// (129,79) → 绿象限）。transform_gradient 的 Pattern 分支填充。
    pub tile_transform: Transform2D,
}

impl CanvasPattern {
    /// 创建图案。
    pub fn new(image_data: ImageData, repetition: PatternRepetition) -> Self {
        Self {
            image_data,
            repetition,
            transform: Transform2D::identity(),
            tile_transform: Transform2D::identity(),
        }
    }
}

/// Canvas 填充/描边样式。
#[derive(Debug, Clone)]
pub enum CanvasStyle {
    /// 纯色。
    Color(Color),
    /// 线性渐变。
    LinearGradient(LinearGradient),
    /// 径向渐变。
    RadialGradient(RadialGradient),
    /// 锥形渐变。
    ConicGradient(ConicGradient),
    /// 图案。
    Pattern(CanvasPattern),
}

impl CanvasStyle {
    /// 默认样式：不透明黑色。
    pub fn default_black() -> Self {
        CanvasStyle::Color(Color::BLACK)
    }

    /// 解析为有效颜色。
    ///
    /// 对于 Color 变体直接使用；
    /// 对于渐变变体在指定偏移量处采样近似颜色；
    /// 对于 Pattern 返回黑色作为回退。
    pub fn resolve_color(&self) -> Color {
        match self {
            CanvasStyle::Color(c) => *c,
            CanvasStyle::LinearGradient(g) => g.sample_color(0.5),
            CanvasStyle::RadialGradient(g) => g.sample_color(0.5),
            CanvasStyle::ConicGradient(g) => g.sample_color(0.0),
            CanvasStyle::Pattern(_) => Color::BLACK,
        }
    }

    /// 为渐变变体添加颜色停止点（spec `CanvasGradient.addColorStop`）。
    /// Color/Pattern 变体为 no-op（非渐变样式无停止点概念）。
    pub fn add_color_stop(&mut self, offset: f32, color: Color) {
        match self {
            CanvasStyle::LinearGradient(g) => g.add_color_stop(offset, color),
            CanvasStyle::RadialGradient(g) => g.add_color_stop(offset, color),
            CanvasStyle::ConicGradient(g) => g.add_color_stop(offset, color),
            _ => {}
        }
    }

    /// 判断是否为渐变样式（光栅化路径分流用）。
    pub fn is_gradient(&self) -> bool {
        matches!(
            self,
            CanvasStyle::LinearGradient(_) | CanvasStyle::RadialGradient(_) | CanvasStyle::ConicGradient(_)
        )
    }

    /// 判断是否需要逐像素光栅化（渐变 + 图案；纯色走 flat 快路径）。fill/stroke 分流用（R3085 扩 Pattern）。
    pub fn is_per_pixel_style(&self) -> bool {
        !matches!(self, CanvasStyle::Color(_))
    }

    /// 在设备空间某点 (x, y) 采样样式颜色（spec canvas 渐变光栅化的核心）。
    ///
    /// - Color：直接返回。
    /// - LinearGradient：将点投影到渐变线 (x0,y0)→(x1,y1) 得参数 t∈[0,1]，再线性插值停止点。
    ///   https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-createlineargradient
    /// - RadialGradient：以内心 (x0,y0,r0) 为基准，按距离归一化到外圆 (x1,y1,r1) 得 t。
    /// - ConicGradient：以中心 (cx,cy) 计相对 start_angle 的角度，归一化到 [0,1]。
    /// - Pattern：按重复模式平铺采样源 ImageData 像素（R3085，`sample_pattern_pixel`）；超出区域透明。
    ///
    /// 偏移量超出 [0,1] 由 `sample_gradient_stops` 钳制到首/末停止点颜色（spec：渐变在停止点之外延伸为端点色）。
    pub fn sample_at(&self, x: f32, y: f32) -> Color {
        match self {
            CanvasStyle::Color(c) => *c,
            CanvasStyle::LinearGradient(g) => {
                let dx = g.x1 - g.x0;
                let dy = g.y1 - g.y0;
                let len2 = dx * dx + dy * dy;
                // R34xx：零长度渐变线（undefined direction）→ 渐变不画（透明——
                // 2d.gradient.interpolate.zerosize.* 期望保持底）。
                if len2 < f32::EPSILON {
                    return Color::TRANSPARENT;
                }
                let t = ((x - g.x0) * dx + (y - g.y0) * dy) / len2;
                sample_gradient_stops(&g.stops, t)
            }
            CanvasStyle::RadialGradient(g) => {
                // R34xx：radial 全几何解（见 `radial_gradient_t`）——cone/相交/相切/退化
                // 圆族全部经二次方程精确解；未覆盖点（cone 背后/判别式负）返透明不画。
                match radial_gradient_t(g, x, y) {
                    Some(t) => sample_gradient_stops(&g.stops, t),
                    None => Color::TRANSPARENT,
                }
            }
            CanvasStyle::ConicGradient(g) => {
                let mut ang = (y - g.cy).atan2(x - g.cx) - g.start_angle;
                // 归一化到 [0, 2π)
                while ang < 0.0 {
                    ang += std::f32::consts::TAU;
                }
                while ang >= std::f32::consts::TAU {
                    ang -= std::f32::consts::TAU;
                }
                sample_gradient_stops(&g.stops, ang / std::f32::consts::TAU)
            }
            CanvasStyle::Pattern(p) => sample_pattern_pixel(p, x, y),
        }
    }
}

/// 径向渐变采样参数 t（R34xx，spec：canvas `createRadialGradient` 光栅化的完整几何）。
///
/// 规范模型：t ∈ [0,1] 的每条 iso-line 是「中心 = lerp(C0,C1,t)、半径 = lerp(r0,r1,t)」
/// 的圆（https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-createradialgradient）。
/// 对点 P 解关于 t 的二次方程 `(x - c(t))² + y² = r(t)²`（归一化到 C0 原点、|C1−C0|=1）：
///
/// ```text
/// a = 1 - (r1-r0)²,  b = -2(x + r0(r1-r0)),  c = x² + y² - r0²
/// ```
///
/// 分支语义（WPT 2d.gradient.radial.cone.* / touch* / equal / bottom / front 驱动）：
/// - 判别式 < 0：点不在任何 iso 圆上 → 渐变未覆盖该点（透明不画——cone.shape1/behind 背后区）。
/// - 实根中取**有效根**（半径 r(t) ≥ 0；t 越过半径穿零点 r0/(r0−r1) 的根是镜像伪根——
///   cone.behind 的 t=−0.44 假根）的**较大值**（Skia 同款取法；cone.top/bottom 的 t>1 端）。
/// - 同心退化（|C1−C0|≈0）回落距离归一化（旧行为，inside*/outside* 等）；r0==r1 时
///   渐变退化为单个圆（测度零）→ 不画（radial.equal）。
fn radial_gradient_t(g: &RadialGradient, x: f32, y: f32) -> Option<f32> {
    let (dx, dy) = (g.x1 - g.x0, g.y1 - g.y0);
    let d2 = dx * dx + dy * dy;
    if d2 < f32::EPSILON {
        // 同心：距离归一化（旧行为）。r0 == r1 → 单个圆（测度零）→ 不画。
        if (g.r1 - g.r0).abs() < f32::EPSILON {
            return None;
        }
        let dist = ((x - g.x0).powi(2) + (y - g.y0).powi(2)).sqrt();
        return Some((dist - g.r0) / (g.r1 - g.r0));
    }
    let d = d2.sqrt();
    // 归一化：P = C0 + px·(C1−C0) + py·⊥(C1−C0)。R34xx：内部用 f64——判别式在切线
    // 边界（cone.shape1 的 (50,25) 恰在 iso 圆上，disc 数学上 = 0）f32 舍入成 −ε 误判 None。
    let (px, py) = (
        ((x - g.x0) as f64 * dx as f64 + (y - g.y0) as f64 * dy as f64) / d2 as f64,
        ((x - g.x0) as f64 * (-dy as f64) + (y - g.y0) as f64 * dx as f64) / d2 as f64,
    );
    let (r0, r1) = (g.r0 as f64 / d as f64, g.r1 as f64 / d as f64);
    let delta = r1 - r0;
    let a = 1.0 - delta * delta;
    let b = -2.0 * (px + r0 * delta);
    let c = px * px + py * py - r0 * r0;
    // |r1−r0| ≈ d（内切抛物线退化）：一次方程。
    let roots: [f64; 2] = if a.abs() < 1e-12 {
        let t = if b.abs() < 1e-12 { f64::NAN } else { -c / b };
        [t, f64::NAN]
    } else {
        let disc = b * b - 4.0 * a * c;
        // R34xx：相对容差——切线边界（cone.shape1 的 (50,25) 恰在 iso 圆上）判别式数学上
        // = 0，f64 表示误差（0.4/1.2/1.8 非精确）产生 ~1e-16 负值；真实判别式负（锥外）量级
        // 远大于此（如 shape1 (1,1) 的 −17）。
        let scale = (b * b).abs().max((4.0 * a * c).abs()).max(1e-30);
        if disc < -1e-10 * scale {
            return None;
        }
        let disc = disc.max(0.0);
        let sq = disc.sqrt();
        let t1 = (-b - sq) / (2.0 * a);
        let t2 = (-b + sq) / (2.0 * a);
        [t1, t2]
    };
    // 有效根：半径 r(t) = r0 + t·delta ≥ 0（等价 Skia x̂_t ≥ 0——cone.behind 的
    // t=−0.44 假根半径已穿零）；取较大有效根（Skia「bigger t」语义）。
    let mut best: Option<f32> = None;
    for t in roots {
        if !t.is_finite() || r0 + t * delta < 0.0 {
            continue;
        }
        let tf = t as f32;
        if best.is_none_or(|b| tf > b) {
            best = Some(tf);
        }
    }
    best
}

/// 图案平铺采样：按重复模式在 (x, y) 处取源 ImageData 像素。
///
/// - Repeat：x/y 均取模回绕。
/// - RepeatX：x 回绕，y 超出图高 → 透明。
/// - RepeatY：y 回绕，x 超出图宽 → 透明。
/// - NoRepeat：x/y 任一超出 → 透明。
///
/// 0×0 图案 → 透明。
fn sample_pattern_pixel(pattern: &CanvasPattern, x: f32, y: f32) -> Color {
    let w = pattern.image_data.width;
    let h = pattern.image_data.height;
    if w == 0 || h == 0 {
        return Color::TRANSPARENT;
    }
    // R34xx：平铺网格锚定在 fill 坐标空间——device 采样点先经 tile_transform（CTM 逆）映射
    // 回 fill 空间再取模（2d.pattern.paint.repeat.coord1 等；恒等 CTM 时零开销近似无）。
    let (fx, fy) = pattern.tile_transform.transform_point(x, y);
    let iw = w as i32;
    let ih = h as i32;
    let ix = fx.floor() as i32;
    let iy = fy.floor() as i32;
    // 按重复模式计算有效 tile 坐标；超出区域返回透明。
    let (tx, ty) = match pattern.repetition {
        PatternRepetition::Repeat => (ix.rem_euclid(iw), iy.rem_euclid(ih)),
        PatternRepetition::RepeatX => {
            if iy < 0 || iy >= ih {
                return Color::TRANSPARENT;
            }
            (ix.rem_euclid(iw), iy)
        }
        PatternRepetition::RepeatY => {
            if ix < 0 || ix >= iw {
                return Color::TRANSPARENT;
            }
            (ix, iy.rem_euclid(ih))
        }
        PatternRepetition::NoRepeat => {
            if ix < 0 || ix >= iw || iy < 0 || iy >= ih {
                return Color::TRANSPARENT;
            }
            (ix, iy)
        }
    };
    let idx = ((ty as u32 * w + tx as u32) * 4) as usize;
    let d = &pattern.image_data.data;
    if idx + 3 < d.len() {
        Color::rgba(d[idx], d[idx + 1], d[idx + 2], d[idx + 3])
    } else {
        Color::TRANSPARENT
    }
}

/// 渐变停止点颜色采样辅助函数。
///
/// 将偏移量限制在 [0.0, 1.0]，找到包围偏移量的两个停止点并线性插值。
fn sample_gradient_stops(stops: &[GradientStop], offset: f32) -> Color {
    // R34xx：无停止点 → 全透明（spec：渐变无 stops 时绘制无效果——2d.gradient.empty 期望
    // 保持背景；旧实现返 BLACK 污染像素）。
    if stops.is_empty() {
        return Color::TRANSPARENT;
    }
    let t = offset.clamp(0.0, 1.0);
    // R34xx：先按 offset 稳定排序（spec：color stops sorted by offset——添加序可能乱序，
    // 0.75 插在 0.5 中间会破坏同 offset 组连续性）；同 offset 保持添加序（稳定排序）。
    let mut sorted: Vec<&GradientStop> = stops.iter().collect();
    sorted.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(std::cmp::Ordering::Equal));
    // R34xx：同 offset 组（spec：同 offset 多个 stop，最后添加者生效）——插值对 =
    // (前组最后, 后组第一)（2d.gradient.interpolate.overlap：t=0.245 用 0.25 组第一蓝、
    // t=0.255 用 0.25 组最后黄——Skia 相邻对语义）。
    let mut groups: Vec<(f32, Color, Color)> = Vec::new(); // (offset, first, last)
    for s in &sorted {
        if let Some(g) = groups.last_mut().filter(|g| (g.0 - s.offset).abs() < f32::EPSILON) {
            g.2 = s.color;
            continue;
        }
        groups.push((s.offset, s.color, s.color));
    }
    if t <= groups[0].0 {
        return groups[0].2;
    }
    let last = groups[groups.len() - 1];
    if t >= last.0 {
        return last.2;
    }
    for i in 0..groups.len() - 1 {
        let (o0, _f0, l0) = groups[i];
        let (o1, f1, _l1) = groups[i + 1];
        if t >= o0 && t <= o1 {
            let span = o1 - o0;
            if span < f32::EPSILON {
                continue;
            }
            let frac = (t - o0) / span;
            return Color::rgba(
                lerp_u8(l0.r, f1.r, frac),
                lerp_u8(l0.g, f1.g, frac),
                lerp_u8(l0.b, f1.b, frac),
                lerp_u8(l0.a, f1.a, frac),
            );
        }
    }
    last.2
}

/// 线性插值两个 u8 值。
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

/// Canvas 状态（用于 save/restore）。
#[derive(Debug, Clone)]
pub(crate) struct CanvasState {
    pub(crate) fill_style: CanvasStyle,
    pub(crate) stroke_style: CanvasStyle,
    pub(crate) line_width: f32,
    pub(crate) font: FontDescriptor,
    pub(crate) global_alpha: f32,
    pub(crate) transform: Transform2D,
    pub(crate) composite_operation: CompositeOperation,
    pub(crate) shadow_color: Color,
    pub(crate) shadow_blur: f32,
    pub(crate) shadow_offset_x: f32,
    pub(crate) shadow_offset_y: f32,
    pub(crate) line_dash: Vec<f32>,
    pub(crate) line_dash_offset: f32,
    pub(crate) line_join: LineJoin,
    pub(crate) line_cap: LineCap,
    /// 图像平滑（抗锯齿）开关。
    pub(crate) image_smoothing_enabled: bool,
    /// 图像平滑质量（R3305）。
    pub(crate) image_smoothing_quality: ImageSmoothingQuality,
    /// 文本对齐。
    pub(crate) text_align: TextAlign,
    /// 文本基线。
    pub(crate) text_baseline: TextBaseline,
    /// 斜接限制。
    pub(crate) miter_limit: f32,
    /// 文本方向。
    pub(crate) direction: TextDirection,
    /// R34xx：裁剪路径（drawing state 的一部分，spec：save/restore 管理 clip——上游
    /// 2d.state.saverestore.clip：restore 后 clip 须回滚）。此前缺失致 restore 后 clip
    /// 残留裁剪后续绘制。
    pub(crate) clip_path: Option<Path2D>,
}

/// Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。
pub struct CanvasContext {
    /// 画布宽度。
    pub(crate) width: u32,
    /// 画布高度。
    pub(crate) height: u32,
    /// 当前填充样式。
    pub(crate) fill_style: CanvasStyle,
    /// 当前描边样式。
    pub(crate) stroke_style: CanvasStyle,
    /// 当前线宽。
    pub(crate) line_width: f32,
    /// 当前字体。
    pub(crate) font: FontDescriptor,
    /// 全局透明度。
    pub(crate) global_alpha: f32,
    /// 变换矩阵。
    pub(crate) transform: Transform2D,
    /// 渲染图元列表。
    pub(crate) primitives: RenderPrimitives,
    /// 状态栈（用于 save/restore）。
    pub(crate) state_stack: Vec<CanvasState>,
    /// 当前路径。
    pub(crate) current_path: Path2D,
    /// 像素缓冲区（RGBA，宽度 × 高度 × 4 字节）。
    pub(crate) pixel_buffer: Vec<u8>,
    /// 当前合成操作模式。
    pub(crate) composite_operation: CompositeOperation,
    /// 当前裁剪路径（如果有）。
    pub(crate) clip_path: Option<Path2D>,
    /// 阴影颜色。
    pub(crate) shadow_color: Color,
    /// 阴影模糊半径。
    pub(crate) shadow_blur: f32,
    /// 阴影水平偏移。
    pub(crate) shadow_offset_x: f32,
    /// 阴影垂直偏移。
    pub(crate) shadow_offset_y: f32,
    /// 线段虚线模式。
    pub(crate) line_dash: Vec<f32>,
    /// 线段虚线偏移。
    pub(crate) line_dash_offset: f32,
    /// 线段连接样式。
    pub(crate) line_join: LineJoin,
    /// 线段端点样式。
    pub(crate) line_cap: LineCap,
    /// 图像平滑（抗锯齿）开关。
    pub(crate) image_smoothing_enabled: bool,
    /// 图像平滑质量（R3305）。
    pub(crate) image_smoothing_quality: ImageSmoothingQuality,
    /// 文本对齐。
    pub(crate) text_align: TextAlign,
    /// 文本基线。
    pub(crate) text_baseline: TextBaseline,
    /// 斜接限制。
    pub(crate) miter_limit: f32,
    /// 文本方向。
    pub(crate) direction: TextDirection,
    /// R34xx：共享字体加载器（headless/testharness 路径注入——@font-face 字体 shape +
    /// 光栅化的真文本像素；None = 无字体栈，fill_text 回落启发式）。
    pub(crate) font_loader: Option<Arc<Mutex<FontLoader>>>,
    /// R34xx：stroke 单次调用去重 mask（段矩形/join/cap 重叠区只合成一次——spec stroke
    /// 每像素画一次；2d.strokeStyle.colorObject.transparency 的 2px 高矩形 50px 线宽
    /// 三段覆盖同像素致 alpha 128 变 224）。None = 非 stroke 绘制。
    pub(crate) stroke_dedup_mask: Option<Vec<u8>>,
    /// R34xx：当前字体解析到的 font_id（set_font 时经 loader 解析器查找）。
    pub(crate) font_id: Option<u32>,
}

/// 文本度量（HTML Canvas `TextMetrics`，
/// https://html.spec.whatwg.org/multipage/canvas.html#textmetrics）。
///
/// R3303：补全 spec 全字段。除 `width` 外均为字体度量启发式近似（canvas crate 无真实字体度量
/// 后端——ascent/descent/baseline 按 `font.size` 比例估，与既有 R3078 ascent=0.8em/descent=0.2em
/// 一致）。真实字体度量（经字体表 hhea/OS/2）为后续 follow-up（须接渲染流字体栈，render-stream
/// 协调点）。即便近似，完整字段集使文本布局库（chart.js 轴尺寸 / 自定义换行）不因缺字段读 NaN。
#[derive(Debug, Clone)]
pub struct TextMetrics {
    /// 文本宽度（advance 宽，原点起向右）。
    pub width: f32,
    /// 实际边界框上方（基线到最上像素的距离）。
    pub actual_bounding_box_ascent: f32,
    /// 实际边界框下方（基线到最下像素的距离）。
    pub actual_bounding_box_descent: f32,
    /// 实际边界框左侧（原点到最左像素的距离；通常 0，左伸字形如 italic 负值）。
    pub actual_bounding_box_left: f32,
    /// 实际边界框右侧（原点到最右像素的距离；≈ width）。
    pub actual_bounding_box_right: f32,
    /// 字体边界框上方（字体 ascent，由字体表给定；近似 0.8em）。
    pub font_bounding_box_ascent: f32,
    /// 字体边界框下方（字体 descent，由字体表给定；近似 0.2em）。
    pub font_bounding_box_descent: f32,
    /// 字母基线距默认基线（alphabetic）的距离（默认基线即 alphabetic → 0）。
    pub alphabetic_baseline: f32,
    /// 悬挂基线距默认基线的距离（Latin 近似 0.8em，悬挂基线在大写字母顶附近）。
    pub hanging_baseline: f32,
    /// 表意基线距默认基线的距离（近似 -0.2em，CJK 字形基线略低于 alphabetic）。
    pub ideographic_baseline: f32,
    /// R34xx：逐字形墨迹矩形（相对基线原点，未含对齐锚定偏移），按字符序——
    /// `(left, top, right, bottom)`。供 `TextMetrics.getActualBoundingBox(start, end)`
    /// 子串 bbox（2d.text.measure.getActualBoundingBox.tentative）。无字体栈时空。
    pub glyph_rects: Vec<(f32, f32, f32, f32)>,
}

/// 图像数据。
#[derive(Debug, Clone)]
pub struct ImageData {
    /// 宽度。
    pub width: u32,
    /// 高度。
    pub height: u32,
    /// RGBA 像素数据。
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 枚举基础测试 ──────────────────────────────────────

    #[test]
    fn test_font_weight_equality() {
        assert_eq!(FontWeight::Normal, FontWeight::Normal);
        assert_ne!(FontWeight::Normal, FontWeight::Bold);
    }

    #[test]
    fn test_font_style_equality() {
        assert_eq!(FontStyle::Normal, FontStyle::Normal);
        assert_ne!(FontStyle::Normal, FontStyle::Italic);
    }

    #[test]
    fn test_text_align_variants() {
        assert_ne!(TextAlign::Start, TextAlign::End);
        assert_ne!(TextAlign::Left, TextAlign::Right);
        assert_ne!(TextAlign::Center, TextAlign::Start);
    }

    #[test]
    fn test_text_baseline_variants() {
        assert_ne!(TextBaseline::Top, TextBaseline::Middle);
        assert_ne!(TextBaseline::Alphabetic, TextBaseline::Bottom);
    }

    #[test]
    fn test_text_direction_default() {
        assert_eq!(TextDirection::default(), TextDirection::Inherit);
    }

    #[test]
    fn test_line_join_default() {
        assert_eq!(LineJoin::default(), LineJoin::Miter);
    }

    #[test]
    fn test_line_cap_default() {
        assert_eq!(LineCap::default(), LineCap::Butt);
    }

    #[test]
    fn test_composite_operation_default() {
        assert_eq!(CompositeOperation::default(), CompositeOperation::SourceOver);
    }

    #[test]
    fn test_pattern_repetition_default() {
        assert_eq!(PatternRepetition::default(), PatternRepetition::Repeat);
    }

    // ── FontDescriptor 测试 ───────────────────────────────

    #[test]
    fn test_font_descriptor_default() {
        let desc = FontDescriptor::default();
        assert_eq!(desc.family, "sans-serif");
        assert!((desc.size - 10.0).abs() < f32::EPSILON);
        assert_eq!(desc.weight, FontWeight::Normal);
        assert_eq!(desc.style, FontStyle::Normal);
    }

    #[test]
    fn test_font_descriptor_clone() {
        let desc = FontDescriptor {
            family: "serif".into(),
            size: 14.0,
            weight: FontWeight::Bold,
            style: FontStyle::Italic,
            small_caps: false,
            weight_value: None,
            letter_spacing: "0px".to_string(),
            word_spacing: "0px".to_string(),
        };
        let cloned = desc.clone();
        assert_eq!(cloned.family, "serif");
        assert_eq!(cloned.size, 14.0);
        assert_eq!(cloned.weight, FontWeight::Bold);
        assert_eq!(cloned.style, FontStyle::Italic);
    }

    #[test]
    fn test_font_descriptor_debug() {
        let desc = FontDescriptor::default();
        let debug = format!("{:?}", desc);
        assert!(debug.contains("sans-serif"));
    }

    // ── FontDescriptor::parse_css 测试（R3304：ctx.font CSS font 串解析）──

    #[test]
    fn test_font_descriptor_parse_css_basic() {
        // 最简：size + family。
        let d = FontDescriptor::parse_css("16px Arial").unwrap();
        assert!((d.size - 16.0).abs() < f32::EPSILON);
        assert_eq!(d.family, "Arial");
        assert_eq!(d.weight, FontWeight::Normal);
        assert_eq!(d.style, FontStyle::Normal);
    }

    #[test]
    fn test_font_descriptor_parse_css_style_weight() {
        // italic bold 在 size 前，序无关。
        let d = FontDescriptor::parse_css("italic bold 20px serif").unwrap();
        assert!((d.size - 20.0).abs() < f32::EPSILON);
        assert_eq!(d.style, FontStyle::Italic);
        assert_eq!(d.weight, FontWeight::Bold);
        assert_eq!(d.family, "serif");

        // 反序同样识别。
        let d2 = FontDescriptor::parse_css("bold italic 20px serif").unwrap();
        assert_eq!(d2.style, FontStyle::Italic);
        assert_eq!(d2.weight, FontWeight::Bold);
    }

    #[test]
    fn test_font_descriptor_parse_css_numeric_weight() {
        // 数字 weight ≥600 → Bold，<600 → Normal。
        let bold = FontDescriptor::parse_css("700 12px sans").unwrap();
        assert_eq!(bold.weight, FontWeight::Bold);
        let normal = FontDescriptor::parse_css("400 12px sans").unwrap();
        assert_eq!(normal.weight, FontWeight::Normal);
    }

    #[test]
    fn test_font_descriptor_parse_css_line_height_dropped() {
        // /line-height 应被丢弃，size 与 family 正确。
        let d = FontDescriptor::parse_css("20px/1.5 Arial").unwrap();
        assert!((d.size - 20.0).abs() < f32::EPSILON);
        assert_eq!(d.family, "Arial");
    }

    #[test]
    fn test_font_descriptor_parse_css_units() {
        // R34xx：em 相对当前字号（默认 10px——'2em' → 20px）；rem 恒 16px。
        assert!((FontDescriptor::parse_css_with_current("2em serif", 10.0).unwrap().size - 20.0).abs() < f32::EPSILON);
        assert!((FontDescriptor::parse_css_with_current("2em serif", 40.0).unwrap().size - 80.0).abs() < f32::EPSILON);
        assert!((FontDescriptor::parse_css("12pt serif").unwrap().size - 16.0).abs() < 0.01); // 12pt * 96/72 = 16
        assert!((FontDescriptor::parse_css("2em serif").unwrap().size - 20.0).abs() < f32::EPSILON); // 2 * 默认 10px
        assert!((FontDescriptor::parse_css("1rem serif").unwrap().size - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_font_descriptor_parse_css_multi_family_preserved() {
        // 逗号多族 family 整体保留（含内部空白）。
        let d = FontDescriptor::parse_css("14px \"Helvetica Neue\", Arial, sans-serif").unwrap();
        assert!((d.size - 14.0).abs() < f32::EPSILON);
        assert!(d.family.contains("Helvetica Neue"));
        assert!(d.family.contains("Arial"));
        assert!(d.family.contains("sans-serif"));
    }

    #[test]
    fn test_font_descriptor_parse_css_invalid() {
        // 缺 size / 缺 family / 空串 → None（real browser 忽略非法 font 串）。
        assert!(FontDescriptor::parse_css("").is_none());
        assert!(FontDescriptor::parse_css("Arial").is_none()); // 缺 size
        assert!(FontDescriptor::parse_css("20px").is_none()); // 缺 family
        assert!(FontDescriptor::parse_css("small Arial").is_none()); // size 关键字不支持 → None
    }

    #[test]
    fn test_font_descriptor_parse_css_variant_dropped() {
        // small-caps（font-variant）应被识别为关键字并丢弃，不破坏后续解析。
        let d = FontDescriptor::parse_css("italic small-caps bold 18px monospace").unwrap();
        assert!((d.size - 18.0).abs() < f32::EPSILON);
        assert_eq!(d.style, FontStyle::Italic);
        assert_eq!(d.weight, FontWeight::Bold);
        assert_eq!(d.family, "monospace");
    }

    // ── Transform2D 测试 ──────────────────────────────────

    #[test]
    fn test_transform_identity() {
        let t = Transform2D::identity();
        assert_eq!(t.a, 1.0);
        assert_eq!(t.b, 0.0);
        assert_eq!(t.c, 0.0);
        assert_eq!(t.d, 1.0);
        assert_eq!(t.e, 0.0);
        assert_eq!(t.f, 0.0);
    }

    #[test]
    fn test_transform_default_is_identity() {
        let t = Transform2D::default();
        let id = Transform2D::identity();
        assert_eq!(t.a, id.a);
        assert_eq!(t.b, id.b);
        assert_eq!(t.c, id.c);
        assert_eq!(t.d, id.d);
        assert_eq!(t.e, id.e);
        assert_eq!(t.f, id.f);
    }

    #[test]
    fn test_transform_translate() {
        let t = Transform2D::translate(10.0, 20.0);
        assert_eq!(t.a, 1.0);
        assert_eq!(t.d, 1.0);
        assert_eq!(t.e, 10.0);
        assert_eq!(t.f, 20.0);
        // translate 应保持点平移
        let (x, y) = t.transform_point(5.0, 5.0);
        assert!((x - 15.0).abs() < f32::EPSILON);
        assert!((y - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_scale() {
        let t = Transform2D::scale(2.0, 3.0);
        assert_eq!(t.a, 2.0);
        assert_eq!(t.d, 3.0);
        let (x, y) = t.transform_point(10.0, 10.0);
        assert!((x - 20.0).abs() < f32::EPSILON);
        assert!((y - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_rotate_90() {
        let t = Transform2D::rotate(std::f32::consts::FRAC_PI_2);
        let (x, y) = t.transform_point(1.0, 0.0);
        // 旋转 90°: (1,0) → (0,1)
        assert!(x.abs() < 0.001, "x should be ~0, got {x}");
        assert!((y - 1.0).abs() < 0.001, "y should be ~1, got {y}");
    }

    #[test]
    fn test_transform_multiply_identity() {
        let id = Transform2D::identity();
        let t = Transform2D::translate(5.0, 10.0);
        let result = id.multiply(&t);
        assert!((result.e - 5.0).abs() < f32::EPSILON);
        assert!((result.f - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transform_multiply_translate_scale() {
        let scale = Transform2D::scale(2.0, 3.0);
        let translate = Transform2D::translate(10.0, 20.0);
        let result = scale.multiply(&translate);
        // multiply = self * other 的矩阵乘法:
        // result.e = scale.a * translate.e + scale.c * translate.f + scale.e
        //          = 2*10 + 0*20 + 0 = 20
        // result.f = scale.b * translate.e + scale.d * translate.f + scale.f
        //          = 0*10 + 3*20 + 0 = 60
        // 对点 (1,1) 应用：result * (1,1) = (2*1+0*1+20, 0*1+3*1+60) = (22, 63)
        let (x, y) = result.transform_point(1.0, 1.0);
        assert!((x - 22.0).abs() < 0.01, "x should be 22, got {x}");
        assert!((y - 63.0).abs() < 0.01, "y should be 63, got {y}");
    }

    #[test]
    fn test_transform_clone_copy() {
        let t = Transform2D::translate(1.0, 2.0);
        let copied = t; // Copy
        assert_eq!(copied.e, 1.0);
        let cloned = t; // Clone (Copy implies Clone)
        assert_eq!(cloned.f, 2.0);
    }

    #[test]
    fn test_transform_debug() {
        let t = Transform2D::identity();
        let debug = format!("{:?}", t);
        assert!(debug.contains("Transform2D"));
    }

    // ── LinearGradient 测试 ───────────────────────────────

    #[test]
    fn test_linear_gradient_new() {
        let g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        assert_eq!(g.x0, 0.0);
        assert_eq!(g.x1, 100.0);
        assert!(g.stops.is_empty());
    }

    #[test]
    fn test_linear_gradient_add_color_stop() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        assert_eq!(g.stops.len(), 2);
        assert_eq!(g.stops[0].offset, 0.0);
        assert_eq!(g.stops[1].offset, 1.0);
    }

    #[test]
    fn test_linear_gradient_sample_empty() {
        let g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        let c = g.sample_color(0.5);
        // R34xx：无停止点 → 全透明（spec；2d.gradient.empty）。
        assert_eq!(c, Color::TRANSPARENT);
    }

    #[test]
    fn test_linear_gradient_sample_single_stop() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.5, Color::rgb(255, 0, 0));
        assert_eq!(g.sample_color(0.0), Color::rgb(255, 0, 0));
        assert_eq!(g.sample_color(1.0), Color::rgb(255, 0, 0));
    }

    #[test]
    fn test_linear_gradient_sample_two_stops() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        let mid = g.sample_color(0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.g, 128);
        assert_eq!(mid.b, 128);
    }

    #[test]
    fn test_linear_gradient_sample_clamp() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        // offset < 0 → clamped to first stop
        assert_eq!(g.sample_color(-1.0), Color::rgb(255, 0, 0));
        // offset > 1 → clamped to last stop
        assert_eq!(g.sample_color(2.0), Color::rgb(0, 0, 255));
    }

    #[test]
    fn test_linear_gradient_clone() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 100.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        let cloned = g.clone();
        assert_eq!(cloned.stops.len(), 1);
    }

    // ── RadialGradient 测试 ───────────────────────────────

    #[test]
    fn test_radial_gradient_new() {
        let g = RadialGradient::new(0.0, 0.0, 0.0, 50.0, 50.0, 50.0);
        assert_eq!(g.x0, 0.0);
        assert_eq!(g.r1, 50.0);
        assert!(g.stops.is_empty());
    }

    #[test]
    fn test_radial_gradient_sample() {
        let mut g = RadialGradient::new(0.0, 0.0, 0.0, 50.0, 50.0, 50.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        let mid = g.sample_color(0.5);
        assert_eq!(mid.r, 128);
        assert_eq!(mid.b, 128);
    }

    // R34xx：radial_gradient_t 完整几何（cone 双曲线族/相切/退化）——WPT 2d.gradient.radial.* 驱动。
    // 用例几何照搬上游：
    // - cone.behind：c0=(120,25,10) c1=(211,25,100) 相交，背后区不画
    // - cone.beside：c0=(0,100,40) c1=(100,100,50) 相离，全部不画
    // - cone.bottom：c0=(210,25,100) c1=(230,25,101) 相交，远端点 t<0 → clamp 0
    // - cone.front：c0=(311,25,10) c1=(210,25,100) 相交，远端点 t>1 → clamp 1
    // - shape1：c0=(55,40,15) c1=(67.5,40,22.5) 相交，锥外判别式负 → None
    #[test]
    fn test_radial_t_cone_behind_not_painted() {
        let g = RadialGradient::new(120.0, 25.0, 10.0, 211.0, 25.0, 100.0);
        // 画布 (50,25) 位于两圆相交 chord 之后 → 半径穿零伪根 → None（透明不画）。
        assert_eq!(radial_gradient_t(&g, 50.0, 25.0), None);
        assert_eq!(radial_gradient_t(&g, 1.0, 25.0), None);
    }

    #[test]
    fn test_radial_t_cone_beside_not_painted() {
        let g = RadialGradient::new(0.0, 100.0, 40.0, 100.0, 100.0, 50.0);
        // 相离：画布像素射线不达外圆 → 判别式负/伪根 → None。
        assert_eq!(radial_gradient_t(&g, 50.0, 25.0), None);
        assert_eq!(radial_gradient_t(&g, 1.0, 1.0), None);
    }

    #[test]
    fn test_radial_t_cone_bottom_clamps_zero() {
        let g = RadialGradient::new(210.0, 25.0, 100.0, 230.0, 25.0, 101.0);
        // (50,25) 在外圆左侧：二次根 t≈-3.16（有效）→ clamp 0（WPT 期望 stop0 色）。
        let t = radial_gradient_t(&g, 50.0, 25.0).unwrap();
        assert!(t < 0.0, "t={t} should be negative (clamp to 0)");
    }

    #[test]
    fn test_radial_t_cone_front_clamps_one() {
        let g = RadialGradient::new(311.0, 25.0, 10.0, 210.0, 25.0, 100.0);
        // (50,25) 远在外圆左侧：较大根 t≈24.6 → clamp 1（WPT 期望 stop1 色）。
        let t = radial_gradient_t(&g, 50.0, 25.0).unwrap();
        assert!(t > 1.0, "t={t} should be > 1 (clamp to 1)");
    }

    #[test]
    fn test_radial_t_cone_shape1_discriminant_negative() {
        let g = RadialGradient::new(55.0, 40.0, 15.0, 67.5, 40.0, 22.5);
        // (1,1) 在锥外：判别式负 → None（WPT shape1 期望保持背景）。
        assert_eq!(radial_gradient_t(&g, 1.0, 1.0), None);
        // (50,1) 同样在 iso 圆族之外（圆锥外）→ None；WPT shape2 的 (50,1) 断言像素
        // 位于其绿三角覆盖区内，故外部 None 与期望一致。
        assert_eq!(radial_gradient_t(&g, 50.0, 1.0), None);
    }

    #[test]
    fn test_radial_t_contained_classic_focal_point() {
        // 经典焦点退化：c0 半径 0（焦点），点 (50,0) 在焦点与外圆之间 →
        // t = |P|/|Q| = 1/3（较大根为 1 = 端点圆本身，取较大根得 1——clamp 仍 1；
        // 断言取较大根语义本身：t=1 而非 1/3）。
        let g = RadialGradient::new(0.0, 0.0, 0.0, 100.0, 0.0, 50.0);
        let t = radial_gradient_t(&g, 50.0, 0.0).unwrap();
        assert!((t - 1.0).abs() < 1e-5, "larger root expected, got {t}");
    }

    #[test]
    fn test_radial_t_equal_radii_concentric_empty() {
        // 同心等半径：单个圆（测度零）→ None（WPT 2d.gradient.radial.equal 期望保持背景）。
        let g = RadialGradient::new(50.0, 25.0, 20.0, 50.0, 25.0, 20.0);
        assert_eq!(radial_gradient_t(&g, 1.0, 1.0), None);
        assert_eq!(radial_gradient_t(&g, 50.0, 25.0), None);
    }

    #[test]
    fn test_radial_t_cylinder_equal_radii_offset() {
        // 等半径不同心（cylinder）：半径恒正，t<0 → clamp 0。
        let g = RadialGradient::new(210.0, 25.0, 100.0, 230.0, 25.0, 100.0);
        let t = radial_gradient_t(&g, 50.0, 25.0).unwrap();
        assert!(t < 0.0, "t={t}");
    }

    #[test]
    fn test_radial_t_tangent_circles_not_painted() {
        // 内切相切（touch1：c0=(150,25,50) c1=(200,25,100)，d=50=r1-r0）：
        // 画布在切点左侧，iso 圆族均不达 → 无有效根。
        let g = RadialGradient::new(150.0, 25.0, 50.0, 200.0, 25.0, 100.0);
        assert_eq!(radial_gradient_t(&g, 50.0, 25.0), None);
        assert_eq!(radial_gradient_t(&g, 98.0, 25.0), None);
    }

    #[test]
    fn test_radial_t_concentric_fallback() {
        // 同心不同径：回落距离归一化（inside1 语义）。
        let g = RadialGradient::new(50.0, 25.0, 0.0, 50.0, 25.0, 100.0);
        let t = radial_gradient_t(&g, 50.0, 25.0).unwrap();
        assert_eq!(t, 0.0);
        let t2 = radial_gradient_t(&g, 100.0, 25.0).unwrap();
        assert_eq!(t2, 0.5);
    }

    // ── ConicGradient 测试 ────────────────────────────────

    #[test]
    fn test_conic_gradient_new() {
        let g = ConicGradient::new(0.0, 50.0, 50.0);
        assert_eq!(g.cx, 50.0);
        assert_eq!(g.cy, 50.0);
        assert!(g.stops.is_empty());
    }

    #[test]
    fn test_conic_gradient_sample() {
        let mut g = ConicGradient::new(0.0, 0.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        let mid = g.sample_color(0.5);
        assert_eq!(mid.r, 128);
    }

    // ── CanvasPattern 测试 ────────────────────────────────

    #[test]
    fn test_canvas_pattern_new() {
        let img = ImageData {
            width: 10,
            height: 10,
            data: vec![0u8; 400],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::Repeat);
        assert_eq!(pattern.repetition, PatternRepetition::Repeat);
        assert_eq!(pattern.image_data.width, 10);
    }

    #[test]
    fn test_canvas_pattern_no_repeat() {
        let img = ImageData {
            width: 5,
            height: 5,
            data: vec![0u8; 100],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::NoRepeat);
        assert_eq!(pattern.repetition, PatternRepetition::NoRepeat);
    }

    // R3085：图案平铺采样（sample_pattern_pixel）。2×2 源四角四色，验证四种重复模式 + 越界透明 + 0×0 透明。
    #[test]
    fn test_sample_pattern_pixel_tiling_r3085() {
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![
                255, 0, 0, 255, // (0,0) red
                0, 255, 0, 255, // (1,0) green
                0, 0, 255, 255, // (0,1) blue
                255, 255, 255, 255, // (1,1) white
            ],
        };
        // Repeat：x/y 均回绕。device (2,0)→tile(0,0) red；(3,1)→tile(1,1) white；(-1,0)→tile(1,0) green。
        let rep = CanvasPattern::new(img.clone(), PatternRepetition::Repeat);
        assert_eq!(sample_pattern_pixel(&rep, 2.0, 0.0), Color::rgba(255, 0, 0, 255));
        assert_eq!(sample_pattern_pixel(&rep, 3.0, 1.0), Color::rgba(255, 255, 255, 255));
        assert_eq!(sample_pattern_pixel(&rep, -1.0, 0.0), Color::rgba(0, 255, 0, 255));
        // RepeatX：x 回绕，y 超出图高 → 透明。device (0,2)→透明；(2,0)→x 回绕 red。
        let rpx = CanvasPattern::new(img.clone(), PatternRepetition::RepeatX);
        assert_eq!(sample_pattern_pixel(&rpx, 0.0, 2.0), Color::TRANSPARENT);
        assert_eq!(sample_pattern_pixel(&rpx, 2.0, 0.0), Color::rgba(255, 0, 0, 255));
        // RepeatY：y 回绕，x 超出图宽 → 透明。device (2,0)→透明（x OOB）；(0,2)→y 回绕行 0 red；(0,3)→y 回绕行 1 blue。
        let rpy = CanvasPattern::new(img.clone(), PatternRepetition::RepeatY);
        assert_eq!(sample_pattern_pixel(&rpy, 2.0, 0.0), Color::TRANSPARENT);
        assert_eq!(sample_pattern_pixel(&rpy, 0.0, 2.0), Color::rgba(255, 0, 0, 255));
        assert_eq!(sample_pattern_pixel(&rpy, 0.0, 3.0), Color::rgba(0, 0, 255, 255));
        // NoRepeat：x/y 任一超出 → 透明；tile 内取像素。device (0,0) red；(2,0) 透明；(0,2) 透明。
        let nrp = CanvasPattern::new(img.clone(), PatternRepetition::NoRepeat);
        assert_eq!(sample_pattern_pixel(&nrp, 0.0, 0.0), Color::rgba(255, 0, 0, 255));
        assert_eq!(sample_pattern_pixel(&nrp, 2.0, 0.0), Color::TRANSPARENT);
        assert_eq!(sample_pattern_pixel(&nrp, 0.0, 2.0), Color::TRANSPARENT);
        // 0×0 图案 → 透明。
        let empty = CanvasPattern::new(
            ImageData {
                width: 0,
                height: 0,
                data: vec![],
            },
            PatternRepetition::Repeat,
        );
        assert_eq!(sample_pattern_pixel(&empty, 0.0, 0.0), Color::TRANSPARENT);
    }

    // ── CanvasStyle 测试 ──────────────────────────────────

    #[test]
    fn test_canvas_style_default_black() {
        let style = CanvasStyle::default_black();
        let color = style.resolve_color();
        assert_eq!(color, Color::BLACK);
    }

    #[test]
    fn test_canvas_style_color_resolve() {
        let style = CanvasStyle::Color(Color::rgb(128, 64, 32));
        assert_eq!(style.resolve_color(), Color::rgb(128, 64, 32));
    }

    #[test]
    fn test_canvas_style_linear_gradient_resolve() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        let style = CanvasStyle::LinearGradient(g);
        let c = style.resolve_color();
        // offset 0.5 → mid-gray
        assert_eq!(c.r, 128);
    }

    #[test]
    fn test_canvas_style_radial_gradient_resolve() {
        let mut g = RadialGradient::new(0.0, 0.0, 0.0, 50.0, 50.0, 50.0);
        g.add_color_stop(0.0, Color::rgb(255, 0, 0));
        g.add_color_stop(1.0, Color::rgb(0, 0, 255));
        let style = CanvasStyle::RadialGradient(g);
        let c = style.resolve_color();
        assert!(c.r > 0);
    }

    #[test]
    fn test_canvas_style_conic_gradient_resolve() {
        let mut g = ConicGradient::new(0.0, 50.0, 50.0);
        g.add_color_stop(0.0, Color::rgb(100, 100, 100));
        let style = CanvasStyle::ConicGradient(g);
        let c = style.resolve_color();
        assert_eq!(c.r, 100);
    }

    #[test]
    fn test_canvas_style_pattern_resolve() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![255, 0, 0, 255],
        };
        let pattern = CanvasPattern::new(img, PatternRepetition::Repeat);
        let style = CanvasStyle::Pattern(pattern);
        assert_eq!(style.resolve_color(), Color::BLACK);
    }

    #[test]
    fn test_canvas_style_clone() {
        let style = CanvasStyle::Color(Color::rgb(1, 2, 3));
        let cloned = style.clone();
        assert_eq!(cloned.resolve_color(), Color::rgb(1, 2, 3));
    }

    // ── TextMetrics 测试 ──────────────────────────────────

    #[test]
    fn test_text_metrics_fields() {
        // R3303：spec 全 10 字段均可读写。
        let metrics = TextMetrics {
            width: 120.5,
            actual_bounding_box_ascent: 10.0,
            actual_bounding_box_descent: 3.0,
            actual_bounding_box_left: -1.0,
            actual_bounding_box_right: 120.5,
            font_bounding_box_ascent: 12.0,
            font_bounding_box_descent: 3.5,
            alphabetic_baseline: 0.0,
            hanging_baseline: 10.0,
            ideographic_baseline: -3.0,
            glyph_rects: Vec::new(),
        };
        assert!((metrics.width - 120.5).abs() < f32::EPSILON);
        assert_eq!(metrics.actual_bounding_box_ascent, 10.0);
        assert_eq!(metrics.actual_bounding_box_descent, 3.0);
        assert_eq!(metrics.actual_bounding_box_left, -1.0);
        assert_eq!(metrics.actual_bounding_box_right, 120.5);
        assert_eq!(metrics.font_bounding_box_ascent, 12.0);
        assert_eq!(metrics.font_bounding_box_descent, 3.5);
        assert_eq!(metrics.alphabetic_baseline, 0.0);
        assert_eq!(metrics.hanging_baseline, 10.0);
        assert_eq!(metrics.ideographic_baseline, -3.0);
    }

    #[test]
    fn test_text_metrics_clone() {
        let m = TextMetrics {
            width: 50.0,
            actual_bounding_box_ascent: 8.0,
            actual_bounding_box_descent: 2.0,
            actual_bounding_box_left: 0.0,
            actual_bounding_box_right: 50.0,
            font_bounding_box_ascent: 8.0,
            font_bounding_box_descent: 2.0,
            alphabetic_baseline: 0.0,
            hanging_baseline: 8.0,
            ideographic_baseline: -2.0,
            glyph_rects: Vec::new(),
        };
        let cloned = m.clone();
        assert_eq!(cloned.width, m.width);
        assert_eq!(cloned.font_bounding_box_ascent, m.font_bounding_box_ascent);
        assert_eq!(cloned.ideographic_baseline, m.ideographic_baseline);
    }

    // ── ImageData 测试 ────────────────────────────────────

    #[test]
    fn test_image_data_fields() {
        let img = ImageData {
            width: 2,
            height: 2,
            data: vec![255; 16], // 2x2 RGBA = 16 bytes
        };
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.data.len(), 16);
    }

    #[test]
    fn test_image_data_clone() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![128, 64, 32, 255],
        };
        let cloned = img.clone();
        assert_eq!(cloned.data, img.data);
    }

    #[test]
    fn test_image_data_debug() {
        let img = ImageData {
            width: 1,
            height: 1,
            data: vec![0; 4],
        };
        let debug = format!("{:?}", img);
        assert!(debug.contains("ImageData"));
    }

    // ── GradientStop 测试 ─────────────────────────────────

    #[test]
    fn test_gradient_stop_fields() {
        let stop = GradientStop {
            offset: 0.5,
            color: Color::rgb(128, 128, 128),
        };
        assert!((stop.offset - 0.5).abs() < f32::EPSILON);
    }

    // ── sample_gradient_stops 间接测试（通过渐变类型）──

    #[test]
    fn test_gradient_three_stops_interpolation() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(0, 0, 0));
        g.add_color_stop(0.5, Color::rgb(128, 128, 128));
        g.add_color_stop(1.0, Color::rgb(255, 255, 255));
        // at 0.25 → between stop0 and stop1
        let c = g.sample_color(0.25);
        assert_eq!(c.r, 64);
        // at 0.75 → between stop1 and stop2
        let c = g.sample_color(0.75);
        assert_eq!(c.r, 192);
    }

    #[test]
    fn test_gradient_identical_stops() {
        let mut g = LinearGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, Color::rgb(128, 128, 128));
        g.add_color_stop(0.0, Color::rgb(128, 128, 128));
        // span ≈ 0 → should return first stop's color
        let c = g.sample_color(0.0);
        assert_eq!(c.r, 128);
    }
}
