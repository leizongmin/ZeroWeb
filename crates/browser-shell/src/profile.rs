//! 浏览器 profile 的平台无关持久化路径与原子写入。

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 单个 profile JSON 文件允许的最大字节数。
pub const MAX_PROFILE_FILE_BYTES: u64 = 1024 * 1024;

/// 一个浏览器 profile 内各类状态文件的路径。
#[derive(Debug, Clone)]
pub struct ProfilePaths {
    root: PathBuf,
}

impl ProfilePaths {
    /// 基于宿主提供的 profile 根目录创建路径集合。
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// 会话文件路径。
    pub fn session(&self) -> PathBuf {
        self.root.join("session.json")
    }

    /// 书签文件路径。
    pub fn bookmarks(&self) -> PathBuf {
        self.root.join("bookmarks.json")
    }

    /// 历史记录文件路径。
    pub fn history(&self) -> PathBuf {
        self.root.join("history.json")
    }

    /// 下载元数据文件路径。
    pub fn downloads(&self) -> PathBuf {
        self.root.join("downloads.json")
    }
}

/// 将 JSON 文本以临时文件、同步和原子替换的顺序写入目标。
pub(crate) fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("profile path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).map_err(|error| format!("create profile directory failed: {error}"))?;

    let temporary = path.with_extension("tmp");
    let mut file =
        File::create(&temporary).map_err(|error| format!("create profile temporary file failed: {error}"))?;
    file.write_all(content.as_bytes())
        .map_err(|error| format!("write profile temporary file failed: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync profile temporary file failed: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("replace profile file failed: {error}"))?;
    Ok(())
}

/// 读取受大小限制的 UTF-8 profile 文件。
pub(crate) fn read_profile(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() > MAX_PROFILE_FILE_BYTES {
        return None;
    }
    fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::{ProfilePaths, atomic_write, read_profile};

    #[test]
    fn profile_paths_stay_under_supplied_root() {
        let root = std::env::temp_dir().join("zero-browser-shell-profile-paths");
        let paths = ProfilePaths::new(&root);
        assert_eq!(paths.session(), root.join("session.json"));
        assert_eq!(paths.bookmarks(), root.join("bookmarks.json"));
        assert_eq!(paths.history(), root.join("history.json"));
        assert_eq!(paths.downloads(), root.join("downloads.json"));
    }

    #[test]
    fn atomic_write_replaces_prior_contents() {
        let root = std::env::temp_dir().join(format!("zero-browser-shell-profile-{}", std::process::id()));
        let path = root.join("state.json");
        atomic_write(&path, "first").unwrap();
        atomic_write(&path, "second").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "second");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn oversized_profile_file_is_rejected_before_reading() {
        let root = std::env::temp_dir().join(format!("zero-browser-shell-profile-size-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        std::fs::write(&path, vec![b'x'; super::MAX_PROFILE_FILE_BYTES as usize + 1]).unwrap();
        assert!(read_profile(&path).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
