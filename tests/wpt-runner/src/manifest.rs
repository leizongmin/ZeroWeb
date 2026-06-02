//! WPT 测试清单解析 — 解析 MANIFEST.json 格式。
//!
//! WPT 使用 MANIFEST.json 记录所有测试文件及其类型（manual、reftest、testharness 等）。
//! 本模块负责解析该格式，提取可运行的测试条目。

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
}
