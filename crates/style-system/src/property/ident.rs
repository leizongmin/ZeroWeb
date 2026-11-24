//! ident() 函数文法与求值（CSS Values 5 §ident）。
//!
//! arbitrary substitution function：写面只验外壳 `ident( <declaration-value> )`
//!（[`ident_call_outer_args`] + [`ident_declaration_value_valid`]），computed
//! 期按 `<ident-arg>+` 拼接求值（[`eval_ident_concat`]）。integer 位（含 math
//! 函数形）求值由调用方注入 sibling/font 上下文。消费属性 view-transition-name
//!（wptfyi-css kc5r93 刀）直连；后续 ident() 消费属性刀复用。
//! FIXME: ident token 转义序列（`\` 形）与非 ASCII（>U+007F）词法未实现——
//! 语料面（ident-function-parsing/computed）只测 ASCII 无转义形。
//! <https://drafts.csswg.org/css-values-5/#ident>

/// 值形是否为 `ident( <args> )` 全值形（平衡括号定位，string token 内 `)` 不
/// 误剥）；返回参数原文（首尾空白保留——外壳文法不规范化）。空参 `ident()`/
/// `ident( )` 返回 None（`<declaration-value>` 非空）。
pub(crate) fn ident_call_outer_args(value: &str) -> Option<&str> {
    let rest = value.strip_prefix("ident(")?;
    let bytes = rest.as_bytes();
    let mut depth = 1usize;
    let mut i = 0;
    let mut in_string: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                in_string = None;
            }
        } else if b == b'"' || b == b'\'' {
            in_string = Some(b);
        } else if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
            if depth == 0 {
                // 必须恰在串尾（尾随内容 = 非 ident() 全值形，落裸 custom-ident
                // 臂裁决）；参数须非空。
                return if i + 1 == bytes.len() && !rest[..i].trim().is_empty() {
                    Some(&rest[..i])
                } else {
                    None
                };
            }
        }
        i += 1;
    }
    None
}

/// 外壳 `<declaration-value>` 文法：顶层无逗号、无 `{}` 块（string token 内
/// 豁免）。ident-arg 外形有效性（`ident(5px)` 等）不在此判——解析时恒收，
/// computed 期 apply 臂裁决（round-trip as specified）。
pub(crate) fn ident_declaration_value_valid(inner: &str) -> bool {
    let bytes = inner.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    let mut in_string: Option<u8> = None;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = in_string {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                in_string = None;
            }
        } else if b == b'"' || b == b'\'' {
            in_string = Some(b);
        } else if b == b'(' {
            depth += 1;
        } else if b == b')' {
            depth -= 1;
        } else if depth == 0 && (b == b',' || b == b'{') {
            return false;
        }
        i += 1;
    }
    true
}

/// 裸 `<custom-ident>` token 文法最小面：首字符 `[a-zA-Z_-]`（`-` 后跟 digit
/// 是 number 非 ident），尾随 `[a-zA-Z0-9_-]`。FIXME: `\` 转义与非 ASCII 未实现。
pub(crate) fn is_custom_ident_token(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() {
        return false;
    }
    let first = b[0];
    if !(first.is_ascii_alphabetic() || first == b'_' || first == b'-') {
        return false;
    }
    if first == b'-' && b.len() >= 2 && b[1].is_ascii_digit() {
        return false;
    }
    b[1..]
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

/// `ident( <ident-arg>+ )` 求值：参数原文（首尾空白保留）按 token 序列拼接——
/// string 去引号、custom-ident/dashed-ident 原文、`<integer>` 字面或 math
/// 函数形（求值见 [`crate::property::parse::eval_int_math_value`]）。非
/// `<ident-arg>+`（5px/rgb()/{}/空参）返回 None = computed 无效（消费属性
/// IACVT）。
pub(crate) fn eval_ident_concat(inner: &str, sibling: Option<(u64, u64)>, font_px: Option<f64>) -> Option<String> {
    let mut out = String::new();
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let b = bytes[i];
        if b == b'"' || b == b'\'' {
            // string token：闭合引号定位（`\"` 转义 FIXME 简化——扫描不过转义）。
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b {
                j += 1;
            }
            if j >= bytes.len() {
                return None;
            }
            out.push_str(&inner[start..j]);
            i = j + 1;
        } else if b == b'-' && i + 1 < bytes.len() && bytes[i + 1] == b'-' {
            // dashed-ident：`--` 前缀词（css-syntax ident token；`-- myident`
            // 的 `--` 落此臂拼入）。
            let end = take_ident_run(bytes, i);
            out.push_str(&inner[i..end]);
            i = end;
        } else if b == b'-' || b == b'+' || b.is_ascii_digit() {
            // <integer> 字面或 math 函数形：取平衡 token 整体求值；`5px`
            //（number 非法形）两路皆败 → None。
            let end = take_balanced_token(bytes, i);
            let tok = inner[i..end].trim();
            let v = tok
                .parse::<i64>()
                .ok()
                .or_else(|| super::parse::eval_int_math_value(tok, sibling, font_px))?;
            out.push_str(&v.to_string());
            i = end;
        } else if b.is_ascii_alphabetic() || b == b'_' {
            // ident 起点：平衡 token（ident + 可选括号体）。函数形（`rgb(...)`）
            // 交 eval_int_math_value 尝试（`calc(1 + 2)` 系）；非 math 函数
            // （rgb() 等）→ None；纯 ident 拼入。
            let end = take_balanced_token(bytes, i);
            let tok = &inner[i..end];
            if tok.contains('(') {
                let v = super::parse::eval_int_math_value(tok, sibling, font_px)?;
                out.push_str(&v.to_string());
            } else {
                out.push_str(tok);
            }
            i = end;
        } else {
            // `,`、`{`、`%` 等非 <ident-arg> token → computed 无效。
            return None;
        }
    }
    if out.is_empty() {
        return None;
    }
    Some(serialize_custom_ident(&out))
}

/// CSSOM serialize an identifier（数字开头形）：首字符 digit → `\` + 两位小写
/// hex + 空格（`33 myident` → `\33 myident`）。FIXME: 完整转义规则（中间
/// digit、非 ASCII、控制字符）未实现——语料面只测数字开头形。
/// <https://drafts.csswg.org/cssom/#serialize-an-identifier>
fn serialize_custom_ident(s: &str) -> String {
    if let Some(&b) = s.as_bytes().first() {
        if b.is_ascii_digit() {
            return format!("\\{:02x} {}", b, &s[1..]);
        }
    }
    s.to_string()
}

/// ident 词素终点（`[a-zA-Z0-9_-]` run，dashed-ident 同文法）。
fn take_ident_run(bytes: &[u8], start: usize) -> usize {
    let mut j = start;
    while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_' || bytes[j] == b'-') {
        j += 1;
    }
    j
}

/// 平衡 token 终点：ident 词素后若跟 `(` 则括号体深度配对（string 不出现于
/// math token——FIXME 简化），否则词素自身。
fn take_balanced_token(bytes: &[u8], start: usize) -> usize {
    let mut j = take_ident_run(bytes, start);
    if j < bytes.len() && bytes[j] == b'(' {
        let mut depth = 0i32;
        while j < bytes.len() {
            match bytes[j] {
                b'(' => depth += 1,
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        return j + 1;
                    }
                }
                _ => {}
            }
            j += 1;
        }
    }
    j
}

/// [`serialize_custom_ident`] 自产形的裸值入口（CSSOM serialize an identifier
/// 数字开头形的反向通道）：`\` + 1-6 hex + 空白 + `[\w-]*`。round-trip 链
/// 二次赋值（computed-testcommon：`setProperty('')` 后重赋 serialized 串
/// `\\33 myident`）依赖此形可写。存原样（再序列化恒等）。FIXME: 完整
/// css-syntax escape 词法（中间转义/非 ASCII/EOF 形）未实现。
pub(crate) fn is_serialized_ident_form(s: &str) -> bool {
    let b = s.as_bytes();
    if b.first() != Some(&b'\\') {
        return false;
    }
    let mut j = 1;
    let mut n = 0;
    while j < b.len() && n < 6 && b[j].is_ascii_hexdigit() {
        j += 1;
        n += 1;
    }
    if n == 0 {
        return false;
    }
    if j < b.len() && b[j] == b' ' {
        j += 1;
    }
    b[j..]
        .iter()
        .all(|&c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}
