//! # 资源预加载
//!
//! 解析 `<link rel="preload">` 和 `<link rel="prefetch">` 提示，
//! 按优先级管理资源加载队列，加速页面关键资源获取。

use std::collections::HashMap;
use std::fmt;

/// 资源提示类型（`<link rel="...">`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceHintType {
    /// `<link rel="preload">` — 当前页面必需资源，高优先级。
    Preload,
    /// `<link rel="prefetch">` — 未来页面可能需要的资源，低优先级。
    Prefetch,
    /// `<link rel="preconnect">` — 预连接到目标源。
    Preconnect,
    /// `<link rel="dns-prefetch">` — 预解析 DNS。
    DnsPrefetch,
}

impl fmt::Display for ResourceHintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceHintType::Preload => write!(f, "preload"),
            ResourceHintType::Prefetch => write!(f, "prefetch"),
            ResourceHintType::Preconnect => write!(f, "preconnect"),
            ResourceHintType::DnsPrefetch => write!(f, "dns-prefetch"),
        }
    }
}

/// 从 `rel` 属性值解析提示类型。
///
/// 返回 None 表示不是已知的资源提示。
pub fn parse_resource_hint(rel: &str) -> Option<ResourceHintType> {
    match rel.to_ascii_lowercase().trim() {
        "preload" => Some(ResourceHintType::Preload),
        "prefetch" => Some(ResourceHintType::Prefetch),
        "preconnect" => Some(ResourceHintType::Preconnect),
        "dns-prefetch" => Some(ResourceHintType::DnsPrefetch),
        _ => None,
    }
}

/// 资源类型（`<link as="...">` 属性值）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// 脚本（`as="script"`）。
    Script,
    /// 样式表（`as="style"`）。
    Style,
    /// 图片（`as="image"`）。
    Image,
    /// 字体（`as="font"`）。
    Font,
    /// 音频（`as="audio"`）。
    Audio,
    /// 视频（`as="video"`）。
    Video,
    /// Fetch/XHR 资源（`as="fetch"`）。
    Fetch,
    /// 文档（`as="document"`）。
    Document,
    /// 嵌入/对象（`as="embed"` / `as="object"`）。
    Embed,
    /// 未知类型。
    Other,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Script => write!(f, "script"),
            ResourceType::Style => write!(f, "style"),
            ResourceType::Image => write!(f, "image"),
            ResourceType::Font => write!(f, "font"),
            ResourceType::Audio => write!(f, "audio"),
            ResourceType::Video => write!(f, "video"),
            ResourceType::Fetch => write!(f, "fetch"),
            ResourceType::Document => write!(f, "document"),
            ResourceType::Embed => write!(f, "embed"),
            ResourceType::Other => write!(f, "other"),
        }
    }
}

/// 从 `as` 属性值解析资源类型。
pub fn parse_resource_type(as_value: &str) -> ResourceType {
    match as_value.to_ascii_lowercase().trim() {
        "script" => ResourceType::Script,
        "style" => ResourceType::Style,
        "image" => ResourceType::Image,
        "font" => ResourceType::Font,
        "audio" => ResourceType::Audio,
        "video" => ResourceType::Video,
        "fetch" => ResourceType::Fetch,
        "document" => ResourceType::Document,
        "embed" | "object" => ResourceType::Embed,
        _ => ResourceType::Other,
    }
}

/// 资源加载优先级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadPriority {
    /// 最低优先级。
    Idle = 0,
    /// 低优先级（prefetch）。
    Low = 1,
    /// 中等优先级。
    Medium = 2,
    /// 高优先级（preload 关键资源）。
    High = 3,
    /// 最高优先级（阻塞渲染的关键资源）。
    Critical = 4,
}

impl fmt::Display for LoadPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadPriority::Idle => write!(f, "idle"),
            LoadPriority::Low => write!(f, "low"),
            LoadPriority::Medium => write!(f, "medium"),
            LoadPriority::High => write!(f, "high"),
            LoadPriority::Critical => write!(f, "critical"),
        }
    }
}

/// 资源加载状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceLoadState {
    /// 等待加载。
    Pending,
    /// 加载中。
    Loading,
    /// 加载完成。
    Loaded,
    /// 加载失败。
    Failed,
}

/// 资源提示条目。
#[derive(Debug, Clone)]
pub struct ResourceHint {
    /// 资源 URL。
    pub url: String,
    /// 提示类型。
    pub hint_type: ResourceHintType,
    /// 资源类型（`as` 属性值）。
    pub resource_type: ResourceType,
    /// 加载优先级。
    pub priority: LoadPriority,
    /// CORS 属性（`crossorigin`）。
    pub cors: bool,
    /// 资源完整性校验（`integrity` 属性）。
    pub integrity: Option<String>,
    /// 当前加载状态。
    pub state: ResourceLoadState,
}

impl ResourceHint {
    /// 从 HTML `<link>` 元素属性创建资源提示。
    ///
    /// # 参数
    /// - `url`: `href` 属性值
    /// - `rel`: `rel` 属性值
    /// - `as_value`: `as` 属性值（可选）
    /// - `crossorigin`: `crossorigin` 属性是否存在
    /// - `integrity`: `integrity` 属性值（可选）
    ///
    /// 返回 None 如果 `rel` 不是已知的资源提示类型。
    pub fn from_link_attrs(
        url: &str,
        rel: &str,
        as_value: Option<&str>,
        crossorigin: bool,
        integrity: Option<&str>,
    ) -> Option<Self> {
        let hint_type = parse_resource_hint(rel)?;
        let resource_type = as_value.map(parse_resource_type).unwrap_or(ResourceType::Other);

        let priority = Self::infer_priority(hint_type, resource_type);

        Some(Self {
            url: url.to_string(),
            hint_type,
            resource_type,
            priority,
            cors: crossorigin,
            integrity: integrity.map(|s| s.to_string()),
            state: ResourceLoadState::Pending,
        })
    }

    /// 根据提示类型和资源类型推断加载优先级。
    fn infer_priority(hint_type: ResourceHintType, resource_type: ResourceType) -> LoadPriority {
        match hint_type {
            ResourceHintType::Preload => {
                // preload 资源按类型分配优先级
                match resource_type {
                    ResourceType::Script => LoadPriority::High,
                    ResourceType::Style => LoadPriority::Critical,
                    ResourceType::Font => LoadPriority::High,
                    ResourceType::Fetch => LoadPriority::Medium,
                    _ => LoadPriority::Medium,
                }
            }
            ResourceHintType::Prefetch => LoadPriority::Low,
            ResourceHintType::Preconnect => LoadPriority::Medium,
            ResourceHintType::DnsPrefetch => LoadPriority::Low,
        }
    }
}

/// 资源预加载管理器。
///
/// 收集页面中的资源提示，按优先级排序，管理加载队列。
#[derive(Debug, Clone, Default)]
pub struct ResourcePreloader {
    /// 已注册的资源提示（按 URL 去重）。
    hints: HashMap<String, ResourceHint>,
}

impl ResourcePreloader {
    /// 创建新的资源预加载管理器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个资源提示。
    ///
    /// 如果相同 URL 已经注册，保留优先级更高的那个。
    pub fn register(&mut self, hint: ResourceHint) {
        self.hints
            .entry(hint.url.clone())
            .and_modify(|existing| {
                // 保留更高优先级的提示
                if hint.priority > existing.priority {
                    *existing = hint.clone();
                }
            })
            .or_insert(hint);
    }

    /// 从 HTML `<link>` 元素属性注册资源提示。
    ///
    /// 如果 `rel` 不是已知的资源提示类型，忽略。
    pub fn register_link(
        &mut self,
        url: &str,
        rel: &str,
        as_value: Option<&str>,
        crossorigin: bool,
        integrity: Option<&str>,
    ) -> bool {
        if let Some(hint) = ResourceHint::from_link_attrs(url, rel, as_value, crossorigin, integrity) {
            self.register(hint);
            true
        } else {
            false
        }
    }

    /// 获取按优先级排序的待加载资源列表。
    ///
    /// 返回所有 `Pending` 状态的资源，按优先级从高到低排序。
    pub fn pending_resources(&self) -> Vec<&ResourceHint> {
        let mut pending: Vec<_> = self
            .hints
            .values()
            .filter(|h| h.state == ResourceLoadState::Pending)
            .collect();
        pending.sort_by_key(|h| std::cmp::Reverse(h.priority));
        pending
    }

    /// 标记资源为加载中。
    pub fn mark_loading(&mut self, url: &str) -> bool {
        if let Some(hint) = self.hints.get_mut(url)
            && hint.state == ResourceLoadState::Pending
        {
            hint.state = ResourceLoadState::Loading;
            true
        } else {
            false
        }
    }

    /// 标记资源为已加载。
    pub fn mark_loaded(&mut self, url: &str) {
        if let Some(hint) = self.hints.get_mut(url) {
            hint.state = ResourceLoadState::Loaded;
        }
    }

    /// 标记资源为加载失败。
    pub fn mark_failed(&mut self, url: &str) {
        if let Some(hint) = self.hints.get_mut(url) {
            hint.state = ResourceLoadState::Failed;
        }
    }

    /// 获取指定 URL 的资源提示。
    pub fn get(&self, url: &str) -> Option<&ResourceHint> {
        self.hints.get(url)
    }

    /// 返回已注册的资源提示数量。
    pub fn len(&self) -> usize {
        self.hints.len()
    }

    /// 检查是否没有注册任何资源提示。
    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
    }

    /// 清除所有资源提示。
    pub fn clear(&mut self) {
        self.hints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resource_hint() {
        assert_eq!(parse_resource_hint("preload"), Some(ResourceHintType::Preload));
        assert_eq!(parse_resource_hint("prefetch"), Some(ResourceHintType::Prefetch));
        assert_eq!(parse_resource_hint("preconnect"), Some(ResourceHintType::Preconnect));
        assert_eq!(parse_resource_hint("dns-prefetch"), Some(ResourceHintType::DnsPrefetch));
        assert_eq!(parse_resource_hint("stylesheet"), None);
        assert_eq!(parse_resource_hint("  Preload  "), Some(ResourceHintType::Preload));
    }

    #[test]
    fn test_parse_resource_type() {
        assert_eq!(parse_resource_type("script"), ResourceType::Script);
        assert_eq!(parse_resource_type("style"), ResourceType::Style);
        assert_eq!(parse_resource_type("image"), ResourceType::Image);
        assert_eq!(parse_resource_type("font"), ResourceType::Font);
        assert_eq!(parse_resource_type("fetch"), ResourceType::Fetch);
        assert_eq!(parse_resource_type("document"), ResourceType::Document);
        assert_eq!(parse_resource_type("unknown"), ResourceType::Other);
        assert_eq!(parse_resource_type("  FONT  "), ResourceType::Font);
    }

    #[test]
    fn test_resource_hint_from_link_attrs_preload() {
        let hint =
            ResourceHint::from_link_attrs("https://cdn.example.com/app.js", "preload", Some("script"), false, None)
                .unwrap();

        assert_eq!(hint.url, "https://cdn.example.com/app.js");
        assert_eq!(hint.hint_type, ResourceHintType::Preload);
        assert_eq!(hint.resource_type, ResourceType::Script);
        assert_eq!(hint.priority, LoadPriority::High);
        assert!(!hint.cors);
        assert!(hint.integrity.is_none());
        assert_eq!(hint.state, ResourceLoadState::Pending);
    }

    #[test]
    fn test_resource_hint_from_link_attrs_prefetch() {
        let hint = ResourceHint::from_link_attrs(
            "https://example.com/next-page.js",
            "prefetch",
            None,
            true,
            Some("sha384-abc123"),
        )
        .unwrap();

        assert_eq!(hint.hint_type, ResourceHintType::Prefetch);
        assert_eq!(hint.resource_type, ResourceType::Other);
        assert_eq!(hint.priority, LoadPriority::Low);
        assert!(hint.cors);
        assert_eq!(hint.integrity, Some("sha384-abc123".to_string()));
    }

    #[test]
    fn test_resource_hint_unknown_rel_returns_none() {
        let result = ResourceHint::from_link_attrs("style.css", "stylesheet", Some("style"), false, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_preload_style_is_critical() {
        let hint = ResourceHint::from_link_attrs("style.css", "preload", Some("style"), false, None).unwrap();
        assert_eq!(hint.priority, LoadPriority::Critical);
    }

    #[test]
    fn test_preload_font_is_high() {
        let hint = ResourceHint::from_link_attrs("font.woff2", "preload", Some("font"), true, None).unwrap();
        assert_eq!(hint.priority, LoadPriority::High);
    }

    #[test]
    fn test_preloader_register_and_pending() {
        let mut preloader = ResourcePreloader::new();
        assert!(preloader.is_empty());

        preloader.register_link("app.js", "preload", Some("script"), false, None);
        preloader.register_link("style.css", "preload", Some("style"), false, None);
        preloader.register_link("next.js", "prefetch", None, false, None);

        assert_eq!(preloader.len(), 3);

        let pending = preloader.pending_resources();
        assert_eq!(pending.len(), 3);
        // style.css (Critical) > app.js (High) > next.js (Low)
        assert_eq!(pending[0].url, "style.css");
        assert_eq!(pending[1].url, "app.js");
        assert_eq!(pending[2].url, "next.js");
    }

    #[test]
    fn test_preloader_dedup_keeps_higher_priority() {
        let mut preloader = ResourcePreloader::new();

        // 先注册为 prefetch（Low）
        preloader.register_link("app.js", "prefetch", None, false, None);
        // 再注册为 preload script（High）— 应覆盖
        preloader.register_link("app.js", "preload", Some("script"), false, None);

        assert_eq!(preloader.len(), 1);
        let hint = preloader.get("app.js").unwrap();
        assert_eq!(hint.hint_type, ResourceHintType::Preload);
        assert_eq!(hint.priority, LoadPriority::High);
    }

    #[test]
    fn test_preloader_mark_loading() {
        let mut preloader = ResourcePreloader::new();
        preloader.register_link("app.js", "preload", Some("script"), false, None);

        assert!(preloader.mark_loading("app.js"));
        assert_eq!(preloader.get("app.js").unwrap().state, ResourceLoadState::Loading);

        // 重复标记返回 false
        assert!(!preloader.mark_loading("app.js"));
    }

    #[test]
    fn test_preloader_mark_loaded() {
        let mut preloader = ResourcePreloader::new();
        preloader.register_link("app.js", "preload", Some("script"), false, None);
        preloader.mark_loading("app.js");
        preloader.mark_loaded("app.js");

        assert_eq!(preloader.get("app.js").unwrap().state, ResourceLoadState::Loaded);
        // loaded 资源不出现在 pending 列表中
        assert!(preloader.pending_resources().is_empty());
    }

    #[test]
    fn test_preloader_mark_failed() {
        let mut preloader = ResourcePreloader::new();
        preloader.register_link("404.js", "preload", Some("script"), false, None);
        preloader.mark_loading("404.js");
        preloader.mark_failed("404.js");

        assert_eq!(preloader.get("404.js").unwrap().state, ResourceLoadState::Failed);
        assert!(preloader.pending_resources().is_empty());
    }

    #[test]
    fn test_preloader_mark_loaded_nonexistent_is_noop() {
        let mut preloader = ResourcePreloader::new();
        preloader.mark_loaded("nonexistent.js");
        assert!(preloader.is_empty());
    }

    #[test]
    fn test_preloader_clear() {
        let mut preloader = ResourcePreloader::new();
        preloader.register_link("app.js", "preload", Some("script"), false, None);
        preloader.clear();
        assert!(preloader.is_empty());
    }

    #[test]
    fn test_preloader_register_link_returns_bool() {
        let mut preloader = ResourcePreloader::new();
        assert!(preloader.register_link("app.js", "preload", Some("script"), false, None));
        assert!(!preloader.register_link("style.css", "stylesheet", Some("style"), false, None));
    }

    #[test]
    fn test_preconnect_and_dns_prefetch() {
        let mut preloader = ResourcePreloader::new();
        preloader.register_link("https://cdn.example.com", "preconnect", None, false, None);
        preloader.register_link("https://api.example.com", "dns-prefetch", None, false, None);

        assert_eq!(preloader.len(), 2);
        let cdn = preloader.get("https://cdn.example.com").unwrap();
        assert_eq!(cdn.hint_type, ResourceHintType::Preconnect);
        assert_eq!(cdn.priority, LoadPriority::Medium);

        let api = preloader.get("https://api.example.com").unwrap();
        assert_eq!(api.hint_type, ResourceHintType::DnsPrefetch);
        assert_eq!(api.priority, LoadPriority::Low);
    }

    #[test]
    fn test_load_priority_ordering() {
        assert!(LoadPriority::Critical > LoadPriority::High);
        assert!(LoadPriority::High > LoadPriority::Medium);
        assert!(LoadPriority::Medium > LoadPriority::Low);
        assert!(LoadPriority::Low > LoadPriority::Idle);
    }

    #[test]
    fn test_display_traits() {
        assert_eq!(ResourceHintType::Preload.to_string(), "preload");
        assert_eq!(ResourceType::Script.to_string(), "script");
        assert_eq!(LoadPriority::High.to_string(), "high");
    }

    #[test]
    fn test_resource_hint_with_integrity() {
        let hint = ResourceHint::from_link_attrs(
            "lib.js",
            "preload",
            Some("script"),
            true,
            Some("sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC"),
        )
        .unwrap();
        assert!(hint.cors);
        assert!(hint.integrity.unwrap().starts_with("sha384"));
    }
}
