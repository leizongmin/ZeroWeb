//! Service Worker 注册表 — 管理页面级 Service Worker 的注册、生命周期和 Fetch 拦截。
//!
//! 本模块实现 Service Worker 的核心数据模型：
//! - [`ServiceWorkerRegistration`] — 单个 SW 注册记录
//! - [`ServiceWorkerRegistry`] — 按 origin 管理所有注册
//! - [`ServiceWorkerState`] — SW 生命周期状态机
//! - [`FetchInterceptResult`] — Fetch 拦截结果
//!
//! ## 生命周期
//!
//! ```text
//! Registered → Installing → Installed → Activating → Activated → Redundant
//! ```
//!
//! ## Fetch 拦截
//!
//! 当一个 origin 下有活跃的 Service Worker 时，该 origin 的 HTTP 请求会先经过 SW 的
//! fetch 事件处理器。SW 可以：
//! - 从 Cache API 返回缓存的响应
//! - 修改请求后转发到网络
//! - 合成一个新的响应
//! - 让请求直接通过到网络（pass-through）

use crate::cache_api::{CacheRequest, CacheResponse, CacheStorage};
use std::collections::HashMap;

/// 从 URL 中提取路径部分。
///
/// - `"https://example.com/app/page?q=1"` → `"/app/page"`
/// - `"/app/page"` → `"/app/page"`
fn extract_path(url: &str) -> &str {
    // 如果是绝对路径（以 / 开头），直接返回
    if url.starts_with('/') {
        let end = url.find(['?', '#']).unwrap_or(url.len());
        return &url[..end];
    }
    // 如果是完整 URL，跳过 scheme://host 部分
    let after_scheme = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
        .unwrap_or(url);
    if let Some(slash_pos) = after_scheme.find('/') {
        let path = &after_scheme[slash_pos..];
        let end = path.find(['?', '#']).unwrap_or(path.len());
        return &path[..end];
    }
    url
}

/// Service Worker 生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceWorkerState {
    /// 已注册，等待安装。
    Registered,
    /// 正在执行 install 事件处理器。
    Installing,
    /// install 完成，等待激活。
    Installed,
    /// 正在执行 activate 事件处理器。
    Activating,
    /// 已激活，可以拦截 fetch 请求。
    Activated,
    /// 已废弃（被新版本替换或注销）。
    Redundant,
}

impl std::fmt::Display for ServiceWorkerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceWorkerState::Registered => write!(f, "registered"),
            ServiceWorkerState::Installing => write!(f, "installing"),
            ServiceWorkerState::Installed => write!(f, "installed"),
            ServiceWorkerState::Activating => write!(f, "activating"),
            ServiceWorkerState::Activated => write!(f, "activated"),
            ServiceWorkerState::Redundant => write!(f, "redundant"),
        }
    }
}

/// Service Worker script update HTTP cache policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ServiceWorkerUpdateViaCache {
    /// Main script bypasses cache; imported scripts may use cache.
    #[default]
    Imports,
    /// Main and imported scripts may use cache.
    All,
    /// Main and imported scripts bypass cache.
    None,
}

/// 单个 Service Worker 注册记录。
#[derive(Debug, Clone)]
pub struct ServiceWorkerRegistration {
    /// 注册的唯一 ID。
    pub id: u64,
    /// SW 脚本的 URL。
    pub script_url: String,
    /// SW 的作用域 URL 前缀。
    pub scope: String,
    /// 注册的 origin（scheme + host + port）。
    pub origin: String,
    /// Script update HTTP cache policy.
    pub update_via_cache: ServiceWorkerUpdateViaCache,
    /// 当前状态。
    pub state: ServiceWorkerState,
    /// SW 脚本内容（从 script_url 获取）。
    pub script_content: Option<String>,
    /// 关联的 Cache Storage（SW 可以访问 `caches` 全局）。
    pub cache_storage: CacheStorage,
}

impl ServiceWorkerRegistration {
    /// 创建新的注册。
    pub fn new(id: u64, script_url: &str, scope: &str, origin: &str) -> Self {
        Self {
            id,
            script_url: script_url.to_string(),
            scope: scope.to_string(),
            origin: origin.to_string(),
            update_via_cache: ServiceWorkerUpdateViaCache::Imports,
            state: ServiceWorkerState::Registered,
            script_content: None,
            cache_storage: CacheStorage::new(),
        }
    }

    /// 检查给定的 URL 是否在 SW 的作用域内。
    ///
    /// 支持：
    /// - 绝对路径（如 `/app/page.html`）
    /// - 完整 URL（如 `https://example.com/app/page.html`）
    pub fn is_in_scope(&self, url: &str) -> bool {
        // 如果 scope 以 / 开头，从 URL 中提取路径部分进行比较
        if self.scope.starts_with('/') {
            let path = extract_path(url);
            return path.starts_with(&self.scope);
        }
        // 否则做前缀匹配（完整 URL scope）
        url.starts_with(&self.scope)
    }

    /// 检查 SW 是否处于活跃状态（可以拦截 fetch）。
    pub fn is_active(&self) -> bool {
        self.state == ServiceWorkerState::Activated
    }

    /// 推进到下一个生命周期状态。
    ///
    /// 返回 `true` 表示成功推进，`false` 表示无法推进。
    pub fn advance_state(&mut self) -> bool {
        let next = match self.state {
            ServiceWorkerState::Registered => Some(ServiceWorkerState::Installing),
            ServiceWorkerState::Installing => Some(ServiceWorkerState::Installed),
            ServiceWorkerState::Installed => Some(ServiceWorkerState::Activating),
            ServiceWorkerState::Activating => Some(ServiceWorkerState::Activated),
            ServiceWorkerState::Activated => None, // 已是终态
            ServiceWorkerState::Redundant => None, // 已废弃
        };
        if let Some(next) = next {
            self.state = next;
            true
        } else {
            false
        }
    }

    /// 将 SW 标记为废弃。
    pub fn mark_redundant(&mut self) {
        self.state = ServiceWorkerState::Redundant;
    }

    /// 尝试从 Cache Storage 匹配请求。
    ///
    /// 返回 `Some(response)` 如果找到了缓存的响应。
    pub fn match_cached(&self, request: &CacheRequest) -> Option<CacheResponse> {
        self.cache_storage.match_request(request).cloned()
    }
}

/// Fetch 拦截结果。
#[derive(Debug, Clone)]
pub enum FetchInterceptResult {
    /// SW 返回了缓存的响应。
    Cached(CacheResponse),
    /// SW 返回了合成的响应。
    Responded(CacheResponse),
    /// SW 未拦截，请求应直接通过到网络。
    PassThrough,
    /// 没有 SW 注册匹配此请求。
    NoWorker,
    /// SW 处理出错。
    Error(String),
}

/// Service Worker 注册表 — 按 origin 管理所有 Service Worker 注册。
///
/// 每个 origin 最多有一个活跃的 Service Worker。新注册同一 scope 的 SW 会
/// 先进入 `Installing` 状态，成功后替换旧的 SW。
#[derive(Debug, Clone, Default)]
pub struct ServiceWorkerRegistry {
    /// 所有注册，按 ID 索引。
    registrations: HashMap<u64, ServiceWorkerRegistration>,
    /// origin → 活跃的注册 ID 映射。
    active_by_origin: HashMap<String, u64>,
    /// 下一个可用的注册 ID。
    next_id: u64,
}

impl ServiceWorkerRegistry {
    /// 创建空的注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个新的 Service Worker。
    ///
    /// 如果该 origin 已有活跃的 SW，新 SW 会进入 `Registered` 状态等待安装。
    /// 返回新注册的 ID。
    pub fn register(&mut self, script_url: &str, scope: &str, origin: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let registration = ServiceWorkerRegistration::new(id, script_url, scope, origin);
        self.registrations.insert(id, registration);

        id
    }

    /// 获取指定 ID 的注册。
    pub fn get(&self, id: u64) -> Option<&ServiceWorkerRegistration> {
        self.registrations.get(&id)
    }

    /// 获取指定 ID 的注册（可变引用）。
    pub fn get_mut(&mut self, id: u64) -> Option<&mut ServiceWorkerRegistration> {
        self.registrations.get_mut(&id)
    }

    /// 获取指定 origin 的活跃 Service Worker。
    pub fn get_active(&self, origin: &str) -> Option<&ServiceWorkerRegistration> {
        self.active_by_origin
            .get(origin)
            .and_then(|id| self.registrations.get(id))
    }

    /// 获取指定 origin 的活跃 Service Worker（可变引用）。
    pub fn get_active_mut(&mut self, origin: &str) -> Option<&mut ServiceWorkerRegistration> {
        if let Some(&id) = self.active_by_origin.get(origin) {
            self.registrations.get_mut(&id)
        } else {
            None
        }
    }

    /// 安装指定 ID 的 Service Worker。
    ///
    /// 将 SW 从 `Registered` 推进到 `Installed` 状态。
    /// 如果安装失败（状态不正确），返回 `false`。
    pub fn install(&mut self, id: u64) -> bool {
        if let Some(reg) = self.registrations.get_mut(&id)
            && reg.state == ServiceWorkerState::Registered
        {
            reg.state = ServiceWorkerState::Installing;
            // 模拟 install 事件执行
            reg.state = ServiceWorkerState::Installed;
            return true;
        }
        false
    }

    /// 激活指定 ID 的 Service Worker。
    ///
    /// 将 SW 从 `Installed` 推进到 `Activated` 状态，并将其设为该 origin 的活跃 SW。
    /// 如果该 origin 已有活跃的 SW，旧 SW 会被标记为 `Redundant`。
    pub fn activate(&mut self, id: u64) -> bool {
        // 先检查状态，再提取 origin
        let origin = {
            let reg = match self.registrations.get(&id) {
                Some(r) => r,
                None => return false,
            };
            if reg.state != ServiceWorkerState::Installed {
                return false;
            }
            reg.origin.clone()
        };

        // 将旧的活跃 SW 标记为废弃
        if let Some(&old_id) = self.active_by_origin.get(&origin)
            && old_id != id
            && let Some(old_reg) = self.registrations.get_mut(&old_id)
        {
            old_reg.mark_redundant();
        }

        // 推进新 SW 的状态
        if let Some(reg) = self.registrations.get_mut(&id) {
            reg.state = ServiceWorkerState::Activating;
            reg.state = ServiceWorkerState::Activated;
            self.active_by_origin.insert(origin, id);
            return true;
        }
        false
    }

    /// 注销指定 ID 的 Service Worker。
    pub fn unregister(&mut self, id: u64) -> bool {
        if let Some(reg) = self.registrations.remove(&id) {
            // 如果是活跃的 SW，从活跃映射中移除
            if reg.state == ServiceWorkerState::Activated {
                self.active_by_origin.remove(&reg.origin);
            }
            true
        } else {
            false
        }
    }

    /// 拦截 Fetch 请求。
    ///
    /// 如果指定 origin 有活跃的 SW，先尝试从 SW 的 Cache Storage 中匹配。
    /// 否则返回 `NoWorker`。
    pub fn intercept_fetch(&self, request: &CacheRequest, origin: &str) -> FetchInterceptResult {
        if let Some(reg) = self.get_active(origin)
            && reg.is_in_scope(&request.url)
        {
            // 尝试从 Cache 匹配
            if let Some(response) = reg.match_cached(request) {
                return FetchInterceptResult::Cached(response);
            }
            // SW 没有缓存匹配，让请求通过
            return FetchInterceptResult::PassThrough;
        }
        FetchInterceptResult::NoWorker
    }

    /// 返回所有注册的数量。
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// 返回注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// 返回活跃的 SW 数量。
    pub fn active_count(&self) -> usize {
        self.active_by_origin.len()
    }

    /// 返回所有 origin 列表（有活跃 SW 的）。
    pub fn active_origins(&self) -> Vec<&str> {
        self.active_by_origin.keys().map(|s| s.as_str()).collect()
    }

    /// 将一个响应缓存到指定 origin 的活跃 SW 的 Cache Storage 中。
    pub fn cache_response(
        &mut self,
        origin: &str,
        cache_name: &str,
        request: CacheRequest,
        response: CacheResponse,
    ) -> bool {
        if let Some(reg) = self.get_active_mut(origin) {
            let cache = reg.cache_storage.open(cache_name);
            let _ = cache.put(request, response);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut reg = ServiceWorkerRegistration::new(1, "/sw.js", "/", "https://example.com");
        assert_eq!(reg.state, ServiceWorkerState::Registered);
        assert!(!reg.is_active());

        assert!(reg.advance_state()); // → Installing
        assert_eq!(reg.state, ServiceWorkerState::Installing);

        assert!(reg.advance_state()); // → Installed
        assert_eq!(reg.state, ServiceWorkerState::Installed);

        assert!(reg.advance_state()); // → Activating
        assert_eq!(reg.state, ServiceWorkerState::Activating);

        assert!(reg.advance_state()); // → Activated
        assert_eq!(reg.state, ServiceWorkerState::Activated);
        assert!(reg.is_active());

        assert!(!reg.advance_state()); // 已是终态
        assert_eq!(reg.state, ServiceWorkerState::Activated);
    }

    #[test]
    fn test_mark_redundant() {
        let mut reg = ServiceWorkerRegistration::new(1, "/sw.js", "/", "https://example.com");
        reg.state = ServiceWorkerState::Activated;
        reg.mark_redundant();
        assert_eq!(reg.state, ServiceWorkerState::Redundant);
        assert!(!reg.is_active());
    }

    #[test]
    fn test_scope_matching() {
        let reg = ServiceWorkerRegistration::new(1, "/sw.js", "/app/", "https://example.com");
        assert!(reg.is_in_scope("/app/page.html"));
        assert!(reg.is_in_scope("/app/sub/page.html"));
        assert!(!reg.is_in_scope("/other/page.html"));
    }

    #[test]
    fn test_scope_root() {
        let reg = ServiceWorkerRegistration::new(1, "/sw.js", "/", "https://example.com");
        assert!(reg.is_in_scope("/anything"));
        assert!(reg.is_in_scope("/"));
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");

        assert_eq!(id, 0);
        assert_eq!(registry.len(), 1);
        assert!(registry.get(id).is_some());

        let reg = registry.get(id).unwrap();
        assert_eq!(reg.script_url, "/sw.js");
        assert_eq!(reg.scope, "/");
        assert_eq!(reg.origin, "https://example.com");
    }

    #[test]
    fn test_registry_install_activate() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");

        assert!(registry.install(id));
        assert_eq!(registry.get(id).unwrap().state, ServiceWorkerState::Installed);
        assert_eq!(registry.active_count(), 0);

        assert!(registry.activate(id));
        assert_eq!(registry.get(id).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn test_registry_install_wrong_state() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");

        // 先安装一次
        assert!(registry.install(id));
        // 再次安装应失败
        assert!(!registry.install(id));
    }

    #[test]
    fn test_registry_activate_replaces_old() {
        let mut registry = ServiceWorkerRegistry::new();

        let id1 = registry.register("/sw-v1.js", "/", "https://example.com");
        registry.install(id1);
        registry.activate(id1);
        assert!(registry.get(id1).unwrap().is_active());

        // 注册新版本
        let id2 = registry.register("/sw-v2.js", "/", "https://example.com");
        registry.install(id2);
        registry.activate(id2);

        // 旧 SW 应该被标记为废弃
        assert_eq!(registry.get(id1).unwrap().state, ServiceWorkerState::Redundant);
        assert!(registry.get(id2).unwrap().is_active());
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn test_candidate_does_not_replace_active_before_activation() {
        let mut registry = ServiceWorkerRegistry::new();
        let active_id = registry.register("/sw-v1.js", "/", "https://example.com");
        assert!(registry.install(active_id));
        assert!(registry.activate(active_id));

        let candidate_id = registry.register("/sw-v2.js", "/", "https://example.com");
        assert_eq!(registry.get_active("https://example.com").unwrap().id, active_id);
        assert_eq!(
            registry.get(candidate_id).unwrap().state,
            ServiceWorkerState::Registered
        );

        assert!(registry.install(candidate_id));
        assert_eq!(registry.get_active("https://example.com").unwrap().id, active_id);
        assert_eq!(registry.get(candidate_id).unwrap().state, ServiceWorkerState::Installed);
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn test_invalid_candidate_activation_preserves_active() {
        let mut registry = ServiceWorkerRegistry::new();
        let active_id = registry.register("/sw-v1.js", "/", "https://example.com");
        assert!(registry.install(active_id));
        assert!(registry.activate(active_id));

        let candidate_id = registry.register("/sw-v2.js", "/", "https://example.com");
        assert!(!registry.activate(candidate_id));

        assert_eq!(registry.get_active("https://example.com").unwrap().id, active_id);
        assert_eq!(registry.get(active_id).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(
            registry.get(candidate_id).unwrap().state,
            ServiceWorkerState::Registered
        );
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn test_unregister_redundant_version_preserves_replacement() {
        let mut registry = ServiceWorkerRegistry::new();
        let old_id = registry.register("/sw-v1.js", "/", "https://example.com");
        assert!(registry.install(old_id));
        assert!(registry.activate(old_id));

        let replacement_id = registry.register("/sw-v2.js", "/", "https://example.com");
        assert!(registry.install(replacement_id));
        assert!(registry.activate(replacement_id));
        assert_eq!(registry.get(old_id).unwrap().state, ServiceWorkerState::Redundant);

        assert!(registry.unregister(old_id));
        assert_eq!(registry.get_active("https://example.com").unwrap().id, replacement_id);
        assert_eq!(registry.active_count(), 1);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_replacement_isolated_from_other_origin() {
        let mut registry = ServiceWorkerRegistry::new();
        let a_v1 = registry.register("/sw-v1.js", "/", "https://a.example");
        let b_v1 = registry.register("/sw-v1.js", "/", "https://b.example");
        assert!(registry.install(a_v1));
        assert!(registry.activate(a_v1));
        assert!(registry.install(b_v1));
        assert!(registry.activate(b_v1));

        let a_v2 = registry.register("/sw-v2.js", "/", "https://a.example");
        assert!(registry.install(a_v2));
        assert!(registry.activate(a_v2));

        assert_eq!(registry.get_active("https://a.example").unwrap().id, a_v2);
        assert_eq!(registry.get_active("https://b.example").unwrap().id, b_v1);
        assert_eq!(registry.get(a_v1).unwrap().state, ServiceWorkerState::Redundant);
        assert_eq!(registry.get(b_v1).unwrap().state, ServiceWorkerState::Activated);
        assert_eq!(registry.active_count(), 2);
    }

    #[test]
    fn test_registry_get_active() {
        let mut registry = ServiceWorkerRegistry::new();
        assert!(registry.get_active("https://example.com").is_none());

        let id = registry.register("/sw.js", "/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        let active = registry.get_active("https://example.com").unwrap();
        assert_eq!(active.id, id);
        assert_eq!(active.script_url, "/sw.js");
    }

    #[test]
    fn test_registry_uninstall() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        assert!(registry.unregister(id));
        assert!(registry.get(id).is_none());
        assert!(registry.get_active("https://example.com").is_none());
        assert_eq!(registry.active_count(), 0);
    }

    #[test]
    fn test_registry_uninstall_nonexistent() {
        let mut registry = ServiceWorkerRegistry::new();
        assert!(!registry.unregister(999));
    }

    #[test]
    fn test_intercept_fetch_no_worker() {
        let registry = ServiceWorkerRegistry::new();
        let request = CacheRequest::new("https://example.com/page.html");
        let result = registry.intercept_fetch(&request, "https://example.com");
        assert!(matches!(result, FetchInterceptResult::NoWorker));
    }

    #[test]
    fn test_intercept_fetch_pass_through() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        let request = CacheRequest::new("https://example.com/page.html");
        let result = registry.intercept_fetch(&request, "https://example.com");
        assert!(matches!(result, FetchInterceptResult::PassThrough));
    }

    #[test]
    fn test_intercept_fetch_cached() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        // 先缓存一个响应
        let request = CacheRequest::new("https://example.com/cached.html");
        let response = CacheResponse::ok(b"cached content".to_vec());
        let cache = registry.get_active_mut("https://example.com").unwrap();
        let _ = cache.cache_storage.open("v1").put(request.clone(), response);

        // 拦截应该返回缓存
        let result = registry.intercept_fetch(&request, "https://example.com");
        match result {
            FetchInterceptResult::Cached(resp) => {
                assert_eq!(resp.status, 200);
                assert_eq!(resp.body, b"cached content".to_vec());
            }
            _ => panic!("expected Cached, got {:?}", result),
        }
    }

    #[test]
    fn test_intercept_fetch_out_of_scope() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/app/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        let request = CacheRequest::new("https://example.com/other/page.html");
        let result = registry.intercept_fetch(&request, "https://example.com");
        assert!(matches!(result, FetchInterceptResult::NoWorker));
    }

    #[test]
    fn test_cache_response_via_registry() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        let request = CacheRequest::new("https://example.com/api/data");
        let response = CacheResponse::new(200, b"hello".to_vec());

        assert!(registry.cache_response("https://example.com", "api-cache", request.clone(), response));

        // 验证缓存可用
        let result = registry.intercept_fetch(&request, "https://example.com");
        assert!(matches!(result, FetchInterceptResult::Cached(_)));
    }

    #[test]
    fn test_cache_response_no_active_worker() {
        let mut registry = ServiceWorkerRegistry::new();
        let request = CacheRequest::new("https://example.com/page");
        let response = CacheResponse::ok(b"test".to_vec());
        assert!(!registry.cache_response("https://example.com", "v1", request, response));
    }

    #[test]
    fn test_multiple_origins() {
        let mut registry = ServiceWorkerRegistry::new();

        let id1 = registry.register("/sw.js", "/", "https://a.com");
        let id2 = registry.register("/sw.js", "/", "https://b.com");

        registry.install(id1);
        registry.activate(id1);
        registry.install(id2);
        registry.activate(id2);

        assert_eq!(registry.active_count(), 2);

        let origins = registry.active_origins();
        assert!(origins.contains(&"https://a.com"));
        assert!(origins.contains(&"https://b.com"));
    }

    #[test]
    fn test_display_state() {
        assert_eq!(format!("{}", ServiceWorkerState::Registered), "registered");
        assert_eq!(format!("{}", ServiceWorkerState::Installing), "installing");
        assert_eq!(format!("{}", ServiceWorkerState::Installed), "installed");
        assert_eq!(format!("{}", ServiceWorkerState::Activating), "activating");
        assert_eq!(format!("{}", ServiceWorkerState::Activated), "activated");
        assert_eq!(format!("{}", ServiceWorkerState::Redundant), "redundant");
    }

    #[test]
    fn test_is_empty() {
        let registry = ServiceWorkerRegistry::new();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_advance_state_from_redundant() {
        let mut reg = ServiceWorkerRegistration::new(1, "/sw.js", "/", "https://example.com");
        reg.state = ServiceWorkerState::Redundant;
        assert!(!reg.advance_state());
        assert_eq!(reg.state, ServiceWorkerState::Redundant);
    }

    #[test]
    fn test_activate_nonexistent() {
        let mut registry = ServiceWorkerRegistry::new();
        assert!(!registry.activate(999));
    }

    #[test]
    fn test_install_nonexistent() {
        let mut registry = ServiceWorkerRegistry::new();
        assert!(!registry.install(999));
    }

    #[test]
    fn test_multiple_registrations_same_origin() {
        let mut registry = ServiceWorkerRegistry::new();

        let id1 = registry.register("/sw-v1.js", "/", "https://example.com");
        let id2 = registry.register("/sw-v2.js", "/", "https://example.com");
        let id3 = registry.register("/sw-v3.js", "/", "https://example.com");

        registry.install(id1);
        registry.activate(id1);
        registry.install(id2);
        registry.activate(id2);
        registry.install(id3);
        registry.activate(id3);

        // Only the last activated SW should be the active one
        assert_eq!(registry.get(id1).unwrap().state, ServiceWorkerState::Redundant);
        assert_eq!(registry.get(id2).unwrap().state, ServiceWorkerState::Redundant);
        assert!(registry.get(id3).unwrap().is_active());

        let active = registry.get_active("https://example.com").unwrap();
        assert_eq!(active.id, id3);
        assert_eq!(active.script_url, "/sw-v3.js");
        assert_eq!(registry.active_count(), 1);
    }

    #[test]
    fn test_unregister_non_active() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");

        // Unregister without installing/activating
        assert!(registry.unregister(id));
        assert!(registry.get(id).is_none());
        assert_eq!(registry.len(), 0);
        assert_eq!(registry.active_count(), 0);
    }

    #[test]
    fn test_intercept_fetch_wrong_origin() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://a.com");
        registry.install(id);
        registry.activate(id);

        let request = CacheRequest::new("https://b.com/page.html");
        let result = registry.intercept_fetch(&request, "https://b.com");
        assert!(matches!(result, FetchInterceptResult::NoWorker));
    }

    #[test]
    fn test_cache_response_and_intercept() {
        let mut registry = ServiceWorkerRegistry::new();

        // Register → install → activate
        let id = registry.register("/sw.js", "/app/", "https://example.com");
        assert!(registry.install(id));
        assert!(registry.activate(id));
        assert!(registry.get(id).unwrap().is_active());

        // Cache a response
        let req = CacheRequest::new("https://example.com/app/data.json");
        let resp = CacheResponse::ok(br#"{"status":"ok"}"#.to_vec());
        assert!(registry.cache_response("https://example.com", "api-cache", req.clone(), resp));

        // Intercept should return the cached response
        let result = registry.intercept_fetch(&req, "https://example.com");
        match result {
            FetchInterceptResult::Cached(r) => {
                assert_eq!(r.status, 200);
                assert_eq!(r.body, br#"{"status":"ok"}"#.to_vec());
            }
            other => panic!("expected Cached, got {:?}", other),
        }

        // A URL outside scope should not be intercepted
        let other_req = CacheRequest::new("https://example.com/other/page.html");
        let other_result = registry.intercept_fetch(&other_req, "https://example.com");
        assert!(matches!(other_result, FetchInterceptResult::NoWorker));
    }

    #[test]
    fn test_scope_matching_full_url() {
        let reg = ServiceWorkerRegistration::new(1, "/sw.js", "/app/", "https://example.com");

        // Full URLs should match based on path (host is not checked when scope is a path)
        assert!(reg.is_in_scope("https://example.com/app/page.html"));
        assert!(reg.is_in_scope("https://example.com/app/sub/deep.html"));
        assert!(!reg.is_in_scope("https://example.com/other/page.html"));
        // Note: scope is a path ("/app/"), so host is not considered —
        // "https://other.com/app/page.html" still matches because the path starts with "/app/"
        assert!(reg.is_in_scope("https://other.com/app/page.html"));

        // Verify a full-URL scope does consider host
        let reg2 = ServiceWorkerRegistration::new(2, "/sw.js", "https://example.com/app/", "https://example.com");
        assert!(reg2.is_in_scope("https://example.com/app/page.html"));
        assert!(!reg2.is_in_scope("https://other.com/app/page.html"));
    }

    #[test]
    fn test_scope_matching_query_string() {
        let reg = ServiceWorkerRegistration::new(1, "/sw.js", "/app/", "https://example.com");

        // Query strings should be ignored for scope matching
        assert!(reg.is_in_scope("/app/page.html?q=1"));
        assert!(reg.is_in_scope("/app/page.html?foo=bar&baz=2"));
        assert!(reg.is_in_scope("https://example.com/app/page.html?v=2"));
    }

    #[test]
    fn test_get_active_mut() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");
        registry.install(id);
        registry.activate(id);

        {
            let active = registry.get_active_mut("https://example.com").unwrap();
            assert_eq!(active.id, id);
            active.script_content = Some("console.log('sw')".to_string());
        }

        let reg = registry.get(id).unwrap();
        assert_eq!(reg.script_content, Some("console.log('sw')".to_string()));
    }

    #[test]
    fn test_register_sequential_ids() {
        let mut registry = ServiceWorkerRegistry::new();

        let id0 = registry.register("/sw0.js", "/", "https://a.com");
        let id1 = registry.register("/sw1.js", "/", "https://b.com");
        let id2 = registry.register("/sw2.js", "/", "https://c.com");

        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_active_origins_empty() {
        let registry = ServiceWorkerRegistry::new();
        let origins = registry.active_origins();
        assert!(origins.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // 覆盖率补充测试
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_extract_path_no_slash_after_scheme() {
        // URL without path after host → returns full url
        let path = super::extract_path("https://example.com");
        assert_eq!(path, "https://example.com");
    }

    #[test]
    fn test_get_mut_existing_registration() {
        let mut registry = ServiceWorkerRegistry::new();
        let id = registry.register("/sw.js", "/", "https://example.com");
        let reg = registry.get_mut(id).unwrap();
        assert_eq!(reg.scope, "/");
    }

    #[test]
    fn test_get_mut_nonexistent() {
        let mut registry = ServiceWorkerRegistry::new();
        assert!(registry.get_mut(999).is_none());
    }

    #[test]
    fn test_activate_nonexistent_id() {
        let mut registry = ServiceWorkerRegistry::new();
        // activate with a nonexistent id should return false
        assert!(!registry.activate(999));
    }
}
