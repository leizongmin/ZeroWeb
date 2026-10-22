//! 浏览器会话持久化 — 保存和恢复标签页状态。
//!
//! 在浏览器关闭时保存当前打开的标签页（URL、标题、导航历史），
//! 在下次启动时恢复之前的会话状态。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::profile::{atomic_write, read_profile};

const MAX_TABS: usize = 10_000;
const MAX_HISTORY_PER_TAB: usize = 10_000;
const MAX_TEXT_BYTES: usize = 16 * 1024;

/// FR-6 崩溃循环守卫：同一 tab 累计崩溃达到该次数后，下次恢复不再自动加载
/// 该 tab 的崩溃页（置空 about:blank）。RFC P5 待决终值，集中定义便于调整。
pub const CRASH_LOOP_SKIPPED_THRESHOLD: u32 = 2;
/// FR-6 观察窗：崩溃或跳过恢复后 tab 存活该时长且无新崩溃，计数清零。
pub const CRASH_LOOP_OBSERVATION_WINDOW_SECS: u64 = 10;

/// 可序列化的浏览器会话快照。
///
/// 包含关闭时的所有标签页状态，用于下次启动时恢复。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// 保存的标签页列表。
    pub tabs: Vec<TabSnapshot>,
    /// 活跃标签页索引（None 表示无活跃标签）。
    pub active_tab_index: Option<usize>,
    /// 是否为干净退出。
    ///
    /// 启动恢复检查完成后即以 `false` 落盘（运行中标记），干净退出时随会话
    /// 写回 `true`；下次启动读到 `false` 表示上次异常退出（崩溃/被杀），
    /// 按崩溃恢复处理（Chrome `exited_cleanly` 同款）。缺省 `false`：旧版本
    /// 写入的会话文件视为未正常关闭。
    #[serde(default)]
    pub clean_exit: bool,
}

/// 单个标签页的快照。
///
/// 保存标签页的核心状态，足以在下次启动时恢复。
/// 标签页 ID 不保存（恢复时重新生成）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSnapshot {
    /// 当前 URL。
    pub url: Option<String>,
    /// 页面标题。
    pub title: Option<String>,
    /// 导航历史。
    pub history: Vec<NavigationSnapshot>,
    /// 导航历史中的当前位置索引。
    pub history_index: usize,
    /// FR-6 崩溃循环守卫：该 tab 的累计崩溃次数（本轮运行内自增，恢复跳过
    /// 判定用）。缺省 0：旧 session.json 无此字段照常加载。
    #[serde(default)]
    pub crash_count: u32,
}

impl TabSnapshot {
    /// 该 tab 是否达到崩溃循环阈值、恢复时应跳过自动加载。
    pub fn is_crash_loop_skipped(&self) -> bool {
        self.crash_count >= CRASH_LOOP_SKIPPED_THRESHOLD
    }
}

/// 导航历史条目快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSnapshot {
    /// URL。
    pub url: String,
    /// 页面标题。
    pub title: Option<String>,
}

impl SessionState {
    /// 创建空的会话状态。
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: None,
            clean_exit: false,
        }
    }

    /// 从标签页信息构建会话状态。
    ///
    /// 接受标签页迭代器和活跃标签索引；`clean_exit` 由调用方按退出语义覆盖
    /// （干净退出持久化路径写 `true`）。
    pub fn from_tabs(tabs: impl Iterator<Item = TabInfo>, active_tab_index: Option<usize>) -> Self {
        Self {
            tabs: tabs
                .map(|t| TabSnapshot {
                    url: t.url,
                    title: t.title,
                    history: t.history,
                    history_index: t.history_index,
                    crash_count: t.crash_count,
                })
                .collect(),
            active_tab_index,
            clean_exit: false,
        }
    }

    /// 返回会话文件的默认路径。
    ///
    /// 遵循 XDG 规范：`~/.config/zeroweb/session.json`；`ZERO_CONFIG_DIR`
    /// （`--profile`）优先。
    pub fn default_session_path() -> PathBuf {
        crate::config_base_dir().join("zeroweb").join("session.json")
    }

    /// 从 JSON 文件加载会话状态。
    ///
    /// 如果文件不存在或解析失败，返回 `None`。
    pub fn load(path: &Path) -> Option<Self> {
        let content = read_profile(path)?;
        let session = serde_json::from_str::<Self>(&content).ok()?;
        session.is_valid().then_some(session)
    }

    /// 从默认路径加载会话状态。
    pub fn load_default() -> Option<Self> {
        Self::load(&Self::default_session_path())
    }

    /// 将会话状态保存到 JSON 文件。
    ///
    /// 自动创建父目录。
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize session: {e}"))?;
        atomic_write(path, &json)
    }

    /// 保存到默认路径。
    pub fn save_default(&self) -> Result<(), String> {
        self.save(&Self::default_session_path())
    }

    /// 标签页数量。
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// 是否为空会话。
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    fn is_valid(&self) -> bool {
        self.tabs.len() <= MAX_TABS
            && self.active_tab_index.is_none_or(|index| index < self.tabs.len())
            && self.tabs.iter().all(|tab| {
                tab.url.as_deref().is_none_or(|url| url.len() <= MAX_TEXT_BYTES)
                    && tab.title.as_deref().is_none_or(|title| title.len() <= MAX_TEXT_BYTES)
                    && tab.history.len() <= MAX_HISTORY_PER_TAB
                    && (tab.history.is_empty() || tab.history_index < tab.history.len())
                    && tab.history.iter().all(|entry| {
                        entry.url.len() <= MAX_TEXT_BYTES
                            && entry.title.as_deref().is_none_or(|title| title.len() <= MAX_TEXT_BYTES)
                    })
            })
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

/// 标签页信息（用于构建会话快照）。
///
/// 由调用方从 Tab 结构提取。
pub struct TabInfo {
    /// 当前 URL。
    pub url: Option<String>,
    /// 页面标题。
    pub title: Option<String>,
    /// 导航历史。
    pub history: Vec<NavigationSnapshot>,
    /// 导航历史中的当前位置索引。
    pub history_index: usize,
    /// 崩溃循环守卫计数（见 [`TabSnapshot::crash_count`]）。
    pub crash_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_new() {
        let session = SessionState::new();
        assert!(session.is_empty());
        assert_eq!(session.tab_count(), 0);
        assert_eq!(session.active_tab_index, None);
    }

    #[test]
    fn load_rejects_a_session_with_an_oversized_url() {
        let root = std::env::temp_dir().join(format!("zero-browser-shell-session-size-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("session.json");
        let oversized_url = format!("https://example.com/{}", "x".repeat(MAX_TEXT_BYTES));
        std::fs::write(&path, format!(r#"{{"tabs":[{{"url":"{oversized_url}","title":null,"history":[],"history_index":0}}],"active_tab_index":0}}"#)).unwrap();
        assert!(SessionState::load(&path).is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_session_state_default() {
        let session = SessionState::default();
        assert!(session.is_empty());
    }

    #[test]
    fn test_session_state_from_tabs() {
        let tabs = vec![
            TabInfo {
                url: Some("https://example.com".to_string()),
                title: Some("Example".to_string()),
                history: vec![NavigationSnapshot {
                    url: "https://example.com".to_string(),
                    title: Some("Example".to_string()),
                }],
                history_index: 0,
                crash_count: 0,
            },
            TabInfo {
                url: Some("https://github.com".to_string()),
                title: Some("GitHub".to_string()),
                history: vec![NavigationSnapshot {
                    url: "https://github.com".to_string(),
                    title: Some("GitHub".to_string()),
                }],
                history_index: 0,
                crash_count: 0,
            },
        ];
        let session = SessionState::from_tabs(tabs.into_iter(), Some(1));
        assert_eq!(session.tab_count(), 2);
        assert_eq!(session.active_tab_index, Some(1));
        assert_eq!(session.tabs[0].url, Some("https://example.com".to_string()));
        assert_eq!(session.tabs[1].title, Some("GitHub".to_string()));
    }

    #[test]
    fn test_session_state_serialization() {
        let session = SessionState {
            tabs: vec![TabSnapshot {
                url: Some("https://example.com".to_string()),
                title: Some("Example".to_string()),
                history: vec![NavigationSnapshot {
                    url: "https://example.com".to_string(),
                    title: Some("Example".to_string()),
                }],
                history_index: 0,
                crash_count: 0,
            }],
            active_tab_index: Some(0),
            clean_exit: false,
        };
        let json = serde_json::to_string_pretty(&session).unwrap();
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tab_count(), 1);
        assert_eq!(deserialized.active_tab_index, Some(0));
        assert_eq!(deserialized.tabs[0].url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_session_save_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("zeroweb_test_session-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("session.json");

        let session = SessionState {
            tabs: vec![
                TabSnapshot {
                    url: Some("https://example.com".to_string()),
                    title: Some("Example".to_string()),
                    history: vec![NavigationSnapshot {
                        url: "https://example.com".to_string(),
                        title: Some("Example".to_string()),
                    }],
                    history_index: 0,
                    crash_count: 0,
                },
                TabSnapshot {
                    url: Some("https://github.com".to_string()),
                    title: None,
                    history: vec![NavigationSnapshot {
                        url: "https://github.com".to_string(),
                        title: None,
                    }],
                    history_index: 0,
                    crash_count: 0,
                },
            ],
            active_tab_index: Some(1),
            clean_exit: false,
        };

        session.save(&path).expect("save should succeed");
        let loaded = SessionState::load(&path).expect("load should succeed");

        assert_eq!(loaded.tab_count(), 2);
        assert_eq!(loaded.active_tab_index, Some(1));
        assert_eq!(loaded.tabs[0].url, Some("https://example.com".to_string()));
        assert_eq!(loaded.tabs[0].title, Some("Example".to_string()));
        assert_eq!(loaded.tabs[1].url, Some("https://github.com".to_string()));
        assert_eq!(loaded.tabs[1].title, None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_load_missing_file() {
        let loaded = SessionState::load(Path::new("/nonexistent/path/session.json"));
        assert!(loaded.is_none());
    }

    #[test]
    fn test_session_load_invalid_json() {
        let dir = std::env::temp_dir().join(format!("zeroweb_test_session_invalid-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("session.json");
        std::fs::write(&path, "not valid json").unwrap();

        let loaded = SessionState::load(&path);
        assert!(loaded.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_session_empty_tabs() {
        let session = SessionState::from_tabs(std::iter::empty(), None);
        assert!(session.is_empty());
        assert_eq!(session.active_tab_index, None);
    }

    #[test]
    fn test_session_tab_with_navigation_history() {
        let session = SessionState {
            tabs: vec![TabSnapshot {
                url: Some("https://c.com".to_string()),
                title: Some("C".to_string()),
                history: vec![
                    NavigationSnapshot {
                        url: "https://a.com".to_string(),
                        title: Some("A".to_string()),
                    },
                    NavigationSnapshot {
                        url: "https://b.com".to_string(),
                        title: Some("B".to_string()),
                    },
                    NavigationSnapshot {
                        url: "https://c.com".to_string(),
                        title: Some("C".to_string()),
                    },
                ],
                history_index: 2,
                crash_count: 0,
            }],
            active_tab_index: Some(0),
            clean_exit: false,
        };
        let json = serde_json::to_string(&session).unwrap();
        let loaded: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.tabs[0].history.len(), 3);
        assert_eq!(loaded.tabs[0].history_index, 2);
        assert_eq!(loaded.tabs[0].history[1].url, "https://b.com");
    }

    #[test]
    fn test_session_save_creates_parent_dirs() {
        let dir = std::env::temp_dir()
            .join(format!("zeroweb_test_session_deep-{}", std::process::id()))
            .join("a")
            .join("b");
        let path = dir.join("session.json");

        let session = SessionState::new();
        session.save(&path).expect("save should create parent dirs");
        assert!(path.exists());

        let _ = std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("zeroweb_test_session_deep-{}", std::process::id())),
        );
    }

    #[test]
    fn session_without_crash_count_field_loads_with_zero() {
        // FR-6：旧 session.json 无 crash_count 字段照常加载，缺省 0。
        let dir = std::env::temp_dir().join(format!("zeroweb_test_session_legacy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("session.json");
        std::fs::write(
            &path,
            r#"{"tabs":[{"url":"https://example.com/","title":null,"history":[],"history_index":0}],"active_tab_index":0,"clean_exit":true}"#,
        )
        .unwrap();
        let session = SessionState::load(&path).expect("legacy session should load");
        assert_eq!(session.tabs[0].crash_count, 0);
        assert!(!session.tabs[0].is_crash_loop_skipped());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn crash_count_round_trips_and_reaches_skip_threshold() {
        let tabs = vec![TabInfo {
            url: Some("https://crash.example/".to_string()),
            title: None,
            history: vec![],
            history_index: 0,
            crash_count: CRASH_LOOP_SKIPPED_THRESHOLD,
        }];
        let session = SessionState::from_tabs(tabs.into_iter(), Some(0));
        assert!(session.tabs[0].is_crash_loop_skipped());
        let dir = std::env::temp_dir().join(format!("zeroweb_test_session_crashcount-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("session.json");
        session.save(&path).expect("save should succeed");
        let loaded = SessionState::load(&path).expect("load should succeed");
        assert_eq!(loaded.tabs[0].crash_count, CRASH_LOOP_SKIPPED_THRESHOLD);
        assert!(loaded.tabs[0].is_crash_loop_skipped());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_session_active_index_beyond_tabs() {
        // active_tab_index 不在有效范围时不应 panic
        let session = SessionState {
            tabs: vec![TabSnapshot {
                url: Some("https://example.com".to_string()),
                title: None,
                history: vec![],
                history_index: 0,
                crash_count: 0,
            }],
            active_tab_index: Some(5), // 超出范围
            clean_exit: false,
        };
        let json = serde_json::to_string(&session).unwrap();
        let loaded: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.active_tab_index, Some(5)); // 保留原始值
    }
}
