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

/// 读取 `file:` URL 指向的本地文件，返回类似 HTTP 200 的响应；目录返回索引列表页
///（Chrome 行为：`file:///tmp/` 列出条目而非报错）。
///
/// WPT 约定：资源旁可有一个 `<file>.headers` sidecar 文件（如 `foo.css.headers`），
/// 内含 HTTP header 行（如 `Content-Type: text/css; charset=iso-8859-1`）。file:// 无真实
/// HTTP 层，须读 sidecar 把 header（尤其 charset）注入响应，供 CSS Syntax §6.2 charset
/// determination 使用（WPT character-encoding-031~037,041 经此设 charset）。
pub fn read_file_url(url: &str) -> Result<HttpResponse, NetError> {
    let path = file_url_to_path(url)?;
    if path.is_dir() {
        return directory_listing_response(url, &path);
    }
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

/// 生成目录索引列表页（类似 HTTP 200 `text/html`）。
///
/// 条目为可点击链接：子目录 href 带 `/` 后缀；文件名按 href 百分号编码、显示文本
/// 按 HTML 转义，条目按名称排序、目录在前。
fn directory_listing_response(url: &str, path: &Path) -> Result<HttpResponse, NetError> {
    let read_dir =
        std::fs::read_dir(path).map_err(|e| NetError::Network(format!("failed to read {}: {e}", path.display())))?;
    let mut entries: Vec<(String, bool)> = read_dir
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type().ok()?.is_dir();
            Some((name, is_dir))
        })
        .collect();
    // 目录在前，同组内按名称排序。
    entries.sort_by(|a, b| (!a.1, &a.0).cmp(&(!b.1, &b.0)));

    let mut html = String::from("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Index of ");
    html.push_str(&html_escape(&path.display().to_string()));
    html.push_str("</title></head><body><h1>Index of ");
    html.push_str(&html_escape(&path.display().to_string()));
    html.push_str("</h1><hr><ul>\n");
    // 父目录导航（相对链接，浏览器解析到上级路径）。
    html.push_str("<li><a href=\"../\">../</a></li>\n");
    for (name, is_dir) in &entries {
        let href_suffix = if *is_dir { "/" } else { "" };
        html.push_str(&format!(
            "<li><a href=\"{}{}\">{}</a></li>\n",
            percent_encode_path_component(name),
            href_suffix,
            html_escape(name)
        ));
    }
    html.push_str("</ul><hr></body></html>");

    Ok(HttpResponse {
        status_code: 200,
        headers: vec![("Content-Type".to_string(), "text/html; charset=utf-8".to_string())],
        body: html.into_bytes(),
        url: url.to_string(),
        redirect_count: 0,
    })
}

/// HTML 文本转义（`& < > " '`），目录列表条目显示用。
fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 路径段百分号编码（RFC 3986 保留字符与非 ASCII 字节 → `%XX`），目录列表 href 用。
fn percent_encode_path_component(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for byte in name.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(byte as char),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
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

    #[test]
    /// T25：目录 URL 返回 `text/html` 索引列表，条目为链接且子目录带 `/` 后缀。
    fn read_file_url_directory_returns_listing() {
        let dir = std::env::temp_dir().join(format!("zero_net_dirlist_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let sub = dir.join("subdir");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(dir.join("page one.html"), b"<html></html>").unwrap();

        let url = url::Url::from_directory_path(&dir).unwrap().to_string();
        let resp = read_file_url(&url).expect("directory listing should succeed");
        assert!(resp.is_success());
        let content_type = resp
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        assert!(content_type.starts_with("text/html"), "content-type is {content_type}");
        let html = resp.text().unwrap();
        // 子目录条目带 `/` 后缀，便于相对导航进目录。
        assert!(html.contains("href=\"subdir/\""), "listing should link subdir: {html}");
        // 空格按 RFC 3986 百分号编码，显示文本保留原样。
        assert!(
            html.contains("href=\"page%20one.html\""),
            "href should be percent-encoded: {html}"
        );
        assert!(html.contains("page one.html"), "display text keeps raw name: {html}");
        // 父目录相对导航链接。
        assert!(html.contains("href=\"../\""), "listing should link parent: {html}");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    /// T25：列表条目按 HTML 转义，防文件名注入标记。
    fn read_file_url_listing_escapes_html_in_names() {
        let dir = std::env::temp_dir().join(format!("zero_net_direscape_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("<img src=x onerror=alert(1)>.txt"), b"hi").unwrap();

        let url = url::Url::from_directory_path(&dir).unwrap().to_string();
        let resp = read_file_url(&url).expect("directory listing should succeed");
        let html = resp.text().unwrap();
        assert!(
            html.contains("&lt;img src=x onerror=alert(1)&gt;.txt"),
            "display name should be escaped: {html}"
        );
        assert!(
            !html.contains("<img src=x"),
            "raw markup must not leak into listing: {html}"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    // ── R3367：file:// 安全/正确性属性锁定 ──
    // 这些属性当前依赖 `url` crate 的 WHATWG file-URL 规范化与 scheme 校验；
    // 锁定它们防止未来重构（换解析器 / 手工拼路径 / 放宽 scheme 校验）静默引入
    // 路径遍历或非 file scheme 任意本地文件读取。

    #[test]
    /// R3367：`file://` URL 中的 `..` 段被规范化，不能路径遍历到任意文件。
    fn file_url_to_path_normalizes_dotdot_segments_r3367() {
        // url::Url::to_file_path 的平台差异：Windows 仅接受盘符路径
        // （file:///C:/...），POSIX 接受 / 根路径。
        #[cfg(not(target_os = "windows"))]
        {
            let path = file_url_to_path("file:///tmp/a/../../etc/hostname").expect("应解析成功");
            assert_eq!(
                path,
                std::path::PathBuf::from("/etc/hostname"),
                "`..` 段须被 url crate 规范化"
            );
        }
        #[cfg(target_os = "windows")]
        {
            let path = file_url_to_path("file:///C:/tmp/a/../../etc/hostname").expect("应解析成功");
            assert_eq!(
                path,
                std::path::PathBuf::from(r"C:\etc\hostname"),
                "`..` 段须被 url crate 规范化（不能穿透盘符）"
            );
        }
    }

    #[test]
    /// R3367：scheme 大小写不敏感（RFC 3986），`FILE:`/`FiLe:` 均识别为 file URL。
    fn is_file_url_is_case_insensitive_r3367() {
        assert!(is_file_url("file:///x"));
        assert!(is_file_url("FILE:///x"));
        assert!(is_file_url("FiLe:///x"));
        assert!(!is_file_url("files:///x"), "files: 不应误判为 file:");
        assert!(!is_file_url("http://x"));
        assert!(
            !is_file_url("httpsfile:///x"),
            "前缀恰好是 file 但非 file: scheme 不应误判"
        );
    }

    #[test]
    /// R3367：非 file scheme（http/ftp 等）经 `file_url_to_path` 必须被拒。
    fn file_url_to_path_rejects_non_file_scheme_r3367() {
        assert!(file_url_to_path("http://example.com/x").is_err());
        assert!(file_url_to_path("ftp://example.com/x").is_err());
        assert!(file_url_to_path("data:text/plain,hi").is_err());
    }

    #[test]
    /// R3367：sidecar `<file>.headers` 的 Content-Type 优先于按扩展名猜测，
    /// 且 sidecar 内自定义 header 被原样注入（用于 WPT charset 注入）。
    fn read_file_url_sidecar_overrides_content_type_r3367() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir();
        let fname = format!("zero_net_sidecar_probe_{}.html", N.fetch_add(1, Ordering::SeqCst));
        let file = dir.join(&fname);
        std::fs::write(&file, b"<html></html>").unwrap();
        let sidecar = dir.join(format!("{fname}.headers"));
        std::fs::write(
            &sidecar,
            b"X-Custom: probe-value\r\nContent-Type: text/html; charset=iso-8859-1\r\n",
        )
        .unwrap();
        let url = url::Url::from_file_path(&file).unwrap().to_string();
        let resp = read_file_url(&url).expect("读取成功");

        // sidecar 指定了 Content-Type → 不应再用扩展名猜测（猜测值无 charset）
        let ct = resp
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
            .expect("应有 Content-Type");
        assert_eq!(
            ct.1, "text/html; charset=iso-8859-1",
            "sidecar Content-Type 须优先于猜测"
        );
        // 仅一个 Content-Type（不应猜测后再追加）
        let ct_count = resp
            .headers
            .iter()
            .filter(|(n, _)| n.eq_ignore_ascii_case("content-type"))
            .count();
        assert_eq!(ct_count, 1, "Content-Type 不应重复");
        // 自定义 header 注入
        let xc = resp
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("x-custom"))
            .expect("应有 X-Custom");
        assert_eq!(xc.1, "probe-value");

        let _ = std::fs::remove_file(file);
        let _ = std::fs::remove_file(sidecar);
    }
}
