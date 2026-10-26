//! import attributes 静态校验（s105，html-semantics-compat）。
//!
//! 语义（WPT import-attributes 判据；提案静态语义 + HTML module type map）：
//! - 属性键 ∉ {type} → SyntaxError，且先于 specifier 解析错误与 type 值校验
//!   （invalid-import-errors-order：unknown key 的 SyntaxError 优先于同句/后句
//!   invalid specifier 的 TypeError）；
//! - type 值 ∉ HTML module type map → TypeError（invalid-type-attribute-error：
//!   notARealType/""/js/javascript 全 TypeError——HTML 集成不注册 "js"/"javascript"，
//!   JS 模块以无属性为默认形态）；
//! - 合法属性（json/css/text）不产生校验错误（数据模块取回/MIME 面归 css/json/
//!   text-module 切片，此处只做静态面）。
//!
//! 仅扫描静态 `import` 语句。 FIXME: `export … from` 属性子句与动态
//! `import()` 的 with 选项未覆盖（语料未及）。
//!
//! https://tc39.es/proposal-import-attributes/
//! https://html.spec.whatwg.org/multipage/webappapis.html#module-type-map

/// import attributes 静态校验失败：`error_ctor` 为 window error 事件 `ev.error`
/// 应当的错误构造器名（SyntaxError/TypeError），`message` 为报告文本。
pub struct ImportAttributeError {
    /// window error 事件 `ev.error` 应当的错误构造器名（"SyntaxError"/"TypeError"）。
    pub error_ctor: &'static str,
    /// 报告文本（经 script_report_module_failure 报向 window）。
    pub message: String,
}

/// HTML module type map 当前注册的 type 值。JS 为无属性默认形态——"js"/"javascript"
/// 未注册（WPT invalid-type-attribute-error 判据：js/javascript/notARealType/""
/// 全 TypeError）；"text" 按 text-module 语料判据收录（WPT text-module 族 32 处
/// `type: "text"`——该域提案形态与 CSS/JSON module 并行）。
const VALID_MODULE_TYPES: [&str; 3] = ["json", "css", "text"];

/// 对模块源码做 import attributes 静态校验：逐静态 import 语句（源码序）先校验
/// 属性键（任一非 `type` → SyntaxError），再校验 type 值（不在
/// [`VALID_MODULE_TYPES`] → TypeError），返回首错。
pub fn validate_static_import_attributes(source: &str) -> Result<(), ImportAttributeError> {
    for stmt in crate::es_module::split_statements(source) {
        let trimmed = stmt.trim();
        let Some(clause) = crate::es_module::strip_import_keyword(trimmed) else {
            continue;
        };
        if !crate::es_module::import_clause_shape_ok(clause) {
            continue;
        }
        let Some(suffix) = suffix_after_specifier_literal(clause) else {
            continue;
        };
        let Some(attrs) = parse_with_clause(suffix)? else {
            continue;
        };
        // 属性键校验全部先于 type 值校验（invalid-import-errors-order 第二模块
        // `{ unknown: "foo", type: "unknown" }` 判 SyntaxError 判据）。
        for (key, _) in &attrs {
            if key != "type" {
                return Err(ImportAttributeError {
                    error_ctor: "SyntaxError",
                    message: format!("Unsupported import attribute key \"{key}\""),
                });
            }
        }
        if let Some((_, ty)) = attrs.iter().find(|(key, _)| key == "type")
            && !VALID_MODULE_TYPES.contains(&ty.as_str())
        {
            return Err(ImportAttributeError {
                error_ctor: "TypeError",
                message: format!("Unsupported module type \"{ty}\""),
            });
        }
    }
    Ok(())
}

/// 定位 import 语句中 specifier 字符串字面量的结束位置，返回其后的剩余子句（用于
/// 探测 `with {…}` 属性子句）。字面量未闭合视为无属性。裸字符串形态（`import"m"`）
/// 字面量在子句首；绑定形态经 `from` 定位（[`crate::es_module::split_from_clause`]
/// 的 spec_part 保证以引号开头）。
pub(crate) fn suffix_after_specifier_literal(clause: &str) -> Option<&str> {
    let literal = if matches!(clause.chars().next(), Some('\'') | Some('"') | Some('`')) {
        clause
    } else {
        crate::es_module::split_from_clause(clause)?.1
    };
    let close = literal.chars().next()?;
    let mut escaped = false;
    for (idx, ch) in literal.char_indices().skip(1) {
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == close {
            return Some(&literal[idx + ch.len_utf8()..]);
        }
    }
    None
}

/// 解析 specifier 之后的 `with {…}` 属性子句；无 with 子句（含 `withstand` 这类
/// 标识符续字）返 None。属性块语法错误（未闭合块 / 值非字符串字面量）按提案语法
/// 静态语义报 SyntaxError。返回有序 (key, value) 列表。
pub(crate) fn parse_with_clause(suffix: &str) -> Result<Option<Vec<(String, String)>>, ImportAttributeError> {
    let rest = match suffix.trim_start().strip_prefix("with") {
        Some(rest) if rest.starts_with('{') || rest.chars().next().is_some_and(char::is_whitespace) => {
            rest.trim_start()
        }
        _ => return Ok(None),
    };
    let Some(body) = rest.strip_prefix('{') else {
        return Ok(None);
    };
    let Some(close_rel) = body.find('}') else {
        return Err(syntax_error("unterminated import attributes block"));
    };
    let mut attrs = Vec::new();
    for entry in split_top_level_commas(&body[..close_rel]) {
        let entry = entry.trim();
        if entry.is_empty() {
            continue; // 尾逗号 `with { type: "json", }`
        }
        let Some((key_raw, value_raw)) = entry.split_once(':') else {
            return Err(syntax_error(&format!("malformed import attribute: {entry}")));
        };
        // 键允许标识符与字符串字面量两种形态（ModuleExportName）
        let key_raw = key_raw.trim();
        let key = if matches!(key_raw.chars().next(), Some('\'') | Some('"'))
            && let Ok(s) = crate::es_module::extract_string_literal(key_raw)
        {
            s
        } else {
            key_raw.to_string()
        };
        let Ok(value) = crate::es_module::extract_string_literal(value_raw.trim()) else {
            return Err(syntax_error(&format!(
                "import attribute values must be string literals: {entry}"
            )));
        };
        attrs.push((key, value));
    }
    Ok(Some(attrs))
}

/// 属性块内按顶层 `,` 切分（跳过字符串字面量内的逗号）。
fn split_top_level_commas(inner: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_str: Option<char> = None;
    let mut escaped = false;
    for (idx, ch) in inner.char_indices() {
        if let Some(close) = in_str {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == close {
                in_str = None;
            }
        } else if matches!(ch, '\'' | '"' | '`') {
            in_str = Some(ch);
        } else if ch == ',' {
            parts.push(&inner[start..idx]);
            start = idx + 1;
        }
    }
    parts.push(&inner[start..]);
    parts
}

fn syntax_error(message: &str) -> ImportAttributeError {
    ImportAttributeError {
        error_ctor: "SyntaxError",
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctor(source: &str) -> Option<&'static str> {
        validate_static_import_attributes(source).err().map(|e| e.error_ctor)
    }

    // invalid-type-attribute-error：notARealType/""/js/javascript 全 TypeError
    //（HTML module type map 不注册 "js"/"javascript"）
    #[test]
    fn invalid_type_values_are_typeerror_s105() {
        for ty in ["notARealType", "", "js", "javascript"] {
            assert_eq!(
                ctor(&format!("import \"./hello.js\" with {{ type: \"{ty}\" }};")),
                Some("TypeError"),
                "type 值 {ty:?} 须 TypeError"
            );
        }
    }

    // unsupported-attribute：未知属性键 → SyntaxError
    #[test]
    fn unknown_attribute_key_is_syntaxerror_s105() {
        assert_eq!(
            ctor("import \"./hello.js\" with { unsupportedAttributeKey: \"foo\" };"),
            Some("SyntaxError")
        );
    }

    // invalid-import-errors-order 三模块：unknown key 先于 invalid specifier /
    // invalid type 值 / 后句 invalid specifier
    #[test]
    fn unknown_key_precedes_specifier_and_type_errors_s105() {
        assert_eq!(
            ctor("import \"INVALID\" with { unknown: \"foo\" };"),
            Some("SyntaxError")
        );
        assert_eq!(
            ctor("import \"./valid\" with { unknown: \"foo\", type: \"unknown\" };"),
            Some("SyntaxError")
        );
        assert_eq!(
            ctor("import \"./valid\" with { unknown: \"foo\" };\nimport \"INVALID\";"),
            Some("SyntaxError")
        );
    }

    #[test]
    fn valid_types_and_no_attributes_pass_s105() {
        assert!(validate_static_import_attributes("import \"./x.js\" with { type: \"json\" };").is_ok());
        assert!(validate_static_import_attributes("import \"./x.js\" with { type: \"css\" };").is_ok());
        assert!(validate_static_import_attributes("import \"./x.js\" with { type: \"text\" };").is_ok());
        assert!(validate_static_import_attributes("import \"./x.js\";").is_ok());
        assert!(validate_static_import_attributes("import v, { a } from \"./x.js\";").is_ok());
        assert!(
            validate_static_import_attributes("import \"./x.js\" with {};").is_ok(),
            "空属性块非错误（empty-attributes-clause 已 Pass 面）"
        );
        assert!(
            validate_static_import_attributes("import v from \"./x.js\" with { type: \"json\", };").is_ok(),
            "尾逗号合法"
        );
    }

    #[test]
    fn prose_and_dynamic_forms_not_misparsed_s105() {
        // 字符串正文里含 import+with 文本——split_statements 字符串上下文隔离
        assert!(validate_static_import_attributes("const s = \"import x from './y' with { type: 'bogus' }\";").is_ok());
        // 动态 import() 不属静态扫描面
        assert!(validate_static_import_attributes("import('./y.js', { with: { type: 'bogus' } });").is_ok());
        // `withstand` 标识符续字不构成 with 子句
        assert!(validate_static_import_attributes("import \"./x.js\" withstand;").is_ok());
    }

    #[test]
    fn compressed_forms_s105() {
        assert!(validate_static_import_attributes("import\"./x.js\"with{type:\"json\"};").is_ok());
        assert_eq!(ctor("import\"./x.js\"with{type:\"bogus\"};"), Some("TypeError"));
    }

    #[test]
    fn malformed_blocks_are_syntaxerror_s105() {
        assert_eq!(
            ctor("import \"./x.js\" with { type: json };"),
            Some("SyntaxError"),
            "值须字符串字面量"
        );
        assert_eq!(
            ctor("import \"./x.js\" with { type: \"json\";"),
            Some("SyntaxError"),
            "块未闭合"
        );
    }
}
