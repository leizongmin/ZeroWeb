//! CSS 词法分析器。
//!
//! 基于 CSS Syntax Module Level 3 规范实现。
//! 将 CSS 字符流转换为 [`Token`] 流。

use std::fmt;

// ── Spanned ──────────────────────────────────────────────────────────

// ── Token ────────────────────────────────────────────────────────────

/// CSS token 类型。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 标识符（如 `div`、`color`、`auto`）。
    Ident(String),
    /// @ 关键字（如 `@media`）。
    AtKeyword(String),
    /// 哈希值（如 `#fff`、`#main`）。
    Hash(String),
    /// 字符串字面量（如 `"hello"`、`'world'`）。
    String(String),
    /// URL（如 `url(image.png)`）。
    Url(String),
    /// 数字。
    Number(f64),
    /// 百分比（如 `50%`）。
    Percentage(f64),
    /// 带单位数字（如 `10px`、`1.5em`）。
    Dimension(f64, String),
    /// 函数调用开始（如 `rgb(`）。
    Function(String),
    /// Unicode 范围（如 `U+0-7F`）。
    UnicodeRange(String, String),
    /// 包含匹配（`~=`）。
    IncludeMatch,
    /// 破折号匹配（`|=`）。
    DashMatch,
    /// 前缀匹配（`^=`）。
    PrefixMatch,
    /// 后缀匹配（`$=`）。
    SuffixMatch,
    /// 子串匹配（`*=`）。
    SubstringMatch,
    /// 列选择器（`||`）。
    Column,
    /// 空白。
    Whitespace,
    /// 冒号（`:`）。
    Colon,
    /// 分号（`;`）。
    Semicolon,
    /// 逗号（`,`）。
    Comma,
    /// 左方括号（`[`）。
    LBracket,
    /// 右方括号（`]`）。
    RBracket,
    /// 左圆括号（`(`）。
    LParen,
    /// 右圆括号（`)`）。
    RParen,
    /// 左花括号（`{`）。
    LBrace,
    /// 右花括号（`}`）。
    RBrace,
    /// 注释内容。
    Comment(String),
    /// 分隔符（如 `.`、`!`、`>`、`+`、`~`、`*` 等单字符）。
    Delim(char),
    /// EOF。
    Eof,
    /// 解析错误。
    Error(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "{}", s),
            Token::AtKeyword(s) => write!(f, "@{}", s),
            Token::Hash(s) => write!(f, "#{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Url(s) => write!(f, "url({})", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::Percentage(n) => write!(f, "{}%", n),
            Token::Dimension(n, u) => write!(f, "{}{}", n, u),
            Token::Function(s) => write!(f, "{}(", s),
            Token::UnicodeRange(start, end) => write!(f, "U+{}-{}", start, end),
            Token::IncludeMatch => write!(f, "~="),
            Token::DashMatch => write!(f, "|="),
            Token::PrefixMatch => write!(f, "^="),
            Token::SuffixMatch => write!(f, "$="),
            Token::SubstringMatch => write!(f, "*="),
            Token::Column => write!(f, "||"),
            Token::Whitespace => write!(f, " "),
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Comment(s) => write!(f, "/* {} */", s),
            Token::Delim(c) => write!(f, "{}", c),
            Token::Eof => write!(f, "<EOF>"),
            Token::Error(s) => write!(f, "<ERROR: {}>", s),
        }
    }
}

// ── Spanned ──────────────────────────────────────────────────────────

/// 带源码位置信息的 token。
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    /// token 本身。
    pub token: Token,
    /// token 起始字节偏移量。
    pub offset: usize,
}

/// 将字节偏移量转换为 (行, 列)，均为 1 起始。
///
/// 换行符 `\n`、`\r\n`、`\r` 均视为换行。
pub fn line_column_from_offset(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let offset = offset.min(bytes.len());
    let mut line = 1;
    let mut col = 1;
    let mut i = 0;
    while i < offset {
        match bytes[i] {
            b'\r' => {
                line += 1;
                col = 1;
                i += 1;
                if i < bytes.len() && bytes[i] == b'\n' {
                    i += 1;
                }
            }
            b'\n' => {
                line += 1;
                col = 1;
                i += 1;
            }
            _ => {
                col += 1;
                i += 1;
            }
        }
    }
    (line, col)
}

// ── Tokenizer ────────────────────────────────────────────────────────

/// CSS 词法分析器。
///
/// 将 CSS 字符流逐个转换为带位置信息的 [`Spanned`]。
///
/// # 示例
///
/// ```
/// use zero_css_parser::Tokenizer;
///
/// let tokenizer = Tokenizer::new("div { color: red; }");
/// let tokens: Vec<_> = tokenizer.collect_tokens();
/// ```
pub struct Tokenizer {
    /// 输入字符。
    chars: Vec<char>,
    /// 当前位置。
    pos: usize,
    /// 原始输入（用于计算字节偏移量）。
    source: String,
}

impl Tokenizer {
    /// 创建新的 tokenizer。
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            source: input.to_string(),
        }
    }

    /// 获取当前位置。
    pub fn position(&self) -> usize {
        self.pos
    }

    /// 获取当前字符位置对应的字节偏移量。
    fn byte_offset(&self) -> usize {
        self.source
            .char_indices()
            .nth(self.pos)
            .map(|(i, _)| i)
            .unwrap_or(self.source.len())
    }

    /// 收集所有 token（不带位置信息）。
    ///
    /// 便捷方法，等价于 `.map(|s| s.token).collect()`。
    pub fn collect_tokens(self) -> Vec<Token> {
        self.map(|s| s.token).collect()
    }

    /// 是否已到达末尾。
    pub fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    // ── 字符访问 ─────────────────────────────────────────────────

    /// 查看当前字符（不消耗）。
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// 查看后续第 n 个字符。
    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    /// 消耗并返回当前字符。
    fn consume(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    /// 消耗当前字符（如果匹配）。
    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.consume();
            true
        } else {
            false
        }
    }

    /// 消耗换行符。
    #[allow(dead_code)]
    fn consume_newline(&mut self) -> bool {
        match self.peek() {
            Some('\n') => {
                self.consume();
                true
            }
            Some('\r') => {
                self.consume();
                self.consume_if('\n');
                true
            }
            Some('\t') => {
                self.consume();
                true
            }
            _ => false,
        }
    }

    /// 检查字符是否为空白。
    fn is_whitespace(c: char) -> bool {
        matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0C')
    }

    /// 检查字符是否为标识符起始字符。
    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == '-' || !c.is_ascii()
    }

    /// 检查字符是否为标识符字符。
    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || !c.is_ascii()
    }

    /// 检查字符是否为数字。
    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    /// 检查字符是否为十六进制数字。
    fn is_hex_digit(c: char) -> bool {
        c.is_ascii_hexdigit()
    }

    // ── 核心 token 化 ──────────────────────────────────────────

    /// 消耗空白。
    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if Self::is_whitespace(c) {
                self.consume();
            } else {
                break;
            }
        }
    }

    /// 消耗注释。
    fn consume_comment(&mut self) -> Option<Token> {
        // 已经确认以 /* 开头
        self.consume(); // *
        let mut content = String::new();
        loop {
            match self.peek() {
                Some('*') => {
                    self.consume();
                    if self.consume_if('/') {
                        return Some(Token::Comment(content));
                    }
                    content.push('*');
                }
                Some(c) => {
                    self.consume();
                    content.push(c);
                }
                None => {
                    return Some(Token::Error("Unterminated comment".to_string()));
                }
            }
        }
    }

    /// 消耗标识符。
    /// 读取 hash token 的 name（CSS hash 允许首字符为数字，如颜色 #00FFFF）。
    ///
    /// 与 `consume_ident`（标识符不可数字开头）不同，hash name 读取所有 ident_char
    /// 与转义序列，首字符可为数字。
    fn consume_hash_name(&mut self) -> String {
        let mut name = String::new();
        loop {
            match self.peek() {
                Some('\\') => {
                    if let Some(escaped) = self.consume_escape() {
                        name.push(escaped);
                    } else {
                        break;
                    }
                }
                Some(c) if Self::is_ident_char(c) => {
                    name.push(self.consume().unwrap());
                }
                _ => break,
            }
        }
        name
    }

    fn consume_ident(&mut self) -> String {
        let mut ident = String::new();

        // 处理开头是转义的情况
        if self.peek() == Some('\\') {
            if let Some(escaped) = self.consume_escape() {
                ident.push(escaped);
            }
            // 继续收集
        } else if let Some(c) = self.peek() {
            if c == '-' {
                ident.push(self.consume().unwrap());
                // 检查下一个字符是否有效
                if let Some(next) = self.peek() {
                    if next == '-' || Self::is_ident_start(next) || Self::is_digit(next) {
                        // 有效，继续
                    } else if next == '\\' {
                        if let Some(escaped) = self.consume_escape() {
                            ident.push(escaped);
                        }
                    } else {
                        // 仅 "-" 本身
                        return ident;
                    }
                }
            } else if Self::is_ident_start(c) {
                ident.push(self.consume().unwrap());
            } else {
                return ident;
            }
        }

        while let Some(c) = self.peek() {
            if Self::is_ident_char(c) {
                ident.push(self.consume().unwrap());
            } else if c == '\\' {
                if let Some(escaped) = self.consume_escape() {
                    ident.push(escaped);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        ident
    }

    /// 消耗转义序列。
    fn consume_escape(&mut self) -> Option<char> {
        // 已经确认以 \ 开头
        self.consume(); // \

        match self.peek() {
            Some(c) if Self::is_hex_digit(c) => {
                let mut hex = String::new();
                while hex.len() < 6 {
                    if let Some(hc) = self.peek() {
                        if Self::is_hex_digit(hc) {
                            hex.push(self.consume().unwrap());
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                // 消耗可选的空白
                if let Some(ws) = self.peek()
                    && Self::is_whitespace(ws)
                {
                    self.consume();
                }
                let codepoint = u32::from_str_radix(&hex, 16).unwrap_or(0);
                if codepoint == 0 || codepoint > 0x10FFFF || (0xD800..=0xDFFF).contains(&codepoint) {
                    Some('\u{FFFD}') // 替换字符
                } else {
                    Some(char::from_u32(codepoint).unwrap_or('\u{FFFD}'))
                }
            }
            Some('\n') | Some('\r') | Some('\x0C') => None, // 换行不能转义
            Some(c) => {
                self.consume();
                Some(c)
            }
            None => Some('\u{FFFD}'), // EOF after backslash
        }
    }

    /// 对字符串做 CSS 转义解码（镜像 [`Tokenizer::consume_escape`] 的算法）。
    ///
    /// 用于在不经完整 tokenization 的场景（如 reftest harness 原始扫描 `url()` 内容）
    /// 下，得到与 tokenizer 一致的解码结果——使 harness 的 url key 与 painter（经
    /// tokenizer 解码）对齐（driving：uri-005 `support/\'green\ block.png` →
    /// `support/'green block.png`）。算法稳定（CSS Syntax §4.3.7），并有 parity 测试
    /// 守此函数与 consume_escape 一致。
    pub fn css_unescape(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c != '\\' {
                out.push(c);
                continue;
            }
            // 反斜杠：镜像 consume_escape
            match chars.peek().copied() {
                Some(h) if Self::is_hex_digit(h) => {
                    let mut hex = String::new();
                    while hex.len() < 6 {
                        if let Some(hc) = chars.peek().copied()
                            && Self::is_hex_digit(hc)
                        {
                            hex.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    // 消耗可选的单个空白
                    if matches!(chars.peek().copied(), Some(w) if Self::is_whitespace(w)) {
                        chars.next();
                    }
                    let codepoint = u32::from_str_radix(&hex, 16).unwrap_or(0);
                    if codepoint == 0 || codepoint > 0x10FFFF || (0xD800..=0xDFFF).contains(&codepoint) {
                        out.push('\u{FFFD}');
                    } else {
                        out.push(char::from_u32(codepoint).unwrap_or('\u{FFFD}'));
                    }
                }
                Some('\n') | Some('\r') | Some('\x0C') => {
                    // 换行不能转义：丢弃反斜杠与换行（行连接语义）
                    chars.next();
                }
                Some(other) => {
                    out.push(other);
                    chars.next();
                }
                None => out.push('\u{FFFD}'),
            }
        }
        out
    }

    /// 消耗数字。
    fn consume_number(&mut self) -> f64 {
        let mut num_str = String::new();

        // 可选符号
        if self.peek() == Some('+') || self.peek() == Some('-') {
            num_str.push(self.consume().unwrap());
        }

        // 整数部分
        while let Some(c) = self.peek() {
            if Self::is_digit(c) {
                num_str.push(self.consume().unwrap());
            } else {
                break;
            }
        }

        // 小数部分
        if self.peek() == Some('.')
            && let Some(next) = self.peek_at(1)
            && Self::is_digit(next)
        {
            num_str.push(self.consume().unwrap()); // .
            while let Some(c) = self.peek() {
                if Self::is_digit(c) {
                    num_str.push(self.consume().unwrap());
                } else {
                    break;
                }
            }
        }

        // 科学计数法（e/E）
        if let Some('e') | Some('E') = self.peek()
            && let Some(next) = self.peek_at(1)
            && (Self::is_digit(next) || next == '+' || next == '-')
        {
            num_str.push(self.consume().unwrap()); // e/E
            if self.peek() == Some('+') || self.peek() == Some('-') {
                num_str.push(self.consume().unwrap());
            }
            while let Some(c) = self.peek() {
                if Self::is_digit(c) {
                    num_str.push(self.consume().unwrap());
                } else {
                    break;
                }
            }
        }

        num_str.parse().unwrap_or(0.0)
    }

    /// 消耗字符串字面量。
    fn consume_string(&mut self, ending: char) -> Token {
        let s = self.consume_string_content(ending);
        Token::String(s)
    }

    /// 消耗字符串内容（不包括引号），返回字符串值。
    fn consume_string_content(&mut self, ending: char) -> String {
        let mut s = String::new();
        loop {
            match self.peek() {
                Some(c) if c == ending => {
                    self.consume();
                    return s;
                }
                Some('\n') | Some('\r') | Some('\x0C') => {
                    // 未终止字符串
                    return s;
                }
                Some('\\') => {
                    // 注意：consume_escape 自身会消耗反斜杠（约定「已确认以 \ 开头」），
                    // 故此处不再提前消耗反斜杠，否则会重复消耗一个真实字符
                    // （历史 bug：`"\""` 把转义引号与闭合引号一并吞掉）。
                    match self.peek_at(1) {
                        // \<换行> = 行连接，跳过反斜杠与换行（含 CRLF）
                        Some('\n') => {
                            self.consume();
                            self.consume();
                        }
                        Some('\r') => {
                            self.consume();
                            self.consume();
                            self.consume_if('\n');
                        }
                        // \<EOF> = 字面反斜杠（CSS2 §4.2：非合法转义时保留反斜杠）
                        None => {
                            self.consume();
                            s.push('\\');
                        }
                        // 合法转义（十六进制 / 转义引号与普通字符）交给 consume_escape
                        Some(_) => {
                            if let Some(escaped) = self.consume_escape() {
                                s.push(escaped);
                            }
                        }
                    }
                }
                Some(c) => {
                    self.consume();
                    s.push(c);
                }
                None => {
                    return s;
                }
            }
        }
    }

    /// 消耗 URL 内容。
    fn consume_url(&mut self) -> Token {
        // 跳过前导空白
        self.consume_whitespace();

        // 检查是否为引号包裹的 URL 参数
        match self.peek() {
            Some('"') | Some('\'') => {
                let quote = self.peek().unwrap();
                self.consume(); // 消耗起始引号
                let url = match self.consume_string(quote) {
                    Token::String(s) => s,
                    _ => String::new(),
                };
                // 消耗可能的尾部空白和 )
                self.consume_whitespace();
                if self.peek() == Some(')') {
                    self.consume();
                }
                return Token::Url(url);
            }
            _ => {}
        }

        let mut url = String::new();
        loop {
            match self.peek() {
                Some(')') => {
                    self.consume();
                    return Token::Url(url);
                }
                Some(c) if Self::is_whitespace(c) => {
                    self.consume();
                    self.consume_whitespace();
                    if self.peek() == Some(')') {
                        self.consume();
                        return Token::Url(url);
                    }
                    return Token::Url(url);
                }
                Some('(') => {
                    // 嵌套括号在无引号 URL 中非法
                    return Token::Error("Invalid character in URL".to_string());
                }
                Some('\\') => {
                    // CSS Syntax L3：无引号 url 允许转义（driving：uri-005
                    // `url(support/\'green\ block.png)` → `support/'green block.png`）。
                    // consume_escape 解码转义字符（含十六进制）；\<换行> 在 url 中非法 → 错误终止。
                    match self.consume_escape() {
                        Some(escaped) => url.push(escaped),
                        None => return Token::Error("Invalid escape in URL".to_string()),
                    }
                }
                Some('"') | Some('\'') => {
                    // CSS Syntax §5.4.7 consume_a_url：无引号 url 中遇 `"`/`'` 是 parse error，
                    // 须**消耗一个字符串**（consume a string）并把其值并入 url。字符串未终止时在
                    // 行尾结束（consume_string_content 见换行即返）——故 `url(foo"bar) }` 的
                    // `"bar) }` 整段被字符串吞掉（含 `}`/`)`），#foo 的 `{` 因此得不到匹配 `}`，
                    // 后续 `#three { background-color: red }` 被收入 #foo 未闭合块（driving:
                    // uri-013 #three）。旧实现把 `"` 当普通字符并入 url → url 视为合法 →
                    // `#three { background-color: red }` 成独立规则应用致红。
                    let quote = self.peek().unwrap();
                    self.consume(); // 消耗起始引号
                    let s = self.consume_string_content(quote);
                    url.push_str(&s);
                }
                Some(c) => {
                    self.consume();
                    url.push(c);
                }
                None => {
                    return Token::Url(url);
                }
            }
        }
    }

    /// 消耗类似标识符的 token（可能是 ident、function 或 at-keyword）。
    fn consume_ident_like(&mut self) -> Token {
        let ident = self.consume_ident();

        if ident.starts_with('@') {
            // 这不应该发生 — @ 处理在主循环中
            Token::Ident(ident)
        } else if self.peek() == Some('(') {
            self.consume(); // (
            // 检查是否为 url()
            if ident.eq_ignore_ascii_case("url") {
                self.consume_url()
            } else {
                Token::Function(ident)
            }
        } else {
            Token::Ident(ident)
        }
    }
}

// ── Iterator ─────────────────────────────────────────────────────────

impl Iterator for Tokenizer {
    type Item = Spanned;

    fn next(&mut self) -> Option<Spanned> {
        if self.is_eof() {
            return None;
        }

        let start_offset = self.byte_offset();
        let c = self.peek()?;

        let token = match c {
            // 空白
            ' ' | '\t' | '\n' | '\r' | '\x0C' => {
                self.consume_whitespace();
                Token::Whitespace
            }

            // 注释
            '/' => {
                if self.peek_at(1) == Some('*') {
                    self.consume(); // /
                    return self.consume_comment().map(|t| Spanned {
                        token: t,
                        offset: start_offset,
                    });
                } else {
                    self.consume();
                    Token::Delim('/')
                }
            }

            // 字符串
            '"' | '\'' => {
                let quote = self.consume().unwrap();
                self.consume_string(quote)
            }

            // Hash
            '#' => {
                self.consume();
                if let Some(next) = self.peek() {
                    if Self::is_ident_char(next) || next == '\\' {
                        // CSS hash token（如颜色 #00FFFF / #4169E1）的 name 允许首字符为
                        // 数字，与标识符（不可数字开头）不同。consume_ident 要求 ident_start
                        // （排除数字），用于 # 会把 #00FFFF 误读为 Hash("")+Number(0)+Ident，
                        // 数字部分被当 Number 解析（#4169E1→#41690 科学计数法、#00FFFF→#0FFFF
                        // 去前导零），破坏 hex 颜色。此处用允许数字开头的 hash name 读取。
                        let name = self.consume_hash_name();
                        Token::Hash(name)
                    } else {
                        Token::Error("Unexpected '#'".to_string())
                    }
                } else {
                    Token::Error("Unexpected '#' at EOF".to_string())
                }
            }

            // 左圆括号
            '(' => {
                self.consume();
                Token::LParen
            }

            // 右圆括号
            ')' => {
                self.consume();
                Token::RParen
            }

            // 左花括号
            '{' => {
                self.consume();
                Token::LBrace
            }

            // 右花括号
            '}' => {
                self.consume();
                Token::RBrace
            }

            // 左方括号
            '[' => {
                self.consume();
                Token::LBracket
            }

            // 右方括号
            ']' => {
                self.consume();
                Token::RBracket
            }

            // 冒号
            ':' => {
                self.consume();
                Token::Colon
            }

            // 分号
            ';' => {
                self.consume();
                Token::Semicolon
            }

            // 逗号
            ',' => {
                self.consume();
                Token::Comma
            }

            // CDO（`<!--`，CSS Syntax §4.1.1）：stylesheet 顶层应忽略——legacy HTML 注释
            // 包裹 `<style>` 块的常见模式。复用 Comment ignorable 通道（同 CDC），顶层被
            // `skip_whitespace` 跳过，不触发 selector 解析 + `skip_malformed_qualified_rule`
            // 吞掉后续真实规则（driving: cdo-cdc-stylesheet-wrap）。
            '<' => {
                if self.peek_at(1) == Some('!') && self.peek_at(2) == Some('-') && self.peek_at(3) == Some('-') {
                    self.consume(); // <
                    self.consume(); // !
                    self.consume(); // -
                    self.consume(); // -
                    Token::Comment("<!--".to_string())
                } else {
                    self.consume();
                    Token::Delim('<')
                }
            }

            // @ 关键字
            '@' => {
                self.consume();
                if let Some(next) = self.peek() {
                    if Self::is_ident_start(next) {
                        let ident = self.consume_ident();
                        Token::AtKeyword(ident)
                    } else {
                        Token::Error("Expected identifier after @".to_string())
                    }
                } else {
                    Token::Error("Unexpected @ at EOF".to_string())
                }
            }

            // 数字
            '0'..='9' | '.' => {
                if c == '.' {
                    // 检查是否是数字开头（.后面跟数字）
                    if let Some(next) = self.peek_at(1) {
                        if !Self::is_digit(next) {
                            self.consume();
                            Token::Delim('.')
                        } else {
                            self.consume_number_and_suffix()
                        }
                    } else {
                        self.consume();
                        Token::Delim('.')
                    }
                } else {
                    self.consume_number_and_suffix()
                }
            }

            // + 或 - 后面跟数字
            '+' | '-' => {
                // CDC（`-->`，CSS Syntax §4.1.1）：stylesheet 顶层应忽略——legacy HTML 注释
                // 包裹 `<style>` 块的常见模式（`<style><!-- ... --></style>`）。复用 Comment
                // ignorable 通道（parser `skip_whitespace` 已跳过 Comment），与 chromium 顶层
                // 忽略一致。须在数字/ident 判定前识别，否则 `-->` 经 `--` ident 路径被拆散，
                // 顶层残 token 触发 `skip_malformed_qualified_rule` 吞掉后续真实规则
                //（driving: cdo-cdc-stylesheet-wrap）。
                if c == '-' && self.peek_at(1) == Some('-') && self.peek_at(2) == Some('>') {
                    self.consume(); // -
                    self.consume(); // -
                    self.consume(); // >
                    return Some(Spanned {
                        token: Token::Comment("-->".to_string()),
                        offset: start_offset,
                    });
                }
                let sign = self.consume().unwrap();
                // 检查是否为数字
                let mut is_number = false;

                if let Some(next) = self.peek() {
                    if Self::is_digit(next) {
                        is_number = true;
                    } else if next == '.'
                        && let Some(after_dot) = self.peek_at(1)
                        && Self::is_digit(after_dot)
                    {
                        is_number = true;
                    }
                }

                if is_number {
                    self.pos -= 1; // 回退，让 consume_number 处理符号
                    let number = self.consume_number();

                    if self.consume_if('%') {
                        Token::Percentage(number)
                    } else if let Some(next) = self.peek()
                        && (Self::is_ident_start(next) || next == '\\')
                    {
                        let unit = self.consume_ident();
                        Token::Dimension(number, unit)
                    } else {
                        Token::Number(number)
                    }
                } else if sign == '-'
                    && let Some(next) = self.peek()
                    && (Self::is_ident_start(next) || next == '\\' || next == '-')
                {
                    self.pos -= 1; // 回退
                    self.consume_ident_like()
                } else if sign == '|' && self.peek() == Some('|') {
                    self.consume();
                    Token::Column
                } else if sign == '+' {
                    Token::Delim('+')
                } else if sign == '-' {
                    Token::Ident("-".to_string())
                } else {
                    Token::Ident(sign.to_string())
                }
            }

            // ~ （~= 或 ~）
            '~' => {
                self.consume();
                if self.consume_if('=') {
                    Token::IncludeMatch
                } else {
                    Token::Delim('~')
                }
            }

            // | （|= 或 ||）
            '|' => {
                self.consume();
                if self.consume_if('=') {
                    Token::DashMatch
                } else if self.consume_if('|') {
                    Token::Column
                } else {
                    Token::Ident("|".to_string())
                }
            }

            // ^ (^=)
            '^' => {
                self.consume();
                if self.consume_if('=') {
                    Token::PrefixMatch
                } else {
                    Token::Ident("^".to_string())
                }
            }

            // $ ($=)
            '$' => {
                self.consume();
                if self.consume_if('=') {
                    Token::SuffixMatch
                } else {
                    Token::Ident("$".to_string())
                }
            }

            // * (*=)
            '*' => {
                self.consume();
                if self.consume_if('=') {
                    Token::SubstringMatch
                } else {
                    Token::Delim('*')
                }
            }

            // ! 作为分隔符
            '!' => {
                self.consume();
                Token::Delim('!')
            }

            // > 作为分隔符
            '>' => {
                self.consume();
                Token::Delim('>')
            }

            // = 作为分隔符（用于属性选择器中的精确匹配 [attr=val]）
            '=' => {
                self.consume();
                Token::Delim('=')
            }

            // 反斜杠转义起始：CSS Syntax §4.3 规定 `\` 后跟合法转义（hex 数字或
            // 任意非换行字符，含 EOF）时，`\` 是 ident 的一部分，应走 ident-like 路径
            //（driving：escapes-002 选择器 `p\.class#id`、`p.class#id \{ ... \}` ——
            // 旧实现 `\` 落 Error，`\{` 被拆成 Error+LBrace 误开声明块，致 `background:red`
            // 错误应用）。仅 `\`+换行为非法转义 → `\` 作 Delim。
            '\\' => {
                let valid_escape = match self.peek_at(1) {
                    Some('\n') | Some('\r') | Some('\x0C') => false,
                    _ => true, // 含 EOF：consume_escape 返回 REPLACEMENT CHAR
                };
                if valid_escape {
                    self.consume_ident_like()
                } else {
                    self.consume();
                    Token::Delim('\\')
                }
            }

            // 标识符
            _ if Self::is_ident_start(c) => self.consume_ident_like(),

            // 未知字符
            _ => {
                self.consume();
                Token::Error(format!("Unexpected character: '{}'", c))
            }
        };

        Some(Spanned {
            token,
            offset: start_offset,
        })
    }
}

impl Tokenizer {
    /// 消耗数字并检查后缀（百分比、单位）。
    fn consume_number_and_suffix(&mut self) -> Token {
        let number = self.consume_number();

        // 检查百分比
        if self.consume_if('%') {
            return Token::Percentage(number);
        }

        // 检查单位（dimension）
        if let Some(next) = self.peek() {
            if Self::is_ident_start(next) {
                let unit = self.consume_ident();
                return Token::Dimension(number, unit);
            }
            // 检查 \ 转义开始的单位
            if next == '\\'
                && let Some(_escaped) = self.peek_at(1)
            {
                let unit = self.consume_ident();
                return Token::Dimension(number, unit);
            }
        }

        Token::Number(number)
    }
}
