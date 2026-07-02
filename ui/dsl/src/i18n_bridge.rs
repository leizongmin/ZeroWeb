//! DSL `i18n:` 对象 → [`LocalizedText`] 桥接（spec FR-013 / DC-10）。
//!
//! [`crate::loader::YamlLoader`] 把 `i18n:` 节点以 [`Value::Object`] 原样保留在 props 上
//! （不依赖 i18n，保持 ui/core ← ui/dsl 的依赖方向）。本模块在求值期把该对象解析为
//! [`zero_ui_i18n::LocalizedText::Message`]：`i18n` 字段为 message id，`params` 中每个
//! 表达式按 [`EvalContext`] 求值为文本/计数参数，闭合 DSL `i18n:` message id 端到端引用。
//! 仅依赖通用 [`zero_ui_i18n`]，不引入浏览器耦合（DC-1）。

use crate::diagnostics::DslError;
use crate::engine::Engine;
use crate::loader::{EvalContext, ExpressionEngine};
use zero_ui_core::binding::Value;
use zero_ui_i18n::{LocalizedText, MessageParams, MessageRef};

/// 把 DSL 保留的 `i18n:` 对象解析为 [`LocalizedText::Message`]（DC-10）。
///
/// 期望 `value` 为 `Value::Object { i18n: <message id 字符串>, params?: { <name>: <expr 字符串> } }`。
/// 每个 param 表达式按 `ctx` 求值：`Text` → 文本参数，`Int`/`Float` → 计数参数。
/// 缺少 `i18n` id、结构不符、表达式解析/求值失败或参数求值为非文本/计数值时返回 [`DslError`]。
///
/// 产出的 [`LocalizedText`] 交由宿主用 [`zero_ui_i18n::I18nProvider`] 解析为最终文案
/// （本函数不持有 catalog，保持 DSL 与 i18n 解析层职责分离）。
pub fn i18n_value_to_message(value: &Value, engine: &Engine, ctx: &EvalContext) -> Result<LocalizedText, DslError> {
    let obj = match value {
        Value::Object(o) => o,
        _ => return Err(DslError::Parse("i18n prop must be an object".into())),
    };
    let id = match obj.get("i18n") {
        Some(Value::Text(s)) => s.as_str(),
        _ => return Err(DslError::Parse("i18n object missing 'i18n' message id string".into())),
    };
    let mut params = MessageParams::new();
    if let Some(Value::Object(pmap)) = obj.get("params") {
        for (name, expr_val) in pmap {
            let src = match expr_val {
                Value::Text(s) => s.as_str(),
                _ => {
                    return Err(DslError::Parse(format!(
                        "i18n param '{name}' must be an expression string"
                    )));
                }
            };
            let expr = engine.parse(src)?;
            match engine.eval(&expr, ctx)? {
                Value::Text(s) => {
                    params.set_text(name, &s);
                }
                Value::Int(n) => {
                    params.set_count(name, n);
                }
                Value::Float(f) => {
                    params.set_count(name, f as i64);
                }
                other => {
                    return Err(DslError::Validate(format!(
                        "i18n param '{name}' resolved to non-text/count value: {other:?}"
                    )));
                }
            }
        }
    }
    let mut mref = MessageRef::new(id);
    mref.params = params;
    Ok(LocalizedText::Message(mref))
}

/// 便捷判断一个 prop 值是否为 DSL `i18n:` 对象（调用方据此决定是否走 i18n 解析）。
pub fn is_i18n_object(value: &Value) -> bool {
    matches!(value, Value::Object(o) if o.contains_key("i18n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::binding::Value;
    use zero_ui_i18n::message::MessageParamValue;
    use zero_ui_i18n::{
        CatalogStore, I18nContext, I18nProvider, LocaleId, MessageCatalog, MessageEntry, MessageId, TextDirection,
    };

    fn ctx_with_state() -> EvalContext {
        // 模拟应用状态：$state.tabs.count = 3；$state.name = "Ada"。
        let mut state = hashbrown::HashMap::new();
        state.insert("tabs".to_string(), {
            let mut tabs = hashbrown::HashMap::new();
            tabs.insert("count".to_string(), Value::Int(3));
            Value::Object(tabs)
        });
        state.insert("name".to_string(), Value::Text("Ada".to_string()));
        EvalContext::default().with_var("state", Value::Object(state))
    }

    #[test]
    fn parses_message_id_and_count_param() {
        // i18n: tabs.count + params.count = $state.tabs.count → Message(count=3)。
        let mut obj = hashbrown::HashMap::new();
        obj.insert("i18n".to_string(), Value::Text("tabs.count".to_string()));
        let mut params = hashbrown::HashMap::new();
        params.insert("count".to_string(), Value::Text("$state.tabs.count".to_string()));
        obj.insert("params".to_string(), Value::Object(params));
        let value = Value::Object(obj);

        let localized = i18n_value_to_message(&value, &Engine, &ctx_with_state()).unwrap();
        let mref = match localized {
            LocalizedText::Message(m) => m,
            _ => panic!("expected Message"),
        };
        assert_eq!(mref.id, MessageId::new("tabs.count"));
        assert_eq!(
            mref.params.entries.get("count"),
            Some(&MessageParamValue::Count(3)),
            "expression $state.tabs.count evaluated to 3"
        );
    }

    #[test]
    fn parses_text_param() {
        let mut obj = hashbrown::HashMap::new();
        obj.insert("i18n".to_string(), Value::Text("greet".to_string()));
        let mut params = hashbrown::HashMap::new();
        params.insert("name".to_string(), Value::Text("$state.name".to_string()));
        obj.insert("params".to_string(), Value::Object(params));

        let localized = i18n_value_to_message(&Value::Object(obj), &Engine, &ctx_with_state()).unwrap();
        let mref = match localized {
            LocalizedText::Message(m) => m,
            _ => panic!("expected Message"),
        };
        assert_eq!(
            mref.params.entries.get("name"),
            Some(&MessageParamValue::Text("Ada".to_string()))
        );
    }

    #[test]
    fn no_params_yields_plain_message_ref() {
        let mut obj = hashbrown::HashMap::new();
        obj.insert("i18n".to_string(), Value::Text("app.title".to_string()));
        let localized = i18n_value_to_message(&Value::Object(obj), &Engine, &EvalContext::default()).unwrap();
        let mref = match localized {
            LocalizedText::Message(m) => m,
            _ => panic!("expected Message"),
        };
        assert_eq!(mref.id, MessageId::new("app.title"));
        assert!(mref.params.entries.is_empty());
    }

    #[test]
    fn missing_id_is_error() {
        let obj = hashbrown::HashMap::new(); // 无 i18n 键
        assert!(i18n_value_to_message(&Value::Object(obj), &Engine, &EvalContext::default()).is_err());
    }

    #[test]
    fn non_object_is_error() {
        assert!(i18n_value_to_message(&Value::Text("x".into()), &Engine, &EvalContext::default()).is_err());
    }

    #[test]
    fn bad_param_expression_is_error() {
        let mut obj = hashbrown::HashMap::new();
        obj.insert("i18n".to_string(), Value::Text("x".to_string()));
        let mut params = hashbrown::HashMap::new();
        params.insert("count".to_string(), Value::Text("$state.tabs.count >".to_string())); // 语法错误
        obj.insert("params".to_string(), Value::Object(params));
        assert!(i18n_value_to_message(&Value::Object(obj), &Engine, &ctx_with_state()).is_err());
    }

    #[test]
    fn end_to_end_resolve_via_catalog() {
        // DSL i18n 对象 → LocalizedText → CatalogStore.resolve → 最终文案（DC-10 闭环）。
        let mut cat = MessageCatalog {
            locale: LocaleId::new("en"),
            direction: TextDirection::Ltr,
            messages: hashbrown::HashMap::new(),
        };
        cat.messages
            .insert(MessageId::new("tabs.count"), MessageEntry::simple("Tabs: {count}"));
        let mut store = CatalogStore::new();
        store.register(cat);
        let ictx = I18nContext {
            locale: LocaleId::new("en"),
            fallback_chain: vec![LocaleId::new("en")],
            direction: TextDirection::Ltr,
        };

        let mut obj = hashbrown::HashMap::new();
        obj.insert("i18n".to_string(), Value::Text("tabs.count".to_string()));
        let mut params = hashbrown::HashMap::new();
        params.insert("count".to_string(), Value::Text("$state.tabs.count".to_string()));
        obj.insert("params".to_string(), Value::Object(params));

        let localized = i18n_value_to_message(&Value::Object(obj), &Engine, &ctx_with_state()).unwrap();
        let resolved = store.resolve(&localized, &ictx).unwrap();
        assert_eq!(resolved.text, "Tabs: 3");
    }

    #[test]
    fn is_i18n_object_detects_marker() {
        let mut obj = hashbrown::HashMap::new();
        obj.insert("i18n".to_string(), Value::Text("x".to_string()));
        assert!(is_i18n_object(&Value::Object(obj)));
        assert!(!is_i18n_object(&Value::Text("plain".into())));
    }
}
