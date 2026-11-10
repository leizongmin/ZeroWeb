//! 杂项端点族——service-workers/cache-storage、common/redirect|blank、dom encoding、websockets security check、xmldecl trickle、load-error-events。

use super::*;

/// common/redirect.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_common_redirect(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // js-dom R141：dom/nodes/encoding.py（Document-characterSet-normalization 两文件
    // 654 subtest 经 `<iframe src="encoding.py?label=X">` 取子文档）——上游是
    // wptserve Python 脚本，wpt-data 无静态文件。此处内置等价生成器：读 ?label=
    // 参数，返回 `<!doctype html><meta charset="X">`（上游脚本逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/dom/nodes/encoding.py）。
    // s248：`/common/redirect.py?location=X`——上游 wptserve 行为等价：302 +
    // `Location`（绝对地址），fetch 管道按 HTTP-redirect fetch 跟随。canvas
    // security `.redirect` 族（addCrossOriginRedirectYellowImage——同源
    // redirect.py 跳跨源 yellow.png，origin-clean 按最终 URL origin 判定）
    // 依赖真实重定向跳；旧 200 伪重定向面（目标体直出 + X-Zero-Final-URL）
    // 无语料消费（R360 Document-URL 消费方为 js_dom_bridge_tests 单测
    // mock，不经本臂），就地升级。location 相对路径按当前 URL 原点解析。
    // https://github.com/web-platform-tests/wpt/blob/3159769/tools/wptserve/wptserve/handlers.py
    let loc = percent_encoding::percent_decode_str(
        path_part
            .split_once('?')
            .map(|(_, q)| q)
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or(""),
    )
    .decode_utf8_lossy()
    .split('&')
    .find_map(|kv| kv.strip_prefix("location="))
    .unwrap_or("")
    .to_string();
    let final_url = if loc.starts_with("http://") || loc.starts_with("https://") {
        loc
    } else {
        // 相对路径按请求原点解析（location.origin）。
        let origin = req
            .url
            .find("://")
            .and_then(|i| req.url[i + 3..].find('/').map(|p| i + 3 + p).or(Some(req.url.len())))
            .map(|end| req.url[..end].to_string())
            .unwrap_or_default();
        format!("{origin}{loc}")
    };
    let mut headers = Vec::new();
    headers.push(("location".into(), final_url));
    wpt_add_fetch_metadata(&mut headers, req, 302);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 302,
        status_text: "Found".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// dom/nodes/encoding.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_dom_encoding(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let label = percent_encoding::percent_decode_str(
        path_part
            .split_once('?')
            .map(|(_, q)| q)
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or(""),
    )
    .decode_utf8_lossy()
    .split('&')
    .find_map(|kv| kv.strip_prefix("label="))
    .unwrap_or("")
    .to_string();
    let body = format!("<!doctype html><meta charset=\"{label}\">");
    let mut headers = Vec::new();
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// encoding 资源 charset 端点族臂（wptfyi-runtime DC-2 encoding iframe 面）——
/// unsupported-labels.window.js / single-byte-decoder-iframe.window.js 的 iframe
/// 载体（wpt-data 无静态文件，上游是 wptserve Python 脚本）。两臂上游逐字等价：
/// - text-plain-charset.py：`Content-Type: text/plain;charset={label}` + body
///   "hello encoding"（supported label 面——传输 charset → document.characterSet）；
/// - text-html-meta-charset.py：`Content-Type: text/html`（无 charset）+ body
///   `<meta charset="{label}">hello encoding`（unsupported label 面——嗅探落空
///   回落 fallback encoding）。
///   https://github.com/web-platform-tests/wpt/blob/315976933870b34d6ea30e3f6643403edae678ba/encoding/resources/
pub(crate) fn route_encoding_charset(
    _ctx: &PyEndpointCtx,
    clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let label = percent_encoding::percent_decode_str(
        path_part
            .split_once('?')
            .map(|(_, q)| q)
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or(""),
    )
    .decode_utf8_lossy()
    .split('&')
    .find_map(|kv| kv.strip_prefix("label="))
    .unwrap_or("")
    .to_string();
    let (content_type, body) = if clean.ends_with("text-html-meta-charset.py") {
        (
            "text/html".to_string(),
            format!("<meta charset=\"{label}\">hello encoding"),
        )
    } else {
        (format!("text/plain;charset={label}"), "hello encoding".to_string())
    };
    let mut headers = Vec::new();
    headers.push(("content-type".into(), content_type));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// cache-storage vary.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_sw_vary(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let vary_value_override = &ctx.vary_value_override;
    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
    let mut headers = wpt_pipe_headers(query);
    // https://github.com/web-platform-tests/wpt/blob/24197a11e8c5bd29a5cb7bdf18135a82be8a8546/service-workers/cache-storage/resources/vary.py
    if query.split('&').any(|pair| pair == "clear-vary-value-override-cookie") {
        *vary_value_override.lock().unwrap() = None;
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "vary cookie cleared".to_string(),
            body_bytes: Some(b"vary cookie cleared".to_vec()),
        }));
    }
    if let Some(vary) = wpt_query_value(query, "set-vary-value-override-cookie") {
        *vary_value_override.lock().unwrap() = Some(vary);
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "vary cookie set".to_string(),
            body_bytes: Some(b"vary cookie set".to_vec()),
        }));
    }
    let omits_credentials = req.credentials.as_deref() == Some("omit");
    let override_vary = (!omits_credentials)
        .then(|| vary_value_override.lock().unwrap().clone())
        .flatten();
    if let Some(vary) = override_vary.or_else(|| wpt_query_value(query, "vary")) {
        headers.push(("vary".into(), vary));
    }
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "vary response".to_string(),
        body_bytes: Some(b"vary response".to_vec()),
    }))
}

/// cache-storage fetch-status.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_sw_fetch_status(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
    let status = wpt_query_value(query, "status")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(200);
    let mut headers = wpt_pipe_headers(query);
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// cors-approved.txt 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_sw_cors_approved(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let mut headers = vec![
        ("Access-Control-Allow-Origin".into(), "*".into()),
        ("content-type".into(), "text/plain".into()),
    ];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "plaintext\n".to_string(),
        body_bytes: Some(b"plaintext\n".to_vec()),
    }))
}

/// cors-denied.txt 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_sw_cors_denied(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let mut headers = vec![("content-type".into(), "text/plain".into())];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "Cross-origin request blocked by missing CORS response header\n".to_string(),
        body_bytes: Some(b"Cross-origin request blocked by missing CORS response header\n".to_vec()),
    }))
}

/// cache-storage redirect.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_sw_cache_redirect(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
    let status = wpt_query_value(query, "status")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(302);
    let mut headers = wpt_pipe_headers(query);
    headers.push((
        "Location".into(),
        "/service-workers/cache-storage/resources/simple.txt".into(),
    ));
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// common/blank.html 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_blank_html(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S22 fixture：common/blank.html（上游为最小空页；wpt-data 未拉
    // 静态本体——redirect 链暴露头面（expose-headers-on-redirect）与 ?pipe=
    // 管道头载体经此处等价供给）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers = wpt_pipe_headers(query);
    headers.push(("content-type".into(), "text/html".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "<!doctype html>\n<meta charset=\"utf-8\">\n<title>Blank</title>\n".to_string(),
        body_bytes: Some(b"<!doctype html>\n<meta charset=\"utf-8\">\n<title>Blank</title>\n".to_vec()),
    }))
}

/// websockets/security/check.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_ws_security_check(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M5-S1 fixture：websockets/security/check.py（上游逐字等价——
    // `b"FAIL" if b'Sec-WebSocket-Key' in request.headers else b"PASS"`）。
    // 该头是 Fetch 规范 forbidden header name——XHR setRequestHeader 须被
    // UA 剥离，服务端收到即 FAIL（security/002 断言面）。
    let has_key = req
        .headers
        .iter()
        .any(|(n, _)| n.eq_ignore_ascii_case("sec-websocket-key"));
    let body = if has_key { "FAIL" } else { "PASS" };
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }))
}

/// xmldecl trickle.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_xmldecl_trickle(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // html-syntax（2026-10-20，xmldecl 收编）：wptserve CGI 静态等价——上游
    // trickle.py 逐字节 trickle 输出固定正文（Content-Type: text/html 无
    // charset；编码由正文 bogo-XML declaration 嗅探，windows-1251 断言面）。
    // 同步 fetch 通道无流式语义，输出与上游脚本逐字一致（"declation" 为
    // 上游原文拼写）。正文含 0xE6 非 UTF-8 字节 → R3021 `__zw_bytes:` wire。
    let body: &[u8] = b"<?xml version=\"1.0\" encoding=\"windows-1251\"?>\n\
        <p>Normal XML declation as slow byte-by-byte trickle</p>\n\
        <p>Test: \xE6</p>\n\
        <p>If &#x0436;, XML decl takes effect</p>";
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/html".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::from_utf8_lossy(body).into_owned(),
        body_bytes: Some(body.to_vec()),
    }))
}

/// load-error-events.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_load_error_events(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // html-semantics M2 s100 fixture：经典脚本 load/error 事件族端点（上游
    // wptserve CGI 静态等价，语义见 wpt_load_error_events_response——语料
    // pull 面不含 .py，此前落通用缺文件 404：200 组动态 appendChild 案取回
    // 404 → error 路径误触发 → load-error-events-1 dynamic load 两行假红）。
    // 404 组依赖 `_r.ok` false → 门 throw → error 路径，status 保真即可。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let (status, body) = wpt_load_error_events_response(query);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/javascript".into()));
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
