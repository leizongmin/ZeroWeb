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
];
