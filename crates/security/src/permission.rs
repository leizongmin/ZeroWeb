//! # 权限模型
//!
//! Web API 权限请求、授权和持久化存储。
//! 支持摄像头、麦克风、地理位置、通知等 API 的权限管理。

use crate::origin::Origin;
use std::collections::HashMap;
use std::fmt;

/// Web API 权限名称。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PermissionName {
    /// 摄像头访问权限。
    Camera,
    /// 麦克风访问权限。
    Microphone,
    /// 地理位置 API 权限。
    Geolocation,
    /// 通知 API 权限。
    Notifications,
    /// 剪贴板读写权限。
    ClipboardRead,
    /// 剪贴板写入权限。
    ClipboardWrite,
    /// 全屏 API 权限。
    Fullscreen,
    /// 指针锁定权限。
    PointerLock,
    /// 屏幕录制权限。
    ScreenCapture,
    /// 后台同步权限。
    BackgroundSync,
    /// 持久化存储权限。
    PersistentStorage,
}

impl fmt::Display for PermissionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionName::Camera => write!(f, "camera"),
            PermissionName::Microphone => write!(f, "microphone"),
            PermissionName::Geolocation => write!(f, "geolocation"),
            PermissionName::Notifications => write!(f, "notifications"),
            PermissionName::ClipboardRead => write!(f, "clipboard-read"),
            PermissionName::ClipboardWrite => write!(f, "clipboard-write"),
            PermissionName::Fullscreen => write!(f, "fullscreen"),
            PermissionName::PointerLock => write!(f, "pointer-lock"),
            PermissionName::ScreenCapture => write!(f, "screen-capture"),
            PermissionName::BackgroundSync => write!(f, "background-sync"),
            PermissionName::PersistentStorage => write!(f, "persistent-storage"),
        }
    }
}

/// 权限状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// 用户已授予权限。
    Granted,
    /// 用户已拒绝权限。
    Denied,
    /// 用户尚未做出选择，下次请求时应弹出提示。
    Prompt,
}

impl fmt::Display for PermissionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermissionState::Granted => write!(f, "granted"),
            PermissionState::Denied => write!(f, "denied"),
            PermissionState::Prompt => write!(f, "prompt"),
        }
    }
}

/// 权限存储键（origin + permission 组合）。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PermissionKey {
    origin: String,
    permission: PermissionName,
}

impl PermissionKey {
    fn new(origin: &Origin, permission: PermissionName) -> Self {
        Self {
            origin: format_origin(origin),
            permission,
        }
    }
}

/// 将 Origin 格式化为字符串用于存储键。
fn format_origin(origin: &Origin) -> String {
    format!("{}://{}:{}", origin.scheme, origin.host, origin.port)
}

/// 权限条目，记录状态和元数据。
#[derive(Debug, Clone)]
struct PermissionEntry {
    /// 当前权限状态。
    state: PermissionState,
    /// 权限被授予或拒绝的时间戳（UNIX 时间戳，秒）。
    #[allow(dead_code)] // 预留给未来过期策略
    timestamp: u64,
}

/// 权限管理器。
///
/// 管理所有源的 Web API 权限状态，支持查询、请求、撤销操作。
/// 权限决策按 origin 隔离存储。
#[derive(Debug, Clone)]
pub struct PermissionManager {
    /// 权限存储（origin+permission → entry）。
    permissions: HashMap<PermissionKey, PermissionEntry>,
}

impl PermissionManager {
    /// 创建新的空权限管理器。
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }

    /// 查询指定源的权限状态。
    ///
    /// 未记录的权限返回 `Prompt` 状态。
    pub fn query(&self, origin: &Origin, permission: PermissionName) -> PermissionState {
        let key = PermissionKey::new(origin, permission);
        self.permissions
            .get(&key)
            .map(|e| e.state)
            .unwrap_or(PermissionState::Prompt)
    }

    /// 请求权限（模拟用户授权）。
    ///
    /// 调用此方法表示用户已选择授予该权限。
    /// 返回更新后的权限状态。
    pub fn grant(&mut self, origin: &Origin, permission: PermissionName, timestamp: u64) -> PermissionState {
        let key = PermissionKey::new(origin, permission);
        self.permissions.insert(key, PermissionEntry {
            state: PermissionState::Granted,
            timestamp,
        });
        PermissionState::Granted
    }

    /// 拒绝权限（模拟用户拒绝）。
    ///
    /// 调用此方法表示用户已选择拒绝该权限。
    /// 返回更新后的权限状态。
    pub fn deny(&mut self, origin: &Origin, permission: PermissionName, timestamp: u64) -> PermissionState {
        let key = PermissionKey::new(origin, permission);
        self.permissions.insert(key, PermissionEntry {
            state: PermissionState::Denied,
            timestamp,
        });
        PermissionState::Denied
    }

    /// 撤销权限（重置为 Prompt 状态）。
    ///
    /// 用于用户在设置页面手动撤销已授权的权限。
    pub fn revoke(&mut self, origin: &Origin, permission: PermissionName) {
        let key = PermissionKey::new(origin, permission);
        self.permissions.remove(&key);
    }

    /// 撤销指定源的所有权限。
    ///
    /// 用于清除站点数据时一并清除权限。
    pub fn revoke_all_for_origin(&mut self, origin: &Origin) {
        let origin_str = format_origin(origin);
        self.permissions.retain(|k, _| k.origin != origin_str);
    }

    /// 获取指定源的所有权限状态列表。
    ///
    /// 返回 (权限名称, 状态) 对的列表。
    pub fn get_all_for_origin(&self, origin: &Origin) -> Vec<(PermissionName, PermissionState)> {
        let origin_str = format_origin(origin);
        self.permissions
            .iter()
            .filter(|(k, _)| k.origin == origin_str)
            .map(|(k, v)| (k.permission.clone(), v.state))
            .collect()
    }

    /// 检查权限是否已授予。
    pub fn is_granted(&self, origin: &Origin, permission: PermissionName) -> bool {
        self.query(origin, permission) == PermissionState::Granted
    }

    /// 返回已存储的权限条目数量。
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// 检查是否没有任何已存储的权限。
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 权限请求结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequestResult {
    /// 请求的权限名称。
    pub permission: PermissionName,
    /// 最终权限状态。
    pub state: PermissionState,
}

/// 根据权限状态决定是否允许 API 调用。
///
/// 返回 true 仅当权限状态为 Granted。
/// Prompt 和 Denied 状态均返回 false（调用方应在 Prompt 时弹出提示）。
pub fn is_permission_allowed(state: PermissionState) -> bool {
    state == PermissionState::Granted
}

/// 根据源的权限配置和 API 需要的权限，决定是否应弹出权限提示。
///
/// 仅在状态为 Prompt 时返回 true。
pub fn should_prompt_user(state: PermissionState) -> bool {
    state == PermissionState::Prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_origin() -> Origin {
        Origin::parse("https://example.com").unwrap()
    }

    fn other_origin() -> Origin {
        Origin::parse("https://other.com").unwrap()
    }

    #[test]
    fn test_permission_name_display() {
        assert_eq!(PermissionName::Camera.to_string(), "camera");
        assert_eq!(PermissionName::Geolocation.to_string(), "geolocation");
        assert_eq!(PermissionName::Notifications.to_string(), "notifications");
    }

    #[test]
    fn test_permission_state_display() {
        assert_eq!(PermissionState::Granted.to_string(), "granted");
        assert_eq!(PermissionState::Denied.to_string(), "denied");
        assert_eq!(PermissionState::Prompt.to_string(), "prompt");
    }

    #[test]
    fn test_query_default_is_prompt() {
        let mgr = PermissionManager::new();
        assert_eq!(
            mgr.query(&test_origin(), PermissionName::Camera),
            PermissionState::Prompt
        );
    }

    #[test]
    fn test_grant_permission() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();

        let state = mgr.grant(&origin, PermissionName::Camera, 1000);
        assert_eq!(state, PermissionState::Granted);
        assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Granted);
        assert!(mgr.is_granted(&origin, PermissionName::Camera));
    }

    #[test]
    fn test_deny_permission() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();

        let state = mgr.deny(&origin, PermissionName::Microphone, 1000);
        assert_eq!(state, PermissionState::Denied);
        assert_eq!(mgr.query(&origin, PermissionName::Microphone), PermissionState::Denied);
        assert!(!mgr.is_granted(&origin, PermissionName::Microphone));
    }

    #[test]
    fn test_revoke_permission() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();

        mgr.grant(&origin, PermissionName::Geolocation, 1000);
        assert_eq!(mgr.query(&origin, PermissionName::Geolocation), PermissionState::Granted);

        mgr.revoke(&origin, PermissionName::Geolocation);
        assert_eq!(mgr.query(&origin, PermissionName::Geolocation), PermissionState::Prompt);
    }

    #[test]
    fn test_revoke_nonexistent_is_noop() {
        let mut mgr = PermissionManager::new();
        mgr.revoke(&test_origin(), PermissionName::Camera);
        assert_eq!(mgr.len(), 0);
    }

    #[test]
    fn test_revoke_all_for_origin() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();
        let other = other_origin();

        mgr.grant(&origin, PermissionName::Camera, 1000);
        mgr.grant(&origin, PermissionName::Microphone, 1000);
        mgr.grant(&other, PermissionName::Geolocation, 1000);

        mgr.revoke_all_for_origin(&origin);
        assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Prompt);
        assert_eq!(mgr.query(&origin, PermissionName::Microphone), PermissionState::Prompt);
        // 其他 origin 不受影响
        assert_eq!(mgr.query(&other, PermissionName::Geolocation), PermissionState::Granted);
    }

    #[test]
    fn test_permissions_isolated_by_origin() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();
        let other = other_origin();

        mgr.grant(&origin, PermissionName::Camera, 1000);
        assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Granted);
        assert_eq!(mgr.query(&other, PermissionName::Camera), PermissionState::Prompt);
    }

    #[test]
    fn test_get_all_for_origin() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();
        let other = other_origin();

        mgr.grant(&origin, PermissionName::Camera, 1000);
        mgr.deny(&origin, PermissionName::Microphone, 2000);
        mgr.grant(&other, PermissionName::Geolocation, 3000);

        let permissions = mgr.get_all_for_origin(&origin);
        assert_eq!(permissions.len(), 2);

        let camera = permissions.iter().find(|(p, _)| *p == PermissionName::Camera);
        assert_eq!(camera.map(|(_, s)| *s), Some(PermissionState::Granted));

        let mic = permissions.iter().find(|(p, _)| *p == PermissionName::Microphone);
        assert_eq!(mic.map(|(_, s)| *s), Some(PermissionState::Denied));
    }

    #[test]
    fn test_overwrite_permission() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();

        mgr.grant(&origin, PermissionName::Camera, 1000);
        assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Granted);

        mgr.deny(&origin, PermissionName::Camera, 2000);
        assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Denied);
    }

    #[test]
    fn test_is_permission_allowed() {
        assert!(is_permission_allowed(PermissionState::Granted));
        assert!(!is_permission_allowed(PermissionState::Denied));
        assert!(!is_permission_allowed(PermissionState::Prompt));
    }

    #[test]
    fn test_should_prompt_user() {
        assert!(should_prompt_user(PermissionState::Prompt));
        assert!(!should_prompt_user(PermissionState::Granted));
        assert!(!should_prompt_user(PermissionState::Denied));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut mgr = PermissionManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);

        mgr.grant(&test_origin(), PermissionName::Camera, 1000);
        assert!(!mgr.is_empty());
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn test_default_trait() {
        let mgr = PermissionManager::default();
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_all_permission_names_display() {
        // 确保所有权限名称都有合理的字符串表示
        let names = [
            PermissionName::Camera,
            PermissionName::Microphone,
            PermissionName::Geolocation,
            PermissionName::Notifications,
            PermissionName::ClipboardRead,
            PermissionName::ClipboardWrite,
            PermissionName::Fullscreen,
            PermissionName::PointerLock,
            PermissionName::ScreenCapture,
            PermissionName::BackgroundSync,
            PermissionName::PersistentStorage,
        ];
        for name in &names {
            let s = name.to_string();
            assert!(!s.is_empty(), "PermissionName::{:?} 显示为空字符串", name);
            assert!(!s.contains(' '), "PermissionName::{:?} 包含空格", name);
        }
    }

    #[test]
    fn test_multiple_permissions_same_origin() {
        let mut mgr = PermissionManager::new();
        let origin = test_origin();

        mgr.grant(&origin, PermissionName::Camera, 1000);
        mgr.grant(&origin, PermissionName::Microphone, 1000);
        mgr.deny(&origin, PermissionName::Geolocation, 1000);

        assert_eq!(mgr.len(), 3);
        assert!(mgr.is_granted(&origin, PermissionName::Camera));
        assert!(mgr.is_granted(&origin, PermissionName::Microphone));
        assert!(!mgr.is_granted(&origin, PermissionName::Geolocation));
    }

    #[test]
    fn test_revoke_all_preserves_other_origins() {
        let mut mgr = PermissionManager::new();
        let o1 = test_origin();
        let o2 = other_origin();

        mgr.grant(&o1, PermissionName::Camera, 1000);
        mgr.grant(&o1, PermissionName::Notifications, 1000);
        mgr.grant(&o2, PermissionName::Camera, 1000);
        mgr.grant(&o2, PermissionName::Geolocation, 1000);

        mgr.revoke_all_for_origin(&o1);
        assert_eq!(mgr.len(), 2);
        assert!(mgr.is_granted(&o2, PermissionName::Camera));
        assert!(mgr.is_granted(&o2, PermissionName::Geolocation));
    }
}
