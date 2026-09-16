//! DevTools frontend 静态 serve（devtools goal M0-P2）。
//!
//! bundle 目录经 `ZW_DEVTOOLS_FRONTEND_DIR` 配置（runtime-config 集中入口），
//! 配置后既有 HTTP 发现面把 `/devtools/<path>` 映射到 bundle 内文件，使
//! `/json` 的 `devtoolsFrontendUrl` 指向可点开的本地 frontend（复用 Chrome
//! DevTools frontend，不自建面板 UI）。
//!
//! 安全边界：路径解析拒绝目录穿越（词法 `..` 检查 + canonicalize 前缀复核），
//! 只读 serve bundle 内文件，不列目录。

use std::io::Write as _;
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};

/// serve 前缀（发现面把该前缀路由进本模块）。
pub(super) const SERVE_PREFIX: &str = "/devtools";

/// bundle 未配置或路径未命中时的提示（点开 devtoolsFrontendUrl 前需先 provision）。
const BUNDLE_NOT_CONFIGURED: &str = "devtools frontend bundle not provisioned: set ZW_DEVTOOLS_FRONTEND_DIR to a devtools-frontend build output directory";

/// 扩展名 → Content-Type（bundle 实际出现的资源类型；未知扩展名按字节流处理）。
fn mime_for_extension(extension: &str) -> &'static str {
    match extension {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// 把 URL 路径部分的 `%xx` 转义解码为字节；非法转义返回 `None`。
fn percent_decode(raw: &str) -> Option<Vec<u8>> {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3)?;
            let value = u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
            out.push(value);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    Some(out)
}

/// 解析 `/devtools/...` 请求 → bundle 内文件绝对路径 + Content-Type。
///
/// 返回 `None` 表示路径非法（穿越/空转义），调用方按 404 拒绝；
/// 路径合法但文件不存在也返回 `Some`，由调用方按 404 应答（与 Chrome 行为一致）。
pub(super) fn resolve_request(root: &Path, raw_path: &str) -> Option<(PathBuf, &'static str)> {
    // 剥前缀与 query/fragment；`/devtools` 与 `/devtools/` 都落到 bundle 根的
    // inspector.html（与 devtoolsFrontendUrl 的入口一致）。
    let mut rest = raw_path.strip_prefix(SERVE_PREFIX)?.to_string();
    if let Some(query) = rest.find(['?', '#']) {
        rest.truncate(query);
    }
    let decoded = percent_decode(&rest)?;
    let decoded = std::str::from_utf8(&decoded).ok()?;
    let relative = decoded.trim_start_matches('/');
    let relative = if relative.is_empty() {
        "inspector.html"
    } else {
        relative
    };

    // 词法穿越检查：逐 component 拒绝 ParentDir 与绝对/Windows 盘符形态。
    let candidate = Path::new(relative);
    let safe = candidate
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if !safe {
        return None;
    }

    // canonicalize 前缀复核：兜住 symlink 与词法检查漏网的逃逸路径。文件不存在的
    // 场景 canonicalize 必然失败——返回未解析路径交给调用方 fs::read 落 404
    // （与 Chrome 对 bundle 内缺失资源返回 404 的行为一致）。
    let canonical_root = root.canonicalize().ok()?;
    let file = canonical_root.join(candidate);
    match file.canonicalize() {
        Ok(canonical_file) => {
            if !canonical_file.starts_with(&canonical_root) {
                return None;
            }
            let mime = canonical_file
                .extension()
                .and_then(|e| e.to_str())
                .map_or("application/octet-stream", mime_for_extension);
            Some((canonical_file, mime))
        }
        Err(_) => Some((file, "application/octet-stream")),
    }
}

/// 写一段 HTTP 应答（字节体，状态行 + Content-Type + Content-Length + 关闭连接）。
fn write_response(stream: &TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let mut writable = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = writable.write_all(head.as_bytes());
    let _ = writable.write_all(body);
    let _ = writable.flush();
}

/// 处理 `/devtools/...` 静态请求（已由发现面路由；`raw_path` 含 query）。
pub(super) fn handle_request(frontend_dir: Option<&Path>, stream: &TcpStream, raw_path: &str) {
    let Some(dir) = frontend_dir else {
        write_response(
            stream,
            "503 Service Unavailable",
            "text/plain; charset=utf-8",
            BUNDLE_NOT_CONFIGURED.as_bytes(),
        );
        return;
    };
    match resolve_request(dir, raw_path) {
        Some((file, mime)) => match std::fs::read(&file) {
            Ok(body) => write_response(stream, "200 OK", mime, &body),
            Err(_) => write_response(stream, "404 Not Found", "text/plain", b"Not Found"),
        },
        None => write_response(stream, "404 Not Found", "text/plain", b"Not Found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 建一个临时 bundle 根（测试内联文件，避免依赖真实 bundle）。
    fn temp_root(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zw-devtools-serve-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolves_entry_and_assets() {
        let root = temp_root("entry");
        fs::write(root.join("inspector.html"), "<html></html>").unwrap();
        let (file, mime) = resolve_request(&root, "/devtools/inspector.html?ws=127.0.0.1:9222").unwrap();
        assert_eq!(file, root.canonicalize().unwrap().join("inspector.html"));
        assert_eq!(mime, "text/html; charset=utf-8");

        fs::write(root.join("chunk-abc123.js"), "export {};").unwrap();
        let (_, mime) = resolve_request(&root, "/devtools/chunk-abc123.js").unwrap();
        assert_eq!(mime, "text/javascript; charset=utf-8");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn bare_prefix_serves_entrypoint() {
        let root = temp_root("bare");
        fs::write(root.join("inspector.html"), "entry").unwrap();
        let (file, _) = resolve_request(&root, "/devtools").unwrap();
        assert!(file.ends_with("inspector.html"));
        let (file, _) = resolve_request(&root, "/devtools/").unwrap();
        assert!(file.ends_with("inspector.html"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_path_traversal() {
        let root = temp_root("traversal");
        fs::write(root.join("inspector.html"), "entry").unwrap();
        // 词法形态（未转义）
        assert!(resolve_request(&root, "/devtools/../Cargo.toml").is_none());
        // 转义形态
        assert!(resolve_request(&root, "/devtools/%2e%2e/Cargo.toml").is_none());
        // 合法路径但文件不存在 → Some（调用方 404）
        let (file, _) = resolve_request(&root, "/devtools/missing.js").unwrap();
        assert!(!file.exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_invalid_percent_escape() {
        let root = temp_root("escape");
        assert!(resolve_request(&root, "/devtools/%zz").is_none());
        assert!(resolve_request(&root, "/devtools/%2").is_none());
        let _ = fs::remove_dir_all(&root);
    }
}
