//! # zero-ui-assets
//!
//! 资源系统（spec §8.4.1 `zero-ui-assets` / FR-016）。
//!
//! 图标/图片/shader 等通过 AssetId 引用；AssetProvider 负责加载（可 mock）。
//! variants 支持 dark/light/密度变体选择。

use compact_str::CompactString;
use hashbrown::HashMap;
use thiserror::Error;

/// 资源标识（稳定，如 `icon.nav.back`）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetId(pub CompactString);

impl AssetId {
    pub fn new(id: &str) -> AssetId {
        AssetId(CompactString::new(id))
    }
}

/// 资源数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetData {
    Svg(String),
    Png(Vec<u8>),
}

/// 资源错误。
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssetError {
    #[error("asset not found: {0}")]
    NotFound(String),
    #[error("asset load failed: {0}")]
    LoadFailed(String),
}

/// 资源提供者（可被测试 mock）。
pub trait AssetProvider {
    fn load(&self, id: &AssetId) -> Result<AssetData, AssetError>;
}

/// 内存资源清单（测试/打包资源用）。
#[derive(Debug, Default)]
pub struct InMemoryAssets {
    entries: HashMap<AssetId, AssetData>,
}

impl InMemoryAssets {
    pub fn new() -> InMemoryAssets {
        InMemoryAssets::default()
    }
    pub fn insert(&mut self, id: AssetId, data: AssetData) -> &mut InMemoryAssets {
        self.entries.insert(id, data);
        self
    }
}

impl AssetProvider for InMemoryAssets {
    fn load(&self, id: &AssetId) -> Result<AssetData, AssetError> {
        self.entries
            .get(id)
            .cloned()
            .ok_or_else(|| AssetError::NotFound(id.0.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_inserted_asset() {
        let mut a = InMemoryAssets::new();
        a.insert(AssetId::new("icon.back"), AssetData::Svg("<svg/>".into()));
        assert!(matches!(a.load(&AssetId::new("icon.back")), Ok(AssetData::Svg(_))));
        assert!(matches!(a.load(&AssetId::new("missing")), Err(AssetError::NotFound(_))));
    }
}
