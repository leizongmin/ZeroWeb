//! # zero-ui-restoration
//!
//! 状态恢复（spec §8.4.1 `zero-ui-restoration` / FR-016 / §8.4.1B session restore）。
//!
//! route stack、scroll offset、selection、widget 临时状态用稳定 RestorationId 持久化；
//! 重启/恢复后 chrome UI 状态可恢复。

use compact_str::CompactString;
use hashbrown::HashMap;
use zero_ui_core::binding::Value;

/// 稳定恢复 id。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestorationId(pub CompactString);

impl RestorationId {
    pub fn new(id: &str) -> RestorationId {
        RestorationId(CompactString::new(id))
    }
}

/// 恢复存储：id → 可序列化值。
#[derive(Debug, Default)]
pub struct RestorationStore {
    map: HashMap<RestorationId, Value>,
}

impl RestorationStore {
    pub fn new() -> RestorationStore {
        RestorationStore::default()
    }

    pub fn save(&mut self, id: RestorationId, value: Value) -> &mut RestorationStore {
        self.map.insert(id, value);
        self
    }

    pub fn restore(&self, id: &RestorationId) -> Option<&Value> {
        self.map.get(id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
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
        // save 返回 &mut，可链式。
        store
            .save(RestorationId::new("a"), Value::Text("x".into()))
            .save(RestorationId::new("b"), Value::Int(1));
        assert_eq!(store.len(), 2);
        assert!(!store.is_empty());
    }
}
