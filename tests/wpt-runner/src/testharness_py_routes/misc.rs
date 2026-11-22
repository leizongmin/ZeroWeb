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

/// speculative-parsing stash.py 臂——wptfyi-html 批十二 fixtures（63 脚本头注释）。
pub(crate) fn route_speculative_stash(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let speculative_stash = &ctx.speculative_stash;
    // html/syntax/speculative-parsing/resources/stash.py 端点仿真（上游 wptserve
    // handler 不在语料，63 脚本头注释）：`?action=put&uuid=&encodingcheck=…` 记
    // `param-encodingcheck: <&encodingcheck= 之后 query 原文>\r\n` + 请求
    // Origin/Accept/Referer 头行（上游 raw_headers 精确名过滤；此处名匹配放宽为
    // 不分大小写、行内保留请求原始拼写——引擎头拼写实况入账，speculative 与
    // nonspeculative 两次 put 自洽比较语义不变）；其余 action（take）取走清空
    // （wptserve stash.take 缺键回 None → wptserve 空 200——miss 即空体）。
    // put 回空体。断言面 = 顶页两次 take 的头序列逐行相等（Referer 由页侧
    // split_header 归一 `/generated/…`）。
    // https://github.com/web-platform-tests/wpt/blob/
    // 315976933870b34d6ea30e3f6643403edae678ba/
    // html/syntax/speculative-parsing/resources/stash.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    if wpt_query_value(query, "action").as_deref() == Some("put") {
        // 上游 str.split("&encodingcheck=")[1]——标记缺省 IndexError → wptserve
        // 500（语料 put URL 恒带标记，此臂仅 1:1 保真）。
        let Some(encodingcheck) = query.split("&encodingcheck=").nth(1) else {
            wpt_add_fetch_metadata(&mut headers, req, 500);
            return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 500,
                status_text: "Internal Server Error".to_string(),
                headers,
                body: String::new(),
                body_bytes: None,
            }));
        };
        let recorded: Vec<String> = req
            .headers
            .iter()
            .filter(|(name, _)| {
                name.eq_ignore_ascii_case("origin")
                    || name.eq_ignore_ascii_case("accept")
                    || name.eq_ignore_ascii_case("referer")
            })
            .map(|(name, value)| format!("{name}: {value}"))
            .collect();
        let uuid = wpt_query_value(query, "uuid").unwrap_or_default();
        if let Ok(mut stash) = speculative_stash.lock() {
            stash.insert(
                uuid,
                format!("param-encodingcheck: {encodingcheck}\r\n{}", recorded.join("\r\n")),
            );
        }
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    let uuid = wpt_query_value(query, "uuid").unwrap_or_default();
    let taken = speculative_stash
        .lock()
        .ok()
        .and_then(|mut stash| stash.remove(&uuid))
        .unwrap_or_default();
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        // R3021 wire 约定：valid-UTF-8 body_bytes 走 body 文本（serialize_response
        // 取 body.clone()）——body 缺设则 JS fetch text() 恒空。批十三勘误：批十二
        // 首发漏设 body，take 内容从未上线（页断言两空 equals 空转），批十二证据
        // 的 50 Pass 归因随之勘误（空 take 支撑的空等空分支），批十三重跑重记。
        body: taken.clone(),
        body_bytes: Some(taken.into_bytes()),
    }))
}

/// common/slow.py 臂——wptfyi-html 批十二 fixtures。
pub(crate) fn route_slow_py(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // 上游 wptserve handler：sleep `?delay=` 毫秒后回空体（framed 页脚本 stall
    // 给扫描器留预取窗；nonspec 助页「block the load event for a bit」）。runner
    // in-process fetch 无真实网络时序，即返空体——页断言不依赖 delay 值：nonspec
    // 侧 put 已由 img 子资源请求先于 iframe load 事件同步保证，speculative 侧
    // 预取窗语义随批十二 leg-2 扫描器刀另行处置（FIXME(wptfyi-html leg-2)）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

// ===== wptfyi-html 批十三：query-encoding 族四端点（批十挂账 .py fixture 备料） =====
//
// html/infrastructure/urls/resolving-urls/query-encoding/resources/ 的
// resource.py / blank.py / stash.py / css-tmpl.py——resolve-url.js 15 面 × 9 基页
// 文档编码矩阵的被测端点（此前未仿真，静态回退 404，批十证据挂账）。上游
// wptserve handler 逐行 1:1：
// https://github.com/web-platform-tests/wpt/blob/
// 315976933870b34d6ea30e3f6643403edae678ba/
// html/infrastructure/urls/resolving-urls/query-encoding/resources/
// 面结果仍受引擎门（iframe/doc.write、worker 通道、CSSOM、canPlayType）制约，
// 备料收益兑现待碰头刀（master.md 批十刀位余量）。

/// 上游 `re.search(r'q=([^&]+)', request.url_parts.query)`——query 首个 `q=`
/// 起至 `&` 的原文（percent-encoding 保持不解码——测试目的即引擎发上线的原文
/// 字节形态）。q 缺省上游 AttributeError → wptserve 500，此处回 None 由臂转。
fn query_encoding_raw_q(query: &str) -> Option<&str> {
    let rest = &query[query.find("q=")? + 2..];
    Some(match rest.find('&') {
        Some(end) => &rest[..end],
        None => rest,
    })
}

/// 上游 handler 异常臂（KeyError/AttributeError/LookupError/资产缺档）→
/// wptserve 500 空态。
fn query_encoding_server_error(
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> zero_engine::fetch_bridge::FetchResponse {
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 500);
    zero_engine::fetch_bridge::FetchResponse {
        status: 500,
        status_text: "Internal Server Error".to_string(),
        headers,
        body: String::new(),
        body_bytes: None,
    }
}

/// Python `str.encode(encoding)`。文本恒 ASCII（worker 引导脚本模板/PI 表/
/// css 面模板 + uuid + q percent-encoding 全 ASCII——resolve-url.js 构造保证）：
/// ASCII 兼容编码字节直通；utf-16be/le 逐 char 双字节展开（Python codec 无
/// BOM——encoding_rs 的 output_encoding 把 utf-16be/le 重定向 UTF-8，语义相反，
/// 故 std 展开而非 encoding_rs）。标签未知 → None（上游 LookupError → 500）。
fn query_encoding_encode(encoding: &str, text: &str) -> Option<Vec<u8>> {
    let enc = encoding_rs::Encoding::for_label(encoding.as_bytes())?;
    if enc == encoding_rs::UTF_16LE {
        return Some(text.encode_utf16().flat_map(u16::to_le_bytes).collect());
    }
    if enc == encoding_rs::UTF_16BE {
        return Some(text.encode_utf16().flat_map(u16::to_be_bytes).collect());
    }
    Some(text.as_bytes().to_vec())
}

/// 响应构造——R3021 wire 约定 body 双设：文本体走 `body`（valid-UTF-8 时
/// serialize_response 取 body.clone()），二进制体（png/video）走 body_bytes
/// 字节线（route_common_redirect 同款）。
fn query_encoding_bytes_response(
    req: &zero_engine::fetch_bridge::FetchRequest,
    content_type: String,
    body: Vec<u8>,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), content_type));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::from_utf8_lossy(&body).into_owned(),
        body_bytes: Some(body),
    }))
}

/// blank.py——`Content-Type: text/html; charset=<encoding>` 空体 200（base-href
/// 面 iframe 载体页 + history 面导航靶；charset 头驱动引擎对空文档的解码）。
pub(crate) fn route_query_encoding_blank(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let encoding = wpt_query_value(query, "encoding").unwrap_or_default();
    query_encoding_bytes_response(req, format!("text/html; charset={encoding}"), Vec::new())
}

/// css-tmpl.py——`sheet = <tmpl> % '\0000E5'`（Python %-format：`%s` 占位 →
/// 7 字符 `\0000E5` 字面）后按 `encoding` 编码为字节体（css 面：样式表编码
/// 驱动引擎对 sheet 内 `url(stash.py?q=%s…)` 的转义解码与 q 重序列化形态——
/// stash 落的是什么，轮询面断言 expected_utf8）。tmpl 按上游
/// isomorphic_decode 逐字节 latin-1 字符化（`wpt_query_value_bytes` 同语义）。
pub(crate) fn route_query_encoding_css_tmpl(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let encoding = wpt_query_value(query, "encoding").unwrap_or_default();
    let tmpl = wpt_query_value_bytes(query, "tmpl").unwrap_or_default();
    let sheet = tmpl.replace("%s", "\\0000E5");
    let Some(body) = query_encoding_encode(&encoding, &sheet) else {
        return Some(Ok(query_encoding_server_error(req)));
    };
    query_encoding_bytes_response(req, format!("text/css; charset={encoding}"), body)
}

/// stash.py——`?id=` 键 put/take（put 覆写记 q 原文；take 取走清空，miss 回
/// 无 Content-Type 空体 200——上游 `return [], u""`）。css/svg 面 poll_for_stash
/// 每 200ms take 直至命中的消费方；案级 query_encoding_stash 承载（wptserve
/// server.stash 语义，per-case 构建、案内键自清、跨案零泄漏）。
pub(crate) fn route_query_encoding_stash(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let id = wpt_query_value(query, "id").unwrap_or_default();
    if wpt_query_value(query, "action").as_deref() == Some("put") {
        let Some(q) = query_encoding_raw_q(query) else {
            return Some(Ok(query_encoding_server_error(req)));
        };
        if let Ok(mut stash) = ctx.query_encoding_stash.lock() {
            stash.insert(id, q.to_string());
        }
        // put 响应体不被断言面消费（表单靶/忽略——上游 `u'Put %s' % q` 同为
        // "Put <q 原文>" 1:1）。
        return query_encoding_bytes_response(req, "text/html".into(), format!("Put {q}").into_bytes());
    }
    let taken = ctx
        .query_encoding_stash
        .lock()
        .ok()
        .and_then(|mut stash| stash.remove(&id));
    let Some(q) = taken else {
        // miss——无 Content-Type（上游空头列表）。
        let mut headers: Vec<(String, String)> = Vec::new();
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    };
    query_encoding_bytes_response(req, "text/html".into(), q.into_bytes())
}

/// worker_importScripts 引导脚本——上游 b""" 模板逐字节（缩进原样；`?q=…` 处
/// 为 JS 源码 6 字符转义序列 `\u00E5` 字面、非 U+00E5 字符，传输编码无关；
/// 唯一 `%s` 由 encoding 填入）。
const QUERY_ENCODING_WORKER_IMPORTSCRIPTS_JS: &str = r#"try {
                     var x = 'importScripts failed to run';
                     importScripts('?q=\u00E5&type=js&var=x&encoding=%s');
                     postMessage(x);
                     close();
                   } catch(ex) {
                     postMessage(String(ex));
                   }"#;

/// worker_worker 引导脚本——上游 b""" 模板逐字节。
const QUERY_ENCODING_WORKER_WORKER_JS: &str = r#"try {
                     var worker = new Worker('?q=\u00E5&type=worker&encoding=%s');
                     worker.onmessage = function(e) {
                       postMessage(e.data);
                       close();
                     };
                   } catch(ex) {
                     postMessage(String(ex));
                   }"#;

/// sharedworker_importScripts 引导脚本——上游 b""" 模板逐字节。
const QUERY_ENCODING_SHAREDWORKER_IMPORTSCRIPTS_JS: &str = r#"var x = 'importScripts failed to run';
                     onconnect = function(e) {
                     var connect_port = e.source;
                     try {
                       importScripts('?q=\u00E5&type=js&var=x&encoding=%s');
                       connect_port.postMessage(x);
                       close();
                     } catch(ex) {
                       connect_port.postMessage(String(ex));
                     }
                   };"#;

/// sharedworker_worker 引导脚本——上游 b""" 模板逐字节。
const QUERY_ENCODING_SHAREDWORKER_WORKER_JS: &str = r#"onconnect = function(e) {
                     var connect_port = e.source;
                     try {
                       var worker = new Worker('?q=\u00E5&type=worker&encoding=%s');
                       worker.onmessage = function(e) {
                         connect_port.postMessage(e.data);
                         close();
                       };
                     } catch(ex) {
                       connect_port.postMessage(String(ex));
                     }
                   };"#;

/// resource.py——resolve-url.js 多型资源端点（q 原文按 type 回显为 html/css/
/// js/worker/sharedworker/eventstream/svg/webvtt 文本体、worker 引导脚本四型
/// 按 encoding 编码、xmlstylesheet_css PI 表、png/video 盘内资产直读）。未知
/// type / q 缺省 / 资产缺档 → 500（上游落 None 或异常 → wptserve 500；
/// worker_sharedworker / sharedworker_sharedworker 在 resolve-url.js 仅构造
/// 不取用，上游亦无分支）。
pub(crate) fn route_query_encoding_resource(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let Some(q) = query_encoding_raw_q(query) else {
        return Some(Ok(query_encoding_server_error(req)));
    };
    let qtype = wpt_query_value(query, "type").unwrap_or_default();
    let encoding = wpt_query_value(query, "encoding").unwrap_or_default();
    // 文本体分支——上游 isomorphic_encode(q) 即 q 原文（ASCII percent-encoding
    // 直书）。
    match qtype.as_str() {
        "html" => query_encoding_bytes_response(req, "text/html; charset=utf-8".into(), q.as_bytes().to_vec()),
        "css" => query_encoding_bytes_response(
            req,
            "text/css; charset=utf-8".into(),
            format!("#test::before {{ content:'{q}' }}").into_bytes(),
        ),
        "js" => {
            let var = wpt_query_value(query, "var").unwrap_or_default();
            query_encoding_bytes_response(
                req,
                "text/javascript; charset=utf-8".into(),
                format!("{var} = '{q}';").into_bytes(),
            )
        }
        "worker" => query_encoding_bytes_response(
            req,
            "text/javascript".into(),
            format!("postMessage('{q}'); close();").into_bytes(),
        ),
        "sharedworker" => query_encoding_bytes_response(
            req,
            "text/javascript".into(),
            format!("onconnect = function(e) {{ e.source.postMessage('{q}'); close(); }};").into_bytes(),
        ),
        "eventstream" => {
            query_encoding_bytes_response(req, "text/event-stream".into(), format!("data: {q}\n\n").into_bytes())
        }
        "svg" => query_encoding_bytes_response(
            req,
            "image/svg+xml".into(),
            format!("<svg xmlns='http://www.w3.org/2000/svg'>{q}</svg>").into_bytes(),
        ),
        "webvtt" => query_encoding_bytes_response(
            req,
            "text/vtt".into(),
            format!("WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n{q}").into_bytes(),
        ),
        // worker 引导脚本四型——charset 头对 worker 解码语义无效仍 1:1 回写。
        "worker_importScripts" | "worker_worker" | "sharedworker_importScripts" | "sharedworker_worker" => {
            let template = match qtype.as_str() {
                "worker_importScripts" => QUERY_ENCODING_WORKER_IMPORTSCRIPTS_JS,
                "worker_worker" => QUERY_ENCODING_WORKER_WORKER_JS,
                "sharedworker_importScripts" => QUERY_ENCODING_SHAREDWORKER_IMPORTSCRIPTS_JS,
                _ => QUERY_ENCODING_SHAREDWORKER_WORKER_JS,
            };
            let Some(body) = query_encoding_encode(&encoding, &template.replace("%s", &encoding)) else {
                return Some(Ok(query_encoding_server_error(req)));
            };
            query_encoding_bytes_response(req, format!("text/javascript; charset={encoding}"), body)
        }
        // xml-stylesheet PI 表（xml 面 iframe 载体）——表体全 ASCII（&#x00E5;/
        // &amp;/encoding 标签），encode 对各编码恒等，仍 1:1 走编码器。
        "xmlstylesheet_css" => {
            let sheet = format!(
                "<?xml-stylesheet href=\"?q=&#x00E5;&amp;type=css&amp;encoding={}\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"/>",
                encoding
            );
            let Some(body) = query_encoding_encode(&encoding, &sheet) else {
                return Some(Ok(query_encoding_server_error(req)));
            };
            query_encoding_bytes_response(req, format!("application/xhtml+xml; charset={encoding}"), body)
        }
        // png/video 盘内资产直读（images//media/ 在 html/ 子树外——green-2x2.png
        // 由 62 号拉取脚本显式补档）。q 原文 → 资产映射按上游逐分支；缺档 →
        // 500（上游 open() 异常）。
        "png" => {
            let image = match q {
                "%E5" | "%26%23229%3B" => "green-1x1.png",
                "%C3%A5" => "green-2x2.png",
                "%3F" => "green-16x16.png",
                _ => "green-256x256.png",
            };
            let Ok(bytes) = std::fs::read(ctx.root.join("images").join(image)) else {
                return Some(Ok(query_encoding_server_error(req)));
            };
            query_encoding_bytes_response(req, "image/png".into(), bytes)
        }
        "video" => {
            let ext = wpt_query_value(query, "ext").unwrap_or_default();
            let video = match q {
                "%E5" | "%26%23229%3B" => "A4",
                "%C3%A5" => "movie_5",
                "%3F" => "green-at-15",
                _ => "movie_300",
            };
            let Ok(bytes) = std::fs::read(ctx.root.join("media").join(format!("{video}.{ext}"))) else {
                return Some(Ok(query_encoding_server_error(req)));
            };
            // 上游 ogv → ogg 响应 MIME 修正。
            let mime_ext = if ext == "ogv" { "ogg" } else { &ext };
            query_encoding_bytes_response(req, format!("video/{mime_ext}"), bytes)
        }
        _ => Some(Ok(query_encoding_server_error(req))),
    }
}
