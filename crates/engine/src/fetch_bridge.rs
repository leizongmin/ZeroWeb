//! P1b S3 / R2923 fetch bridge——共享于 browser `tab_js_worker` 与 renderer `js_worker`。
//!
//! 持 fetch handler cell + [`AsyncResolver`]；`register` 在 sandbox 注 `__zw_fetch` 回调。
//! 各 app 在 `js_worker_main` 构造 `FetchBridge`（传入自身 resolver），调 `register(sandbox)`
//! 注 `__zw_fetch` 回调；`SetFetchHandler` 命令 arm 调 `set_handler` 注入生产 handler。
//! `__zw_fetch` 回调非阻塞——子线程抓取 + `resolver.resolve` 回投（不冻结 JS worker）。
//!
//! **R2923 fetch 完整化**：handler 收 [`FetchRequest`]（method/url/headers/body）返
//! [`FetchResponse`]（status/status_text/headers/body）——支持非 GET（POST/PUT/DELETE/PATCH/
//! HEAD/OPTIONS）、请求头/请求体、响应状态码/响应头。GET 行为零回归（method 默认 GET、body=None）。
//!
//! `default_fetch_handler`（生产 HTTP 经 `zero_net::HttpClient::send`）由各 app 提供：
//! `zero-engine` 不依赖 `zero-net`（避免循环依赖），故生产 handler 留在 app 层。
//!
//! **wire 格式**（host→JS，经 `resolver.resolve` 单串）：成功 = `"__zwfr:"` 后接 4 个 `\x1f`
//! 分隔字段 status / status_text / headersWire / body；headersWire = `name\x1evalue\x1e...`
//! （flat，奇偶配对）。错误 = `"__zw_fetch_error:"` 后接 msg（旧约定，shim 落 ok:false）。body 为末字段
//! （取第 3 个 `\x1f` 之后全部），可含 `\x1f`；status/status_text/headersWire 不含控制分隔符。

use std::sync::{Arc, Mutex};

use zero_script_sandbox::Sandbox;

use crate::async_resolver::AsyncResolver;

/// JS `fetch` 请求——method（GET/POST/...）、url、headers 列表、可选 body（UTF-8 文本）。
///
/// 生产由各 app 提供 `default_fetch_handler`（经 `zero_net::HttpClient::send` 真实 HTTP，
/// 支持全方法/头/体）；测试用合成实现。
#[derive(Debug, Clone)]
pub struct FetchRequest {
    /// 请求 URL。
    pub url: String,
    /// HTTP 方法（大写：GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS；未知 → GET）。
    pub method: String,
    /// 请求头 (name, value) 列表。
    pub headers: Vec<(String, String)>,
    /// 请求体（UTF-8 文本；GET/HEAD 通常 None）。
    pub body: Option<String>,
    /// 请求体（原始字节；R3020 byte-wire——Blob/FormData multipart 二进制保真，csv-decimal 经 wire 传递）。
    /// 二进制 body 时 `body=None, body_bytes=Some(bytes)`；文本 body 时 `body=Some(text), body_bytes=None`。
    pub body_bytes: Option<Vec<u8>>,
}

/// JS `fetch` 响应——status/status_text/headers/body。
#[derive(Debug, Clone)]
pub struct FetchResponse {
    /// HTTP 状态码。
    pub status: u16,
    /// 状态码原因短语（"OK"/"Not Found"/...）。
    pub status_text: String,
    /// 响应头 (name, value) 列表。
    pub headers: Vec<(String, String)>,
    /// 响应体（UTF-8 文本；非 UTF-8 经 lossy 转——R3021 body_bytes 携带原始字节，body 为 lossy 文本回退）。
    pub body: String,
    /// 响应体原始字节（R3021 byte-wire——二进制 body 经 `__zw_bytes:` csv-decimal wire 传 JS，response.blob()/
    /// arrayBuffer() 取保真字节）。None 或 valid-UTF-8 → wire 用 body 文本（高效 + 向后兼容）。
    pub body_bytes: Option<Vec<u8>>,
}

impl FetchResponse {
    /// 构造 200 OK + body 的便捷响应（无头）。
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            status_text: "OK".to_string(),
            headers: Vec::new(),
            body: body.into(),
            body_bytes: None,
        }
    }
}

/// JS `fetch` 的抓取函数类型——收 [`FetchRequest`] 返 [`FetchResponse`] 或 error 串。
/// 生产由各 app 提供 `default_fetch_handler`（经 net client 真实 HTTP）；测试用合成实现。
pub type FetchHandler = Arc<dyn Fn(&FetchRequest) -> Result<FetchResponse, String> + Send + Sync>;

/// 单元分隔符（field 间）/ 记录分隔符（header name/value 间）——HTTP 文本不含，安全。
const FIELD_SEP: char = '\x1f';
const HEADER_SEP: char = '\x1e';
const WIRE_PREFIX: &str = "__zwfr:";
const ERR_PREFIX: &str = "__zw_fetch_error:";
/// 二进制 body wire 前缀（R3020）——shim 把 Blob/FormData 字节编码为 `__zw_bytes:` + csv-decimal
/// （`72,101,108`）传 host，host 解码为 `Vec<u8>`，闭合二进制保真（旧路径 `TextDecoder.decode` lossy）。
/// 文本 body 永不带此前缀（按 body 类型决定，非内容匹配），故无歧义。
const BYTES_PREFIX: &str = "__zw_bytes:";

/// 编码字节为 `__zw_bytes:` + csv-decimal wire（供测试对称 + 文档）。空字节 → 仅前缀。
pub fn encode_body_bytes(bytes: &[u8]) -> String {
    let mut s = String::from(BYTES_PREFIX);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&b.to_string());
    }
    s
}

/// 解码 body wire：`__zw_bytes:` 前缀 → csv-decimal → `Vec<u8>`；无前缀或 malformed → None（文本 body，
/// 调用方按原样 String 处理）。空字节体（`__zw_bytes:` 后空）→ `Some([])`。
pub fn decode_body_bytes_raw(wire: &str) -> Option<Vec<u8>> {
    if !wire.starts_with(BYTES_PREFIX) {
        return None;
    }
    let rest = &wire[BYTES_PREFIX.len()..];
    if rest.is_empty() {
        return Some(Vec::new());
    }
    let mut out = Vec::new();
    for part in rest.split(',') {
        match part.parse::<u8>() {
            Ok(b) => out.push(b),
            Err(_) => return None, // malformed csv → 回落文本（保守，不丢数据）
        }
    }
    Some(out)
}

/// 把响应头列表编码为 `name\x1evalue\x1e...` wire（空列表 → 空串）。
fn encode_headers(headers: &[(String, String)]) -> String {
    let mut out = String::new();
    for (i, (n, v)) in headers.iter().enumerate() {
        if i > 0 {
            out.push(HEADER_SEP);
        }
        out.push_str(n);
        out.push(HEADER_SEP);
        out.push_str(v);
    }
    out
}

/// 解码请求头 wire（`name\x1evalue\x1e...`）为 (name,value) 列表；奇数尾项忽略。
fn decode_headers(wire: &str) -> Vec<(String, String)> {
    if wire.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = wire.split(HEADER_SEP).collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < parts.len() {
        out.push((parts[i].to_string(), parts[i + 1].to_string()));
        i += 2;
    }
    out
}

/// 序列化 [`FetchResponse`] 为 host→JS wire（`__zwfr:` + status + `\x1f` + ... + body）。
/// R3021：二进制 body（非 UTF-8）经 `__zw_bytes:` csv-decimal wire（与请求侧对称，response.blob()/
/// arrayBuffer() 二进制保真）；UTF-8 body 原样文本（高效 + 向后兼容）。
fn serialize_response(resp: &FetchResponse) -> String {
    let body_field = match &resp.body_bytes {
        Some(bb) if std::str::from_utf8(bb).is_err() => encode_body_bytes(bb),
        _ => resp.body.clone(),
    };
    format!(
        "{WIRE_PREFIX}{status}{FIELD_SEP}{status_text}{FIELD_SEP}{headers}{FIELD_SEP}{body}",
        status = resp.status,
        status_text = resp.status_text,
        headers = encode_headers(&resp.headers),
        body = body_field,
    )
}

/// P1b S3 fetch bridge——共享 fetch 机制（handler cell + `__zw_fetch` 注册 + 非阻塞抓取）。
///
/// 各 app 在 `js_worker_main` 构造（传入包装自身 resolver 的 `AsyncResolver`），调
/// [`FetchBridge::register`] 注 `__zw_fetch(id, method, url, headersWire, body)` 回调；app 的
/// `SetFetchHandler` 命令 arm 调 [`FetchBridge::set_handler`] 注入生产 handler。`__zw_fetch` 回调
/// 非阻塞——子线程抓取 + `resolver.resolve` 回投——JS worker 不在 fetch 期间冻结。handler 未注入
/// 时子线程 resolve 错误标记（shim 落 Response.ok=false，不悬挂）。
pub struct FetchBridge {
    handler_cell: Arc<Mutex<Option<FetchHandler>>>,
    resolver: AsyncResolver,
}

impl FetchBridge {
    /// 构造——`resolver` 用于 `__zw_fetch` 抓取完成后 resolve Promise（复用 S1 通路）。
    pub fn new(resolver: AsyncResolver) -> Self {
        Self {
            handler_cell: Arc::new(Mutex::new(None)),
            resolver,
        }
    }

    /// 注入 fetch handler（各 app 的 `SetFetchHandler` 命令 arm 调用）。
    /// chicken-and-egg 解：app 在 js_worker spawn 后（WebView/net pool 就绪后）注入。
    pub fn set_handler(&self, handler: FetchHandler) {
        if let Ok(mut cell) = self.handler_cell.lock() {
            *cell = Some(handler);
        }
    }

    /// 注册 `__zw_fetch(id, method, url, headersWire, body)` 回调——JS `fetch(input, init)` 经 shim 调此。
    /// **非阻塞**：回调锁内仅克隆 handler Option（`FetchHandler=Arc` 廉价）后立即返，
    /// 子线程 `std::thread::spawn` 抓取（`h(&req)`）+ `resolver.resolve` 回投——JS worker 不冻结。
    /// handler 未注入时子线程 resolve 错误标记。
    pub fn register(&self, sandbox: &mut dyn Sandbox) {
        let handler_cell = Arc::clone(&self.handler_cell);
        let resolver = self.resolver.clone();
        sandbox.register_callback(
            "__zw_fetch",
            Box::new(move |args: &[String]| -> String {
                let id = args.first().cloned().unwrap_or_default();
                let method = args.get(1).cloned().unwrap_or_else(|| "GET".to_string());
                let url = args.get(2).cloned().unwrap_or_default();
                let headers = decode_headers(&args.get(3).cloned().unwrap_or_default());
                let body_raw = args.get(4).cloned().unwrap_or_default();
                // R3020：二进制 body 经 `__zw_bytes:` csv-decimal wire 解码为 body_bytes（Blob/FormData 二进制保真）；
                // 文本 body 原样入 body。body_bytes 与 body 互斥（二进制时 body=None）。
                let (body, body_bytes) = if body_raw.is_empty() {
                    (None, None)
                } else if let Some(bytes) = decode_body_bytes_raw(&body_raw) {
                    (None, Some(bytes))
                } else {
                    (Some(body_raw), None)
                };
                let req = FetchRequest {
                    url,
                    method,
                    headers,
                    body,
                    body_bytes,
                };
                let handler_opt: Option<FetchHandler> = handler_cell.lock().ok().and_then(|c| c.as_ref().cloned());
                let resolver = resolver.clone();
                std::thread::spawn(move || {
                    let result = match handler_opt {
                        Some(h) => match h(&req) {
                            Ok(resp) => serialize_response(&resp),
                            Err(e) => format!("{ERR_PREFIX}{e}"),
                        },
                        None => format!("{ERR_PREFIX}no-handler"),
                    };
                    resolver.resolve(&id, &result);
                });
                String::new()
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_wire_round_trip() {
        let hs = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("X-Test".to_string(), "a=b; c=d".to_string()),
        ];
        let wire = encode_headers(&hs);
        assert_eq!(wire, "Content-Type\x1eapplication/json\x1eX-Test\x1ea=b; c=d");
        let back = decode_headers(&wire);
        assert_eq!(back, hs);
    }

    #[test]
    fn decode_headers_empty_and_odd() {
        assert!(decode_headers("").is_empty());
        // 奇数尾项忽略（不可配对）。
        let odd = decode_headers("a\x1eb\x1ec");
        assert_eq!(odd, vec![("a".to_string(), "b".to_string())]);
    }

    #[test]
    fn response_wire_parses_body_with_field_sep() {
        // body 含 `\x1f`（末字段取第 3 个分隔符后全部，不被截断）。
        let resp = FetchResponse {
            status: 201,
            status_text: "Created".to_string(),
            headers: vec![("Location".to_string(), "/x/1".to_string())],
            body: "line1\x1fline2".to_string(),
            body_bytes: None,
        };
        let wire = serialize_response(&resp);
        assert!(wire.starts_with("__zwfr:201\x1fCreated\x1fLocation\x1e/x/1\x1f"));
        // body 末字段含 \x1f 完整保留（split 后 index≥3 全部 rejoin，与 shim indexOf 切片一致）。
        let parts: Vec<&str> = wire.split('\x1f').collect();
        let body_part = parts[3..].join("\x1f");
        assert_eq!(body_part, "line1\x1fline2");
    }

    #[test]
    fn response_ok_helper() {
        let r = FetchResponse::ok("hello");
        assert_eq!(r.status, 200);
        assert_eq!(r.status_text, "OK");
        assert!(r.headers.is_empty());
        assert_eq!(r.body, "hello");
    }

    #[test]
    fn body_bytes_wire_round_trip() {
        // R3020：csv-decimal byte-wire 往返——含非 UTF-8 字节（0xFF/0x00/0x80），二进制保真。
        let bytes = vec![0x48u8, 0x69, 0x00, 0x80, 0xFF, 0x0A, 0x2C]; // 含 ',' 字节本身（0x2C）须正确编解码
        let wire = encode_body_bytes(&bytes);
        assert_eq!(wire, "__zw_bytes:72,105,0,128,255,10,44");
        let back = decode_body_bytes_raw(&wire).expect("prefix wire 解码须成功");
        assert_eq!(back, bytes);
    }

    #[test]
    fn body_bytes_wire_empty_and_text_fallback() {
        // 空 byte 体 → 仅前缀 → Some([])。
        let empty = encode_body_bytes(&[]);
        assert_eq!(empty, "__zw_bytes:");
        assert_eq!(decode_body_bytes_raw(&empty), Some(Vec::new()));
        // 文本 body（无前缀）→ None（调用方按文本处理）。
        assert_eq!(decode_body_bytes_raw("plain text body"), None);
        assert_eq!(decode_body_bytes_raw(""), None);
        // malformed csv（超 u8 范围）→ None（保守回落文本，不丢数据）。
        assert_eq!(decode_body_bytes_raw("__zw_bytes:72,999"), None);
    }

    #[test]
    fn response_wire_binary_body_byte_wire_r3021() {
        // R3021：非 UTF-8 response body 经 __zw_bytes: csv-decimal wire（与请求侧对称）；UTF-8 body 原样文本。
        let bin = FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: Vec::new(),
            body: String::from_utf8_lossy(&[0xFF, 0x00, 0x80, 72, 105]).to_string(),
            body_bytes: Some(vec![0xFF, 0x00, 0x80, 72, 105]),
        };
        let wire = serialize_response(&bin);
        assert!(
            wire.ends_with("__zw_bytes:255,0,128,72,105"),
            "非 UTF-8 body 经 byte-wire：{wire}"
        );
        // body_bytes=None → 原样文本（向后兼容）。
        let txt = FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: Vec::new(),
            body: "hello".to_string(),
            body_bytes: None,
        };
        let wire2 = serialize_response(&txt);
        assert!(
            wire2.ends_with("hello") && !wire2.contains("__zw_bytes:"),
            "无 body_bytes → 文本：{wire2}"
        );
        // body_bytes 为 valid UTF-8 → 仍用文本（高效，避免无谓 byte-wire 开销）。
        let valid = FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: Vec::new(),
            body: "hello".to_string(),
            body_bytes: Some(b"hello".to_vec()),
        };
        let wire3 = serialize_response(&valid);
        assert!(
            wire3.ends_with("hello") && !wire3.contains("__zw_bytes:"),
            "valid-UTF-8 body_bytes 仍用文本：{wire3}"
        );
    }
}
