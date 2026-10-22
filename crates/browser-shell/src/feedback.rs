//! FR-5/FR-6 崩溃报告反馈包 — 报告列举/删除与用户同意后的本地导出。
//!
//! 导出是显式两步面（RFC §4.4）：先列举预览（文件名 + 角色 + 时间 + 大小，
//! 可单项删除），用户指定目标路径后才生成。依赖图无 zip crate，按 RFC §3
//! 回退为目录形态（IF-4 调整）：`<目标>/zeroweb-feedback-<版本>-<日期>/`，
//! 内含 `reports/`（勾选报告原样拷贝）、`environment.json`（版本/hash/os/
//! arch/报告清单）、可选 `session-summary.json`（tab 计数；含 URL 需显式
//! 勾选，NFR-1 默认不含）。拒绝/不操作 = 不产生任何文件。

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::crash_report::CrashRole;

/// 反馈包目录名前缀（IF-4，目录形态）。
const FEEDBACK_DIR_PREFIX: &str = "zeroweb-feedback-";

/// environment.json / session-summary.json 的 schema 版本。
pub const FEEDBACK_SCHEMA_VERSION: u32 = 1;

/// 单份崩溃报告的列举条目（预览面字段：文件名 + 角色 + 时间 + 大小）。
#[derive(Debug, Clone, Serialize)]
pub struct CrashReportEntry {
    /// 报告文件名（不含目录，如 `crash-1792668836-renderer.json`）。
    pub file_name: String,
    /// 角色稳定名（browser/renderer/compositor/image_decoder）。
    pub role: String,
    /// 崩溃时刻 unix 秒（取自文件名，非 mtime）。
    pub timestamp: u64,
    /// 文件字节数。
    pub size_bytes: u64,
}

/// 会话摘要（IF-4 可选件）。URL 仅在用户显式勾选时携带，默认 `None`。
#[derive(Debug, Clone)]
pub struct SessionSummary {
    /// 会话中的标签页数量（不含无痕标签）。
    pub tab_count: usize,
    /// 达到崩溃循环阈值、恢复时被跳过自动加载的 tab 数（FR-6 跳过说明）。
    pub crash_loop_skipped_tabs: usize,
    /// 各 tab URL；仅在用户显式勾选"包含 URL"时为 `Some`。
    pub urls: Option<Vec<String>>,
}

/// 反馈包导出选项。
#[derive(Debug, Clone, Default)]
pub struct FeedbackExportOptions {
    /// 是否附 `session-summary.json`。
    pub session_summary: Option<SessionSummary>,
}

/// 解析报告文件名为 `(unix 秒, 角色)`。
///
/// 仅接受 `crash-<纯数字>-<角色>.json` 且角色为 [`CrashRole`] 稳定名；其余
/// （含含路径分隔符、`..`、目录名等）一律 `None`——文件名来自目录枚举与用户
/// 选项，是信任边界输入。
pub fn parse_report_file_name(file_name: &str) -> Option<(u64, CrashRole)> {
    let rest = file_name.strip_prefix("crash-")?.strip_suffix(".json")?;
    if rest.is_empty() || file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return None;
    }
    let (stamp, role_str) = rest.split_once('-')?;
    if stamp.is_empty() || !stamp.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let timestamp = stamp.parse().ok()?;
    let role = CrashRole::ALL.iter().copied().find(|r| r.as_str() == role_str)?;
    Some((timestamp, role))
}

/// 列举报告目录中的崩溃报告，按文件名（即时间）升序。
///
/// 目录不存在返回空表；不合规文件名跳过。
pub fn list_reports(dir: &Path) -> Vec<CrashReportEntry> {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut entries: Vec<CrashReportEntry> = read_dir
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_file() {
                return None;
            }
            let file_name = path.file_name()?.to_str()?.to_string();
            let (timestamp, role) = parse_report_file_name(&file_name)?;
            let size_bytes = fs::metadata(&path).ok()?.len();
            Some(CrashReportEntry {
                file_name,
                role: role.as_str().to_string(),
                timestamp,
                size_bytes,
            })
        })
        .collect();
    entries.sort_unstable_by(|a, b| a.file_name.cmp(&b.file_name));
    entries
}

/// 删除单份报告。
///
/// 文件名先过 [`parse_report_file_name`] 校验（拒绝穿越/任意路径），再在报告
/// 目录内删除。
pub fn delete_report(dir: &Path, file_name: &str) -> Result<(), String> {
    if parse_report_file_name(file_name).is_none() {
        return Err(format!("invalid crash report file name: {file_name}"));
    }
    let path = dir.join(file_name);
    fs::remove_file(&path).map_err(|e| format!("failed to delete {}: {e}", path.display()))
}

/// unix 秒 → `YYYYMMDD`（UTC）。
///
/// 依赖图无 chrono，按 civil-from-days（Howard Hinnant 算法）换算。
pub fn format_date_ymd(unix_secs: u64) -> String {
    let days = unix_secs / 86_400;
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}{m:02}{d:02}")
}

/// 导出反馈包（目录形态），返回创建的目录路径。
///
/// `selected` 为用户勾选的报告文件名列表；目标目录下已存在同名反馈包时拒绝
/// （不覆盖、不合并，显式语义）。任一步失败时尽力清理已创建的目录，不留残缺
/// 包。调用前不创建任何文件——拒绝/不操作即无副作用。
pub fn export_feedback(
    reports_dir: &Path,
    target_dir: &Path,
    version: &str,
    git_commit: &str,
    selected: &[String],
    options: &FeedbackExportOptions,
) -> Result<PathBuf, String> {
    // 信任边界：先全量校验勾选项，任何非法文件名在触碰文件系统前拒绝。
    let mut selected_entries = Vec::with_capacity(selected.len());
    let all = list_reports(reports_dir);
    for name in selected {
        if parse_report_file_name(name).is_none() {
            return Err(format!("invalid crash report file name: {name}"));
        }
        let entry = all
            .iter()
            .find(|e| &e.file_name == name)
            .ok_or_else(|| format!("report not found in {}: {name}", reports_dir.display()))?;
        selected_entries.push(entry.clone());
    }

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let feedback_dir = target_dir.join(format!("{FEEDBACK_DIR_PREFIX}{version}-{}", format_date_ymd(now_secs)));
    if feedback_dir.exists() {
        return Err(format!("feedback package already exists: {}", feedback_dir.display()));
    }
    let export = || -> Result<(), String> {
        fs::create_dir_all(feedback_dir.join("reports")).map_err(|e| e.to_string())?;
        for entry in &selected_entries {
            let source = reports_dir.join(&entry.file_name);
            let dest = feedback_dir.join("reports").join(&entry.file_name);
            fs::copy(&source, &dest).map_err(|e| format!("copy {} failed: {e}", entry.file_name))?;
        }
        let environment = FeedbackEnvironment {
            schema_version: FEEDBACK_SCHEMA_VERSION,
            version,
            git_commit,
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            generated_at: now_secs,
            reports: selected_entries.clone(),
        };
        let json = serde_json::to_string_pretty(&environment).map_err(|e| e.to_string())?;
        fs::write(feedback_dir.join("environment.json"), json).map_err(|e| e.to_string())?;
        if let Some(summary) = &options.session_summary {
            let json = summary.to_json()?;
            fs::write(feedback_dir.join("session-summary.json"), json).map_err(|e| e.to_string())?;
        }
        Ok(())
    };
    if let Err(error) = export() {
        // 防残缺包：失败即清理本次创建的目录。
        let _ = fs::remove_dir_all(&feedback_dir);
        return Err(error);
    }
    Ok(feedback_dir)
}

/// environment.json 结构（IF-4：版本/hash/os/arch/报告清单）。
#[derive(Debug, Serialize)]
struct FeedbackEnvironment<'a> {
    schema_version: u32,
    version: &'a str,
    git_commit: &'a str,
    os: &'static str,
    arch: &'static str,
    generated_at: u64,
    reports: Vec<CrashReportEntry>,
}

/// session-summary.json 的序列化形态（URL 缺省不落，NFR-1）。
#[derive(Debug, Serialize)]
struct SessionSummarySerialized<'a> {
    schema_version: u32,
    tab_count: usize,
    crash_loop_skipped_tabs: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    urls: Option<&'a Vec<String>>,
}

impl SessionSummary {
    fn to_json(&self) -> Result<String, String> {
        let serialized = SessionSummarySerialized {
            schema_version: FEEDBACK_SCHEMA_VERSION,
            tab_count: self.tab_count,
            crash_loop_skipped_tabs: self.crash_loop_skipped_tabs,
            urls: self.urls.as_ref(),
        };
        serde_json::to_string_pretty(&serialized).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zeroweb-feedback-{tag}-{}", std::process::id()))
    }

    #[test]
    fn parse_accepts_valid_report_names_and_rejects_traversal() {
        let (ts, role) = parse_report_file_name("crash-1792668836-renderer.json").unwrap();
        assert_eq!(ts, 1_792_668_836);
        assert_eq!(role.as_str(), "renderer");
        assert!(parse_report_file_name("crash-1792668836-image_decoder.json").is_some());
        for bad in [
            "crash-1792668836.json",
            "crash--renderer.json",
            "crash-12x3-renderer.json",
            "crash-1792668836-unknown.json",
            "../crash-1792668836-renderer.json",
            "sub/crash-1792668836-renderer.json",
            "crash-1792668836-renderer.json.bak",
            "",
        ] {
            assert!(parse_report_file_name(bad).is_none(), "should reject {bad:?}");
        }
    }

    #[test]
    fn list_reports_skips_foreign_files_and_sorts_by_time() {
        let root = temp_root("list");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("crash-0000000002-renderer.json"), "{}").unwrap();
        fs::write(root.join("crash-0000000001-browser.json"), "{}").unwrap();
        fs::write(root.join("unrelated.txt"), "x").unwrap();
        fs::create_dir_all(root.join("crash-0000000003-renderer.json")).unwrap();
        let entries = list_reports(&root);
        let names: Vec<&str> = entries.iter().map(|e| e.file_name.as_str()).collect();
        assert_eq!(
            names,
            ["crash-0000000001-browser.json", "crash-0000000002-renderer.json"]
        );
        assert_eq!(entries[0].role, "browser");
        assert_eq!(entries[0].size_bytes, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn delete_report_rejects_bad_names_and_removes_existing() {
        let root = temp_root("delete");
        fs::create_dir_all(&root).unwrap();
        let name = "crash-0000000005-compositor.json";
        fs::write(root.join(name), "{}").unwrap();
        assert!(delete_report(&root, "../../etc/passwd").is_err());
        assert!(delete_report(&root, "missing.json").is_err());
        delete_report(&root, name).unwrap();
        assert!(!root.join(name).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_creates_directory_package_with_environment_and_copies() {
        let root = temp_root("export");
        let reports = root.join("crash-reports");
        let target = root.join("out");
        fs::create_dir_all(&reports).unwrap();
        fs::write(reports.join("crash-0000000001-browser.json"), r#"{"role":"browser"}"#).unwrap();
        fs::write(reports.join("crash-0000000002-renderer.json"), r#"{"role":"renderer"}"#).unwrap();

        let options = FeedbackExportOptions {
            session_summary: Some(SessionSummary {
                tab_count: 3,
                crash_loop_skipped_tabs: 1,
                urls: None,
            }),
        };
        let dir = export_feedback(
            &reports,
            &target,
            "26.10.22",
            "be38b2ee1828",
            &["crash-0000000001-browser.json".to_string()],
            &options,
        )
        .unwrap();
        assert!(
            dir.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("zeroweb-feedback-26.10.22-")
        );
        // 未勾选的报告不进包。
        assert_eq!(fs::read_dir(dir.join("reports")).unwrap().count(), 1);
        assert_eq!(
            fs::read(dir.join("reports").join("crash-0000000001-browser.json")).unwrap(),
            fs::read(reports.join("crash-0000000001-browser.json")).unwrap()
        );
        let env: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("environment.json")).unwrap()).unwrap();
        assert_eq!(env["schema_version"], FEEDBACK_SCHEMA_VERSION);
        assert_eq!(env["version"], "26.10.22");
        assert_eq!(env["git_commit"], "be38b2ee1828");
        assert_eq!(env["reports"][0]["file_name"], "crash-0000000001-browser.json");
        // 摘要默认不含 URL 字段。
        let summary_raw = fs::read_to_string(dir.join("session-summary.json")).unwrap();
        assert!(!summary_raw.contains("url"));
        let summary: serde_json::Value = serde_json::from_str(&summary_raw).unwrap();
        assert_eq!(summary["tab_count"], 3);
        assert_eq!(summary["crash_loop_skipped_tabs"], 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_includes_urls_only_when_explicitly_opted_in() {
        let root = temp_root("urls");
        let reports = root.join("crash-reports");
        fs::create_dir_all(&reports).unwrap();
        fs::write(reports.join("crash-0000000001-browser.json"), "{}").unwrap();
        let options = FeedbackExportOptions {
            session_summary: Some(SessionSummary {
                tab_count: 1,
                crash_loop_skipped_tabs: 0,
                urls: Some(vec!["https://example.com/".to_string()]),
            }),
        };
        let dir = export_feedback(
            &reports,
            &root.join("out"),
            "1",
            "c",
            &["crash-0000000001-browser.json".to_string()],
            &options,
        )
        .unwrap();
        let summary_raw = fs::read_to_string(dir.join("session-summary.json")).unwrap();
        assert!(summary_raw.contains("https://example.com/"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_rejects_existing_package_bad_names_and_missing_files_without_side_effects() {
        let root = temp_root("reject");
        let reports = root.join("crash-reports");
        fs::create_dir_all(&reports).unwrap();
        fs::write(reports.join("crash-0000000001-browser.json"), "{}").unwrap();
        let options = FeedbackExportOptions::default();
        let selected = ["crash-0000000001-browser.json".to_string()];

        // 目标已存在同名包：拒绝且不改动既有内容。
        let target = root.join("out");
        let first = export_feedback(&reports, &target, "1", "c", &selected, &options).unwrap();
        assert!(export_feedback(&reports, &target, "1", "c", &selected, &options).is_err());
        assert!(first.join("environment.json").exists());

        // 非法文件名在触碰文件系统前拒绝。
        assert!(
            export_feedback(
                &reports,
                target.parent().unwrap(),
                "1",
                "c",
                &["../x.json".to_string()],
                &options
            )
            .is_err()
        );
        // 勾选了不存在的报告：拒绝。
        assert!(
            export_feedback(
                &reports,
                target.parent().unwrap(),
                "2",
                "c",
                &["crash-9999999999-browser.json".to_string()],
                &options
            )
            .is_err()
        );
        // 空勾选也生成合法的空包（用户明确导出的显式动作）。
        let empty = export_feedback(&reports, target.parent().unwrap(), "3", "c", &[], &options).unwrap();
        assert_eq!(fs::read_dir(empty.join("reports")).unwrap().count(), 0);
        assert!(!empty.join("session-summary.json").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn format_date_ymd_matches_known_utc_dates() {
        assert_eq!(format_date_ymd(0), "19700101");
        // 2026-10-22 的 unix 秒（S3 验收期间的真实时间戳）。
        assert_eq!(format_date_ymd(1_792_668_836), "20261022");
        assert_eq!(format_date_ymd(951_782_400), "20000229");
    }
}
