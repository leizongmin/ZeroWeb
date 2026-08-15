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
            // WPT ids are URL-like stable identifiers, not host filesystem paths. Keep
            // smoke lists and reports identical on Windows and Unix.
            let relative_str = stable_case_id(relative);
            if should_skip(&relative_str, &skip_list) {
                continue;
            }

            // 跳过参考文件：以 -ref.html/-ref.xht 结尾的文件是参考页面，不应作为测试用例运行
            let file_stem = relative.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if file_stem.ends_with("-ref") || file_stem.ends_with("-reference") || file_stem.contains("-notref") {
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
                let raw_ref = reference.ref_path.trim();

                // about:blank 是 WPT reftest 的特殊参考（空白文档，常用于 match「应渲染为空白」
                // 的用例）。它不是文件路径，不读磁盘——直接当空 HTML。否则 read_to_string 会
                // 报 No such file 并把测试误排除出分母（DC-14 分母真实性，R551/R552 谱系）。
                //
                // ref_base_dir = 参考文件所在目录，用于解析参考页相对图片 URL。
                // about:blank 无文件，回落到测试文件目录。
                let test_base = test_path.parent().map(|p| p.to_path_buf());
                let (ref_html, ref_base_dir) = if raw_ref == "about:blank" {
                    (
                        String::from("<!DOCTYPE html><html><head></head><body></body></html>"),
                        test_base.clone(),
                    )
                } else {
                    let ref_path = resolve_ref_path(wpt_data_dir, &test_path, raw_ref);
                    let ref_base = ref_path.parent().map(|p| p.to_path_buf());
                    match std::fs::read_to_string(&ref_path) {
                        Ok(html) => (html, ref_base),
                        Err(e) => {
                            errors.push(format!("{} ref {}: {}", relative_str, reference.ref_path, e));
                            continue;
                        }
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
                    base_dir: test_base,
                    ref_base_dir,
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

fn stable_case_id(relative: &Path) -> String {
    relative.to_string_lossy().replace('\\', "/")
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
    /// 测试文件所在目录（用于解析相对图片路径）。
    pub base_dir: Option<PathBuf>,
    /// 参考文件所在目录（用于解析参考页相对图片路径）。
    ///
    /// 参考文件常位于 `reference/` 子目录，其相对图片 URL（如 `../support/x.png`）
    /// 必须相对参考文件自身目录解析。about:blank 参考无文件，回落到测试目录。
    pub ref_base_dir: Option<PathBuf>,
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
            ref_base_dir: self.ref_base_dir.clone(),
        }
    }

    /// 生成 ReftestConfig。
    pub fn to_config(&self, viewport_width: u32, viewport_height: u32) -> ReftestConfig {
        ReftestConfig::for_category(self.category).with_viewport(viewport_width, viewport_height)
    }
}

/// 解析 reftest 参考文件路径。
///
/// - 绝对 WPT 路径（以 `/` 开头，如 `/css/reference/foo.xht`）：相对 wpt-data 根解析。
///   注意不能用 `Path::join`：当 join 的参数是绝对路径时，Rust 会丢弃 base、从文件系统
///   根查找，导致已存在的 `/css/reference/...` ref 报「No such file」并把测试误排除出
///   分母（DC-14 分母真实性缺口，R546 / R551 谱系）。
/// - 相对路径（如 `ref.html`、`../reference/foo.xht`）：相对测试文件父目录解析。
///
/// 先对 ref_path 做 `trim()`：上游 WPT 偶有 href 值带尾随空白（如
/// `border-collapse-005-ref.html `），浏览器按 URL 解析语义会 strip 掉，ZeroWeb 加载器
/// 须一致处理，否则 `Path::join` 拼出带空格的文件名报「No such file」并把测试误排除出
/// 分母（R552，R551 谱系）。
pub(super) fn resolve_ref_path(wpt_data_dir: &Path, test_path: &Path, ref_path: &str) -> PathBuf {
    // 剥离 query（如 `transform-interpolation-ref.html?matrix`——WPT 参数化 ref，
    // runner 不支持参数化渲染，剥离后加载同一文件；2026-08-07）
    let ref_path = ref_path.split('?').next().unwrap_or(ref_path).trim();
    if ref_path.starts_with('/') {
        wpt_data_dir.join(ref_path.trim_start_matches('/'))
    } else {
        test_path.parent().unwrap_or(Path::new(".")).join(ref_path)
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
    // R34xx（canvas-2d goal M3）：canvas 专项的 reftest/oracle A/B 面——rendering-compat
    // 的 skip list 把 html/canvas/ 排除（其 reftest 面归 canvas 专项）；REFTEST_INCLUDE_CANVAS=1
    // 时忽略 canvas 相关 skip 模式（canvas 专项 oracle 测量用，不影响兄弟 goal 分母）。
    let include_canvas = std::env::var("REFTEST_INCLUDE_CANVAS").as_deref() == Ok("1");
    for pattern in skip_list {
        // 跳过注释行和空行
        if pattern.starts_with('#') || pattern.is_empty() {
            continue;
        }
        let pat_lower = pattern.to_lowercase();
        if include_canvas && (pat_lower == "canvas/" || pat_lower == "html/canvas/" || pat_lower == "offscreencanvas/")
        {
            continue;
        }
        // 支持简单的路径前缀匹配
        if path_lower.contains(&pat_lower) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::stable_case_id;
    use std::path::Path;

    #[test]
    fn case_ids_use_url_separators_on_every_platform() {
        assert_eq!(
            stable_case_id(Path::new(r"css\CSS2\abspos\case.xht")),
            "css/CSS2/abspos/case.xht"
        );
    }
}
