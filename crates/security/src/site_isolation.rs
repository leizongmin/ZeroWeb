//! # 站点隔离
//!
//! 跨站 iframe 的隔离策略，防止跨站 iframe 访问父页面 DOM。
//! 实现 site-per-process 模型的安全边界检查。

use crate::origin::Origin;
use std::collections::HashMap;
use std::fmt;

/// 站点（scheme + registered domain）。
///
/// 站点是比 Origin 更粗粒度的分组：`https://sub.example.com` 和
/// `https://other.example.com` 是不同的 Origin，但属于同一个站点
/// `example.com`。站点隔离按站点划分进程边界。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Site {
    /// 协议（scheme）。
    pub scheme: String,
    /// 注册域名（eTLD+1）。
    pub registrable_domain: String,
}

impl Site {
    /// 从 Origin 提取站点。
    ///
    /// registrable_domain 经 zero-psl 按 [Public Suffix List] 语义计算（eTLD+1），
    /// 而非朴素「取末两段」——后者会把 `a.github.io` 与 `b.github.io` 误判为同站。
    /// 例如 `sub.example.com` → `example.com`，`example.com` → `example.com`，
    /// 但 `a.github.io` → `a.github.io`（`*.github.io` 为公共后缀，各用户子域独立）。
    ///
    /// [Public Suffix List]: https://publicsuffix.org/
    pub fn from_origin(origin: &Origin) -> Self {
        let registrable_domain = zero_psl::PublicSuffixList::shared().registrable_domain(&origin.host);
        Self {
            scheme: origin.scheme.clone(),
            registrable_domain,
        }
    }

    /// 检查两个 Origin 是否属于同一站点。
    pub fn is_same_site(a: &Origin, b: &Origin) -> bool {
        Self::from_origin(a) == Self::from_origin(b)
    }
}

impl fmt::Display for Site {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.scheme, self.registrable_domain)
    }
}

/// 渲染进程信息。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessId(pub u64);

impl fmt::Display for ProcessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "process:{}", self.0)
    }
}

/// 渲染进程描述。
#[derive(Debug, Clone)]
struct RenderProcess {
    /// 进程关联的站点。
    site: Site,
    /// 进程锁定的源（可选，严格隔离时使用）。
    locked_origin: Option<Origin>,
}

/// 站点隔离策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IsolationPolicy {
    /// 不隔离：所有页面共享同一进程（不安全）。
    None,
    /// 按站点隔离：同站共享进程，跨站分进程。
    #[default]
    SiteIsolated,
    /// 按源隔离：每个源独立进程（最安全但开销最大）。
    StrictOriginIsolated,
}

/// 站点隔离管理器。
///
/// 管理渲染进程到站点的映射，决定 iframe 是否需要独立进程。
#[derive(Debug, Clone)]
pub struct SiteIsolationManager {
    /// 隔离策略。
    policy: IsolationPolicy,
    /// 进程映射（ProcessId → RenderProcess）。
    processes: HashMap<ProcessId, RenderProcess>,
    /// 站点到进程的反向映射（Site → ProcessId）。
    site_to_process: HashMap<Site, ProcessId>,
    /// 下一个进程 ID。
    next_process_id: u64,
}

impl SiteIsolationManager {
    /// 创建新的站点隔离管理器。
    pub fn new(policy: IsolationPolicy) -> Self {
        Self {
            policy,
            processes: HashMap::new(),
            site_to_process: HashMap::new(),
            next_process_id: 1,
        }
    }

    /// 使用默认策略创建管理器。
    pub fn with_default_policy() -> Self {
        Self::new(IsolationPolicy::default())
    }

    /// 获取当前隔离策略。
    pub fn policy(&self) -> IsolationPolicy {
        self.policy
    }

    /// 为指定源获取或创建渲染进程。
    ///
    /// - `None` 策略：所有源共享同一进程。
    /// - `SiteIsolated` 策略：同站点共享进程。
    /// - `StrictOriginIsolated` 策略：每个源独立进程。
    pub fn get_or_create_process(&mut self, origin: &Origin) -> ProcessId {
        match self.policy {
            IsolationPolicy::None => {
                // 所有源共享进程 ID 0
                self.processes.entry(ProcessId(0)).or_insert_with(|| RenderProcess {
                    site: Site::from_origin(origin),
                    locked_origin: None,
                });
                ProcessId(0)
            }
            IsolationPolicy::SiteIsolated => {
                let site = Site::from_origin(origin);
                if let Some(pid) = self.site_to_process.get(&site) {
                    return pid.clone();
                }
                let pid = self.allocate_process_id();
                self.processes.insert(
                    pid.clone(),
                    RenderProcess {
                        site: site.clone(),
                        locked_origin: None,
                    },
                );
                self.site_to_process.insert(site, pid.clone());
                pid
            }
            IsolationPolicy::StrictOriginIsolated => {
                // 按源精确匹配
                let found = self
                    .processes
                    .iter()
                    .find(|(_, proc)| proc.locked_origin.as_ref() == Some(origin))
                    .map(|(pid, _)| pid.clone());
                if let Some(pid) = found {
                    return pid;
                }
                let pid = self.allocate_process_id();
                self.processes.insert(
                    pid.clone(),
                    RenderProcess {
                        site: Site::from_origin(origin),
                        locked_origin: Some(origin.clone()),
                    },
                );
                pid
            }
        }
    }

    /// 检查 iframe 是否需要独立进程。
    ///
    /// 当 iframe 的源与父页面源跨站时，需要独立进程。
    pub fn needs_separate_process(&self, parent_origin: &Origin, iframe_origin: &Origin) -> bool {
        match self.policy {
            IsolationPolicy::None => false,
            IsolationPolicy::SiteIsolated => !Site::is_same_site(parent_origin, iframe_origin),
            IsolationPolicy::StrictOriginIsolated => parent_origin != iframe_origin,
        }
    }

    /// 检查跨站 iframe 是否可以访问父页面 DOM。
    ///
    /// 站点隔离下，跨站 iframe **始终无法**访问父页面 DOM。
    /// 这是站点隔离的核心安全保证。
    pub fn can_access_parent_dom(&self, parent_origin: &Origin, iframe_origin: &Origin) -> bool {
        match self.policy {
            IsolationPolicy::None => {
                // 无隔离时，仅同源可访问
                parent_origin == iframe_origin
            }
            IsolationPolicy::SiteIsolated | IsolationPolicy::StrictOriginIsolated => {
                // 站点隔离下，跨站完全不可访问；同站仍需同源
                parent_origin == iframe_origin
            }
        }
    }

    /// 获取指定源所在进程的 ID。
    pub fn get_process_for_origin(&self, origin: &Origin) -> Option<ProcessId> {
        match self.policy {
            IsolationPolicy::None => self.processes.get(&ProcessId(0)).map(|_| ProcessId(0)),
            IsolationPolicy::SiteIsolated => {
                let site = Site::from_origin(origin);
                self.site_to_process.get(&site).cloned()
            }
            IsolationPolicy::StrictOriginIsolated => self.processes.iter().find_map(|(pid, proc)| {
                if proc.locked_origin.as_ref() == Some(origin) {
                    Some(pid.clone())
                } else {
                    None
                }
            }),
        }
    }

    /// 检查两个源是否在同一进程中。
    pub fn are_in_same_process(&self, origin_a: &Origin, origin_b: &Origin) -> bool {
        match self.policy {
            IsolationPolicy::None => true,
            _ => {
                let proc_a = self.get_process_for_origin(origin_a);
                let proc_b = self.get_process_for_origin(origin_b);
                proc_a.is_some() && proc_a == proc_b
            }
        }
    }

    /// 移除指定进程。
    pub fn remove_process(&mut self, pid: &ProcessId) -> bool {
        if let Some(proc) = self.processes.remove(pid) {
            self.site_to_process.remove(&proc.site);
            true
        } else {
            false
        }
    }

    /// 返回活跃进程数量。
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    fn allocate_process_id(&mut self) -> ProcessId {
        let id = self.next_process_id;
        self.next_process_id += 1;
        ProcessId(id)
    }
}

impl Default for SiteIsolationManager {
    fn default() -> Self {
        Self::with_default_policy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(url: &str) -> Origin {
        Origin::parse(url).unwrap()
    }

    #[test]
    fn test_site_from_origin_subdomain() {
        let o = origin("https://sub.example.com");
        let site = Site::from_origin(&o);
        assert_eq!(site.registrable_domain, "example.com");
        assert_eq!(site.scheme, "https");
    }

    #[test]
    fn test_site_from_origin_apex() {
        let o = origin("https://example.com");
        let site = Site::from_origin(&o);
        assert_eq!(site.registrable_domain, "example.com");
    }

    #[test]
    fn test_site_from_origin_deep_subdomain() {
        let o = origin("https://a.b.c.example.com");
        let site = Site::from_origin(&o);
        assert_eq!(site.registrable_domain, "example.com");
    }

    #[test]
    fn test_site_from_origin_ip_address() {
        let o = origin("http://127.0.0.1:8080");
        let site = Site::from_origin(&o);
        assert_eq!(site.registrable_domain, "127.0.0.1");
    }

    #[test]
    fn test_is_same_site_true() {
        let a = origin("https://sub1.example.com");
        let b = origin("https://sub2.example.com");
        assert!(Site::is_same_site(&a, &b));
    }

    #[test]
    fn test_is_same_site_false_different_domain() {
        let a = origin("https://example.com");
        let b = origin("https://other.com");
        assert!(!Site::is_same_site(&a, &b));
    }

    #[test]
    fn test_is_same_site_false_different_scheme() {
        let a = origin("https://example.com");
        let b = origin("http://example.com");
        assert!(!Site::is_same_site(&a, &b));
    }

    #[test]
    fn test_site_display() {
        let o = origin("https://sub.example.com");
        let site = Site::from_origin(&o);
        assert_eq!(site.to_string(), "https://example.com");
    }

    /// R3380 回归锁定：`*.github.io` 是公共后缀，各用户子域是独立注册域。
    /// 朴素「取末两段」会把 a.github.io / b.github.io 都判为 github.io（同站），
    /// 正确 PSL 语义应判为不同站点（cookie same-site / site-isolation 依据）。
    #[test]
    fn test_psl_wildcard_suffix_isolates_github_user_subdomains() {
        let a = origin("https://a.github.io");
        let b = origin("https://b.github.io");
        assert_eq!(Site::from_origin(&a).registrable_domain, "a.github.io");
        assert_eq!(Site::from_origin(&b).registrable_domain, "b.github.io");
        assert!(
            !Site::is_same_site(&a, &b),
            "a.github.io 与 b.github.io 经 PSL 须判为不同站点"
        );
    }

    /// R3380 回归锁定：多标签公共后缀（co.uk）正确折叠。
    #[test]
    fn test_psl_multilabel_suffix_co_uk() {
        let a = origin("https://sub.example.co.uk");
        let site = Site::from_origin(&a);
        assert_eq!(site.registrable_domain, "example.co.uk");
    }

    #[test]
    fn test_none_policy_all_same_process() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::None);
        let a = origin("https://example.com");
        let b = origin("https://other.com");

        let pid_a = mgr.get_or_create_process(&a);
        let pid_b = mgr.get_or_create_process(&b);
        assert_eq!(pid_a, pid_b);
        assert_eq!(mgr.process_count(), 1);
    }

    #[test]
    fn test_none_policy_never_needs_separate() {
        let mgr = SiteIsolationManager::new(IsolationPolicy::None);
        let parent = origin("https://example.com");
        let iframe = origin("https://evil.com");
        assert!(!mgr.needs_separate_process(&parent, &iframe));
    }

    #[test]
    fn test_site_isolated_same_site_same_process() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let a = origin("https://sub1.example.com");
        let b = origin("https://sub2.example.com");

        let pid_a = mgr.get_or_create_process(&a);
        let pid_b = mgr.get_or_create_process(&b);
        assert_eq!(pid_a, pid_b);
        assert_eq!(mgr.process_count(), 1);
    }

    #[test]
    fn test_site_isolated_cross_site_different_process() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let a = origin("https://example.com");
        let b = origin("https://other.com");

        let pid_a = mgr.get_or_create_process(&a);
        let pid_b = mgr.get_or_create_process(&b);
        assert_ne!(pid_a, pid_b);
        assert_eq!(mgr.process_count(), 2);
    }

    #[test]
    fn test_site_isolated_needs_separate_process() {
        let mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let parent = origin("https://example.com");
        let same_site_iframe = origin("https://sub.example.com");
        let cross_site_iframe = origin("https://evil.com");

        assert!(!mgr.needs_separate_process(&parent, &same_site_iframe));
        assert!(mgr.needs_separate_process(&parent, &cross_site_iframe));
    }

    #[test]
    fn test_strict_origin_isolated_all_different() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::StrictOriginIsolated);
        let a = origin("https://sub1.example.com");
        let b = origin("https://sub2.example.com");

        let pid_a = mgr.get_or_create_process(&a);
        let pid_b = mgr.get_or_create_process(&b);
        assert_ne!(pid_a, pid_b);
        assert_eq!(mgr.process_count(), 2);
    }

    #[test]
    fn test_strict_origin_isolated_same_origin_same_process() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::StrictOriginIsolated);
        let a = origin("https://example.com");
        let b = origin("https://example.com");

        let pid_a = mgr.get_or_create_process(&a);
        let pid_b = mgr.get_or_create_process(&b);
        assert_eq!(pid_a, pid_b);
    }

    #[test]
    fn test_can_access_parent_dom_only_same_origin() {
        let mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let parent = origin("https://example.com");

        // 同源可访问
        assert!(mgr.can_access_parent_dom(&parent, &parent));

        // 跨源不可访问（即使同站）
        let sub = origin("https://sub.example.com");
        assert!(!mgr.can_access_parent_dom(&parent, &sub));

        // 跨站不可访问
        let evil = origin("https://evil.com");
        assert!(!mgr.can_access_parent_dom(&parent, &evil));
    }

    #[test]
    fn test_can_access_parent_dom_none_policy() {
        let mgr = SiteIsolationManager::new(IsolationPolicy::None);
        let parent = origin("https://example.com");
        let other = origin("https://other.com");

        assert!(mgr.can_access_parent_dom(&parent, &parent));
        assert!(!mgr.can_access_parent_dom(&parent, &other));
    }

    #[test]
    fn test_are_in_same_process() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let a = origin("https://sub.example.com");
        let b = origin("https://other.example.com");
        let c = origin("https://evil.com");

        mgr.get_or_create_process(&a);
        mgr.get_or_create_process(&c);

        assert!(mgr.are_in_same_process(&a, &b)); // 同站
        assert!(!mgr.are_in_same_process(&a, &c)); // 跨站
    }

    #[test]
    fn test_remove_process() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let a = origin("https://example.com");
        let pid = mgr.get_or_create_process(&a);

        assert!(mgr.remove_process(&pid));
        assert_eq!(mgr.process_count(), 0);
        assert!(!mgr.remove_process(&pid)); // 已删除
    }

    #[test]
    fn test_default_policy_is_site_isolated() {
        let mgr = SiteIsolationManager::default();
        assert_eq!(mgr.policy(), IsolationPolicy::SiteIsolated);
    }

    #[test]
    fn test_process_id_display() {
        let pid = ProcessId(42);
        assert_eq!(pid.to_string(), "process:42");
    }

    #[test]
    fn test_get_process_for_origin() {
        let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
        let a = origin("https://example.com");
        let pid = mgr.get_or_create_process(&a);

        let found = mgr.get_process_for_origin(&origin("https://sub.example.com"));
        assert_eq!(found, Some(pid));

        let not_found = mgr.get_process_for_origin(&origin("https://other.com"));
        assert_eq!(not_found, None);
    }
}
