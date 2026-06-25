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
pub fn read_file_url(url: &str) -> Result<HttpResponse, NetError> {
    let path = file_url_to_path(url)?;
    let body =
        std::fs::read(&path).map_err(|e| NetError::Network(format!("failed to read {}: {e}", path.display())))?;
    let content_type = guess_content_type(&path);
    Ok(HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".to_string(), content_type.to_string())],
        body,
        url: url.to_string(),
        redirect_count: 0,
    })
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
