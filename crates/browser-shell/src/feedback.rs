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

/// unix 秒 → `(年, 月, 日)`（UTC）。
///
/// 依赖图无 chrono，按 civil-from-days（Howard Hinnant 算法）换算。
fn civil_from_unix(unix_secs: u64) -> (i64, u32, u32) {
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
    let year = if m <= 2 { y + 1 } else { y };
    (year, m as u32, d as u32)
}

/// unix 秒 → `YYYYMMDD`（UTC）。
pub fn format_date_ymd(unix_secs: u64) -> String {
    let (y, m, d) = civil_from_unix(unix_secs);
    format!("{y:04}{m:02}{d:02}")
}

/// 两种形态共用的导出载荷：成员字节 + 元数据。目录/ZIP 写出前一次性备齐，
/// 保证两形态内容逐字节一致（同一 environment.json、同一报告拷贝）。
struct FeedbackPayload {
    /// 包名主干（不含扩展名）：`zeroweb-feedback-<版本>-<日期>`。
    stem: String,
    /// 各成员 `(归档内路径, 字节)`；归档内路径已含 `reports/` 前缀。
    members: Vec<(String, Vec<u8>)>,
}

/// 校验勾选项并备齐两种形态共用的成员字节。
///
/// 信任边界：任何非法文件名在触碰文件系统前拒绝；报告读取失败（导出竞态下
/// 报告被删）也在此拒绝，不产出残缺包。
fn prepare_feedback_payload(
    reports_dir: &Path,
    version: &str,
    git_commit: &str,
    selected: &[String],
    options: &FeedbackExportOptions,
) -> Result<FeedbackPayload, String> {
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
    let mut members = Vec::with_capacity(selected_entries.len() + 2);
    for entry in &selected_entries {
        let bytes = fs::read(reports_dir.join(&entry.file_name))
            .map_err(|e| format!("read {} failed: {e}", entry.file_name))?;
        members.push((format!("reports/{}", entry.file_name), bytes));
    }
    let environment = FeedbackEnvironment {
        schema_version: FEEDBACK_SCHEMA_VERSION,
        version,
        git_commit,
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        generated_at: now_secs,
        reports: selected_entries,
    };
    let json = serde_json::to_string_pretty(&environment).map_err(|e| e.to_string())?;
    members.push(("environment.json".to_string(), json.into_bytes()));
    if let Some(summary) = &options.session_summary {
        let json = summary.to_json()?;
        members.push(("session-summary.json".to_string(), json.into_bytes()));
    }
    Ok(FeedbackPayload {
        stem: format!("{FEEDBACK_DIR_PREFIX}{version}-{}", format_date_ymd(now_secs)),
        members,
    })
}

/// 导出反馈包（目录形态，IF-4 调整），返回创建的目录路径。
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
    let payload = prepare_feedback_payload(reports_dir, version, git_commit, selected, options)?;
    let feedback_dir = target_dir.join(&payload.stem);
    if feedback_dir.exists() {
        return Err(format!("feedback package already exists: {}", feedback_dir.display()));
    }
    let export = || -> Result<(), String> {
        // 目录形态恒建 reports/ 子目录（空勾选的空包也保持既有布局）。
        fs::create_dir_all(feedback_dir.join("reports")).map_err(|e| e.to_string())?;
        for (relative, bytes) in &payload.members {
            fs::write(feedback_dir.join(relative), bytes).map_err(|e| e.to_string())?;
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

/// 待落盘的反馈包 zip：建议文件名（含 `.zip`）+ 归档字节。
///
/// GUI 导出经下载管线落盘（同名去重归下载层），CLI 导出直接写目标目录
/// （已存在拒绝），两者共用 `build_feedback_zip_bytes`。
pub struct FeedbackZipPayload {
    /// 建议文件名：`zeroweb-feedback-<版本>-<日期>.zip`。
    pub file_name: String,
    /// 完整归档字节（stored ZIP）。
    pub bytes: Vec<u8>,
}

/// 在内存中构建反馈包 zip（GUI 导出入口用）。
///
/// 归档为 **stored（不压缩）ZIP**：崩溃报告本身是 ≤64KiB 的 JSON 小文件，压缩
/// 收益有限；不引入 zip/压缩依赖（依赖图约束与 S4 目录形态同源，§3 实现来源
/// "优先 zip crate 若依赖图已有"——没有，且根 Cargo.lock 是并行流共享面）。
/// 成员与目录形态逐字节一致。
pub fn build_feedback_zip_bytes(
    reports_dir: &Path,
    version: &str,
    git_commit: &str,
    selected: &[String],
    options: &FeedbackExportOptions,
) -> Result<FeedbackZipPayload, String> {
    let payload = prepare_feedback_payload(reports_dir, version, git_commit, selected, options)?;
    let bytes = zip_store_bytes(&payload.members).map_err(|error| format!("zip build failed: {error}"))?;
    Ok(FeedbackZipPayload {
        file_name: format!("{}.zip", payload.stem),
        bytes,
    })
}

/// 导出反馈包（zip 形态，RFC §4.4 首选/IF-4 原始形态），返回创建的 zip 路径。
///
/// 目标目录下已存在同名 zip 时拒绝（不覆盖）；失败时尽力清理半成品文件。
pub fn export_feedback_zip(
    reports_dir: &Path,
    target_dir: &Path,
    version: &str,
    git_commit: &str,
    selected: &[String],
    options: &FeedbackExportOptions,
) -> Result<PathBuf, String> {
    let payload = build_feedback_zip_bytes(reports_dir, version, git_commit, selected, options)?;
    let zip_path = target_dir.join(&payload.file_name);
    if zip_path.exists() {
        return Err(format!("feedback package already exists: {}", zip_path.display()));
    }
    // 目录形态经 create_dir_all 连带创建 target 父目录；zip 形态显式对齐
    // 同一语义（用户给尚不存在的目标目录是常态）。
    if let Err(error) = fs::create_dir_all(target_dir).and_then(|()| fs::write(&zip_path, payload.bytes)) {
        let _ = fs::remove_file(&zip_path);
        return Err(error.to_string());
    }
    Ok(zip_path)
}

/// 组装 stored（不压缩、无 zip64）ZIP 归档字节。
///
/// 布局：每成员 local file header + 数据，随后 central directory 与 EOCD
/// （PKWARE APPNOTE.TXT §4.3）。成员名均为 ASCII（`reports/*.json` 等），
/// 无需 UTF-8 标志位；总成员数与体积远低于 zip64 阈值。
fn zip_store_bytes(members: &[(String, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let (seconds, minutes, hours, day, month, year) = dos_date_time_parts();
    // DOS 字段宽 5/6/5 与 7/4/5 位（APPNOTE §4.1.5）；年份上限 2107。
    let dos_time = ((hours as u16) << 11) | ((minutes as u16) << 5) | (seconds / 2) as u16;
    let dos_date = ((year.clamp(1980, 2107) as u16 - 1980) << 9) | ((month as u16) << 5) | day as u16;

    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in members {
        let name_bytes = name.as_bytes();
        if name_bytes.len() > u16::MAX as usize || data.len() > u32::MAX as usize {
            return Err(format!("member too large for stored zip: {name}"));
        }
        let offset = out.len() as u32;
        let crc = crc32_ieee(data);
        // Local file header：sig、版本 2.0、无标志位、method 0（stored）。
        out.extend_from_slice(&0x0403_4b50_u32.to_le_bytes());
        out.extend_from_slice(&20_u16.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(&dos_time.to_le_bytes());
        out.extend_from_slice(&dos_date.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0_u16.to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(data);
        // Central directory header：同字段 + 固定盘号/注释长度 0。
        central.extend_from_slice(&0x0201_4b50_u32.to_le_bytes());
        central.extend_from_slice(&20_u16.to_le_bytes());
        central.extend_from_slice(&20_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&dos_time.to_le_bytes());
        central.extend_from_slice(&dos_date.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u16.to_le_bytes());
        central.extend_from_slice(&0_u32.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name_bytes);
    }

    let entries = members.len() as u16;
    let central_offset = out.len() as u32;
    let central_size = central.len() as u32;
    out.extend_from_slice(&central);
    // EOCD。
    out.extend_from_slice(&0x0605_4b50_u32.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    out.extend_from_slice(&entries.to_le_bytes());
    out.extend_from_slice(&entries.to_le_bytes());
    out.extend_from_slice(&central_size.to_le_bytes());
    out.extend_from_slice(&central_offset.to_le_bytes());
    out.extend_from_slice(&0_u16.to_le_bytes());
    Ok(out)
}

/// 当前 UTC 时刻的时间字段分量 `(秒, 分, 时, 日, 月, 公元年)`。
///
/// 拆出独立函数便于单测；DOS 年份基点 1980 的换算由 `zip_store_bytes` 做。
fn dos_date_time_parts() -> (u32, u32, u32, u32, u32, i64) {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (year, month, day) = civil_from_unix(secs);
    let daytime = secs % 86_400;
    (
        (daytime % 60) as u32,
        (daytime / 60 % 60) as u32,
        (daytime / 3600) as u32,
        day,
        month,
        year,
    )
}

/// CRC-32/ISO-HDLC（IEEE 802.3，多项式 0xEDB88320 反射），ZIP 规定算法。
fn crc32_ieee(data: &[u8]) -> u32 {
    // 成员总量上限为保留上限（10×64KiB），逐位实现足够且免建表状态。
    let mut crc = 0xFFFF_FFFF_u32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
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

    #[test]
    fn build_feedback_zip_bytes_matches_export_feedback_zip_output() {
        let root = temp_root("zip-build");
        let reports = root.join("crash-reports");
        fs::create_dir_all(&reports).unwrap();
        fs::write(reports.join("crash-0000000001-browser.json"), r#"{"role":"browser"}"#).unwrap();
        let options = FeedbackExportOptions::default();
        let selected = ["crash-0000000001-browser.json".to_string()];
        // GUI 导出（内存字节）与 CLI 导出（落盘文件）同管线。两次调用各自取
        // now()，跨秒边界时 DOS 时间字段会不同——按成员名+内容比对，文件名
        // 各自匹配命名模式。
        let built = build_feedback_zip_bytes(&reports, "26.10.22", "beef", &selected, &options).unwrap();
        assert!(built.file_name.starts_with("zeroweb-feedback-26.10.22-"));
        assert!(built.file_name.ends_with(".zip"));
        let on_disk =
            export_feedback_zip(&reports, &root.join("out"), "26.10.22", "beef", &selected, &options).unwrap();
        let disk_name = on_disk.file_name().unwrap().to_str().unwrap();
        assert!(disk_name.starts_with("zeroweb-feedback-26.10.22-") && disk_name.ends_with(".zip"));
        let disk_bytes = fs::read(&on_disk).unwrap();
        let report_bytes: &[u8] = b"{\"role\":\"browser\"}";
        assert!(
            contains_member(&built.bytes, "reports/crash-0000000001-browser.json", report_bytes),
            "built zip missing report member"
        );
        assert!(
            contains_member(&disk_bytes, "reports/crash-0000000001-browser.json", report_bytes),
            "disk zip missing report member"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn export_zip_creates_missing_target_directory() {
        let root = temp_root("zip-mkdir");
        let reports = root.join("crash-reports");
        fs::create_dir_all(&reports).unwrap();
        fs::write(reports.join("crash-0000000001-browser.json"), "{}").unwrap();
        let options = FeedbackExportOptions::default();
        // 目标目录整体不存在（含父链）：与目录形态同语义，递归创建。
        let target = root.join("nested/target");
        let zip_path = export_feedback_zip(
            &reports,
            &target,
            "1",
            "c",
            &["crash-0000000001-browser.json".to_string()],
            &options,
        )
        .unwrap();
        assert!(zip_path.is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn crc32_ieee_matches_known_vectors() {
        // CRC-32/ISO-HDLC 标准校验向量。
        assert_eq!(crc32_ieee(b""), 0);
        assert_eq!(crc32_ieee(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32_ieee(b"The quick brown fox jumps over the lazy dog"), 0x414F_A339);
    }

    #[test]
    fn zip_store_bytes_produces_parseable_archive_with_exact_members() {
        let members = vec![
            (
                "reports/crash-0000000001-browser.json".to_string(),
                b"{\"a\":1}".to_vec(),
            ),
            ("environment.json".to_string(), b"{}".to_vec()),
        ];
        let zip = zip_store_bytes(&members).unwrap();
        // 起始 local header 签名与 EOCD 签名。
        assert_eq!(&zip[..4], &0x0403_4b50_u32.to_le_bytes());
        assert_eq!(&zip[zip.len() - 22..zip.len() - 18], &0x0605_4b50_u32.to_le_bytes());
        // central directory 计数 == 成员数（EOCD 偏移 10 处的 entries 字段）。
        let eocd = &zip[zip.len() - 22..];
        let entries = u16::from_le_bytes([eocd[10], eocd[11]]);
        assert_eq!(entries as usize, members.len());
        let central_count = zip.windows(4).filter(|w| *w == 0x0201_4b50_u32.to_le_bytes()).count();
        assert_eq!(central_count, members.len());
        // stored：method 字段为 0，compressed == uncompressed 尺寸。
        let local = &zip[..];
        let method = u16::from_le_bytes([local[8], local[9]]);
        assert_eq!(method, 0);
        let size = u32::from_le_bytes([local[18], local[19], local[20], local[21]]);
        assert_eq!(size as usize, members[0].1.len());
    }

    #[test]
    fn export_zip_mirrors_directory_members_and_rejects_duplicates() {
        let root = temp_root("zip");
        let reports = root.join("crash-reports");
        fs::create_dir_all(&reports).unwrap();
        fs::write(reports.join("crash-0000000001-browser.json"), r#"{"role":"browser"}"#).unwrap();
        let options = FeedbackExportOptions {
            session_summary: Some(SessionSummary {
                tab_count: 2,
                crash_loop_skipped_tabs: 0,
                urls: None,
            }),
        };
        let selected = ["crash-0000000001-browser.json".to_string()];

        // 与目录形态同参数导出：成员逐字节一致（路径与内容）。
        let dir = export_feedback(&reports, &root.join("out"), "26.10.22", "beef", &selected, &options).unwrap();
        let zip_path =
            export_feedback_zip(&reports, &root.join("out"), "26.10.22", "beef", &selected, &options).unwrap();
        assert!(
            zip_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("zeroweb-feedback-26.10.22-")
        );
        assert!(zip_path.to_str().unwrap().ends_with(".zip"));
        let zip_bytes = fs::read(&zip_path).unwrap();
        for entry in walk_members(&dir) {
            // 目录形态的每个文件都应能在 zip 中按相对路径找到，内容一致。
            assert!(
                contains_member(&zip_bytes, &entry.0, &entry.1),
                "zip member {} missing or differs",
                entry.0
            );
        }

        // 已存在同名 zip：拒绝且不覆盖（内容不变）。
        let before = fs::read(&zip_path).unwrap();
        assert!(export_feedback_zip(&reports, &root.join("out"), "26.10.22", "beef", &selected, &options).is_err());
        assert_eq!(fs::read(&zip_path).unwrap(), before);
        fs::remove_dir_all(root).unwrap();
    }

    /// 收集目录形态包内全部文件的 `(相对路径, 内容)`。
    fn walk_members(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let relative = path.strip_prefix(dir).unwrap().to_string_lossy().to_string();
                for sub in walk_members(&path) {
                    out.push((format!("{relative}/{}", sub.0), sub.1));
                }
            } else {
                out.push((
                    path.strip_prefix(dir).unwrap().to_string_lossy().to_string(),
                    fs::read(&path).unwrap(),
                ));
            }
        }
        out.sort();
        out
    }

    /// 在 stored zip 字节中按成员名查找并逐字节比对内容（仅本地验收用，
    /// 不做通用解析；归档由本 crate 自产，布局见 `zip_store_bytes`）。
    fn contains_member(zip: &[u8], name: &str, data: &[u8]) -> bool {
        let name_bytes = name.as_bytes();
        let mut cursor = 0;
        while let Some(pos) = zip[cursor..]
            .windows(4)
            .position(|w| w == 0x0403_4b50_u32.to_le_bytes())
        {
            let header = cursor + pos;
            let name_len = u16::from_le_bytes([zip[header + 26], zip[header + 27]]) as usize;
            let size =
                u32::from_le_bytes([zip[header + 22], zip[header + 23], zip[header + 24], zip[header + 25]]) as usize;
            let name_start = header + 30;
            let data_start = name_start + name_len;
            if &zip[name_start..data_start] == name_bytes && &zip[data_start..data_start + size] == data {
                return true;
            }
            cursor = data_start + size;
            if cursor >= zip.len() {
                break;
            }
        }
        false
    }
}
