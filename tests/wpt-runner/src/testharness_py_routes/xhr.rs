//! XMLHttpRequest 端点族——upload/form/echo-content-type、CORS 判据与 preflight 缓存族、
//! authentication 状态机、redirect/corsenabled/status、get-set-cookie/asis。

use super::*;

/// xhr upload.py 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_upload(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S19 fixture：xhr/resources/upload.py（上游行为等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
    // upload.py）。解析 multipart/form-data 体：无 filename 的 part 按名排序
    // 输出 `key=value,`，随后 `\n`，再 filename part 按名排序输出
    // `key=filename:ctype:len,`。
    let boundary = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
        .and_then(|(_, v)| {
            let lower = v.to_lowercase();
            lower
                .find("boundary=")
                .map(|idx| v[idx + 9..].trim().trim_matches('"').to_string())
        })
        .unwrap_or_default();
    let body = req.body_bytes.clone().unwrap_or_default();
    let mut text_parts: Vec<(String, String)> = Vec::new();
    let mut file_parts: Vec<(String, String, String, usize)> = Vec::new();
    if !boundary.is_empty() && !body.is_empty() {
        let delim = format!("--{}", boundary);
        let delim_bytes = delim.as_bytes();
        let mut segments: Vec<&[u8]> = Vec::new();
        let mut start = 0;
        for i in 0..body.len() {
            if body[i..].starts_with(delim_bytes) {
                if i >= start + 2 {
                    segments.push(&body[start..i]);
                }
                start = i + delim_bytes.len();
            }
        }
        for seg in segments {
            // 每段：前导 \r\n + 头部 + \r\n\r\n + 体（尾随 \r\n 去除）。
            let seg = seg.strip_prefix(b"\r\n").unwrap_or(seg);
            let sep = match seg.windows(4).position(|w| w == b"\r\n\r\n") {
                Some(pos) => pos,
                None => continue,
            };
            let head = String::from_utf8_lossy(&seg[..sep]).to_lowercase();
            let mut value = &seg[sep + 4..];
            if value.ends_with(b"\r\n") {
                value = &value[..value.len() - 2];
            }
            let mut name = String::new();
            let mut filename: Option<String> = None;
            let mut ctype = String::new();
            for line in head.split("\r\n") {
                if let Some(cd) = line.strip_prefix("content-disposition:") {
                    for attr in cd.split(';') {
                        let attr = attr.trim();
                        if let Some(nv) = attr.strip_prefix("name=") {
                            name = nv.trim_matches('"').to_string();
                        } else if let Some(fv) = attr.strip_prefix("filename=") {
                            filename = Some(fv.trim_matches('"').to_string());
                        }
                    }
                } else if let Some(ct) = line.strip_prefix("content-type:") {
                    ctype = ct.trim().to_string();
                }
            }
            if name.is_empty() {
                continue;
            }
            match filename {
                Some(fname) => file_parts.push((
                    name,
                    fname,
                    if ctype.is_empty() { "None".into() } else { ctype },
                    value.len(),
                )),
                None => text_parts.push((name, String::from_utf8_lossy(value).into_owned())),
            }
        }
    }
    text_parts.sort_by(|a, b| a.0.cmp(&b.0));
    file_parts.sort_by(|a, b| a.0.cmp(&b.0));
    let mut content = String::new();
    for (k, v) in &text_parts {
        content.push_str(&format!("{}={},", k, v));
    }
    content.push('\n');
    for (k, f, ct, len) in &file_parts {
        content.push_str(&format!("{}={}:{}:{},", k, f, ct, len));
    }
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: content.clone(),
        body_bytes: Some(content.into_bytes()),
    }));
}

/// xhr form.py 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_form(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S21 fixture：xhr/resources/form.py（上游行为等价——multipart
    // 体按文档序回显 `name:value;`；text part 提取与 upload.py 同型）。
    let boundary = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
        .and_then(|(_, v)| {
            let lower = v.to_lowercase();
            lower
                .find("boundary=")
                .map(|idx| v[idx + 9..].trim().trim_matches('"').to_string())
        })
        .unwrap_or_default();
    let body = req.body_bytes.clone().unwrap_or_default();
    let mut content = String::new();
    if !boundary.is_empty() && !body.is_empty() {
        let delim = format!("--{}", boundary);
        let delim_bytes = delim.as_bytes();
        let mut segments: Vec<&[u8]> = Vec::new();
        let mut start = 0;
        for i in 0..body.len() {
            if body[i..].starts_with(delim_bytes) {
                if i >= start + 2 {
                    segments.push(&body[start..i]);
                }
                start = i + delim_bytes.len();
            }
        }
        for seg in segments {
            let seg = seg.strip_prefix(b"\r\n").unwrap_or(seg);
            let sep = match seg.windows(4).position(|w| w == b"\r\n\r\n") {
                Some(pos) => pos,
                None => continue,
            };
            let head = String::from_utf8_lossy(&seg[..sep]).to_lowercase();
            let mut value = &seg[sep + 4..];
            if value.ends_with(b"\r\n") {
                value = &value[..value.len() - 2];
            }
            let mut name = String::new();
            let mut has_filename = false;
            for line in head.split("\r\n") {
                if let Some(cd) = line.strip_prefix("content-disposition:") {
                    for attr in cd.split(';') {
                        let attr = attr.trim();
                        if let Some(nv) = attr.strip_prefix("name=") {
                            name = nv.trim_matches('"').to_string();
                        } else if attr.starts_with("filename=") {
                            has_filename = true;
                        }
                    }
                }
            }
            // 上游 form.py 对 text part 回显（file part 跳过——本次用例无）。
            if !name.is_empty() && !has_filename {
                content.push_str(&format!("{}:{};", name, String::from_utf8_lossy(value)));
            }
        }
    }
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: content.clone(),
        body_bytes: Some(content.into_bytes()),
    }));
}

/// xhr echo-content-type.py 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_echo_content_type(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S21 fixture：xhr/resources/echo-content-type.py（上游行为
    // 等价——回显请求 Content-Type 头，content-type-unmodified 面）。
    let ct = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: ct.clone(),
        body_bytes: Some(ct.into_bytes()),
    }));
}

/// xhr access-control-origin-header 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_access_control_origin_header(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S21 fixture：xhr/resources/access-control-origin-header.py
    //（上游行为等价——ACAO 回显 Origin + body "PASS: Cross-domain access
    // allowed.\nHTTP_ORIGIN: <origin>"，access-control-basic-allow- 面）。
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    headers.push(("access-control-allow-origin".into(), origin.clone()));
    headers.push(("access-control-allow-credentials".into(), "true".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let body = format!("PASS: Cross-domain access allowed.\nHTTP_ORIGIN: {}", origin);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr over-1-meg.txt 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_over_1_meg(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S21 fixture：xhr/resources/over-1-meg.txt（上游行为等价——
    // 1.16MB "abcd" 重复体，over-1-meg 面）。
    let body = "abcd".repeat(290_000);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr access-control-basic-allow-star 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_basic_allow_star(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S17 fixture：xhr/resources/access-control-basic-allow-star.py
    //（上游逐字等价——ACAO * + PASS 体）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    headers.push(("access-control-allow-origin".into(), "*".into()));
    let body = "PASS: Cross-domain access allowed.".to_string();
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr access-control-basic-allow(-no-credentials) 族 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_basic_allow(
    _ctx: &PyEndpointCtx,
    clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S17 fixture：xhr/resources/access-control-basic-allow.py 与
    // access-control-basic-allow-no-credentials.py（上游逐字等价——ACAO 回显
    // 请求 Origin；allow 变体另带 ACAC true，no-credentials 变体不带）。
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| "*".to_string());
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    headers.push(("access-control-allow-origin".into(), origin));
    if clean.ends_with("/access-control-basic-allow.py") {
        headers.push(("access-control-allow-credentials".into(), "true".into()));
    }
    let body = "PASS: Cross-domain access allowed.".to_string();
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr redirect-cors 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_xhr_redirect_cors(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S17 fixture：xhr/resources/redirect-cors.py（上游行为等价——
    // GET → 302 + Location；?allow_origin= 回显 Origin、?allow_header= 透传
    // ACAH；OPTIONS preflight 面待 engine preflight 语义，此实现 GET 面）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let location = wpt_query_value(query, "location").unwrap_or_default();
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    if req.method == "OPTIONS" {
        // 上游语义：preflight 不重定向（redirect_preflight 才 302）+ ACAM/Max-Age。
        headers.push(("access-control-allow-methods".into(), "GET".into()));
        headers.push(("access-control-max-age".into(), "1".into()));
        if wpt_query_value(query, "allow_origin").is_some() {
            headers.push(("access-control-allow-origin".into(), origin));
        }
        if let Some(allow_header) = wpt_query_value(query, "allow_header") {
            headers.push(("access-control-allow-headers".into(), allow_header));
        }
        let preflight_redirect = wpt_query_value(query, "redirect_preflight").is_some();
        let status: u16 = if preflight_redirect { 302 } else { 200 };
        if preflight_redirect {
            headers.push(("location".into(), location));
        }
        wpt_add_fetch_metadata(&mut headers, req, status);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status,
            status_text: wpt_status_text(status).to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    headers.push(("location".into(), location));
    if wpt_query_value(query, "allow_origin").is_some() {
        headers.push(("access-control-allow-origin".into(), origin));
    }
    if let Some(allow_header) = wpt_query_value(query, "allow_header") {
        headers.push(("access-control-allow-headers".into(), allow_header));
    }
    wpt_add_fetch_metadata(&mut headers, req, 302);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 302,
        status_text: wpt_status_text(302).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }));
}

/// xhr authentication 族谓词——authentication.py 全等 + authN 目录三委托端点。
pub(crate) fn pred_xhr_authentication(clean: &str, _req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
    clean == "xhr/resources/authentication.py"
        || (clean.starts_with("xhr/resources/auth")
            && (clean.ends_with("/auth.py")
                || clean.ends_with("/corsenabled.py")
                || clean.ends_with("/corsenabled-no-authorize.py")))
}

/// xhr authentication.py + authN 三委托端点（Custom 谓词） 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_authentication(
    _ctx: &PyEndpointCtx,
    clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api-compat-v3 M2 fixture：xhr/resources/authentication.py 与
    // auth1-11/{auth.py, corsenabled.py, corsenabled-no-authorize.py}
    //（上游逐字——https://github.com/web-platform-tests/wpt/blob/
    // 3159769/xhr/resources/authentication.py；authN/*.py 皆委托
    // authentication.py，N 目录仅隔离 UA auth 会话缓存）：
    // 无 Authorization（Basic）→ token(X-User 头) stash 已记 → 200
    // "FAIL (did not authorize)"；否则 stash 记 token 后 401
    // Unauthorized + `WWW-Authenticate: Basic realm="test"`（体 "FAIL
    // (should be transparent)"——UA 凭 open()/URL 凭据透明重试后
    // 不可见）。有 Authorization → x-challenge DID/DID-NOT（stash take
    // 判定）+ XHR-USER/SES-USER 头 + 体 `user\npass`。corsenabled 变体
    // 加 CORS 应答（ACAO 回显 Origin + ACAC true + ACAM GET + ACAH
    // "authorization, x-user, x-pass"（no-authorize 变体无 authorization）
    // + expose x-challenge/xhr-user/ses-user），OPTIONS → 200 空体。
    let req_header = |name: &str| -> Option<String> {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    let token = req_header("x-user");
    let credentials = req_header("authorization").and_then(|v| {
        let b64 = v.strip_prefix("Basic ").or_else(|| v.strip_prefix("basic "))?;
        use base64::Engine as _;
        let decoded = base64::engine::general_purpose::STANDARD.decode(b64.trim()).ok()?;
        let (user, pass) = String::from_utf8_lossy(&decoded)
            .split_once(':')
            .map(|(u, p)| (u.to_string(), p.to_string()))?;
        Some((user, pass))
    });
    let corsenabled = clean.ends_with("/corsenabled.py") || clean.ends_with("/corsenabled-no-authorize.py");
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    if corsenabled {
        headers.push((
            "access-control-allow-origin".into(),
            req_header("origin").unwrap_or_default(),
        ));
        headers.push(("access-control-allow-credentials".into(), "true".into()));
        headers.push(("access-control-allow-methods".into(), "GET".into()));
        headers.push((
            "access-control-allow-headers".into(),
            if clean.ends_with("/corsenabled-no-authorize.py") {
                "x-user, x-pass".into()
            } else {
                "authorization, x-user, x-pass".into()
            },
        ));
        headers.push((
            "access-control-expose-headers".into(),
            "x-challenge, xhr-user, ses-user".into(),
        ));
        if req.method == "OPTIONS" {
            wpt_add_fetch_metadata(&mut headers, req, 200);
            return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: String::new(),
                body_bytes: Some(Vec::new()),
            }));
        }
    }
    let Some((user, pass)) = credentials else {
        let stash_hit = token
            .as_ref()
            .and_then(|t| WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(t)))
            .is_some();
        if stash_hit {
            let body = "FAIL (did not authorize)".to_string();
            wpt_add_fetch_metadata(&mut headers, req, 200);
            return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: body.clone(),
                body_bytes: Some(body.into_bytes()),
            }));
        }
        if let Some(t) = &token
            && let Ok(mut stash) = WPT_TOKEN_STASH.lock()
        {
            stash.insert(t.clone(), "1".into());
        }
        headers.push(("www-authenticate".into(), "Basic realm=\"test\"".into()));
        let body = "FAIL (should be transparent)".to_string();
        wpt_add_fetch_metadata(&mut headers, req, 401);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 401,
            status_text: "Unauthorized".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        }));
    };
    let challenge = token
        .as_ref()
        .and_then(|t| WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(t)))
        .is_some_and(|v| v == "1");
    headers.push(("x-challenge".into(), if challenge { "DID" } else { "DID-NOT" }.into()));
    if let Some(t) = &token {
        headers.push(("xhr-user".into(), t.clone()));
    }
    headers.push(("ses-user".into(), user.clone()));
    let body = format!("{}\n{}", user, pass);
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr image.gif 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_image_gif(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S20 fixture：xhr/resources/image.gif（responseurl 重定向
    // 目标——1×1 GIF 43 字节标准头尾）。
    let body: Vec<u8> = vec![
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff,
        0xff, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3b,
    ];
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "image/gif".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(body),
    }));
}

/// xhr trickle.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_trickle(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S21 fixture：xhr/resources/trickle.py（上游行为等价——count 块
    // × 每块 13 字节（与 fetch/api/resources/trickle.py 同 "TEST_TRICKLE\n" 块形，
    // 单测锚定该体）；specifylength=1 时补 Content-Length: <total>。逐块延迟
    // 由 finite 模型折叠——响应一次到位，progress 事件由 shim fill 侧派发）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let count = wpt_query_value(query, "count")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(10)
        .min(128);
    let body = "TEST_TRICKLE\n".repeat(count);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    if path_part.contains("specifylength=1") {
        headers.push(("content-length".into(), (count * 13).to_string()));
    }
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr top.txt 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_top_txt(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S18 fixture：xhr/resources/top.txt（cors-expose-star 载体，
    // ?pipe=header(...) 管道头经既有 wpt_pipe_headers 处理）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers = wpt_pipe_headers(query);
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "top\n".to_string(),
        body_bytes: Some(b"top\n".to_vec()),
    }));
}

/// xhr access-control-basic-put-allow 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_basic_put_allow(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S22 fixture：xhr/resources/access-control-basic-put-allow.py
    //（上游逐字等价——OPTIONS: ACAM PUT + ACAO 回显 + ACAC；PUT: ACAO 回显 +
    // 体 "PASS: Cross-domain access allowed.\n<body>"，non-cors-safelisted
    // method 面）。
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    if req.method == "OPTIONS" {
        headers.push(("access-control-allow-credentials".into(), "true".into()));
        headers.push(("access-control-allow-methods".into(), "PUT".into()));
        headers.push(("access-control-allow-origin".into(), origin));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    if req.method == "PUT" {
        headers.push(("access-control-allow-credentials".into(), "true".into()));
        headers.push(("access-control-allow-origin".into(), origin));
        let body = format!(
            "PASS: Cross-domain access allowed.\n{}",
            req.body_bytes
                .as_ref()
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .or_else(|| req.body.clone())
                .unwrap_or_else(|| "Could not read in content.".into())
        );
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        }));
    }
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: format!("Wrong method: {}", req.method),
        body_bytes: Some(format!("Wrong method: {}", req.method).into_bytes()),
    }));
}

/// xhr reset-token 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_reset_token(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S22 fixture：xhr/resources/reset-token.py（上游逐字等价——
    // stash[token] 置空 + "PASS"，preflight-cache 族用例起点）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let token = wpt_query_value(query, "token").unwrap_or_default();
    if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
        stash.insert(token, String::new());
    }
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    headers.push(("access-control-allow-origin".into(), origin));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "PASS".to_string(),
        body_bytes: Some(b"PASS".to_vec()),
    }));
}

/// xhr preflight-cache 三联（DirSuffixes） 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_preflight_cache(
    _ctx: &PyEndpointCtx,
    clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S22 fixture：xhr/resources/access-control-basic-preflight-cache*
    // （上游逐字等价的 token 状态机——OPTIONS 到达序断言 preflight cache 行为：
    // cache.py 二请求须复用缓存、timeout.py 须过期重发、invalidation.py 须
    // method/header 变更失效。状态迁移见上游 .py 三件）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let token = wpt_query_value(query, "token").unwrap_or_default();
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let is_timeout = clean.ends_with("-timeout.py");
    let is_invalidation = clean.ends_with("-invalidation.py");
    // stash take 语义（上游 request.server.stash.take）——缺省或空值均视
    // Uninitialized（Python falsy：reset-token.py put 的是 b""）。
    let state = if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
        stash.remove(&token).unwrap_or_default()
    } else {
        String::new()
    };
    let state = if state.is_empty() {
        "Uninitialized".to_string()
    } else {
        state
    };
    let method = req.method.clone();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    headers.push(("access-control-allow-origin".into(), origin));
    headers.push(("access-control-allow-credentials".into(), "true".into()));
    let (status, body, next_state) = match state.as_str() {
        "Uninitialized" => {
            if method == "OPTIONS" {
                headers.push(("access-control-allow-methods".into(), "PUT".into()));
                if is_timeout {
                    headers.push(("access-control-allow-headers".into(), "x-test".into()));
                }
                headers.push((
                    "access-control-max-age".into(),
                    if is_timeout { "1".into() } else { "10".into() },
                ));
                (200, String::new(), "OPTIONSSent".to_string())
            } else {
                (400, format!("FAIL {method}: Uninitialized"), state)
            }
        }
        "OPTIONSSent" => {
            if method == "PUT" {
                (200, "PASS: First PUT request.".to_string(), "FirstPUTSent".to_string())
            } else {
                (400, format!("FAIL {method}: OPTIONSSent"), state)
            }
        }
        "FirstPUTSent" => {
            if method == "PUT" && !is_timeout && !is_invalidation {
                (200, "PASS: Second PUT request. Preflight worked.".to_string(), state)
            } else if method == "OPTIONS" {
                headers.push((
                    "access-control-allow-methods".into(),
                    if is_invalidation {
                        "PUT, XMETHOD".into()
                    } else {
                        "PUT".into()
                    },
                ));
                if is_timeout || is_invalidation {
                    headers.push(("access-control-allow-headers".into(), "x-test".into()));
                }
                (200, String::new(), "SecondOPTIONSSent".to_string())
            } else {
                (
                    400,
                    format!("FAIL {method}: Second PUT request sent without preflight"),
                    state,
                )
            }
        }
        "SecondOPTIONSSent" => {
            if method == "PUT" || method == "XMETHOD" {
                (200, "PASS: Second OPTIONS request was sent.".to_string(), state)
            } else {
                (400, format!("FAIL {method}: SecondOPTIONSSent"), state)
            }
        }
        _ => (400, format!("FAIL {method}: {state}"), state),
    };
    if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
        stash.insert(token, next_state);
    }
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr preflight-request-*-returns-* 对（DirSuffixes） 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_preflight_returns(
    _ctx: &PyEndpointCtx,
    clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S22 fixture：access-control-preflight-request-*-returns-*.py
    //（上游逐字等价——OPTIONS: ACAO 回显（或 *）+ ACAH X-Test（或 *）；
    // GET: ACAO * + X-Test 在 → "PASS"，否则 400）。
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let star_variant = clean.ends_with("-allow-headers-returns-star.py");
    let mut headers: Vec<(String, String)> = Vec::new();
    if req.method == "OPTIONS" {
        headers.push((
            "access-control-allow-origin".into(),
            if star_variant { "*".into() } else { origin },
        ));
        headers.push((
            "access-control-allow-headers".into(),
            if star_variant { "*".into() } else { "X-Test".into() },
        ));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    headers.push(("access-control-allow-origin".into(), "*".into()));
    let has_x_test = req.headers.iter().any(|(n, _)| n.eq_ignore_ascii_case("x-test"));
    if has_x_test {
        headers.push(("content-type".into(), "text/plain".into()));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "PASS".to_string(),
            body_bytes: Some(b"PASS".to_vec()),
        }));
    }
    wpt_add_fetch_metadata(&mut headers, req, 400);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 400,
        status_text: wpt_status_text(400).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }));
}

/// xhr headers.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_headers(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/headers.py（上游逐字等价——
    // getResponseHeader 响应头控制面：同名双值 comma join、空值、UTF-8
    // 字节值（上游 `u"…".encode("utf-8")` 的 ByteString 头值视图即三字符
    // U+00E2 U+0080 U+00A6）与 Set-Cookie 隐藏面；Server/Date 为 wptserve
    // 每响应自动头，fixture 固定合法 IMF-fixdate——server-date 案仅断言
    // 非 null）。
    let mut headers: Vec<(String, String)> = vec![
        ("content-type".into(), "text/plain".into()),
        ("x-custom-header".into(), "test".into()),
        ("set-cookie".into(), "test".into()),
        ("set-cookie2".into(), "test".into()),
        ("x-custom-header-empty".into(), String::new()),
        ("x-custom-header-comma".into(), "1".into()),
        ("x-custom-header-comma".into(), "2".into()),
        ("x-custom-header-bytes".into(), "\u{00e2}\u{0080}\u{00a6}".into()),
        ("server".into(), "SimpleHTTP/0.6 Python/3.12".into()),
        ("date".into(), "Sun, 06 Nov 1994 08:49:37 GMT".into()),
    ];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "TEST".to_string(),
        body_bytes: Some(b"TEST".to_vec()),
    }));
}

/// xhr chunked.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_chunked(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/chunked.py（上游语义等价——Trailer
    // 声明 + chunked 帧体；runner 管道无传输层，体直给解帧后拼接串，
    // trailer 头不并入响应头——chunked-trailer 案断言 Trailer 可见而
    // X-Test-Me 不可见）。
    let body = "First chunk\r\nSecond chunk\r\nYet another (third) chunk\r\n\
                Yet another (fourth) chunk\r\n";
    let mut headers: Vec<(String, String)> = vec![
        ("transfer-encoding".into(), "chunked".into()),
        ("trailer".into(), "X-Test-Me".into()),
        ("content-type".into(), "text/plain".into()),
    ];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }));
}

/// xhr parse-headers.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_parse_headers(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/parse-headers.py（上游逐字等价——
    // ?my-custom-header= 回显为响应头；头值 HTTP 语义 OWS trim（wptserve
    // 头解析层行为，等价 JS val.trim() 断言）；值含 NUL 原样下发，由引擎
    // 响应头 NUL 门转网络错误（headers-normalize-response NUL 族同步
    // send() 断言 NetworkError）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let val = wpt_query_value(query, "my-custom-header").unwrap_or_default();
    let trimmed = val.trim_matches(|c| c == ' ' || c == '\t');
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("my-custom-header".into(), trimmed.to_string()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }));
}

/// xhr cors-safelisted-response-headers 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_safelisted_response_headers(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/access-control-basic-cors-safelisted-
    // response-headers.py（上游逐字等价——safelisted 七头 + 非 safelisted
    // x-test 过滤断言面 + ACAO *；content-length 为 wptserve 每响应自动头，
    // 体 33 字节）。
    let body = "PASS: Cross-domain access allowed.";
    let mut headers: Vec<(String, String)> = vec![
        ("access-control-allow-origin".into(), "*".into()),
        ("cache-control".into(), "no cache".into()),
        ("content-language".into(), "en".into()),
        ("content-length".into(), "33".into()),
        ("content-type".into(), "text/plain".into()),
        ("expires".into(), "Fri, 30 Oct 1998 14:19:41 GMT".into()),
        ("last-modified".into(), "Tue, 15 Nov 1994 12:45:26 GMT".into()),
        ("pragma".into(), "no-cache".into()),
        ("x-test".into(), "foobar".into()),
    ];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }));
}

/// xhr cors-safelisted-request-headers 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_safelisted_request_headers(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/access-control-basic-cors-safelisted-
    // request-headers.py（上游逐字等价——safelisted 请求头跨源 POST 直达
    //（非 POST 拒 400 = 简单请求不预检），回显四头值（缺 <None>）+ ACAO
    // 回显 + ACAC true + Cache-Control no-store）。
    let req_header = |name: &str| -> Option<String> {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    let body = format!(
        "Accept: {}\nAccept-Language: {}\nContent-Language: {}\nContent-Type: {}\n",
        req_header("accept").unwrap_or_else(|| "<None>".into()),
        req_header("accept-language").unwrap_or_else(|| "<None>".into()),
        req_header("content-language").unwrap_or_else(|| "<None>".into()),
        req_header("content-type").unwrap_or_else(|| "<None>".into()),
    );
    let origin = req_header("origin").unwrap_or_default();
    let mut headers: Vec<(String, String)> = vec![
        ("cache-control".into(), "no-store".into()),
        ("access-control-allow-credentials".into(), "true".into()),
        ("access-control-allow-origin".into(), origin),
        ("content-type".into(), "text/plain".into()),
    ];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr no-custom-header-on-preflight 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_no_custom_header_on_preflight(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/no-custom-header-on-preflight.py
    //（上游逐字等价——OPTIONS 不得携带自定义头 x-test：携 → 400 FAIL，
    // 否则 stash[token]=PASS；GET 携 x-test 回显 stash（take 语义，缺省
    // Uninitialized），否则 400；ACAO * + ACAH x-test + Max-Age 0 恒设）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let token = wpt_query_value(query, "token").unwrap_or_default();
    let has_x_test = req.headers.iter().any(|(n, _)| n.eq_ignore_ascii_case("x-test"));
    let mut headers: Vec<(String, String)> = vec![
        ("access-control-allow-origin".into(), "*".into()),
        ("access-control-allow-headers".into(), "x-test".into()),
        ("access-control-max-age".into(), "0".into()),
    ];
    let (status, body) = if req.method == "OPTIONS" {
        if has_x_test {
            (400, "FAIL: Invalid header in preflight request.".to_string())
        } else {
            if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
                stash.insert(token, "PASS".to_string());
            }
            (200, String::new())
        }
    } else if has_x_test {
        let state = if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
            stash.remove(&token).unwrap_or_else(|| "Uninitialized".to_string())
        } else {
            "Uninitialized".to_string()
        };
        (200, state)
    } else {
        (400, "FAIL: X-Test header missing in request".to_string())
    };
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr preflight-request-header-lowercase 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_preflight_header_lowercase(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/access-control-preflight-request-
    // header-lowercase.py（上游逐字等价——OPTIONS 臂 ACRH 逗号表含 x-test
    // → ACAH X-Test（大写名授权，请求侧匹配须大小写不敏感），否则 400；
    // GET 臂携 x-test → PASS，否则 400；ACAO * + Max-Age 0 + Cache-Control
    // no-store 恒设）。
    let mut headers: Vec<(String, String)> = vec![
        ("cache-control".into(), "no-store".into()),
        ("access-control-allow-origin".into(), "*".into()),
        ("access-control-max-age".into(), "0".into()),
    ];
    let acrh = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
        .map(|(_, v)| v.clone());
    let (status, body) = if req.method == "OPTIONS" {
        let listed = acrh
            .map(|v| v.split(',').map(|s| s.trim()).any(|s| s.eq_ignore_ascii_case("x-test")))
            .unwrap_or(false);
        if listed {
            headers.push(("access-control-allow-headers".into(), "X-Test".into()));
            (200, String::new())
        } else {
            (400, String::new())
        }
    } else {
        let has_x_test = req.headers.iter().any(|(n, _)| n.eq_ignore_ascii_case("x-test"));
        if has_x_test {
            (200, "PASS".to_string())
        } else {
            (400, String::new())
        }
    };
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr preflight-request-header-sorted 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_preflight_header_sorted(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/access-control-preflight-request-
    // header-sorted.py（上游逐字等价——OPTIONS 臂 ACRH 须精确等于五头排序
    // 串（无空格 `,` 连接），不等 → 400；GET 臂携 x-custom-s → PASS，否则
    // 400 FAIL；ACAO 回显 origin + Cache-Control no-store 恒设）。
    const SORTED_HEADERS: &str = "x-custom-s,x-custom-test,x-custom-u,x-custom-ua,x-custom-v";
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = vec![
        ("cache-control".into(), "no-store".into()),
        ("access-control-allow-origin".into(), origin),
    ];
    let acrh = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
        .map(|(_, v)| v.clone());
    let (status, body) = if req.method == "OPTIONS" {
        headers.push(("access-control-max-age".into(), "0".into()));
        headers.push(("access-control-allow-headers".into(), SORTED_HEADERS.into()));
        if acrh.as_deref() == Some(SORTED_HEADERS) {
            (200, String::new())
        } else {
            (400, String::new())
        }
    } else {
        let has_x_custom_s = req.headers.iter().any(|(n, _)| n.eq_ignore_ascii_case("x-custom-s"));
        if has_x_custom_s {
            (200, "PASS".to_string())
        } else {
            (400, "FAIL".to_string())
        }
    };
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr preflight-request-headers-origin 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_preflight_headers_origin(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/access-control-preflight-request-
    // headers-origin.py（上游逐字等价——OPTIONS 臂 ACRH 含 "origin" → 400
    //（spec 禁把 origin 列入 ACRH），否则 ACAH x-pass；GET 臂体 = x-pass
    // 头值回显；ACAO * + Cache-Control no-store 恒设）。
    let mut headers: Vec<(String, String)> = vec![
        ("cache-control".into(), "no-store".into()),
        ("access-control-allow-origin".into(), "*".into()),
    ];
    let acrh = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
        .map(|(_, v)| v.clone());
    let (status, body) = if req.method == "OPTIONS" {
        if acrh.map(|v| v.to_lowercase().contains("origin")).unwrap_or(false) {
            (
                400,
                "Error: 'origin' included in Access-Control-Request-Headers".to_string(),
            )
        } else {
            headers.push(("access-control-allow-headers".into(), "x-pass".into()));
            (200, String::new())
        }
    } else {
        let x_pass = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("x-pass"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        (200, x_pass)
    };
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr preflight-request-invalid-status 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_preflight_invalid_status(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2 fixture：xhr/resources/access-control-preflight-request-
    // invalid-status.py（上游逐字等价——OPTIONS 臂 ?code= 存在 → 该状态码
    //（301/400/501 面——非 2xx preflight 响应须网络错误），恒设 Max-Age 1 +
    // ACAH x-pass；GET 臂 200；ACAO 回显 origin + Cache-Control no-store
    // 恒设）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let code = wpt_query_value(query, "code").and_then(|c| c.parse::<u16>().ok());
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = vec![
        ("cache-control".into(), "no-store".into()),
        ("access-control-allow-origin".into(), origin),
    ];
    let status = if req.method == "OPTIONS" {
        headers.push(("access-control-max-age".into(), "1".into()));
        headers.push(("access-control-allow-headers".into(), "x-pass".into()));
        code.unwrap_or(200)
    } else {
        200
    };
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }));
}

/// xhr echo-content-cors 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_echo_content_cors(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S22 fixture：xhr/resources/echo-content-cors.py（上游行为
    // 等价——ACAO/ACAH/ACAM 回显（?origin= 参数优先于请求头——上游同型怪癖）
    // + X-Request-* 探针头 + ACAC true + 体回显请求体，cors-upload 面）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let q_origin = wpt_query_value(query, "origin");
    let req_header = |name: &str| -> Option<String> {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    let origin = q_origin.clone().or_else(|| req_header("origin")).unwrap_or_default();
    let acrh = q_origin
        .clone()
        .or_else(|| req_header("access-control-request-headers"));
    let acrm = q_origin.or_else(|| req_header("access-control-request-method"));
    let mut headers: Vec<(String, String)> = vec![
        ("x-request-method".into(), req.method.clone()),
        (
            "x-request-content-length".into(),
            req_header("content-length").unwrap_or_else(|| "NO".into()),
        ),
        (
            "x-request-content-type".into(),
            req_header("content-type").unwrap_or_else(|| "NO".into()),
        ),
        ("access-control-allow-credentials".into(), "true".into()),
        ("content-type".into(), "text/plain".into()),
    ];
    if !origin.is_empty() {
        headers.push(("access-control-allow-origin".into(), origin));
    }
    if let Some(h) = acrh {
        headers.push(("access-control-allow-headers".into(), h));
    }
    if let Some(m) = acrm {
        headers.push(("access-control-allow-methods".into(), format!("OPTIONS, {m}")));
    }
    let body = req
        .body_bytes
        .as_ref()
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .or_else(|| req.body.clone())
        .unwrap_or_default();
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr corsenabled 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_corsenabled(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S16 fixture：xhr/resources/corsenabled.py（上游逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
    // corsenabled.py）。CORS 全开（ACAO * + ACAC + methods/headers/expose）+
    // X-Request-* 探针头 + ?delay=；体 "Test"。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers: Vec<(String, String)> = vec![
        ("access-control-allow-origin".into(), "*".into()),
        ("access-control-allow-credentials".into(), "true".into()),
        ("access-control-allow-methods".into(), "GET, POST, PUT, FOO".into()),
        ("access-control-allow-headers".into(), "x-test, x-foo".into()),
        (
            "access-control-expose-headers".into(),
            "x-request-method, x-request-content-type, x-request-query, x-request-content-length, x-request-data"
                .into(),
        ),
    ];
    let delay_ms = wpt_query_value(query, "delay")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .min(10_000);
    if delay_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
    }
    // net-api M3：上游 `?safelist_content_type` → **追加第二条**
    // `Access-Control-Allow-Headers: content-type`（上游逐字等价补齐——
    // wpt corsenabled.py safelist_content_type 分支；同名多实例经 Headers
    // get 合并「x-test, x-foo, content-type」，preflight ACAH 名单即含
    // content-type。send-redirect-to-cors safelist 族显式 CT 断言面）。
    if wpt_query_value(query, "safelist_content_type").is_some() {
        headers.push(("access-control-allow-headers".into(), "content-type".into()));
    }
    let req_header = |name: &str| -> Option<String> {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    headers.push(("x-request-method".into(), req.method.clone()));
    let req_query = path_part
        .split_once('?')
        .map(|(_, q)| q.to_string())
        .unwrap_or_default();
    headers.push((
        "x-request-query".into(),
        if req_query.is_empty() { "NO".into() } else { req_query },
    ));
    headers.push((
        "x-request-content-length".into(),
        req_header("content-length").unwrap_or_else(|| "NO".into()),
    ));
    headers.push((
        "x-request-content-type".into(),
        req_header("content-type").unwrap_or_else(|| "NO".into()),
    ));
    headers.push((
        "x-request-data".into(),
        // net-api M3：上游 `request.body` 回显**任意**体（fetch_bridge 契约
        // 文本体在 `body`、二进制体在 `body_bytes` 互斥）——旧实现只读
        // body_bytes，文本体（XHR send(string)/fetch body string）回显空串
        //（send-redirect-to-cors 307/308 body 断言面）。
        req.body_bytes
            .as_ref()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .or_else(|| req.body.clone())
            .unwrap_or_default(),
    ));
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "Test".to_string(),
        body_bytes: Some(b"Test".to_vec()),
    }));
}

/// xhr redirect.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_redirect(
    _ctx: &PyEndpointCtx,
    clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S16 fixture：xhr/resources/redirect.py（上游逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
    // redirect.py）。?code=（默认 302）+ ?location=（缺省本路径?followed）；
    // ?followed= → 200 "MAGIC HAPPENED"（Content:Type 破损头照抄——上游语义）；
    // 否则 → code + Location + "TEST"。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let code: u16 = wpt_query_value(query, "code")
        .and_then(|v| v.parse().ok())
        .unwrap_or(302);
    let followed = wpt_query_value(query, "followed").is_some();
    if followed {
        let mut headers: Vec<(String, String)> = Vec::new();
        headers.push(("content:type".into(), "text/plain".into()));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "MAGIC HAPPENED".to_string(),
            body_bytes: Some(b"MAGIC HAPPENED".to_vec()),
        }));
    }
    let location = wpt_query_value(query, "location").unwrap_or_else(|| format!("{}?followed", clean));
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("location".into(), location));
    wpt_add_fetch_metadata(&mut headers, req, code);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: code,
        status_text: wpt_status_text(code).to_string(),
        headers,
        body: "TEST".to_string(),
        body_bytes: Some(b"TEST".to_vec()),
    }));
}

/// xhr well-formed.xml 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_well_formed_xml(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S16 fixture：xhr/resources/well-formed.xml（上游逐字——
    // responseurl-after-abort 的 HEADERS_RECEIVED/LOADING abort 时序载体）。
    let body = "<html xmlns=\"http://www.w3.org/1999/xhtml\">\n  <p id=\"n&#49;\">1</p>\n  <p xmlns=\"namespacesarejuststrings\" id=\"n2\">2</p>\n</html>\n".to_string();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("access-control-allow-origin".into(), "*".into()));
    headers.push(("content-type".into(), "application/xml".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr pass.txt 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_pass_txt(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S16 fixture：xhr/resources/pass.txt（abort-progress-events
    // 的 send+abort 竞态载体——内容不断言，请求须可进入在飞态）。
    // M4-S18：ACAO *（runner 页面 https 而 get-host-info HTTP_ORIGIN 为 http——
    // 跨 scheme hop 的 cors check 放行面，runner 环境适配）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("access-control-allow-origin".into(), "*".into()));
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "PASS\n".to_string(),
        body_bytes: Some(b"PASS\n".to_vec()),
    }));
}

/// xhr content.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_xhr_content(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S15 fixture：xhr/resources/content.py（上游逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
    // content.py）。回显请求体（?content= 覆写）+ Content-type text/plain
    //（?response_charset_label= 追加 ;charset=）+ X-Request-Method/Query/
    // Content-Length/Content-Type 请求探针头。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers: Vec<(String, String)> = Vec::new();
    let ctype = match wpt_query_value(query, "response_charset_label") {
        Some(label) => format!("text/plain;charset={}", label),
        None => "text/plain".to_string(),
    };
    headers.push(("content-type".into(), ctype));
    headers.push(("x-request-method".into(), req.method.clone()));
    let req_query = path_part
        .split_once('?')
        .map(|(_, q)| q.to_string())
        .unwrap_or_default();
    headers.push((
        "x-request-query".into(),
        if req_query.is_empty() { "NO".into() } else { req_query },
    ));
    let req_header = |name: &str| -> Option<String> {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    headers.push((
        "x-request-content-length".into(),
        req_header("content-length").unwrap_or_else(|| "NO".into()),
    ));
    headers.push((
        "x-request-content-type".into(),
        req_header("content-type").unwrap_or_else(|| "NO".into()),
    ));
    let body = wpt_query_value(query, "content").unwrap_or_else(|| {
        let bytes_body = req
            .body_bytes
            .as_ref()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_default();
        if bytes_body.is_empty() {
            req.body.clone().unwrap_or_default()
        } else {
            bytes_body
        }
    });
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// status.py（xhr/ 前缀 + fetch/api 精确——Custom 谓词） 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_status_py(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M2-S3c：fetch/api/resources/status.py 同语义接入——上游
    // fetch/api/resources/status.py 与 xhr/resources/status.py 逐字同源
    //（code/text/content/type + X-Request-Method）。此前仅 xhr/ 前缀
    // 命中，fetch/api 面落静态读库 404（response-null-body「expected
    // 204 but got 404」、text-utf8 fetched-data 族）。
    // net-api M3-S2 fixture：xhr/resources/status.py（上游逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/
    // status.py）。?code=（默认 200）&text=（默认 OMG）&content=&type=
    // → 自定义状态行 + Content-Type + body。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let code: u16 = wpt_query_value(query, "code")
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    let text = wpt_query_value(query, "text").unwrap_or_else(|| "OMG".into());
    // net-api M3-S2：content 保留原始字节（%XX → byte——shift-jis 等非 UTF-8
    // 测试载荷经 body_bytes 保真，不落 lossy UTF-8）。
    // net-api M3（六）：解码走 `wpt_query_value_bytes`（逐字节 Latin-1 映射）
    // 再 char→u8 还原——旧实现经 `wpt_query_value`（UTF-8 lossy）后二次
    // percent-decode，非 UTF-8 载荷（%83%65…、%FF、%C2%F0）已被 lossy 成
    // U+FFFD 字节，engine 侧 override/charset 解码面永不可能命中。
    let content_bytes: Vec<u8> = wpt_query_value_bytes(query, "content")
        .map(|v| v.chars().map(|c| c as u8).collect())
        .unwrap_or_default();
    let ctype = wpt_query_value(query, "type").unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), ctype));
    headers.push(("x-request-method".into(), req.method.clone()));
    wpt_add_fetch_metadata(&mut headers, req, code);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: code,
        status_text: text,
        headers,
        body: String::from_utf8_lossy(&content_bytes).into_owned(),
        body_bytes: Some(content_bytes),
    }));
}

/// status.py 谓词——xhr/ 前缀或 fetch/api/resources/status.py 全等。
pub(crate) fn pred_status_py(clean: &str, _req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
    clean.ends_with("/status.py") && (clean.starts_with("xhr/") || clean == "fetch/api/resources/status.py")
}

/// xhr/resources/get-set-cookie.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_get_set_cookie(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let get_set_cookie_stash = &ctx.get_set_cookie_stash;
    // net-api-compat-v3 M3 fixture：get-set-cookie.py cookie 端点仿真（上游
    // wptserve handler——pin 3159769 xhr/resources/get-set-cookie.py 逐语义）：
    // ① 响应恒 Set-Cookie 两枚——`WK-test=1`（会话 cookie）与
    //    `WK-test-secure=1; Secure`（安全 cookie）；`?clear=1` 时均带过期
    //    Expires（上游为昨天——epoch 同为过去时刻，删除语义等价）。
    // ② 响应体 = 请求实际送达的 cookie 回显（逐枚前置空格 ` name=value`，
    //    无则空体——上游 `content + b" " + cookie + b"=" + value`）。
    // ③ Secure cookie 不随 http 请求送达（RFC 6265bis §5.4——runner 页锚
    //    http://wpt.test，WK-test-secure 恒不在回显体——测试核心断言）。
    // 方法面：测试用 XHR POST（GET 语义等价），分支先于非 GET 兜底门。
    // https://github.com/web-platform-tests/wpt/blob/3159769/xhr/resources/get-set-cookie.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let is_clear = wpt_query_value(query, "clear").as_deref() == Some("1");
    let is_secure_req = req.url.starts_with("https:");
    let mut jar = match get_set_cookie_stash.lock() {
        Ok(jar) => jar,
        Err(poisoned) => poisoned.into_inner(),
    };
    // 回显体先于本响应 Set-Cookie 生效（上游 response.content = 请求所携 cookie；
    // secure 且非 https 请求者不送达）。
    let mut body = String::new();
    for (name, (value, secure)) in jar.iter() {
        if *secure && !is_secure_req {
            continue;
        }
        body.push(' ');
        body.push_str(name);
        body.push('=');
        body.push_str(value);
    }
    if is_clear {
        jar.remove("WK-test");
        jar.remove("WK-test-secure");
    } else {
        jar.insert("WK-test".to_string(), ("1".to_string(), false));
        jar.insert("WK-test-secure".to_string(), ("1".to_string(), true));
    }
    drop(jar);
    let clear_expires = "Thu, 01 Jan 1970 00:00:00 GMT";
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    headers.push((
        "set-cookie".into(),
        if is_clear {
            format!("WK-test=1; Expires={clear_expires}")
        } else {
            "WK-test=1".to_string()
        },
    ));
    headers.push((
        "set-cookie".into(),
        if is_clear {
            format!("WK-test-secure=1; Secure; Expires={clear_expires}")
        } else {
            "WK-test-secure=1; Secure".to_string()
        },
    ));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}

/// xhr/resources/*.asis 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_xhr_asis(
    _ctx: &PyEndpointCtx,
    clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S15 fixture：xhr/resources/*.asis 原始 HTTP 响应文件（上游
    // 逐字嵌入——https://github.com/web-platform-tests/wpt/blob/3159769/xhr/
    // resources/*.asis；wptserve 原样发送文件字节，行尾 LF 解析宽松）。
    // 解析：首行状态行，后续非空行 `name: value` 头（空值合法），其余为体。
    let raw = match clean.rsplit('/').next().unwrap_or("") {
        "header-content-length.asis" => "HTTP/1.0 200 NANANA\nCONTENT-LENGTH:  0\n",
        "header-content-length-twice.asis" => "HTTP/1.0 200 NANANA\nCONTENT-LENGTH:  0\ncontent-length:\t 0\n",
        "headers-double-empty.asis" => "HTTP/1.1 444 HI\ndouble-trouble:\ndouble-trouble:\n",
        "headers-basic.asis" => "HTTP/1.1 280 HELLO\nfoo-test: 1\nfoo-test: 2\nfoo-test: 3\n",
        "headers.asis" => {
            "HTTP/1.1 200 YAYAYAYA\nfoo-TEST: 1\nFOO-test: 2\n__Custom: token\nALSO-here: Mr. PB\newok: lego\n"
        }
        "headers-some-are-empty.asis" => {
            "HTTP/1.0 200 MEH\nHEYA:\t \nHEYA: \u{000B}\u{000C}\nHEYA: 1\t\nHEYA:\nHEYA:\t\nHEYA: \t2\n"
        }
        "headers-www-authenticate.asis" => {
            "HTTP/1.1 280 HELLO\nwww-authenticate: 1\nwww-authenticate: 2\nwww-authenticate: 3, 4\n"
        }
        _ => {
            return Some(Err(format!("fixture not embedded: {}", clean)));
        }
    };
    let mut lines = raw.lines();
    let status_line = lines.next().unwrap_or("HTTP/1.0 200 OK");
    let mut parts = status_line.splitn(3, ' ');
    let _http = parts.next().unwrap_or("HTTP/1.0");
    let status: u16 = parts.next().unwrap_or("200").parse().unwrap_or(200);
    let status_text = parts.next().unwrap_or("").to_string();
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut in_body = false;
    for line in lines {
        if in_body {
            body_lines.push(line);
            continue;
        }
        if line.is_empty() {
            in_body = true;
            continue;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((
                name.to_lowercase(),
                value.strip_prefix(' ').unwrap_or(value).to_string(),
            ));
        }
    }
    let body = if body_lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", body_lines.join("\n"))
    };
    wpt_add_fetch_metadata(&mut headers, req, status);
    return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text,
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }));
}
