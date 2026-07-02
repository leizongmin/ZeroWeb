//! # zero-ui-assets
//!
//! 资源系统（spec §8.4.1 `zero-ui-assets` / FR-016 / IF-010 `AssetProvider` /
//! §8.8 asset manifest variant resolution 测 / §8.4.1B toolbar 图标·favicon fallback·theme package）。
//!
//! 资源（图标/图片/字体/shader/locale/theme）通过 [`AssetId`] + [`AssetVariant`]（主题 × 密度）引用；
//! [`AssetProvider`] 加载（可 mock）。[`InMemoryAssets`] 提供变体清单 + **fallback 解析**：精确变体
//! 缺失时按「同密度 Any 主题 → 同主题 1x → Any 1x」回退，仍找不到才 `NotFound`。

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

/// 资源主题变体。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariantTheme {
    /// 主题无关（默认变体）。
    Any,
    Light,
    Dark,
}

/// 资源变体：主题 × 像素密度（1x/2x/3x）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetVariant {
    pub theme: VariantTheme,
    pub density: u8,
}

impl AssetVariant {
    pub const fn new(theme: VariantTheme, density: u8) -> AssetVariant {
        AssetVariant { theme, density }
    }

    /// 默认变体（Any 主题 / 1x）。
    pub const fn any() -> AssetVariant {
        AssetVariant::new(VariantTheme::Any, 1)
    }
    pub const fn light(density: u8) -> AssetVariant {
        AssetVariant::new(VariantTheme::Light, density)
    }
    pub const fn dark(density: u8) -> AssetVariant {
        AssetVariant::new(VariantTheme::Dark, density)
    }

    pub fn with_density(self, density: u8) -> AssetVariant {
        AssetVariant::new(self.theme, density)
    }
    pub fn with_theme(self, theme: VariantTheme) -> AssetVariant {
        AssetVariant::new(theme, self.density)
    }
}

impl Default for AssetVariant {
    fn default() -> AssetVariant {
        AssetVariant::any()
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

/// 资源提供者（IF-010；可被测试 mock）。
///
/// `load(id, variant)` 按 [`AssetVariant`] 解析；实现应提供 fallback（见 [`InMemoryAssets`]）。
pub trait AssetProvider {
    fn load(&self, id: &AssetId, variant: AssetVariant) -> Result<AssetData, AssetError>;
}

/// 内存资源清单（测试 / 打包资源用）。按 `(AssetId, AssetVariant)` 存储变体条目。
#[derive(Debug, Default)]
pub struct InMemoryAssets {
    entries: HashMap<(AssetId, AssetVariant), AssetData>,
}

impl InMemoryAssets {
    pub fn new() -> InMemoryAssets {
        InMemoryAssets::default()
    }

    /// 插入一个**变体特定**条目（如 dark/2x 的图标）。
    pub fn insert(&mut self, id: AssetId, variant: AssetVariant, data: AssetData) -> &mut InMemoryAssets {
        self.entries.insert((id, variant), data);
        self
    }

    /// 插入默认变体条目（Any 主题 / 1x）——即不区分主题/密度的通用资源。
    pub fn insert_default(&mut self, id: AssetId, data: AssetData) -> &mut InMemoryAssets {
        self.insert(id, AssetVariant::any(), data)
    }
}

impl AssetProvider for InMemoryAssets {
    fn load(&self, id: &AssetId, variant: AssetVariant) -> Result<AssetData, AssetError> {
        // §8.8 variant resolution + fallback：
        //   1. 精确 (theme, density)
        //   2. 同密度 Any 主题
        //   3. 同主题 1x（基础密度）
        //   4. Any 1x（完全默认）
        for cand in candidate_variants(variant) {
            if let Some(data) = self.entries.get(&(id.clone(), cand)) {
                return Ok(data.clone());
            }
        }
        Err(AssetError::NotFound(format!(
            "{} ({:?}, {}x)",
            id.0, variant.theme, variant.density
        )))
    }
}

/// fallback 候选变体序列（精确 → Any 同密度 → 同主题 1x → Any 1x）。去重后返回。
fn candidate_variants(requested: AssetVariant) -> Vec<AssetVariant> {
    let raw = [
        requested,
        AssetVariant::new(VariantTheme::Any, requested.density),
        AssetVariant::new(requested.theme, 1),
        AssetVariant::any(),
    ];
    let mut out: Vec<AssetVariant> = Vec::new();
    for v in raw {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_asset_inserted_default() {
        let mut a = InMemoryAssets::new();
        a.insert_default(AssetId::new("icon.back"), AssetData::Svg("<svg/>".into()));
        // 任意变体都命中默认条目（fallback 到 Any/1x）。
        assert!(matches!(
            a.load(&AssetId::new("icon.back"), AssetVariant::any()),
            Ok(AssetData::Svg(_))
        ));
        assert!(matches!(
            a.load(&AssetId::new("icon.back"), AssetVariant::dark(2)),
            Ok(AssetData::Svg(_))
        ));
        // 真正缺失 → NotFound。
        assert!(matches!(
            a.load(&AssetId::new("missing"), AssetVariant::any()),
            Err(AssetError::NotFound(_))
        ));
    }

    #[test]
    fn exact_variant_preferred_over_fallback() {
        // §8.8 variant resolution：精确变体优先于 fallback。
        let mut a = InMemoryAssets::new();
        a.insert_default(AssetId::new("icon.menu"), AssetData::Svg("<svg default/>".into()));
        a.insert(
            AssetId::new("icon.menu"),
            AssetVariant::dark(2),
            AssetData::Svg("<svg dark-2x/>".into()),
        );
        // 请求 dark/2x → 精确命中。
        let got = a.load(&AssetId::new("icon.menu"), AssetVariant::dark(2)).unwrap();
        match got {
            AssetData::Svg(s) => assert!(s.contains("dark-2x"), "exact dark/2x preferred, got {s}"),
            _ => panic!("expected Svg"),
        }
        // 请求 light/2x → 无精确，fallback 到 Any/1x 默认。
        let got = a.load(&AssetId::new("icon.menu"), AssetVariant::light(2)).unwrap();
        match got {
            AssetData::Svg(s) => assert!(s.contains("default"), "fallback to default, got {s}"),
            _ => panic!("expected Svg"),
        }
    }

    #[test]
    fn fallback_theme_agnostic_at_requested_density() {
        // 有 Any/2x 通用条目：请求 dark/2x 应回退到 Any/2x（同密度），而非 Any/1x。
        let mut a = InMemoryAssets::new();
        a.insert(
            AssetId::new("icon.star"),
            AssetVariant::new(VariantTheme::Any, 2),
            AssetData::Svg("<svg any-2x/>".into()),
        );
        let got = a.load(&AssetId::new("icon.star"), AssetVariant::dark(2)).unwrap();
        match got {
            AssetData::Svg(s) => assert!(s.contains("any-2x"), "fallback to Any at same density, got {s}"),
            _ => panic!("expected Svg"),
        }
    }

    #[test]
    fn fallback_to_base_density_when_missing_high_density() {
        // 有 dark/1x，请求 dark/3x → 同主题 1x 回退。
        let mut a = InMemoryAssets::new();
        a.insert(
            AssetId::new("icon.close"),
            AssetVariant::dark(1),
            AssetData::Png(vec![1]),
        );
        let got = a.load(&AssetId::new("icon.close"), AssetVariant::dark(3)).unwrap();
        assert!(matches!(got, AssetData::Png(_)));
    }

    #[test]
    fn variant_constructors() {
        assert_eq!(AssetVariant::any(), AssetVariant::new(VariantTheme::Any, 1));
        assert_eq!(AssetVariant::light(2), AssetVariant::new(VariantTheme::Light, 2));
        assert_eq!(AssetVariant::dark(3), AssetVariant::new(VariantTheme::Dark, 3));
        assert_eq!(
            AssetVariant::any().with_density(2),
            AssetVariant::new(VariantTheme::Any, 2)
        );
        assert_eq!(
            AssetVariant::light(1).with_theme(VariantTheme::Dark),
            AssetVariant::dark(1)
        );
    }

    #[test]
    fn candidate_variants_dedup() {
        // 请求 Any/1 → 所有候选都退化成 Any/1，去重后只剩一个。
        let c = candidate_variants(AssetVariant::any());
        assert_eq!(c, vec![AssetVariant::any()]);
        // 请求 Dark/2 → 4 个候选。
        let c = candidate_variants(AssetVariant::dark(2));
        assert_eq!(
            c,
            vec![
                AssetVariant::dark(2),
                AssetVariant::new(VariantTheme::Any, 2),
                AssetVariant::dark(1),
                AssetVariant::any(),
            ]
        );
    }
}
