//! 本地崩溃报告 — browser panic 与 renderer 异常退出的白名单 JSON 报告。
//!
//! 报告只含 IF-3 白名单字段：无 URL、页面正文、Cookie、token、用户路径
//! （NFR-1）。panic 消息截断至 [`MAX_PANIC_MESSAGE_BYTES`]，panic 位置仅接受
//! 仓内路径（转项目相对，非仓路径丢弃）；renderer 报告只取 OS 退出状态段，
//! stderr 尾巴（可能含页面串）不进报告。本地保留上限
//! [`MAX_RETAINED_REPORTS`] 份，超出按文件名时间戳删最旧；单文件 ≤
//! [`MAX_REPORT_BYTES`]。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::profile::atomic_write;

/// 当前崩溃报告 schema 版本（IF-3）。
pub const CRASH_REPORT_SCHEMA_VERSION: u32 = 1;

/// 单份报告允许的最大字节数（IF-3）。
pub const MAX_REPORT_BYTES: usize = 64 * 1024;

/// 本地保留报告上限（RFC D4/P5）；超出删最旧。
pub const MAX_RETAINED_REPORTS: usize = 10;

/// panic 消息截断上限（RFC §4.2）。
pub const MAX_PANIC_MESSAGE_BYTES: usize = 2 * 1024;

/// panic location 截断上限（防超长路径撑大报告）。
const MAX_LOCATION_BYTES: usize = 512;

/// 崩溃角色（IF-3 白名单枚举）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashRole {
    /// browser 主进程。
    Browser,
    /// renderer 子进程。
    Renderer,
    /// compositor 子进程。
    Compositor,
    /// 图片解码子进程。
    ImageDecoder,
}

impl CrashRole {
    /// 全部角色（文件名解析与枚举校验用）。
    pub const ALL: &'static [CrashRole] = &[
        CrashRole::Browser,
        CrashRole::Renderer,
        CrashRole::Compositor,
        CrashRole::ImageDecoder,
    ];

    /// 报告文件名与 JSON `role` 字段使用的稳定名称。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::Renderer => "renderer",
            Self::Compositor => "compositor",
            Self::ImageDecoder => "image_decoder",
        }
    }
}

/// 崩溃报告（IF-3 白名单字段；白名单外字段一律不落，NFR-1）。
#[derive(Debug, Clone, Serialize)]
pub struct CrashReport {
    /// schema 版本，当前 [`CRASH_REPORT_SCHEMA_VERSION`]。
    pub schema_version: u32,
    /// 角色名称（[`CrashRole::as_str`]）。
    pub role: &'static str,
    /// 产品版本 `YY.M.D`（由宿主注入 [`crate::CrashReport`] 构造参数）。
    pub version: String,
    /// 构建 commit 短 hash。
    pub git_commit: String,
    /// 操作系统（`std::env::consts::OS`）。
    pub os: &'static str,
    /// 体系结构（`std::env::consts::ARCH`）。
    pub arch: &'static str,
    /// 发生线程名；无名线程为线程 id 调试形式。
    pub thread: String,
    /// unix 秒时间戳。
    pub timestamp: u64,
    /// panic 消息（≤2 KiB；仅 browser panic 报告）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panic_message: Option<String>,
    /// panic 位置（仓内路径 `文件:行`；仅 browser panic，非仓路径不落）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panic_location: Option<String>,
    /// 子进程 OS 退出状态文本（如 `signal: 9 (SIGKILL)`；仅子进程异常退出）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renderer_exit_code: Option<String>,
}

impl CrashReport {
    /// 以当前进程环境构造基础报告；版本与 commit 由宿主注入。
    pub fn new(role: CrashRole, version: &str, git_commit: &str) -> Self {
        let thread = std::thread::current();
        Self {
            schema_version: CRASH_REPORT_SCHEMA_VERSION,
            role: role.as_str(),
            version: version.to_string(),
            git_commit: git_commit.to_string(),
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
            thread: thread
                .name()
                .map(str::to_string)
                .unwrap_or_else(|| format!("{:?}", thread.id())),
            timestamp: unix_seconds(),
            panic_message: None,
            panic_location: None,
            renderer_exit_code: None,
        }
    }

    /// 从 browser panic 钩子信息构造报告：消息截断、位置相对化（NFR-1）。
    pub fn from_panic(info: &std::panic::PanicHookInfo<'_>, version: &str, git_commit: &str) -> Self {
        let payload = info.payload();
        let message = payload
            .downcast_ref::<&str>()
            .map(|message| truncate_utf8(message, MAX_PANIC_MESSAGE_BYTES))
            .or_else(|| {
                payload
                    .downcast_ref::<String>()
                    .map(|message| truncate_utf8(message, MAX_PANIC_MESSAGE_BYTES))
            })
            .unwrap_or_else(|| "<非字符串 panic 载荷>".to_string());
        let location = info.location().and_then(|loc| {
            let text = truncate_utf8(&format!("{}:{}", loc.file(), loc.line()), MAX_LOCATION_BYTES);
            repo_relative_location(&text)
        });
        let mut report = Self::new(CrashRole::Browser, version, git_commit);
        report.panic_message = Some(message);
        report.panic_location = location;
        report
    }

    /// 附加子进程退出状态（仅 OS 状态段，stderr 尾巴不进报告）。
    pub fn with_exit_status(mut self, exit_reason: &str) -> Self {
        self.renderer_exit_code = renderer_exit_status(exit_reason);
        self
    }
}

/// 崩溃报告目录：`<配置目录>/zeroweb/crash-reports/`（随 `HOME`/XDG 隔离；
/// `ZERO_CONFIG_DIR`（`--profile`）优先）。
pub fn crash_reports_dir() -> PathBuf {
    crate::config_base_dir().join("zeroweb").join("crash-reports")
}

/// 写入一份崩溃报告（原子替换 + 写前滚动删除），返回报告路径。
pub fn write_report(dir: &Path, report: &CrashReport) -> io::Result<PathBuf> {
    let mut json = serde_json::to_string(report).map_err(io::Error::other)?;
    if json.len() > MAX_REPORT_BYTES {
        // 防线：白名单字段均定长或截断，超限只能来自宿主注入的超长字段；
        // 丢 panic 消息后仍超限则拒绝写入（IF-3 单文件上限不可破）。
        let mut slim = report.clone();
        slim.panic_message = None;
        json = serde_json::to_string(&slim).map_err(io::Error::other)?;
        if json.len() > MAX_REPORT_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "crash report exceeds 64 KiB cap",
            ));
        }
    }
    fs::create_dir_all(dir)?;
    enforce_retention(dir, MAX_RETAINED_REPORTS - 1);
    let path = dir.join(format!("crash-{}-{}.json", report.timestamp, report.role));
    atomic_write(&path, &json).map_err(io::Error::other)?;
    Ok(path)
}

/// 滚动删除：目录内仅保留最新 `keep` 份报告，返回删除数。
///
/// 文件名含 unix 秒时间戳（`crash-<unix秒>-<角色>.json`），字典序即时间序。
pub fn enforce_retention(dir: &Path, keep: usize) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut reports: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("crash-") && name.ends_with(".json"))
        })
        .collect();
    if reports.len() <= keep {
        return 0;
    }
    reports.sort_unstable();
    let excess = reports.len() - keep;
    reports
        .iter()
        .take(excess)
        .filter(|victim| fs::remove_file(victim).is_ok())
        .count()
}

/// 从 renderer 退出原因提取 OS 状态文本（`renderer exited: <状态>` 段）。
///
/// `; stderr tail:` 之后的内容可能含页面 URL 与日志串（NFR-1），不进报告；
/// 非标准原因（句柄不可用、状态检查失败等）返回 `None`，报告仍写入但不带
/// 退出码。
pub fn renderer_exit_status(exit_reason: &str) -> Option<String> {
    let status = exit_reason
        .strip_prefix("renderer exited: ")?
        .split("; stderr tail:")
        .next()?;
    let status = status.trim();
    (!status.is_empty()).then(|| status.to_string())
}

/// 仓内 panic 位置转项目相对路径；非仓路径（registry 依赖、宿主目录等）
/// 返回 `None`（NFR-1：用户路径不入报告）。
///
/// cargo 构建下 rustc 对 workspace 内 crate 的 `file!()` 本就是 workspace
/// 相对路径，直接放行；绝对路径仅在接受 workspace 前缀剥离后放行。
pub fn repo_relative_location(location: &str) -> Option<String> {
    let path = Path::new(location);
    if path.is_relative() {
        return (!location.is_empty()).then(|| location.to_string());
    }
    strip_workspace_prefix(workspace_root(), location)
}

fn strip_workspace_prefix(root: &Path, location: &str) -> Option<String> {
    if root.as_os_str().is_empty() {
        return None;
    }
    let relative = Path::new(location).strip_prefix(root).ok()?;
    let text = relative.to_string_lossy();
    (!text.is_empty()).then(|| text.into_owned())
}

/// workspace 根（编译期注入；browser-shell 位于 `<root>/crates/browser-shell`）。
fn workspace_root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_default()
    })
}

/// 安装 browser 进程 panic 钩子（FR-4）：白名单报告落盘后调用原钩子保持
/// 既有行为；报告写入失败不阻断进程退出（RFC §6）。报告目录由调用方指定
/// （生产路径传 [`crash_reports_dir()`]，隔离验收可注入独立目录）。
pub fn install_browser_panic_hook(report_dir: PathBuf, version: &'static str, git_commit: &'static str) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        record_panic(&report_dir, version, git_commit, info);
        previous(info);
    }));
}

/// 单次 panic 的报告写入（钩子体）：重入保护 + 写失败静默降级。
fn record_panic(dir: &Path, version: &str, git_commit: &str, info: &std::panic::PanicHookInfo<'_>) {
    static WRITING: AtomicBool = AtomicBool::new(false);
    if WRITING.swap(true, Ordering::SeqCst) {
        return; // 写报告自身 panic：静默放弃，不递归（RFC §4.2/§6）。
    }
    let report = CrashReport::from_panic(info, version, git_commit);
    let result = write_report(dir, &report);
    WRITING.store(false, Ordering::SeqCst);
    if let Err(error) = result {
        // 静默降级：不经 tracing（钩子内避免重入日志子系统），仅 stderr 一行。
        eprintln!("crash report write failed: {error}");
    }
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// 截断到字节上限（UTF-8 字符边界对齐）。
fn truncate_utf8(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::{
        CrashReport, CrashRole, MAX_PANIC_MESSAGE_BYTES, MAX_REPORT_BYTES, MAX_RETAINED_REPORTS, enforce_retention,
        renderer_exit_status, repo_relative_location, strip_workspace_prefix, truncate_utf8, write_report,
    };

    fn probe_report(timestamp: u64) -> CrashReport {
        let mut report = CrashReport::new(CrashRole::Browser, "26.10.22", "probe");
        report.timestamp = timestamp;
        report
    }

    fn unique_dir(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("zw-crash-{tag}-{}", std::process::id()))
    }

    #[test]
    fn panic_hook_writes_whitelist_report_in_real_process() {
        let dir = unique_dir("hook");
        let _ = fs::remove_dir_all(&dir);
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new({
            let dir = dir.clone();
            move |info| super::record_panic(&dir, "26.10.22", "probecommit", info)
        }));
        let message = "intentional probe panic for FR-4";
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            panic!("{message}");
        }));
        std::panic::set_hook(previous);
        assert!(result.is_err(), "probe panic must unwind");

        let mut entries: Vec<std::path::PathBuf> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .collect();
        assert_eq!(entries.len(), 1, "exactly one report expected");
        let path = entries.pop().unwrap();
        assert!(
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("crash-") && name.ends_with("-browser.json")),
            "IF-3 file name layout, got {}",
            path.display()
        );
        let json = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // 白名单字段集合精确匹配（NFR-1：白名单外字段一律不落）。
        let mut keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "arch",
                "git_commit",
                "os",
                "panic_location",
                "panic_message",
                "role",
                "schema_version",
                "thread",
                "timestamp",
                "version",
            ]
        );
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["role"], "browser");
        assert_eq!(value["version"], "26.10.22");
        assert_eq!(value["git_commit"], "probecommit");
        assert_eq!(value["panic_message"], message);
        // 仓内路径相对化：无绝对路径、无用户目录泄漏（NFR-1）。
        let location = value["panic_location"].as_str().unwrap();
        assert!(
            location.starts_with("crates/browser-shell/src/crash_report.rs:"),
            "repo-relative location expected, got {location}"
        );
        assert!(!json.contains(std::env::temp_dir().to_string_lossy().as_ref()));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn whitelist_report_for_renderer_has_no_panic_fields() {
        let dir = unique_dir("renderer");
        let report = CrashReport::new(CrashRole::Renderer, "26.10.22", "probe").with_exit_status(
            "renderer exited: signal: 9 (SIGKILL); stderr tail: page https://secret.example render log",
        );
        let path = write_report(&dir, &report).unwrap();
        let json = fs::read_to_string(&path).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["role"], "renderer");
        assert_eq!(value["renderer_exit_code"], "signal: 9 (SIGKILL)");
        assert!(value.get("panic_message").is_none());
        assert!(value.get("panic_location").is_none());
        // stderr 尾巴（可能含 URL）不得进入报告。
        assert!(!json.contains("secret.example"));
        assert!(!json.contains("stderr tail"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn renderer_exit_status_keeps_os_status_and_drops_stderr_tail() {
        assert_eq!(
            renderer_exit_status("renderer exited: signal: 6 (SIGABRT); stderr tail: noise"),
            Some("signal: 6 (SIGABRT)".to_string())
        );
        assert_eq!(
            renderer_exit_status("renderer exited: exit status: 101"),
            Some("exit status: 101".to_string())
        );
        assert_eq!(renderer_exit_status("renderer process handle is unavailable"), None);
        assert_eq!(renderer_exit_status("无法检查 renderer 进程状态: boom"), None);
        assert_eq!(renderer_exit_status("renderer exited: ; stderr tail: x"), None);
    }

    #[test]
    fn non_workspace_panic_location_is_dropped() {
        let root = Path::new("/build/zeroweb");
        assert_eq!(
            strip_workspace_prefix(root, "/build/zeroweb/crates/browser-shell/src/lib.rs:12"),
            Some("crates/browser-shell/src/lib.rs:12".to_string())
        );
        // 非仓路径（registry 依赖、宿主目录）不落报告（NFR-1）。
        assert_eq!(
            strip_workspace_prefix(root, "/home/user/.cargo/registry/src/x.rs:1"),
            None
        );
        assert_eq!(strip_workspace_prefix(root, "/etc/passwd:1"), None);
        assert_eq!(strip_workspace_prefix(Path::new(""), "/build/zeroweb/x.rs:1"), None);
        // cargo 构建下 rustc 传入的 workspace 相对路径直接放行。
        assert_eq!(
            repo_relative_location("crates/browser-shell/src/crash_report.rs:7"),
            Some("crates/browser-shell/src/crash_report.rs:7".to_string())
        );
    }

    #[test]
    fn long_panic_message_is_truncated_on_char_boundary() {
        let ascii = "x".repeat(MAX_PANIC_MESSAGE_BYTES + 100);
        assert_eq!(
            truncate_utf8(&ascii, MAX_PANIC_MESSAGE_BYTES).len(),
            MAX_PANIC_MESSAGE_BYTES
        );
        // 多字节字符边界：截断点回退到字符边界，不产生无效 UTF-8。
        let multibyte = "驼".repeat(2000);
        let truncated = truncate_utf8(&multibyte, MAX_PANIC_MESSAGE_BYTES);
        assert!(truncated.len() <= MAX_PANIC_MESSAGE_BYTES);
        assert!(truncated.chars().all(|c| c == '驼'));
        assert_eq!(truncate_utf8("short", 100), "short");
    }

    #[test]
    fn retention_prunes_oldest_and_keeps_cap() {
        let dir = unique_dir("retention");
        let _ = fs::remove_dir_all(&dir);
        // 时间戳用与真实 unix 秒同宽（10 位）的值：文件名字典序 = 时间序
        // （RFC §4.2 的固定宽度前提）。
        let base: u64 = 1_754_697_501;
        let count = MAX_RETAINED_REPORTS as u64 + 2;
        for offset in 0..count {
            write_report(&dir, &probe_report(base + offset)).unwrap();
        }
        let mut remaining: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
            .collect();
        assert_eq!(remaining.len(), MAX_RETAINED_REPORTS, "cap must hold");
        // 最旧两份被删（写前保留 MAX-1 份，新写入后共 MAX 份）。
        assert!(!remaining.contains(&format!("crash-{base}-browser.json")));
        assert!(!remaining.contains(&format!("crash-{}-browser.json", base + 1)));
        assert!(remaining.contains(&format!("crash-{}-browser.json", base + 2)));
        assert!(remaining.contains(&format!("crash-{}-browser.json", base + count - 1)));

        // 独立调用：已在上限时再清不动。
        assert_eq!(enforce_retention(&dir, MAX_RETAINED_REPORTS), 0);
        assert_eq!(enforce_retention(&dir, 5), 5);
        remaining = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
            .collect();
        assert_eq!(remaining.len(), 5);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversize_report_is_rejected() {
        let dir = unique_dir("oversize");
        let mut report = probe_report(1);
        report.version = "v".repeat(MAX_REPORT_BYTES + 1);
        let error = write_report(&dir, &report).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn report_file_name_follows_if3_layout() {
        let dir = unique_dir("layout");
        let path = write_report(&dir, &probe_report(1_754_697_600)).unwrap();
        assert_eq!(path.file_name().unwrap(), "crash-1754697600-browser.json");
        let _ = fs::remove_dir_all(&dir);
    }
}
