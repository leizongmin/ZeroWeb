//! 子资源 fetch 元数据 — 供 [`AsyncFetchHost`] 传递优先级与类型。

/// 异步 fetch 元数据（经 IPC 头或调度器传递）。
#[derive(Debug, Clone, Copy)]
pub struct ResourceFetchMeta {
    /// 资源类型 hint：`document` / `style` / `script` / `image` / `font` / …
    pub resource_type: &'static str,
    /// 优先级 0–4（越大越优先）。
    pub priority: u8,
}

impl ResourceFetchMeta {
    /// 主文档。
    pub const DOCUMENT: Self = Self {
        resource_type: "document",
        priority: 4,
    };
    /// 外链样式表。
    pub const STYLESHEET: Self = Self {
        resource_type: "style",
        priority: 4,
    };
    /// 脚本。
    pub const SCRIPT: Self = Self {
        resource_type: "script",
        priority: 3,
    };
    /// 图片。
    pub const IMAGE: Self = Self {
        resource_type: "image",
        priority: 2,
    };
    /// 音视频、`source` 与文本轨道资源。
    pub const MEDIA: Self = Self {
        resource_type: "media",
        priority: 2,
    };
    /// 字体。
    pub const FONT: Self = Self {
        resource_type: "font",
        priority: 3,
    };
    /// `<link rel=prefetch>`。
    pub const PREFETCH: Self = Self {
        resource_type: "prefetch",
        priority: 1,
    };
    /// `<link rel=preload>`（按 `as` 可再调 priority）。
    pub fn preload(as_type: &str) -> Self {
        let resource_type: &'static str = match as_type {
            "style" => "style",
            "document" => "document",
            "script" => "script",
            "font" => "font",
            "image" => "image",
            "media" => "media",
            _ => "preload",
        };
        let priority = match as_type {
            "style" | "document" => 4,
            "script" | "font" => 3,
            "image" | "media" => 2,
            _ => 2,
        };
        Self {
            resource_type,
            priority,
        }
    }
}
