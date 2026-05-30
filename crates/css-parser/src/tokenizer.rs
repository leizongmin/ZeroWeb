//! CSS 词法分析器。
//!
//! 基于 CSS Syntax Module Level 3 规范实现。
//! 将 CSS 字符流转换为 [`Token`] 流。

use std::fmt;

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

// ── Tokenizer ────────────────────────────────────────────────────────

/// CSS 词法分析器。
///
/// 将 CSS 字符流逐个转换为 [`Token`]。
///
/// # 示例
///
/// ```
/// use zero_css_parser::Tokenizer;
///
/// let mut tokenizer = Tokenizer::new("div { color: red; }");
/// let tokens: Vec<_> = tokenizer.collect();
/// ```
pub struct Tokenizer {
    /// 输入字符。
    chars: Vec<char>,
    /// 当前位置。
    pos: usize,
}

impl Tokenizer {
    /// 创建新的 tokenizer。
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    /// 获取当前位置。
    pub fn position(&self) -> usize {
        self.pos
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
                    && Self::is_whitespace(ws) {
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
                && Self::is_digit(next) {
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
                && (Self::is_digit(next) || next == '+' || next == '-') {
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
        let mut s = String::new();
        loop {
            match self.peek() {
                Some(c) if c == ending => {
                    self.consume();
                    return Token::String(s);
                }
                Some('\n') | Some('\r') | Some('\x0C') => {
                    // 未终止字符串
                    return Token::String(s);
                }
                Some('\\') => {
                    self.consume();
                    match self.peek() {
                        Some('\n') => {
                            self.consume();
                            // 续行，跳过
                        }
                        Some('\r') => {
                            self.consume();
                            self.consume_if('\n');
                        }
                        Some(_c) => {
                            if let Some(escaped) = self.consume_escape() {
                                s.push(escaped);
                            }
                        }
                        None => {
                            s.push('\\');
                        }
                    }
                }
                Some(c) => {
                    self.consume();
                    s.push(c);
                }
                None => {
                    return Token::String(s);
                }
            }
        }
    }

    /// 消耗 URL 内容。
    fn consume_url(&mut self) -> Token {
        // 跳过前导空白
        self.consume_whitespace();

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
                Some('"') | Some('\'') | Some('(') | Some('\\') => {
                    // 非法字符在无引号 URL 中
                    return Token::Error("Invalid character in URL".to_string());
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
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.is_eof() {
            return None;
        }

        let c = self.peek()?;

        match c {
            // 空白
            ' ' | '\t' | '\n' | '\r' | '\x0C' => {
                self.consume_whitespace();
                Some(Token::Whitespace)
            }

            // 注释
            '/' => {
                if self.peek_at(1) == Some('*') {
                    self.consume(); // /
                    self.consume_comment()
                } else {
                    self.consume();
                    Some(Token::Error("Unexpected '/'".to_string()))
                }
            }

            // 字符串
            '"' | '\'' => {
                let quote = self.consume().unwrap();
                Some(self.consume_string(quote))
            }

            // Hash
            '#' => {
                self.consume();
                if let Some(next) = self.peek() {
                    if Self::is_ident_char(next) || next == '\\' {
                        let ident = self.consume_ident();
                        Some(Token::Hash(ident))
                    } else {
                        Some(Token::Error("Unexpected '#'".to_string()))
                    }
                } else {
                    Some(Token::Error("Unexpected '#' at EOF".to_string()))
                }
            }

            // 左圆括号
            '(' => {
                self.consume();
                Some(Token::LParen)
            }

            // 右圆括号
            ')' => {
                self.consume();
                Some(Token::RParen)
            }

            // 左花括号
            '{' => {
                self.consume();
                Some(Token::LBrace)
            }

            // 右花括号
            '}' => {
                self.consume();
                Some(Token::RBrace)
            }

            // 左方括号
            '[' => {
                self.consume();
                Some(Token::LBracket)
            }

            // 右方括号
            ']' => {
                self.consume();
                Some(Token::RBracket)
            }

            // 冒号
            ':' => {
                self.consume();
                Some(Token::Colon)
            }

            // 分号
            ';' => {
                self.consume();
                Some(Token::Semicolon)
            }

            // 逗号
            ',' => {
                self.consume();
                Some(Token::Comma)
            }

            // @ 关键字
            '@' => {
                self.consume();
                if let Some(next) = self.peek() {
                    if Self::is_ident_start(next) {
                        let ident = self.consume_ident();
                        Some(Token::AtKeyword(ident))
                    } else {
                        Some(Token::Error("Expected identifier after @".to_string()))
                    }
                } else {
                    Some(Token::Error("Unexpected @ at EOF".to_string()))
                }
            }

            // 数字
            '0'..='9' | '.' => {
                if c == '.' {
                    // 检查是否是数字开头（.后面跟数字）
                    if let Some(next) = self.peek_at(1) {
                        if !Self::is_digit(next) {
                            self.consume();
                            return Some(Token::Delim('.'));
                        }
                    } else {
                        self.consume();
                        return Some(Token::Delim('.'));
                    }
                }

                let number = self.consume_number();

                // 检查百分比
                if self.consume_if('%') {
                    return Some(Token::Percentage(number));
                }

                // 检查单位（dimension）
                if let Some(next) = self.peek() {
                    if Self::is_ident_start(next) {
                        let unit = self.consume_ident();
                        return Some(Token::Dimension(number, unit));
                    }
                    // 检查 \ 转义开始的单位
                    if next == '\\'
                        && let Some(_escaped) = self.peek_at(1) {
                            let unit = self.consume_ident();
                            return Some(Token::Dimension(number, unit));
                        }
                }

                Some(Token::Number(number))
            }

            // + 或 - 后面跟数字
            '+' | '-' => {
                let sign = self.consume().unwrap();
                // 检查是否为数字
                let mut is_number = false;

                if let Some(next) = self.peek() {
                    if Self::is_digit(next) {
                        is_number = true;
                    } else if next == '.'
                        && let Some(after_dot) = self.peek_at(1)
                            && Self::is_digit(after_dot) {
                                is_number = true;
                            }
                }

                if is_number {
                    self.pos -= 1; // 回退，让 consume_number 处理符号
                    let number = self.consume_number();

                    if self.consume_if('%') {
                        return Some(Token::Percentage(number));
                    }

                    if let Some(next) = self.peek()
                        && (Self::is_ident_start(next) || next == '\\') {
                            let unit = self.consume_ident();
                            return Some(Token::Dimension(number, unit));
                        }

                    return Some(Token::Number(number));
                }

                // 检查 ident-start（以 - 开头的标识符）
                if sign == '-'
                    && let Some(next) = self.peek()
                        && (Self::is_ident_start(next) || next == '\\' || next == '-') {
                            self.pos -= 1; // 回退
                            return Some(self.consume_ident_like());
                        }

                // 特殊组合器
                if sign == '|' && self.peek() == Some('|') {
                    self.consume();
                    return Some(Token::Column);
                }

                // 当不是数字开头且不是标识符时，+/- 作为分隔符
                if sign == '+' {
                    Some(Token::Delim('+'))
                } else if sign == '-' {
                    // 单独的 - 作为标识符
                    Some(Token::Ident("-".to_string()))
                } else {
                    Some(Token::Ident(sign.to_string()))
                }
            }

            // 点号（如果不是数字开头）
            // ~ （~= 或 ~）
            '~' => {
                self.consume();
                if self.consume_if('=') {
                    Some(Token::IncludeMatch)
                } else {
                    Some(Token::Delim('~'))
                }
            }

            // | （|= 或 ||）
            '|' => {
                self.consume();
                if self.consume_if('=') {
                    Some(Token::DashMatch)
                } else if self.consume_if('|') {
                    Some(Token::Column)
                } else {
                    Some(Token::Ident("|".to_string()))
                }
            }

            // ^ (^=)
            '^' => {
                self.consume();
                if self.consume_if('=') {
                    Some(Token::PrefixMatch)
                } else {
                    Some(Token::Ident("^".to_string()))
                }
            }

            // $ ($=)
            '$' => {
                self.consume();
                if self.consume_if('=') {
                    Some(Token::SuffixMatch)
                } else {
                    Some(Token::Ident("$".to_string()))
                }
            }

            // * (*=)
            '*' => {
                self.consume();
                if self.consume_if('=') {
                    Some(Token::SubstringMatch)
                } else {
                    Some(Token::Delim('*'))
                }
            }

            // ! 作为分隔符
            '!' => {
                self.consume();
                Some(Token::Delim('!'))
            }

            // > 作为分隔符
            '>' => {
                self.consume();
                Some(Token::Delim('>'))
            }

            // = 作为分隔符（用于属性选择器中的精确匹配 [attr=val]）
            '=' => {
                self.consume();
                Some(Token::Delim('='))
            }

            // 标识符
            _ if Self::is_ident_start(c) => {
                Some(self.consume_ident_like())
            }

            // 未知字符
            _ => {
                self.consume();
                Some(Token::Error(format!("Unexpected character: '{}'", c)))
            }
        }
    }
}
