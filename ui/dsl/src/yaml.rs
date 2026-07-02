//! 受限 YAML 子集解析器（spec §8.4.7 YAML DSL / FR-008）。
//!
//! 仅支持 DSL 所需子集：
//! - 块映射（`key: value`）、块序列（`- item`）、嵌套缩进。
//! - 标量：纯标量（null/true/false/int/float/text）、单引号 `'...'`、双引号 `"..."`。
//! - 流集合：`[a, b, c]`、`{k: v, k2: v2}`（可嵌套）。
//! - 行内注释 `#`（引号内不识别）。
//!
//! **不支持**（DSL 文本值均单行，超出子集属非法输入）：锚点/别名 `&`/`*`、多文档 `---`、
//! 块标量 `|`/`>`、多行纯标量、Tab 缩进。
//!
//! 仓内自实现，避免引入 serde_yaml（已弃用）/ serde_yml 依赖（spec §8.4.7 / 依赖自治条款）。
//! 产出中间 AST [`YamlValue`]，由 [`crate::loader`] 转换为强类型 [`zero_ui_core::widget::WidgetSpec`]。

use crate::diagnostics::DslError;

/// 受限 YAML 中间 AST。
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    Seq(Vec<YamlValue>),
    /// 映射；保留插入顺序，重复键在加载器层报错。
    Map(Vec<(String, YamlValue)>),
}

impl YamlValue {
    /// 作为文本（标量），失败返回 None。
    pub fn as_text(&self) -> Option<&str> {
        match self {
            YamlValue::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// 作为映射切片。
    pub fn as_map(&self) -> Option<&[(String, YamlValue)]> {
        match self {
            YamlValue::Map(entries) => Some(entries),
            _ => None,
        }
    }

    /// 作为序列切片。
    pub fn as_seq(&self) -> Option<&[YamlValue]> {
        match self {
            YamlValue::Seq(items) => Some(items),
            _ => None,
        }
    }

    /// 按键查映射值。
    pub fn get(&self, key: &str) -> Option<&YamlValue> {
        match self {
            YamlValue::Map(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
// 预处理：分行 → 去注释 → dash 展开 → Line 列表
// ════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
struct Line {
    indent: usize,
    /// 去注释、去尾部空白后的内容。序列标记行为内容 `"-"。
    content: String,
}

/// 解析源码为 [`YamlValue`]。
pub fn parse(src: &str) -> Result<YamlValue, DslError> {
    let lines = preprocess(src)?;
    if lines.is_empty() {
        return Ok(YamlValue::Null);
    }
    let root_indent = lines[0].indent;
    let mut i = 0usize;
    let v = parse_block(&lines, &mut i, root_indent)?;
    if i != lines.len() {
        return Err(DslError::Parse(format!(
            "YAML 结构无法归约：第 {} 行缩进 {} 与上下文不匹配",
            i + 1,
            lines[i].indent
        )));
    }
    Ok(v)
}

fn preprocess(src: &str) -> Result<Vec<Line>, DslError> {
    let mut raw: Vec<Line> = Vec::new();
    for raw_line in src.lines() {
        let bytes = raw_line.as_bytes();
        // 缩进：仅空格；Tab 报错（YAML 禁止 Tab 缩进）。
        let mut indent = 0usize;
        let mut j = 0usize;
        while j < bytes.len() {
            match bytes[j] {
                b' ' => {
                    indent += 1;
                    j += 1;
                }
                b'\t' => return Err(DslError::Parse("YAML 禁止用 Tab 缩进".into())),
                _ => break,
            }
        }
        let rest = &raw_line[j..];
        let decommented = strip_comment(rest)?;
        let trimmed = decommented.trim_end();
        if trimmed.is_empty() {
            continue; // 空行或纯注释
        }
        raw.push(Line {
            indent,
            content: trimmed.to_string(),
        });
    }
    // dash 展开（递归处理 `- - item` 嵌套序列）。
    let mut out: Vec<Line> = Vec::with_capacity(raw.len());
    for line in &raw {
        out.extend(expand_dash(line));
    }
    Ok(out)
}

/// 把序列项行（`- xxx`）展开为 marker `-` + 注入内容行（indent + 2）。
/// 若注入内容本身又是 `- xxx`（嵌套序列），递归展开。
fn expand_dash(line: &Line) -> Vec<Line> {
    if line.content == "-" {
        return vec![line.clone()];
    }
    if let Some(rest) = line.content.strip_prefix("- ") {
        let after = rest.trim_start();
        let mut out = vec![Line {
            indent: line.indent,
            content: "-".to_string(),
        }];
        if !after.is_empty() {
            let injected = Line {
                indent: line.indent + 2,
                content: after.to_string(),
            };
            out.extend(expand_dash(&injected));
        }
        out
    } else {
        vec![line.clone()]
    }
}

/// 去除行内注释 `#`（仅当 `#` 在行首或前导为空白，且不在引号内）。
fn strip_comment(rest: &str) -> Result<String, DslError> {
    let bytes = rest.as_bytes();
    let mut out = String::with_capacity(rest.len());
    let mut i = 0usize;
    let mut in_single = false;
    let mut in_double = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_single {
            out.push(c as char);
            if c == b'\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }
        if in_double {
            out.push(c as char);
            if c == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        // 非引号上下文
        if c == b'#' && (i == 0 || bytes[i - 1] == b' ') {
            break;
        }
        if c == b'\'' {
            in_single = true;
        } else if c == b'"' {
            in_double = true;
        }
        out.push(c as char);
        i += 1;
    }
    Ok(out)
}

// ════════════════════════════════════════════════════════════════════════
// 块结构解析（递归下降，按缩进）
// ════════════════════════════════════════════════════════════════════════

fn parse_block(lines: &[Line], i: &mut usize, indent: usize) -> Result<YamlValue, DslError> {
    let content = &lines[*i].content;
    if content == "-" {
        parse_seq(lines, i, indent)
    } else if split_key(content).is_some() {
        parse_map(lines, i, indent)
    } else {
        // 单行标量 / 流集合
        let v = parse_scalar_or_flow(content)?;
        *i += 1;
        Ok(v)
    }
}

fn parse_map(lines: &[Line], i: &mut usize, indent: usize) -> Result<YamlValue, DslError> {
    let mut entries: Vec<(String, YamlValue)> = Vec::new();
    while *i < lines.len() {
        let line = &lines[*i];
        if line.indent != indent {
            break;
        }
        if line.content == "-" {
            break; // 同缩进序列打断映射
        }
        let Some((key, rest)) = split_key(&line.content) else {
            break;
        };
        *i += 1;
        let val = if rest.is_empty() {
            // 嵌套块或 null
            if *i < lines.len() && lines[*i].indent > indent {
                parse_block(lines, i, lines[*i].indent)?
            } else {
                YamlValue::Null
            }
        } else {
            parse_scalar_or_flow(&rest)?
        };
        if entries.iter().any(|(k, _)| k == &key) {
            return Err(DslError::Parse(format!("YAML 重复键 '{key}'")));
        }
        entries.push((key, val));
    }
    Ok(YamlValue::Map(entries))
}

fn parse_seq(lines: &[Line], i: &mut usize, indent: usize) -> Result<YamlValue, DslError> {
    let mut items: Vec<YamlValue> = Vec::new();
    while *i < lines.len() {
        let line = &lines[*i];
        if line.indent != indent || line.content != "-" {
            break;
        }
        *i += 1; // 消费 marker
        if *i < lines.len() && lines[*i].indent > indent {
            items.push(parse_block(lines, i, lines[*i].indent)?);
        } else {
            items.push(YamlValue::Null); // `-` 后无内容 → null 项
        }
    }
    Ok(YamlValue::Seq(items))
}

/// 在顶层（非引号、非 `[]{}` 内）查找 `key:` 分隔（`:` 后须为空格/EOL）。
/// 找到返回 `(key, rest_after_colon_trimmed)`。
fn split_key(content: &str) -> Option<(String, String)> {
    let bytes = content.as_bytes();
    let mut i = 0usize;
    let mut depth = 0i32;
    let mut in_single = false;
    let mut in_double = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_single {
            if c == b'\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }
        if in_double {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'\'' => in_single = true,
            b'"' => in_double = true,
            b'[' | b'{' => depth += 1,
            b']' | b'}' => depth -= 1,
            b':' if depth == 0 => {
                let next = bytes.get(i + 1).copied();
                if matches!(next, None | Some(b' ') | Some(b'\t')) {
                    let key = content[..i].trim().to_string();
                    if !key.is_empty() {
                        let rest = content[i + 1..].trim_start().to_string();
                        return Some((key, rest));
                    }
                    return None;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ════════════════════════════════════════════════════════════════════════
// 标量 / 流集合
// ════════════════════════════════════════════════════════════════════════

fn parse_scalar_or_flow(s: &str) -> Result<YamlValue, DslError> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(YamlValue::Null);
    }
    let first = s.as_bytes()[0];
    match first {
        b'[' | b'{' => parse_flow(s),
        b'"' | b'\'' => parse_quoted_scalar(s),
        _ => Ok(plain_scalar(s)),
    }
}

/// 纯标量类型推断：null/bool/int/float，否则 Text。
fn plain_scalar(s: &str) -> YamlValue {
    match s {
        "null" | "Null" | "NULL" | "~" => return YamlValue::Null,
        "true" | "True" | "TRUE" => return YamlValue::Bool(true),
        "false" | "False" | "FALSE" => return YamlValue::Bool(false),
        _ => {}
    }
    if let Ok(i) = s.parse::<i64>() {
        return YamlValue::Int(i);
    }
    if looks_numeric(s)
        && let Ok(f) = s.parse::<f64>()
    {
        return YamlValue::Float(f);
    }
    YamlValue::Text(s.to_string())
}

/// 是否形如数字字面量（首字符为数字/符号/`.`，避免把 `inf`/`nan` 误判）。
fn looks_numeric(s: &str) -> bool {
    let first = match s.as_bytes().first() {
        Some(c) => *c,
        None => return false,
    };
    first.is_ascii_digit() || first == b'-' || first == b'+' || first == b'.'
}

fn parse_quoted_scalar(s: &str) -> Result<YamlValue, DslError> {
    let bytes = s.as_bytes();
    if bytes[0] == b'"' {
        let mut out = String::new();
        let mut i = 1usize;
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'\\' && i + 1 < bytes.len() {
                let e = bytes[i + 1];
                out.push(match e {
                    b'"' => '"',
                    b'\\' => '\\',
                    b'/' => '/',
                    b'n' => '\n',
                    b't' => '\t',
                    b'r' => '\r',
                    other => other as char,
                });
                i += 2;
                continue;
            }
            if c == b'"' {
                let trailing = s[i + 1..].trim();
                if !trailing.is_empty() {
                    return Err(DslError::Parse(format!("双引号后多余内容：{trailing}")));
                }
                return Ok(YamlValue::Text(out));
            }
            out.push(c as char);
            i += 1;
        }
        return Err(DslError::Parse("未闭合的双引号字符串".into()));
    }
    // 单引号：`''` → `'`
    if s.len() < 2 || !s.ends_with('\'') {
        return Err(DslError::Parse("未闭合的单引号字符串".into()));
    }
    let body = &s[1..s.len() - 1];
    let unescaped = body.replace("''", "'");
    Ok(YamlValue::Text(unescaped))
}

/// 流集合解析（`[...]` / `{...}`，可嵌套）。
fn parse_flow(s: &str) -> Result<YamlValue, DslError> {
    let mut p = FlowParser {
        chars: s.chars().collect(),
        pos: 0,
    };
    p.skip_ws();
    let v = p.parse_value()?;
    p.skip_ws();
    if p.pos != p.chars.len() {
        return Err(DslError::Parse(format!("流集合后多余字符：{}", p.tail())));
    }
    Ok(v)
}

struct FlowParser {
    chars: Vec<char>,
    pos: usize,
}

impl FlowParser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    fn tail(&self) -> String {
        self.chars[self.pos..].iter().collect()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn parse_value(&mut self) -> Result<YamlValue, DslError> {
        self.skip_ws();
        match self.peek() {
            Some('[') => self.parse_seq(),
            Some('{') => self.parse_map(),
            Some('"') | Some('\'') => self.parse_quoted(),
            _ => self.parse_plain(),
        }
    }

    fn parse_seq(&mut self) -> Result<YamlValue, DslError> {
        self.bump(); // [
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(YamlValue::Seq(items));
        }
        loop {
            let v = self.parse_value()?;
            items.push(v);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                    self.skip_ws();
                    if self.peek() == Some(']') {
                        break;
                    }
                }
                Some(']') => break,
                Some(c) => return Err(DslError::Parse(format!("流序列期望 ',' 或 ']'，得到 '{c}'"))),
                None => return Err(DslError::Parse("流序列未闭合 ']'".into())),
            }
        }
        self.bump(); // ]
        Ok(YamlValue::Seq(items))
    }

    fn parse_map(&mut self) -> Result<YamlValue, DslError> {
        self.bump(); // {
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(YamlValue::Map(entries));
        }
        loop {
            self.skip_ws();
            let key = self.parse_flow_key()?;
            self.skip_ws();
            let val = if self.peek() == Some(':') {
                self.bump();
                self.parse_value()?
            } else {
                YamlValue::Null
            };
            entries.push((key, val));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.bump();
                    self.skip_ws();
                    if self.peek() == Some('}') {
                        break;
                    }
                }
                Some('}') => break,
                Some(c) => return Err(DslError::Parse(format!("流映射期望 ',' 或 '}}'，得到 '{c}'"))),
                None => return Err(DslError::Parse("流映射未闭合 '}}'".into())),
            }
        }
        self.bump(); // }
        Ok(YamlValue::Map(entries))
    }

    /// 流映射键：引号串或裸标识符（到 `:` 为止）。
    fn parse_flow_key(&mut self) -> Result<String, DslError> {
        match self.peek() {
            Some('"') | Some('\'') => {
                let v = self.parse_quoted()?;
                match v {
                    YamlValue::Text(s) => Ok(s),
                    _ => Err(DslError::Parse("流映射键必须是文本".into())),
                }
            }
            _ => {
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == ':' || c == ',' || c == '}' || c.is_whitespace() {
                        break;
                    }
                    s.push(c);
                    self.bump();
                }
                Ok(s.trim().to_string())
            }
        }
    }

    fn parse_quoted(&mut self) -> Result<YamlValue, DslError> {
        let quote = self.bump().ok_or_else(|| DslError::Parse("空流标量".into()))?;
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if quote == '"' && c == '\\' {
                self.bump();
                let e = self.bump().ok_or_else(|| DslError::Parse("坏转义".into()))?;
                out.push(match e {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    other => other,
                });
                continue;
            }
            if quote == '\'' && c == '\'' {
                self.bump();
                if self.peek() == Some('\'') {
                    out.push('\'');
                    self.bump();
                    continue;
                }
                break;
            }
            if c == quote {
                self.bump();
                return Ok(YamlValue::Text(out));
            }
            out.push(c);
            self.bump();
        }
        Err(DslError::Parse("流标量引号未闭合".into()))
    }

    /// 裸标量（到顶层 `,`/`]`/`}` 为止），再做类型推断。
    fn parse_plain(&mut self) -> Result<YamlValue, DslError> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == ',' || c == ']' || c == '}' {
                break;
            }
            s.push(c);
            self.bump();
        }
        Ok(plain_scalar(s.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, YamlValue)]) -> YamlValue {
        YamlValue::Map(pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect())
    }

    // ── 标量 ──────────────────────────────────────────────────────────
    #[test]
    fn scalar_inference() {
        assert_eq!(plain_scalar("null"), YamlValue::Null);
        assert_eq!(plain_scalar("true"), YamlValue::Bool(true));
        assert_eq!(plain_scalar("False"), YamlValue::Bool(false));
        assert_eq!(plain_scalar("42"), YamlValue::Int(42));
        assert_eq!(plain_scalar("-7"), YamlValue::Int(-7));
        assert_eq!(plain_scalar("1.5"), YamlValue::Float(1.5));
        assert_eq!(plain_scalar("hello"), YamlValue::Text("hello".into()));
        // inf 不当作 float（looks_numeric 守卫）。
        assert_eq!(plain_scalar("inf"), YamlValue::Text("inf".into()));
    }

    #[test]
    fn quoted_scalars() {
        assert_eq!(
            parse(r#""double \\ \n end""#).unwrap(),
            YamlValue::Text("double \\ \n end".into())
        );
        assert_eq!(parse(r#"'it''s here'"#).unwrap(), YamlValue::Text("it's here".into()));
    }

    // ── 块映射 / 序列 / 嵌套 ──────────────────────────────────────────
    #[test]
    fn simple_map() {
        let v = parse("a: 1\nb: hello\nc: true").unwrap();
        assert_eq!(
            v,
            map(&[
                ("a", YamlValue::Int(1)),
                ("b", YamlValue::Text("hello".into())),
                ("c", YamlValue::Bool(true))
            ])
        );
    }

    #[test]
    fn nested_map_and_seq() {
        let src = "props:\n  label: OK\n  count: 3\ntags:\n  - a\n  - b\n";
        let v = parse(src).unwrap();
        assert_eq!(
            v.get("props"),
            Some(&map(&[
                ("label", YamlValue::Text("OK".into())),
                ("count", YamlValue::Int(3))
            ]))
        );
        assert_eq!(
            v.get("tags"),
            Some(&YamlValue::Seq(vec![
                YamlValue::Text("a".into()),
                YamlValue::Text("b".into())
            ]))
        );
    }

    #[test]
    fn seq_of_maps_with_inline_key() {
        // DSL 典型：children 下每个项是内联映射。
        let src = "children:\n  - component: Text\n    props:\n      text: hi\n  - component: Button\n";
        let v = parse(src).unwrap();
        let children = v.get("children").and_then(YamlValue::as_seq).unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].get("component").and_then(YamlValue::as_text), Some("Text"));
        assert_eq!(
            children[0]
                .get("props")
                .and_then(|p| p.get("text"))
                .and_then(YamlValue::as_text),
            Some("hi")
        );
        assert_eq!(
            children[1].get("component").and_then(YamlValue::as_text),
            Some("Button")
        );
    }

    #[test]
    fn nested_seq_of_seq() {
        let v = parse("- - 1\n  - 2\n- - 3\n").unwrap();
        assert_eq!(
            v,
            YamlValue::Seq(vec![
                YamlValue::Seq(vec![YamlValue::Int(1), YamlValue::Int(2)]),
                YamlValue::Seq(vec![YamlValue::Int(3)]),
            ])
        );
    }

    #[test]
    fn empty_seq_item_is_null() {
        let v = parse("items:\n  -\n  - x\n").unwrap();
        let s = v.get("items").and_then(YamlValue::as_seq).unwrap();
        assert_eq!(s, &[YamlValue::Null, YamlValue::Text("x".into())]);
    }

    // ── 流集合 ────────────────────────────────────────────────────────
    #[test]
    fn flow_collections() {
        assert_eq!(
            parse("tags: [a, b, 3]").unwrap(),
            map(&[(
                "tags",
                YamlValue::Seq(vec![
                    YamlValue::Text("a".into()),
                    YamlValue::Text("b".into()),
                    YamlValue::Int(3)
                ])
            )])
        );
        assert_eq!(
            parse("meta: {x: 1, y: 2}").unwrap(),
            map(&[("meta", map(&[("x", YamlValue::Int(1)), ("y", YamlValue::Int(2))]))])
        );
        assert_eq!(
            parse("nested: [[1, 2], [3]]").unwrap(),
            map(&[(
                "nested",
                YamlValue::Seq(vec![
                    YamlValue::Seq(vec![YamlValue::Int(1), YamlValue::Int(2)]),
                    YamlValue::Seq(vec![YamlValue::Int(3)])
                ])
            )])
        );
    }

    // ── 注释 / 空白 ───────────────────────────────────────────────────
    #[test]
    fn comments_stripped() {
        let v = parse("a: 1 # inline\n# full line\nb: 'x # y'\n").unwrap();
        assert_eq!(v.get("a"), Some(&YamlValue::Int(1)));
        assert_eq!(v.get("b"), Some(&YamlValue::Text("x # y".into())));
    }

    #[test]
    fn value_with_colon_kept() {
        // URL 值：`:` 后非空格 → 不当映射键。
        let v = parse("url: http://example.com/path").unwrap();
        assert_eq!(v.get("url"), Some(&YamlValue::Text("http://example.com/path".into())));
    }

    // ── 错误路径 ──────────────────────────────────────────────────────
    #[test]
    fn errors() {
        assert!(parse("a: 1\n  b: 2\n c: 3").is_err()); // 缩进不一致
        assert!(parse("dup: 1\ndup: 2").is_err()); // 重复键
        assert!(parse("\tindented").is_err()); // Tab 缩进
        assert!(parse(r#""unterminated"#).is_err()); // 未闭合串
    }

    #[test]
    fn empty_source_is_null() {
        assert_eq!(parse("").unwrap(), YamlValue::Null);
        assert_eq!(parse("# only comment\n").unwrap(), YamlValue::Null);
    }
}
