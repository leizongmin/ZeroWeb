//! # zero-webview-api
//!
//! 面向外部应用的稳定嵌入接口。
//!
//! 提供构建器模式创建 WebView、导航、注入 JS、回调、渲染表面输出。

#![warn(missing_docs)]

/// WebView 错误类型
#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    /// 初始化失败
    #[error("初始化失败: {0}")]
    InitializationFailed(String),
    /// 导航失败
    #[error("导航失败: {0}")]
    NavigationFailed(String),
    /// 脚本执行失败
    #[error("脚本执行失败: {0}")]
    ScriptExecutionFailed(String),
}

/// WebView 构建器
pub struct WebViewBuilder {
    width: u32,
    height: u32,
}

impl WebViewBuilder {
    /// 创建新的 WebView 构建器
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
        }
    }

    /// 设置窗口尺寸
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// 构建 WebView 实例
    pub fn build(self) -> Result<WebView, WebViewError> {
        // TODO: 在 M10 里程碑中实现
        Err(WebViewError::InitializationFailed(
            "WebView 尚未实现".to_string(),
        ))
    }
}

impl Default for WebViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// WebView 实例
pub struct WebView {
    // TODO: 在 M10 里程碑中实现
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webview_builder_default() {
        let builder = WebViewBuilder::new();
        assert_eq!(builder.width, 800);
        assert_eq!(builder.height, 600);
    }
}
