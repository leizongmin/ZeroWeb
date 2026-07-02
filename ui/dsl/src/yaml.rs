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

/// 块/流嵌套深度上限（lei-deep-review：防恶意深层嵌套 YAML 栈溢出）。
/// 合法 UI WidgetSpec 极少嵌套超过 ~10 层；取 100 留足余量，与 engine.rs
/// 的 `MAX_PARSE_DEPTH=64` 同类守护（解析器不得对任意输入 crash）。
const MAX_YAML_DEPTH: usize = 100;

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
    let v = parse_block(&lines, &mut i, root_indent, 0)?;
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
///
/// **迭代**实现（lei-deep-review 修复）：原递归版对单行 N 个 `- ` 前缀
/// （`- - - ... - item`）会递归 N 深 → 足够多时栈溢出。改迭代消除递归，
/// 输出与原递归版逐位等价（每层 marker 在当前 indent，内容 indent + 2 累加）。
fn expand_dash(line: &Line) -> Vec<Line> {
    let mut out = Vec::new();
    let mut cur_indent = line.indent;
    let mut cur_content = line.content.clone();
    loop {
        if cur_content == "-" {
            out.push(Line {
                indent: cur_indent,
                content: "-".to_string(),
            });
            break;
        }
        if let Some(rest) = cur_content.strip_prefix("- ") {
            let after = rest.trim_start();
            out.push(Line {
                indent: cur_indent,
                content: "-".to_string(),
            });
            if after.is_empty() {
                break;
            }
            cur_indent += 2;
            cur_content = after.to_string();
        } else {
            out.push(Line {
                indent: cur_indent,
                content: cur_content,
            });
            break;
        }
    }
    out
}

/// 去除行内注释 `#`（仅当 `#` 在行首或前导为空白，且不在引号内）。
///
/// 返回**原串切片**（`rest[..cut]` 或 `rest`），不做 byte→char 重建——
/// 保证非 ASCII（中文/日文/韩文/emoji）内容保真（lei-deep-review 修复）。
fn strip_comment(rest: &str) -> Result<String, DslError> {
    let bytes = rest.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0usize;
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
            if c == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        // 非引号上下文：`#` 在行首或前导空格 → 注释起点。
        // `#` 是 ASCII（0x23），其起始字节位置必为 UTF-8 字符边界 → `rest[..i]` 切片合法。
        if c == b'#' && (i == 0 || bytes[i - 1] == b' ') {
            return Ok(rest[..i].to_string());
        }
        if c == b'\'' {
            in_single = true;
        } else if c == b'"' {
            in_double = true;
        }
        i += 1;
    }
    Ok(rest.to_string())
}

// ════════════════════════════════════════════════════════════════════════
// 块结构解析（递归下降，按缩进）
// ════════════════════════════════════════════════════════════════════════

fn parse_block(lines: &[Line], i: &mut usize, indent: usize, depth: usize) -> Result<YamlValue, DslError> {
    if depth > MAX_YAML_DEPTH {
        return Err(DslError::EvalResourceLimit(format!("YAML 嵌套深度 > {MAX_YAML_DEPTH}")));
    }
    let content = &lines[*i].content;
    if content == "-" {
        parse_seq(lines, i, indent, depth)
    } else if split_key(content).is_some() {
        parse_map(lines, i, indent, depth)
    } else {
        // 单行标量 / 流集合
        let v = parse_scalar_or_flow(content)?;
        *i += 1;
        Ok(v)
    }
}

fn parse_map(lines: &[Line], i: &mut usize, indent: usize, depth: usize) -> Result<YamlValue, DslError> {
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
            // 嵌套块或 null（下一层 depth + 1）
            if *i < lines.len() && lines[*i].indent > indent {
                parse_block(lines, i, lines[*i].indent, depth + 1)?
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

fn parse_seq(lines: &[Line], i: &mut usize, indent: usize, depth: usize) -> Result<YamlValue, DslError> {
    let mut items: Vec<YamlValue> = Vec::new();
    while *i < lines.len() {
        let line = &lines[*i];
        if line.indent != indent || line.content != "-" {
            break;
        }
        *i += 1; // 消费 marker
        if *i < lines.len() && lines[*i].indent > indent {
            items.push(parse_block(lines, i, lines[*i].indent, depth + 1)?);
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
        // 双引号：按 **char** 迭代（非 byte），保证非 ASCII 内容保真 + 正确处理转义
        // （lei-deep-review 修复：原 byte→char 重建会损坏中文等多字节字符）。
        let chars: Vec<char> = s.chars().collect();
        let mut out = String::new();
        let mut i = 1usize;
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' && i + 1 < chars.len() {
                let e = chars[i + 1];
                out.push(match e {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    other => other,
                });
                i += 2;
                continue;
            }
            if c == '"' {
                let trailing: String = chars[i + 1..].iter().collect();
                let trailing = trailing.trim();
                if !trailing.is_empty() {
                    return Err(DslError::Parse(format!("双引号后多余内容：{trailing}")));
                }
                return Ok(YamlValue::Text(out));
            }
            out.push(c);
            i += 1;
        }
        return Err(DslError::Parse("未闭合的双引号字符串".into()));
    }
    // 单引号：`''` → `'`（切片操作，UTF-8 天然保真）
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
        depth: 0,
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
    /// 当前嵌套深度（lei-deep-review：防 `[[[...]]]` / `{a:{b:{...}}}` 栈溢出）。
    depth: usize,
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
        self.depth += 1;
        if self.depth > MAX_YAML_DEPTH {
            return Err(DslError::EvalResourceLimit(format!(
                "YAML 流集合嵌套深度 > {MAX_YAML_DEPTH}"
            )));
        }
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
        self.depth += 1;
        if self.depth > MAX_YAML_DEPTH {
            return Err(DslError::EvalResourceLimit(format!(
                "YAML 流集合嵌套深度 > {MAX_YAML_DEPTH}"
            )));
        }
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

    // ── 深度审查（lei-deep-review）：UTF-8 保真 + 嵌套深度守卫 ──────────────
    #[test]
    fn unicode_unquoted_plain_scalar_preserved() {
        // 非 ASCII（中文）纯标量此前经 strip_comment 的 byte→char 重建被损坏。
        // 对支持 i18n（DC-10）的浏览器 UI SDK，中文 DSL 标签必须保真。
        let v = parse("label: 中文").unwrap();
        assert_eq!(v.get("label"), Some(&YamlValue::Text("中文".into())));
    }

    #[test]
    fn unicode_double_quoted_string_preserved() {
        // 双引号非 ASCII 此前经 parse_quoted_scalar 的 byte→char 重建被损坏。
        let v = parse(r#"label: "你好，世界""#).unwrap();
        assert_eq!(v.get("label"), Some(&YamlValue::Text("你好，世界".into())));
    }

    #[test]
    fn unicode_in_flow_collection_preserved() {
        // 流集合内非 ASCII（FlowParser 已 char-based，回归守卫防退化）。
        let v = parse(r#"tags: [中文, "日本語"]"#).unwrap();
        let s = v.get("tags").and_then(YamlValue::as_seq).unwrap();
        assert_eq!(s, &[YamlValue::Text("中文".into()), YamlValue::Text("日本語".into()),]);
    }

    #[test]
    fn unicode_with_inline_comment_preserved() {
        // 非 ASCII + 行内注释：strip_comment 必须切到 # 前，且不损坏非 ASCII 内容。
        let v = parse("label: 中文 # 注释").unwrap();
        assert_eq!(v.get("label"), Some(&YamlValue::Text("中文".into())));
    }

    #[test]
    fn deeply_nested_block_yaml_rejected_by_depth_guard() {
        // 极深嵌套 YAML 此前无深度守卫 → 足够深时栈溢出崩溃整个进程。
        // 现 > MAX_YAML_DEPTH → 干净 EvalResourceLimit（不 crash）。
        let mut src = String::new();
        for d in 0..(MAX_YAML_DEPTH + 20) {
            for _ in 0..(d * 2) {
                src.push(' ');
            }
            src.push_str("a:\n");
        }
        match parse(&src) {
            Err(DslError::EvalResourceLimit(_)) => {}
            other => panic!("expected EvalResourceLimit for deep block nesting, got {other:?}"),
        }
    }

    #[test]
    fn deeply_nested_flow_rejected_by_depth_guard() {
        // 极深嵌套流集合 [[[[...]]]] 此前无深度守卫 → 栈溢出；现 > MAX_YAML_DEPTH → 干净错误。
        let open = "[".repeat(MAX_YAML_DEPTH + 20);
        let close = "]".repeat(MAX_YAML_DEPTH + 20);
        let src = format!("x: {open}{close}");
        match parse(&src) {
            Err(DslError::EvalResourceLimit(_)) => {}
            other => panic!("expected EvalResourceLimit for deep flow nesting, got {other:?}"),
        }
    }

    #[test]
    fn deeply_nested_dash_seq_does_not_overflow() {
        // 单行 N 个 `- ` 前缀（`- - - ... - item`）此前 expand_dash 递归 N 深 → 栈溢出。
        // 改迭代后不再递归；即使超量也由块深度守卫兜住（不 crash）。
        let dash_line = "- ".repeat(50) + "item";
        let v = parse(&dash_line).unwrap();
        // 50 层嵌套 Seq，最内层 Text("item")。
        let mut cur = &v;
        for _ in 0..50 {
            match cur {
                YamlValue::Seq(items) if items.len() == 1 => cur = &items[0],
                other => panic!("expected nested single-item Seq, got {other:?}"),
            }
        }
        assert_eq!(cur, &YamlValue::Text("item".into()));
    }
}
