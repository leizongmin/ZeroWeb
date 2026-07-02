//! Message 引用与文案模型（spec IF-007）。

use crate::plural::PluralCategory;
use compact_str::CompactString;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// 稳定 message id（点分命名，如 `browser.address.placeholder`，spec FR-013）。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct MessageId(pub CompactString);

impl MessageId {
    pub fn new(id: &str) -> MessageId {
        MessageId(CompactString::new(id))
    }
}

/// 参数值（受控类型，禁止任意对象）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageParamValue {
    Text(String),
    Count(i64),
}

/// 参数集合。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageParams {
    pub entries: HashMap<CompactString, MessageParamValue>,
}

impl MessageParams {
    pub fn new() -> MessageParams {
        MessageParams::default()
    }
    pub fn set_text(&mut self, name: &str, value: &str) -> &mut MessageParams {
        self.entries
            .insert(CompactString::new(name), MessageParamValue::Text(value.to_string()));
        self
    }
    pub fn set_count(&mut self, name: &str, value: i64) -> &mut MessageParams {
        self.entries
            .insert(CompactString::new(name), MessageParamValue::Count(value));
        self
    }
}

/// 带参数的 message 引用。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageRef {
    pub id: MessageId,
    pub params: MessageParams,
}

impl MessageRef {
    pub fn new(id: &str) -> MessageRef {
        MessageRef {
            id: MessageId::new(id),
            params: MessageParams::new(),
        }
    }
}

/// 用户可见字符串（spec IF-007 `LocalizedText`）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LocalizedText {
    /// 直接字面量（仅调试/无 i18n 场景；production DSL 不应硬编码可见文案，spec FR-013）。
    Literal(String),
    /// message id 引用（推荐）。
    Message(MessageRef),
}

/// 单条 message 的目录项（spec IF-007 message entry）。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MessageEntry {
    /// 默认文案（含 `{param}` 占位）。
    pub value: String,
    /// 翻译人员/审核工具用，不进入运行时可见文本。
    pub description: Option<String>,
    /// plural 变体（按 CLDR plural category）。
    pub plural_forms: HashMap<PluralCategory, String>,
}

impl MessageEntry {
    pub fn simple(value: &str) -> MessageEntry {
        MessageEntry {
            value: value.to_string(),
            description: None,
            plural_forms: HashMap::new(),
        }
    }
}
