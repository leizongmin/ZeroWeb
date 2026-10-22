//! 本地更新源检查与原子应用 — manifest 校验、暂存、前缀翻转与回滚。
//!
//! 安装根布局（RFC §4.1）：`releases/<版本>/`（各版本完整前缀）、`current`
//! （symlink → 活跃版本目录）、`rollback`（symlink → 上一活跃版本目录）、
//! `staging/`（可丢弃暂存）。翻转前任何失败只污染 staging（下次触发先清空）；
//! 翻转本身是单次 rename；翻转后健康检查失败自动翻回 rollback。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// manifest.json 允许的最大字节数。
pub const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

/// 当前支持的更新清单 schema 版本。
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// 更新清单（IF-2 schema v1）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    /// schema 版本，仅接受 [`SUPPORTED_SCHEMA_VERSION`]。
    pub schema_version: u32,
    /// 更新包产品版本，格式 `YY.M.D`。
    pub version: String,
    /// 源目录内的相对文件名（拒绝路径穿越）。
    pub artifact: String,
    /// 更新包 SHA256，64 位小写 hex。
    pub sha256: String,
}

/// 更新流程失败原因（证据与拒绝原因命名对齐 IF-2/RFC §4.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateError {
    /// manifest 缺失、超限、解析失败或字段非法。
    ManifestInvalid,
    /// 候选版本与当前相同。
    UpToDate,
    /// 候选版本低于当前。
    Downgrade,
    /// 源目录中不存在清单声明的 artifact。
    ArtifactMissing,
    /// artifact 复制到暂存失败。
    CopyFailed,
    /// SHA256 校验不符。
    HashMismatch,
    /// 解包失败或暂存前缀缺少 zero-browser。
    ExtractFailed,
    /// 暂存自检（--version 与 manifest 不符）失败。
    StagingCheckFailed,
    /// symlink 翻转失败。
    FlipFailed,
    /// 翻转后健康检查失败（已自动回滚）。
    HealthCheckFailed,
}

impl UpdateError {
    /// 稳定的机器可读名称（用于 CLI 输出与证据）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManifestInvalid => "MANIFEST_INVALID",
            Self::UpToDate => "UP_TO_DATE",
            Self::Downgrade => "DOWNGRADE",
            Self::ArtifactMissing => "ARTIFACT_MISSING",
            Self::CopyFailed => "COPY_FAILED",
            Self::HashMismatch => "HASH_MISMATCH",
            Self::ExtractFailed => "EXTRACT_FAILED",
            Self::StagingCheckFailed => "STAGING_CHECK_FAILED",
            Self::FlipFailed => "FLIP_FAILED",
            Self::HealthCheckFailed => "HEALTH_CHECK_FAILED",
        }
    }
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 一次成功应用的更新结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedUpdate {
    /// 新活跃版本。
    pub version: String,
    /// 应用前的活跃版本（首次安装为 `None`）。
    pub previous_version: Option<String>,
}

/// 解析并校验 manifest 文本（FR-2/NFR-4）。
pub fn parse_manifest(text: &str) -> Result<UpdateManifest, UpdateError> {
    let manifest: UpdateManifest = serde_json::from_str(text).map_err(|_| UpdateError::ManifestInvalid)?;
    if manifest.schema_version != SUPPORTED_SCHEMA_VERSION {
        return Err(UpdateError::ManifestInvalid);
    }
    validate_version(&manifest.version)?;
    if !is_safe_artifact_name(&manifest.artifact) {
        return Err(UpdateError::ManifestInvalid);
    }
    if manifest.sha256.len() != 64
        || manifest
            .sha256
            .chars()
            .any(|c| !c.is_ascii_hexdigit() || c.is_ascii_uppercase())
    {
        return Err(UpdateError::ManifestInvalid);
    }
    Ok(manifest)
}

/// 比较两个 `YY.M.D` 版本序（候选 > 当前 → Greater）。
pub fn compare_versions(current: &str, candidate: &str) -> Result<std::cmp::Ordering, UpdateError> {
    let parse = |value: &str| -> Result<[u16; 3], UpdateError> {
        let parts: Vec<_> = value
            .split('.')
            .map(|p| p.parse::<u16>())
            .collect::<Result<_, _>>()
            .map_err(|_| UpdateError::ManifestInvalid)?;
        if parts.len() != 3 {
            return Err(UpdateError::ManifestInvalid);
        }
        Ok([parts[0], parts[1], parts[2]])
    };
    let current = parse(current)?;
    let candidate = parse(candidate)?;
    Ok(candidate.cmp(&current))
}

/// 计算文件 SHA256（64 位小写 hex）。
pub fn sha256_file(path: &Path) -> Result<String, UpdateError> {
    let bytes = fs::read(path).map_err(|_| UpdateError::CopyFailed)?;
    let digest = Sha256::digest(&bytes);
    let hex = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
    Ok(hex)
}

/// 生产解包器：`dpkg-deb -x`（复用平台 dpkg 套件，M1 已验证）。
pub fn dpkg_deb_extract(deb: &Path, dest: &Path) -> Result<(), UpdateError> {
    let status = Command::new("dpkg-deb")
        .arg("-x")
        .arg(deb)
        .arg(dest)
        .status()
        .map_err(|_| UpdateError::ExtractFailed)?;
    if !status.success() {
        return Err(UpdateError::ExtractFailed);
    }
    Ok(())
}

/// 生产版本探针：运行 `<prefix>/usr/bin/zero-browser --version` 取版本段。
pub fn probe_prefix_version(prefix: &Path) -> Option<String> {
    let output = Command::new(prefix.join("usr/bin/zero-browser"))
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    // 形如 `ZeroBrowser 26.10.22 (commit <hash>)`，取第二段。
    stdout.split_whitespace().nth(1).map(str::to_string)
}

/// 执行一次更新检查与应用（FR-2/FR-3/FR-7）。
///
/// `extract` / `probe_version` 注入便于测试；生产路径分别传
/// [`dpkg_deb_extract`] 与 [`probe_prefix_version`]。任一失败时旧前缀保持
/// 活跃（翻转前失败只污染 staging，翻转后失败自动回滚）。
pub fn apply_update(
    source_dir: &Path,
    install_root: &InstallRoot,
    current_version: &str,
    extract: &dyn Fn(&Path, &Path) -> Result<(), UpdateError>,
    probe_version: &dyn Fn(&Path) -> Option<String>,
) -> Result<AppliedUpdate, UpdateError> {
    // 1. 读 manifest。
    let manifest_path = source_dir.join("manifest.json");
    let metadata = fs::metadata(&manifest_path).map_err(|_| UpdateError::ManifestInvalid)?;
    if metadata.len() > MAX_MANIFEST_BYTES {
        return Err(UpdateError::ManifestInvalid);
    }
    let text = fs::read_to_string(&manifest_path).map_err(|_| UpdateError::ManifestInvalid)?;
    let manifest = parse_manifest(&text)?;

    // 2. 版本序：相等/降级拒绝（compare 返回 candidate.cmp(current)：Greater 即候选更新）。
    match compare_versions(current_version, &manifest.version)? {
        std::cmp::Ordering::Equal => return Err(UpdateError::UpToDate),
        std::cmp::Ordering::Less => return Err(UpdateError::Downgrade),
        std::cmp::Ordering::Greater => {}
    }

    // 3. artifact 存在性（路径穿越已在 parse_manifest 拒绝）。
    let source_artifact = source_dir.join(&manifest.artifact);
    if !source_artifact.is_file() {
        return Err(UpdateError::ArtifactMissing);
    }

    // 4. 清空并重建 staging（上次中断残留一并清除）。
    let staging = install_root.staging_dir();
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|_| UpdateError::CopyFailed)?;
    }
    fs::create_dir_all(&staging).map_err(|_| UpdateError::CopyFailed)?;

    // 5. 复制 artifact 到 staging（复制中断允许残留，下次触发步骤 4 清空）。
    let staged_artifact = staging.join(&manifest.artifact);
    fs::copy(&source_artifact, &staged_artifact).map_err(|_| UpdateError::CopyFailed)?;

    // 6-9. 校验、解包、自检、翻转；复制完成后任何失败按 RFC §4.1 清空 staging。
    let result = verify_extract_flip(
        install_root,
        &manifest,
        &staging,
        &staged_artifact,
        extract,
        probe_version,
    );
    if result.is_err() && !matches!(result, Err(UpdateError::CopyFailed)) {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

/// 校验、解包、暂存自检与翻转（`apply_update` 的复制后阶段）。
fn verify_extract_flip(
    install_root: &InstallRoot,
    manifest: &UpdateManifest,
    staging: &Path,
    staged_artifact: &Path,
    extract: &dyn Fn(&Path, &Path) -> Result<(), UpdateError>,
    probe_version: &dyn Fn(&Path) -> Option<String>,
) -> Result<AppliedUpdate, UpdateError> {
    // 6. SHA256 校验。
    let actual = sha256_file(staged_artifact)?;
    if actual != manifest.sha256 {
        return Err(UpdateError::HashMismatch);
    }

    // 7. 解包到 staging/prefix。
    let staged_prefix = staging.join("prefix");
    extract(staged_artifact, &staged_prefix)?;
    if !staged_prefix.join("usr/bin/zero-browser").is_file() {
        return Err(UpdateError::ExtractFailed);
    }

    // 8. 暂存自检：版本必须与 manifest 一致。
    if probe_version(&staged_prefix).as_deref() != Some(manifest.version.as_str()) {
        return Err(UpdateError::StagingCheckFailed);
    }

    // 9. 翻转：staging/prefix → releases/<v>，current 原子指向新版本。
    let previous = install_root.active_version();
    install_root
        .flip_to(&manifest.version, &staged_prefix)
        .map_err(|error| {
            tracing::warn!("update flip failed, keeping old prefix active: {error}");
            UpdateError::FlipFailed
        })?;

    // 10. 健康检查：经 current 链接探测新版本。
    if probe_version(&install_root.current_prefix()).as_deref() != Some(manifest.version.as_str()) {
        // 自动回滚：翻回上一版本。
        if let Some(previous) = previous.clone() {
            let _ = install_root.repoint_current(&previous);
        }
        return Err(UpdateError::HealthCheckFailed);
    }

    // 成功：更新 rollback 指向应用前版本，清理 staging。
    if let Some(previous) = &previous {
        install_root.set_rollback(previous);
    }
    let _ = fs::remove_dir_all(staging);
    Ok(AppliedUpdate {
        version: manifest.version.clone(),
        previous_version: previous,
    })
}

/// 显式回滚：把 current 翻回 rollback 指向的版本（FR-3）。
pub fn rollback(
    install_root: &InstallRoot,
    probe_version: &dyn Fn(&Path) -> Option<String>,
) -> Result<String, UpdateError> {
    let target = install_root.rollback_target().ok_or(UpdateError::FlipFailed)?;
    install_root
        .repoint_current(&target)
        .map_err(|_| UpdateError::FlipFailed)?;
    if probe_version(&install_root.current_prefix()).is_none() {
        return Err(UpdateError::HealthCheckFailed);
    }
    Ok(target)
}

/// 校验 `YY.M.D` 版本格式与合法日期（闰年感知，与 product-version 同规则）。
fn validate_version(value: &str) -> Result<(), UpdateError> {
    let parts: Vec<_> = value
        .split('.')
        .map(|p| p.parse::<u16>())
        .collect::<Result<_, _>>()
        .map_err(|_| UpdateError::ManifestInvalid)?;
    if parts.len() != 3 || parts[0] > 99 || !(1..=12).contains(&parts[1]) || parts[2] == 0 {
        return Err(UpdateError::ManifestInvalid);
    }
    let days_in_month = match parts[1] {
        2 if parts[0] % 4 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if parts[2] > days_in_month {
        return Err(UpdateError::ManifestInvalid);
    }
    Ok(())
}

/// artifact 文件名安全校验：仅允许源目录内的普通文件名（NFR-4 防穿越）。
fn is_safe_artifact_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('.')
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains("..")
        && name != "manifest.json"
}

/// 安装根目录布局与翻转操作。
#[derive(Debug, Clone)]
pub struct InstallRoot {
    root: PathBuf,
}

impl InstallRoot {
    /// 基于安装根目录创建布局句柄。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// 版本目录根。
    pub fn releases_dir(&self) -> PathBuf {
        self.root.join("releases")
    }

    /// 暂存目录。
    pub fn staging_dir(&self) -> PathBuf {
        self.root.join("staging")
    }

    /// current 链接路径。
    fn current_link(&self) -> PathBuf {
        self.root.join("current")
    }

    /// rollback 链接路径。
    fn rollback_link(&self) -> PathBuf {
        self.root.join("rollback")
    }

    /// current 指向的版本前缀目录（供健康检查/启动使用）。
    pub fn current_prefix(&self) -> PathBuf {
        self.root.join("current")
    }

    /// 当前活跃版本名；current 缺失或悬空时 `None`。
    pub fn active_version(&self) -> Option<String> {
        fs::read_link(self.current_link())
            .ok()?
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }

    /// rollback 指向的版本名；缺失或悬空时 `None`。
    pub fn rollback_target(&self) -> Option<String> {
        fs::read_link(self.rollback_link())
            .ok()?
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }

    /// 把暂存前缀移入 releases 并原子翻转 current（调用方保证 version 合法）。
    fn flip_to(&self, version: &str, staged_prefix: &Path) -> Result<(), String> {
        #[cfg(unix)]
        {
            let releases = self.releases_dir();
            fs::create_dir_all(&releases).map_err(|error| format!("create releases failed: {error}"))?;
            let target = releases.join(version);
            if target.exists() {
                fs::remove_dir_all(&target).map_err(|error| format!("clear stale release failed: {error}"))?;
            }
            fs::rename(staged_prefix, &target).map_err(|error| format!("promote staging failed: {error}"))?;
            self.repoint_current(version)
        }
        #[cfg(not(unix))]
        {
            let _ = (version, staged_prefix);
            Err("update flip is not supported on this platform".to_string())
        }
    }

    /// 原子重指 current 链接到 `releases/<version>`（临时链接 + rename）。
    fn repoint_current(&self, version: &str) -> Result<(), String> {
        self.repoint_link(&self.current_link(), version)
    }

    /// 设置 rollback 链接（尽力而为，失败仅记日志）。
    fn set_rollback(&self, version: &str) {
        if let Err(error) = self.repoint_link(&self.rollback_link(), version) {
            tracing::warn!("set rollback link failed: {error}");
        }
    }

    /// 临时链接 + rename 的原子链接替换。链接目标用 `releases/<版本>` 相对
    /// 形式，整个安装根目录搬移后链接仍然有效。
    fn repoint_link(&self, link: &Path, version: &str) -> Result<(), String> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let temporary = link.with_extension("link-new");
            let _ = fs::remove_file(&temporary);
            symlink(Path::new("releases").join(version), &temporary)
                .map_err(|error| format!("create link failed: {error}"))?;
            fs::rename(&temporary, link).map_err(|error| format!("swap link failed: {error}"))?;
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = (link, version);
            Err("update flip is not supported on this platform".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用假解包器：把 artifact 文件内容（约定为版本号）写到
    /// `<dest>/usr/bin/zero-browser` 标记文件；artifact 名含 "broken" 时额外
    /// 写 BROKEN 标记模拟坏包（探针对其返回 None）。
    fn fake_extract(deb: &Path, dest: &Path) -> Result<(), UpdateError> {
        let version = fs::read_to_string(deb).map_err(|_| UpdateError::ExtractFailed)?;
        let bin_dir = dest.join("usr/bin");
        fs::create_dir_all(&bin_dir).map_err(|_| UpdateError::ExtractFailed)?;
        fs::write(bin_dir.join("zero-browser"), version).map_err(|_| UpdateError::ExtractFailed)?;
        if deb.file_name().unwrap().to_string_lossy().contains("broken") {
            fs::write(dest.join("BROKEN"), "").map_err(|_| UpdateError::ExtractFailed)?;
        }
        Ok(())
    }

    /// 测试用探针：读取假解包器写的标记文件；含 BROKEN 标记的返回 None。
    fn fake_probe(prefix: &Path) -> Option<String> {
        if prefix.join("BROKEN").exists() {
            return None;
        }
        fs::read_to_string(prefix.join("usr/bin/zero-browser")).ok()
    }

    /// 构造一个最小更新源：manifest.json + 假 artifact（内容任意，哈希真实计算）。
    fn make_source(root: &Path, version: &str, artifact_name: &str, body: &str) -> UpdateManifest {
        fs::create_dir_all(root).unwrap();
        fs::write(root.join(artifact_name), body).unwrap();
        let sha = {
            use sha2::Digest;
            let digest = sha2::Sha256::digest(body.as_bytes());
            digest.iter().map(|b| format!("{b:02x}")).collect::<String>()
        };
        let manifest = UpdateManifest {
            schema_version: 1,
            version: version.to_string(),
            artifact: artifact_name.to_string(),
            sha256: sha,
        };
        fs::write(root.join("manifest.json"), serde_json::to_string(&manifest).unwrap()).unwrap();
        manifest
    }

    fn unique_root(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("zw-update-{tag}-{}", std::process::id()))
    }

    #[test]
    fn parses_valid_manifest() {
        let text = r#"{"schema_version":1,"version":"26.10.23","artifact":"zero-browser_26.10.23_amd64.deb","sha256":"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"}"#;
        let manifest = parse_manifest(text).unwrap();
        assert_eq!(manifest.version, "26.10.23");
        assert_eq!(manifest.schema_version, 1);
    }

    #[test]
    fn rejects_malicious_and_malformed_manifests() {
        let good_artifact = "zero-browser.deb";
        let good_sha = "a".repeat(64);
        let cases = [
            r#"{"schema_version":2,"version":"26.10.23","artifact":"x.deb","sha256":""}"#,
            r#"{"schema_version":1,"version":"26.2.30","artifact":"x.deb","sha256":""}"#, // 2 月 30 日
            r#"{"schema_version":1,"version":"26.13.1","artifact":"x.deb","sha256":""}"#, // 13 月
            r#"{"schema_version":1,"version":"26.10.23","artifact":"../evil.deb","sha256":""}"#,
            r#"{"schema_version":1,"version":"26.10.23","artifact":"/etc/evil.deb","sha256":""}"#,
            r#"{"schema_version":1,"version":"26.10.23","artifact":"a\\b.deb","sha256":""}"#,
            r#"{"schema_version":1,"version":"26.10.23","artifact":"..deb","sha256":""}"#,
            r#"{"schema_version":1,"version":"26.10.23","artifact":"manifest.json","sha256":""}"#,
        ];
        for case in cases {
            assert!(parse_manifest(case).is_err(), "must reject: {case}");
        }
        // sha256 大写/长度不符单独校验（JSON 内嵌引号转义冗长）。
        let mut manifest = format!(
            r#"{{"schema_version":1,"version":"26.10.23","artifact":"{good_artifact}","sha256":"{}"}}"#,
            "A".repeat(64)
        );
        assert!(parse_manifest(&manifest).is_err());
        manifest = format!(
            r#"{{"schema_version":1,"version":"26.10.23","artifact":"{good_artifact}","sha256":"{good_sha}1"}}"#
        );
        assert!(parse_manifest(&manifest).is_err());
        // 截断 JSON。
        assert!(parse_manifest("{\"schema_version\":1,").is_err());
    }

    #[test]
    fn compares_versions_in_ymd_order() {
        // 语义：候选 > 当前 → Greater（apply_update 视 Less 为降级）。
        assert_eq!(
            compare_versions("26.10.22", "26.10.23").unwrap(),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_versions("26.10.22", "26.10.22").unwrap(),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_versions("26.10.22", "25.12.31").unwrap(),
            std::cmp::Ordering::Less
        );
        assert!(compare_versions("26.10.22", "bad").is_err());
    }

    #[test]
    fn applies_update_end_to_end_with_fake_extract() {
        let source = unique_root("source");
        let install = unique_root("install");
        make_source(&source, "26.10.23", "zero-browser_26.10.23.deb", "26.10.23");

        // 首次安装：current 尚不存在（previous None）。
        let root = InstallRoot::new(&install);
        let applied = apply_update(&source, &root, "26.10.22", &fake_extract, &fake_probe).unwrap();
        assert_eq!(applied.version, "26.10.23");
        assert_eq!(root.active_version(), Some("26.10.23".to_string()));
        assert_eq!(fake_probe(&root.current_prefix()).as_deref(), Some("26.10.23"));
        assert!(!root.staging_dir().exists(), "staging must be cleaned after success");

        // 再更新：rollback 指向 26.10.23，current 翻到 26.11.01。
        make_source(&source, "26.11.1", "zero-browser_26.11.1.deb", "26.11.1");
        let applied = apply_update(&source, &root, "26.10.23", &fake_extract, &fake_probe).unwrap();
        assert_eq!(applied.previous_version.as_deref(), Some("26.10.23"));
        assert_eq!(root.active_version(), Some("26.11.1".to_string()));
        assert_eq!(root.rollback_target(), Some("26.10.23".to_string()));

        // 显式回滚回到 26.10.23。
        let restored = rollback(&root, &fake_probe).unwrap();
        assert_eq!(restored, "26.10.23");
        assert_eq!(fake_probe(&root.current_prefix()).as_deref(), Some("26.10.23"));

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&install);
    }

    #[test]
    fn rejects_up_to_date_downgrade_missing_artifact_and_bad_hash() {
        let source = unique_root("reject-source");
        let install = unique_root("reject-install");
        let root = InstallRoot::new(&install);
        let extract = |_: &Path, _: &Path| -> Result<(), UpdateError> { panic!("must not extract") };

        make_source(&source, "26.10.22", "zero.deb", "same");
        assert_eq!(
            apply_update(&source, &root, "26.10.22", &extract, &fake_probe),
            Err(UpdateError::UpToDate)
        );
        assert_eq!(
            apply_update(&source, &root, "26.10.23", &extract, &fake_probe),
            Err(UpdateError::Downgrade)
        );

        make_source(&source, "26.10.24", "zero.deb", "new");
        fs::remove_file(source.join("zero.deb")).unwrap();
        assert_eq!(
            apply_update(&source, &root, "26.10.22", &extract, &fake_probe),
            Err(UpdateError::ArtifactMissing)
        );

        make_source(&source, "26.10.24", "zero.deb", "new");
        let manifest = parse_manifest(&fs::read_to_string(source.join("manifest.json")).unwrap()).unwrap();
        let tampered = UpdateManifest {
            sha256: "0".repeat(64),
            ..manifest
        };
        fs::write(source.join("manifest.json"), serde_json::to_string(&tampered).unwrap()).unwrap();
        assert_eq!(
            apply_update(&source, &root, "26.10.22", &extract, &fake_probe),
            Err(UpdateError::HashMismatch)
        );
        assert!(
            !root.staging_dir().exists(),
            "staging must be cleaned after hash mismatch"
        );

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&install);
    }

    #[test]
    fn broken_package_fails_staging_check_and_keeps_old_active() {
        let source = unique_root("broken-source");
        let install = unique_root("broken-install");
        let root = InstallRoot::new(&install);
        make_source(&source, "26.10.22", "zero-browser_26.10.22.deb", "26.10.22");

        // 先装一个旧版本。
        apply_update(&source, &root, "26.10.21", &fake_extract, &fake_probe).unwrap();

        // broken 包：解包后探针失败 → 暂存自检拒绝，旧版本保持活跃。
        make_source(&source, "26.10.23", "zero-browser_broken_26.10.23.deb", "26.10.23");
        assert_eq!(
            apply_update(&source, &root, "26.10.22", &fake_extract, &fake_probe),
            Err(UpdateError::StagingCheckFailed)
        );
        assert_eq!(root.active_version(), Some("26.10.22".to_string()));
        assert!(!root.staging_dir().exists());

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&install);
    }

    #[test]
    fn failed_health_check_auto_rolls_back_to_previous() {
        let source = unique_root("health-source");
        let install = unique_root("health-install");
        let root = InstallRoot::new(&install);
        make_source(&source, "26.10.22", "zero-browser_26.10.22.deb", "26.10.22");
        apply_update(&source, &root, "26.10.21", &fake_extract, &fake_probe).unwrap();

        // broken 包通过暂存自检的路径：让 BROKEN 标记只在经 current 访问时出现
        // 不可行——直接构造探针对暂存说谎、对 current 如实失败：
        let source2 = unique_root("health-source2");
        make_source(&source2, "26.10.23", "zero-browser_26.10.23.deb", "26.10.23");
        let lying_probe = |prefix: &Path| -> Option<String> {
            // 暂存路径在 staging/ 下：谎报 manifest 版本让其通过自检；
            // 其余路径（翻转后的 current）返回 None 模拟健康检查失败。
            if prefix.ends_with("staging/prefix") {
                return Some("26.10.23".to_string());
            }
            None
        };
        assert_eq!(
            apply_update(&source2, &root, "26.10.22", &fake_extract, &lying_probe),
            Err(UpdateError::HealthCheckFailed)
        );
        // 自动回滚：current 仍指旧版本且可探测。
        assert_eq!(root.active_version(), Some("26.10.22".to_string()));
        assert_eq!(fake_probe(&root.current_prefix()).as_deref(), Some("26.10.22"));

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&source2);
        let _ = fs::remove_dir_all(&install);
    }

    #[test]
    fn stale_staging_is_cleared_on_next_apply() {
        let source = unique_root("stale-source");
        let install = unique_root("stale-install");
        let root = InstallRoot::new(&install);
        make_source(&source, "26.10.23", "zero-browser_26.10.23.deb", "26.10.23");

        // 模拟上次中断残留：staging 内有半成品。
        let staging = root.staging_dir();
        fs::create_dir_all(staging.join("prefix/usr/bin")).unwrap();
        fs::write(staging.join("zero-browser_26.10.23.deb"), "stale-bytes").unwrap();

        let applied = apply_update(&source, &root, "26.10.22", &fake_extract, &fake_probe).unwrap();
        assert_eq!(applied.version, "26.10.23");
        assert_eq!(fake_probe(&root.current_prefix()).as_deref(), Some("26.10.23"));

        let _ = fs::remove_dir_all(&source);
        let _ = fs::remove_dir_all(&install);
    }
}
