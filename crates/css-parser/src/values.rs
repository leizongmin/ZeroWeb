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
    }
}

// ── 解析函数 ────────────────────────────────────────────────────────

/// 解析 CSS 颜色值。
///
/// 支持命名颜色、十六进制颜色（#RGB、#RRGGBB、#RGBA、#RRGGBBAA）、
/// `rgb()`/`rgba()` 和 `hsl()`/`hsla()` 函数。
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

/// 解析命名颜色。
///
/// 支持至少 16 种基本 CSS 颜色。
fn parse_named_color(value: &str) -> Option<ColorValue> {
    // 基本 CSS 颜色映射
    let lower = value.to_ascii_lowercase();
    match lower.as_str() {
        "black" => Some(ColorValue::Rgba(0, 0, 0, 255)),
        "white" => Some(ColorValue::Rgba(255, 255, 255, 255)),
        "red" => Some(ColorValue::Rgba(255, 0, 0, 255)),
        "green" => Some(ColorValue::Rgba(0, 128, 0, 255)),
        "blue" => Some(ColorValue::Rgba(0, 0, 255, 255)),
        "yellow" => Some(ColorValue::Rgba(255, 255, 0, 255)),
        "cyan" | "aqua" => Some(ColorValue::Rgba(0, 255, 255, 255)),
        "magenta" | "fuchsia" => Some(ColorValue::Rgba(255, 0, 255, 255)),
        "silver" => Some(ColorValue::Rgba(192, 192, 192, 255)),
        "gray" | "grey" => Some(ColorValue::Rgba(128, 128, 128, 255)),
        "maroon" => Some(ColorValue::Rgba(128, 0, 0, 255)),
        "olive" => Some(ColorValue::Rgba(128, 128, 0, 255)),
        "lime" => Some(ColorValue::Rgba(0, 255, 0, 255)),
        "teal" => Some(ColorValue::Rgba(0, 128, 128, 255)),
        "navy" => Some(ColorValue::Rgba(0, 0, 128, 255)),
        "purple" => Some(ColorValue::Rgba(128, 0, 128, 255)),
        "orange" => Some(ColorValue::Rgba(255, 165, 0, 255)),
        _ => Some(ColorValue::Named(value.to_string())),
    }
}

/// 解析 CSS 长度值。
///
/// 支持格式如 `"10px"`、`"1.5em"`、`"2rem"`、`"100vh"`、`"50%"`、`"auto"` 等。
pub fn parse_length(value: &str) -> Option<LengthValue> {
    let value = value.trim();

    // 处理 auto 关键字
    if value.eq_ignore_ascii_case("auto") {
        return Some(LengthValue::Auto);
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

        // 读取函数名
        let name_start = pos;
        while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
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

    if first_lower.starts_with("from ") || first_lower.contains(" at ") {
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

    // 解析 "at <position>"
    if let Some(at_pos) = lower.find(" at ") {
        let pos_str = &s[at_pos + 4..];
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
}
