//! `file:` URL 本地文件读取。

use std::path::{Path, PathBuf};

use crate::{HttpResponse, NetError};

/// 判断 URL 是否为 `file:` 协议。
pub fn is_file_url(url: &str) -> bool {
    url.as_bytes()
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"file:"))
}

/// 将 `file:` URL 转为本地文件路径。
pub fn file_url_to_path(url: &str) -> Result<PathBuf, NetError> {
    let parsed = url::Url::parse(url).map_err(|e| NetError::UrlParse(e.to_string()))?;
    if parsed.scheme() != "file" {
        return Err(NetError::UrlParse(format!("not a file URL: {url}")));
    }
    parsed
        .to_file_path()
        .map_err(|()| NetError::UrlParse(format!("file URL cannot be converted to path: {url}")))
}

fn guess_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "text/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

/// 读取 `file:` URL 指向的本地文件，返回类似 HTTP 200 的响应。
///
/// WPT 约定：资源旁可有一个 `<file>.headers` sidecar 文件（如 `foo.css.headers`），
/// 内含 HTTP header 行（如 `Content-Type: text/css; charset=iso-8859-1`）。file:// 无真实
/// HTTP 层，须读 sidecar 把 header（尤其 charset）注入响应，供 CSS Syntax §6.2 charset
/// determination 使用（WPT character-encoding-031~037,041 经此设 charset）。
pub fn read_file_url(url: &str) -> Result<HttpResponse, NetError> {
    let path = file_url_to_path(url)?;
    let body =
        std::fs::read(&path).map_err(|e| NetError::Network(format!("failed to read {}: {e}", path.display())))?;

    // 读 `<file>.headers` sidecar（若存在），提取 header 行。
    let mut headers = read_headers_sidecar(&path);
    let has_content_type = headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-type"));
    if !has_content_type {
        // sidecar 未指定 Content-Type → 按扩展名猜测补上。
        headers.push(("Content-Type".to_string(), guess_content_type(&path).to_string()));
    }

    Ok(HttpResponse {
        status_code: 200,
        headers,
        body,
        url: url.to_string(),
        redirect_count: 0,
    })
}

/// 读取 `<path>.headers` sidecar 文件，解析 `Name: Value` 行为 header 列表。
///
/// 文件不存在则返回空 vec。格式遵循 WPT 约定（每行一个 header，`:` 分隔名值）。
fn read_headers_sidecar(path: &Path) -> Vec<(String, String)> {
    // 拼接 `<path>.headers`：直接在路径字符串后加 ".headers"，避免 with_extension 对
    // 无扩展名 / 多扩展名路径的边界行为。
    let sidecar = PathBuf::from(format!("{}.headers", path.display()));
    let Ok(content) = std::fs::read_to_string(&sidecar) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_url_loads_local_html() {
        let dir = std::env::temp_dir();
        let file = dir.join("zero_net_file_url_test.html");
        std::fs::write(&file, b"<html><body>local</body></html>").unwrap();

        let url = url::Url::from_file_path(&file).unwrap().to_string();
        let resp = read_file_url(&url).expect("read_file_url should succeed");
        assert!(resp.is_success());
        assert_eq!(resp.text().unwrap(), "<html><body>local</body></html>");

        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn read_file_url_missing_file_errors() {
        let dir = std::env::temp_dir();
        let file = dir.join("zero_net_file_url_missing_test.html");
        let url = url::Url::from_file_path(&file).unwrap().to_string();
        assert!(read_file_url(&url).is_err());
    }
}
