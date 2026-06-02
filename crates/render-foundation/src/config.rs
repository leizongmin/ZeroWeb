//! 渲染配置 — 后端模式选择。

use std::str::FromStr;

/// 渲染后端模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// 自动选择可用后端，优先 GPU，失败时降级到 CPU。
    #[default]
    Auto,
    /// 强制 GPU 渲染。
    Gpu,
    /// 强制 CPU 软件渲染。
    Cpu,
}

impl RenderMode {
    /// 默认环境变量名。
    pub const ENV_VAR: &'static str = "ZEROWEB_RENDERER";

    /// 从环境变量读取渲染模式。
    ///
    /// 环境变量未设置时返回 `Ok(None)`；设置为无效值时返回错误。
    pub fn from_env() -> Result<Option<Self>, String> {
        match std::env::var(Self::ENV_VAR) {
            Ok(value) => value.parse().map(Some),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(err) => Err(format!("{} is not valid UTF-8: {err}", Self::ENV_VAR)),
        }
    }

    /// 命令行帮助中展示的允许值。
    pub fn values() -> &'static str {
        "auto|gpu|cpu"
    }
}

impl FromStr for RenderMode {
    type Err = String;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "gpu" => Ok(Self::Gpu),
            "cpu" | "software" | "soft" => Ok(Self::Cpu),
            other => Err(format!("invalid renderer '{other}', expected {}", Self::values())),
        }
    }
}

impl std::fmt::Display for RenderMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Auto => "auto",
            Self::Gpu => "gpu",
            Self::Cpu => "cpu",
        };
        f.write_str(value)
    }
}

#[cfg(test)]
mod tests {
    use super::RenderMode;

    #[test]
    fn parse_render_mode_values() {
        assert_eq!("auto".parse(), Ok(RenderMode::Auto));
        assert_eq!("GPU".parse(), Ok(RenderMode::Gpu));
        assert_eq!(" cpu ".parse(), Ok(RenderMode::Cpu));
        assert_eq!("software".parse(), Ok(RenderMode::Cpu));
    }

    #[test]
    fn parse_render_mode_rejects_invalid_value() {
        let err = "metal".parse::<RenderMode>().expect_err("invalid mode");
        assert!(err.contains("auto|gpu|cpu"));
    }

    #[test]
    fn render_mode_display_matches_cli_values() {
        assert_eq!(RenderMode::Auto.to_string(), "auto");
        assert_eq!(RenderMode::Gpu.to_string(), "gpu");
        assert_eq!(RenderMode::Cpu.to_string(), "cpu");
    }
}
