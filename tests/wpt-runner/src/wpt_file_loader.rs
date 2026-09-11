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

            // 每个测试文件一个 case，携带全部参考（WPT「Multiple References」：
            // match 至少一个匹配 + mismatch 全部不匹配，由 combine_multi_ref 聚合）。
            let test_base = test_path.parent().map(|p| p.to_path_buf());
            let mut refs = Vec::with_capacity(references.len());
            for reference in &references {
                let raw_ref = reference.ref_path.trim();

                // about:blank 是 WPT reftest 的特殊参考（空白文档，常用于 match「应渲染为空白」
                // 的用例）。它不是文件路径，不读磁盘——直接当空 HTML。否则 read_to_string 会
                // 报 No such file 并把测试误排除出分母（DC-14 分母真实性，R551/R552 谱系）。
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
                refs.push(FileRef {
                    ref_html,
                    ref_base_dir,
                    is_match: reference.is_match(),
                });
            }
            if refs.is_empty() {
                continue; // 参考文件全部读取失败（错误已在上方记录）
            }

            cases.push(FileReftestCase {
                id: relative_str.clone(),
                test_html: test_html.clone(),
                refs,
                category: ReftestCategory::from_path(&relative_str),
                base_dir: test_base,
            });
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

/// 多参考测试的单条参考（WPT「Multiple References」：一个测试文件可声明多条
/// `<link rel="match">` / `<link rel="mismatch">`）。
pub struct FileRef {
    /// 参考 HTML 内容。
    pub ref_html: String,
    /// 参考文件所在目录（用于解析参考页相对图片路径）。
    ///
    /// 参考文件常位于 `reference/` 子目录，其相对图片 URL（如 `../support/x.png`）
    /// 必须相对参考文件自身目录解析。about:blank 参考无文件，回落到测试目录。
    pub ref_base_dir: Option<PathBuf>,
    /// 比较模式：true=match，false=mismatch。
    pub is_match: bool,
}

/// 文件加载的上游 reftest case。
pub struct FileReftestCase {
    /// 测试标识符（相对于 wpt-data 的路径；多参考测试不再展开 `#N` 变体后缀）。
    pub id: String,
    /// 测试 HTML 内容。
    pub test_html: String,
    /// 全部参考（≥1），按 `<link>` 声明序排列。
    pub refs: Vec<FileRef>,
    /// 分类。
    pub category: ReftestCategory,
    /// 测试文件所在目录（用于解析相对图片路径）。
    pub base_dir: Option<PathBuf>,
}

impl FileReftestCase {
    /// 首个参考（单参考测试的唯一参考；多参考测试的 diagnostics 用首页）。
    pub fn first_ref(&self) -> &FileRef {
        &self.refs[0]
    }

    /// 转换为第 `ref_idx` 条参考的 ReftestCase（运行器使用的类型）。
    pub fn to_reftest_case(&self, ref_idx: usize) -> ReftestCase {
        let r = &self.refs[ref_idx];
        ReftestCase {
            id: self.id.clone(),
            test_html: self.test_html.clone(),
            ref_html: r.ref_html.clone(),
            css: String::new(),
            is_match: r.is_match,
            ref_base_dir: r.ref_base_dir.clone(),
        }
    }

    /// 生成 ReftestConfig。
    pub fn to_config(&self, viewport_width: u32, viewport_height: u32) -> ReftestConfig {
        ReftestConfig::for_category(self.category).with_viewport(viewport_width, viewport_height)
    }
}

/// WPT 多参考聚合判定（web-platform-tests.org/writing-tests/reftests.html
/// 「Multiple References」）：「If there are any match references, at least one must
/// match, and if there are any mismatch references, all must mismatch.」
///
/// 旧模型把每条参考展开成独立 case（每条都须 pass）——多 match 参考的测试被按 AND
/// 记账，任一条配对不上即永红（如 table-anonymous-objects-115#0 配对的
/// no_red_3x3_monospace_table-ref 与该测试内容无关，永远不可能匹配）。上游语义为
/// match 取 any、mismatch 取 all，本函数按此聚合逐参考比较结果。
///
/// `per_ref`: (is_match, 该参考的比较结果)，顺序与 `refs` 一致。
/// `representative` 为报告用的代表性比较下标（mismatch 失配优先，其次 diff 最小的
/// 匹配成功项，再其次 diff 最小的任一 match 比较）。
pub struct MultiRefVerdict {
    pub passed: bool,
    pub representative: usize,
}

pub fn combine_multi_ref(per_ref: &[(bool, crate::reftest::ReftestResult)]) -> MultiRefVerdict {
    let mut mismatch_ok = true;
    let mut first_mismatch_fail: Option<usize> = None;
    for (i, (is_match, result)) in per_ref.iter().enumerate() {
        if !is_match && !result.passed {
            mismatch_ok = false;
            first_mismatch_fail.get_or_insert(i);
        }
    }
    let match_idxs: Vec<usize> = per_ref
        .iter()
        .enumerate()
        .filter(|(_, (is_match, _))| *is_match)
        .map(|(i, _)| i)
        .collect();
    let match_ok = match_idxs.is_empty() || match_idxs.iter().any(|&i| per_ref[i].1.passed);

    // 代表性比较：mismatch 失配 > pass 的 match 中 diff 最小 > match 中 diff 最小 >
    // 兜底 0（纯 mismatch 全过）。
    let by_diff = |&a: &usize, &b: &usize| {
        per_ref[a]
            .1
            .diff_ratio
            .partial_cmp(&per_ref[b].1.diff_ratio)
            .unwrap_or(std::cmp::Ordering::Equal)
    };
    let representative = if let Some(i) = first_mismatch_fail {
        i
    } else {
        match_idxs
            .iter()
            .copied()
            .filter(|&i| per_ref[i].1.passed)
            .min_by(by_diff)
            .or_else(|| match_idxs.iter().copied().min_by(by_diff))
            .unwrap_or_default()
    };

    MultiRefVerdict {
        passed: match_ok && mismatch_ok,
        representative,
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
    use super::{MultiRefVerdict, combine_multi_ref, stable_case_id};
    use crate::reftest::ReftestResult;
    use std::path::Path;

    #[test]
    fn case_ids_use_url_separators_on_every_platform() {
        assert_eq!(
            stable_case_id(Path::new(r"css\CSS2\abspos\case.xht")),
            "css/CSS2/abspos/case.xht"
        );
    }

    /// 构造聚合测试用的比较结果。
    fn result(id: &str, is_match: bool, passed: bool, diff_ratio: f64) -> (bool, ReftestResult) {
        (
            is_match,
            ReftestResult {
                id: id.to_string(),
                passed,
                diff_pixels: 0,
                total_pixels: 480000,
                diff_ratio,
                max_channel_diff: 0,
                subpixel_diff_pixels: 0,
                message: String::new(),
                test_near_solid: false,
            },
        )
    }

    // WPT「Multiple References」：match 至少一个匹配 + mismatch 全部不匹配。

    #[test]
    fn multi_ref_match_any_passes() {
        // table-anonymous-objects-115 谱系：两条 match 参考，一条配对不上（永红）
        // 一条匹配 → 上游语义应 pass。
        let per = vec![result("t", true, false, 0.0174), result("t", true, true, 0.0006)];
        let MultiRefVerdict { passed, .. } = combine_multi_ref(&per);
        assert!(passed, "任一 match 匹配即应通过");
    }

    #[test]
    fn multi_ref_match_all_fail_stays_red() {
        let per = vec![result("t", true, false, 0.02), result("t", true, false, 0.03)];
        let MultiRefVerdict { passed, .. } = combine_multi_ref(&per);
        assert!(!passed, "全部 match 失配应保持红");
    }

    #[test]
    fn multi_ref_mismatch_must_all_mismatch() {
        // match 匹配但 mismatch 参考与 test 渲染相同（min_mismatch_ratio 不达标）→ fail。
        let per = vec![result("t", true, true, 0.0), result("t", false, false, 0.0)];
        let MultiRefVerdict { passed, representative } = combine_multi_ref(&per);
        assert!(!passed, "mismatch 参考未失配应不通过");
        assert_eq!(representative, 1, "报告应指向失配的 mismatch 比较");
    }

    #[test]
    fn multi_ref_mismatch_all_mismatch_passes() {
        let per = vec![result("t", true, true, 0.001), result("t", false, true, 0.2)];
        let MultiRefVerdict { passed, .. } = combine_multi_ref(&per);
        assert!(passed, "match 匹配 + mismatch 全失配应通过");
    }

    #[test]
    fn multi_ref_representative_prefers_passing_min_diff() {
        let per = vec![result("t", true, true, 0.05), result("t", true, true, 0.001)];
        let MultiRefVerdict { representative, .. } = combine_multi_ref(&per);
        assert_eq!(representative, 1, "报告应取匹配成功中 diff 最小的比较");
    }
}
