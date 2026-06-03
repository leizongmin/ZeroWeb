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
    fn from_env_unset() {
        // 临时设置环境变量
        let var = std::env::var("ZEROWEB_RENDERER");
        unsafe { std::env::remove_var("ZEROWEB_RENDERER") };

        let result = RenderMode::from_env();
        assert_eq!(result, Ok(None));

        // 恢复环境变量
        if let Ok(val) = var {
            unsafe { std::env::set_var("ZEROWEB_RENDERER", val) };
        }
    }

    #[test]
    fn from_env_valid_values() {
        let test_cases = [
            ("auto", RenderMode::Auto),
            ("gpu", RenderMode::Gpu),
            ("cpu", RenderMode::Cpu),
            ("CPU", RenderMode::Cpu),
            ("  GPU  ", RenderMode::Gpu),
            ("Software", RenderMode::Cpu),
            ("soft", RenderMode::Cpu),
        ];

        for (input, expected) in test_cases {
            let var = std::env::var("ZEROWEB_RENDERER");
            unsafe { std::env::set_var("ZEROWEB_RENDERER", input) };

            let result = RenderMode::from_env();
            assert_eq!(result, Ok(Some(expected)));

            // 恢复环境变量
            if let Ok(val) = var {
                unsafe { std::env::set_var("ZEROWEB_RENDERER", val) };
            } else {
                unsafe { std::env::remove_var("ZEROWEB_RENDERER") };
            }
        }
    }

    #[test]
    fn from_env_invalid_value() {
        let var = std::env::var("ZEROWEB_RENDERER");
        unsafe { std::env::set_var("ZEROWEB_RENDERER", "invalid_mode") };

        let result = RenderMode::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("auto|gpu|cpu"));

        // 恢复环境变量
        if let Ok(val) = var {
            unsafe { std::env::set_var("ZEROWEB_RENDERER", val) };
        } else {
            unsafe { std::env::remove_var("ZEROWEB_RENDERER") };
        }
    }

    #[test]
    fn from_env_non_utf8() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let var = std::env::var("ZEROWEB_RENDERER");

        // 设置一个非 UTF-8 字符串
        let non_utf8 = OsString::from_vec(vec![0xFF, 0xFE]);
        unsafe { std::env::set_var("ZEROWEB_RENDERER", non_utf8) };

        let result = RenderMode::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not valid UTF-8"));

        // 恢复环境变量
        if let Ok(val) = var {
            unsafe { std::env::set_var("ZEROWEB_RENDERER", val) };
        } else {
            unsafe { std::env::remove_var("ZEROWEB_RENDERER") };
        }
    }

    #[test]
    fn render_mode_values_documentation() {
        let values = RenderMode::values();
        assert_eq!(values, "auto|gpu|cpu");
    }

    #[test]
    fn render_mode_equality() {
        assert_eq!(RenderMode::Auto, RenderMode::Auto);
        assert_eq!(RenderMode::Gpu, RenderMode::Gpu);
        assert_eq!(RenderMode::Cpu, RenderMode::Cpu);
        assert_ne!(RenderMode::Auto, RenderMode::Gpu);
        assert_ne!(RenderMode::Auto, RenderMode::Cpu);
        assert_ne!(RenderMode::Gpu, RenderMode::Cpu);
    }

    #[test]
    fn parse_case_insensitive() {
        let test_cases = [
            ("AUTO", RenderMode::Auto),
            ("Gpu", RenderMode::Gpu),
            ("cPu", RenderMode::Cpu),
            ("aUtO", RenderMode::Auto),
        ];

        for (input, expected) in test_cases {
            let parsed: RenderMode = input.parse().unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn render_mode_default() {
        let mode = RenderMode::default();
        assert_eq!(mode, RenderMode::Auto);
    }

    #[test]
    fn render_mode_display_format() {
        assert_eq!(format!("{}", RenderMode::Auto), "auto");
        assert_eq!(format!("{}", RenderMode::Gpu), "gpu");
        assert_eq!(format!("{}", RenderMode::Cpu), "cpu");
    }

    #[test]
    fn render_mode_debug_format() {
        assert_eq!(format!("{:?}", RenderMode::Auto), "Auto");
        assert_eq!(format!("{:?}", RenderMode::Gpu), "Gpu");
        assert_eq!(format!("{:?}", RenderMode::Cpu), "Cpu");
    }

    #[test]
    fn render_mode_clone() {
        let mode = RenderMode::Gpu;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn render_mode_copy() {
        fn test_copy(m: RenderMode) -> RenderMode {
            m
        }

        let mode = RenderMode::Cpu;
        let copied = test_copy(mode);
        assert_eq!(mode, copied);
    }

    #[test]
    fn parse_whitespace_only() {
        // 空字符串
        let result: Result<RenderMode, _> = "".parse();
        assert!(result.is_err());

        // 只有空格
        let result: Result<RenderMode, _> = "   ".parse();
        assert!(result.is_err());
    }

    #[test]
    fn parse_with_newlines() {
        let test_cases = [
            ("auto\n", RenderMode::Auto),
            ("gpu\r\n", RenderMode::Gpu),
            ("cpu \t ", RenderMode::Cpu),
        ];

        for (input, expected) in test_cases {
            let parsed: RenderMode = input.parse().unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn from_env_preserves_unset() {
        // 确保环境变量未设置
        let var = std::env::var("ZEROWEB_RENDERER");
        if let Ok(_) = var {
            unsafe { std::env::remove_var("ZEROWEB_RENDERER") };
        }

        // 多次调用应该返回相同结果
        assert_eq!(RenderMode::from_env(), Ok(None));
        assert_eq!(RenderMode::from_env(), Ok(None));
    }

    #[test]
    fn render_mode_values_string() {
        let values = RenderMode::values();
        // 检查包含所有有效值
        assert!(values.contains("auto"));
        assert!(values.contains("gpu"));
        assert!(values.contains("cpu"));
        // 检查分隔符
        assert!(values.contains('|'));
        // 检查没有无效值
        assert!(!values.contains("metal"));
        assert!(!values.contains("invalid"));
    }

    #[test]
    fn render_mode_string_round_trip() {
        // 测试从字符串解析再格式化是否保持一致
        let modes = [RenderMode::Auto, RenderMode::Gpu, RenderMode::Cpu];

        for mode in modes {
            let s = mode.to_string();
            let parsed: RenderMode = s.parse().unwrap();
            assert_eq!(mode, parsed);
        }
    }
}
