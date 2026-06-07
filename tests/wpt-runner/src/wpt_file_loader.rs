//! 基于文件的上游 WPT reftest 加载器。
//!
//! 从 wpt-data/ 目录读取真实上游 WPT HTML 文件，
//! 解析 <link rel="match/mismatch"> 标签找到参考文件，
//! 生成可运行的 ReftestCase 列表。

use std::path::{Path, PathBuf};

use crate::manifest::extract_reftest_links;
use crate::reftest::{ReftestCase, ReftestCategory, ReftestConfig};

/// 从指定目录加载所有上游 WPT reftest。
///
/// 目录结构应为 wpt-data/css/...，其中每个 .html/.xht 文件
/// 包含 <link rel="match" href="ref.html"> 或 <link rel="mismatch"> 标签。
///
/// 跳过不在 skip list 中的文件，跳过没有 <link rel=match/mismatch> 的文件。
pub fn load_file_reftests(wpt_data_dir: &Path) -> Vec<FileReftestCase> {
    let mut cases = Vec::new();
    let mut errors = Vec::new();

    // 加载 skip list
    let skip_list = load_skip_list(wpt_data_dir);

    // 递归查找所有 .html 和 .xht 文件
    if let Ok(entries) = walk_dir(wpt_data_dir) {
        for test_path in entries {
            let ext = test_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "html" && ext != "xht" && ext != "htm" {
                continue;
            }

            // 检查 skip list
            let relative = test_path.strip_prefix(wpt_data_dir).unwrap_or(&test_path);
            let relative_str = relative.to_string_lossy().to_string();
            if should_skip(&relative_str, &skip_list) {
                continue;
            }

            // 读取测试 HTML
            let test_html = match std::fs::read_to_string(&test_path) {
                Ok(html) => html,
                Err(e) => {
                    errors.push(format!("{}: {}", relative_str, e));
                    continue;
                }
            };

            // 解析 <link rel="match/mismatch"> 标签
            let references = extract_reftest_links(&test_html);
            if references.is_empty() {
                continue; // 非 reftest 文件，跳过
            }

            // 为每个 reference 创建一个 reftest case
            for (ref_idx, reference) in references.iter().enumerate() {
                let ref_path = test_path.parent().unwrap_or(Path::new(".")).join(&reference.ref_path);

                let ref_html = match std::fs::read_to_string(&ref_path) {
                    Ok(html) => html,
                    Err(e) => {
                        errors.push(format!("{} ref {}: {}", relative_str, reference.ref_path, e));
                        continue;
                    }
                };

                let id = if references.len() == 1 {
                    relative_str.clone()
                } else {
                    format!("{}#{}", relative_str, ref_idx)
                };

                cases.push(FileReftestCase {
                    id,
                    test_html: test_html.clone(),
                    ref_html,
                    is_match: reference.is_match(),
                    category: ReftestCategory::from_path(&relative_str),
                });
            }
        }
    }

    if !errors.is_empty() {
        eprintln!("Warnings during file reftest loading:");
        for err in &errors {
            eprintln!("  {}", err);
        }
    }

    cases.sort_by(|a, b| a.id.cmp(&b.id));
    cases
}

/// 文件加载的上游 reftest case。
pub struct FileReftestCase {
    /// 测试标识符（相对于 wpt-data 的路径）。
    pub id: String,
    /// 测试 HTML 内容。
    pub test_html: String,
    /// 参考 HTML 内容。
    pub ref_html: String,
    /// 比较模式：true=match，false=mismatch。
    pub is_match: bool,
    /// 分类。
    pub category: ReftestCategory,
}

impl FileReftestCase {
    /// 转换为 ReftestCase（运行器使用的类型）。
    pub fn to_reftest_case(&self) -> ReftestCase {
        ReftestCase {
            id: self.id.clone(),
            test_html: self.test_html.clone(),
            ref_html: self.ref_html.clone(),
            css: String::new(),
            is_match: self.is_match,
        }
    }

    /// 生成 ReftestConfig。
    pub fn to_config(&self, viewport_width: u32, viewport_height: u32) -> ReftestConfig {
        ReftestConfig {
            category: self.category,
            viewport_width,
            viewport_height,
            ..Default::default()
        }
    }
}

/// 递归遍历目录，收集所有文件路径。
fn walk_dir(dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        if let Ok(entries) = std::fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else {
                    files.push(path);
                }
            }
        }
    }
    Ok(files)
}

/// 加载 skip list 文件。
fn load_skip_list(_wpt_data_dir: &Path) -> Vec<String> {
    // 从 wpt-data 旁边的 reftest-skip-list.txt 加载
    let skip_path = PathBuf::from("tests/wpt-runner/reftest-skip-list.txt");
    let content = match std::fs::read_to_string(&skip_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| l.trim().to_string())
        .collect()
}

/// 检查路径是否匹配 skip list。
fn should_skip(relative_path: &str, skip_list: &[String]) -> bool {
    let path_lower = relative_path.to_lowercase();
    for pattern in skip_list {
        // 跳过注释行和空行
        if pattern.starts_with('#') || pattern.is_empty() {
            continue;
        }
        // 支持简单的路径前缀匹配
        if path_lower.contains(&pattern.to_lowercase()) {
            return true;
        }
    }
    false
}
