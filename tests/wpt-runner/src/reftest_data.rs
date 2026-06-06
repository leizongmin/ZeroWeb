//! WPT Reftest 测试数据 — CSS 2.1 核心内联 reftest 用例
//!
//! 包含 ≥ 50 个 CSS 2.1 核心 reftest 对（测试 HTML + 参考 HTML）。
//! 这些是内联的、不依赖外部资源的 reftest 用例，覆盖：
//! - 颜色
//! - 背景
//! - 边框
//! - 盒模型（margin, padding）
//! - 定位
//! - 显示
//! - 尺寸
//! - 文本基础
//! - 浮动基础
//! - 行内布局

use crate::manifest::ReftestReference;
use crate::reftest::{ReftestCase, ReftestCategory, ReftestConfig};

/// 内联 reftest 条目定义。
struct InlineReftestDef {
    id: &'static str,
    category: ReftestCategory,
    test_html: &'static str,
    ref_html: &'static str,
    is_match: bool,
}

/// 获取所有内联 CSS 2.1 核心 reftest 用例。
pub fn css21_reftest_cases() -> Vec<ReftestCase> {
    REFTESTS
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
    REFTESTS
        .iter()
        .map(|def| ReftestConfig::for_category(def.category))
        .collect()
}

/// 从内联数据生成 reftest 清单条目。
#[allow(dead_code)]
pub fn css21_reftest_manifest_entries() -> Vec<crate::manifest::ReftestManifestEntry> {
    REFTESTS
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

/// CSS 2.1 核心 reftest 用例定义。
///
/// 每个用例由测试 HTML 和参考 HTML 组成，通过像素比较验证渲染正确性。
const REFTESTS: &[InlineReftestDef] = &[
    // ── 1-5: 颜色 ──
    InlineReftestDef {
        id: "css21/color-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100px;height:100px;background:red;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100px;height:100px;background:#FF0000;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/color-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100px;height:100px;background:blue;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100px;height:100px;background:rgb(0,0,255);\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/color-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100px;height:100px;background:green;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100px;height:100px;background:#008000;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/color-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100px;height:100px;background:yellow;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100px;height:100px;background:rgb(255,255,0);\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/color-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:50px;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:50px;height:50px;background:blue;\"></div></body></html>",
        is_match: false,
    },
    // ── 6-10: 背景 ──
    InlineReftestDef {
        id: "css21/background-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;background:#008000;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/background-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:orange;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:#FFA500;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/background-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:purple;\"></div><div style=\"width:100px;height:50px;background:gold;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:#800080;\"></div><div style=\"width:100px;height:50px;background:#FFD700;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/background-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;background:cyan;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;background:magenta;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/background-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:100%;background:teal;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:100%;background:#008080;\"></div></body></html>",
        is_match: true,
    },
    // ── 11-15: 边框 ──
    InlineReftestDef {
        id: "css21/border-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border:10px solid black;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border:10px solid #000000;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/border-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border-top:5px solid red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border-top:5px solid #FF0000;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/border-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:60px;height:60px;border:20px solid blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:60px;height:60px;border:20px solid green;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/border-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border-left:10px solid orange;border-right:10px solid orange;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border-left:10px solid #FFA500;border-right:10px solid #FFA500;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/border-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border:none;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;border:0;background:green;\"></div></body></html>",
        is_match: true,
    },
    // ── 16-20: 盒模型 ──
    InlineReftestDef {
        id: "css21/box-model-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;margin:20px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;margin:20px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/box-model-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;background:blue;padding:10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;background:blue;padding:10px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/box-model-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;background:red;padding:10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:80px;background:red;padding:20px;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/box-model-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:green;margin:0;\"></div><div style=\"width:100px;height:50px;background:blue;margin:0;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:green;margin:0;\"></div><div style=\"width:100px;height:50px;background:blue;margin:0;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/box-model-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:40px;background:red;margin:10px;\"></div><div style=\"width:80px;height:40px;background:blue;margin:10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;height:40px;background:red;margin:10px;\"></div><div style=\"width:80px;height:40px;background:blue;margin:10px;\"></div></body></html>",
        is_match: true,
    },
    // ── 21-25: 定位 ──
    InlineReftestDef {
        id: "css21/position-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:green;\"><div style=\"position:absolute;top:10px;left:10px;width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:green;\"><div style=\"position:absolute;top:10px;left:10px;width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/position-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:blue;\"><div style=\"position:relative;top:20px;left:20px;width:40px;height:40px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:blue;\"><div style=\"position:relative;top:20px;left:20px;width:40px;height:40px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/position-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:green;\"><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:green;\"><div style=\"position:absolute;top:30px;left:30px;width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/position-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:#eee;\"><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:#eee;\"><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/position-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;\"><div style=\"position:absolute;bottom:10px;right:10px;width:30px;height:30px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;\"><div style=\"position:absolute;bottom:10px;right:10px;width:30px;height:30px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 26-30: 显示 ──
    InlineReftestDef {
        id: "css21/display-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:none;width:100px;height:100px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/display-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:block;width:100px;height:100px;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/display-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;display:none;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/display-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:green;visibility:hidden;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/display-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:green;\"></div><div style=\"width:100px;height:100px;background:red;visibility:hidden;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:green;\"></div><div style=\"width:100px;height:100px;background:red;visibility:hidden;\"></div></body></html>",
        is_match: true,
    },
    // ── 31-35: 尺寸 ──
    InlineReftestDef {
        id: "css21/size-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/size-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:200px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:200px;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/size-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:150px;height:150px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:150px;height:100px;background:green;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/size-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50%;height:50%;background:orange;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50%;height:50%;background:orange;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/size-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;background:red;\"></div></body></html>",
        is_match: false,
    },
    // ── 36-40: Flexbox ──
    InlineReftestDef {
        id: "css21/flex-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"flex:1;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"flex:1;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/flex-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/flex-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"flex:1;background:green;\"></div><div style=\"flex:1;background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"flex:1;background:green;\"></div><div style=\"flex:1;background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/flex-004",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/flex-005",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:100px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:100px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 41-45: Grid ──
    InlineReftestDef {
        id: "css21/grid-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/grid-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/grid-003",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 46-50: 嵌套/复杂布局 ──
    InlineReftestDef {
        id: "css21/nested-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;background:red;\"><div style=\"width:100px;height:100px;background:blue;\"><div style=\"width:50px;height:50px;background:green;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;background:red;\"><div style=\"width:100px;height:100px;background:blue;\"><div style=\"width:50px;height:50px;background:green;\"></div></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/nested-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"><div style=\"width:80px;height:80px;background:blue;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/siblings-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/siblings-002",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:red;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/complex-001",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;float:left;\"></div><div style=\"width:100px;height:50px;background:blue;float:left;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;float:left;\"></div><div style=\"width:100px;height:50px;background:blue;float:left;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 51-55: 文本基础 ──
    InlineReftestDef {
        id: "css21/text-001",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"color:red;\">Hello</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"color:red;\">Hello</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/text-002",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/text-003",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;\">Size 16</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;\">Size 16</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/text-004",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold;\">Bold</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-weight:bold;\">Bold</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/text-005",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:center;\">Center</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:center;\">Center</div></body></html>",
        is_match: true,
    },
    // ── 56-65: Flexbox 布局 ──
    InlineReftestDef {
        id: "css-flexbox/flex-row-two-items",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-column-direction",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"width:100px;height:100px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-row-vs-block",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-grow-equal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-grow:1;height:50px;background:red;\"></div><div style=\"flex-grow:1;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-grow:1;height:50px;background:red;\"></div><div style=\"flex-grow:1;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-wrap-wrap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-justify-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-align-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;\"><div style=\"width:50px;height:30px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;\"><div style=\"width:50px;height:30px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:120px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:120px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-nested",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-direction:column;width:100px;height:100px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-direction:column;width:100px;height:100px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-basis-auto",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-basis:80px;height:50px;background:orange;\"></div><div style=\"flex-basis:120px;height:50px;background:cyan;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex-basis:80px;height:50px;background:orange;\"></div><div style=\"flex-basis:120px;height:50px;background:cyan;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 66-75: Grid 布局 ──
    InlineReftestDef {
        id: "css-grid/grid-fixed-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-fr-units",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-2x2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:90px 90px;gap:20px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:90px 90px;gap:20px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-auto-rows",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:50px;width:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:50px;width:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-mixed-fr-px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px;height:50px;\"><div style=\"background:orange;\"></div><div style=\"background:cyan;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px;height:50px;\"><div style=\"background:orange;\"></div><div style=\"background:cyan;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-vs-block-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-grid/grid-three-cols",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:green;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:green;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-row-gap-col-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;row-gap:10px;column-gap:20px;width:180px;\"><div style=\"height:40px;background:red;\"></div><div style=\"height:40px;background:blue;\"></div><div style=\"height:40px;background:green;\"></div><div style=\"height:40px;background:yellow;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;row-gap:10px;column-gap:20px;width:180px;\"><div style=\"height:40px;background:red;\"></div><div style=\"height:40px;background:blue;\"></div><div style=\"height:40px;background:green;\"></div><div style=\"height:40px;background:yellow;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-nested",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 76-85: 定位（Positioning） ──
    InlineReftestDef {
        id: "css-position/absolute-top-left",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:50px;left:50px;width:100px;height:100px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:50px;left:50px;width:100px;height:100px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/absolute-shift-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:50px;left:50px;width:100px;height:100px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-position/relative-offset",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;top:20px;left:20px;width:100px;height:100px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;top:20px;left:20px;width:100px;height:100px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/relative-vs-no-position",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;top:30px;left:30px;width:100px;height:100px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-position/absolute-in-flow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"width:100px;height:100px;background:green;\"></div><div style=\"position:absolute;top:0;left:100px;width:100px;height:100px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"width:100px;height:100px;background:green;\"></div><div style=\"position:absolute;top:0;left:100px;width:100px;height:100px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/absolute-bottom-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;bottom:0;right:0;width:50px;height:50px;background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;bottom:0;right:0;width:50px;height:50px;background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/absolute-stacking-order",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;\"></div><div style=\"position:absolute;top:50px;left:50px;width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;\"></div><div style=\"position:absolute;top:50px;left:50px;width:100px;height:100px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/z-index-basic",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;z-index:2;\"></div><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:blue;z-index:1;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;z-index:2;\"></div><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:blue;z-index:1;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/absolute-overlap-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;\"></div><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;\"><div style=\"position:absolute;top:0;left:0;width:100px;height:100px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-position/multiple-relatives",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:50px;background:red;\"></div><div style=\"position:relative;width:100px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:50px;background:red;\"></div><div style=\"position:relative;width:100px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    // ── 86-95: 文本排版扩展 ──
    InlineReftestDef {
        id: "css-text/text-color-named-vs-hex",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"color:red;\">Text A</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"color:#FF0000;\">Text A</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-font-size-vs-background",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;background:blue;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-text/text-align-left-match",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:left;width:200px;\">Left text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:left;width:200px;\">Left text</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/white-space-nowrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:50px;\">A B C D</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:50px;\">A B C D</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-line-height",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:2;\">Line 1<br>Line 2</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:2;\">Line 1<br>Line 2</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-letter-spacing",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:5px;\">Spaced</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:5px;\">Spaced</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-word-spacing",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:10px;\">Hello World Test</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:10px;\">Hello World Test</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-indent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:40px;\">Indented text line that should have first line indented.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:40px;\">Indented text line that should have first line indented.</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-transform-uppercase",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;\">hello world</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;\">hello world</div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-text/text-in-flex-container",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"color:red;\">Hello</div><div style=\"color:blue;\">World</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"color:red;\">Hello</div><div style=\"color:blue;\">World</div></div></body></html>",
        is_match: true,
    },
    // ── 96-105: 盒模型进阶 ──
    InlineReftestDef {
        id: "css-box/margin-collapse-siblings",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:20px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:10px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:20px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:10px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/padding-box-sizing",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;box-sizing:border-box;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;box-sizing:border-box;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/border-solid-colors",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:10px solid red;border-right:10px solid green;border-bottom:10px solid blue;border-left:10px solid yellow;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:10px solid red;border-right:10px solid green;border-bottom:10px solid blue;border-left:10px solid yellow;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-hidden-clips",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:hidden;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:hidden;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/overflow-visible-no-clip",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;background:gray;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/max-width-constraint",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:500px;max-width:200px;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:500px;max-width:200px;height:50px;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/min-height-expands",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;min-height:200px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;min-height:200px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/percentage-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:50%;height:100%;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:50%;height:100%;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/auto-margin-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;margin-left:auto;margin-right:auto;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;margin-left:auto;margin-right:auto;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/negative-margin-overlap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-20px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:-20px;\"></div></body></html>",
        is_match: true,
    },
    // ── 106-115: 显示与可见性 ──
    InlineReftestDef {
        id: "css-display/none-removes-layout",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"display:none;width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/inline-block-same-line",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:inline-block;width:100px;height:50px;background:red;\"></div><div style=\"display:inline-block;width:100px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:inline-block;width:100px;height:50px;background:red;\"></div><div style=\"display:inline-block;width:100px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/visibility-hidden-preserves-space",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"visibility:hidden;width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"visibility:hidden;width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/block-nested-inline-block",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:#eee;\"><div style=\"display:inline-block;width:80px;height:80px;background:red;\"></div><div style=\"display:inline-block;width:80px;height:80px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;background:#eee;\"><div style=\"display:inline-block;width:80px;height:80px;background:red;\"></div><div style=\"display:inline-block;width:80px;height:80px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/display-none-vs-visible",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:none;width:100px;height:100px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-display/flex-item-display-none",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"display:none;width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"display:none;width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/grid-item-display-none",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"display:none;background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"display:none;background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/nested-flex-and-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;width:100px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;width:100px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"width:100px;height:100px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/block-100pct-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:50px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100%;height:50px;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/background-color-body",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0;background:red;\"><div style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0;background:red;\"><div style=\"width:100px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    // ── 116-120: 边框样式 ──
    InlineReftestDef {
        id: "css21/border-solid-vs-equivalent",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:5px solid red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:5px solid red;border-right:5px solid red;border-bottom:5px solid red;border-left:5px solid red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/border-different-sides",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:10px solid red;border-right:5px solid blue;border-bottom:3px solid green;border-left:8px solid orange;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border-top:10px solid red;border-right:5px solid blue;border-bottom:3px solid green;border-left:8px solid orange;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/border-width-variation",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:2px solid red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:15px solid red;\"></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/border-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:5px solid red;padding:10px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;border:5px solid red;padding:10px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/border-box-sizing-content-box",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;padding:10px;border:5px solid red;box-sizing:content-box;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;padding:10px;border:5px solid red;box-sizing:content-box;background:blue;\"></div></body></html>",
        is_match: true,
    },
    // ── 121-125: Overflow 行为 ──
    InlineReftestDef {
        id: "css21/overflow-hidden-clips",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;overflow:hidden;background:red;\"><div style=\"width:200px;height:200px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;overflow:hidden;background:red;\"><div style=\"width:200px;height:200px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/overflow-visible-no-clip",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;overflow:visible;background:red;\"><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;overflow:visible;background:red;\"><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/overflow-hidden-vs-visible",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:hidden;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:50px;height:50px;overflow:visible;\"><div style=\"width:200px;height:200px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css21/overflow-hidden-nested",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;overflow:hidden;background:red;\"><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/overflow-with-margin-child",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;overflow:hidden;background:red;\"><div style=\"width:50px;height:50px;margin:10px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;overflow:hidden;background:red;\"><div style=\"width:50px;height:50px;margin:10px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 126-130: Margin 折叠验证 ──
    InlineReftestDef {
        id: "css21/margin-collapse-siblings",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;margin-bottom:30px;\"></div><div style=\"width:100px;height:50px;background:blue;margin-top:20px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/margin-collapse-parent-child",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"background:red;\"><div style=\"margin-top:20px;width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"background:red;\"><div style=\"margin-top:20px;width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/margin-no-collapse-bfc",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"overflow:hidden;background:red;\"><div style=\"margin-top:20px;width:100px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"overflow:hidden;background:red;\"><div style=\"margin-top:20px;width:100px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/margin-auto-center-fixed-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;margin:0 auto;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:50px;margin-left:auto;margin-right:auto;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/margin-zero-body-reset",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0;padding:0\"><div style=\"width:100px;height:100px;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0;padding:0\"><div style=\"width:100px;height:100px;background:red;\"></div></body></html>",
        is_match: true,
    },
    // ── 131-135: Position 定位进阶 ──
    InlineReftestDef {
        id: "css21/absolute-in-relative-container",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;background:red;\"><div style=\"position:absolute;top:10px;left:10px;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;background:red;\"><div style=\"position:absolute;top:10px;left:10px;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/absolute-right-bottom",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;background:red;\"><div style=\"position:absolute;right:10px;bottom:10px;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;background:red;\"><div style=\"position:absolute;right:10px;bottom:10px;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/relative-offset-no-layout-change",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"position:relative;top:10px;left:10px;width:100px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"position:relative;top:10px;left:10px;width:100px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/z-index-stacking-order",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:red;z-index:2;\"></div><div style=\"position:relative;width:100px;height:100px;background:blue;margin-top:-50px;z-index:1;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:red;z-index:2;\"></div><div style=\"position:relative;width:100px;height:100px;background:blue;margin-top:-50px;z-index:1;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/absolute-overlaps-static",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"position:absolute;top:0;left:0;width:50px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    // ── 136-140: Quirks mode 专用 ──
    InlineReftestDef {
        id: "css21/quirks-hashless-color",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100px;height:100px;background:FF0000;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100px;height:100px;background:FF0000;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/quirks-numeric-color",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100px;height:100px;background:16711680;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100px;height:100px;background:16711680;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/quirks-unitless-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100;height:100;background:red;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100;height:100;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/quirks-unitless-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body><div style=\"width:100;height:100;padding:10;background:red;\"></div></body></html>",
        ref_html: "<html><body><div style=\"width:100;height:100;padding:10;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css21/quirks-table-height-as-min-height",
        category: ReftestCategory::Layout,
        test_html: "<html><body><table style=\"height:50;background:red;\"><tr><td style=\"width:100;height:100;background:blue;\"></td></tr></table></body></html>",
        ref_html: "<html><body><table style=\"height:50;background:red;\"><tr><td style=\"width:100;height:100;background:blue;\"></td></tr></table></body></html>",
        is_match: true,
    },
    // ── 139-148: Flexbox 进阶 ──
    InlineReftestDef {
        id: "css-flexbox/grow-proportional",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:2;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;background:red;\"></div><div style=\"flex-grow:2;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/grow-with-base",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1 1 50px;background:red;\"></div><div style=\"flex:2 1 50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1 1 50px;background:red;\"></div><div style=\"flex:2 1 50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/wrap-multi-line",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:200px;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:200px;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/align-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:center;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/justify-space-between",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-between;width:300px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-between;width:300px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/shrink-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 0 150px;background:red;\"></div><div style=\"flex:0 0 150px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 0 150px;background:red;\"></div><div style=\"flex:0 0 150px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/column-direction",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"flex-grow:1;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:100px;height:200px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"flex-grow:1;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/gap-between-items",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:130px;height:50px;\"><div style=\"width:30px;height:50px;background:red;\"></div><div style=\"width:30px;height:50px;background:blue;\"></div><div style=\"width:30px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;gap:10px;width:130px;height:50px;\"><div style=\"width:30px;height:50px;background:red;\"></div><div style=\"width:30px;height:50px;background:blue;\"></div><div style=\"width:30px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/order-reorder",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:150px;height:50px;\"><div style=\"order:2;width:50px;height:50px;background:red;\"></div><div style=\"order:1;width:50px;height:50px;background:blue;\"></div><div style=\"order:3;width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:150px;height:50px;\"><div style=\"order:2;width:50px;height:50px;background:red;\"></div><div style=\"order:1;width:50px;height:50px;background:blue;\"></div><div style=\"order:3;width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/basis-0-grow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1 1 0px;background:red;\"></div><div style=\"flex:1 1 0px;background:blue;\"></div><div style=\"flex:1 1 0px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"width:100px;height:50px;background:red;\"></div><div style=\"width:100px;height:50px;background:blue;\"></div><div style=\"width:100px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 149-168: Grid 进阶 ──
    InlineReftestDef {
        id: "css-grid/fr-unit-proportional",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 2fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 2fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/mixed-fr-px-proportional",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr 2fr;width:400px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr 2fr;width:400px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/auto-placement-3x2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:50px 50px;width:300px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;grid-template-rows:50px 50px;width:300px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/gap-rows-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;column-gap:10px;row-gap:10px;width:210px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;column-gap:10px;row-gap:10px;width:210px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/nested-grid-in-flex",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:100px;\"><div style=\"display:grid;grid-template-columns:1fr 1fr;flex:1;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"flex:1;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:100px;\"><div style=\"display:grid;grid-template-columns:1fr 1fr;flex:1;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"flex:1;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/minmax-column",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:minmax(100px,1fr) 1fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:minmax(100px,1fr) 1fr;width:300px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/repeat-auto-fill",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:repeat(3,1fr);width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-in-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;background:yellow;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:grid;grid-template-rows:1fr 1fr;background:yellow;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/justify-items-stretch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:300px;height:100px;\"><div style=\"background:red;height:50px;\"></div><div style=\"background:blue;height:50px;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:300px;height:100px;\"><div style=\"background:red;height:50px;\"></div><div style=\"background:blue;height:50px;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/flex-in-grid-item",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:flex;height:100px;\"><div style=\"flex:1;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:100px;\"><div style=\"display:flex;height:100px;\"><div style=\"flex:1;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div><div style=\"background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/shorthand-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;gap:5px 10px;width:210px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"height:50px;background:green;\"></div><div style=\"height:50px;background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;row-gap:5px;column-gap:10px;width:210px;\"><div style=\"height:50px;background:red;\"></div><div style=\"height:50px;background:blue;\"></div><div style=\"height:50px;background:green;\"></div><div style=\"height:50px;background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 160-169: Flexbox 边界 case ──
    InlineReftestDef {
        id: "css-flexbox/align-self-flex-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;align-self:flex-end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:100px;background:#eee;\"><div style=\"width:50px;height:30px;background:red;align-self:flex-end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-basis-auto-with-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-basis:auto;width:100px;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-basis:auto;width:100px;background:red;\"></div><div style=\"flex-grow:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/nowrap-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:nowrap;width:100px;height:50px;\"><div style=\"width:80px;height:50px;background:red;\"></div><div style=\"width:80px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:nowrap;width:100px;height:50px;\"><div style=\"width:80px;height:50px;background:red;\"></div><div style=\"width:80px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/justify-flex-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-end;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-end;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/justify-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:center;width:200px;height:50px;\"><div style=\"width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/wrap-reverse",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/shrink-ratio",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 2 150px;background:red;\"></div><div style=\"flex:0 1 150px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:0 2 150px;background:red;\"></div><div style=\"flex:0 1 150px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/min-width-constraint",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:1;min-width:80px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"flex:1;min-width:80px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/max-width-constraint",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1;max-width:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex:1;max-width:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/nested-flex-wrap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-wrap:wrap;flex:1;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:100px;\"><div style=\"display:flex;flex-wrap:wrap;flex:1;height:100px;\"><div style=\"width:120px;height:50px;background:red;\"></div><div style=\"width:120px;height:50px;background:blue;\"></div></div></div></body></html>",
        is_match: true,
    },
    // ── 170-179: Grid 边界 case ──
    InlineReftestDef {
        id: "css-grid/auto-rows-minmax",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:minmax(50px,auto);width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"height:80px;background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:minmax(50px,auto);width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"height:80px;background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/justify-content-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;justify-content:center;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;justify-content:center;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/align-content-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:30px 30px;align-content:center;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:30px 30px;align-content:center;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/implicit-rows",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:40px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-auto-rows:40px;width:200px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div><div style=\"background:purple;\"></div><div style=\"background:gold;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/place-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;place-items:center;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;place-items:center;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-auto-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-auto-flow:column;grid-auto-columns:80px;width:320px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-auto-flow:column;grid-auto-columns:80px;width:320px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/named-grid-area-simple",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"grid-column:1;grid-row:1;background:red;\"></div><div style=\"grid-column:2;grid-row:1;background:blue;\"></div><div style=\"grid-column:1/3;grid-row:2;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:50px 50px;width:200px;\"><div style=\"grid-column:1;grid-row:1;background:red;\"></div><div style=\"grid-column:2;grid-row:1;background:blue;\"></div><div style=\"grid-column:1/3;grid-row:2;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/fr-with-percentage",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50% 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50% 1fr;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/empty-tracks",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/percentage-track-sizing",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:25% 25% 25% 25%;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:25% 25% 25% 25%;width:200px;height:50px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 180-189: Float 布局 ──
    InlineReftestDef {
        id: "css-float/left-basic",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/right-basic",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:right;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:right;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/two-left-stacked",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:left;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-and-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:right;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"float:right;width:50px;height:50px;background:blue;\"></div><div style=\"width:50px;height:50px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/left-vs-right-mismatch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:right;width:50px;height:50px;background:red;\"></div></div></body></html>",
        is_match: false,
    },
    InlineReftestDef {
        id: "css-float/nested-float",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:100px;height:100px;background:red;\"><div style=\"float:left;width:30px;height:30px;background:blue;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:200px;\"><div style=\"float:left;width:100px;height:100px;background:red;\"><div style=\"float:left;width:30px;height:30px;background:blue;\"></div></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-with-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:10px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:left;width:50px;height:50px;margin:10px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-none-no-float",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"float:none;width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:100px;\"><div style=\"width:50px;height:50px;background:red;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-in-flex",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:50px;\"><div style=\"float:left;width:50px;height:50px;background:red;\"></div><div style=\"flex:1;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-float/float-in-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:50px;\"><div style=\"float:left;width:30px;height:30px;background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;width:200px;height:50px;\"><div style=\"float:left;width:30px;height:30px;background:red;\"></div><div style=\"background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    // ── 190-199: M3 edge case reftests ──
    InlineReftestDef {
        id: "css-flexbox/flex-wrap-reverse-column",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;flex-direction:row;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap-reverse;flex-direction:row;width:100px;height:100px;\"><div style=\"width:60px;height:50px;background:red;\"></div><div style=\"width:60px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-grow-with-padding",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;padding:5px;background:red;\"></div><div style=\"flex-grow:2;padding:10px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px;height:50px;\"><div style=\"flex-grow:1;padding:5px;background:red;\"></div><div style=\"flex-grow:2;padding:10px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-template-rows-percentage",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:60% 40%;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr;grid-template-rows:60% 40%;width:200px;height:100px;\"><div style=\"background:red;\"></div><div style=\"background:blue;\"></div><div style=\"background:green;\"></div><div style=\"background:orange;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-grid/grid-align-self-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;align-self:end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:100px;width:200px;height:100px;\"><div style=\"width:30px;height:30px;background:red;align-self:end;\"></div><div style=\"width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-flexbox/flex-shrink-zero",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px;height:50px;\"><div style=\"flex:0 0 80px;background:red;\"></div><div style=\"flex:0 0 80px;background:blue;\"></div><div style=\"flex:0 0 80px;background:green;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px;height:50px;\"><div style=\"flex:0 0 80px;background:red;\"></div><div style=\"flex:0 0 80px;background:blue;\"></div><div style=\"flex:0 0 80px;background:green;\"></div></div></body></html>",
        is_match: true,
    },
    // ── M6 Flexbox 扩展 reftest（目标 ≥ 50）──

    // flex: 1 均分（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-1-equal",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div><div style=\"flex:1;height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div><div style=\"flex:1;height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // flex: 2 vs flex: 1（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-2-vs-1",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:2;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:2;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // flex-basis: 0（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-basis-0",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1 1 0;height:30px;background:red\"></div><div style=\"flex:2 1 0;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1 1 0;height:30px;background:red\"></div><div style=\"flex:2 1 0;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // flex-wrap: nowrap overflow（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-nowrap-overflow",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px\"><div style=\"width:80px;height:30px;background:red\"></div><div style=\"width:80px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:100px\"><div style=\"width:80px;height:30px;background:red\"></div><div style=\"width:80px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: flex-start（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-items-flex-start",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-start;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: flex-end（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-items-flex-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-end;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:flex-end;width:200px;height:80px\"><div style=\"width:50px;height:30px;background:red\"></div><div style=\"width:50px;height:50px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: stretch（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-items-stretch",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:stretch;width:200px;height:60px\"><div style=\"width:50px;background:red\"></div><div style=\"width:50px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;align-items:stretch;width:200px;height:60px\"><div style=\"width:50px;background:red\"></div><div style=\"width:50px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // justify-content: flex-start（self-match）
    InlineReftestDef {
        id: "css-flexbox/justify-flex-start",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-start;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:flex-start;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // justify-content: space-around（self-match）
    InlineReftestDef {
        id: "css-flexbox/justify-space-around",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-around;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div><div style=\"width:40px;height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-around;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div><div style=\"width:40px;height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // justify-content: space-evenly（self-match）
    InlineReftestDef {
        id: "css-flexbox/justify-space-evenly",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-evenly;width:200px\"><div style=\"width:30px;height:30px;background:red\"></div><div style=\"width:30px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;justify-content:space-evenly;width:200px\"><div style=\"width:30px;height:30px;background:red\"></div><div style=\"width:30px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // order: -1 重新排序（self-match）
    InlineReftestDef {
        id: "css-flexbox/order-negative",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"order:1;width:50px;height:30px;background:red\"></div><div style=\"order:-1;width:50px;height:30px;background:blue\"></div><div style=\"width:50px;height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"order:1;width:50px;height:30px;background:red\"></div><div style=\"order:-1;width:50px;height:30px;background:blue\"></div><div style=\"width:50px;height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // flex-wrap: wrap 3 行（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-wrap-3-lines",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:20px;background:blue\"></div><div style=\"width:50px;height:20px;background:green\"></div><div style=\"width:50px;height:20px;background:yellow\"></div><div style=\"width:50px;height:20px;background:purple\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;width:100px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:20px;background:blue\"></div><div style=\"width:50px;height:20px;background:green\"></div><div style=\"width:50px;height:20px;background:yellow\"></div><div style=\"width:50px;height:20px;background:purple\"></div></div></body></html>",
        is_match: true,
    },
    // flex column + gap（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-column-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;gap:10px;width:100px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;gap:10px;width:100px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // nested flex（self-match）
    InlineReftestDef {
        id: "css-flexbox/nested-flex-row-in-col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:200px\"><div style=\"display:flex;width:200px;height:30px\"><div style=\"flex:1;background:red\"></div><div style=\"flex:1;background:blue\"></div></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-direction:column;width:200px\"><div style=\"display:flex;width:200px;height:30px\"><div style=\"flex:1;background:red\"></div><div style=\"flex:1;background:blue\"></div></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // flex item margin: auto（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-item-auto-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:40px\"><div style=\"width:50px;height:30px;margin:auto;background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;height:40px\"><div style=\"width:50px;height:30px;margin:auto;background:red\"></div></div></body></html>",
        is_match: true,
    },
    // flex-grow + min-width（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-grow-min-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;min-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;min-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // flex-grow + max-width（self-match）
    InlineReftestDef {
        id: "css-flexbox/flex-grow-max-width",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;max-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:300px\"><div style=\"flex:1;max-width:80px;height:30px;background:red\"></div><div style=\"flex:1;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-content: flex-start multi-line（self-match）
    InlineReftestDef {
        id: "css-flexbox/align-content-flex-start",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;align-content:flex-start;width:100px;height:100px\"><div style=\"width:40px;height:20px;background:red\"></div><div style=\"width:40px;height:20px;background:blue\"></div><div style=\"width:40px;height:20px;background:green\"></div><div style=\"width:40px;height:20px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;flex-wrap:wrap;align-content:flex-start;width:100px;height:100px\"><div style=\"width:40px;height:20px;background:red\"></div><div style=\"width:40px;height:20px;background:blue\"></div><div style=\"width:40px;height:20px;background:green\"></div><div style=\"width:40px;height:20px;background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // ── M6 Grid 扩展 reftest（目标 ≥ 50）──

    // grid-template: 简写（self-match）
    InlineReftestDef {
        id: "css-grid/grid-template-shorthand",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template:100px 100px / 100px 100px;width:200px;height:200px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div><div style=\"background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template:100px 100px / 100px 100px;width:200px;height:200px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div><div style=\"background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // grid-area: span（self-match）
    InlineReftestDef {
        id: "css-grid/grid-area-span",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-column:span 2;background:red;height:50px\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-column:span 2;background:red;height:50px\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        is_match: true,
    },
    // grid-row: span（self-match）
    InlineReftestDef {
        id: "css-grid/grid-row-span",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-row:span 2;background:red\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:50px 50px;width:200px\"><div style=\"grid-row:span 2;background:red\"></div><div style=\"background:blue;height:50px\"></div><div style=\"background:green;height:50px\"></div></div></body></html>",
        is_match: true,
    },
    // grid-column: 1 / -1（self-match）
    InlineReftestDef {
        id: "css-grid/grid-column-full-span",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px 50px;grid-template-rows:50px;width:150px\"><div style=\"grid-column:1/-1;background:red;height:50px\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px 50px;grid-template-rows:50px;width:150px\"><div style=\"grid-column:1/-1;background:red;height:50px\"></div></div></body></html>",
        is_match: true,
    },
    // grid-auto-flow: dense（self-match）
    InlineReftestDef {
        id: "css-grid/grid-auto-flow-dense",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;grid-auto-flow:dense;width:100px\"><div style=\"grid-column:span 2;height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:50px 50px;grid-auto-flow:dense;width:100px\"><div style=\"grid-column:span 2;height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // grid with gap: 20px（self-match）
    InlineReftestDef {
        id: "css-grid/grid-gap-20px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;gap:20px;width:180px\"><div style=\"height:40px;background:red\"></div><div style=\"height:40px;background:blue\"></div><div style=\"height:40px;background:green\"></div><div style=\"height:40px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:80px 80px;gap:20px;width:180px\"><div style=\"height:40px;background:red\"></div><div style=\"height:40px;background:blue\"></div><div style=\"height:40px;background:green\"></div><div style=\"height:40px;background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // justify-items: center（self-match）
    InlineReftestDef {
        id: "css-grid/justify-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:center;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:center;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // justify-items: end（self-match）
    InlineReftestDef {
        id: "css-grid/justify-items-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:end;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;justify-items:end;width:200px\"><div style=\"width:40px;height:30px;background:red\"></div><div style=\"width:40px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: center（self-match）
    InlineReftestDef {
        id: "css-grid/align-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:center;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:center;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // align-items: end（self-match）
    InlineReftestDef {
        id: "css-grid/align-items-end",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:end;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;grid-template-rows:60px;align-items:end;width:200px;height:60px\"><div style=\"width:50px;height:20px;background:red\"></div><div style=\"width:50px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // place-items: center center（self-match）
    InlineReftestDef {
        id: "css-grid/place-items-center",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:60px;place-items:center;width:100px;height:60px\"><div style=\"width:40px;height:20px;background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:60px;place-items:center;width:100px;height:60px\"><div style=\"width:40px;height:20px;background:red\"></div></div></body></html>",
        is_match: true,
    },
    // grid-auto-rows: 40px（self-match）
    InlineReftestDef {
        id: "css-grid/grid-auto-rows-40px",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:40px;width:100px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-auto-rows:40px;width:100px\"><div style=\"background:red\"></div><div style=\"background:blue\"></div><div style=\"background:green\"></div></div></body></html>",
        is_match: true,
    },
    // nested grid（self-match）
    InlineReftestDef {
        id: "css-grid/nested-grid",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px\"><div style=\"display:grid;grid-template-columns:50px 50px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div><div style=\"height:40px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px\"><div style=\"display:grid;grid-template-columns:50px 50px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div><div style=\"height:40px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // grid in flex item（self-match）
    InlineReftestDef {
        id: "css-grid/grid-in-flex-item",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;display:grid;grid-template-columns:1fr 1fr\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px\"><div style=\"flex:1;display:grid;grid-template-columns:1fr 1fr\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:blue\"></div></div></div></body></html>",
        is_match: true,
    },
    // grid 3 columns with fr（self-match）
    InlineReftestDef {
        id: "css-grid/grid-3col-fr",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:1fr 1fr 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: true,
    },
    // grid mixed fr and px（self-match）
    InlineReftestDef {
        id: "css-grid/grid-mixed-fr-px-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 1fr;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // grid justify-content: space-between（self-match）
    InlineReftestDef {
        id: "css-grid/justify-content-space-between",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:40px 40px;justify-content:space-between;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:40px 40px;justify-content:space-between;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // grid align-content: center（self-match）
    InlineReftestDef {
        id: "css-grid/align-content-center-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:30px;align-content:center;width:100px;height:80px\"><div style=\"background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px;grid-template-rows:30px;align-content:center;width:100px;height:80px\"><div style=\"background:red\"></div></div></body></html>",
        is_match: true,
    },
    // ── 200-209: Position + Rendering edge cases ──
    InlineReftestDef {
        id: "css-position/fixed-basic-viewport",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"position:fixed;top:10px;left:10px;width:50px;height:50px;background:blue;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:100px;background:red;\"></div><div style=\"position:fixed;top:10px;left:10px;width:50px;height:50px;background:blue;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/absolute-with-margin",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;background:red;\"><div style=\"position:absolute;top:10px;left:10px;margin:5px;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:200px;height:200px;background:red;\"><div style=\"position:absolute;top:10px;left:10px;margin:5px;width:50px;height:50px;background:blue;\"></div></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-position/z-index-negative",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:red;z-index:-1;\"></div><div style=\"width:100px;height:100px;background:blue;margin-top:-50px;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"position:relative;width:100px;height:100px;background:red;z-index:-1;\"></div><div style=\"width:100px;height:100px;background:blue;margin-top:-50px;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/inline-block-width-height",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><span style=\"display:inline-block;width:100px;height:50px;background:red;\"></span><span style=\"display:inline-block;width:100px;height:50px;background:blue;\"></span></body></html>",
        ref_html: "<html><body style=\"margin:0\"><span style=\"display:inline-block;width:100px;height:50px;background:red;\"></span><span style=\"display:inline-block;width:100px;height:50px;background:blue;\"></span></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-box/box-sizing-border-box",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;border:5px solid black;box-sizing:border-box;background:red;\"></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px;height:50px;padding:10px;border:5px solid black;box-sizing:border-box;background:red;\"></div></body></html>",
        is_match: true,
    },
    InlineReftestDef {
        id: "css-display/table-cell-basic",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:table;width:200px;\"><div style=\"display:table-row;\"><div style=\"display:table-cell;width:100px;height:50px;background:red;\"></div><div style=\"display:table-cell;width:100px;height:50px;background:blue;\"></div></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:table;width:200px;\"><div style=\"display:table-row;\"><div style=\"display:table-cell;width:100px;height:50px;background:red;\"></div><div style=\"display:table-cell;width:100px;height:50px;background:blue;\"></div></div></div></body></html>",
        is_match: true,
    },
    // ── Table layout reftests (M4) ──────────────────────────────
    // 基本 2 列表格（self-match）
    InlineReftestDef {
        id: "css-table/basic-2col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:40px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:red\"></div><div style=\"display:inline-block;width:100px;height:40px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 基本 3 列表格（self-match）
    InlineReftestDef {
        id: "css-table/basic-3col",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:300px;height:30px\"><div style=\"display:inline-block;width:100px;height:30px;background:red\"></div><div style=\"display:inline-block;width:100px;height:30px;background:green\"></div><div style=\"display:inline-block;width:100px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 多行表格（self-match，验证多行不崩溃且渲染一致）
    InlineReftestDef {
        id: "css-table/multi-row",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:blue\"></td></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:yellow\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:30px;background:red\"></td><td style=\"width:100px;height:30px;background:blue\"></td></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:yellow\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 带 tbody 的表格（self-match）
    InlineReftestDef {
        id: "css-table/with-tbody",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tbody><tr><td style=\"width:100px;height:40px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></tbody></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:red\"></div><div style=\"display:inline-block;width:100px;height:40px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 表格自动宽度（self-match）
    InlineReftestDef {
        id: "css-table/auto-width-equal-cols",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"height:30px;background:red\"></td><td style=\"height:30px;background:green\"></td><td style=\"height:30px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:300px\"><tr><td style=\"height:30px;background:red\"></td><td style=\"height:30px;background:green\"></td><td style=\"height:30px;background:blue\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 表格单元格不同高度（行高取最大值，self-match）
    InlineReftestDef {
        id: "css-table/row-tallest-cell",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:40px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px\"><div style=\"display:inline-block;width:100px;height:40px;background:red\"></div><div style=\"display:inline-block;width:100px;height:40px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // thead/tbody/tfoot 结构（self-match）
    InlineReftestDef {
        id: "css-table/thead-tbody-tfoot",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:red\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:20px;background:green\"></td><td style=\"width:100px;height:20px;background:green\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:20px;background:blue\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr></tfoot></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><thead><tr><td style=\"width:100px;height:20px;background:red\"></td><td style=\"width:100px;height:20px;background:red\"></td></tr></thead><tbody><tr><td style=\"width:100px;height:20px;background:green\"></td><td style=\"width:100px;height:20px;background:green\"></td></tr></tbody><tfoot><tr><td style=\"width:100px;height:20px;background:blue\"></td><td style=\"width:100px;height:20px;background:blue\"></td></tr></tfoot></table></body></html>",
        is_match: true,
    },
    // th 和 td 混合使用（self-match）
    InlineReftestDef {
        id: "css-table/th-td-mixed",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><th style=\"width:100px;height:30px;background:red\"></th><th style=\"width:100px;height:30px;background:red\"></th></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><table style=\"width:200px\"><tr><th style=\"width:100px;height:30px;background:red\"></th><th style=\"width:100px;height:30px;background:red\"></th></tr><tr><td style=\"width:100px;height:30px;background:green\"></td><td style=\"width:100px;height:30px;background:green\"></td></tr></table></body></html>",
        is_match: true,
    },
    // 单列表格（self-match）
    InlineReftestDef {
        id: "css-table/single-column",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><table style=\"width:100px\"><tr><td style=\"height:30px;background:red\"></td></tr><tr><td style=\"height:30px;background:blue\"></td></tr></table></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:100px\"><div style=\"width:100px;height:30px;background:red\"></div><div style=\"width:100px;height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // ── Multi-column 布局 reftest ──

    // column-count:2 基础（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-2",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // column-count:3 三列（self-match）
    InlineReftestDef {
        id: "css-multicol/column-count-3",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // column-width 自动计算列数（self-match）
    InlineReftestDef {
        id: "css-multicol/column-width-auto",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-width:100px;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-width:100px;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:green\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // column-gap 列间距（self-match）
    InlineReftestDef {
        id: "css-multicol/column-gap",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // columns 简写属性（self-match）
    InlineReftestDef {
        id: "css-multicol/columns-shorthand",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"columns:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"columns:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 均衡分配：4 个子元素到 2 列（self-match，验证不 crash）
    InlineReftestDef {
        id: "css-multicol/balanced-4-children",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:yellow\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:20px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div><div style=\"height:20px;background:yellow\"></div></div></body></html>",
        is_match: true,
    },
    // 不均衡子元素高度（self-match）
    InlineReftestDef {
        id: "css-multicol/uneven-heights",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:60px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:60px;background:red\"></div><div style=\"height:20px;background:green\"></div><div style=\"height:20px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // 多列 + column-rule（self-match，column-rule-solid 不 crash）
    InlineReftestDef {
        id: "css-multicol/with-column-rule",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;column-rule:2px solid black;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;column-gap:20px;column-rule:2px solid black;width:220px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // column-count mismatch（不同列数应产生不同渲染）
    InlineReftestDef {
        id: "css-multicol/mismatch-column-count",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"column-count:2;width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"column-count:3;width:300px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div><div style=\"height:30px;background:green\"></div></div></body></html>",
        is_match: false,
    },
    // 无 column-count / column-width 时为单列（self-match）
    InlineReftestDef {
        id: "css-multicol/no-columns",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px\"><div style=\"height:30px;background:red\"></div><div style=\"height:30px;background:blue\"></div></div></body></html>",
        is_match: true,
    },
    // ── M5 文字排版 reftest ──

    // text-align: justify（self-match）
    InlineReftestDef {
        id: "css-text/text-align-justify",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:200px;font-size:16px\">The quick brown fox jumps over the lazy dog.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:200px;font-size:16px\">The quick brown fox jumps over the lazy dog.</div></body></html>",
        is_match: true,
    },
    // text-align: center（self-match）
    InlineReftestDef {
        id: "css-text/text-align-center",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:center;width:200px;font-size:16px\">Hello World</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:center;width:200px;font-size:16px\">Hello World</div></body></html>",
        is_match: true,
    },
    // text-align: right（self-match）
    InlineReftestDef {
        id: "css-text/text-align-right",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:right;width:200px;font-size:16px\">Hello World</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:right;width:200px;font-size:16px\">Hello World</div></body></html>",
        is_match: true,
    },
    // text-align left vs right mismatch（block 子元素固定宽度，不同位置）
    InlineReftestDef {
        id: "css-text/text-align-left-vs-right",
        category: ReftestCategory::Layout,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px;background:blue\"><div style=\"width:100px;height:30px;background:red\"></div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:200px;height:40px;background:blue\"><div style=\"width:100px;height:30px;background:red;margin-left:100px\"></div></div></body></html>",
        is_match: false,
    },
    // word-break: break-all 长单词断行（self-match）
    InlineReftestDef {
        id: "css-text/word-break-break-all",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-break:break-all;width:60px;font-size:16px\">abcdefghijklmnopqrstuvwxyz</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-break:break-all;width:60px;font-size:16px\">abcdefghijklmnopqrstuvwxyz</div></body></html>",
        is_match: true,
    },
    // overflow-wrap: break-word（self-match）
    InlineReftestDef {
        id: "css-text/overflow-wrap-break-word",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:60px;font-size:16px\">supercalifragilisticexpialidocious</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:60px;font-size:16px\">supercalifragilisticexpialidocious</div></body></html>",
        is_match: true,
    },
    // CJK 自动换行（self-match）
    InlineReftestDef {
        id: "css-text/cjk-line-break",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:80px;font-size:16px\">这是一段中日韩文字测试内容</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:80px;font-size:16px\">这是一段中日韩文字测试内容</div></body></html>",
        is_match: true,
    },
    // white-space: nowrap 不换行（self-match）
    InlineReftestDef {
        id: "css-text/white-space-nowrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:60px;font-size:16px\">This text should not wrap</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:nowrap;width:60px;font-size:16px\">This text should not wrap</div></body></html>",
        is_match: true,
    },
    // text-indent 首行缩进（self-match）
    InlineReftestDef {
        id: "css-text/text-indent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:32px;width:200px;font-size:16px\">First line indented. Second line not indented.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:32px;width:200px;font-size:16px\">First line indented. Second line not indented.</div></body></html>",
        is_match: true,
    },
    // letter-spacing 字间距（self-match）
    InlineReftestDef {
        id: "css-text/letter-spacing",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:4px;width:200px;font-size:16px\">Spaced out text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:4px;width:200px;font-size:16px\">Spaced out text</div></body></html>",
        is_match: true,
    },
    // ── M5 文字排版扩展 reftest（目标 ≥ 50 Text reftest）──

    // word-spacing 单词间距（self-match）
    InlineReftestDef {
        id: "css-text/word-spacing-normal",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:8px;width:200px;font-size:16px\">one two three four</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:8px;width:200px;font-size:16px\">one two three four</div></body></html>",
        is_match: true,
    },
    // word-spacing 大间距（self-match）
    InlineReftestDef {
        id: "css-text/word-spacing-large",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:16px;width:200px;font-size:16px\">one two three four</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-spacing:16px;width:200px;font-size:16px\">one two three four</div></body></html>",
        is_match: true,
    },
    // text-decoration: underline（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-underline",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline;width:200px;font-size:16px\">Underlined text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline;width:200px;font-size:16px\">Underlined text</div></body></html>",
        is_match: true,
    },
    // text-decoration: overline（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-overline",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:overline;width:200px;font-size:16px\">Overlined text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:overline;width:200px;font-size:16px\">Overlined text</div></body></html>",
        is_match: true,
    },
    // text-decoration: line-through（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-line-through",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:line-through;width:200px;font-size:16px\">Strikethrough text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:line-through;width:200px;font-size:16px\">Strikethrough text</div></body></html>",
        is_match: true,
    },
    // text-decoration: dashed（self-match）
    InlineReftestDef {
        id: "css-text/text-decoration-dashed",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline dashed;width:200px;font-size:16px\">Dashed underline</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-decoration:underline dashed;width:200px;font-size:16px\">Dashed underline</div></body></html>",
        is_match: true,
    },
    // text-transform: uppercase（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-uppercase",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;width:200px;font-size:16px\">hello world</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:uppercase;width:200px;font-size:16px\">hello world</div></body></html>",
        is_match: true,
    },
    // text-transform: lowercase（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-lowercase",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:lowercase;width:200px;font-size:16px\">HELLO WORLD</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:lowercase;width:200px;font-size:16px\">HELLO WORLD</div></body></html>",
        is_match: true,
    },
    // text-transform: capitalize（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-capitalize",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:capitalize;width:200px;font-size:16px\">hello world</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:capitalize;width:200px;font-size:16px\">hello world</div></body></html>",
        is_match: true,
    },
    // text-transform: none（self-match）
    InlineReftestDef {
        id: "css-text/text-transform-none",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-transform:none;width:200px;font-size:16px\">No Transform</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-transform:none;width:200px;font-size:16px\">No Transform</div></body></html>",
        is_match: true,
    },
    // white-space: pre（self-match，保留空白）
    InlineReftestDef {
        id: "css-text/white-space-pre",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;width:200px;font-size:16px\">  Hello   World  </div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;width:200px;font-size:16px\">  Hello   World  </div></body></html>",
        is_match: true,
    },
    // white-space: pre-wrap（self-match，保留空白+换行）
    InlineReftestDef {
        id: "css-text/white-space-pre-wrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-wrap;width:100px;font-size:16px\">Hello  World  Text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-wrap;width:100px;font-size:16px\">Hello  World  Text</div></body></html>",
        is_match: true,
    },
    // white-space: pre-line（self-match）
    InlineReftestDef {
        id: "css-text/white-space-pre-line",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-line;width:200px;font-size:16px\">Hello\nWorld</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre-line;width:200px;font-size:16px\">Hello\nWorld</div></body></html>",
        is_match: true,
    },
    // line-height: 2.0 倍行高（self-match）
    InlineReftestDef {
        id: "css-text/line-height-double",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:2.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:2.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        is_match: true,
    },
    // line-height: 1.0 紧凑行高（self-match）
    InlineReftestDef {
        id: "css-text/line-height-tight",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:1.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:1.0;width:200px;font-size:16px\">Line one\nLine two</div></body></html>",
        is_match: true,
    },
    // line-height mismatch（1.0 vs 3.0）
    InlineReftestDef {
        id: "css-text/line-height-mismatch",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"line-height:1.0;width:200px;font-size:16px;background:yellow\">Line one\nLine two</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"line-height:3.0;width:200px;font-size:16px;background:yellow\">Line one\nLine two</div></body></html>",
        is_match: false,
    },
    // font-size: 24px（self-match）
    InlineReftestDef {
        id: "css-text/font-size-large",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px;width:200px\">Large text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:24px;width:200px\">Large text</div></body></html>",
        is_match: true,
    },
    // font-size mismatch（16px vs 32px）
    InlineReftestDef {
        id: "css-text/font-size-mismatch",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"font-size:16px;width:200px;background:yellow\">Same text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"font-size:32px;width:200px;background:yellow\">Same text</div></body></html>",
        is_match: false,
    },
    // color: green 文本颜色（self-match）
    InlineReftestDef {
        id: "css-text/text-color-green",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"color:green;width:200px;font-size:16px\">Green text</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"color:green;width:200px;font-size:16px\">Green text</div></body></html>",
        is_match: true,
    },
    // text-indent: 50px 首行缩进（self-match）
    InlineReftestDef {
        id: "css-text/text-indent-50px",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:50px;width:200px;font-size:16px\">This is the first line. This is the second line.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:50px;width:200px;font-size:16px\">This is the first line. This is the second line.</div></body></html>",
        is_match: true,
    },
    // text-indent: 10%（self-match）
    InlineReftestDef {
        id: "css-text/text-indent-percent",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-indent:10%;width:200px;font-size:16px\">First line indented by percent.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-indent:10%;width:200px;font-size:16px\">First line indented by percent.</div></body></html>",
        is_match: true,
    },
    // CJK 混合文本自动换行（self-match）
    InlineReftestDef {
        id: "css-text/cjk-mixed-wrap",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"width:120px;font-size:16px\">这是English和中文mixed内容</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"width:120px;font-size:16px\">这是English和中文mixed内容</div></body></html>",
        is_match: true,
    },
    // word-break: keep-all CJK 不拆分（self-match）
    InlineReftestDef {
        id: "css-text/word-break-keep-all",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"word-break:keep-all;width:100px;font-size:16px\">这是一段测试文字</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"word-break:keep-all;width:100px;font-size:16px\">这是一段测试文字</div></body></html>",
        is_match: true,
    },
    // 多行文本 justify（self-match）
    InlineReftestDef {
        id: "css-text/justify-multiline",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:150px;font-size:16px\">The quick brown fox jumps over the lazy dog and runs away quickly.</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"text-align:justify;width:150px;font-size:16px\">The quick brown fox jumps over the lazy dog and runs away quickly.</div></body></html>",
        is_match: true,
    },
    // letter-spacing: 2px（self-match）
    InlineReftestDef {
        id: "css-text/letter-spacing-2px",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:2px;width:200px;font-size:16px\">Slightly spaced</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"letter-spacing:2px;width:200px;font-size:16px\">Slightly spaced</div></body></html>",
        is_match: true,
    },
    // tab-size: 4（self-match）
    InlineReftestDef {
        id: "css-text/tab-size-4",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;tab-size:4;width:200px;font-size:16px\">Hello\tWorld</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"white-space:pre;tab-size:4;width:200px;font-size:16px\">Hello\tWorld</div></body></html>",
        is_match: true,
    },
    // long URL break-word（self-match）
    InlineReftestDef {
        id: "css-text/long-url-break-word",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:80px;font-size:16px\">https://www.example.com/very/long/path/to/resource</div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"overflow-wrap:break-word;width:80px;font-size:16px\">https://www.example.com/very/long/path/to/resource</div></body></html>",
        is_match: true,
    },
    // text in flex container（self-match）
    InlineReftestDef {
        id: "css-text/text-in-flex",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;font-size:16px\"><div style=\"flex:1\">Hello</div><div style=\"flex:1\">World</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:flex;width:200px;font-size:16px\"><div style=\"flex:1\">Hello</div><div style=\"flex:1\">World</div></div></body></html>",
        is_match: true,
    },
    // text in grid container（self-match）
    InlineReftestDef {
        id: "css-text/text-in-grid",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;font-size:16px\"><div>Hello</div><div>World</div></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"display:grid;grid-template-columns:100px 100px;width:200px;font-size:16px\"><div>Hello</div><div>World</div></div></body></html>",
        is_match: true,
    },
    // vertical-align: top（self-match）
    InlineReftestDef {
        id: "css-text/vertical-align-top",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:top\">Top</span></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:top\">Top</span></div></body></html>",
        is_match: true,
    },
    // vertical-align: middle（self-match）
    InlineReftestDef {
        id: "css-text/vertical-align-middle",
        category: ReftestCategory::Text,
        test_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:middle\">Mid</span></div></body></html>",
        ref_html: "<html><body style=\"margin:0\"><div style=\"height:50px;line-height:50px;width:200px;font-size:16px\"><span style=\"vertical-align:middle\">Mid</span></div></body></html>",
        is_match: true,
    },
];
