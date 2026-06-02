//! 欢迎页和错误页面 HTML 模板

/// 欢迎页 HTML
pub const WELCOME_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>ZeroBrowser</title></head>
<body style="font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #f8f9fa;">
  <div style="text-align: center;">
    <h1 style="color: #333; font-size: 48px;">ZeroBrowser</h1>
    <p style="color: #666; font-size: 18px;">基于 Rust 的跨平台浏览器</p>
    <p style="color: #999; font-size: 14px;">在地址栏输入 URL 开始浏览</p>
  </div>
</body>
</html>"#;

/// 生成错误页面 HTML
pub fn generate_error_page(url: &str, error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>加载失败</title></head>
<body style="font-family: sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #fff3f3;">
  <div style="text-align: center;">
    <h1 style="color: #c62828;">页面加载失败</h1>
    <p style="color: #555;">无法加载: <code>{url}</code></p>
    <p style="color: #888; font-size: 14px;">错误: {error}</p>
  </div>
</body>
</html>"#
    )
}
