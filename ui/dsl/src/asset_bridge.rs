//! DSL `asset:` 引用 → [`AssetId`] 桥接（spec FR-016 / DC-6 phase-5）。
//!
//! [`crate::loader::YamlLoader`] 把 `asset:` 形式的 prop 值以 [`Value::Object`]（`{ asset: <id> }`）
//! 原样保留；本模块在求值/装配期把它解析为 [`zero_ui_assets::AssetId`]，供宿主用
//! [`AssetProvider`](zero_ui_assets::AssetProvider) 加载实际资源（图标/图片/shader）。
//! 仅依赖通用 [`zero_ui_assets`]，不引入浏览器耦合（DC-1）。

use zero_ui_assets::AssetId;
use zero_ui_core::binding::Value;

/// 判断 prop 值是否为 DSL `asset:` 引用对象（`{ asset: <id> }`）。
pub fn is_asset_object(value: &Value) -> bool {
    matches!(value, Value::Object(o) if o.contains_key("asset"))
}

/// 从 `{ asset: <id> }` 提取 [`AssetId`]；非该结构或 id 非文本返回 `None`。
pub fn asset_id_of(value: &Value) -> Option<AssetId> {
    if let Value::Object(o) = value
        && let Some(Value::Text(id)) = o.get("asset")
    {
        return Some(AssetId::new(id));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset_object(id: &str) -> Value {
        let mut m = hashbrown::HashMap::new();
        m.insert("asset".to_string(), Value::Text(id.to_string()));
        Value::Object(m)
    }

    #[test]
    fn extracts_asset_id_from_object() {
        let v = asset_object("icon.nav.back");
        assert_eq!(asset_id_of(&v), Some(AssetId::new("icon.nav.back")));
        assert!(is_asset_object(&v));
    }

    #[test]
    fn ignores_non_asset_values() {
        assert!(!is_asset_object(&Value::Text("icon.nav.back".into())));
        assert!(asset_id_of(&Value::Text("icon.nav.back".into())).is_none());
        // 对象但无 asset 键。
        let mut m = hashbrown::HashMap::new();
        m.insert("label".to_string(), Value::Text("x".into()));
        assert!(!is_asset_object(&Value::Object(m)));
    }

    #[test]
    fn ignores_asset_key_with_non_text_value() {
        let mut m = hashbrown::HashMap::new();
        m.insert("asset".to_string(), Value::Int(7));
        assert!(is_asset_object(&Value::Object(m.clone())));
        assert!(asset_id_of(&Value::Object(m)).is_none(), "non-text asset id → None");
    }
}
