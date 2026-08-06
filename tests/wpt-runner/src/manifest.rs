//! WPT 测试清单解析 — 解析 MANIFEST.json 格式，支持 reftest 元数据和 fuzzy 注解。
//!
//! WPT 使用 MANIFEST.json 记录所有测试文件及其类型（manual、reftest、testharness 等）。
//! 本模块负责解析该格式，提取可运行的测试条目，包括 reftest 的参考文件和 fuzzy 容差。

use serde::Deserialize;
use std::path::Path;

/// WPT 测试类型。
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestType {
    /// testharness.js 自动化测试。
    Testharness,
    /// 手动测试。
    Manual,
    /// 参考测试（对比渲染结果）。
    Reftest,
    /// 交互测试。
    Wdspec,
    /// 性能测试。
    Performance,
}

/// 单个测试条目 — 清单中的一条测试记录。
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestItem {
    /// 测试文件路径（相对于 WPT 根目录）。
    pub path: String,
    /// 测试类型。
    #[serde(rename = "type")]
    pub test_type: Option<String>,
}

/// WPT MANIFEST.json 的顶层结构。
///
/// 实际 WPT manifest 使用按类型分组的格式，例如：
/// ```json
/// {
///   "items": {
///     "testharness": { "path/to/test.html": [...] },
///     "reftest": { ... }
///   }
/// }
/// ```
/// 本模块提供简化解析，同时支持扁平列表格式用于内置测试。
#[derive(Debug, Clone, Deserialize)]
pub struct WptManifest {
    /// 按类型分组的测试条目（标准 WPT 格式）。
    #[serde(default)]
    pub items: Option<serde_json::Value>,
    /// 扁平测试列表（用于内置测试的简化格式）。
    #[serde(default)]
    pub tests: Option<Vec<ManifestItem>>,
}

/// 解析后的测试条目 — 统一表示不同来源的测试。
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParsedTestEntry {
    /// 测试文件路径或标识符。
    pub path: String,
    /// 测试类型标签。
    pub test_type: String,
}

// ── Reftest 专用结构 ──

/// WPT fuzzy 容差元数据 — per-test 或 per-reference 的容差声明。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FuzzyMeta {
    /// 允许的最大单像素颜色差异。
    pub max_diff: Option<u32>,
    /// 允许的最大差异像素总数。
    pub total_pixels: Option<u32>,
}

impl FuzzyMeta {
    /// 空的 fuzzy 元数据（无容差覆盖）。
    pub fn none() -> Self {
        Self {
            max_diff: None,
            total_pixels: None,
        }
    }

    /// 从 MANIFEST.json 的 fuzzy 字段解析。
    pub fn from_json_value(value: &serde_json::Value) -> Self {
        let mut meta = Self::none();
        if let Some(obj) = value.as_object() {
            if let Some(max_diff) = obj.get("maxDiff").and_then(|v| v.as_u64()) {
                meta.max_diff = Some(max_diff as u32);
            }
            if let Some(total_pixels) = obj.get("totalPixels").and_then(|v| v.as_u64()) {
                meta.total_pixels = Some(total_pixels as u32);
            }
        }
        meta
    }
}

/// Reftest 参考关系 — 测试文件与参考文件之间的比较关系。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReftestReference {
    /// 参考文件路径（相对于 WPT 根目录）。
    pub ref_path: String,
    /// 比较类型："==" (match) 或 "!=" (mismatch)。
    pub relation: String,
}

impl ReftestReference {
    /// 是否为 match 比较（应该相同）。
    #[allow(dead_code)]
    pub fn is_match(&self) -> bool {
        self.relation == "==" || self.relation == "="
    }
}

/// Reftest 清单条目 — 从 MANIFEST.json 解析的完整 reftest 信息。
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReftestManifestEntry {
    /// 测试文件路径（相对于 WPT 根目录）。
    pub test_path: String,
    /// 参考文件列表及其比较关系。
    pub references: Vec<ReftestReference>,
    /// Fuzzy 容差元数据（测试文件的容差）。
    pub fuzzy: FuzzyMeta,
}

/// 从文件路径解析 WPT MANIFEST.json。
pub fn parse_manifest_file(path: &Path) -> Result<Vec<ParsedTestEntry>, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read manifest file {}: {e}", path.display()))?;
    parse_manifest_json(&content)
}

/// 从 JSON 字符串解析 WPT manifest。
pub fn parse_manifest_json(json: &str) -> Result<Vec<ParsedTestEntry>, String> {
    let manifest: WptManifest =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse manifest JSON: {e}"))?;

    let mut entries = Vec::new();

    // 优先使用扁平格式
    if let Some(tests) = &manifest.tests {
        for item in tests {
            entries.push(ParsedTestEntry {
                path: item.path.clone(),
                test_type: item.test_type.clone().unwrap_or_else(|| "testharness".to_string()),
            });
        }
    }

    // 解析标准 WPT items 格式
    if let Some(items) = &manifest.items
        && let Some(obj) = items.as_object()
    {
        for (type_name, type_entries) in obj {
            if let Some(path_map) = type_entries.as_object() {
                for (path, _details) in path_map {
                    entries.push(ParsedTestEntry {
                        path: path.clone(),
                        test_type: type_name.clone(),
                    });
                }
            }
        }
    }

    Ok(entries)
}

/// 从 MANIFEST.json 中提取 reftest 条目（带参考文件和 fuzzy 元数据）。
#[allow(dead_code)]
///
/// WPT MANIFEST.json 中 reftest 的格式：
/// ```json
/// {
///   "items": {
///     "reftest": {
///       "css/CSS2/colors/color-001.html": [
///         [{
///           "path": "css/CSS2/colors/color-001.html",
///           "references": [[["css/CSS2/colors/color-001-ref.html", "=="]]],
///           "fuzzy": [["css/CSS2/colors/color-001.html", {"maxDiff": 5, "totalPixels": 10}]]
///         }]
///       ]
///     }
///   }
/// }
/// ```
pub fn parse_reftest_entries(json: &str) -> Result<Vec<ReftestManifestEntry>, String> {
    let manifest: WptManifest =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse manifest JSON: {e}"))?;

    let mut entries = Vec::new();

    if let Some(items) = &manifest.items
        && let Some(obj) = items.as_object()
        && let Some(reftest_items) = obj.get("reftest")
        && let Some(path_map) = reftest_items.as_object()
    {
        for (test_path, variants) in path_map {
            if let Some(variants_arr) = variants.as_array() {
                // 取第一个 variant（大多数 reftest 只有一个 variant）
                if let Some(first_variant) = variants_arr.first() {
                    // variant 可能是数组的数组
                    let variant_items = if first_variant.is_array() {
                        first_variant.as_array().unwrap().clone()
                    } else {
                        vec![first_variant.clone()]
                    };

                    for variant_item in &variant_items {
                        let mut references = Vec::new();
                        let mut fuzzy = FuzzyMeta::none();

                        if let Some(obj) = variant_item.as_object() {
                            // 解析 references
                            if let Some(refs) = obj.get("references")
                                && let Some(refs_arr) = refs.as_array()
                            {
                                for ref_group in refs_arr {
                                    // 每个 ref_group 是 [[path, relation], ...]
                                    if let Some(group_arr) = ref_group.as_array() {
                                        for ref_pair in group_arr {
                                            if let Some(pair) = ref_pair.as_array()
                                                && pair.len() == 2
                                                && let (Some(path), Some(relation)) =
                                                    (pair[0].as_str(), pair[1].as_str())
                                            {
                                                references.push(ReftestReference {
                                                    ref_path: path.to_string(),
                                                    relation: relation.to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }

                            // 解析 fuzzy
                            if let Some(fuzzy_val) = obj.get("fuzzy") {
                                fuzzy = parse_fuzzy_meta(fuzzy_val, test_path);
                            }
                        }

                        if !references.is_empty() {
                            entries.push(ReftestManifestEntry {
                                test_path: test_path.clone(),
                                references,
                                fuzzy,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(entries)
}

/// 解析 MANIFEST.json 中的 fuzzy 字段。
///
/// fuzzy 字段可能是：
/// - 对象格式：`{"maxDiff": 5, "totalPixels": 10}`
/// - 数组格式：`[["path/to/test.html", {"maxDiff": 5, "totalPixels": 10}]]`
#[allow(dead_code)]
fn parse_fuzzy_meta(fuzzy_val: &serde_json::Value, test_path: &str) -> FuzzyMeta {
    // 直接对象格式
    if fuzzy_val.is_object() {
        return FuzzyMeta::from_json_value(fuzzy_val);
    }

    // 数组格式：查找匹配 test_path 的条目
    if let Some(arr) = fuzzy_val.as_array() {
        for item in arr {
            if let Some(pair) = item.as_array()
                && pair.len() == 2
                && let Some(path) = pair[0].as_str()
                && (path == test_path || path == "*" || path.ends_with("/*"))
            {
                return FuzzyMeta::from_json_value(&pair[1]);
            }
        }
        // 未找到匹配路径，使用第一个条目作为回退
        if let Some(first) = arr.first()
            && let Some(pair) = first.as_array()
            && pair.len() == 2
        {
            return FuzzyMeta::from_json_value(&pair[1]);
        }
    }

    FuzzyMeta::none()
}

/// 按类型过滤测试条目。
#[allow(dead_code)]
pub fn filter_by_type(entries: &[ParsedTestEntry], test_type: &str) -> Vec<ParsedTestEntry> {
    entries.iter().filter(|e| e.test_type == test_type).cloned().collect()
}

/// 按路径前缀过滤测试条目。
#[allow(dead_code)]
pub fn filter_by_path_prefix(entries: &[ParsedTestEntry], prefix: &str) -> Vec<ParsedTestEntry> {
    entries.iter().filter(|e| e.path.starts_with(prefix)).cloned().collect()
}

/// 从 HTML 内容中提取 reftest 链接（`<link rel="match">` 或 `<link rel="mismatch">`）。
///
/// WPT reftest 文件通过 `<link>` 标签指定参考文件：
/// ```html
/// <link rel="match" href="green-ref.html">
/// <link rel="mismatch" href="red-ref.html">
/// ```
#[allow(dead_code)]
pub fn extract_reftest_links(html: &str) -> Vec<ReftestReference> {
    let mut references = Vec::new();
    let mut pos = 0;

    while pos < html.len() {
        // 跳过 HTML 注释：注释内的 `<link>` 不应参与解析（2026-08-07，
        // 例：css-transform-inherit-rotate.html 中被注释的 match link）
        if let Some(comment_start) = html[pos..].find("<!--") {
            let abs_cs = pos + comment_start;
            if let Some(link_start) = html[pos..].find("<link")
                && abs_cs < pos + link_start
            {
                pos = match html[abs_cs..].find("-->") {
                    Some(e) => abs_cs + e + 3,
                    None => html.len(),
                };
                continue;
            }
        }
        // 查找 <link 标签
        if let Some(link_start) = html[pos..].find("<link") {
            let abs_start = pos + link_start;
            // 找到标签结束
            if let Some(tag_end) = html[abs_start..].find('>') {
                let tag_content = &html[abs_start..abs_start + tag_end];
                let tag_lower = tag_content.to_lowercase();

                // 检查 rel 属性
                let is_match = contains_attr_value(&tag_lower, "rel", "match");
                let is_mismatch = contains_attr_value(&tag_lower, "rel", "mismatch");

                if is_match || is_mismatch {
                    // 提取 href 属性值
                    if let Some(href) = extract_attr_value(tag_content, "href") {
                        references.push(ReftestReference {
                            ref_path: href,
                            relation: if is_match { "==".to_string() } else { "!=".to_string() },
                        });
                    }
                }

                pos = abs_start + tag_end + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    references
}

/// 检查 HTML 标签属性是否包含指定值。
#[allow(dead_code)]
fn contains_attr_value(tag: &str, attr: &str, value: &str) -> bool {
    // 简单匹配：查找 attr="value" 或 attr='value' 模式
    let patterns = [
        format!("{attr}=\"{value}\""),
        format!("{attr}='{value}'"),
        format!("{attr}={value}"),
    ];
    for pattern in &patterns {
        if tag.contains(pattern) {
            return true;
        }
    }
    // 处理空格分隔的多值属性（如 rel="help match"）
    if let Some(attr_start) = tag.find(attr) {
        let after_attr = &tag[attr_start + attr.len()..];
        let after_attr = after_attr.trim_start();
        if let Some(after_eq) = after_attr.strip_prefix('=') {
            let after_eq = after_eq.trim_start();
            let quote_char = if after_eq.starts_with('"') {
                '"'
            } else if after_eq.starts_with('\'') {
                '\''
            } else {
                '\0'
            };
            if quote_char != '\0'
                && let Some(end) = after_eq[1..].find(quote_char)
            {
                let attr_val = &after_eq[1..1 + end];
                // 检查空格分隔的值列表
                for v in attr_val.split_whitespace() {
                    if v == value {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// 从 HTML 标签中提取属性值。
#[allow(dead_code)]
fn extract_attr_value(tag: &str, attr: &str) -> Option<String> {
    let attr_pattern = format!("{attr}=");
    let idx = tag.find(&attr_pattern)?;
    let after = &tag[idx + attr_pattern.len()..];
    let after = after.trim_start();

    let (value, _) = if let Some(rest) = after.strip_prefix('"') {
        let end = rest.find('"')?;
        (rest[..end].to_string(), end + 2)
    } else if let Some(rest) = after.strip_prefix('\'') {
        let end = rest.find('\'')?;
        (rest[..end].to_string(), end + 2)
    } else {
        let end = after
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(after.len());
        (after[..end].to_string(), end)
    };

    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flat_manifest() {
        let json = r#"{
            "tests": [
                { "path": "html/test1.html", "type": "testharness" },
                { "path": "html/test2.html", "type": "manual" }
            ]
        }"#;
        let entries = parse_manifest_json(json).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, "html/test1.html");
        assert_eq!(entries[0].test_type, "testharness");
        assert_eq!(entries[1].test_type, "manual");
    }

    #[test]
    fn test_parse_standard_manifest() {
        let json = r#"{
            "items": {
                "testharness": {
                    "html/dom/test1.html": [[]],
                    "html/dom/test2.html": [[]]
                },
                "reftest": {
                    "css/color/green-ref.html": [[]]
                }
            }
        }"#;
        let entries = parse_manifest_json(json).unwrap();
        assert_eq!(entries.len(), 3);

        let th = filter_by_type(&entries, "testharness");
        assert_eq!(th.len(), 2);

        let reftest = filter_by_type(&entries, "reftest");
        assert_eq!(reftest.len(), 1);
    }

    #[test]
    fn test_parse_empty_manifest() {
        let json = r#"{}"#;
        let entries = parse_manifest_json(json).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_invalid_json() {
        let result = parse_manifest_json("{invalid}");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_by_path_prefix() {
        let entries = vec![
            ParsedTestEntry {
                path: "html/dom/test1.html".to_string(),
                test_type: "testharness".to_string(),
            },
            ParsedTestEntry {
                path: "css/color/test.html".to_string(),
                test_type: "reftest".to_string(),
            },
            ParsedTestEntry {
                path: "html/layout/test2.html".to_string(),
                test_type: "testharness".to_string(),
            },
        ];
        let filtered = filter_by_path_prefix(&entries, "html/");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_manifest_file_not_found() {
        let result = parse_manifest_file(Path::new("/nonexistent/MANIFEST.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_manifest_without_type() {
        let json = r#"{
            "tests": [
                { "path": "html/no-type.html" }
            ]
        }"#;
        let entries = parse_manifest_json(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].test_type, "testharness");
    }

    // ── Reftest 解析测试 ──

    #[test]
    fn test_parse_reftest_entries_basic() {
        let json = r#"{
            "items": {
                "reftest": {
                    "css/CSS2/colors/color-001.html": [
                        [{
                            "path": "css/CSS2/colors/color-001.html",
                            "references": [[["css/CSS2/colors/color-001-ref.html", "=="]]]
                        }]
                    ]
                }
            }
        }"#;
        let entries = parse_reftest_entries(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].test_path, "css/CSS2/colors/color-001.html");
        assert_eq!(entries[0].references.len(), 1);
        assert_eq!(entries[0].references[0].ref_path, "css/CSS2/colors/color-001-ref.html");
        assert!(entries[0].references[0].is_match());
    }

    #[test]
    fn test_parse_reftest_entries_with_fuzzy() {
        let json = r#"{
            "items": {
                "reftest": {
                    "css/text/test.html": [
                        [{
                            "path": "css/text/test.html",
                            "references": [[["css/text/ref.html", "=="]]],
                            "fuzzy": [["css/text/test.html", {"maxDiff": 10, "totalPixels": 100}]]
                        }]
                    ]
                }
            }
        }"#;
        let entries = parse_reftest_entries(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].fuzzy.max_diff, Some(10));
        assert_eq!(entries[0].fuzzy.total_pixels, Some(100));
    }

    #[test]
    fn test_parse_reftest_entries_multiple_refs() {
        let json = r#"{
            "items": {
                "reftest": {
                    "css/test/multi.html": [
                        [{
                            "path": "css/test/multi.html",
                            "references": [
                                [["css/test/ref1.html", "=="]],
                                [["css/test/ref2.html", "!="]]
                            ]
                        }]
                    ]
                }
            }
        }"#;
        let entries = parse_reftest_entries(json).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].references.len(), 2);
        assert!(entries[0].references[0].is_match());
        assert!(!entries[0].references[1].is_match());
    }

    #[test]
    fn test_parse_reftest_entries_no_reftest_section() {
        let json = r#"{
            "items": {
                "testharness": {
                    "html/test.html": [[]]
                }
            }
        }"#;
        let entries = parse_reftest_entries(json).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_fuzzy_meta_from_json_value() {
        let val = serde_json::json!({"maxDiff": 5, "totalPixels": 20});
        let meta = FuzzyMeta::from_json_value(&val);
        assert_eq!(meta.max_diff, Some(5));
        assert_eq!(meta.total_pixels, Some(20));
    }

    #[test]
    fn test_fuzzy_meta_partial() {
        let val = serde_json::json!({"maxDiff": 3});
        let meta = FuzzyMeta::from_json_value(&val);
        assert_eq!(meta.max_diff, Some(3));
        assert_eq!(meta.total_pixels, None);
    }

    // ── HTML 链接提取测试 ──

    #[test]
    fn test_extract_reftest_links_match() {
        let html = r#"<!DOCTYPE html><html><head><link rel="match" href="green-ref.html"></head><body><div style="background:green"></div></body></html>"#;
        let refs = extract_reftest_links(html);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].ref_path, "green-ref.html");
        assert!(refs[0].is_match());
    }

    #[test]
    fn test_extract_reftest_links_mismatch() {
        let html = r#"<html><head><link rel="mismatch" href='red-ref.html'></head></html>"#;
        let refs = extract_reftest_links(html);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].ref_path, "red-ref.html");
        assert!(!refs[0].is_match());
    }

    #[test]
    fn test_extract_reftest_links_multiple() {
        let html = r#"<head><link rel="match" href="ref1.html"><link rel="mismatch" href="ref2.html"></head>"#;
        let refs = extract_reftest_links(html);
        assert_eq!(refs.len(), 2);
        assert!(refs[0].is_match());
        assert!(!refs[1].is_match());
    }

    #[test]
    fn test_extract_reftest_links_no_links() {
        let html = r#"<html><body><div>Hello</div></body></html>"#;
        let refs = extract_reftest_links(html);
        assert!(refs.is_empty());
    }

    #[test]
    fn test_extract_reftest_links_non_reftest_link() {
        let html = r#"<head><link rel="stylesheet" href="style.css"><link rel="help" href="spec.html"></head>"#;
        let refs = extract_reftest_links(html);
        assert!(refs.is_empty());
    }
}
