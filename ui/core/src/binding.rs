//! 数据绑定模型 — Value 类型、props 映射、状态路径、绑定声明。
//!
//! `Value` 是 UI SDK 通用的受控值类型，被 props、action payload 与（M3）DSL 表达式共用。
//! spec IF-005 把 `Expression` 求值结果也收敛到 `Value`。

use compact_str::CompactString;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// 受控动态值。禁止承载可执行/脚本对象。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
    /// 有序数组。
    Array(Vec<Value>),
    /// 字符串 → 值的对象（保留插入顺序语义由调用方负责）。
    Object(HashMap<String, Value>),
}

impl Value {
    /// 是否为「真值」（DSL `if` / `visible_when` 判断用）。
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::Text(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }
}

/// DSL/typecheck 用的静态值类型（spec IF-005 `ValueType`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueType {
    Null,
    Bool,
    Int,
    Float,
    Number,
    Text,
    Array,
    Object,
    /// 未知/尚未推断。
    Any,
}

impl Value {
    /// 推断单个值的类型（数字统一归到 Int/Float；`Number` 仅在 typecheck 合并分支时产生）。
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Bool(_) => ValueType::Bool,
            Value::Int(_) => ValueType::Int,
            Value::Float(_) => ValueType::Float,
            Value::Text(_) => ValueType::Text,
            Value::Array(_) => ValueType::Array,
            Value::Object(_) => ValueType::Object,
        }
    }
}

/// Props 容器：component 名 → 属性值。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PropsMap {
    pub entries: HashMap<CompactString, Value>,
}

impl PropsMap {
    pub fn new() -> PropsMap {
        PropsMap::default()
    }

    pub fn insert(&mut self, key: &str, value: Value) -> &mut PropsMap {
        self.entries.insert(CompactString::new(key), value);
        self
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries.get(key)
    }
}

/// 点分状态路径（如 `tabs.active.url`），DSL `Path` 表达式与双向绑定使用。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StatePath(pub Vec<CompactString>);

impl StatePath {
    pub fn parse(dot_path: &str) -> StatePath {
        let segments = dot_path
            .split('.')
            .filter(|s| !s.is_empty())
            .map(CompactString::new)
            .collect();
        StatePath(segments)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// 单条绑定声明（spec IF-005 `Binding`）：把 widget 的某属性绑定到一个表达式/状态路径。
///
/// M1 只承载结构；表达式求值在 `ui/dsl`（M3）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    /// 目标属性名（如 `enabled`、`text`）。
    pub target: CompactString,
    /// 绑定源（M1 用字符串形式承载表达式原文，M3 由 parser 解析为 `Expression`）。
    pub source: CompactString,
}

/// 绑定 schema：声明已知属性 → 期望 `ValueType`，供 DSL typecheck（M3）使用。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BindingSchema {
    pub props: HashMap<CompactString, ValueType>,
}

impl BindingSchema {
    pub fn declare(&mut self, prop: &str, ty: ValueType) -> &mut BindingSchema {
        self.props.insert(CompactString::new(prop), ty);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_truthy_and_type() {
        assert!(!Value::Null.is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(5).is_truthy());
        assert!(Value::Text("x".into()).is_truthy());
        assert!(!Value::Text("".into()).is_truthy());
        assert_eq!(Value::Float(1.0).value_type(), ValueType::Float);
        assert_eq!(Value::Bool(true).value_type(), ValueType::Bool);
    }

    #[test]
    fn props_map_roundtrip() {
        let mut p = PropsMap::new();
        p.insert("label", Value::Text("hello".into()));
        p.insert("count", Value::Int(3));
        assert_eq!(p.get("label"), Some(&Value::Text("hello".into())));
        assert_eq!(p.get("count"), Some(&Value::Int(3)));
        assert_eq!(p.get("missing"), None);
    }

    #[test]
    fn state_path_parse() {
        let p = StatePath::parse("tabs.active.url");
        assert_eq!(p.0.len(), 3);
        assert_eq!(p.0[2].as_str(), "url");
        assert!(StatePath::parse("").is_empty());
        assert!(StatePath::parse("a..b").0.len() == 2); // 空段被过滤
    }

    #[test]
    fn binding_schema_declare() {
        let mut s = BindingSchema::default();
        s.declare("enabled", ValueType::Bool).declare("count", ValueType::Int);
        assert_eq!(s.props.get("enabled"), Some(&ValueType::Bool));
    }
}
