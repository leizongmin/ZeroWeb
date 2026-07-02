//! # zero-ui-restoration
//!
//! 状态恢复（spec §8.4.1 `zero-ui-restoration` / FR-016 / IF-010 `RestorationStore` /
//! §8.4.1B session restore）。
//!
//! route stack、scroll offset、TextInput selection、widget 临时状态用稳定 [`RestorationId`]
//! 持久化；store 支持序列化为 JSON（重启/恢复后 chrome UI 状态可恢复，spec §8.4.1B）。
//!
//! 只恢复 **app UI 状态**；网页 session/history 仍由浏览器模型和 WebView 负责（spec §8.4.10）。

use compact_str::CompactString;
use hashbrown::HashMap;
use zero_ui_core::binding::Value;

/// 稳定恢复 id（点分命名，如 `route.stack`、`tab.2.scroll.y`、`address.selection`）。
///
/// 提供 DC-13 列出的可恢复状态（route/scroll/input selection）的便捷构造器，
/// 以及命名空间前缀（per-tab / per-scope 隔离）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestorationId(pub CompactString);

impl RestorationId {
    pub fn new(id: &str) -> RestorationId {
        RestorationId(CompactString::new(id))
    }

    /// 路由栈恢复点（DC-13：route 可恢复）。
    pub fn route_stack() -> RestorationId {
        RestorationId(CompactString::new("route.stack"))
    }

    /// 某节点的垂直滚动 offset（DC-13：scroll 可恢复）。`widget` 为稳定 WidgetId 字符串。
    pub fn scroll_y(widget: &str) -> RestorationId {
        RestorationId(CompactString::new(format!("{}.scroll.y", widget)))
    }

    /// 某 TextInput 的选区（caret start/end，DC-13：input selection 可恢复）。
    pub fn selection(widget: &str) -> RestorationId {
        RestorationId(CompactString::new(format!("{}.selection", widget)))
    }

    /// 在本 id 前加命名空间前缀（如 `tab.2.`），用于 per-scope 隔离恢复。
    pub fn namespaced(&self, prefix: &str) -> RestorationId {
        RestorationId(CompactString::new(format!("{}{}", prefix, self.0)))
    }
}

/// 恢复存储：id → 可序列化值（spec IF-010 `RestorationStore`）。
///
/// 提供保存/恢复/取出/命名空间清除，以及整体 JSON 序列化（`to_json`/`from_json`），
/// 供宿主在应用重启时持久化与回填 app UI 状态。
#[derive(Debug, Default, Clone)]
pub struct RestorationStore {
    map: HashMap<RestorationId, Value>,
}

impl RestorationStore {
    pub fn new() -> RestorationStore {
        RestorationStore::default()
    }

    /// 保存一个恢复点（链式）。
    pub fn save(&mut self, id: RestorationId, value: Value) -> &mut RestorationStore {
        self.map.insert(id, value);
        self
    }

    /// 取恢复值（不删除）。
    pub fn restore(&self, id: &RestorationId) -> Option<&Value> {
        self.map.get(id)
    }

    /// 取出恢复值（删除，用于一次性回填后清除）。
    pub fn take(&mut self, id: &RestorationId) -> Option<Value> {
        self.map.remove(id)
    }

    /// 移除一个恢复点；返回是否确实存在。
    pub fn remove(&mut self, id: &RestorationId) -> bool {
        self.map.remove(id).is_some()
    }

    /// 清除所有 id 以 `prefix` 开头的恢复点（per-scope 清理，如关闭 tab 时清 `tab.2.*`）。
    /// 返回移除条数。
    pub fn clear_namespace(&mut self, prefix: &str) -> usize {
        let before = self.map.len();
        self.map.retain(|id, _| !id.0.starts_with(prefix));
        before - self.map.len()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 序列化为 JSON 字符串（spec §8.4.1B 重启持久化）。
    ///
    /// 失败仅在与非 JSON 兼容值冲突时（当前 `Value` 全变体均可序列化，实际不会失败）。
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&SerdeEntries::from_store(&self.map))
    }

    /// 从 JSON 字符串恢复（反序列化为新 store）。
    pub fn from_json(json: &str) -> Result<RestorationStore, serde_json::Error> {
        let entries: SerdeEntries = serde_json::from_str(json)?;
        Ok(RestorationStore {
            map: entries.into_map(),
        })
    }
}

/// 内部序列化形态：`{ entries: [(id 字符串, Value), ...] }`，保持 `RestorationId` 对 JSON 友好。
#[derive(serde::Serialize, serde::Deserialize)]
struct SerdeEntries {
    entries: Vec<(String, Value)>,
}

impl SerdeEntries {
    fn from_store(map: &HashMap<RestorationId, Value>) -> SerdeEntries {
        // 按 id 排序，保证 to_json 输出确定（便于 snapshot 测试 / diff）。
        let mut pairs: Vec<(String, Value)> = map.iter().map(|(k, v)| (k.0.to_string(), v.clone())).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        SerdeEntries { entries: pairs }
    }

    fn into_map(self) -> HashMap<RestorationId, Value> {
        self.entries
            .into_iter()
            .map(|(k, v)| (RestorationId(CompactString::new(k)), v))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_restore_roundtrip() {
        let mut store = RestorationStore::new();
        store.save(RestorationId::new("address.text"), Value::Text("zero.example".into()));
        store.save(RestorationId::new("scroll.y"), Value::Int(420));
        assert_eq!(
            store.restore(&RestorationId::new("address.text")),
            Some(&Value::Text("zero.example".into()))
        );
        assert_eq!(store.restore(&RestorationId::new("scroll.y")), Some(&Value::Int(420)));
        assert!(store.restore(&RestorationId::new("missing")).is_none());
    }

    #[test]
    fn len_is_empty_and_save_chain() {
        let mut store = RestorationStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        store
            .save(RestorationId::new("a"), Value::Text("x".into()))
            .save(RestorationId::new("b"), Value::Int(1));
        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());
    }

    #[test]
    fn take_removes_value() {
        let mut store = RestorationStore::new();
        store.save(RestorationId::new("x"), Value::Int(7));
        assert_eq!(store.take(&RestorationId::new("x")), Some(Value::Int(7)));
        // 取出后已删除。
        assert!(store.restore(&RestorationId::new("x")).is_none());
        assert_eq!(store.len(), 0);
        // take 不存在的 key → None。
        assert!(store.take(&RestorationId::new("nope")).is_none());
    }

    #[test]
    fn typed_id_constructors_and_namespace() {
        // route.stack
        assert_eq!(RestorationId::route_stack().0.as_str(), "route.stack");
        // scroll per widget
        assert_eq!(RestorationId::scroll_y("viewport").0.as_str(), "viewport.scroll.y");
        // selection per widget
        assert_eq!(
            RestorationId::selection("address_bar").0.as_str(),
            "address_bar.selection"
        );
        // 命名空间前缀（per-tab 隔离）。
        let base = RestorationId::scroll_y("viewport");
        let namespaced = base.namespaced("tab.2.");
        assert_eq!(namespaced.0.as_str(), "tab.2.viewport.scroll.y");
    }

    #[test]
    fn clear_namespace_removes_prefixed_only() {
        let mut store = RestorationStore::new();
        store.save(RestorationId::new("tab.0.scroll.y"), Value::Int(10));
        store.save(RestorationId::new("tab.0.selection"), Value::Int(0));
        store.save(RestorationId::new("tab.1.scroll.y"), Value::Int(20));
        store.save(RestorationId::new("route.stack"), Value::Text("home".into()));

        // 清 tab.0.* → 移除 2 条。
        let removed = store.clear_namespace("tab.0.");
        assert_eq!(removed, 2);
        assert_eq!(store.len(), 2);
        // tab.1 与 route 不受影响。
        assert!(store.restore(&RestorationId::new("tab.1.scroll.y")).is_some());
        assert!(store.restore(&RestorationId::new("route.stack")).is_some());
        // 前缀只是字符串前缀：清 "tab." 会清掉所有 tab.*。
        let removed2 = store.clear_namespace("tab.");
        assert_eq!(removed2, 1, "tab.1.scroll.y removed");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn json_persistence_roundtrip() {
        // §8.4.1B：重启后 chrome UI 状态可恢复——store 序列化为 JSON 再反序列化回填。
        let mut store = RestorationStore::new();
        store.save(RestorationId::route_stack(), Value::Text("settings".into()));
        store.save(RestorationId::scroll_y("viewport"), Value::Float(128.5));
        store.save(RestorationId::selection("address_bar"), Value::Int(3));
        store.save(
            RestorationId::new("flags"),
            Value::Array(vec![Value::Bool(true), Value::Bool(false)]),
        );

        let json = store.to_json().expect("serialize");
        // JSON 含各恢复点。
        assert!(json.contains("route.stack"));
        assert!(json.contains("viewport.scroll.y"));
        assert!(json.contains("address_bar.selection"));

        let restored = RestorationStore::from_json(&json).expect("deserialize");
        assert_eq!(restored.len(), store.len());
        assert_eq!(
            restored.restore(&RestorationId::route_stack()),
            Some(&Value::Text("settings".into()))
        );
        assert_eq!(
            restored.restore(&RestorationId::scroll_y("viewport")),
            Some(&Value::Float(128.5))
        );
        assert_eq!(
            restored.restore(&RestorationId::selection("address_bar")),
            Some(&Value::Int(3))
        );
        assert_eq!(
            restored.restore(&RestorationId::new("flags")),
            Some(&Value::Array(vec![Value::Bool(true), Value::Bool(false)]))
        );
    }

    #[test]
    fn to_json_is_deterministic() {
        // 同一 store 多次序列化结果一致（按 id 排序，便于 snapshot/diff）。
        let mut store = RestorationStore::new();
        store.save(RestorationId::new("z"), Value::Int(1));
        store.save(RestorationId::new("a"), Value::Int(2));
        store.save(RestorationId::new("m"), Value::Int(3));
        let j1 = store.to_json().unwrap();
        let j2 = store.to_json().unwrap();
        assert_eq!(j1, j2);
        // 排序：a 在 m 前，m 在 z 前。
        let a = j1.find("\"a\"").unwrap();
        let m = j1.find("\"m\"").unwrap();
        let z = j1.find("\"z\"").unwrap();
        assert!(a < m && m < z, "entries sorted by id: {j1}");
    }

    #[test]
    fn from_json_invalid_returns_error() {
        assert!(RestorationStore::from_json("{not valid json").is_err());
    }

    #[test]
    fn empty_store_serializes_to_empty_entries() {
        let store = RestorationStore::new();
        let json = store.to_json().unwrap();
        assert_eq!(json, "{\"entries\":[]}", "empty store → empty entries array");
        let back = RestorationStore::from_json(&json).unwrap();
        assert!(back.is_empty());
    }
}
