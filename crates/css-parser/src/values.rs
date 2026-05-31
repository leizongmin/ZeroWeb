//! CSS 属性值类型。
//!
//! 定义 CSS 属性值的类型化表示，以及解析函数。

/// CSS 长度值。
#[derive(Debug, Clone, PartialEq)]
pub enum LengthValue {
    /// 绝对长度（px）。
    Px(f64),
    /// em 单位。
    Em(f64),
    /// rem 单位。
    Rem(f64),
    /// vh 单位。
    Vh(f64),
    /// vw 单位。
    Vw(f64),
    /// vmin 单位。
    Vmin(f64),
    /// vmax 单位。
    Vmax(f64),
    /// ch 单位。
    Ch(f64),
    /// 百分比值（0-100）。
    Percentage(f64),
    /// auto 关键字。
    Auto,
    /// min-content 关键字 — 最小内容宽度。
    MinContent,
    /// max-content 关键字 — 最大内容宽度。
    MaxContent,
    /// 数学表达式（calc/min/max/clamp），在样式解析阶段无法直接求值，
    /// 需要在 [`resolve_computed_style`](crate::resolve_computed_style) 阶段用完整上下文求值。
    Calc(Box<CalcExpr>),
    /// fit-content() 函数，将尺寸限制为内容最大宽度不超过给定值。
    /// 参数可以是长度或百分比。
    FitContent(Box<LengthValue>),
}

/// CSS 颜色值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColorValue {
    /// RGB 颜色。
    Rgba(u8, u8, u8, u8),
    /// HSL 颜色。
    Hsla(f64, f64, f64, f64),
    /// 命名颜色。
    Named(String),
    /// transparent。
    Transparent,
    /// currentColor。
    CurrentColor,
}

/// CSS display 值。
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayValue {
    /// block。
    Block,
    /// inline。
    Inline,
    /// inline-block。
    InlineBlock,
    /// flex。
    Flex,
    /// inline-flex。
    InlineFlex,
    /// grid。
    Grid,
    /// inline-grid。
    InlineGrid,
    /// none。
    None,
    /// contents。
    Contents,
    /// flow。
    Flow,
    /// flow-root。
    FlowRoot,
    /// list-item。
    ListItem,
}

/// CSS float 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FloatValue {
    /// none（默认值）。
    None,
    /// left。
    Left,
    /// right。
    Right,
    /// inline-start。
    InlineStart,
    /// inline-end。
    InlineEnd,
}

/// CSS clear 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ClearValue {
    /// none（默认值）。
    None,
    /// left。
    Left,
    /// right。
    Right,
    /// both。
    Both,
    /// inline-start。
    InlineStart,
    /// inline-end。
    InlineEnd,
}

/// CSS position 值。
#[derive(Debug, Clone, PartialEq)]
pub enum PositionValue {
    /// static。
    Static,
    /// relative。
    Relative,
    /// absolute。
    Absolute,
    /// fixed。
    Fixed,
    /// sticky。
    Sticky,
}

/// CSS overflow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowValue {
    /// visible。
    Visible,
    /// hidden。
    Hidden,
    /// scroll。
    Scroll,
    /// auto。
    Auto,
    /// clip。
    Clip,
}

/// CSS list-style-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStyleTypeValue {
    /// disc（默认值）。
    Disc,
    /// circle。
    Circle,
    /// square。
    Square,
    /// decimal。
    Decimal,
    /// decimal-leading-zero。
    DecimalLeadingZero,
    /// lower-roman。
    LowerRoman,
    /// upper-roman。
    UpperRoman,
    /// lower-alpha / lower-latin。
    LowerAlpha,
    /// upper-alpha / upper-latin。
    UpperAlpha,
    /// none。
    None,
}

/// CSS list-style-position 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ListStylePositionValue {
    /// outside（默认值）。
    Outside,
    /// inside。
    Inside,
}

/// CSS flex-direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FlexDirectionValue {
    /// row。
    Row,
    /// row-reverse。
    RowReverse,
    /// column。
    Column,
    /// column-reverse。
    ColumnReverse,
}

/// CSS flex-wrap 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FlexWrapValue {
    /// nowrap。
    Nowrap,
    /// wrap。
    Wrap,
    /// wrap-reverse。
    WrapReverse,
}

/// CSS justify-content / align-items 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AlignmentValue {
    /// flex-start。
    FlexStart,
    /// flex-end。
    FlexEnd,
    /// center。
    Center,
    /// space-between。
    SpaceBetween,
    /// space-around。
    SpaceAround,
    /// space-evenly。
    SpaceEvenly,
    /// stretch。
    Stretch,
    /// start。
    Start,
    /// end。
    End,
    /// baseline。
    Baseline,
}

/// CSS box-sizing 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BoxSizingValue {
    /// content-box。
    ContentBox,
    /// border-box。
    BorderBox,
}

/// CSS visibility 值。
#[derive(Debug, Clone, PartialEq)]
pub enum VisibilityValue {
    /// visible。
    Visible,
    /// hidden。
    Hidden,
    /// collapse。
    Collapse,
}

/// CSS word-break 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WordBreakValue {
    /// normal。
    Normal,
    /// break-all。
    BreakAll,
    /// keep-all。
    KeepAll,
    /// break-word。
    BreakWord,
}

/// CSS writing-mode 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WritingModeValue {
    /// horizontal-tb。
    HorizontalTb,
    /// vertical-rl。
    VerticalRl,
    /// vertical-lr。
    VerticalLr,
}

/// CSS text-decoration-line 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextDecorationLineValue {
    /// none。
    None,
    /// underline。
    Underline,
    /// overline。
    Overline,
    /// line-through。
    LineThrough,
    /// blink。
    Blink,
}

/// CSS text-transform 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextTransformValue {
    /// none。
    None,
    /// uppercase。
    Uppercase,
    /// lowercase。
    Lowercase,
    /// capitalize。
    Capitalize,
}

/// CSS font-weight 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontWeightValue {
    /// 绝对权重（100-900）。
    Absolute(u16),
    /// bold。
    Bold,
    /// normal。
    Normal,
    /// bolder。
    Bolder,
    /// lighter。
    Lighter,
}

/// CSS font-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontStyleValue {
    /// normal。
    Normal,
    /// italic。
    Italic,
    /// oblique。
    Oblique(Option<f64>),
}

/// CSS 自定义属性引用（`var()` 函数）。
#[derive(Debug, Clone, PartialEq)]
pub struct VarReference {
    /// 自定义属性名（如 `--main-color`）。
    pub name: String,
    /// 回退值。
    pub fallback: Option<String>,
}

/// CSS calc() 表达式。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcExpr {
    /// 数值常量。
    Number(f64),
    /// 长度值（带单位）。
    Length(LengthValue),
    /// 二元运算：left op right。
    BinaryOp(Box<CalcExpr>, CalcOp, Box<CalcExpr>),
    /// min() 函数：取所有参数中的最小值。
    Min(Vec<CalcExpr>),
    /// max() 函数：取所有参数中的最大值。
    Max(Vec<CalcExpr>),
    /// clamp(min, val, max) 函数：将 val 限制在 [min, max] 范围内。
    Clamp {
        /// 最小值。
        min: Box<CalcExpr>,
        /// 首选值。
        val: Box<CalcExpr>,
        /// 最大值。
        max: Box<CalcExpr>,
    },
}

/// CSS calc() 运算符。
#[derive(Debug, Clone, PartialEq)]
pub enum CalcOp {
    /// 加法。
    Add,
    /// 减法。
    Subtract,
    /// 乘法。
    Multiply,
    /// 除法。
    Divide,
}

/// CSS calc() 表达式求值上下文。
///
/// 提供相对单位转换为像素值所需的参考尺寸。
#[derive(Debug, Clone, Default)]
pub struct CalcContext {
    /// 父元素长度，用于百分比计算。
    pub parent_length: Option<f64>,
    /// 当前字体大小（px），用于 em 单位转换。
    pub font_size: Option<f64>,
    /// 根元素字体大小（px），用于 rem 单位转换。
    pub root_font_size: Option<f64>,
    /// 视口高度（px），用于 vh/vmin/vmax 单位转换。
    pub viewport_height: Option<f64>,
    /// 视口宽度（px），用于 vw/vmin/vmax 单位转换。
    pub viewport_width: Option<f64>,
    /// "0" 字形宽度（px），用于 ch 单位转换。
    pub ch_width: Option<f64>,
}

/// calc() 表达式解析器内部状态。
struct CalcParser<'a> {
    /// 待解析的输入切片。
    input: &'a str,
    /// 当前位置（字节偏移）。
    pos: usize,
    /// 当前递归深度。
    depth: u32,
}

/// 最大递归深度限制。
const MAX_CALC_DEPTH: u32 = 10;

impl<'a> CalcParser<'a> {
    /// 跳过前导空白。
    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    /// 查看当前剩余输入。
    fn peek_rest(&self) -> &'a str {
        &self.input[self.pos..]
    }

    /// 尝试消费指定前缀。
    fn try_consume(&mut self, prefix: &str) -> bool {
        let rest = self.peek_rest();
        if rest.starts_with(prefix) {
            self.pos += prefix.len();
            true
        } else {
            false
        }
    }

    /// 解析顶层表达式（处理 + - 运算符，优先级较低）。
    fn parse_expr(&mut self) -> Option<CalcExpr> {
        let mut left = self.parse_term()?;

        loop {
            self.skip_whitespace();
            let rest = self.peek_rest();
            if rest.starts_with(')') || rest.is_empty() {
                break;
            }
            if rest.starts_with('+') {
                self.pos += 1;
                let right = self.parse_term()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Add, Box::new(right));
            } else if rest.starts_with('-') {
                // 区分减号和负号：减号前面有操作数
                self.pos += 1;
                let right = self.parse_term()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Subtract, Box::new(right));
            } else {
                break;
            }
        }

        Some(left)
    }

    /// 解析高优先级项（处理 * / 运算符）。
    fn parse_term(&mut self) -> Option<CalcExpr> {
        let mut left = self.parse_factor()?;

        loop {
            self.skip_whitespace();
            let rest = self.peek_rest();
            if rest.starts_with('*') {
                self.pos += 1;
                let right = self.parse_factor()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Multiply, Box::new(right));
            } else if rest.starts_with('/') {
                self.pos += 1;
                let right = self.parse_factor()?;
                left = CalcExpr::BinaryOp(Box::new(left), CalcOp::Divide, Box::new(right));
            } else {
                break;
            }
        }

        Some(left)
    }

    /// 解析原子因子：数字、长度值、嵌套 calc() 或括号表达式。
    fn parse_factor(&mut self) -> Option<CalcExpr> {
        self.skip_whitespace();

        // 处理负号前缀
        let neg = if self.peek_rest().starts_with('-') {
            // 判断是否为负号（而非减号）：后面紧跟数字或 calc(
            let after = self.peek_rest()[1..].trim_start();
            if after.starts_with(|c: char| c.is_ascii_digit() || c == '.') || after.starts_with("calc(") {
                self.pos += 1;
                true
            } else {
                false
            }
        } else {
            false
        };

        self.skip_whitespace();

        let mut expr = if self.try_consume("calc(") {
            // 嵌套 calc() 表达式
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let inner = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            inner
        } else if self.try_consume("min(") {
            // min(v1, v2, ...) 函数
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let args = self.parse_comma_list()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            CalcExpr::Min(args)
        } else if self.try_consume("max(") {
            // max(v1, v2, ...) 函数
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let args = self.parse_comma_list()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            CalcExpr::Max(args)
        } else if self.try_consume("clamp(") {
            // clamp(min, val, max) 函数
            if self.depth >= MAX_CALC_DEPTH {
                return None;
            }
            self.depth += 1;
            let min = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(",") {
                return None;
            }
            let val = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(",") {
                return None;
            }
            let max = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            self.depth -= 1;
            CalcExpr::Clamp {
                min: Box::new(min),
                val: Box::new(val),
                max: Box::new(max),
            }
        } else if self.try_consume("(") {
            // 括号表达式
            let inner = self.parse_expr()?;
            self.skip_whitespace();
            if !self.try_consume(")") {
                return None;
            }
            inner
        } else {
            // 解析原子操作数：数值或长度值
            self.parse_atom()?
        };

        if neg {
            expr = CalcExpr::BinaryOp(Box::new(CalcExpr::Number(0.0)), CalcOp::Subtract, Box::new(expr));
        }

        Some(expr)
    }

    /// 解析逗号分隔的表达式列表（用于 min/max 函数）。
    fn parse_comma_list(&mut self) -> Option<Vec<CalcExpr>> {
        let mut args = Vec::new();
        args.push(self.parse_expr()?);
        loop {
            self.skip_whitespace();
            if !self.try_consume(",") {
                break;
            }
            args.push(self.parse_expr()?);
        }
        Some(args)
    }

    /// 解析原子操作数（数值或带单位的长度值）。
    fn parse_atom(&mut self) -> Option<CalcExpr> {
        self.skip_whitespace();
        let rest = self.peek_rest();

        // 从当前位置读取到下一个运算符、空白、右括号或逗号
        let end = rest
            .bytes()
            .position(|b| b == b'+' || b == b'-' || b == b'*' || b == b'/' || b == b')' || b == b',')
            .unwrap_or(rest.len());

        if end == 0 {
            return None;
        }

        let token = rest[..end].trim();
        if token.is_empty() {
            return None;
        }

        self.pos += rest[..end].len();

        // 尝试解析为纯数字
        if let Ok(num) = token.parse::<f64>() {
            return Some(CalcExpr::Number(num));
        }

        // 尝试解析为长度值
        if let Some(length) = parse_length(token) {
            return Some(CalcExpr::Length(length));
        }

        None
    }
}

/// 解析 CSS calc() 表达式。
///
/// 支持格式如 `"calc(100% - 20px)"`、`"calc(50% + 10px)"`、`"calc(2 * 10px)"`。
/// 支持嵌套 calc 表达式如 `"calc(calc(100% - 20px) / 2)"`。
/// 运算符优先级：`*` `/` 高于 `+` `-`。
pub fn parse_calc(value: &str) -> Option<CalcExpr> {
    let value = value.trim();

    // 检查 calc(...) 包装
    if !value.starts_with("calc(") || !value.ends_with(')') {
        return None;
    }

    let inner = value.get(5..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }

    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };

    let expr = parser.parse_expr()?;

    // 确保整个输入已被消费
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }

    Some(expr)
}

/// 解析 CSS 数学函数（calc/min/max/clamp）。
///
/// 根据前缀自动识别并解析对应的数学函数。
/// 返回统一的 [`CalcExpr`] 表达式树。
pub fn parse_math_function(value: &str) -> Option<CalcExpr> {
    let value = value.trim();

    if value.starts_with("calc(") && value.ends_with(')') {
        parse_calc(value)
    } else if value.starts_with("min(") && value.ends_with(')') {
        parse_min(value)
    } else if value.starts_with("max(") && value.ends_with(')') {
        parse_max(value)
    } else if value.starts_with("clamp(") && value.ends_with(')') {
        parse_clamp(value)
    } else {
        None
    }
}

/// 解析 CSS min() 函数。
///
/// 格式：`min(v1, v2, ...)` — 取所有参数中的最小值。
pub fn parse_min(value: &str) -> Option<CalcExpr> {
    let value = value.trim();
    if !value.starts_with("min(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.get(4..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }
    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };
    let args = parser.parse_comma_list()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }
    Some(CalcExpr::Min(args))
}

/// 解析 CSS max() 函数。
///
/// 格式：`max(v1, v2, ...)` — 取所有参数中的最大值。
pub fn parse_max(value: &str) -> Option<CalcExpr> {
    let value = value.trim();
    if !value.starts_with("max(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.get(4..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }
    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };
    let args = parser.parse_comma_list()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }
    Some(CalcExpr::Max(args))
}

/// 解析 CSS clamp() 函数。
///
/// 格式：`clamp(min, val, max)` — 将 val 限制在 [min, max] 范围。
pub fn parse_clamp(value: &str) -> Option<CalcExpr> {
    let value = value.trim();
    if !value.starts_with("clamp(") || !value.ends_with(')') {
        return None;
    }
    let inner = value.get(6..value.len() - 1)?.trim();
    if inner.is_empty() {
        return None;
    }
    let mut parser = CalcParser {
        input: inner,
        pos: 0,
        depth: 0,
    };
    let min = parser.parse_expr()?;
    parser.skip_whitespace();
    if !parser.try_consume(",") {
        return None;
    }
    let val = parser.parse_expr()?;
    parser.skip_whitespace();
    if !parser.try_consume(",") {
        return None;
    }
    let max = parser.parse_expr()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return None;
    }
    Some(CalcExpr::Clamp {
        min: Box::new(min),
        val: Box::new(val),
        max: Box::new(max),
    })
}

/// 计算 CSS calc() 表达式的像素值。
///
/// `parent_length` 用于解析百分比值（如 `100%` = `parent_length`）。
/// 返回计算结果（像素）。
pub fn eval_calc(expr: &CalcExpr, parent_length: Option<f64>) -> Option<f64> {
    let ctx = CalcContext {
        parent_length,
        ..Default::default()
    };
    eval_calc_with_context(expr, &ctx)
}

/// 使用完整上下文计算 CSS calc() 表达式的像素值。
///
/// 支持所有单位：px、百分比、em、rem、vh、vw、vmin、vmax、ch。
/// 相对单位需要对应的上下文字段已设置，否则返回 `None`。
pub fn eval_calc_with_context(expr: &CalcExpr, ctx: &CalcContext) -> Option<f64> {
    match expr {
        CalcExpr::Number(n) => Some(*n),
        CalcExpr::Length(lv) => resolve_length_to_px(lv, ctx),
        CalcExpr::BinaryOp(left, op, right) => {
            let lv = eval_calc_with_context(left, ctx)?;
            let rv = eval_calc_with_context(right, ctx)?;
            match op {
                CalcOp::Add => Some(lv + rv),
                CalcOp::Subtract => Some(lv - rv),
                CalcOp::Multiply => Some(lv * rv),
                CalcOp::Divide => {
                    if rv == 0.0 {
                        None
                    } else {
                        Some(lv / rv)
                    }
                }
            }
        }
        CalcExpr::Min(args) => {
            let vals: Vec<f64> = args.iter().filter_map(|a| eval_calc_with_context(a, ctx)).collect();
            if vals.is_empty() {
                None
            } else {
                Some(vals.into_iter().reduce(f64::min).unwrap())
            }
        }
        CalcExpr::Max(args) => {
            let vals: Vec<f64> = args.iter().filter_map(|a| eval_calc_with_context(a, ctx)).collect();
            if vals.is_empty() {
                None
            } else {
                Some(vals.into_iter().reduce(f64::max).unwrap())
            }
        }
        CalcExpr::Clamp { min, val, max } => {
            let min_v = eval_calc_with_context(min, ctx)?;
            let val_v = eval_calc_with_context(val, ctx)?;
            let max_v = eval_calc_with_context(max, ctx)?;
            Some(val_v.clamp(min_v, max_v))
        }
    }
}

/// 将长度值解析为像素值。
///
/// 使用 [`CalcContext`] 中提供的参考尺寸转换相对单位。
fn resolve_length_to_px(lv: &LengthValue, ctx: &CalcContext) -> Option<f64> {
    match lv {
        LengthValue::Px(v) => Some(*v),
        LengthValue::Percentage(pct) => ctx.parent_length.map(|pl| pct / 100.0 * pl),
        LengthValue::Em(v) => ctx.font_size.map(|fs| v * fs),
        LengthValue::Rem(v) => ctx.root_font_size.map(|rfs| v * rfs),
        LengthValue::Vh(v) => ctx.viewport_height.map(|vh| v * vh / 100.0),
        LengthValue::Vw(v) => ctx.viewport_width.map(|vw| v * vw / 100.0),
        LengthValue::Vmin(v) => match (ctx.viewport_width, ctx.viewport_height) {
            (Some(vw), Some(vh)) => Some(v * vw.min(vh) / 100.0),
            _ => None,
        },
        LengthValue::Vmax(v) => match (ctx.viewport_width, ctx.viewport_height) {
            (Some(vw), Some(vh)) => Some(v * vw.max(vh) / 100.0),
            _ => None,
        },
        LengthValue::Ch(v) => ctx.ch_width.map(|cw| v * cw),
        LengthValue::Auto => None,
        LengthValue::Calc(expr) => eval_calc_with_context(expr, ctx),
        LengthValue::FitContent(inner) => resolve_length_to_px(inner, ctx),
        // min-content/max-content 需要内容信息才能计算，此处返回 None
        LengthValue::MinContent | LengthValue::MaxContent => None,
    }
}

// ── 解析函数 ────────────────────────────────────────────────────────

/// 解析 CSS 颜色值。
///
/// 支持命名颜色、十六进制颜色（#RGB、#RRGGBB、#RGBA、#RRGGBBAA）、
/// `rgb()`/`rgba()`、`hsl()`/`hsla()` 和 `hwb()` 函数。
pub fn parse_color(value: &str) -> Option<ColorValue> {
    let value = value.trim();

    // 特殊关键字
    if value.eq_ignore_ascii_case("transparent") {
        return Some(ColorValue::Transparent);
    }
    if value.eq_ignore_ascii_case("currentColor") || value == "currentcolor" {
        return Some(ColorValue::CurrentColor);
    }

    // 十六进制颜色
    if value.starts_with('#') {
        return parse_hex_color(value);
    }

    // rgb() / rgba() 函数
    if value.starts_with("rgb(") || value.starts_with("rgba(") {
        return parse_rgb_function(value);
    }

    // hsl() / hsla() 函数
    if value.starts_with("hsl(") || value.starts_with("hsla(") {
        return parse_hsl_function(value);
    }

    // hwb() 函数
    if value.starts_with("hwb(") {
        return parse_hwb_function(value);
    }

    // 命名颜色
    parse_named_color(value)
}

/// 解析十六进制颜色。
fn parse_hex_color(value: &str) -> Option<ColorValue> {
    let hex = &value[1..]; // 去掉 #
    match hex.len() {
        3 => {
            // #RGB → RRGGBB
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let r = hex_char_to_byte(c0, c0);
            let g = hex_char_to_byte(c1, c1);
            let b = hex_char_to_byte(c2, c2);
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        4 => {
            // #RGBA → RRGGBBAA
            let mut chars = hex.chars();
            let c0 = chars.next()?;
            let c1 = chars.next()?;
            let c2 = chars.next()?;
            let c3 = chars.next()?;
            let r = hex_char_to_byte(c0, c0);
            let g = hex_char_to_byte(c1, c1);
            let b = hex_char_to_byte(c2, c2);
            let a = hex_char_to_byte(c3, c3);
            Some(ColorValue::Rgba(r, g, b, a))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(ColorValue::Rgba(r, g, b, 255))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(ColorValue::Rgba(r, g, b, a))
        }
        _ => None,
    }
}

/// 将两个十六进制字符合并为一个字节（重复单字符，如 'f' → 0xFF）。
fn hex_char_to_byte(c1: char, c2: char) -> u8 {
    let s = format!("{}{}", c1, c2);
    u8::from_str_radix(&s, 16).unwrap_or(0)
}

/// 解析 rgb() / rgba() 函数。
fn parse_rgb_function(value: &str) -> Option<ColorValue> {
    // 提取括号内的内容
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let r = parse_color_component(parts[0].trim())?;
    let g = parse_color_component(parts[1].trim())?;
    let b = parse_color_component(parts[2].trim())?;
    let a = if parts.len() > 3 {
        parse_alpha_component(parts[3].trim())?
    } else {
        255u8
    };

    Some(ColorValue::Rgba(r, g, b, a))
}

/// 解析颜色分量（0-255 或 0%-100%）。
fn parse_color_component(s: &str) -> Option<u8> {
    if s.ends_with('%') {
        let pct: f64 = s.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some(v.round().clamp(0.0, 255.0) as u8)
    }
}

/// 解析 alpha 分量（0-1 或 0%-100%）。
fn parse_alpha_component(s: &str) -> Option<u8> {
    if s.ends_with('%') {
        let pct: f64 = s.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some((v * 255.0).round().clamp(0.0, 255.0) as u8)
    }
}

/// 解析 hsl() / hsla() 函数。
fn parse_hsl_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let h: f64 = parts[0].trim().trim_end_matches("deg").parse().ok()?;
    let s: f64 = parts[1].trim().trim_end_matches('%').parse().ok()?;
    let l: f64 = parts[2].trim().trim_end_matches('%').parse().ok()?;
    let a = if parts.len() > 3 {
        parts[3].trim().parse().ok()?
    } else {
        1.0
    };

    Some(ColorValue::Hsla(h, s, l, a))
}

/// 将 HWB 颜色转换为 RGBA。
///
/// 参数：
/// - `h`：色相（度），0-360
/// - `w`：白度（0-1 比例）
/// - `b`：黑度（0-1 比例）
/// - `a`：透明度（0-1 比例）
///
/// 如果 W+B > 1，两者按比例缩小使总和为 1。
pub fn hwb_to_rgba(h: f64, w: f64, b: f64, a: f64) -> (u8, u8, u8, u8) {
    // 钳制 W+B 到 100%
    let mut ww = w.clamp(0.0, 1.0);
    let mut bb = b.clamp(0.0, 1.0);
    if ww + bb > 1.0 {
        let scale = 1.0 / (ww + bb);
        ww *= scale;
        bb *= scale;
    }

    // 先将 HWB 转为 HSL 再转为 RGB
    // HWB → RGB 标准算法：
    // 先算出没有白度/黑度影响的纯色 RGB，再与白/黑混合
    let h_norm = (h % 360.0) / 60.0;
    let sector = h_norm.floor() as i32;
    let f = h_norm - sector as f64;

    // 6 个扇区的纯色分量
    let (r_pure, g_pure, b_pure) = match sector % 6 {
        0 => (1.0, f, 0.0),
        1 => (1.0 - f, 1.0, 0.0),
        2 => (0.0, 1.0, f),
        3 => (0.0, 1.0 - f, 1.0),
        4 => (f, 0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - f),
    };

    // 混合：result = color * (1 - W - B) + W
    let factor = 1.0 - ww - bb;
    let r = (r_pure * factor + ww).clamp(0.0, 1.0);
    let g = (g_pure * factor + ww).clamp(0.0, 1.0);
    let bv = (b_pure * factor + ww).clamp(0.0, 1.0);

    let alpha = a.clamp(0.0, 1.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (bv * 255.0).round() as u8,
        (alpha * 255.0).round() as u8,
    )
}

/// 解析 hwb() 颜色函数。
///
/// 格式：`hwb(H W B)` 或 `hwb(H W B / A)`，其中 H 为色相（数字），
/// W 为白度（百分比），B 为黑度（百分比），A 为可选的透明度。
fn parse_hwb_function(value: &str) -> Option<ColorValue> {
    let start = value.find('(')?;
    let end = value.rfind(')')?;
    let inner = value.get(start + 1..end)?.trim();

    // 检查是否有斜杠分隔的 alpha
    let slash_pos = inner.find('/');
    let main_part = match slash_pos {
        Some(pos) => inner[..pos].trim(),
        None => inner,
    };
    let alpha_str = slash_pos.map(|pos| inner[pos + 1..].trim());

    // 按空格分割：H W B
    let parts: Vec<&str> = main_part.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let h: f64 = parts[0].trim_end_matches("deg").parse().ok()?;
    let w_pct: f64 = parts[1].trim_end_matches('%').parse().ok()?;
    let b_pct: f64 = parts[2].trim_end_matches('%').parse().ok()?;
    let w = w_pct / 100.0;
    let b = b_pct / 100.0;
    let a = if let Some(a_str) = alpha_str {
        if a_str.ends_with('%') {
            a_str.trim_end_matches('%').parse::<f64>().ok()? / 100.0
        } else {
            a_str.parse::<f64>().ok()?
        }
    } else {
        1.0
    };

    let (r, g, bv, av) = hwb_to_rgba(h, w, b, a);
    Some(ColorValue::Rgba(r, g, bv, av))
}

/// 解析命名颜色。
///
/// 支持全部 148 种 CSS 标准命名颜色。
fn parse_named_color(value: &str) -> Option<ColorValue> {
    let lower = value.to_ascii_lowercase();
    let rgba = |r: u8, g: u8, b: u8| Some(ColorValue::Rgba(r, g, b, 255));
    match lower.as_str() {
        // CSS 基础 16 色
        "black" => rgba(0, 0, 0),
        "white" => rgba(255, 255, 255),
        "red" => rgba(255, 0, 0),
        "green" => rgba(0, 128, 0),
        "blue" => rgba(0, 0, 255),
        "yellow" => rgba(255, 255, 0),
        "cyan" | "aqua" => rgba(0, 255, 255),
        "magenta" | "fuchsia" => rgba(255, 0, 255),
        "silver" => rgba(192, 192, 192),
        "gray" | "grey" => rgba(128, 128, 128),
        "maroon" => rgba(128, 0, 0),
        "olive" => rgba(128, 128, 0),
        "lime" => rgba(0, 255, 0),
        "teal" => rgba(0, 128, 128),
        "navy" => rgba(0, 0, 128),
        "purple" => rgba(128, 0, 128),
        "orange" => rgba(255, 165, 0),
        // 扩展命名颜色 (A-F)
        "aliceblue" => rgba(240, 248, 255),
        "antiquewhite" => rgba(250, 235, 215),
        "aquamarine" => rgba(127, 255, 212),
        "azure" => rgba(240, 255, 255),
        "beige" => rgba(245, 245, 220),
        "bisque" => rgba(255, 228, 196),
        "blanchedalmond" => rgba(255, 235, 205),
        "burlywood" => rgba(222, 184, 135),
        "cadetblue" => rgba(95, 158, 160),
        "chartreuse" => rgba(127, 255, 0),
        "chocolate" => rgba(210, 105, 30),
        "coral" => rgba(255, 127, 80),
        "cornflowerblue" => rgba(100, 149, 237),
        "cornsilk" => rgba(255, 248, 220),
        "crimson" => rgba(220, 20, 60),
        "darkblue" => rgba(0, 0, 139),
        "darkcyan" => rgba(0, 139, 139),
        "darkgoldenrod" => rgba(184, 134, 11),
        "darkgray" | "darkgrey" => rgba(169, 169, 169),
        "darkgreen" => rgba(0, 100, 0),
        "darkkhaki" => rgba(189, 183, 107),
        "darkmagenta" => rgba(139, 0, 139),
        "darkolivegreen" => rgba(85, 107, 47),
        "darkorange" => rgba(255, 140, 0),
        "darkorchid" => rgba(153, 50, 204),
        "darkred" => rgba(139, 0, 0),
        "darksalmon" => rgba(233, 150, 122),
        "darkseagreen" => rgba(143, 188, 143),
        "darkslateblue" => rgba(72, 61, 139),
        "darkslategray" | "darkslategrey" => rgba(47, 79, 79),
        "darkturquoise" => rgba(0, 206, 209),
        "darkviolet" => rgba(148, 0, 211),
        "deeppink" => rgba(255, 20, 147),
        "deepskyblue" => rgba(0, 191, 255),
        "dimgray" | "dimgrey" => rgba(105, 105, 105),
        "dodgerblue" => rgba(30, 144, 255),
        "firebrick" => rgba(178, 34, 34),
        "floralwhite" => rgba(255, 250, 240),
        "forestgreen" => rgba(34, 139, 34),
        // G-L
        "gainsboro" => rgba(220, 220, 220),
        "ghostwhite" => rgba(248, 248, 255),
        "gold" => rgba(255, 215, 0),
        "goldenrod" => rgba(218, 165, 32),
        "greenyellow" => rgba(173, 255, 47),
        "honeydew" => rgba(240, 255, 240),
        "hotpink" => rgba(255, 105, 180),
        "indianred" => rgba(205, 92, 92),
        "indigo" => rgba(75, 0, 130),
        "ivory" => rgba(255, 255, 240),
        "khaki" => rgba(240, 230, 140),
        "lavender" => rgba(230, 230, 250),
        "lavenderblush" => rgba(255, 240, 245),
        "lawngreen" => rgba(124, 252, 0),
        "lemonchiffon" => rgba(255, 250, 205),
        "lightblue" => rgba(173, 216, 230),
        "lightcoral" => rgba(240, 128, 128),
        "lightcyan" => rgba(224, 255, 255),
        "lightgoldenrodyellow" => rgba(250, 250, 210),
        "lightgray" | "lightgrey" => rgba(211, 211, 211),
        "lightgreen" => rgba(144, 238, 144),
        "lightpink" => rgba(255, 182, 193),
        "lightsalmon" => rgba(255, 160, 122),
        "lightseagreen" => rgba(32, 178, 170),
        "lightskyblue" => rgba(135, 206, 250),
        "lightslategray" | "lightslategrey" => rgba(119, 136, 153),
        "lightsteelblue" => rgba(176, 196, 222),
        "lightyellow" => rgba(255, 255, 224),
        "limegreen" => rgba(50, 205, 50),
        "linen" => rgba(250, 240, 230),
        // M-P
        "mediumaquamarine" => rgba(102, 205, 170),
        "mediumblue" => rgba(0, 0, 205),
        "mediumorchid" => rgba(186, 85, 211),
        "mediumpurple" => rgba(147, 112, 219),
        "mediumseagreen" => rgba(60, 179, 113),
        "mediumslateblue" => rgba(123, 104, 238),
        "mediumspringgreen" => rgba(0, 250, 154),
        "mediumturquoise" => rgba(72, 209, 204),
        "mediumvioletred" => rgba(199, 21, 133),
        "midnightblue" => rgba(25, 25, 112),
        "mintcream" => rgba(245, 255, 250),
        "mistyrose" => rgba(255, 228, 225),
        "moccasin" => rgba(255, 228, 181),
        "navajowhite" => rgba(255, 222, 173),
        "oldlace" => rgba(253, 245, 230),
        "olivedrab" => rgba(107, 142, 35),
        "orangered" => rgba(255, 69, 0),
        "orchid" => rgba(218, 112, 214),
        "palegoldenrod" => rgba(238, 232, 170),
        "palegreen" => rgba(152, 251, 152),
        "paleturquoise" => rgba(175, 238, 238),
        "palevioletred" => rgba(219, 112, 147),
        "papayawhip" => rgba(255, 239, 213),
        "peachpuff" => rgba(255, 218, 185),
        "peru" => rgba(205, 133, 63),
        "pink" => rgba(255, 192, 203),
        "plum" => rgba(221, 160, 221),
        "powderblue" => rgba(176, 224, 230),
        // R-T
        "rosybrown" => rgba(188, 143, 143),
        "royalblue" => rgba(65, 105, 225),
        "saddlebrown" => rgba(139, 69, 19),
        "salmon" => rgba(250, 128, 114),
        "sandybrown" => rgba(244, 164, 96),
        "seagreen" => rgba(46, 139, 87),
        "seashell" => rgba(255, 245, 238),
        "sienna" => rgba(160, 82, 45),
        "skyblue" => rgba(135, 206, 235),
        "slateblue" => rgba(106, 90, 205),
        "slategray" | "slategrey" => rgba(112, 128, 144),
        "snow" => rgba(255, 250, 250),
        "springgreen" => rgba(0, 255, 127),
        "steelblue" => rgba(70, 130, 180),
        "tan" => rgba(210, 180, 140),
        "thistle" => rgba(216, 191, 216),
        "tomato" => rgba(255, 99, 71),
        "turquoise" => rgba(64, 224, 208),
        // U-Z
        "violet" => rgba(238, 130, 238),
        "wheat" => rgba(245, 222, 179),
        "whitesmoke" => rgba(245, 245, 245),
        "yellowgreen" => rgba(154, 205, 50),
        // transparent 和 currentColor 由 parse_color_value 直接处理
        "transparent" => Some(ColorValue::Transparent),
        "currentcolor" => Some(ColorValue::CurrentColor),
        _ => None,
    }
}

/// 解析 CSS 长度值。
///
/// 支持格式如 `"10px"`、`"1.5em"`、`"2rem"`、`"100vh"`、`"50%"`、`"auto"`、
/// `"fit-content(200px)"` 等。
pub fn parse_length(value: &str) -> Option<LengthValue> {
    let value = value.trim();

    // 处理 auto 关键字
    if value.eq_ignore_ascii_case("auto") {
        return Some(LengthValue::Auto);
    }

    // 处理 min-content/max-content 关键字
    if value.eq_ignore_ascii_case("min-content") {
        return Some(LengthValue::MinContent);
    }
    if value.eq_ignore_ascii_case("max-content") {
        return Some(LengthValue::MaxContent);
    }

    // 处理 fit-content() 函数
    if value.starts_with("fit-content(") && value.ends_with(')') {
        let inner = &value["fit-content(".len()..value.len() - 1];
        let inner = inner.trim();
        // fit-content() 不接受空参数
        if inner.is_empty() {
            return None;
        }
        let arg = parse_length(inner)?;
        return Some(LengthValue::FitContent(Box::new(arg)));
    }

    // 从字符串末尾扫描，找到单位部分的起始位置。
    // 单位部分由字母组成（可能以 '%' 结尾）；数字部分在单位之前。
    // 这样可以正确处理科学计数法（如 "1e2px"），因为 'e' 在数字部分内。
    let unit_start = find_unit_start(value);

    let num_str = &value[..unit_start];
    let unit = &value[unit_start..];

    let num: f64 = num_str.parse().ok()?;

    match unit {
        "px" => Some(LengthValue::Px(num)),
        "em" => Some(LengthValue::Em(num)),
        "rem" => Some(LengthValue::Rem(num)),
        "vh" => Some(LengthValue::Vh(num)),
        "vw" => Some(LengthValue::Vw(num)),
        "vmin" => Some(LengthValue::Vmin(num)),
        "vmax" => Some(LengthValue::Vmax(num)),
        "ch" => Some(LengthValue::Ch(num)),
        "%" => Some(LengthValue::Percentage(num)),
        // Per CSS spec, a bare zero without units is a valid length (0px).
        "" if num == 0.0 => Some(LengthValue::Px(0.0)),
        _ => None,
    }
}

/// 从字符串末尾找到单位部分的起始索引。
///
/// 从右向左扫描：跳过 '%'（如果有），然后跳过连续的字母字符，
/// 剩下的就是数字部分的结束位置。
fn find_unit_start(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = bytes.len();

    // 跳过末尾的 '%'
    if i > 0 && bytes[i - 1] == b'%' {
        i -= 1;
        return i;
    }

    // 从末尾向前跳过连续的 ASCII 字母（单位名）
    while i > 0 && bytes[i - 1].is_ascii_alphabetic() {
        i -= 1;
    }

    i
}

/// 解析 CSS display 属性值。
pub fn parse_display(value: &str) -> Option<DisplayValue> {
    match value.trim() {
        "block" => Some(DisplayValue::Block),
        "inline" => Some(DisplayValue::Inline),
        "inline-block" => Some(DisplayValue::InlineBlock),
        "flex" => Some(DisplayValue::Flex),
        "inline-flex" => Some(DisplayValue::InlineFlex),
        "grid" => Some(DisplayValue::Grid),
        "inline-grid" => Some(DisplayValue::InlineGrid),
        "none" => Some(DisplayValue::None),
        "contents" => Some(DisplayValue::Contents),
        "flow" => Some(DisplayValue::Flow),
        "flow-root" => Some(DisplayValue::FlowRoot),
        "list-item" => Some(DisplayValue::ListItem),
        _ => None,
    }
}

/// 解析 CSS position 属性值。
pub fn parse_position(value: &str) -> Option<PositionValue> {
    match value.trim() {
        "static" => Some(PositionValue::Static),
        "relative" => Some(PositionValue::Relative),
        "absolute" => Some(PositionValue::Absolute),
        "fixed" => Some(PositionValue::Fixed),
        "sticky" => Some(PositionValue::Sticky),
        _ => None,
    }
}

/// 解析 CSS overflow 属性值。
pub fn parse_overflow(value: &str) -> Option<OverflowValue> {
    match value.trim() {
        "visible" => Some(OverflowValue::Visible),
        "hidden" => Some(OverflowValue::Hidden),
        "scroll" => Some(OverflowValue::Scroll),
        "auto" => Some(OverflowValue::Auto),
        "clip" => Some(OverflowValue::Clip),
        _ => None,
    }
}

/// 解析 CSS float 属性值。
pub fn parse_float(value: &str) -> Option<FloatValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(FloatValue::None),
        "left" => Some(FloatValue::Left),
        "right" => Some(FloatValue::Right),
        "inline-start" => Some(FloatValue::InlineStart),
        "inline-end" => Some(FloatValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS clear 属性值。
pub fn parse_clear(value: &str) -> Option<ClearValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(ClearValue::None),
        "left" => Some(ClearValue::Left),
        "right" => Some(ClearValue::Right),
        "both" => Some(ClearValue::Both),
        "inline-start" => Some(ClearValue::InlineStart),
        "inline-end" => Some(ClearValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS list-style-type 属性值。
pub fn parse_list_style_type(value: &str) -> Option<ListStyleTypeValue> {
    match value.trim().to_lowercase().as_str() {
        "disc" => Some(ListStyleTypeValue::Disc),
        "circle" => Some(ListStyleTypeValue::Circle),
        "square" => Some(ListStyleTypeValue::Square),
        "decimal" => Some(ListStyleTypeValue::Decimal),
        "decimal-leading-zero" => Some(ListStyleTypeValue::DecimalLeadingZero),
        "lower-roman" => Some(ListStyleTypeValue::LowerRoman),
        "upper-roman" => Some(ListStyleTypeValue::UpperRoman),
        "lower-alpha" | "lower-latin" => Some(ListStyleTypeValue::LowerAlpha),
        "upper-alpha" | "upper-latin" => Some(ListStyleTypeValue::UpperAlpha),
        "none" => Some(ListStyleTypeValue::None),
        _ => None,
    }
}

/// 解析 CSS list-style-position 属性值。
pub fn parse_list_style_position(value: &str) -> Option<ListStylePositionValue> {
    match value.trim().to_lowercase().as_str() {
        "outside" => Some(ListStylePositionValue::Outside),
        "inside" => Some(ListStylePositionValue::Inside),
        _ => None,
    }
}

/// 解析 CSS flex-direction 属性值。
pub fn parse_flex_direction(value: &str) -> Option<FlexDirectionValue> {
    match value.trim() {
        "row" => Some(FlexDirectionValue::Row),
        "row-reverse" => Some(FlexDirectionValue::RowReverse),
        "column" => Some(FlexDirectionValue::Column),
        "column-reverse" => Some(FlexDirectionValue::ColumnReverse),
        _ => None,
    }
}

/// 解析 CSS flex-wrap 属性值。
pub fn parse_flex_wrap(value: &str) -> Option<FlexWrapValue> {
    match value.trim() {
        "nowrap" => Some(FlexWrapValue::Nowrap),
        "wrap" => Some(FlexWrapValue::Wrap),
        "wrap-reverse" => Some(FlexWrapValue::WrapReverse),
        _ => None,
    }
}

/// 解析 CSS justify-content / align-items 属性值。
pub fn parse_alignment(value: &str) -> Option<AlignmentValue> {
    match value.trim() {
        "flex-start" => Some(AlignmentValue::FlexStart),
        "flex-end" => Some(AlignmentValue::FlexEnd),
        "center" => Some(AlignmentValue::Center),
        "space-between" => Some(AlignmentValue::SpaceBetween),
        "space-around" => Some(AlignmentValue::SpaceAround),
        "space-evenly" => Some(AlignmentValue::SpaceEvenly),
        "stretch" => Some(AlignmentValue::Stretch),
        "start" => Some(AlignmentValue::Start),
        "end" => Some(AlignmentValue::End),
        "baseline" => Some(AlignmentValue::Baseline),
        _ => None,
    }
}

/// 解析 CSS box-sizing 属性值。
pub fn parse_box_sizing(value: &str) -> Option<BoxSizingValue> {
    match value.trim() {
        "content-box" => Some(BoxSizingValue::ContentBox),
        "border-box" => Some(BoxSizingValue::BorderBox),
        _ => None,
    }
}

/// 解析 CSS visibility 属性值。
pub fn parse_visibility(value: &str) -> Option<VisibilityValue> {
    match value.trim() {
        "visible" => Some(VisibilityValue::Visible),
        "hidden" => Some(VisibilityValue::Hidden),
        "collapse" => Some(VisibilityValue::Collapse),
        _ => None,
    }
}

/// 解析 CSS word-break 属性值。
pub fn parse_word_break(value: &str) -> Option<WordBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(WordBreakValue::Normal),
        "break-all" => Some(WordBreakValue::BreakAll),
        "keep-all" => Some(WordBreakValue::KeepAll),
        "break-word" => Some(WordBreakValue::BreakWord),
        _ => None,
    }
}

/// 解析 CSS writing-mode 属性值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        _ => None,
    }
}

/// 解析 CSS text-decoration-line 值。
pub fn parse_text_decoration_line(value: &str) -> Option<TextDecorationLineValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextDecorationLineValue::None),
        "underline" => Some(TextDecorationLineValue::Underline),
        "overline" => Some(TextDecorationLineValue::Overline),
        "line-through" => Some(TextDecorationLineValue::LineThrough),
        "blink" => Some(TextDecorationLineValue::Blink),
        _ => None,
    }
}

/// 解析 CSS text-transform 值。
pub fn parse_text_transform(value: &str) -> Option<TextTransformValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextTransformValue::None),
        "uppercase" => Some(TextTransformValue::Uppercase),
        "lowercase" => Some(TextTransformValue::Lowercase),
        "capitalize" => Some(TextTransformValue::Capitalize),
        _ => None,
    }
}

/// 解析 CSS text-indent 属性值。
///
/// 支持长度值（如 `2em`、`20px`）和百分比值（如 `10%`）。
/// 不支持 `auto` 关键字。
pub fn parse_text_indent(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_length(v)
}

/// 解析 CSS letter-spacing / word-spacing 值。
/// "normal" 映射为 LengthValue::Px(0.0)。
pub fn parse_spacing(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("normal") {
        return Some(LengthValue::Px(0.0));
    }
    parse_length(v)
}

/// 解析 CSS font-weight 属性值。
pub fn parse_font_weight(value: &str) -> Option<FontWeightValue> {
    match value.trim() {
        "bold" => Some(FontWeightValue::Bold),
        "normal" => Some(FontWeightValue::Normal),
        "bolder" => Some(FontWeightValue::Bolder),
        "lighter" => Some(FontWeightValue::Lighter),
        s => {
            let w: u16 = s.parse().ok()?;
            if (100..=900).contains(&w) {
                Some(FontWeightValue::Absolute(w))
            } else {
                None
            }
        }
    }
}

/// 解析 CSS font-style 属性值。
pub fn parse_font_style(value: &str) -> Option<FontStyleValue> {
    let value = value.trim();
    if value == "normal" {
        Some(FontStyleValue::Normal)
    } else if value == "italic" {
        Some(FontStyleValue::Italic)
    } else if value.starts_with("oblique") {
        let angle_str = value.strip_prefix("oblique")?.trim();
        if angle_str.is_empty() {
            Some(FontStyleValue::Oblique(None))
        } else {
            // 处理 "(angle)" 或 "(angledeg)" 形式
            let angle_str = angle_str
                .strip_prefix('(')
                .unwrap_or(angle_str)
                .strip_suffix(')')
                .unwrap_or(angle_str);
            let angle: f64 = angle_str.trim_end_matches("deg").trim().parse().ok()?;
            Some(FontStyleValue::Oblique(Some(angle)))
        }
    } else {
        None
    }
}

// ── CSS Scroll Snap 值类型 ──────────────────────────────────────────

/// CSS scroll-snap-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapTypeValue {
    /// none。
    None,
    /// mandatory（必须吸附）。
    Mandatory,
    /// proximity（接近时吸附）。
    Proximity,
}

/// CSS scroll-snap-type 轴。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAxis {
    /// x 轴。
    X,
    /// y 轴。
    Y,
    /// 两个轴。
    Both,
}

/// CSS scroll-snap-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAlignValue {
    /// none。
    None,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
}

/// CSS scroll-snap-stop 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapStopValue {
    /// normal。
    Normal,
    /// always。
    Always,
}

/// CSS container-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerTypeValue {
    /// normal。
    Normal,
    /// size。
    Size,
    /// inline-size。
    InlineSize,
}

/// 解析 CSS scroll-snap-type 属性值。
///
/// 支持格式如 `"none"`、`"x mandatory"`、`"y proximity"`、`"both mandatory"`。
/// 返回 (strictness, axis) 元组。
pub fn parse_scroll_snap_type(value: &str) -> Option<(ScrollSnapTypeValue, Option<ScrollSnapAxis>)> {
    let value = value.trim().to_ascii_lowercase();

    if value == "none" {
        return Some((ScrollSnapTypeValue::None, None));
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut strictness = None;
    let mut axis = None;

    for part in parts {
        match part {
            "mandatory" => strictness = Some(ScrollSnapTypeValue::Mandatory),
            "proximity" => strictness = Some(ScrollSnapTypeValue::Proximity),
            "x" => axis = Some(ScrollSnapAxis::X),
            "y" => axis = Some(ScrollSnapAxis::Y),
            "both" => axis = Some(ScrollSnapAxis::Both),
            _ => return None,
        }
    }

    strictness.map(|s| (s, axis))
}

/// 解析 CSS scroll-snap-align 属性值。
pub fn parse_scroll_snap_align(value: &str) -> Option<ScrollSnapAlignValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ScrollSnapAlignValue::None),
        "start" => Some(ScrollSnapAlignValue::Start),
        "end" => Some(ScrollSnapAlignValue::End),
        "center" => Some(ScrollSnapAlignValue::Center),
        _ => None,
    }
}

/// 解析 CSS scroll-snap-stop 属性值。
pub fn parse_scroll_snap_stop(value: &str) -> Option<ScrollSnapStopValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(ScrollSnapStopValue::Normal),
        "always" => Some(ScrollSnapStopValue::Always),
        _ => None,
    }
}

/// 解析 CSS container-type 属性值。
pub fn parse_container_type(value: &str) -> Option<ContainerTypeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(ContainerTypeValue::Normal),
        "size" => Some(ContainerTypeValue::Size),
        "inline-size" => Some(ContainerTypeValue::InlineSize),
        _ => None,
    }
}

/// CSS vertical-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum VerticalAlignValue {
    /// baseline（默认值）— 元素基线与父元素基线对齐。
    Baseline,
    /// top — 元素顶部与行盒顶部对齐。
    Top,
    /// middle — 元素中部与父元素基线 + 半 x-height 处对齐。
    Middle,
    /// bottom — 元素底部与行盒底部对齐。
    Bottom,
    /// text-top — 元素顶部与父元素字体的顶部对齐。
    TextTop,
    /// text-bottom — 元素底部与父元素字体的底部对齐。
    TextBottom,
    /// sub — 元素基线下移至适合下标的位置。
    Sub,
    /// super — 元素基线上移至适合上标的位置。
    Super,
}

/// CSS cursor 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CursorValue {
    /// auto。
    Auto,
    /// default。
    Default,
    /// pointer。
    Pointer,
    /// move。
    Move,
    /// text。
    Text,
    /// wait。
    Wait,
    /// crosshair。
    Crosshair,
    /// not-allowed。
    NotAllowed,
    /// grab。
    Grab,
    /// grabbing。
    Grabbing,
    /// help。
    Help,
    /// progress。
    Progress,
    /// n-resize。
    NResize,
    /// s-resize。
    SResize,
    /// e-resize。
    EResize,
    /// w-resize。
    WResize,
    /// ne-resize。
    NeResize,
    /// nw-resize。
    NwResize,
    /// se-resize。
    SeResize,
    /// sw-resize。
    SwResize,
    /// col-resize。
    ColResize,
    /// row-resize。
    RowResize,
    /// all-scroll。
    AllScroll,
    /// zoom-in。
    ZoomIn,
    /// zoom-out。
    ZoomOut,
    /// none。
    None,
}

/// 解析 CSS cursor 属性值。
pub fn parse_cursor(value: &str) -> Option<CursorValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(CursorValue::Auto),
        "default" => Some(CursorValue::Default),
        "pointer" => Some(CursorValue::Pointer),
        "move" => Some(CursorValue::Move),
        "text" => Some(CursorValue::Text),
        "wait" => Some(CursorValue::Wait),
        "crosshair" => Some(CursorValue::Crosshair),
        "not-allowed" => Some(CursorValue::NotAllowed),
        "grab" => Some(CursorValue::Grab),
        "grabbing" => Some(CursorValue::Grabbing),
        "help" => Some(CursorValue::Help),
        "progress" => Some(CursorValue::Progress),
        "n-resize" => Some(CursorValue::NResize),
        "s-resize" => Some(CursorValue::SResize),
        "e-resize" => Some(CursorValue::EResize),
        "w-resize" => Some(CursorValue::WResize),
        "ne-resize" => Some(CursorValue::NeResize),
        "nw-resize" => Some(CursorValue::NwResize),
        "se-resize" => Some(CursorValue::SeResize),
        "sw-resize" => Some(CursorValue::SwResize),
        "col-resize" => Some(CursorValue::ColResize),
        "row-resize" => Some(CursorValue::RowResize),
        "all-scroll" => Some(CursorValue::AllScroll),
        "zoom-in" => Some(CursorValue::ZoomIn),
        "zoom-out" => Some(CursorValue::ZoomOut),
        "none" => Some(CursorValue::None),
        _ => None,
    }
}

/// 解析 CSS opacity 属性值。
///
/// 支持数值（0.0-1.0）和百分比（如 `50%` → 0.5）。
/// 结果限制在 [0.0, 1.0] 范围内。
pub fn parse_opacity(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.ends_with('%') {
        let pct: f64 = value.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0).clamp(0.0, 1.0))
    } else {
        let num: f64 = value.parse().ok()?;
        Some(num.clamp(0.0, 1.0))
    }
}

/// 解析 CSS vertical-align 属性值。
pub fn parse_vertical_align(value: &str) -> Option<VerticalAlignValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(VerticalAlignValue::Baseline),
        "top" => Some(VerticalAlignValue::Top),
        "middle" => Some(VerticalAlignValue::Middle),
        "bottom" => Some(VerticalAlignValue::Bottom),
        "text-top" => Some(VerticalAlignValue::TextTop),
        "text-bottom" => Some(VerticalAlignValue::TextBottom),
        "sub" => Some(VerticalAlignValue::Sub),
        "super" => Some(VerticalAlignValue::Super),
        _ => None,
    }
}

/// 解析 1-4 个长度值的简写属性（如 scroll-margin、scroll-padding）。
///
/// 返回 [top, right, bottom, left]（按 CSS 简写规则展开）。
pub fn parse_length_shorthand(value: &str) -> Option<[LengthValue; 4]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            let v = parse_length(parts[0])?;
            Some([v.clone(), v.clone(), v.clone(), v])
        }
        2 => {
            let tb = parse_length(parts[0])?;
            let lr = parse_length(parts[1])?;
            Some([tb.clone(), lr.clone(), tb, lr])
        }
        3 => {
            let top = parse_length(parts[0])?;
            let lr = parse_length(parts[1])?;
            let bottom = parse_length(parts[2])?;
            Some([top, lr.clone(), bottom, lr])
        }
        4 => {
            let top = parse_length(parts[0])?;
            let right = parse_length(parts[1])?;
            let bottom = parse_length(parts[2])?;
            let left = parse_length(parts[3])?;
            Some([top, right, bottom, left])
        }
        _ => None,
    }
}

/// 解析 CSS var() 函数引用。
///
/// 支持格式如 `var(--name)` 和 `var(--name, fallback)`。
pub fn parse_var(value: &str) -> Option<VarReference> {
    let value = value.trim();

    // 检查是否以 var( 开头
    if !value.starts_with("var(") || !value.ends_with(')') {
        return None;
    }

    // 提取括号内的内容
    let inner = value.get(4..value.len() - 1)?.trim();

    // 找到逗号（如果有）
    if let Some(comma_pos) = inner.find(',') {
        let name = inner[..comma_pos].trim().to_string();
        let fallback = inner[comma_pos + 1..].trim().to_string();
        Some(VarReference {
            name,
            fallback: Some(fallback),
        })
    } else {
        Some(VarReference {
            name: inner.to_string(),
            fallback: None,
        })
    }
}

// ── CSS Page Break 值类型 ──────────────────────────────────────────────

/// CSS page-break 属性值（page-break-before、page-break-after、page-break-inside）。
#[derive(Debug, Clone, PartialEq)]
pub enum PageBreakValue {
    /// auto。
    Auto,
    /// always。
    Always,
    /// avoid。
    Avoid,
    /// left。
    Left,
    /// right。
    Right,
}

/// 解析 CSS page-break 属性值。
pub fn parse_page_break(value: &str) -> Option<PageBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(PageBreakValue::Auto),
        "always" => Some(PageBreakValue::Always),
        "avoid" => Some(PageBreakValue::Avoid),
        "left" => Some(PageBreakValue::Left),
        "right" => Some(PageBreakValue::Right),
        _ => None,
    }
}

/// CSS box-decoration-break 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BoxDecorationBreakValue {
    /// slice。
    Slice,
    /// clone。
    Clone,
}

/// 解析 CSS box-decoration-break 属性值。
pub fn parse_box_decoration_break(value: &str) -> Option<BoxDecorationBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "slice" => Some(BoxDecorationBreakValue::Slice),
        "clone" => Some(BoxDecorationBreakValue::Clone),
        _ => None,
    }
}

/// CSS image-rendering 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ImageRenderingValue {
    /// auto。
    Auto,
    /// smooth。
    Smooth,
    /// high-quality。
    HighQuality,
    /// pixelated。
    Pixelated,
    /// crisp-edges。
    CrispEdges,
}

/// 解析 CSS image-rendering 属性值。
pub fn parse_image_rendering(value: &str) -> Option<ImageRenderingValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(ImageRenderingValue::Auto),
        "smooth" => Some(ImageRenderingValue::Smooth),
        "high-quality" => Some(ImageRenderingValue::HighQuality),
        "pixelated" => Some(ImageRenderingValue::Pixelated),
        "crisp-edges" => Some(ImageRenderingValue::CrispEdges),
        _ => None,
    }
}

/// CSS isolation 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationValue {
    /// auto。
    Auto,
    /// isolate。
    Isolate,
}

/// 解析 CSS isolation 属性值。
pub fn parse_isolation(value: &str) -> Option<IsolationValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(IsolationValue::Auto),
        "isolate" => Some(IsolationValue::Isolate),
        _ => None,
    }
}

/// CSS break-inside 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakInsideValue {
    /// auto。
    Auto,
    /// avoid。
    Avoid,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS break-before / break-after 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakValue {
    /// auto。
    Auto,
    /// avoid。
    Avoid,
    /// column。
    Column,
    /// page。
    Page,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS column-rule-width 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleWidthValue {
    /// medium。
    Medium,
    /// thin。
    Thin,
    /// thick。
    Thick,
    /// 长度值。
    Length(LengthValue),
}

/// CSS column-rule-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleStyleValue {
    /// none。
    None,
    /// hidden。
    Hidden,
    /// dotted。
    Dotted,
    /// dashed。
    Dashed,
    /// solid。
    Solid,
    /// double。
    Double,
    /// groove。
    Groove,
    /// ridge。
    Ridge,
    /// inset。
    Inset,
    /// outset。
    Outset,
}

/// 解析 CSS break-inside 属性值。
pub fn parse_break_inside(value: &str) -> Option<BreakInsideValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakInsideValue::Auto),
        "avoid" => Some(BreakInsideValue::Avoid),
        "avoid-page" => Some(BreakInsideValue::AvoidPage),
        "avoid-column" => Some(BreakInsideValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS break-before 属性值。
pub fn parse_break_before(value: &str) -> Option<BreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakValue::Auto),
        "avoid" => Some(BreakValue::Avoid),
        "column" => Some(BreakValue::Column),
        "page" => Some(BreakValue::Page),
        "avoid-page" => Some(BreakValue::AvoidPage),
        "avoid-column" => Some(BreakValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS break-after 属性值。
pub fn parse_break_after(value: &str) -> Option<BreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakValue::Auto),
        "avoid" => Some(BreakValue::Avoid),
        "column" => Some(BreakValue::Column),
        "page" => Some(BreakValue::Page),
        "avoid-page" => Some(BreakValue::AvoidPage),
        "avoid-column" => Some(BreakValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS column-rule-width 属性值。
pub fn parse_column_rule_width(value: &str) -> Option<ColumnRuleWidthValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "medium" => Some(ColumnRuleWidthValue::Medium),
        "thin" => Some(ColumnRuleWidthValue::Thin),
        "thick" => Some(ColumnRuleWidthValue::Thick),
        _ => parse_length(&v).map(ColumnRuleWidthValue::Length),
    }
}

/// 解析 CSS column-rule-style 属性值。
pub fn parse_column_rule_style(value: &str) -> Option<ColumnRuleStyleValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ColumnRuleStyleValue::None),
        "hidden" => Some(ColumnRuleStyleValue::Hidden),
        "dotted" => Some(ColumnRuleStyleValue::Dotted),
        "dashed" => Some(ColumnRuleStyleValue::Dashed),
        "solid" => Some(ColumnRuleStyleValue::Solid),
        "double" => Some(ColumnRuleStyleValue::Double),
        "groove" => Some(ColumnRuleStyleValue::Groove),
        "ridge" => Some(ColumnRuleStyleValue::Ridge),
        "inset" => Some(ColumnRuleStyleValue::Inset),
        "outset" => Some(ColumnRuleStyleValue::Outset),
        _ => None,
    }
}

/// CSS direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum DirectionValue {
    /// ltr（默认值）— 从左到右。
    Ltr,
    /// rtl — 从右到左。
    Rtl,
}

/// 解析 CSS direction 属性值。
pub fn parse_direction(value: &str) -> Option<DirectionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ltr" => Some(DirectionValue::Ltr),
        "rtl" => Some(DirectionValue::Rtl),
        _ => None,
    }
}

/// CSS unicode-bidi 值。
#[derive(Debug, Clone, PartialEq)]
pub enum UnicodeBidiValue {
    /// normal（默认值）。
    Normal,
    /// embed。
    Embed,
    /// isolate。
    Isolate,
    /// bidi-override。
    BidiOverride,
    /// isolate-override。
    IsolateOverride,
    /// plaintext。
    Plaintext,
}

/// 解析 CSS unicode-bidi 属性值。
pub fn parse_unicode_bidi(value: &str) -> Option<UnicodeBidiValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(UnicodeBidiValue::Normal),
        "embed" => Some(UnicodeBidiValue::Embed),
        "isolate" => Some(UnicodeBidiValue::Isolate),
        "bidi-override" => Some(UnicodeBidiValue::BidiOverride),
        "isolate-override" => Some(UnicodeBidiValue::IsolateOverride),
        "plaintext" => Some(UnicodeBidiValue::Plaintext),
        _ => None,
    }
}

/// CSS tab-size 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TabSizeValue {
    /// 数字值（空格数）。
    Number(u32),
    /// 长度值（如 px、em）。
    Length(LengthValue),
}

/// 解析 CSS tab-size 属性值。
///
/// 支持整数（如 `4`）和长度值（如 `20px`、`1em`）。
pub fn parse_tab_size(value: &str) -> Option<TabSizeValue> {
    let value = value.trim();
    // 先尝试解析为整数
    if let Ok(n) = value.parse::<u32>() {
        return Some(TabSizeValue::Number(n));
    }
    // 再尝试解析为长度值
    parse_length(value).map(TabSizeValue::Length)
}

/// CSS overflow-wrap 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowWrapValue {
    /// normal。
    Normal,
    /// break-word。
    BreakWord,
    /// anywhere。
    Anywhere,
}

/// 解析 CSS overflow-wrap 属性值。
pub fn parse_overflow_wrap(value: &str) -> Option<OverflowWrapValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(OverflowWrapValue::Normal),
        "break-word" => Some(OverflowWrapValue::BreakWord),
        "anywhere" => Some(OverflowWrapValue::Anywhere),
        _ => None,
    }
}

/// CSS text-align-last 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlignLastValue {
    /// auto。
    Auto,
    /// start。
    Start,
    /// end。
    End,
    /// left。
    Left,
    /// right。
    Right,
    /// center。
    Center,
    /// justify。
    Justify,
}

/// 解析 CSS text-align-last 属性值。
pub fn parse_text_align_last(value: &str) -> Option<TextAlignLastValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(TextAlignLastValue::Auto),
        "start" => Some(TextAlignLastValue::Start),
        "end" => Some(TextAlignLastValue::End),
        "left" => Some(TextAlignLastValue::Left),
        "right" => Some(TextAlignLastValue::Right),
        "center" => Some(TextAlignLastValue::Center),
        "justify" => Some(TextAlignLastValue::Justify),
        _ => None,
    }
}

/// CSS font-variant-numeric 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontVariantNumericValue {
    /// normal。
    Normal,
    /// ordinal。
    Ordinal,
    /// slashed-zero。
    SlashedZero,
    /// lining-nums。
    LiningNums,
    /// oldstyle-nums。
    OldstyleNums,
    /// proportional-nums。
    ProportionalNums,
    /// tabular-nums。
    TabularNums,
    /// diagonal-fractions。
    DiagonalFractions,
    /// stacked-fractions。
    StackedFractions,
}

/// 解析 CSS font-variant-numeric 属性值。
pub fn parse_font_variant_numeric(value: &str) -> Option<FontVariantNumericValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(FontVariantNumericValue::Normal),
        "ordinal" => Some(FontVariantNumericValue::Ordinal),
        "slashed-zero" => Some(FontVariantNumericValue::SlashedZero),
        "lining-nums" => Some(FontVariantNumericValue::LiningNums),
        "oldstyle-nums" => Some(FontVariantNumericValue::OldstyleNums),
        "proportional-nums" => Some(FontVariantNumericValue::ProportionalNums),
        "tabular-nums" => Some(FontVariantNumericValue::TabularNums),
        "diagonal-fractions" => Some(FontVariantNumericValue::DiagonalFractions),
        "stacked-fractions" => Some(FontVariantNumericValue::StackedFractions),
        _ => None,
    }
}

// ── CSS Transition 值类型 ──────────────────────────────────────────────

/// CSS transition-timing-function / animation-timing-function 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunctionValue {
    /// ease。
    Ease,
    /// linear。
    Linear,
    /// ease-in。
    EaseIn,
    /// ease-out。
    EaseOut,
    /// ease-in-out。
    EaseInOut,
    /// cubic-bezier(x1, y1, x2, y2)。
    CubicBezier(f64, f64, f64, f64),
    /// step-start。
    StepStart,
    /// step-end。
    StepEnd,
    /// steps(n, position)。
    Steps(i32, Option<StepPosition>),
}

/// steps() 的位置参数。
#[derive(Debug, Clone, PartialEq)]
pub enum StepPosition {
    /// jump-start / start。
    Start,
    /// jump-end / end（默认）。
    End,
    /// jump-both。
    Both,
    /// jump-none。
    None,
}

/// CSS animation-direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationDirectionValue {
    /// normal。
    Normal,
    /// reverse。
    Reverse,
    /// alternate。
    Alternate,
    /// alternate-reverse。
    AlternateReverse,
}

/// CSS animation-fill-mode 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationFillModeValue {
    /// none。
    None,
    /// forwards。
    Forwards,
    /// backwards。
    Backwards,
    /// both。
    Both,
}

/// CSS animation-play-state 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPlayStateValue {
    /// running。
    Running,
    /// paused。
    Paused,
}

/// 解析 CSS animation-direction 值。
pub fn parse_animation_direction(value: &str) -> Option<AnimationDirectionValue> {
    match value.trim() {
        "normal" => Some(AnimationDirectionValue::Normal),
        "reverse" => Some(AnimationDirectionValue::Reverse),
        "alternate" => Some(AnimationDirectionValue::Alternate),
        "alternate-reverse" => Some(AnimationDirectionValue::AlternateReverse),
        _ => None,
    }
}

/// 解析 CSS animation-fill-mode 值。
pub fn parse_animation_fill_mode(value: &str) -> Option<AnimationFillModeValue> {
    match value.trim() {
        "none" => Some(AnimationFillModeValue::None),
        "forwards" => Some(AnimationFillModeValue::Forwards),
        "backwards" => Some(AnimationFillModeValue::Backwards),
        "both" => Some(AnimationFillModeValue::Both),
        _ => None,
    }
}

/// 解析 CSS animation-play-state 值。
pub fn parse_animation_play_state(value: &str) -> Option<AnimationPlayStateValue> {
    match value.trim() {
        "running" => Some(AnimationPlayStateValue::Running),
        "paused" => Some(AnimationPlayStateValue::Paused),
        _ => None,
    }
}

/// 解析 CSS transition-timing-function 值。
pub fn parse_timing_function(value: &str) -> Option<TimingFunctionValue> {
    let value = value.trim();

    match value {
        "ease" => Some(TimingFunctionValue::Ease),
        "linear" => Some(TimingFunctionValue::Linear),
        "ease-in" => Some(TimingFunctionValue::EaseIn),
        "ease-out" => Some(TimingFunctionValue::EaseOut),
        "ease-in-out" => Some(TimingFunctionValue::EaseInOut),
        "step-start" => Some(TimingFunctionValue::StepStart),
        "step-end" => Some(TimingFunctionValue::StepEnd),
        _ if value.starts_with("cubic-bezier(") => {
            let inner = extract_parens_content(value, "cubic-bezier")?;
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                return None;
            }
            let x1 = parts[0].parse::<f64>().ok()?;
            let y1 = parts[1].parse::<f64>().ok()?;
            let x2 = parts[2].parse::<f64>().ok()?;
            let y2 = parts[3].parse::<f64>().ok()?;
            Some(TimingFunctionValue::CubicBezier(x1, y1, x2, y2))
        }
        _ if value.starts_with("steps(") => {
            let inner = extract_parens_content(value, "steps")?;
            let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
            let n: i32 = parts.first()?.parse().ok()?;
            let position = if parts.len() > 1 {
                Some(parse_step_position(parts[1])?)
            } else {
                None
            };
            Some(TimingFunctionValue::Steps(n, position))
        }
        _ => None,
    }
}

/// 解析 steps() 位置参数。
fn parse_step_position(s: &str) -> Option<StepPosition> {
    match s.trim() {
        "jump-start" | "start" => Some(StepPosition::Start),
        "jump-end" | "end" => Some(StepPosition::End),
        "jump-both" | "both" => Some(StepPosition::Both),
        "jump-none" | "none" => Some(StepPosition::None),
        _ => None,
    }
}

/// 提取函数括号内的内容。
fn extract_parens_content<'a>(value: &'a str, func_name: &str) -> Option<&'a str> {
    let prefix = format!("{}(", func_name);
    if !value.starts_with(&prefix) || !value.ends_with(')') {
        return None;
    }
    Some(&value[func_name.len() + 1..value.len() - 1])
}

/// 解析 CSS 时间值（如 `"0.3s"`、`"200ms"`）。
///
/// 返回秒为单位的 f64 值。
pub fn parse_time(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.ends_with("ms") {
        let ms: f64 = value.trim_end_matches("ms").trim().parse().ok()?;
        Some(ms / 1000.0)
    } else if value.ends_with('s') {
        let s: f64 = value.trim_end_matches('s').trim().parse().ok()?;
        Some(s)
    } else {
        None
    }
}

// ── CSS Transform 值类型 ──────────────────────────────────────────────

/// CSS transform 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TransformValue {
    /// none。
    None,
    /// 变换函数列表。
    List(Vec<TransformFunction>),
}

/// CSS 单个变换函数。
#[derive(Debug, Clone, PartialEq)]
pub enum TransformFunction {
    /// translate(tx, ty)。
    Translate(f64, f64),
    /// translateX(tx)。
    TranslateX(f64),
    /// translateY(ty)。
    TranslateY(f64),
    /// rotate(angle) — 角度（度数）。
    Rotate(f64),
    /// scale(sx, sy)。
    Scale(f64, Option<f64>),
    /// scaleX(sx)。
    ScaleX(f64),
    /// scaleY(sy)。
    ScaleY(f64),
    /// skew(ax, ay) — 角度（度数）。
    Skew(f64, Option<f64>),
    /// rotateX(angle) — 绕 X 轴旋转（度数）。
    RotateX(f64),
    /// rotateY(angle) — 绕 Y 轴旋转（度数）。
    RotateY(f64),
    /// rotateZ(angle) — 绕 Z 轴旋转（度数）。
    RotateZ(f64),
    /// translate3d(tx, ty, tz) — 三维平移。
    Translate3d(f64, f64, f64),
    /// scale3d(sx, sy, sz) — 三维缩放。
    Scale3d(f64, f64, f64),
    /// rotate3d(x, y, z, angle) — 绕任意轴旋转。
    Rotate3d(f64, f64, f64, f64),
    /// perspective(length) — 透视距离。
    Perspective(f64),
    /// matrix(a, b, c, d, e, f) — 二维矩阵变换。
    Matrix(f64, f64, f64, f64, f64, f64),
}

/// 解析 CSS transform 属性值。
///
/// 支持格式如 `"translate(10px, 20px) rotate(45deg) scale(2)"`。
pub fn parse_transform(value: &str) -> Option<TransformValue> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("none") {
        return Some(TransformValue::None);
    }

    let mut functions = Vec::new();
    let mut pos = 0;
    let bytes = value.as_bytes();

    while pos < bytes.len() {
        // 跳过空白
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        // 读取函数名（允许字母和数字，如 translate3d、scale3d、rotate3d）
        let name_start = pos;
        while pos < bytes.len() && (bytes[pos].is_ascii_alphabetic() || bytes[pos].is_ascii_digit()) {
            pos += 1;
        }
        let name = &value[name_start..pos];

        // 跳过空白和 '('
        while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() || bytes[pos] != b'(' {
            return None;
        }
        pos += 1; // skip '('

        // 找到匹配的 ')'
        let args_start = pos;
        let mut depth = 1;
        while pos < bytes.len() && depth > 0 {
            if bytes[pos] == b'(' {
                depth += 1;
            } else if bytes[pos] == b')' {
                depth -= 1;
            }
            pos += 1;
        }
        let args_str = value[args_start..pos - 1].trim();

        // 解析函数
        if let Some(func) = parse_transform_function(name, args_str) {
            functions.push(func);
        } else {
            return None;
        }
    }

    if functions.is_empty() {
        None
    } else {
        Some(TransformValue::List(functions))
    }
}

/// 解析单个变换函数。
fn parse_transform_function(name: &str, args: &str) -> Option<TransformFunction> {
    match name {
        "translate" => {
            let vals = parse_transform_args(args)?;
            let tx = vals.first().copied()?;
            let ty = vals.get(1).copied().unwrap_or(0.0);
            Some(TransformFunction::Translate(tx, ty))
        }
        "translateX" => {
            let vals = parse_transform_args(args)?;
            let tx = vals.first().copied()?;
            Some(TransformFunction::TranslateX(tx))
        }
        "translateY" => {
            let vals = parse_transform_args(args)?;
            let ty = vals.first().copied()?;
            Some(TransformFunction::TranslateY(ty))
        }
        "rotate" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::Rotate(angle))
        }
        "scale" => {
            let vals = parse_transform_args(args)?;
            let sx = vals.first().copied()?;
            let sy = vals.get(1).copied();
            Some(TransformFunction::Scale(sx, sy))
        }
        "scaleX" => {
            let vals = parse_transform_args(args)?;
            let sx = vals.first().copied()?;
            Some(TransformFunction::ScaleX(sx))
        }
        "scaleY" => {
            let vals = parse_transform_args(args)?;
            let sy = vals.first().copied()?;
            Some(TransformFunction::ScaleY(sy))
        }
        "skew" => {
            let vals = parse_transform_args(args)?;
            let ax = vals.first().copied()?;
            let ay = vals.get(1).copied();
            Some(TransformFunction::Skew(ax, ay))
        }
        "rotateX" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::RotateX(angle))
        }
        "rotateY" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::RotateY(angle))
        }
        "rotateZ" => {
            let angle = parse_angle(args)?;
            Some(TransformFunction::RotateZ(angle))
        }
        "translate3d" => {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return None;
            }
            let tx = parse_css_number(parts[0])?;
            let ty = parse_css_number(parts[1])?;
            let tz = parse_css_number(parts[2])?;
            Some(TransformFunction::Translate3d(tx, ty, tz))
        }
        "scale3d" => {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() != 3 {
                return None;
            }
            let sx = parse_css_number(parts[0])?;
            let sy = parse_css_number(parts[1])?;
            let sz = parse_css_number(parts[2])?;
            Some(TransformFunction::Scale3d(sx, sy, sz))
        }
        "rotate3d" => {
            let parts: Vec<&str> = args.split(',').map(|s| s.trim()).collect();
            if parts.len() != 4 {
                return None;
            }
            let x = parse_css_number(parts[0])?;
            let y = parse_css_number(parts[1])?;
            let z = parse_css_number(parts[2])?;
            let angle = parse_angle(parts[3])?;
            Some(TransformFunction::Rotate3d(x, y, z, angle))
        }
        "perspective" => {
            let val = parse_css_number(args)?;
            if val <= 0.0 {
                return None;
            }
            Some(TransformFunction::Perspective(val))
        }
        "matrix" => {
            let vals = parse_transform_args(args)?;
            if vals.len() != 6 {
                return None;
            }
            Some(TransformFunction::Matrix(
                vals[0], vals[1], vals[2], vals[3], vals[4], vals[5],
            ))
        }
        _ => None,
    }
}

/// 解析变换参数列表（逗号或空格分隔的数值）。
fn parse_transform_args(args: &str) -> Option<Vec<f64>> {
    let mut result = Vec::new();
    for part in args.split(|c: char| c == ',' || c.is_whitespace()) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        // 尝试解析为带单位的角度或长度
        if let Some(val) = parse_css_number(part) {
            result.push(val);
        } else {
            return None;
        }
    }
    if result.is_empty() { None } else { Some(result) }
}

/// 解析 CSS 数值（可能带 px/deg/rad/turn 等单位）。
///
/// 返回原始数值（px 直接返回数值，deg 转为度数）。
fn parse_css_number(s: &str) -> Option<f64> {
    let s = s.trim();
    if s.ends_with("deg") {
        s.trim_end_matches("deg").trim().parse::<f64>().ok()
    } else if s.ends_with("rad") {
        let rad: f64 = s.trim_end_matches("rad").trim().parse().ok()?;
        Some(rad.to_degrees())
    } else if s.ends_with("turn") {
        let turn: f64 = s.trim_end_matches("turn").trim().parse().ok()?;
        Some(turn * 360.0)
    } else if s.ends_with("px") || s.ends_with("em") || s.ends_with("rem") {
        // 对于 translate，返回数值部分
        let num_end = s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')?;
        s[..num_end].parse::<f64>().ok()
    } else {
        s.parse::<f64>().ok()
    }
}

/// 解析角度值（返回度数）。
fn parse_angle(s: &str) -> Option<f64> {
    parse_css_number(s)
}

// ── CSS Gradient 值类型 ──────────────────────────────────────────────

/// CSS 渐变方向。
#[derive(Debug, Clone, PartialEq)]
pub enum GradientDirection {
    /// 角度（度数）。
    Angle(f64),
    /// to top。
    ToTop,
    /// to bottom。
    ToBottom,
    /// to left。
    ToLeft,
    /// to right。
    ToRight,
    /// to top left / to left top。
    ToTopLeft,
    /// to top right / to right top。
    ToTopRight,
    /// to bottom left / to left bottom。
    ToBottomLeft,
    /// to bottom right / to right bottom。
    ToBottomRight,
}

/// CSS 渐变色标。
#[derive(Debug, Clone, PartialEq)]
pub struct GradientColorStop {
    /// 颜色值。
    pub color: ColorValue,
    /// 位置提示（百分比或长度），如 `50%`、`10px`。
    pub position: Option<LengthValue>,
}

/// CSS linear-gradient() 值。
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// 渐变方向，默认为 to bottom。
    pub direction: GradientDirection,
    /// 色标列表。
    pub stops: Vec<GradientColorStop>,
    /// 是否为 repeating-linear-gradient。
    pub repeating: bool,
}

/// CSS radial-gradient 形状。
#[derive(Debug, Clone, PartialEq)]
pub enum RadialShape {
    /// circle。
    Circle,
    /// ellipse。
    Ellipse,
}

/// CSS radial-gradient 尺寸。
#[derive(Debug, Clone, PartialEq)]
pub enum RadialSize {
    /// closest-side。
    ClosestSide,
    /// farthest-side。
    FarthestSide,
    /// closest-corner。
    ClosestCorner,
    /// farthest-corner（默认）。
    FarthestCorner,
    /// 明确的半径值。
    Length(LengthValue),
}

/// CSS radial-gradient() 值。
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// 形状，默认为 ellipse。
    pub shape: RadialShape,
    /// 尺寸，默认为 farthest-corner。
    pub size: RadialSize,
    /// 中心位置 X，默认为 center (50%)。
    pub position_x: LengthValue,
    /// 中心位置 Y，默认为 center (50%)。
    pub position_y: LengthValue,
    /// 色标列表。
    pub stops: Vec<GradientColorStop>,
    /// 是否为 repeating-radial-gradient。
    pub repeating: bool,
}

/// CSS conic-gradient() 值。
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    /// 起始角度（度数），默认为 0。
    pub from_angle: f64,
    /// 中心位置 X，默认为 center (50%)。
    pub position_x: LengthValue,
    /// 中心位置 Y，默认为 center (50%)。
    pub position_y: LengthValue,
    /// 色标列表。
    pub stops: Vec<GradientColorStop>,
    /// 是否为 repeating-conic-gradient。
    pub repeating: bool,
}

/// CSS 渐变值（所有渐变类型的统一表示）。
#[derive(Debug, Clone, PartialEq)]
pub enum GradientValue {
    /// linear-gradient() / repeating-linear-gradient()。
    Linear(LinearGradient),
    /// radial-gradient() / repeating-radial-gradient()。
    Radial(RadialGradient),
    /// conic-gradient() / repeating-conic-gradient()。
    Conic(ConicGradient),
}

/// 解析 CSS 渐变值。
///
/// 支持格式：
/// - `linear-gradient(direction, color-stop1, color-stop2, ...)`
/// - `radial-gradient(shape size at position, color-stop1, ...)`
/// - `conic-gradient(from angle at position, color-stop1, ...)`
/// - 以及对应的 repeating- 变体。
pub fn parse_gradient(value: &str) -> Option<GradientValue> {
    let value = value.trim();

    let (func_name, inner) = split_function_call(value)?;

    match func_name.to_ascii_lowercase().as_str() {
        "linear-gradient" => parse_linear_gradient_inner(inner, false),
        "repeating-linear-gradient" => parse_linear_gradient_inner(inner, true),
        "radial-gradient" => parse_radial_gradient_inner(inner, false),
        "repeating-radial-gradient" => parse_radial_gradient_inner(inner, true),
        "conic-gradient" => parse_conic_gradient_inner(inner, false),
        "repeating-conic-gradient" => parse_conic_gradient_inner(inner, true),
        _ => None,
    }
}

/// 将函数调用拆分为 (函数名, 括号内内容)。
fn split_function_call(value: &str) -> Option<(String, &str)> {
    let paren_pos = value.find('(')?;
    let name = &value[..paren_pos];
    if !value.ends_with(')') {
        return None;
    }
    let inner = &value[paren_pos + 1..value.len() - 1];
    Some((name.to_string(), inner))
}

/// 解析 linear-gradient 内部参数。
fn parse_linear_gradient_inner(inner: &str, repeating: bool) -> Option<GradientValue> {
    let args = split_gradient_args(inner)?;
    if args.is_empty() {
        return None;
    }

    let mut direction = GradientDirection::ToBottom;
    let mut stop_start = 0;

    // 检查第一个参数是否为方向
    let first = args[0].trim();
    if let Some(dir) = parse_linear_direction(first) {
        direction = dir;
        stop_start = 1;
    }

    let stops = parse_color_stops(&args[stop_start..])?;
    if stops.is_empty() {
        return None;
    }

    Some(GradientValue::Linear(LinearGradient {
        direction,
        stops,
        repeating,
    }))
}

/// 解析 linear-gradient 方向参数。
fn parse_linear_direction(s: &str) -> Option<GradientDirection> {
    let s = s.trim();
    // 角度
    if let Some(angle) = parse_angle(s) {
        return Some(GradientDirection::Angle(angle));
    }
    // to 关键字方向
    match s.to_ascii_lowercase().as_str() {
        "to top" => Some(GradientDirection::ToTop),
        "to bottom" => Some(GradientDirection::ToBottom),
        "to left" => Some(GradientDirection::ToLeft),
        "to right" => Some(GradientDirection::ToRight),
        "to top left" | "to left top" => Some(GradientDirection::ToTopLeft),
        "to top right" | "to right top" => Some(GradientDirection::ToTopRight),
        "to bottom left" | "to left bottom" => Some(GradientDirection::ToBottomLeft),
        "to bottom right" | "to right bottom" => Some(GradientDirection::ToBottomRight),
        _ => None,
    }
}

/// 解析 radial-gradient 内部参数。
fn parse_radial_gradient_inner(inner: &str, repeating: bool) -> Option<GradientValue> {
    let args = split_gradient_args(inner)?;
    if args.is_empty() {
        return None;
    }

    let mut shape = RadialShape::Ellipse;
    let mut size = RadialSize::FarthestCorner;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);
    let mut stop_start = 0;

    // 第一个参数可能包含 shape/size/position
    let first = args[0].trim();
    let first_lower = first.to_ascii_lowercase();

    if first_lower.starts_with("circle")
        || first_lower.starts_with("ellipse")
        || first_lower.starts_with("closest")
        || first_lower.starts_with("farthest")
        || first_lower.contains(" at ")
    {
        // 解析 shape + size + at position
        if let Some((s, sz, px, py)) = parse_radial_shape_and_position(first) {
            shape = s;
            size = sz;
            pos_x = px;
            pos_y = py;
        }
        stop_start = 1;
    }

    let stops = parse_color_stops(&args[stop_start..])?;
    if stops.is_empty() {
        return None;
    }

    Some(GradientValue::Radial(RadialGradient {
        shape,
        size,
        position_x: pos_x,
        position_y: pos_y,
        stops,
        repeating,
    }))
}

/// 解析 radial-gradient 的 shape、size 和 at position。
fn parse_radial_shape_and_position(s: &str) -> Option<(RadialShape, RadialSize, LengthValue, LengthValue)> {
    let mut shape = RadialShape::Ellipse;
    let mut size = RadialSize::FarthestCorner;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);

    let lower = s.to_ascii_lowercase();

    // 解析 "at x y" 位置
    if let Some(at_pos) = lower.find(" at ") {
        let pos_str = &s[at_pos + 4..];
        if let Some((px, py)) = parse_position_pair(pos_str) {
            pos_x = px;
            pos_y = py;
        }
        // 解析 at 之前的部分为 shape/size
        let shape_str = s[..at_pos].trim();
        parse_radial_shape_size(shape_str, &mut shape, &mut size);
    } else {
        parse_radial_shape_size(s, &mut shape, &mut size);
    }

    Some((shape, size, pos_x, pos_y))
}

/// 解析 radial shape 和 size 关键字。
fn parse_radial_shape_size(s: &str, shape: &mut RadialShape, size: &mut RadialSize) {
    let lower = s.trim().to_ascii_lowercase();

    // 检查长度值（如 "50px 100px" 或 "circle 50px"）
    let parts: Vec<&str> = lower.split_whitespace().collect();
    for part in parts {
        match part {
            "circle" => *shape = RadialShape::Circle,
            "ellipse" => *shape = RadialShape::Ellipse,
            "closest-side" => *size = RadialSize::ClosestSide,
            "farthest-side" => *size = RadialSize::FarthestSide,
            "closest-corner" => *size = RadialSize::ClosestCorner,
            "farthest-corner" => *size = RadialSize::FarthestCorner,
            _ => {
                // 尝试解析为长度值
                if let Some(lv) = parse_length(part) {
                    *size = RadialSize::Length(lv);
                }
            }
        }
    }
}

/// 解析位置对（如 "center center"、"50% 50%"、"left top"）。
fn parse_position_pair(s: &str) -> Option<(LengthValue, LengthValue)> {
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();

    match parts.len() {
        1 => {
            let p = parse_position_keyword(parts[0]);
            Some((p.clone(), p))
        }
        2 => {
            let px = parse_position_keyword(parts[0]);
            let py = parse_position_keyword(parts[1]);
            Some((px, py))
        }
        _ => None,
    }
}

/// 解析位置关键字为 LengthValue。
fn parse_position_keyword(s: &str) -> LengthValue {
    match s.to_ascii_lowercase().as_str() {
        "center" => LengthValue::Percentage(50.0),
        "left" => LengthValue::Percentage(0.0),
        "right" => LengthValue::Percentage(100.0),
        "top" => LengthValue::Percentage(0.0),
        "bottom" => LengthValue::Percentage(100.0),
        other => parse_length(other).unwrap_or(LengthValue::Percentage(50.0)),
    }
}

/// 解析 conic-gradient 内部参数。
fn parse_conic_gradient_inner(inner: &str, repeating: bool) -> Option<GradientValue> {
    let args = split_gradient_args(inner)?;
    if args.is_empty() {
        return None;
    }

    let mut from_angle = 0.0;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);
    let mut stop_start = 0;

    let first = args[0].trim();
    let first_lower = first.to_ascii_lowercase();

    if first_lower.starts_with("from ") || first_lower.starts_with("at ") || first_lower.contains(" at ") {
        if let Some((angle, px, py)) = parse_conic_config(first) {
            from_angle = angle;
            pos_x = px;
            pos_y = py;
        }
        stop_start = 1;
    }

    let stops = parse_color_stops(&args[stop_start..])?;
    if stops.is_empty() {
        return None;
    }

    Some(GradientValue::Conic(ConicGradient {
        from_angle,
        position_x: pos_x,
        position_y: pos_y,
        stops,
        repeating,
    }))
}

/// 解析 conic-gradient 的 from angle 和 at position 配置。
fn parse_conic_config(s: &str) -> Option<(f64, LengthValue, LengthValue)> {
    let mut angle = 0.0;
    let mut pos_x = LengthValue::Percentage(50.0);
    let mut pos_y = LengthValue::Percentage(50.0);

    let lower = s.to_ascii_lowercase();

    // 解析 "from <angle>"
    if let Some(from_pos) = lower.find("from ") {
        let after_from = &s[from_pos + 5..];
        // 找到 from 和 at 之间的部分作为角度
        let at_pos = after_from.to_ascii_lowercase().find(" at ").unwrap_or(after_from.len());
        let angle_str = after_from[..at_pos].trim();
        if !angle_str.is_empty() {
            angle = parse_angle(angle_str).unwrap_or(0.0);
        }
    }

    // 解析 "at <position>"（支持 "from X at Y" 和直接 "at Y"）
    let at_keyword = if lower.starts_with("at ") {
        Some(0)
    } else {
        lower.find(" at ")
    };
    if let Some(at_pos) = at_keyword {
        let pos_str = &s[at_pos + 3..];
        // 在第一个逗号处截断，避免渐变色标干扰位置解析
        let pos_str = pos_str.split(',').next().unwrap_or(pos_str).trim();
        if let Some((px, py)) = parse_position_pair(pos_str) {
            pos_x = px;
            pos_y = py;
        }
    }

    Some((angle, pos_x, pos_y))
}

/// 将渐变参数按顶层逗号分割（不分割括号内的逗号）。
fn split_gradient_args(inner: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    let bytes = inner.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                args.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < inner.len() {
        args.push(&inner[start..]);
    }

    Some(args)
}

/// 解析色标列表。
fn parse_color_stops(args: &[&str]) -> Option<Vec<GradientColorStop>> {
    let mut stops = Vec::new();
    for arg in args {
        let arg = arg.trim();
        if arg.is_empty() {
            continue;
        }
        stops.push(parse_color_stop(arg)?);
    }
    Some(stops)
}

/// 解析单个色标（如 `red`、`red 50%`、`#00ff00 10px`）。
fn parse_color_stop(s: &str) -> Option<GradientColorStop> {
    let s = s.trim();

    // 尝试 "color position" 格式
    // 从右往左找位置部分
    let last_space = s.rfind(' ');
    if let Some(space_pos) = last_space {
        let color_str = &s[..space_pos];
        let pos_str = &s[space_pos + 1..];

        if let Some(color) = parse_color(color_str)
            && let Some(position) = parse_length(pos_str)
        {
            return Some(GradientColorStop {
                color,
                position: Some(position),
            });
        }
    }

    // 仅颜色
    let color = parse_color(s)?;
    Some(GradientColorStop { color, position: None })
}

/// 解析 CSS grid-area 简写属性值。
///
/// 支持格式：
/// - 单值：`"header"` → 四个值均为 `"header"`
/// - `"auto"` → 四个值均为 `"auto"`
/// - 四值斜杠分隔：`"1 / 2 / 3 / 4"` → `("1", "2", "3", "4")`
/// - 两值斜杠分隔：`"1 / 3"` → `("1", "auto", "3", "auto")`
/// - 三值斜杠分隔：`"1 / 2 / 3"` → `("1", "2", "3", "auto")`
///
/// 返回 `(row_start, row_end, col_start, col_end)` 原始字符串元组，
/// 由 style-system 调用 `parse_grid_line` 转换为 `GridLineValue`。
pub fn parse_grid_area(input: &str) -> Option<(String, String, String, String)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    // 包含斜杠 → 按斜杠分割
    if input.contains('/') {
        let parts: Vec<&str> = input.split('/').map(|s| s.trim()).collect();
        match parts.len() {
            1 => {
                // 单值（斜杠后为空，不合法）
                let v = parts[0].to_string();
                if v.is_empty() {
                    return None;
                }
                Some((v.clone(), v.clone(), v.clone(), v))
            }
            2 => {
                // row-start / col-start
                let rs = parts[0].to_string();
                let cs = parts[1].to_string();
                if rs.is_empty() || cs.is_empty() {
                    return None;
                }
                Some((rs, "auto".to_string(), cs, "auto".to_string()))
            }
            3 => {
                // row-start / row-end / col-start
                let rs = parts[0].to_string();
                let re = parts[1].to_string();
                let cs = parts[2].to_string();
                if rs.is_empty() || re.is_empty() || cs.is_empty() {
                    return None;
                }
                Some((rs, re, cs, "auto".to_string()))
            }
            4 => {
                // row-start / row-end / col-start / col-end
                let rs = parts[0].to_string();
                let re = parts[1].to_string();
                let cs = parts[2].to_string();
                let ce = parts[3].to_string();
                if rs.is_empty() || re.is_empty() || cs.is_empty() || ce.is_empty() {
                    return None;
                }
                Some((rs, re, cs, ce))
            }
            _ => None,
        }
    } else {
        // 单值，所有四个都设为同一值
        let v = input.to_string();
        Some((v.clone(), v.clone(), v.clone(), v))
    }
}

/// CSS text-shadow 值。
#[derive(Debug, Clone, PartialEq)]
pub struct TextShadowValue {
    /// 水平偏移量。
    pub offset_x: LengthValue,
    /// 垂直偏移量。
    pub offset_y: LengthValue,
    /// 模糊半径。
    pub blur_radius: LengthValue,
    /// 阴影颜色。
    pub color: ColorValue,
}

/// 解析 CSS text-shadow 值。
///
/// 格式：`"none"` | `"<offset-x> <offset-y> [<blur-radius>] [<color>]"`。
pub fn parse_text_shadow(value: &str) -> Option<TextShadowValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(TextShadowValue {
            offset_x: LengthValue::Px(0.0),
            offset_y: LengthValue::Px(0.0),
            blur_radius: LengthValue::Px(0.0),
            color: ColorValue::Rgba(0, 0, 0, 255),
        });
    }
    // 解析 "2px 2px 4px red" 或 "2px 2px" 或 "2px 2px red"
    let parts: Vec<&str> = v.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let ox = parse_length(parts[0])?;
    let oy = parse_length(parts[1])?;
    let (blur, color) = if parts.len() >= 3 {
        if let Some(c) = parse_color(parts[2]) {
            (LengthValue::Px(0.0), c)
        } else if let Some(b) = parse_length(parts[2]) {
            let c = if parts.len() >= 4 {
                parse_color(parts[3]).unwrap_or(ColorValue::Rgba(0, 0, 0, 255))
            } else {
                ColorValue::Rgba(0, 0, 0, 255)
            };
            (b, c)
        } else {
            (LengthValue::Px(0.0), ColorValue::Rgba(0, 0, 0, 255))
        }
    } else {
        (LengthValue::Px(0.0), ColorValue::Rgba(0, 0, 0, 255))
    };
    Some(TextShadowValue {
        offset_x: ox,
        offset_y: oy,
        blur_radius: blur,
        color,
    })
}

/// CSS box-shadow 单个阴影。
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadowValue {
    /// 水平偏移量。
    pub offset_x: LengthValue,
    /// 垂直偏移量。
    pub offset_y: LengthValue,
    /// 模糊半径。
    pub blur_radius: LengthValue,
    /// 扩展半径。
    pub spread_radius: LengthValue,
    /// 阴影颜色。
    pub color: ColorValue,
    /// 是否为内阴影。
    pub inset: bool,
}

/// 解析 CSS box-shadow 值。
///
/// 格式：`"none"` | `"[inset] <offset-x> <offset-y> [<blur>] [<spread>] [<color>]"`。
pub fn parse_box_shadow(value: &str) -> Option<BoxShadowValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("none") {
        return Some(BoxShadowValue {
            offset_x: LengthValue::Px(0.0),
            offset_y: LengthValue::Px(0.0),
            blur_radius: LengthValue::Px(0.0),
            spread_radius: LengthValue::Px(0.0),
            color: ColorValue::Rgba(0, 0, 0, 255),
            inset: false,
        });
    }
    let lower = v.to_ascii_lowercase();
    let inset = lower.starts_with("inset");
    let rest = if inset { v[5..].trim_start() } else { v };
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let ox = parse_length(parts[0])?;
    let oy = parse_length(parts[1])?;
    let blur = if parts.len() >= 3 {
        parse_length(parts[2]).unwrap_or(LengthValue::Px(0.0))
    } else {
        LengthValue::Px(0.0)
    };
    let spread = if parts.len() >= 4 {
        parse_length(parts[3]).unwrap_or(LengthValue::Px(0.0))
    } else {
        LengthValue::Px(0.0)
    };
    // 颜色在最后一个非长度 token 或默认黑色
    let color = parts
        .iter()
        .find_map(|p| parse_color(p))
        .unwrap_or(ColorValue::Rgba(0, 0, 0, 255));
    Some(BoxShadowValue {
        offset_x: ox,
        offset_y: oy,
        blur_radius: blur,
        spread_radius: spread,
        color,
        inset,
    })
}

// ── CSS text-overflow / table / caption / border-collapse / resize 值类型 ──

/// CSS text-overflow 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextOverflowValue {
    /// clip（默认值）— 裁剪溢出内容。
    Clip,
    /// ellipsis — 显示省略号。
    Ellipsis,
    /// 自定义字符串。
    String(String),
}

/// 解析 CSS text-overflow 属性值。
///
/// 支持 `clip`、`ellipsis` 和自定义字符串（带引号）。
pub fn parse_text_overflow(value: &str) -> Option<TextOverflowValue> {
    let v = value.trim();
    match v {
        "clip" => Some(TextOverflowValue::Clip),
        "ellipsis" => Some(TextOverflowValue::Ellipsis),
        s => {
            // 支持引号包裹的自定义字符串，如 `"…"` 或 `'...'`
            if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
                || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
            {
                let inner = &s[1..s.len() - 1];
                if inner.is_empty() {
                    return None;
                }
                Some(TextOverflowValue::String(inner.to_string()))
            } else {
                None
            }
        }
    }
}

/// CSS table-layout 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TableLayoutValue {
    /// auto（默认值）— 自动表格布局。
    Auto,
    /// fixed — 固定表格布局。
    Fixed,
}

/// 解析 CSS table-layout 属性值。
pub fn parse_table_layout(value: &str) -> Option<TableLayoutValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(TableLayoutValue::Auto),
        "fixed" => Some(TableLayoutValue::Fixed),
        _ => None,
    }
}

/// CSS caption-side 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CaptionSideValue {
    /// top（默认值）— 标题在表格上方。
    Top,
    /// bottom — 标题在表格下方。
    Bottom,
}

/// 解析 CSS caption-side 属性值。
pub fn parse_caption_side(value: &str) -> Option<CaptionSideValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "top" => Some(CaptionSideValue::Top),
        "bottom" => Some(CaptionSideValue::Bottom),
        _ => None,
    }
}

/// CSS border-collapse 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BorderCollapseValue {
    /// separate（默认值）— 分离边框模型。
    Separate,
    /// collapse — 合并边框模型。
    Collapse,
}

/// 解析 CSS border-collapse 属性值。
pub fn parse_border_collapse(value: &str) -> Option<BorderCollapseValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "separate" => Some(BorderCollapseValue::Separate),
        "collapse" => Some(BorderCollapseValue::Collapse),
        _ => None,
    }
}

/// CSS resize 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeValue {
    /// none（默认值）— 不可调整大小。
    None,
    /// both — 水平和垂直均可调整。
    Both,
    /// horizontal — 仅水平。
    Horizontal,
    /// vertical — 仅垂直。
    Vertical,
    /// block — 块方向。
    Block,
    /// inline — 行内方向。
    Inline,
}

/// 解析 CSS resize 属性值。
pub fn parse_resize(value: &str) -> Option<ResizeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ResizeValue::None),
        "both" => Some(ResizeValue::Both),
        "horizontal" => Some(ResizeValue::Horizontal),
        "vertical" => Some(ResizeValue::Vertical),
        "block" => Some(ResizeValue::Block),
        "inline" => Some(ResizeValue::Inline),
        _ => None,
    }
}

// ── CSS Interaction / Performance Hint 值类型 ──────────────────────────

/// CSS overscroll-behavior 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverscrollBehaviorValue {
    /// auto（默认值）— 浏览器默认滚动溢出行为。
    Auto,
    /// contain — 阻止滚动链传播到祖先元素。
    Contain,
    /// none — 阻止滚动链和默认溢出行为。
    None,
}

/// 解析 CSS overscroll-behavior 属性值。
pub fn parse_overscroll_behavior(value: &str) -> Option<OverscrollBehaviorValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(OverscrollBehaviorValue::Auto),
        "contain" => Some(OverscrollBehaviorValue::Contain),
        "none" => Some(OverscrollBehaviorValue::None),
        _ => None,
    }
}

/// CSS touch-action 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TouchActionValue {
    /// auto（默认值）— 浏览器处理所有触摸操作。
    Auto,
    /// none — 禁用所有触摸操作。
    None,
    /// pan-x — 仅允许水平平移。
    PanX,
    /// pan-y — 仅允许垂直平移。
    PanY,
    /// pan-x pan-y — 允许水平和垂直平移。
    PanXPanY,
    /// manipulation — 仅允许平移和缩放（禁用双击缩放）。
    Manipulation,
}

/// 解析 CSS touch-action 属性值。
pub fn parse_touch_action(value: &str) -> Option<TouchActionValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(TouchActionValue::Auto),
        "none" => Some(TouchActionValue::None),
        "pan-x" => Some(TouchActionValue::PanX),
        "pan-y" => Some(TouchActionValue::PanY),
        "pan-x pan-y" | "pan-y pan-x" => Some(TouchActionValue::PanXPanY),
        "manipulation" => Some(TouchActionValue::Manipulation),
        _ => None,
    }
}

/// CSS user-select 值。
#[derive(Debug, Clone, PartialEq)]
pub enum UserSelectValue {
    /// auto（默认值）— 由浏览器决定。
    Auto,
    /// text — 可选择文本。
    Text,
    /// none — 禁止选择。
    None,
    /// all — 点击即全选。
    All,
    /// contain — 选择限制在元素内。
    Contain,
}

/// 解析 CSS user-select 属性值。
pub fn parse_user_select(value: &str) -> Option<UserSelectValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(UserSelectValue::Auto),
        "text" => Some(UserSelectValue::Text),
        "none" => Some(UserSelectValue::None),
        "all" => Some(UserSelectValue::All),
        "contain" => Some(UserSelectValue::Contain),
        _ => None,
    }
}

/// CSS will-change 值。
#[derive(Debug, Clone, PartialEq)]
pub enum WillChangeValue {
    /// auto（默认值）— 无特别提示。
    Auto,
    /// scroll-position — 预期滚动位置会变化。
    ScrollPosition,
    /// contents — 预期内容会变化。
    Contents,
    /// 自定义属性名（如 transform、opacity）。
    Custom(String),
}

/// 解析 CSS will-change 属性值。
pub fn parse_will_change(value: &str) -> Option<WillChangeValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(WillChangeValue::Auto),
        "scroll-position" => Some(WillChangeValue::ScrollPosition),
        "contents" => Some(WillChangeValue::Contents),
        _ => {
            // 接受任意标识符（如 transform、opacity、top、left）
            if v.is_empty() {
                return None;
            }
            // 简单验证：只包含字母、数字、连字符
            if v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                Some(WillChangeValue::Custom(v))
            } else {
                None
            }
        }
    }
}

/// CSS pointer-events 值。
#[derive(Debug, Clone, PartialEq)]
pub enum PointerEventsValue {
    /// auto（默认值）— 元素是指针事件的目标。
    Auto,
    /// none — 元素不是指针事件的目标。
    None,
    /// visiblePainted — SVG：可见且填充/描边区域。
    VisiblePainted,
    /// visibleFill — SVG：可见且填充区域。
    VisibleFill,
    /// visibleStroke — SVG：可见且描边区域。
    VisibleStroke,
    /// visible — SVG：可见区域。
    Visible,
    /// painted — SVG：填充/描边区域（不论可见性）。
    Painted,
    /// fill — SVG：填充区域。
    Fill,
    /// stroke — SVG：描边区域。
    Stroke,
    /// all — SVG：所有区域。
    All,
    /// inherit — 显式继承。
    Inherit,
}

/// 解析 CSS pointer-events 属性值。
pub fn parse_pointer_events(value: &str) -> Option<PointerEventsValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(PointerEventsValue::Auto),
        "none" => Some(PointerEventsValue::None),
        "visiblepainted" => Some(PointerEventsValue::VisiblePainted),
        "visiblefill" => Some(PointerEventsValue::VisibleFill),
        "visiblestroke" => Some(PointerEventsValue::VisibleStroke),
        "visible" => Some(PointerEventsValue::Visible),
        "painted" => Some(PointerEventsValue::Painted),
        "fill" => Some(PointerEventsValue::Fill),
        "stroke" => Some(PointerEventsValue::Stroke),
        "all" => Some(PointerEventsValue::All),
        "inherit" => Some(PointerEventsValue::Inherit),
        _ => None,
    }
}

// ── CSS Counter 值类型 ──────────────────────────────────────────────

/// CSS counter-increment / counter-reset 单个计数器操作值。
#[derive(Debug, Clone, PartialEq)]
pub struct CounterActionValue {
    /// 计数器名称。
    pub name: String,
    /// 增量或重置值，None 表示默认（increment=1, reset=0）。
    pub value: Option<i32>,
}

/// 解析单个计数器操作值。
///
/// 格式：`"counter-name"` 或 `"counter-name 5"`。
pub fn parse_counter_action(input: &str) -> Option<CounterActionValue> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let parts: Vec<&str> = input.split_whitespace().collect();
    let name = parts.first()?.to_string();
    // 计数器名称不能是 none
    if name.eq_ignore_ascii_case("none") {
        return None;
    }
    let value = if parts.len() > 1 {
        Some(parts[1].parse::<i32>().ok()?)
    } else {
        None
    };
    Some(CounterActionValue { name, value })
}

/// 解析计数器操作列表。
///
/// 格式：`"section 1 subsection"` → `[CounterActionValue { name: "section", value: Some(1) }, CounterActionValue { name: "subsection", value: None }]`。
/// 特殊值 `"none"` 返回空列表。
pub fn parse_counter_list(input: &str) -> Option<Vec<CounterActionValue>> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("none") {
        return Some(vec![]);
    }
    let mut result = Vec::new();
    let mut tokens = input.split_whitespace().peekable();
    while let Some(name) = tokens.next() {
        if name.eq_ignore_ascii_case("none") {
            return None;
        }
        // 检查下一个 token 是否为整数
        let value = if tokens.peek().is_some_and(|t| t.parse::<i32>().is_ok()) {
            tokens.next().and_then(|t| t.parse::<i32>().ok())
        } else {
            None
        };
        result.push(CounterActionValue {
            name: name.to_string(),
            value,
        });
    }
    if result.is_empty() {
        return None;
    }
    Some(result)
}

// ── CSS Content 值类型 ──────────────────────────────────────────────

/// CSS content 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContentValue {
    /// normal（默认值）。
    Normal,
    /// none。
    None,
    /// 字符串内容。
    String(String),
    /// attr() 函数引用。
    Attr(String),
    /// counter() 函数引用。
    Counter {
        /// 计数器名称。
        name: String,
        /// 可选的列表样式类型。
        style: Option<String>,
    },
}

/// 解析 CSS content 属性值。
///
/// 支持格式：`normal`、`none`、字符串、`attr(name)`、`counter(name)` 或 `counter(name, style)`。
pub fn parse_content(input: &str) -> Option<ContentValue> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("normal") {
        return Some(ContentValue::Normal);
    }
    if input.eq_ignore_ascii_case("none") {
        return Some(ContentValue::None);
    }
    // 字符串：引号包裹
    if (input.starts_with('"') && input.ends_with('"')) || (input.starts_with('\'') && input.ends_with('\'')) {
        if input.len() < 2 {
            return None;
        }
        return Some(ContentValue::String(input[1..input.len() - 1].to_string()));
    }
    // attr(name)
    if input.starts_with("attr(") && input.ends_with(')') {
        let inner = input[5..input.len() - 1].trim();
        if inner.is_empty() {
            return None;
        }
        return Some(ContentValue::Attr(inner.to_string()));
    }
    // counter(name) 或 counter(name, style)
    if input.starts_with("counter(") && input.ends_with(')') {
        let inner = input[8..input.len() - 1].trim();
        if inner.is_empty() {
            return None;
        }
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        let name = parts.first()?.to_string();
        let style = if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        };
        return Some(ContentValue::Counter { name, style });
    }
    None
}

// ── CSS Quotes 值类型 ──────────────────────────────────────────────

/// CSS quotes 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum QuotesValue {
    /// none — 不使用引号。
    None,
    /// auto — 使用基于内容语言的引号。
    Auto,
    /// 引号对列表，每对为 (open, close)。
    Pairs(Vec<(String, String)>),
}

/// 解析 CSS quotes 属性值。
///
/// 支持格式：
/// - `none`
/// - `auto`
/// - 引号对列表：`"«" "»" "‹" "›"`（开引号和闭引号交替出现）
pub fn parse_quotes(input: &str) -> Option<QuotesValue> {
    let input = input.trim();
    if input.eq_ignore_ascii_case("none") {
        return Some(QuotesValue::None);
    }
    if input.eq_ignore_ascii_case("auto") {
        return Some(QuotesValue::Auto);
    }
    // 解析引号对：交替出现的引号字符串
    let mut pairs = Vec::new();
    let mut chars = input.chars().peekable();
    loop {
        // 跳过空白
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        if chars.peek().is_none() {
            break;
        }
        // 读取开引号
        let open = parse_quoted_string_chars(&mut chars)?;
        // 跳过空白
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        // 读取闭引号
        let close = parse_quoted_string_chars(&mut chars)?;
        pairs.push((open, close));
    }
    if pairs.is_empty() {
        return None;
    }
    Some(QuotesValue::Pairs(pairs))
}

/// 从字符流中解析引号包裹的字符串内容。
fn parse_quoted_string_chars(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<String> {
    let quote = chars.peek()?;
    if *quote != '"' && *quote != '\'' {
        return None;
    }
    let q = chars.next()?; // 消费开头引号
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c == q {
            return Some(result);
        }
        if c == '\\' {
            if let Some(escaped) = chars.next() {
                result.push(escaped);
            }
        } else {
            result.push(c);
        }
    }
    None
}

// ── CSS Contain 值类型 ──────────────────────────────────────────────

/// CSS contain 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainValue {
    /// none（默认值）。
    None,
    /// strict — 等价于 layout style paint。
    Strict,
    /// content — 等价于 layout style paint size。
    Content,
    /// size。
    Size,
    /// layout。
    Layout,
    /// style。
    Style,
    /// paint。
    Paint,
    /// 多个值的位掩码组合。
    Custom(u8),
}

/// contain 属性的位标志常量。
impl ContainValue {
    /// size 标志位。
    pub const FLAG_SIZE: u8 = 0x01;
    /// layout 标志位。
    pub const FLAG_LAYOUT: u8 = 0x02;
    /// style 标志位。
    pub const FLAG_STYLE: u8 = 0x04;
    /// paint 标志位。
    pub const FLAG_PAINT: u8 = 0x08;
}

/// 解析 CSS contain 属性值。
///
/// 支持格式：
/// - `"none"` — 无包含。
/// - `"strict"` — 等价于 `layout style paint`。
/// - `"content"` — 等价于 `layout style paint size`。
/// - 单个关键字：`"size"`、`"layout"`、`"style"`、`"paint"`。
/// - 多个空格分隔的关键字：`"layout style paint"`。
pub fn parse_contain(value: &str) -> Option<ContainValue> {
    let value = value.trim().to_ascii_lowercase();

    match value.as_str() {
        "none" => Some(ContainValue::None),
        "strict" => Some(ContainValue::Strict),
        "content" => Some(ContainValue::Content),
        "size" => Some(ContainValue::Size),
        "layout" => Some(ContainValue::Layout),
        "style" => Some(ContainValue::Style),
        "paint" => Some(ContainValue::Paint),
        _ => {
            // 解析空格分隔的关键字列表
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty() {
                return None;
            }

            let mut flags: u8 = 0;
            for part in parts {
                match part {
                    "size" => flags |= ContainValue::FLAG_SIZE,
                    "layout" => flags |= ContainValue::FLAG_LAYOUT,
                    "style" => flags |= ContainValue::FLAG_STYLE,
                    "paint" => flags |= ContainValue::FLAG_PAINT,
                    _ => return None,
                }
            }

            if flags == 0 {
                None
            } else {
                Some(ContainValue::Custom(flags))
            }
        }
    }
}

// ── CSS Column 值类型 ──────────────────────────────────────────────

/// CSS column-count 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnCountValue {
    /// auto。
    Auto,
    /// 正整数值。
    Number(u32),
}

/// 解析 CSS column-count 属性值。
///
/// 支持格式如 `"auto"`、`"3"`。
pub fn parse_column_count(value: &str) -> Option<ColumnCountValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(ColumnCountValue::Auto);
    }
    let n: u32 = value.parse().ok()?;
    if n > 0 { Some(ColumnCountValue::Number(n)) } else { None }
}

/// CSS column-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnWidthValue {
    /// auto。
    Auto,
    /// 长度值。
    Length(LengthValue),
}

/// 解析 CSS column-width 属性值。
///
/// 支持格式如 `"auto"`、`"200px"`、`"10em"`。
pub fn parse_column_width(value: &str) -> Option<ColumnWidthValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(ColumnWidthValue::Auto);
    }
    parse_length(value).map(ColumnWidthValue::Length)
}

// ── CSS Object Fit 值类型 ──────────────────────────────────────────

/// CSS object-fit 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectFitValue {
    /// fill。
    Fill,
    /// contain。
    Contain,
    /// cover。
    Cover,
    /// none。
    None,
    /// scale-down。
    ScaleDown,
}

/// 解析 CSS object-fit 属性值。
pub fn parse_object_fit(value: &str) -> Option<ObjectFitValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fill" => Some(ObjectFitValue::Fill),
        "contain" => Some(ObjectFitValue::Contain),
        "cover" => Some(ObjectFitValue::Cover),
        "none" => Some(ObjectFitValue::None),
        "scale-down" => Some(ObjectFitValue::ScaleDown),
        _ => None,
    }
}

// ── CSS Filter 值类型 ──────────────────────────────────────────────

/// CSS filter 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum FilterValue {
    /// none。
    None,
    /// blur(px)。
    Blur(f32),
    /// brightness(number)。
    Brightness(f32),
    /// contrast(number)。
    Contrast(f32),
    /// grayscale(number)。
    Grayscale(f32),
    /// hue-rotate(deg)。
    HueRotate(f32),
    /// invert(number)。
    Invert(f32),
    /// opacity(number)。
    Opacity(f32),
    /// saturate(number)。
    Saturate(f32),
    /// sepia(number)。
    Sepia(f32),
    /// drop-shadow(x-offset, y-offset, blur-radius, color)。
    DropShadow(f32, f32, f32, ColorValue),
}

/// 解析 CSS filter 属性值。
///
/// 支持格式如 `"none"`、`"blur(5px)"`、`"brightness(1.5)"` 等。
pub fn parse_filter(value: &str) -> Option<FilterValue> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("none") {
        return Some(FilterValue::None);
    }

    // 解析单个 filter 函数
    if let Some(paren_pos) = value.find('(') {
        let func_name = value[..paren_pos].trim();
        if !value.ends_with(')') {
            return None;
        }
        let inner = value[paren_pos + 1..value.len() - 1].trim();

        match func_name.to_ascii_lowercase().as_str() {
            "blur" => {
                let px: f32 = parse_filter_length_px(inner)?;
                Some(FilterValue::Blur(px))
            }
            "brightness" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Brightness(n))
            }
            "contrast" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Contrast(n))
            }
            "grayscale" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Grayscale(n))
            }
            "hue-rotate" => {
                let deg: f32 = parse_filter_angle(inner)?;
                Some(FilterValue::HueRotate(deg))
            }
            "invert" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Invert(n))
            }
            "opacity" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Opacity(n))
            }
            "saturate" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Saturate(n))
            }
            "sepia" => {
                let n: f32 = parse_filter_number(inner)?;
                Some(FilterValue::Sepia(n))
            }
            "drop-shadow" => parse_drop_shadow(inner),
            _ => None,
        }
    } else {
        None
    }
}

/// 解析 filter 函数中的长度值（返回 px 数值）。
fn parse_filter_length_px(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with("px") {
        s.trim_end_matches("px").trim().parse::<f32>().ok()
    } else {
        // 无单位值在 blur 中无效，但尝试解析为纯数值
        s.parse::<f32>().ok()
    }
}

/// 解析 filter 函数中的数值（0-1 范围，也接受百分比和大于 1 的值）。
fn parse_filter_number(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with('%') {
        let pct: f32 = s.trim_end_matches('%').parse().ok()?;
        Some(pct / 100.0)
    } else {
        s.parse::<f32>().ok()
    }
}

/// 解析 filter 函数中的角度值（返回度数）。
fn parse_filter_angle(s: &str) -> Option<f32> {
    let s = s.trim();
    if s.ends_with("deg") {
        s.trim_end_matches("deg").trim().parse::<f32>().ok()
    } else if s.ends_with("rad") {
        let rad: f32 = s.trim_end_matches("rad").trim().parse().ok()?;
        Some(rad.to_degrees())
    } else if s.ends_with("turn") {
        let turn: f32 = s.trim_end_matches("turn").trim().parse().ok()?;
        Some(turn * 360.0)
    } else {
        s.parse::<f32>().ok()
    }
}

/// 解析 drop-shadow 参数。
///
/// 格式：`x-offset y-offset blur-radius color` 或 `x-offset y-offset color`。
fn parse_drop_shadow(inner: &str) -> Option<FilterValue> {
    // 简化解析：按空格分割，识别颜色值
    let parts: Vec<&str> = inner.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let x: f32 = parts[0].parse().ok()?;
    let y: f32 = parts[1].parse().ok()?;
    // 尝试解析第三个参数为 blur 或 color
    let (blur, color) = if parts.len() >= 4 {
        let blur: f32 = parts[2].parse().ok()?;
        let color = parse_color(parts[3..].join(" ").as_str())?;
        (blur, color)
    } else {
        // 第三个参数是颜色
        let color = parse_color(parts[2..].join(" ").as_str())?;
        (0.0, color)
    };

    Some(FilterValue::DropShadow(x, y, blur, color))
}

// ── CSS Appearance 值类型 ──────────────────────────────────────────────

/// CSS appearance 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AppearanceValue {
    /// none。
    None,
    /// auto。
    Auto,
    /// button。
    Button,
    /// checkbox。
    Checkbox,
    /// listbox。
    Listbox,
    /// menulist。
    Menulist,
    /// meter。
    Meter,
    /// progress-bar。
    ProgressBar,
    /// push-button。
    PushButton,
    /// radio。
    Radio,
    /// searchfield。
    Searchfield,
    /// slider-horizontal。
    SliderHorizontal,
    /// square-button。
    SquareButton,
    /// textarea。
    Textarea,
    /// textfield。
    Textfield,
}

/// 解析 CSS appearance 属性值。
pub fn parse_appearance(value: &str) -> Option<AppearanceValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(AppearanceValue::None),
        "auto" => Some(AppearanceValue::Auto),
        "button" => Some(AppearanceValue::Button),
        "checkbox" => Some(AppearanceValue::Checkbox),
        "listbox" => Some(AppearanceValue::Listbox),
        "menulist" => Some(AppearanceValue::Menulist),
        "meter" => Some(AppearanceValue::Meter),
        "progress-bar" => Some(AppearanceValue::ProgressBar),
        "push-button" => Some(AppearanceValue::PushButton),
        "radio" => Some(AppearanceValue::Radio),
        "searchfield" => Some(AppearanceValue::Searchfield),
        "slider-horizontal" => Some(AppearanceValue::SliderHorizontal),
        "square-button" => Some(AppearanceValue::SquareButton),
        "textarea" => Some(AppearanceValue::Textarea),
        "textfield" => Some(AppearanceValue::Textfield),
        _ => None,
    }
}

/// CSS accent-color 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum AccentColorValue {
    /// auto。
    Auto,
    /// 指定颜色。
    Color(ColorValue),
}

/// 解析 CSS accent-color 属性值。
///
/// 支持格式：`auto` 或任意有效 CSS 颜色值。
pub fn parse_accent_color(value: &str) -> Option<AccentColorValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(AccentColorValue::Auto);
    }
    parse_color(v).map(AccentColorValue::Color)
}

/// CSS caret-color 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum CaretColorValue {
    /// auto。
    Auto,
    /// 指定颜色。
    Color(ColorValue),
}

/// 解析 CSS caret-color 属性值。
///
/// 支持格式：`auto` 或任意有效 CSS 颜色值。
pub fn parse_caret_color(value: &str) -> Option<CaretColorValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(CaretColorValue::Auto);
    }
    parse_color(v).map(CaretColorValue::Color)
}

// ── CSS Mix Blend Mode 值类型 ──────────────────────────────────────────

/// CSS mix-blend-mode 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum MixBlendModeValue {
    /// normal（默认值）。
    Normal,
    /// multiply。
    Multiply,
    /// screen。
    Screen,
    /// overlay。
    Overlay,
    /// darken。
    Darken,
    /// lighten。
    Lighten,
    /// color-dodge。
    ColorDodge,
    /// color-burn。
    ColorBurn,
    /// hard-light。
    HardLight,
    /// soft-light。
    SoftLight,
    /// difference。
    Difference,
    /// exclusion。
    Exclusion,
    /// hue。
    Hue,
    /// saturation。
    Saturation,
    /// color。
    Color,
    /// luminosity。
    Luminosity,
}

/// 解析 CSS mix-blend-mode 属性值。
pub fn parse_mix_blend_mode(value: &str) -> Option<MixBlendModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(MixBlendModeValue::Normal),
        "multiply" => Some(MixBlendModeValue::Multiply),
        "screen" => Some(MixBlendModeValue::Screen),
        "overlay" => Some(MixBlendModeValue::Overlay),
        "darken" => Some(MixBlendModeValue::Darken),
        "lighten" => Some(MixBlendModeValue::Lighten),
        "color-dodge" => Some(MixBlendModeValue::ColorDodge),
        "color-burn" => Some(MixBlendModeValue::ColorBurn),
        "hard-light" => Some(MixBlendModeValue::HardLight),
        "soft-light" => Some(MixBlendModeValue::SoftLight),
        "difference" => Some(MixBlendModeValue::Difference),
        "exclusion" => Some(MixBlendModeValue::Exclusion),
        "hue" => Some(MixBlendModeValue::Hue),
        "saturation" => Some(MixBlendModeValue::Saturation),
        "color" => Some(MixBlendModeValue::Color),
        "luminosity" => Some(MixBlendModeValue::Luminosity),
        _ => None,
    }
}

/// CSS scrollbar-width 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollbarWidthValue {
    /// auto（默认值）— 浏览器默认滚动条宽度。
    Auto,
    /// thin — 细滚动条。
    Thin,
    /// none — 隐藏滚动条。
    None,
}

/// 解析 CSS scrollbar-width 属性值。
pub fn parse_scrollbar_width(value: &str) -> Option<ScrollbarWidthValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(ScrollbarWidthValue::Auto),
        "thin" => Some(ScrollbarWidthValue::Thin),
        "none" => Some(ScrollbarWidthValue::None),
        _ => None,
    }
}

/// CSS scrollbar-gutter 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollbarGutterValue {
    /// auto（默认值）— 仅在内容溢出时保留滚动条空间。
    Auto,
    /// stable — 始终保留滚动条空间。
    Stable,
    /// stable both-edges — 在两侧都保留滚动条空间。
    StableBothEdges,
}

/// 解析 CSS scrollbar-gutter 属性值。
///
/// 支持格式：`auto`、`stable`、`stable both-edges`。
pub fn parse_scrollbar_gutter(value: &str) -> Option<ScrollbarGutterValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(ScrollbarGutterValue::Auto),
        "stable" => Some(ScrollbarGutterValue::Stable),
        "stable both-edges" | "both-edges stable" => Some(ScrollbarGutterValue::StableBothEdges),
        _ => None,
    }
}

// ── CSS Text Wrap 值类型 ──────────────────────────────────────────────

/// CSS text-wrap 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextWrapValue {
    /// wrap（默认值）— 允许自动换行。
    Wrap,
    /// nowrap — 禁止自动换行。
    Nowrap,
    /// balance — 均衡换行。
    Balance,
    /// pretty — 优先美观换行。
    Pretty,
    /// stable — 稳定换行。
    Stable,
}

/// 解析 CSS text-wrap 属性值。
pub fn parse_text_wrap(value: &str) -> Option<TextWrapValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "wrap" => Some(TextWrapValue::Wrap),
        "nowrap" => Some(TextWrapValue::Nowrap),
        "balance" => Some(TextWrapValue::Balance),
        "pretty" => Some(TextWrapValue::Pretty),
        "stable" => Some(TextWrapValue::Stable),
        _ => None,
    }
}

// ── CSS Hyphens 值类型 ──────────────────────────────────────────────

/// CSS hyphens 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum HyphensValue {
    /// none（默认值）— 不使用连字符断词。
    None,
    /// manual — 手动断词（需使用软连字符）。
    Manual,
    /// auto — 自动断词。
    Auto,
}

/// 解析 CSS hyphens 属性值。
pub fn parse_hyphens(value: &str) -> Option<HyphensValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(HyphensValue::None),
        "manual" => Some(HyphensValue::Manual),
        "auto" => Some(HyphensValue::Auto),
        _ => None,
    }
}

// ── CSS Line Clamp 值类型 ──────────────────────────────────────────────

/// CSS line-clamp 属性值（-webkit-line-clamp）。
#[derive(Debug, Clone, PartialEq)]
pub enum LineClampValue {
    /// none（默认值）— 不限制行数。
    None,
    /// 限制为指定行数。
    Count(u32),
}

/// 解析 CSS line-clamp 属性值。
///
/// 支持格式如 `"none"`、`"3"`。
pub fn parse_line_clamp(value: &str) -> Option<LineClampValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("none") {
        return Some(LineClampValue::None);
    }
    let n: u32 = value.parse().ok()?;
    if n > 0 { Some(LineClampValue::Count(n)) } else { None }
}

// ── CSS Background 值类型 ──────────────────────────────────────────────

/// CSS background-image 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundImageValue {
    /// none（默认值）— 无背景图片。
    None,
    /// url(<string>) — 指定背景图片 URL。
    Url(String),
}

/// 解析 CSS background-image 属性值。
///
/// 支持格式如 `"none"`、`"url(image.png)"`。
pub fn parse_background_image(value: &str) -> Option<BackgroundImageValue> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("none") {
        return Some(BackgroundImageValue::None);
    }

    // 解析 url(...) 函数
    if value.starts_with("url(") && value.ends_with(')') {
        let inner = value.get(4..value.len() - 1)?;
        let url = inner.trim();
        // 去除可选的引号
        let url = if (url.starts_with('"') && url.ends_with('"')) || (url.starts_with('\'') && url.ends_with('\'')) {
            url.get(1..url.len() - 1)?
        } else {
            url
        };
        if url.is_empty() {
            return None;
        }
        return Some(BackgroundImageValue::Url(url.to_string()));
    }

    None
}

/// CSS background-position 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundPositionValue {
    /// center。
    Center,
    /// left。
    Left,
    /// right。
    Right,
    /// top。
    Top,
    /// bottom。
    Bottom,
    /// 长度值（如 10px）。
    Length(f32),
    /// 百分比值（如 50%）。
    Percent(f32),
    /// 两个值组合（水平 垂直）。
    TwoValue(Box<BackgroundPositionValue>, Box<BackgroundPositionValue>),
}

/// 解析 CSS background-position 属性值。
///
/// 支持单个关键字、长度/百分比，以及两个值的组合（水平 垂直）。
pub fn parse_background_position(value: &str) -> Option<BackgroundPositionValue> {
    let value = value.trim();
    let lower = value.to_ascii_lowercase();

    // 先检查是否为两个值组合
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if parts.len() == 2 {
        let first = parse_position_component(parts[0])?;
        let second = parse_position_component(parts[1])?;
        return Some(BackgroundPositionValue::TwoValue(Box::new(first), Box::new(second)));
    }

    // 单个关键字
    match lower.as_str() {
        "center" => return Some(BackgroundPositionValue::Center),
        "left" => return Some(BackgroundPositionValue::Left),
        "right" => return Some(BackgroundPositionValue::Right),
        "top" => return Some(BackgroundPositionValue::Top),
        "bottom" => return Some(BackgroundPositionValue::Bottom),
        _ => {}
    }

    // 单个百分比
    if lower.ends_with('%') {
        let pct: f32 = lower.trim_end_matches('%').parse().ok()?;
        return Some(BackgroundPositionValue::Percent(pct));
    }

    // 单个长度值
    if let Some(LengthValue::Px(px)) = parse_length(&lower) {
        return Some(BackgroundPositionValue::Length(px as f32));
    }

    None
}

/// 解析 background-position 的单个分量。
fn parse_position_component(s: &str) -> Option<BackgroundPositionValue> {
    match s {
        "center" => Some(BackgroundPositionValue::Center),
        "left" => Some(BackgroundPositionValue::Left),
        "right" => Some(BackgroundPositionValue::Right),
        "top" => Some(BackgroundPositionValue::Top),
        "bottom" => Some(BackgroundPositionValue::Bottom),
        _ => {
            if s.ends_with('%') {
                let pct: f32 = s.trim_end_matches('%').parse().ok()?;
                Some(BackgroundPositionValue::Percent(pct))
            } else if let Some(LengthValue::Px(px)) = parse_length(s) {
                Some(BackgroundPositionValue::Length(px as f32))
            } else {
                None
            }
        }
    }
}

/// CSS background-repeat 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundRepeatValue {
    /// repeat — 水平和垂直方向都重复。
    Repeat,
    /// repeat-x — 仅水平方向重复。
    RepeatX,
    /// repeat-y — 仅垂直方向重复。
    RepeatY,
    /// no-repeat — 不重复。
    NoRepeat,
    /// space — 均匀分布。
    Space,
    /// round — 缩放后重复。
    Round,
}

/// 解析 CSS background-repeat 属性值。
pub fn parse_background_repeat(value: &str) -> Option<BackgroundRepeatValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "repeat" => Some(BackgroundRepeatValue::Repeat),
        "repeat-x" => Some(BackgroundRepeatValue::RepeatX),
        "repeat-y" => Some(BackgroundRepeatValue::RepeatY),
        "no-repeat" => Some(BackgroundRepeatValue::NoRepeat),
        "space" => Some(BackgroundRepeatValue::Space),
        "round" => Some(BackgroundRepeatValue::Round),
        _ => None,
    }
}

/// CSS background-size 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundSizeValue {
    /// auto（默认值）— 背景图片保持原始尺寸。
    Auto,
    /// cover — 缩放图片以完全覆盖容器。
    Cover,
    /// contain — 缩放图片以完整显示在容器内。
    Contain,
    /// 长度值（如 100px）。
    Length(f32),
    /// 百分比值（如 50%）。
    Percent(f32),
}

/// 解析 CSS background-size 属性值。
///
/// 支持关键字（auto、cover、contain）和带单位的长度/百分比值。
pub fn parse_background_size(value: &str) -> Option<BackgroundSizeValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "auto" => Some(BackgroundSizeValue::Auto),
        "cover" => Some(BackgroundSizeValue::Cover),
        "contain" => Some(BackgroundSizeValue::Contain),
        _ => {
            if v.ends_with('%') {
                let pct: f32 = v.trim_end_matches('%').parse().ok()?;
                Some(BackgroundSizeValue::Percent(pct))
            } else if let Some(lv) = parse_length(&v) {
                match lv {
                    LengthValue::Px(n) => Some(BackgroundSizeValue::Length(n as f32)),
                    LengthValue::Em(n) => Some(BackgroundSizeValue::Length(n as f32)),
                    LengthValue::Rem(n) => Some(BackgroundSizeValue::Length(n as f32)),
                    _ => None,
                }
            } else {
                None
            }
        }
    }
}

/// CSS background-attachment 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundAttachmentValue {
    /// scroll（默认值）— 背景随元素内容滚动。
    Scroll,
    /// fixed — 背景相对于视口固定。
    Fixed,
    /// local — 背景随元素本地内容滚动。
    Local,
}

/// 解析 CSS background-attachment 属性值。
pub fn parse_background_attachment(value: &str) -> Option<BackgroundAttachmentValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "scroll" => Some(BackgroundAttachmentValue::Scroll),
        "fixed" => Some(BackgroundAttachmentValue::Fixed),
        "local" => Some(BackgroundAttachmentValue::Local),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_timing_function ──

    #[test]
    fn test_parse_timing_function_keywords() {
        assert_eq!(parse_timing_function("ease"), Some(TimingFunctionValue::Ease));
        assert_eq!(parse_timing_function("linear"), Some(TimingFunctionValue::Linear));
        assert_eq!(parse_timing_function("ease-in"), Some(TimingFunctionValue::EaseIn));
        assert_eq!(parse_timing_function("ease-out"), Some(TimingFunctionValue::EaseOut));
        assert_eq!(
            parse_timing_function("ease-in-out"),
            Some(TimingFunctionValue::EaseInOut)
        );
        assert_eq!(
            parse_timing_function("step-start"),
            Some(TimingFunctionValue::StepStart)
        );
        assert_eq!(parse_timing_function("step-end"), Some(TimingFunctionValue::StepEnd));
    }

    #[test]
    fn test_parse_timing_function_cubic_bezier() {
        let result = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)");
        assert_eq!(result, Some(TimingFunctionValue::CubicBezier(0.25, 0.1, 0.25, 1.0)));
    }

    #[test]
    fn test_parse_timing_function_steps() {
        assert_eq!(
            parse_timing_function("steps(4)"),
            Some(TimingFunctionValue::Steps(4, None))
        );
        assert_eq!(
            parse_timing_function("steps(4, end)"),
            Some(TimingFunctionValue::Steps(4, Some(StepPosition::End)))
        );
        assert_eq!(
            parse_timing_function("steps(4, start)"),
            Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start)))
        );
        assert_eq!(
            parse_timing_function("steps(2, jump-both)"),
            Some(TimingFunctionValue::Steps(2, Some(StepPosition::Both)))
        );
    }

    #[test]
    fn test_parse_timing_function_invalid() {
        assert_eq!(parse_timing_function("invalid"), None);
    }

    // ── parse_time ──

    #[test]
    fn test_parse_time_seconds() {
        assert_eq!(parse_time("0.3s"), Some(0.3));
        assert_eq!(parse_time("1s"), Some(1.0));
        assert_eq!(parse_time("2.5s"), Some(2.5));
    }

    #[test]
    fn test_parse_time_milliseconds() {
        assert_eq!(parse_time("200ms"), Some(0.2));
        assert_eq!(parse_time("1000ms"), Some(1.0));
        assert_eq!(parse_time("50ms"), Some(0.05));
    }

    #[test]
    fn test_parse_time_invalid() {
        assert_eq!(parse_time("10"), None);
        assert_eq!(parse_time("abc"), None);
    }

    #[test]
    fn test_parse_time_zero() {
        assert_eq!(parse_time("0s"), Some(0.0));
        assert_eq!(parse_time("0ms"), Some(0.0));
    }

    // ── parse_calc ──

    #[test]
    fn test_parse_calc_percentage_minus_px() {
        let expr = parse_calc("calc(100% - 20px)");
        let expr = expr.expect("should parse calc(100% - 20px)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Length(LengthValue::Percentage(100.0)));
                assert_eq!(*op, CalcOp::Subtract);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(20.0)));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_parse_calc_percentage_plus_px() {
        let expr = parse_calc("calc(50% + 10px)");
        let expr = expr.expect("should parse calc(50% + 10px)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Length(LengthValue::Percentage(50.0)));
                assert_eq!(*op, CalcOp::Add);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(10.0)));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_parse_calc_multiply() {
        let expr = parse_calc("calc(2 * 10px)");
        let expr = expr.expect("should parse calc(2 * 10px)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Number(2.0));
                assert_eq!(*op, CalcOp::Multiply);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(10.0)));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_parse_calc_divide() {
        let expr = parse_calc("calc(100px / 2)");
        let expr = expr.expect("should parse calc(100px / 2)");
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(**left, CalcExpr::Length(LengthValue::Px(100.0)));
                assert_eq!(*op, CalcOp::Divide);
                assert_eq!(**right, CalcExpr::Number(2.0));
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }
    }

    #[test]
    fn test_eval_calc_percentage_minus_px() {
        let expr = parse_calc("calc(100% - 20px)").unwrap();
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(180.0));
    }

    #[test]
    fn test_eval_calc_percentage_plus_px() {
        let expr = parse_calc("calc(50% + 10px)").unwrap();
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(110.0));
    }

    #[test]
    fn test_eval_calc_multiply() {
        let expr = parse_calc("calc(2 * 10px)").unwrap();
        let result = eval_calc(&expr, None);
        assert_eq!(result, Some(20.0));
    }

    #[test]
    fn test_eval_calc_divide() {
        let expr = parse_calc("calc(100px / 2)").unwrap();
        let result = eval_calc(&expr, None);
        assert_eq!(result, Some(50.0));
    }

    #[test]
    fn test_parse_calc_invalid() {
        assert_eq!(parse_calc("calc()"), None);
        assert_eq!(parse_calc("calc("), None);
        assert_eq!(parse_calc("not-a-calc"), None);
        assert_eq!(parse_calc(""), None);
    }

    #[test]
    fn test_eval_calc_percentage_without_parent() {
        let expr = parse_calc("calc(50% + 10px)").unwrap();
        // 百分比没有 parent_length，应返回 None
        assert_eq!(eval_calc(&expr, None), None);
    }

    // ── parse_calc 嵌套与优先级 ──

    #[test]
    /// 测试 calc() 基本嵌套：calc(calc(100% - 20px) / 2)
    fn test_calc_nested_basic() {
        let expr = parse_calc("calc(calc(100% - 20px) / 2)");
        let expr = expr.expect("should parse nested calc");
        // 整体结构：外层除法，左操作数为内层减法
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(*op, CalcOp::Divide);
                assert_eq!(**right, CalcExpr::Number(2.0));
                // 内层 calc(100% - 20px)
                match left.as_ref() {
                    CalcExpr::BinaryOp(inner_left, inner_op, inner_right) => {
                        assert_eq!(**inner_left, CalcExpr::Length(LengthValue::Percentage(100.0)));
                        assert_eq!(*inner_op, CalcOp::Subtract);
                        assert_eq!(**inner_right, CalcExpr::Length(LengthValue::Px(20.0)));
                    }
                    _ => panic!("expected inner BinaryOp, got {left:?}"),
                }
            }
            _ => panic!("expected outer BinaryOp, got {expr:?}"),
        }

        // 求值验证：parent_length=200, 100%-20px=180, 180/2=90
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(90.0));
    }

    #[test]
    /// 测试 calc() 双重嵌套：calc(calc(10px + 5px) * calc(2))
    fn test_calc_double_nesting() {
        let expr = parse_calc("calc(calc(10px + 5px) * calc(2))");
        let expr = expr.expect("should parse double nested calc");
        // 外层乘法
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(*op, CalcOp::Multiply);
                // 左侧 calc(10px + 5px)
                match left.as_ref() {
                    CalcExpr::BinaryOp(il, io, ir) => {
                        assert_eq!(**il, CalcExpr::Length(LengthValue::Px(10.0)));
                        assert_eq!(*io, CalcOp::Add);
                        assert_eq!(**ir, CalcExpr::Length(LengthValue::Px(5.0)));
                    }
                    _ => panic!("expected left inner BinaryOp, got {left:?}"),
                }
                // 右侧 calc(2)
                match right.as_ref() {
                    CalcExpr::Number(n) => assert_eq!(*n, 2.0),
                    _ => panic!("expected right Number, got {right:?}"),
                }
            }
            _ => panic!("expected outer BinaryOp, got {expr:?}"),
        }

        // 求值：(10+5)*2=30
        let result = eval_calc(&expr, None);
        assert_eq!(result, Some(30.0));
    }

    #[test]
    /// 测试 calc() 混合运算（运算符优先级与从左到右）：
    /// calc(100% - 10px + 5px) 应按从左到右顺序求值
    fn test_calc_mixed_operations() {
        let expr = parse_calc("calc(100% - 10px + 5px)");
        let expr = expr.expect("should parse mixed operations");
        // + 和 - 同优先级，从左到右：(100% - 10px) + 5px
        match &expr {
            CalcExpr::BinaryOp(left, op, right) => {
                assert_eq!(*op, CalcOp::Add);
                assert_eq!(**right, CalcExpr::Length(LengthValue::Px(5.0)));
                match left.as_ref() {
                    CalcExpr::BinaryOp(ll, lo, lr) => {
                        assert_eq!(**ll, CalcExpr::Length(LengthValue::Percentage(100.0)));
                        assert_eq!(*lo, CalcOp::Subtract);
                        assert_eq!(**lr, CalcExpr::Length(LengthValue::Px(10.0)));
                    }
                    _ => panic!("expected left BinaryOp, got {left:?}"),
                }
            }
            _ => panic!("expected BinaryOp, got {expr:?}"),
        }

        // 求值：parent_length=200, (200-10)+5=195
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(195.0));
    }

    // ── min() / max() / clamp() 解析与求值 ──

    /// 测试 min() 基本解析。
    #[test]
    fn test_parse_min_basic() {
        let expr = parse_min("min(100px, 50%)").unwrap();
        match &expr {
            CalcExpr::Min(args) => assert_eq!(args.len(), 2),
            _ => panic!("expected Min, got {expr:?}"),
        }
    }

    /// 测试 min() 多参数。
    #[test]
    fn test_parse_min_three_args() {
        let expr = parse_min("min(100px, 50%, 200px)").unwrap();
        match &expr {
            CalcExpr::Min(args) => assert_eq!(args.len(), 3),
            _ => panic!("expected Min, got {expr:?}"),
        }
    }

    /// 测试 min() 求值：取最小值。
    #[test]
    fn test_eval_min_basic() {
        let expr = parse_min("min(100px, 50%)").unwrap();
        // parent_length=300, 50%=150, min(100,150)=100
        let result = eval_calc(&expr, Some(300.0));
        assert_eq!(result, Some(100.0));
    }

    /// 测试 min() 求值：百分比更小。
    #[test]
    fn test_eval_min_percentage_smaller() {
        let expr = parse_min("min(200px, 25%)").unwrap();
        // parent_length=400, 25%=100, min(200,100)=100
        let result = eval_calc(&expr, Some(400.0));
        assert_eq!(result, Some(100.0));
    }

    /// 测试 min() 包含 calc() 嵌套。
    #[test]
    fn test_parse_min_with_calc() {
        let expr = parse_min("min(calc(100% - 20px), 300px)").unwrap();
        // parent_length=400, 100%-20px=380, min(380,300)=300
        let result = eval_calc(&expr, Some(400.0));
        assert_eq!(result, Some(300.0));
    }

    /// 测试 max() 基本解析。
    #[test]
    fn test_parse_max_basic() {
        let expr = parse_max("max(100px, 50%)").unwrap();
        match &expr {
            CalcExpr::Max(args) => assert_eq!(args.len(), 2),
            _ => panic!("expected Max, got {expr:?}"),
        }
    }

    /// 测试 max() 求值：取最大值。
    #[test]
    fn test_eval_max_basic() {
        let expr = parse_max("max(100px, 50%)").unwrap();
        // parent_length=300, 50%=150, max(100,150)=150
        let result = eval_calc(&expr, Some(300.0));
        assert_eq!(result, Some(150.0));
    }

    /// 测试 max() 三参数求值。
    #[test]
    fn test_eval_max_three_args() {
        let expr = parse_max("max(10px, 20px, 15px)").unwrap();
        let result = eval_calc(&expr, None);
        assert_eq!(result, Some(20.0));
    }

    /// 测试 clamp() 基本解析。
    #[test]
    fn test_parse_clamp_basic() {
        let expr = parse_clamp("clamp(100px, 50%, 300px)").unwrap();
        match &expr {
            CalcExpr::Clamp { min, val, max } => {
                assert_eq!(**min, CalcExpr::Length(LengthValue::Px(100.0)));
                assert_eq!(**val, CalcExpr::Length(LengthValue::Percentage(50.0)));
                assert_eq!(**max, CalcExpr::Length(LengthValue::Px(300.0)));
            }
            _ => panic!("expected Clamp, got {expr:?}"),
        }
    }

    /// 测试 clamp() 求值：val 在范围内。
    #[test]
    fn test_eval_clamp_in_range() {
        let expr = parse_clamp("clamp(100px, 50%, 300px)").unwrap();
        // parent_length=400, 50%=200, clamp(100,200,300)=200
        let result = eval_calc(&expr, Some(400.0));
        assert_eq!(result, Some(200.0));
    }

    /// 测试 clamp() 求值：val 小于 min，结果为 min。
    #[test]
    fn test_eval_clamp_below_min() {
        let expr = parse_clamp("clamp(100px, 10%, 300px)").unwrap();
        // parent_length=400, 10%=40, clamp(100,40,300)=100
        let result = eval_calc(&expr, Some(400.0));
        assert_eq!(result, Some(100.0));
    }

    /// 测试 clamp() 求值：val 大于 max，结果为 max。
    #[test]
    fn test_eval_clamp_above_max() {
        let expr = parse_clamp("clamp(100px, 80%, 300px)").unwrap();
        // parent_length=400, 80%=320, clamp(100,320,300)=300
        let result = eval_calc(&expr, Some(400.0));
        assert_eq!(result, Some(300.0));
    }

    /// 测试 parse_math_function 分发。
    #[test]
    fn test_parse_math_function_dispatch() {
        assert!(parse_math_function("calc(100px + 10px)").is_some());
        assert!(parse_math_function("min(100px, 50%)").is_some());
        assert!(parse_math_function("max(100px, 50%)").is_some());
        assert!(parse_math_function("clamp(100px, 50%, 300px)").is_some());
        assert!(parse_math_function("invalid(100px)").is_none());
    }

    /// 测试 min()/max()/clamp() 无效输入。
    #[test]
    fn test_parse_min_max_clamp_invalid() {
        assert_eq!(parse_min(""), None);
        assert_eq!(parse_min("min()"), None);
        assert_eq!(parse_min("min("), None);
        assert_eq!(parse_max(""), None);
        assert_eq!(parse_max("max()"), None);
        assert_eq!(parse_clamp(""), None);
        assert_eq!(parse_clamp("clamp()"), None);
        assert_eq!(parse_clamp("clamp(100px, 50%)"), None); // 缺少第三个参数
    }

    /// 测试 min()/max() 嵌套使用。
    #[test]
    fn test_parse_min_nested_max() {
        let expr = parse_min("min(max(100px, 50%), 300px)").unwrap();
        // parent_length=200, max(100,100)=100, min(100,300)=100
        let result = eval_calc(&expr, Some(200.0));
        assert_eq!(result, Some(100.0));
    }

    /// 测试 clamp() 内部使用 calc()。
    #[test]
    fn test_parse_clamp_with_calc() {
        let expr = parse_clamp("clamp(50px, calc(100% - 20px), 500px)").unwrap();
        // parent_length=400, 100%-20px=380, clamp(50,380,500)=380
        let result = eval_calc(&expr, Some(400.0));
        assert_eq!(result, Some(380.0));
    }

    // ── float/clear 解析测试 ──

    #[test]
    fn test_parse_float_values() {
        assert_eq!(parse_float("left"), Some(FloatValue::Left));
        assert_eq!(parse_float("right"), Some(FloatValue::Right));
        assert_eq!(parse_float("none"), Some(FloatValue::None));
        assert_eq!(parse_float("inline-start"), Some(FloatValue::InlineStart));
        assert_eq!(parse_float("inline-end"), Some(FloatValue::InlineEnd));
        assert_eq!(parse_float("center"), None);
        assert_eq!(parse_float(""), None);
    }

    #[test]
    fn test_parse_clear_values() {
        assert_eq!(parse_clear("left"), Some(ClearValue::Left));
        assert_eq!(parse_clear("right"), Some(ClearValue::Right));
        assert_eq!(parse_clear("both"), Some(ClearValue::Both));
        assert_eq!(parse_clear("none"), Some(ClearValue::None));
        assert_eq!(parse_clear("inline-start"), Some(ClearValue::InlineStart));
        assert_eq!(parse_clear("inline-end"), Some(ClearValue::InlineEnd));
        assert_eq!(parse_clear("all"), None);
    }

    #[test]
    fn test_parse_float_case_insensitive() {
        // CSS 关键字不区分大小写
        assert_eq!(parse_float("LEFT"), Some(FloatValue::Left));
        assert_eq!(parse_float(" Left "), Some(FloatValue::Left));
        assert_eq!(parse_float("None"), Some(FloatValue::None));
    }

    #[test]
    fn test_parse_clear_whitespace() {
        assert_eq!(parse_clear("  both  "), Some(ClearValue::Both));
    }

    // ── list-style 解析测试 ──

    #[test]
    fn test_parse_list_style_type_values() {
        assert_eq!(parse_list_style_type("disc"), Some(ListStyleTypeValue::Disc));
        assert_eq!(parse_list_style_type("circle"), Some(ListStyleTypeValue::Circle));
        assert_eq!(parse_list_style_type("square"), Some(ListStyleTypeValue::Square));
        assert_eq!(parse_list_style_type("decimal"), Some(ListStyleTypeValue::Decimal));
        assert_eq!(
            parse_list_style_type("decimal-leading-zero"),
            Some(ListStyleTypeValue::DecimalLeadingZero)
        );
        assert_eq!(
            parse_list_style_type("lower-roman"),
            Some(ListStyleTypeValue::LowerRoman)
        );
        assert_eq!(
            parse_list_style_type("upper-roman"),
            Some(ListStyleTypeValue::UpperRoman)
        );
        assert_eq!(
            parse_list_style_type("lower-alpha"),
            Some(ListStyleTypeValue::LowerAlpha)
        );
        assert_eq!(
            parse_list_style_type("lower-latin"),
            Some(ListStyleTypeValue::LowerAlpha)
        );
        assert_eq!(
            parse_list_style_type("upper-alpha"),
            Some(ListStyleTypeValue::UpperAlpha)
        );
        assert_eq!(parse_list_style_type("none"), Some(ListStyleTypeValue::None));
        assert_eq!(parse_list_style_type("invalid"), None);
    }

    #[test]
    fn test_parse_list_style_type_case_insensitive() {
        assert_eq!(parse_list_style_type("DISC"), Some(ListStyleTypeValue::Disc));
        assert_eq!(parse_list_style_type("Decimal"), Some(ListStyleTypeValue::Decimal));
    }

    #[test]
    fn test_parse_list_style_position_values() {
        assert_eq!(
            parse_list_style_position("outside"),
            Some(ListStylePositionValue::Outside)
        );
        assert_eq!(
            parse_list_style_position("inside"),
            Some(ListStylePositionValue::Inside)
        );
        assert_eq!(parse_list_style_position("center"), None);
    }

    // ── parse_grid_area 测试 ──

    #[test]
    /// 测试 grid-area 单个命名区域
    fn test_parse_grid_area_named_area() {
        let result = parse_grid_area("header").unwrap();
        assert_eq!(
            result,
            (
                "header".to_string(),
                "header".to_string(),
                "header".to_string(),
                "header".to_string()
            )
        );
    }

    #[test]
    /// 测试 grid-area auto
    fn test_parse_grid_area_auto() {
        let result = parse_grid_area("auto").unwrap();
        assert_eq!(
            result,
            (
                "auto".to_string(),
                "auto".to_string(),
                "auto".to_string(),
                "auto".to_string()
            )
        );
    }

    #[test]
    /// 测试 grid-area 四值斜杠分隔
    fn test_parse_grid_area_four_values() {
        let result = parse_grid_area("1 / 2 / 3 / 4").unwrap();
        assert_eq!(
            result,
            ("1".to_string(), "2".to_string(), "3".to_string(), "4".to_string())
        );
    }

    #[test]
    /// 测试 grid-area 两值斜杠分隔（row-start / col-start）
    fn test_parse_grid_area_two_values() {
        let result = parse_grid_area("1 / 3").unwrap();
        assert_eq!(
            result,
            ("1".to_string(), "auto".to_string(), "3".to_string(), "auto".to_string())
        );
    }

    #[test]
    /// 测试 grid-area 三值斜杠分隔
    fn test_parse_grid_area_three_values() {
        let result = parse_grid_area("1 / 2 / 3").unwrap();
        assert_eq!(
            result,
            ("1".to_string(), "2".to_string(), "3".to_string(), "auto".to_string())
        );
    }

    #[test]
    /// 测试 grid-area 包含 span 关键字
    fn test_parse_grid_area_span() {
        let result = parse_grid_area("span 2 / span 3 / span 1 / span 4").unwrap();
        assert_eq!(
            result,
            (
                "span 2".to_string(),
                "span 3".to_string(),
                "span 1".to_string(),
                "span 4".to_string()
            )
        );
    }

    #[test]
    /// 测试 grid-area 带空白
    fn test_parse_grid_area_whitespace() {
        let result = parse_grid_area("  header  ").unwrap();
        assert_eq!(
            result,
            (
                "header".to_string(),
                "header".to_string(),
                "header".to_string(),
                "header".to_string()
            )
        );

        let result = parse_grid_area("  1  /  2  /  3  /  4  ").unwrap();
        assert_eq!(
            result,
            ("1".to_string(), "2".to_string(), "3".to_string(), "4".to_string())
        );
    }

    #[test]
    /// 测试 grid-area 无效输入
    fn test_parse_grid_area_invalid() {
        assert_eq!(parse_grid_area(""), None);
        assert_eq!(parse_grid_area("   "), None);
    }

    // ── parse_word_break 测试 ──

    #[test]
    fn test_parse_word_break_normal() {
        assert_eq!(parse_word_break("normal"), Some(WordBreakValue::Normal));
    }

    #[test]
    fn test_parse_word_break_break_all() {
        assert_eq!(parse_word_break("break-all"), Some(WordBreakValue::BreakAll));
    }

    #[test]
    fn test_parse_word_break_keep_all() {
        assert_eq!(parse_word_break("keep-all"), Some(WordBreakValue::KeepAll));
    }

    #[test]
    fn test_parse_word_break_invalid() {
        assert_eq!(parse_word_break("invalid"), None);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 3D Transform 函数解析测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    /// 测试 rotateX(45deg) 解析
    fn test_parse_rotate_x() {
        let result = parse_transform("rotateX(45deg)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::RotateX(45.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 rotateY(-90deg) 使用 rad 单位
    fn test_parse_rotate_y() {
        // -π/2 rad ≈ -90°
        let result = parse_transform("rotateY(-1.5708rad)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                let angle = match &fns[0] {
                    TransformFunction::RotateY(a) => *a,
                    other => panic!("expected RotateY, got {other:?}"),
                };
                assert!((angle - (-90.0)).abs() < 1.0, "angle should be near -90, got {angle}");
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 rotateZ(0.5turn) 解析（0.5 圈 = 180°）
    fn test_parse_rotate_z() {
        let result = parse_transform("rotateZ(0.5turn)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::RotateZ(180.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 translate3d(10px, 20px, 30px) 解析
    fn test_parse_translate_3d() {
        let result = parse_transform("translate3d(10px, 20px, 30px)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::Translate3d(10.0, 20.0, 30.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 scale3d(1, 2, 3) 解析
    fn test_parse_scale_3d() {
        let result = parse_transform("scale3d(1, 2, 3)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::Scale3d(1.0, 2.0, 3.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 rotate3d(1, 0, 0, 45deg) 解析
    fn test_parse_rotate_3d() {
        let result = parse_transform("rotate3d(1, 0, 0, 45deg)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::Rotate3d(1.0, 0.0, 0.0, 45.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 perspective(500px) 解析
    fn test_parse_perspective_func() {
        let result = parse_transform("perspective(500px)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::Perspective(500.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }

        // perspective(0) 应返回 None（必须为正数）
        assert!(parse_transform("perspective(0)").is_none());
        // perspective(-100) 应返回 None（必须为正数）
        assert!(parse_transform("perspective(-100)").is_none());
    }

    #[test]
    /// 测试 matrix(1, 0, 0, 1, 10, 20) 解析
    fn test_parse_matrix() {
        let result = parse_transform("matrix(1, 0, 0, 1, 10, 20)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 1);
                assert_eq!(fns[0], TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 10.0, 20.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }

        // matrix 需要 6 个参数
        assert!(parse_transform("matrix(1, 0, 0)").is_none());
    }

    #[test]
    /// 测试组合 3D 变换：translate3d(10px, 0, 0) rotateY(45deg)
    fn test_parse_combined_3d_transforms() {
        let result = parse_transform("translate3d(10px, 0, 0) rotateY(45deg)").unwrap();
        match result {
            TransformValue::List(fns) => {
                assert_eq!(fns.len(), 2);
                assert_eq!(fns[0], TransformFunction::Translate3d(10.0, 0.0, 0.0));
                assert_eq!(fns[1], TransformFunction::RotateY(45.0));
            }
            _ => panic!("expected TransformValue::List, got {result:?}"),
        }
    }

    #[test]
    /// 测试 transform: none 返回 None 变体
    fn test_parse_transform_none() {
        let result = parse_transform("none").unwrap();
        assert_eq!(result, TransformValue::None);
    }

    #[test]
    /// 测试 transform 无效输入
    fn test_parse_transform_invalid() {
        assert!(parse_transform("").is_none());
        assert!(parse_transform("unknown(10px)").is_none());
    }

    // ═══════════════════════════════════════════════════════════════════
    // Counter / Content / Quotes 解析测试
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_parse_counter_action_name_only() {
        let result = parse_counter_action("section").unwrap();
        assert_eq!(result.name, "section");
        assert_eq!(result.value, None);
    }

    #[test]
    fn test_parse_counter_action_with_value() {
        let result = parse_counter_action("section 5").unwrap();
        assert_eq!(result.name, "section");
        assert_eq!(result.value, Some(5));
    }

    #[test]
    fn test_parse_counter_action_negative() {
        let result = parse_counter_action("counter -3").unwrap();
        assert_eq!(result.name, "counter");
        assert_eq!(result.value, Some(-3));
    }

    #[test]
    fn test_parse_counter_action_none_rejected() {
        assert_eq!(parse_counter_action("none"), None);
        assert_eq!(parse_counter_action(""), None);
    }

    #[test]
    fn test_parse_counter_list_none() {
        let result = parse_counter_list("none").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_counter_list_single() {
        let result = parse_counter_list("section").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "section");
        assert_eq!(result[0].value, None);
    }

    #[test]
    fn test_parse_counter_list_multiple() {
        let result = parse_counter_list("section 1 subsection").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "section");
        assert_eq!(result[0].value, Some(1));
        assert_eq!(result[1].name, "subsection");
        assert_eq!(result[1].value, None);
    }

    #[test]
    fn test_parse_counter_list_invalid() {
        assert_eq!(parse_counter_list(""), None);
    }

    #[test]
    fn test_parse_content_normal() {
        assert_eq!(parse_content("normal"), Some(ContentValue::Normal));
    }

    #[test]
    fn test_parse_content_none() {
        assert_eq!(parse_content("none"), Some(ContentValue::None));
    }

    #[test]
    fn test_parse_content_string_double_quotes() {
        assert_eq!(
            parse_content("\"Hello\""),
            Some(ContentValue::String("Hello".to_string()))
        );
    }

    #[test]
    fn test_parse_content_string_single_quotes() {
        assert_eq!(
            parse_content("'World'"),
            Some(ContentValue::String("World".to_string()))
        );
    }

    #[test]
    fn test_parse_content_attr() {
        assert_eq!(
            parse_content("attr(data-label)"),
            Some(ContentValue::Attr("data-label".to_string()))
        );
    }

    #[test]
    fn test_parse_content_counter_no_style() {
        let result = parse_content("counter(section)").unwrap();
        match result {
            ContentValue::Counter { name, style } => {
                assert_eq!(name, "section");
                assert_eq!(style, None);
            }
            _ => panic!("expected Counter variant"),
        }
    }

    #[test]
    fn test_parse_content_counter_with_style() {
        let result = parse_content("counter(section, upper-roman)").unwrap();
        match result {
            ContentValue::Counter { name, style } => {
                assert_eq!(name, "section");
                assert_eq!(style, Some("upper-roman".to_string()));
            }
            _ => panic!("expected Counter variant"),
        }
    }

    #[test]
    fn test_parse_content_invalid() {
        assert_eq!(parse_content("unknown-value"), None);
        assert_eq!(parse_content(""), None);
    }

    #[test]
    fn test_parse_quotes_none() {
        assert_eq!(parse_quotes("none"), Some(QuotesValue::None));
    }

    #[test]
    fn test_parse_quotes_auto() {
        assert_eq!(parse_quotes("auto"), Some(QuotesValue::Auto));
    }

    #[test]
    fn test_parse_quotes_single_pair() {
        let result = parse_quotes(r#""«" "»""#).unwrap();
        match result {
            QuotesValue::Pairs(pairs) => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
            }
            _ => panic!("expected Pairs"),
        }
    }

    #[test]
    fn test_parse_quotes_two_pairs() {
        let result = parse_quotes(r#""«" "»" "‹" "›""#).unwrap();
        match result {
            QuotesValue::Pairs(pairs) => {
                assert_eq!(pairs.len(), 2);
                assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
                assert_eq!(pairs[1], ("‹".to_string(), "›".to_string()));
            }
            _ => panic!("expected Pairs"),
        }
    }

    #[test]
    fn test_parse_quotes_single_quotes() {
        let result = parse_quotes("'\"' '\"'").unwrap();
        match result {
            QuotesValue::Pairs(pairs) => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0], ("\"".to_string(), "\"".to_string()));
            }
            _ => panic!("expected Pairs"),
        }
    }

    #[test]
    fn test_parse_quotes_invalid() {
        assert_eq!(parse_quotes(""), None);
        assert_eq!(parse_quotes("random"), None);
    }

    // ── Page Break 测试 ──

    #[test]
    fn test_parse_page_break_auto() {
        assert_eq!(parse_page_break("auto"), Some(PageBreakValue::Auto));
    }

    #[test]
    fn test_parse_page_break_always() {
        assert_eq!(parse_page_break("always"), Some(PageBreakValue::Always));
    }

    #[test]
    fn test_parse_page_break_avoid() {
        assert_eq!(parse_page_break("avoid"), Some(PageBreakValue::Avoid));
    }

    #[test]
    fn test_parse_page_break_left_right() {
        assert_eq!(parse_page_break("left"), Some(PageBreakValue::Left));
        assert_eq!(parse_page_break("right"), Some(PageBreakValue::Right));
    }

    #[test]
    fn test_parse_page_break_invalid() {
        assert_eq!(parse_page_break("invalid"), None);
    }

    // ── BoxDecorationBreak 测试 ──

    #[test]
    fn test_parse_box_decoration_break() {
        assert_eq!(
            parse_box_decoration_break("slice"),
            Some(BoxDecorationBreakValue::Slice)
        );
        assert_eq!(
            parse_box_decoration_break("clone"),
            Some(BoxDecorationBreakValue::Clone)
        );
        assert_eq!(parse_box_decoration_break("invalid"), None);
    }

    // ── ImageRendering 测试 ──

    #[test]
    fn test_parse_image_rendering() {
        assert_eq!(parse_image_rendering("auto"), Some(ImageRenderingValue::Auto));
        assert_eq!(parse_image_rendering("smooth"), Some(ImageRenderingValue::Smooth));
        assert_eq!(
            parse_image_rendering("high-quality"),
            Some(ImageRenderingValue::HighQuality)
        );
        assert_eq!(parse_image_rendering("pixelated"), Some(ImageRenderingValue::Pixelated));
        assert_eq!(
            parse_image_rendering("crisp-edges"),
            Some(ImageRenderingValue::CrispEdges)
        );
        assert_eq!(parse_image_rendering("invalid"), None);
    }

    // ── Isolation 测试 ──

    #[test]
    fn test_parse_isolation() {
        assert_eq!(parse_isolation("auto"), Some(IsolationValue::Auto));
        assert_eq!(parse_isolation("isolate"), Some(IsolationValue::Isolate));
        assert_eq!(parse_isolation("invalid"), None);
    }

    // ── OverscrollBehavior 测试 ──

    #[test]
    fn test_parse_overscroll_behavior() {
        assert_eq!(parse_overscroll_behavior("auto"), Some(OverscrollBehaviorValue::Auto));
        assert_eq!(
            parse_overscroll_behavior("contain"),
            Some(OverscrollBehaviorValue::Contain)
        );
        assert_eq!(parse_overscroll_behavior("none"), Some(OverscrollBehaviorValue::None));
        assert_eq!(parse_overscroll_behavior("invalid"), None);
    }

    #[test]
    fn test_parse_overscroll_behavior_case_insensitive() {
        assert_eq!(parse_overscroll_behavior("AUTO"), Some(OverscrollBehaviorValue::Auto));
        assert_eq!(
            parse_overscroll_behavior(" Contain "),
            Some(OverscrollBehaviorValue::Contain)
        );
    }

    // ── TouchAction 测试 ──

    #[test]
    fn test_parse_touch_action() {
        assert_eq!(parse_touch_action("auto"), Some(TouchActionValue::Auto));
        assert_eq!(parse_touch_action("none"), Some(TouchActionValue::None));
        assert_eq!(parse_touch_action("pan-x"), Some(TouchActionValue::PanX));
        assert_eq!(parse_touch_action("pan-y"), Some(TouchActionValue::PanY));
        assert_eq!(parse_touch_action("manipulation"), Some(TouchActionValue::Manipulation));
        assert_eq!(parse_touch_action("invalid"), None);
    }

    #[test]
    fn test_parse_touch_action_pan_both() {
        assert_eq!(parse_touch_action("pan-x pan-y"), Some(TouchActionValue::PanXPanY));
        assert_eq!(parse_touch_action("pan-y pan-x"), Some(TouchActionValue::PanXPanY));
    }

    // ── UserSelect 测试 ──

    #[test]
    fn test_parse_user_select() {
        assert_eq!(parse_user_select("auto"), Some(UserSelectValue::Auto));
        assert_eq!(parse_user_select("text"), Some(UserSelectValue::Text));
        assert_eq!(parse_user_select("none"), Some(UserSelectValue::None));
        assert_eq!(parse_user_select("all"), Some(UserSelectValue::All));
        assert_eq!(parse_user_select("contain"), Some(UserSelectValue::Contain));
        assert_eq!(parse_user_select("invalid"), None);
    }

    // ── WillChange 测试 ──

    #[test]
    fn test_parse_will_change() {
        assert_eq!(parse_will_change("auto"), Some(WillChangeValue::Auto));
        assert_eq!(
            parse_will_change("scroll-position"),
            Some(WillChangeValue::ScrollPosition)
        );
        assert_eq!(parse_will_change("contents"), Some(WillChangeValue::Contents));
        assert_eq!(
            parse_will_change("transform"),
            Some(WillChangeValue::Custom("transform".to_string()))
        );
        assert_eq!(
            parse_will_change("opacity"),
            Some(WillChangeValue::Custom("opacity".to_string()))
        );
        assert_eq!(parse_will_change(""), None);
    }

    // ── PointerEvents 测试 ──

    #[test]
    fn test_parse_pointer_events() {
        assert_eq!(parse_pointer_events("auto"), Some(PointerEventsValue::Auto));
        assert_eq!(parse_pointer_events("none"), Some(PointerEventsValue::None));
        assert_eq!(
            parse_pointer_events("visiblePainted"),
            Some(PointerEventsValue::VisiblePainted)
        );
        assert_eq!(
            parse_pointer_events("visibleFill"),
            Some(PointerEventsValue::VisibleFill)
        );
        assert_eq!(
            parse_pointer_events("visibleStroke"),
            Some(PointerEventsValue::VisibleStroke)
        );
        assert_eq!(parse_pointer_events("visible"), Some(PointerEventsValue::Visible));
        assert_eq!(parse_pointer_events("painted"), Some(PointerEventsValue::Painted));
        assert_eq!(parse_pointer_events("fill"), Some(PointerEventsValue::Fill));
        assert_eq!(parse_pointer_events("stroke"), Some(PointerEventsValue::Stroke));
        assert_eq!(parse_pointer_events("all"), Some(PointerEventsValue::All));
        assert_eq!(parse_pointer_events("inherit"), Some(PointerEventsValue::Inherit));
        assert_eq!(parse_pointer_events("invalid"), None);
    }

    #[test]
    fn test_parse_pointer_events_case_insensitive() {
        assert_eq!(parse_pointer_events("NONE"), Some(PointerEventsValue::None));
        assert_eq!(
            parse_pointer_events(" VisiblePainted "),
            Some(PointerEventsValue::VisiblePainted)
        );
    }

    // ── OverflowWrap 测试 ──

    #[test]
    fn test_parse_overflow_wrap_normal() {
        assert_eq!(parse_overflow_wrap("normal"), Some(OverflowWrapValue::Normal));
    }

    #[test]
    fn test_parse_overflow_wrap_break_word() {
        assert_eq!(parse_overflow_wrap("break-word"), Some(OverflowWrapValue::BreakWord));
    }

    #[test]
    fn test_parse_overflow_wrap_anywhere() {
        assert_eq!(parse_overflow_wrap("anywhere"), Some(OverflowWrapValue::Anywhere));
    }

    #[test]
    fn test_parse_overflow_wrap_invalid() {
        assert_eq!(parse_overflow_wrap("invalid"), None);
    }

    #[test]
    fn test_parse_overflow_wrap_case_insensitive() {
        assert_eq!(parse_overflow_wrap("BREAK-WORD"), Some(OverflowWrapValue::BreakWord));
        assert_eq!(parse_overflow_wrap(" Anywhere "), Some(OverflowWrapValue::Anywhere));
    }

    // ── TextAlignLast 测试 ──

    #[test]
    fn test_parse_text_align_last_auto() {
        assert_eq!(parse_text_align_last("auto"), Some(TextAlignLastValue::Auto));
    }

    #[test]
    fn test_parse_text_align_last_start_end() {
        assert_eq!(parse_text_align_last("start"), Some(TextAlignLastValue::Start));
        assert_eq!(parse_text_align_last("end"), Some(TextAlignLastValue::End));
    }

    #[test]
    fn test_parse_text_align_last_left_right_center() {
        assert_eq!(parse_text_align_last("left"), Some(TextAlignLastValue::Left));
        assert_eq!(parse_text_align_last("right"), Some(TextAlignLastValue::Right));
        assert_eq!(parse_text_align_last("center"), Some(TextAlignLastValue::Center));
    }

    #[test]
    fn test_parse_text_align_last_justify() {
        assert_eq!(parse_text_align_last("justify"), Some(TextAlignLastValue::Justify));
    }

    #[test]
    fn test_parse_text_align_last_invalid() {
        assert_eq!(parse_text_align_last("invalid"), None);
    }

    #[test]
    fn test_parse_text_align_last_case_insensitive() {
        assert_eq!(parse_text_align_last("JUSTIFY"), Some(TextAlignLastValue::Justify));
        assert_eq!(parse_text_align_last(" Center "), Some(TextAlignLastValue::Center));
    }

    // ── FontVariantNumeric 测试 ──

    #[test]
    fn test_parse_font_variant_numeric_normal() {
        assert_eq!(
            parse_font_variant_numeric("normal"),
            Some(FontVariantNumericValue::Normal)
        );
    }

    #[test]
    fn test_parse_font_variant_numeric_ordinal() {
        assert_eq!(
            parse_font_variant_numeric("ordinal"),
            Some(FontVariantNumericValue::Ordinal)
        );
    }

    #[test]
    fn test_parse_font_variant_numeric_slashed_zero() {
        assert_eq!(
            parse_font_variant_numeric("slashed-zero"),
            Some(FontVariantNumericValue::SlashedZero)
        );
    }

    #[test]
    fn test_parse_font_variant_numeric_num_styles() {
        assert_eq!(
            parse_font_variant_numeric("lining-nums"),
            Some(FontVariantNumericValue::LiningNums)
        );
        assert_eq!(
            parse_font_variant_numeric("oldstyle-nums"),
            Some(FontVariantNumericValue::OldstyleNums)
        );
        assert_eq!(
            parse_font_variant_numeric("proportional-nums"),
            Some(FontVariantNumericValue::ProportionalNums)
        );
        assert_eq!(
            parse_font_variant_numeric("tabular-nums"),
            Some(FontVariantNumericValue::TabularNums)
        );
    }

    #[test]
    fn test_parse_font_variant_numeric_fractions() {
        assert_eq!(
            parse_font_variant_numeric("diagonal-fractions"),
            Some(FontVariantNumericValue::DiagonalFractions)
        );
        assert_eq!(
            parse_font_variant_numeric("stacked-fractions"),
            Some(FontVariantNumericValue::StackedFractions)
        );
    }

    #[test]
    fn test_parse_font_variant_numeric_invalid() {
        assert_eq!(parse_font_variant_numeric("invalid"), None);
    }

    #[test]
    fn test_parse_font_variant_numeric_case_insensitive() {
        assert_eq!(
            parse_font_variant_numeric("ORDINAL"),
            Some(FontVariantNumericValue::Ordinal)
        );
        assert_eq!(
            parse_font_variant_numeric(" Lining-Nums "),
            Some(FontVariantNumericValue::LiningNums)
        );
    }

    // ── Direction 测试 ──

    #[test]
    fn test_parse_direction_ltr() {
        assert_eq!(parse_direction("ltr"), Some(DirectionValue::Ltr));
    }

    #[test]
    fn test_parse_direction_rtl() {
        assert_eq!(parse_direction("rtl"), Some(DirectionValue::Rtl));
    }

    #[test]
    fn test_parse_direction_case_insensitive() {
        assert_eq!(parse_direction("LTR"), Some(DirectionValue::Ltr));
        assert_eq!(parse_direction("Rtl"), Some(DirectionValue::Rtl));
        assert_eq!(parse_direction("  ltr  "), Some(DirectionValue::Ltr));
    }

    #[test]
    fn test_parse_direction_invalid() {
        assert_eq!(parse_direction("invalid"), None);
        assert_eq!(parse_direction(""), None);
    }

    // ── UnicodeBidi 测试 ──

    #[test]
    fn test_parse_unicode_bidi_normal() {
        assert_eq!(parse_unicode_bidi("normal"), Some(UnicodeBidiValue::Normal));
    }

    #[test]
    fn test_parse_unicode_bidi_all_values() {
        assert_eq!(parse_unicode_bidi("embed"), Some(UnicodeBidiValue::Embed));
        assert_eq!(parse_unicode_bidi("isolate"), Some(UnicodeBidiValue::Isolate));
        assert_eq!(
            parse_unicode_bidi("bidi-override"),
            Some(UnicodeBidiValue::BidiOverride)
        );
        assert_eq!(
            parse_unicode_bidi("isolate-override"),
            Some(UnicodeBidiValue::IsolateOverride)
        );
        assert_eq!(parse_unicode_bidi("plaintext"), Some(UnicodeBidiValue::Plaintext));
    }

    #[test]
    fn test_parse_unicode_bidi_case_insensitive() {
        assert_eq!(parse_unicode_bidi("NORMAL"), Some(UnicodeBidiValue::Normal));
        assert_eq!(parse_unicode_bidi("  Embed  "), Some(UnicodeBidiValue::Embed));
    }

    #[test]
    fn test_parse_unicode_bidi_invalid() {
        assert_eq!(parse_unicode_bidi("invalid"), None);
        assert_eq!(parse_unicode_bidi(""), None);
    }

    // ── TabSize 测试 ──

    #[test]
    fn test_parse_tab_size_number() {
        assert_eq!(parse_tab_size("4"), Some(TabSizeValue::Number(4)));
        assert_eq!(parse_tab_size("8"), Some(TabSizeValue::Number(8)));
        assert_eq!(parse_tab_size("0"), Some(TabSizeValue::Number(0)));
    }

    #[test]
    fn test_parse_tab_size_length() {
        assert_eq!(
            parse_tab_size("20px"),
            Some(TabSizeValue::Length(LengthValue::Px(20.0)))
        );
        assert_eq!(parse_tab_size("1em"), Some(TabSizeValue::Length(LengthValue::Em(1.0))));
    }

    #[test]
    fn test_parse_tab_size_case_insensitive() {
        assert_eq!(parse_tab_size("  4  "), Some(TabSizeValue::Number(4)));
    }

    #[test]
    fn test_parse_tab_size_invalid() {
        assert_eq!(parse_tab_size("-1"), None);
        assert_eq!(parse_tab_size("abc"), None);
        assert_eq!(parse_tab_size(""), None);
    }

    // ═══════════════════════════════════════════════════════════════════
    // 边界条件/错误路径边缘测试
    // ═══════════════════════════════════════════════════════════════════

    /// 测试 eval_calc 除以零返回 None
    #[test]
    fn test_eval_calc_divide_by_zero() {
        let expr = parse_calc("calc(10px / 0)").unwrap();
        let result = eval_calc(&expr, None);
        // 除以 0 应返回 None（除法分支的边界保护）
        assert_eq!(result, None);
    }

    /// 测试 parse_length 边界条件：负值、零值、无单位非零、科学计数法
    #[test]
    fn test_parse_length_boundary_conditions() {
        // 负值应正常解析
        assert_eq!(parse_length("-10px"), Some(LengthValue::Px(-10.0)));
        assert_eq!(parse_length("-5em"), Some(LengthValue::Em(-5.0)));
        // 零值（无单位）应解析为 Px(0.0)
        assert_eq!(parse_length("0"), Some(LengthValue::Px(0.0)));
        // 非零无单位值应返回 None
        assert_eq!(parse_length("5"), None);
        // 未知单位应返回 None
        assert_eq!(parse_length("10abc"), None);
        // 百分比零值
        assert_eq!(parse_length("0%"), Some(LengthValue::Percentage(0.0)));
        // 负百分比
        assert_eq!(parse_length("-50%"), Some(LengthValue::Percentage(-50.0)));
        // 科学计数法解析
        assert_eq!(parse_length("1e2px"), Some(LengthValue::Px(100.0)));
    }

    /// 测试 parse_color 边界条件：无效十六进制长度、超出范围的 rgb 分量、空 hwb
    #[test]
    fn test_parse_color_edge_cases() {
        // 无效十六进制长度（2、5、7 位）应返回 None
        assert_eq!(parse_color("#12"), None);
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(parse_color("#1234567"), None);
        // 仅 # 号应返回 None
        assert_eq!(parse_color("#"), None);
        // rgb 超出范围的分量（>255）应被 clamp
        let result = parse_color("rgb(300, -10, 128)");
        assert!(result.is_some());
        match result {
            Some(ColorValue::Rgba(r, g, b, _)) => {
                assert_eq!(r, 255); // 300 被钳制到 255
                assert_eq!(g, 0); // -10 被钳制到 0
                assert_eq!(b, 128);
            }
            _ => panic!("expected Rgba"),
        }
        // rgb 只有 2 个分量应返回 None
        assert_eq!(parse_color("rgb(255, 0)"), None);
        // hwb() 无效格式（缺少参数）应返回 None
        assert_eq!(parse_color("hwb(120 50%)"), None);
    }

    /// 测试 parse_opacity 边界条件：0、1、超出范围值、百分比边界
    #[test]
    fn test_parse_opacity_boundary() {
        // 0.0 和 1.0 边界
        assert_eq!(parse_opacity("0"), Some(0.0));
        assert_eq!(parse_opacity("1"), Some(1.0));
        assert_eq!(parse_opacity("0.0"), Some(0.0));
        assert_eq!(parse_opacity("1.0"), Some(1.0));
        // 超出范围值应被 clamp
        assert_eq!(parse_opacity("-0.5"), Some(0.0));
        assert_eq!(parse_opacity("2.0"), Some(1.0));
        // 百分比边界
        assert_eq!(parse_opacity("0%"), Some(0.0));
        assert_eq!(parse_opacity("100%"), Some(1.0));
        assert_eq!(parse_opacity("150%"), Some(1.0));
        assert_eq!(parse_opacity("-10%"), Some(0.0));
        // 非法输入
        assert_eq!(parse_opacity("abc"), None);
        assert_eq!(parse_opacity(""), None);
    }

    /// 测试 parse_var 边界条件：空名称、回退值为空、嵌套 var
    #[test]
    fn test_parse_var_edge_cases() {
        // 基本解析带回退值
        let result = parse_var("var(--color, red)").unwrap();
        assert_eq!(result.name, "--color");
        assert_eq!(result.fallback, Some("red".to_string()));
        // 仅名称，无回退
        let result = parse_var("var(--spacing)").unwrap();
        assert_eq!(result.name, "--spacing");
        assert_eq!(result.fallback, None);
        // 不以 var( 开头应返回 None
        assert_eq!(parse_var("calc(10px)"), None);
        // 空字符串应返回 None
        assert_eq!(parse_var(""), None);
        // 缺少右括号应返回 None
        assert_eq!(parse_var("var(--color"), None);
    }

    // ── break-inside 测试 ──

    #[test]
    fn test_parse_break_inside_valid() {
        assert_eq!(parse_break_inside("auto"), Some(BreakInsideValue::Auto));
        assert_eq!(parse_break_inside("avoid"), Some(BreakInsideValue::Avoid));
        assert_eq!(parse_break_inside("avoid-page"), Some(BreakInsideValue::AvoidPage));
        assert_eq!(parse_break_inside("avoid-column"), Some(BreakInsideValue::AvoidColumn));
    }

    #[test]
    fn test_parse_break_inside_case_insensitive() {
        assert_eq!(parse_break_inside("AVOID"), Some(BreakInsideValue::Avoid));
        assert_eq!(parse_break_inside("  Avoid-Page  "), Some(BreakInsideValue::AvoidPage));
    }

    #[test]
    fn test_parse_break_inside_invalid() {
        assert_eq!(parse_break_inside("column"), None);
        assert_eq!(parse_break_inside("page"), None);
        assert_eq!(parse_break_inside("invalid"), None);
        assert_eq!(parse_break_inside(""), None);
    }

    // ── break-before / break-after 测试 ──

    #[test]
    fn test_parse_break_before_valid() {
        assert_eq!(parse_break_before("auto"), Some(BreakValue::Auto));
        assert_eq!(parse_break_before("avoid"), Some(BreakValue::Avoid));
        assert_eq!(parse_break_before("column"), Some(BreakValue::Column));
        assert_eq!(parse_break_before("page"), Some(BreakValue::Page));
        assert_eq!(parse_break_before("avoid-page"), Some(BreakValue::AvoidPage));
        assert_eq!(parse_break_before("avoid-column"), Some(BreakValue::AvoidColumn));
    }

    #[test]
    fn test_parse_break_after_valid() {
        assert_eq!(parse_break_after("auto"), Some(BreakValue::Auto));
        assert_eq!(parse_break_after("avoid"), Some(BreakValue::Avoid));
        assert_eq!(parse_break_after("column"), Some(BreakValue::Column));
        assert_eq!(parse_break_after("page"), Some(BreakValue::Page));
        assert_eq!(parse_break_after("avoid-page"), Some(BreakValue::AvoidPage));
        assert_eq!(parse_break_after("avoid-column"), Some(BreakValue::AvoidColumn));
    }

    #[test]
    fn test_parse_break_before_after_invalid() {
        assert_eq!(parse_break_before("always"), None);
        assert_eq!(parse_break_before("invalid"), None);
        assert_eq!(parse_break_after("left"), None);
        assert_eq!(parse_break_after(""), None);
    }

    // ── column-rule-width 测试 ──

    #[test]
    fn test_parse_column_rule_width_keywords() {
        assert_eq!(parse_column_rule_width("medium"), Some(ColumnRuleWidthValue::Medium));
        assert_eq!(parse_column_rule_width("thin"), Some(ColumnRuleWidthValue::Thin));
        assert_eq!(parse_column_rule_width("thick"), Some(ColumnRuleWidthValue::Thick));
    }

    #[test]
    fn test_parse_column_rule_width_length() {
        assert_eq!(
            parse_column_rule_width("2px"),
            Some(ColumnRuleWidthValue::Length(LengthValue::Px(2.0)))
        );
        assert_eq!(
            parse_column_rule_width("0.5em"),
            Some(ColumnRuleWidthValue::Length(LengthValue::Em(0.5)))
        );
    }

    #[test]
    fn test_parse_column_rule_width_invalid() {
        assert_eq!(parse_column_rule_width("invalid"), None);
        assert_eq!(parse_column_rule_width(""), None);
    }

    // ── column-rule-style 测试 ──

    #[test]
    fn test_parse_column_rule_style_all_values() {
        assert_eq!(parse_column_rule_style("none"), Some(ColumnRuleStyleValue::None));
        assert_eq!(parse_column_rule_style("hidden"), Some(ColumnRuleStyleValue::Hidden));
        assert_eq!(parse_column_rule_style("dotted"), Some(ColumnRuleStyleValue::Dotted));
        assert_eq!(parse_column_rule_style("dashed"), Some(ColumnRuleStyleValue::Dashed));
        assert_eq!(parse_column_rule_style("solid"), Some(ColumnRuleStyleValue::Solid));
        assert_eq!(parse_column_rule_style("double"), Some(ColumnRuleStyleValue::Double));
        assert_eq!(parse_column_rule_style("groove"), Some(ColumnRuleStyleValue::Groove));
        assert_eq!(parse_column_rule_style("ridge"), Some(ColumnRuleStyleValue::Ridge));
        assert_eq!(parse_column_rule_style("inset"), Some(ColumnRuleStyleValue::Inset));
        assert_eq!(parse_column_rule_style("outset"), Some(ColumnRuleStyleValue::Outset));
    }

    #[test]
    fn test_parse_column_rule_style_case_insensitive() {
        assert_eq!(parse_column_rule_style("SOLID"), Some(ColumnRuleStyleValue::Solid));
        assert_eq!(
            parse_column_rule_style("  Dotted  "),
            Some(ColumnRuleStyleValue::Dotted)
        );
    }

    #[test]
    fn test_parse_column_rule_style_invalid() {
        assert_eq!(parse_column_rule_style("invalid"), None);
        assert_eq!(parse_column_rule_style(""), None);
    }

    // ── Appearance 测试 ──

    #[test]
    fn test_parse_appearance_none() {
        assert_eq!(parse_appearance("none"), Some(AppearanceValue::None));
    }

    #[test]
    fn test_parse_appearance_auto() {
        assert_eq!(parse_appearance("auto"), Some(AppearanceValue::Auto));
    }

    #[test]
    fn test_parse_appearance_widgets() {
        assert_eq!(parse_appearance("button"), Some(AppearanceValue::Button));
        assert_eq!(parse_appearance("checkbox"), Some(AppearanceValue::Checkbox));
        assert_eq!(parse_appearance("listbox"), Some(AppearanceValue::Listbox));
        assert_eq!(parse_appearance("menulist"), Some(AppearanceValue::Menulist));
        assert_eq!(parse_appearance("meter"), Some(AppearanceValue::Meter));
        assert_eq!(parse_appearance("progress-bar"), Some(AppearanceValue::ProgressBar));
        assert_eq!(parse_appearance("push-button"), Some(AppearanceValue::PushButton));
        assert_eq!(parse_appearance("radio"), Some(AppearanceValue::Radio));
        assert_eq!(parse_appearance("searchfield"), Some(AppearanceValue::Searchfield));
        assert_eq!(
            parse_appearance("slider-horizontal"),
            Some(AppearanceValue::SliderHorizontal)
        );
        assert_eq!(parse_appearance("square-button"), Some(AppearanceValue::SquareButton));
        assert_eq!(parse_appearance("textarea"), Some(AppearanceValue::Textarea));
        assert_eq!(parse_appearance("textfield"), Some(AppearanceValue::Textfield));
    }

    #[test]
    fn test_parse_appearance_case_insensitive() {
        assert_eq!(parse_appearance("NONE"), Some(AppearanceValue::None));
        assert_eq!(parse_appearance("  Auto  "), Some(AppearanceValue::Auto));
        assert_eq!(parse_appearance("BUTTON"), Some(AppearanceValue::Button));
    }

    #[test]
    fn test_parse_appearance_invalid() {
        assert_eq!(parse_appearance("invalid"), None);
        assert_eq!(parse_appearance(""), None);
    }

    // ── AccentColor 测试 ──

    #[test]
    fn test_parse_accent_color_auto() {
        assert_eq!(parse_accent_color("auto"), Some(AccentColorValue::Auto));
    }

    #[test]
    fn test_parse_accent_color_named() {
        assert_eq!(
            parse_accent_color("red"),
            Some(AccentColorValue::Color(ColorValue::Rgba(255, 0, 0, 255)))
        );
        assert_eq!(
            parse_accent_color("blue"),
            Some(AccentColorValue::Color(ColorValue::Rgba(0, 0, 255, 255)))
        );
    }

    #[test]
    fn test_parse_accent_color_hex() {
        assert_eq!(
            parse_accent_color("#ff0000"),
            Some(AccentColorValue::Color(ColorValue::Rgba(255, 0, 0, 255)))
        );
        assert_eq!(
            parse_accent_color("#0f0"),
            Some(AccentColorValue::Color(ColorValue::Rgba(0, 255, 0, 255)))
        );
    }

    #[test]
    fn test_parse_accent_color_rgb() {
        let result = parse_accent_color("rgb(100, 200, 50)");
        assert!(result.is_some());
        match result.unwrap() {
            AccentColorValue::Color(ColorValue::Rgba(r, g, b, a)) => {
                assert_eq!(r, 100);
                assert_eq!(g, 200);
                assert_eq!(b, 50);
                assert_eq!(a, 255);
            }
            _ => panic!("expected Color variant"),
        }
    }

    #[test]
    fn test_parse_accent_color_invalid() {
        assert_eq!(parse_accent_color("not-a-color"), None);
        assert_eq!(parse_accent_color(""), None);
    }

    // ── CaretColor 测试 ──

    #[test]
    fn test_parse_caret_color_auto() {
        assert_eq!(parse_caret_color("auto"), Some(CaretColorValue::Auto));
    }

    #[test]
    fn test_parse_caret_color_named() {
        assert_eq!(
            parse_caret_color("green"),
            Some(CaretColorValue::Color(ColorValue::Rgba(0, 128, 0, 255)))
        );
    }

    #[test]
    fn test_parse_caret_color_hex() {
        assert_eq!(
            parse_caret_color("#abcdef"),
            Some(CaretColorValue::Color(ColorValue::Rgba(0xAB, 0xCD, 0xEF, 255)))
        );
    }

    #[test]
    fn test_parse_caret_color_transparent() {
        assert_eq!(
            parse_caret_color("transparent"),
            Some(CaretColorValue::Color(ColorValue::Transparent))
        );
    }

    #[test]
    fn test_parse_caret_color_invalid() {
        assert_eq!(parse_caret_color("not-a-color"), None);
        assert_eq!(parse_caret_color(""), None);
    }

    // ── MixBlendMode 测试 ──

    #[test]
    fn test_parse_mix_blend_mode_normal() {
        assert_eq!(parse_mix_blend_mode("normal"), Some(MixBlendModeValue::Normal));
    }

    #[test]
    fn test_parse_mix_blend_mode_all_values() {
        assert_eq!(parse_mix_blend_mode("multiply"), Some(MixBlendModeValue::Multiply));
        assert_eq!(parse_mix_blend_mode("screen"), Some(MixBlendModeValue::Screen));
        assert_eq!(parse_mix_blend_mode("overlay"), Some(MixBlendModeValue::Overlay));
        assert_eq!(parse_mix_blend_mode("darken"), Some(MixBlendModeValue::Darken));
        assert_eq!(parse_mix_blend_mode("lighten"), Some(MixBlendModeValue::Lighten));
        assert_eq!(parse_mix_blend_mode("color-dodge"), Some(MixBlendModeValue::ColorDodge));
        assert_eq!(parse_mix_blend_mode("color-burn"), Some(MixBlendModeValue::ColorBurn));
        assert_eq!(parse_mix_blend_mode("hard-light"), Some(MixBlendModeValue::HardLight));
        assert_eq!(parse_mix_blend_mode("soft-light"), Some(MixBlendModeValue::SoftLight));
        assert_eq!(parse_mix_blend_mode("difference"), Some(MixBlendModeValue::Difference));
        assert_eq!(parse_mix_blend_mode("exclusion"), Some(MixBlendModeValue::Exclusion));
        assert_eq!(parse_mix_blend_mode("hue"), Some(MixBlendModeValue::Hue));
        assert_eq!(parse_mix_blend_mode("saturation"), Some(MixBlendModeValue::Saturation));
        assert_eq!(parse_mix_blend_mode("color"), Some(MixBlendModeValue::Color));
        assert_eq!(parse_mix_blend_mode("luminosity"), Some(MixBlendModeValue::Luminosity));
    }

    #[test]
    fn test_parse_mix_blend_mode_case_insensitive() {
        assert_eq!(parse_mix_blend_mode("NORMAL"), Some(MixBlendModeValue::Normal));
        assert_eq!(parse_mix_blend_mode("  Multiply  "), Some(MixBlendModeValue::Multiply));
        assert_eq!(parse_mix_blend_mode("COLOR-DODGE"), Some(MixBlendModeValue::ColorDodge));
    }

    #[test]
    fn test_parse_mix_blend_mode_invalid() {
        assert_eq!(parse_mix_blend_mode("invalid"), None);
        assert_eq!(parse_mix_blend_mode(""), None);
        assert_eq!(parse_mix_blend_mode("inherit"), None);
    }

    // ── ScrollbarWidth 测试 ──

    #[test]
    fn test_parse_scrollbar_width_auto() {
        assert_eq!(parse_scrollbar_width("auto"), Some(ScrollbarWidthValue::Auto));
    }

    #[test]
    fn test_parse_scrollbar_width_thin() {
        assert_eq!(parse_scrollbar_width("thin"), Some(ScrollbarWidthValue::Thin));
    }

    #[test]
    fn test_parse_scrollbar_width_none() {
        assert_eq!(parse_scrollbar_width("none"), Some(ScrollbarWidthValue::None));
    }

    #[test]
    fn test_parse_scrollbar_width_case_insensitive() {
        assert_eq!(parse_scrollbar_width("AUTO"), Some(ScrollbarWidthValue::Auto));
        assert_eq!(parse_scrollbar_width("  Thin  "), Some(ScrollbarWidthValue::Thin));
        assert_eq!(parse_scrollbar_width("NONE"), Some(ScrollbarWidthValue::None));
    }

    #[test]
    fn test_parse_scrollbar_width_invalid() {
        assert_eq!(parse_scrollbar_width("thick"), None);
        assert_eq!(parse_scrollbar_width(""), None);
    }

    // ── ScrollbarGutter 测试 ──

    #[test]
    fn test_parse_scrollbar_gutter_auto() {
        assert_eq!(parse_scrollbar_gutter("auto"), Some(ScrollbarGutterValue::Auto));
    }

    #[test]
    fn test_parse_scrollbar_gutter_stable() {
        assert_eq!(parse_scrollbar_gutter("stable"), Some(ScrollbarGutterValue::Stable));
    }

    #[test]
    fn test_parse_scrollbar_gutter_stable_both_edges() {
        assert_eq!(
            parse_scrollbar_gutter("stable both-edges"),
            Some(ScrollbarGutterValue::StableBothEdges)
        );
    }

    #[test]
    fn test_parse_scrollbar_gutter_case_insensitive() {
        assert_eq!(parse_scrollbar_gutter("AUTO"), Some(ScrollbarGutterValue::Auto));
        assert_eq!(parse_scrollbar_gutter("  Stable  "), Some(ScrollbarGutterValue::Stable));
        assert_eq!(
            parse_scrollbar_gutter("STABLE BOTH-EDGES"),
            Some(ScrollbarGutterValue::StableBothEdges)
        );
    }

    #[test]
    fn test_parse_scrollbar_gutter_invalid() {
        assert_eq!(parse_scrollbar_gutter("both"), None);
        assert_eq!(parse_scrollbar_gutter("both-edges"), None);
        assert_eq!(parse_scrollbar_gutter(""), None);
        assert_eq!(parse_scrollbar_gutter("invalid"), None);
    }

    // ── text-wrap 解析测试 ──

    #[test]
    fn test_parse_text_wrap_wrap() {
        assert_eq!(parse_text_wrap("wrap"), Some(TextWrapValue::Wrap));
    }

    #[test]
    fn test_parse_text_wrap_nowrap() {
        assert_eq!(parse_text_wrap("nowrap"), Some(TextWrapValue::Nowrap));
    }

    #[test]
    fn test_parse_text_wrap_balance() {
        assert_eq!(parse_text_wrap("balance"), Some(TextWrapValue::Balance));
    }

    #[test]
    fn test_parse_text_wrap_pretty() {
        assert_eq!(parse_text_wrap("pretty"), Some(TextWrapValue::Pretty));
    }

    #[test]
    fn test_parse_text_wrap_stable() {
        assert_eq!(parse_text_wrap("stable"), Some(TextWrapValue::Stable));
    }

    #[test]
    fn test_parse_text_wrap_case_insensitive() {
        assert_eq!(parse_text_wrap("Wrap"), Some(TextWrapValue::Wrap));
        assert_eq!(parse_text_wrap("NOWRAP"), Some(TextWrapValue::Nowrap));
        assert_eq!(parse_text_wrap("Balance"), Some(TextWrapValue::Balance));
    }

    #[test]
    fn test_parse_text_wrap_invalid() {
        assert_eq!(parse_text_wrap("invalid"), None);
        assert_eq!(parse_text_wrap(""), None);
        assert_eq!(parse_text_wrap("auto"), None);
    }

    // ── hyphens 解析测试 ──

    #[test]
    fn test_parse_hyphens_none() {
        assert_eq!(parse_hyphens("none"), Some(HyphensValue::None));
    }

    #[test]
    fn test_parse_hyphens_manual() {
        assert_eq!(parse_hyphens("manual"), Some(HyphensValue::Manual));
    }

    #[test]
    fn test_parse_hyphens_auto() {
        assert_eq!(parse_hyphens("auto"), Some(HyphensValue::Auto));
    }

    #[test]
    fn test_parse_hyphens_case_insensitive() {
        assert_eq!(parse_hyphens("None"), Some(HyphensValue::None));
        assert_eq!(parse_hyphens("MANUAL"), Some(HyphensValue::Manual));
        assert_eq!(parse_hyphens("Auto"), Some(HyphensValue::Auto));
    }

    #[test]
    fn test_parse_hyphens_invalid() {
        assert_eq!(parse_hyphens("invalid"), None);
        assert_eq!(parse_hyphens(""), None);
        assert_eq!(parse_hyphens("all"), None);
    }

    // ── line-clamp 解析测试 ──

    #[test]
    fn test_parse_line_clamp_none() {
        assert_eq!(parse_line_clamp("none"), Some(LineClampValue::None));
    }

    #[test]
    fn test_parse_line_clamp_count() {
        assert_eq!(parse_line_clamp("3"), Some(LineClampValue::Count(3)));
        assert_eq!(parse_line_clamp("1"), Some(LineClampValue::Count(1)));
        assert_eq!(parse_line_clamp("10"), Some(LineClampValue::Count(10)));
    }

    #[test]
    fn test_parse_line_clamp_case_insensitive() {
        assert_eq!(parse_line_clamp("None"), Some(LineClampValue::None));
        assert_eq!(parse_line_clamp("NONE"), Some(LineClampValue::None));
    }

    #[test]
    fn test_parse_line_clamp_invalid() {
        assert_eq!(parse_line_clamp("0"), None);
        assert_eq!(parse_line_clamp("-1"), None);
        assert_eq!(parse_line_clamp("1.5"), None);
        assert_eq!(parse_line_clamp("auto"), None);
        assert_eq!(parse_line_clamp(""), None);
    }

    // ── background-image 解析测试 ──

    #[test]
    fn test_parse_background_image_none() {
        assert_eq!(parse_background_image("none"), Some(BackgroundImageValue::None));
    }

    #[test]
    fn test_parse_background_image_url() {
        assert_eq!(
            parse_background_image("url(image.png)"),
            Some(BackgroundImageValue::Url("image.png".to_string()))
        );
    }

    #[test]
    fn test_parse_background_image_url_quoted() {
        assert_eq!(
            parse_background_image("url(\"image.png\")"),
            Some(BackgroundImageValue::Url("image.png".to_string()))
        );
        assert_eq!(
            parse_background_image("url('image.png')"),
            Some(BackgroundImageValue::Url("image.png".to_string()))
        );
    }

    #[test]
    fn test_parse_background_image_url_with_path() {
        assert_eq!(
            parse_background_image("url(/path/to/image.png)"),
            Some(BackgroundImageValue::Url("/path/to/image.png".to_string()))
        );
    }

    #[test]
    fn test_parse_background_image_case_insensitive() {
        assert_eq!(parse_background_image("NONE"), Some(BackgroundImageValue::None));
        assert_eq!(parse_background_image("None"), Some(BackgroundImageValue::None));
    }

    #[test]
    fn test_parse_background_image_invalid() {
        assert_eq!(parse_background_image(""), None);
        assert_eq!(parse_background_image("invalid"), None);
        assert_eq!(parse_background_image("url()"), None);
    }

    // ── background-position 解析测试 ──

    #[test]
    fn test_parse_background_position_keywords() {
        assert_eq!(
            parse_background_position("center"),
            Some(BackgroundPositionValue::Center)
        );
        assert_eq!(parse_background_position("left"), Some(BackgroundPositionValue::Left));
        assert_eq!(parse_background_position("right"), Some(BackgroundPositionValue::Right));
        assert_eq!(parse_background_position("top"), Some(BackgroundPositionValue::Top));
        assert_eq!(
            parse_background_position("bottom"),
            Some(BackgroundPositionValue::Bottom)
        );
    }

    #[test]
    fn test_parse_background_position_percent() {
        assert_eq!(
            parse_background_position("50%"),
            Some(BackgroundPositionValue::Percent(50.0))
        );
        assert_eq!(
            parse_background_position("0%"),
            Some(BackgroundPositionValue::Percent(0.0))
        );
        assert_eq!(
            parse_background_position("100%"),
            Some(BackgroundPositionValue::Percent(100.0))
        );
    }

    #[test]
    fn test_parse_background_position_length() {
        assert_eq!(
            parse_background_position("10px"),
            Some(BackgroundPositionValue::Length(10.0))
        );
        assert_eq!(
            parse_background_position("0px"),
            Some(BackgroundPositionValue::Length(0.0))
        );
    }

    #[test]
    fn test_parse_background_position_two_values() {
        let result = parse_background_position("left top");
        assert!(result.is_some());
        if let Some(BackgroundPositionValue::TwoValue(h, v)) = result {
            assert_eq!(*h, BackgroundPositionValue::Left);
            assert_eq!(*v, BackgroundPositionValue::Top);
        } else {
            panic!("Expected TwoValue");
        }
    }

    #[test]
    fn test_parse_background_position_two_values_mixed() {
        let result = parse_background_position("center 50%");
        assert!(result.is_some());
        if let Some(BackgroundPositionValue::TwoValue(h, v)) = result {
            assert_eq!(*h, BackgroundPositionValue::Center);
            assert_eq!(*v, BackgroundPositionValue::Percent(50.0));
        } else {
            panic!("Expected TwoValue");
        }
    }

    #[test]
    fn test_parse_background_position_case_insensitive() {
        assert_eq!(
            parse_background_position("Center"),
            Some(BackgroundPositionValue::Center)
        );
        assert_eq!(parse_background_position("LEFT"), Some(BackgroundPositionValue::Left));
    }

    #[test]
    fn test_parse_background_position_invalid() {
        assert_eq!(parse_background_position(""), None);
        assert_eq!(parse_background_position("invalid"), None);
    }

    // ── background-repeat 解析测试 ──

    #[test]
    fn test_parse_background_repeat_values() {
        assert_eq!(parse_background_repeat("repeat"), Some(BackgroundRepeatValue::Repeat));
        assert_eq!(
            parse_background_repeat("repeat-x"),
            Some(BackgroundRepeatValue::RepeatX)
        );
        assert_eq!(
            parse_background_repeat("repeat-y"),
            Some(BackgroundRepeatValue::RepeatY)
        );
        assert_eq!(
            parse_background_repeat("no-repeat"),
            Some(BackgroundRepeatValue::NoRepeat)
        );
        assert_eq!(parse_background_repeat("space"), Some(BackgroundRepeatValue::Space));
        assert_eq!(parse_background_repeat("round"), Some(BackgroundRepeatValue::Round));
    }

    #[test]
    fn test_parse_background_repeat_case_insensitive() {
        assert_eq!(parse_background_repeat("REPEAT"), Some(BackgroundRepeatValue::Repeat));
        assert_eq!(
            parse_background_repeat("No-Repeat"),
            Some(BackgroundRepeatValue::NoRepeat)
        );
        assert_eq!(
            parse_background_repeat("REPEAT-X"),
            Some(BackgroundRepeatValue::RepeatX)
        );
    }

    #[test]
    fn test_parse_background_repeat_invalid() {
        assert_eq!(parse_background_repeat(""), None);
        assert_eq!(parse_background_repeat("invalid"), None);
        assert_eq!(parse_background_repeat("repeat z"), None);
    }

    // ── background-size 解析测试 ──

    #[test]
    fn test_parse_background_size_keywords() {
        assert_eq!(parse_background_size("auto"), Some(BackgroundSizeValue::Auto));
        assert_eq!(parse_background_size("cover"), Some(BackgroundSizeValue::Cover));
        assert_eq!(parse_background_size("contain"), Some(BackgroundSizeValue::Contain));
    }

    #[test]
    fn test_parse_background_size_length() {
        assert_eq!(parse_background_size("100px"), Some(BackgroundSizeValue::Length(100.0)));
        assert_eq!(parse_background_size("1.5em"), Some(BackgroundSizeValue::Length(1.5)));
        assert_eq!(parse_background_size("2rem"), Some(BackgroundSizeValue::Length(2.0)));
    }

    #[test]
    fn test_parse_background_size_percent() {
        assert_eq!(parse_background_size("50%"), Some(BackgroundSizeValue::Percent(50.0)));
        assert_eq!(parse_background_size("100%"), Some(BackgroundSizeValue::Percent(100.0)));
    }

    #[test]
    fn test_parse_background_size_case_insensitive() {
        assert_eq!(parse_background_size("AUTO"), Some(BackgroundSizeValue::Auto));
        assert_eq!(parse_background_size("Cover"), Some(BackgroundSizeValue::Cover));
        assert_eq!(parse_background_size("CONTAIN"), Some(BackgroundSizeValue::Contain));
    }

    #[test]
    fn test_parse_background_size_invalid() {
        assert_eq!(parse_background_size(""), None);
        assert_eq!(parse_background_size("invalid"), None);
    }

    // ── background-attachment 解析测试 ──

    #[test]
    fn test_parse_background_attachment_values() {
        assert_eq!(
            parse_background_attachment("scroll"),
            Some(BackgroundAttachmentValue::Scroll)
        );
        assert_eq!(
            parse_background_attachment("fixed"),
            Some(BackgroundAttachmentValue::Fixed)
        );
        assert_eq!(
            parse_background_attachment("local"),
            Some(BackgroundAttachmentValue::Local)
        );
    }

    #[test]
    fn test_parse_background_attachment_case_insensitive() {
        assert_eq!(
            parse_background_attachment("SCROLL"),
            Some(BackgroundAttachmentValue::Scroll)
        );
        assert_eq!(
            parse_background_attachment("Fixed"),
            Some(BackgroundAttachmentValue::Fixed)
        );
        assert_eq!(
            parse_background_attachment("LOCAL"),
            Some(BackgroundAttachmentValue::Local)
        );
    }

    #[test]
    fn test_parse_background_attachment_invalid() {
        assert_eq!(parse_background_attachment(""), None);
        assert_eq!(parse_background_attachment("invalid"), None);
        assert_eq!(parse_background_attachment("scroll fixed"), None);
    }
}
