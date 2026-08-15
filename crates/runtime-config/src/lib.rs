//! ZeroWeb 运行时环境变量的集中定义与解析。
//!
//! 业务 crate 不应直接读取此处列出的环境变量；新增产品级开关时，先在本 crate
//! 定义名称、默认值与解析函数，再同步更新 `docs/runtime-environment.md`。

use std::path::PathBuf;

/// 已支持的、面向浏览器运行时的环境变量说明。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentVariable {
    /// 变量名。
    pub name: &'static str,
    /// 未设置时的默认值或行为。
    pub default: &'static str,
    /// 简短用途说明。
    pub description: &'static str,
}

/// 产品级运行时开关的权威清单。
pub const ENVIRONMENT_VARIABLES: &[EnvironmentVariable] = &[
    EnvironmentVariable {
        name: "ZEROWEB_RENDERER",
        default: "auto",
        description: "渲染后端：auto、gpu 或 cpu",
    },
    EnvironmentVariable {
        name: "ZERO_BROWSER_MULTIPROCESS",
        default: "enabled",
        description: "renderer 子进程",
    },
    EnvironmentVariable {
        name: "ZERO_RENDERER_PATH",
        default: "automatic discovery",
        description: "renderer 可执行文件路径",
    },
    EnvironmentVariable {
        name: "ZERO_PRIVATE",
        default: "disabled",
        description: "隐私浏览（不写 HTTP 磁盘缓存）",
    },
    EnvironmentVariable {
        name: "ZERO_CACHE_DIR",
        default: "platform cache directory",
        description: "HTTP 磁盘缓存目录",
    },
    EnvironmentVariable {
        name: "ZERO_HTTP2",
        default: "enabled",
        description: "HTTP/2",
    },
    EnvironmentVariable {
        name: "ZERO_NOPROXY",
        default: "disabled",
        description: "绕过系统和环境代理",
    },
    EnvironmentVariable {
        name: "ZERO_MAX_CONNECTIONS_PER_ORIGIN",
        default: "6",
        description: "每 origin 最大并发连接数",
    },
    EnvironmentVariable {
        name: "ZERO_MAX_CONNECTIONS_TOTAL",
        default: "24",
        description: "全局最大并发请求数",
    },
    EnvironmentVariable {
        name: "ZERO_BROWSER_COLOR_SCHEME",
        default: "system",
        description: "覆盖 prefers-color-scheme",
    },
    EnvironmentVariable {
        name: "ZERO_BROWSER_UI_LANG",
        default: "locale or en",
        description: "浏览器 UI 语言",
    },
    EnvironmentVariable {
        name: "ZERO_SCROLL_BLIT",
        default: "enabled",
        description: "滚动位图复用",
    },
    EnvironmentVariable {
        name: "ZW_RENDER_THREAD",
        default: "enabled",
        description: "持久 CPU 渲染工作线程",
    },
    EnvironmentVariable {
        name: "ZW_IMAGE_DECODER_PROCESS",
        default: "enabled",
        description: "独立图像解码进程",
    },
    EnvironmentVariable {
        name: "ZW_IMAGE_DECODER_BIN",
        default: "zero-image-decoder",
        description: "图像解码器路径",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_PROCESS",
        default: "enabled",
        description: "独立 compositor 进程",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_BIN",
        default: "automatic discovery",
        description: "compositor 可执行文件路径",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_ASYNC_SCROLL",
        default: "enabled",
        description: "compositor 异步滚动",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_UI_FRAMES",
        default: "enabled",
        description: "compositor UI 帧",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_SHM",
        default: "enabled on Linux",
        description: "Linux POSIX 共享内存帧",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_GPU_ZERO_COPY",
        default: "enabled on Linux",
        description: "Linux GPU 零拷贝消费",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_PRESENT",
        default: "enabled",
        description: "Viz present",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_OWNED_PRESENT",
        default: "enabled",
        description: "compositor 持有最终 present",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_GPU",
        default: "enabled on Linux",
        description: "compositor GPU 光栅化",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_GPU_IMAGE",
        default: "enabled on Linux",
        description: "GPU shared-image 通道",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_GPU_TEXTURE_EXPORT",
        default: "enabled on Linux",
        description: "GPU dma-buf 导出",
    },
    EnvironmentVariable {
        name: "ZW_BROWSER_GPU_DMABUF_IMPORT",
        default: "enabled on Linux",
        description: "Browser GPU dma-buf 导入",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_SCROLL_TRANSFORM",
        default: "enabled",
        description: "compositor 侧滚动变换",
    },
    EnvironmentVariable {
        name: "ZW_RENDERER_SECCOMP",
        default: "enabled",
        description: "renderer seccomp 沙箱",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_SANDBOX",
        default: "enabled",
        description: "compositor 环境沙箱",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_SECCOMP",
        default: "enabled",
        description: "compositor seccomp 沙箱",
    },
    EnvironmentVariable {
        name: "ZW_COMPOSITOR_LANDLOCK",
        default: "enabled",
        description: "compositor Landlock 沙箱",
    },
];

/// `1` 或不区分大小写的 `true` 才表示启用。
pub fn enabled_when_true(name: &str) -> bool {
    std::env::var(name).is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

/// 默认启用；仅 `0` 或不区分大小写的 `false` 禁用。
pub fn enabled_by_default(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .is_none_or(|value| value != "0" && !value.eq_ignore_ascii_case("false"))
}

/// 默认启用；仅精确值 `0` 禁用，用于已有的兼容性 kill-switch 语义。
pub fn enabled_unless_zero(name: &str) -> bool {
    std::env::var(name).as_deref() != Ok("0")
}

/// 可选路径配置；空字符串也视为未配置。
pub fn optional_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

/// 可选 UTF-8 字符串配置；空字符串视为未配置。
pub fn optional_string(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}

/// 正整数配置，不合法值回退默认值。
pub fn positive_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

/// 渲染模式的原始环境变量值；未设置时为 `None`。
pub fn renderer_mode() -> Result<Option<String>, String> {
    std::env::var("ZEROWEB_RENDERER")
        .map(Some)
        .or_else(|error| match error {
            std::env::VarError::NotPresent => Ok(None),
            std::env::VarError::NotUnicode(_) => Err("ZEROWEB_RENDERER is not valid UTF-8".to_string()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_unique_names() {
        let mut names = ENVIRONMENT_VARIABLES
            .iter()
            .map(|variable| variable.name)
            .collect::<Vec<_>>();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), ENVIRONMENT_VARIABLES.len());
    }
}
