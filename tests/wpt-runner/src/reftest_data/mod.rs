//! WPT Reftest 测试数据 — 按类别分模块管理
//!
//! 包含 CSS 2.1 核心及各 CSS 模块的 inline reftest 用例。
//! 每个子模块对应一个 CSS 类别（css21、css-flexbox、css-grid 等）。

mod css21;
mod css_box;
mod css_display;
mod css_flexbox;
mod css_float;
mod css_grid;
mod css_multicol;
mod css_position;
mod css_table;
mod css_text;

use crate::manifest::ReftestReference;
use crate::reftest::{ReftestCase, ReftestCategory, ReftestConfig};

/// 内联 reftest 条目定义。
pub(super) struct InlineReftestDef {
    pub id: &'static str,
    pub category: ReftestCategory,
    pub test_html: &'static str,
    pub ref_html: &'static str,
    pub is_match: bool,
}

/// 获取所有内联 reftest 用例（来自全部子模块）。
pub fn css21_reftest_cases() -> Vec<ReftestCase> {
    all_reftests()
        .iter()
        .map(|def| ReftestCase {
            id: def.id.to_string(),
            test_html: def.test_html.to_string(),
            ref_html: def.ref_html.to_string(),
            css: String::new(),
            is_match: def.is_match,
        })
        .collect()
}

/// 获取每个 reftest 的推荐配置（包含分类容差）。
pub fn css21_reftest_configs() -> Vec<ReftestConfig> {
    all_reftests()
        .iter()
        .map(|def| ReftestConfig::for_category(def.category))
        .collect()
}

/// 从内联数据生成 reftest 清单条目。
#[allow(dead_code)]
pub fn css21_reftest_manifest_entries() -> Vec<crate::manifest::ReftestManifestEntry> {
    all_reftests()
        .iter()
        .map(|def| crate::manifest::ReftestManifestEntry {
            test_path: format!("css/CSS2/inline/{}.html", def.id),
            references: vec![ReftestReference {
                ref_path: format!("css/CSS2/inline/{}-ref.html", def.id),
                relation: if def.is_match {
                    "==".to_string()
                } else {
                    "!=".to_string()
                },
            }],
            fuzzy: crate::manifest::FuzzyMeta::none(),
        })
        .collect()
}

/// 合并所有子模块的 reftest 数据。
fn all_reftests() -> Vec<&'static InlineReftestDef> {
    let mut all = Vec::new();
    all.extend(css21::reftests().iter());
    all.extend(css_flexbox::reftests().iter());
    all.extend(css_grid::reftests().iter());
    all.extend(css_position::reftests().iter());
    all.extend(css_display::reftests().iter());
    all.extend(css_box::reftests().iter());
    all.extend(css_multicol::reftests().iter());
    all.extend(css_float::reftests().iter());
    all.extend(css_table::reftests().iter());
    all.extend(css_text::reftests().iter());
    all
}
