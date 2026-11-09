//! fetch/api 端点族——redirect.py（S12 preflight-redirect 收编面）、method/authentication、
//! echo-headers/delay/dump-authorization、clean-stash/preflight、方法豁免兜底门与 OPTIONS blanket、inspect-headers、trickle/bad-chunk。

use super::*;

/// fetch/api/resources/ echo-content(.h2) 族 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_fetch_echo_content(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api 二期 M2-S3 fixture：fetch/api/basic/echo-content.py 与
    // echo-content.h2.py（上游行为等价——原样回显请求体，request-upload
    // 面断言 resp.text() == 上传体）。`.h2.py` 端点上游走 h2 流式上传；
    // runner 侧流→字节泵尽后经常规 wire 到达，此处不区分。
    let body = req
        .body_bytes
        .clone()
        .or_else(|| req.body.clone().map(|s| s.into_bytes()))
        .unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let lossy = String::from_utf8_lossy(&body).into_owned();
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: lossy,
        body_bytes: Some(body),
    }))
}

/// fetch/api/resources/ redirect.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_fetch_redirect(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S17/S23 fixture：fetch/api/resources/redirect.py（上游行为
    // 等价）：ACAO 回显 Origin（无 Origin → *）+ ACAC（Origin 在时）+
    // ?redirect_status=（默认 302）+ Location（**附全量原 query + count**——
    // 上游 urlencode(url_parameters) 语义，M4-S23 对齐）。OPTIONS 分支：
    // 无 ?redirect_preflight → 200 不重定向（preflight 须不跟随——
    // cors-preflight-redirect 面）；有 → **落穿重定向逻辑返回 redirect_status**
    //（上游同构——preflight 命中重定向 → fetch 判失败）。
    // net-api-compat-v3 M1b：升上游逐字语义（pin 315976933870 fetch/api/
    // resources/redirect.py）——?token= per-token stash {count,preflight}
    //（take→put，重置走 clean-stash.py）、?max_count= 超限回 200 + 文本
    // str(count-1)（redirect-count 断言体=跟随跳数）、?simple= 不附 query/
    // count（redirect-location-escape 断言 resp.url 精确后缀）、
    // ?redirect_referrerpolicy= → Referrer-Policy 透传（redirect-referrer-
    // override 面）。无 token 时 count 不持久（每请求回 1——上游 stash
    // take 未命中即默认 0 语义；process-global 原子计数废除）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let status: u16 = wpt_query_value(query, "redirect_status")
        .and_then(|v| v.parse().ok())
        .unwrap_or(302);
    let token = wpt_query_value(query, "token");
    // stash 值 `count\x1fpreflight`；键 `redirect\x1f{token}` 命名空间——
    // runner stash 是 flat HashMap（上游 wptserve stash 为 per-token 多键
    // dict，端点间不冲突），若直写 {token} 键，preflight.py fixture 会把
    // 首字段 count 误读为 did-preflight（cors-redirect 20 案 P→F 回归根
    // 因，2026-11-06）；clean-stash.py 同步清两键（上游语义：clean 按
    // token 抹全部端点状态）。
    let stash_key = |tok: &str| format!("redirect\x1f{}", tok);
    let stash_put = |stash: &mut std::collections::HashMap<String, String>, key: &str, count: u64, preflight: &str| {
        stash.insert(key.to_string(), format!("{}\x1f{}", count, preflight));
    };
    let mut count: u64 = 0;
    let mut preflight_flag = false;
    if let Some(tok) = &token
        && let Ok(mut stash) = WPT_TOKEN_STASH.lock()
        && let Some(v) = stash.remove(&stash_key(tok))
    {
        let mut parts = v.split('\x1f');
        count = parts.next().and_then(|c| c.parse().ok()).unwrap_or(0);
        preflight_flag = parts.next() == Some("1");
    }
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone());
    let mut headers: Vec<(String, String)> = vec![
        ("content-type".into(), "text/plain".into()),
        ("cache-control".into(), "no-cache".into()),
        ("pragma".into(), "no-cache".into()),
        (
            "access-control-allow-origin".into(),
            origin.clone().unwrap_or_else(|| "*".into()),
        ),
    ];
    if origin.is_some() {
        headers.push(("access-control-allow-credentials".into(), "true".into()));
    }
    if req.method == "OPTIONS" {
        preflight_flag = true;
        if let Some(allow_headers) = wpt_query_value(query, "allow_headers") {
            headers.push(("access-control-allow-headers".into(), allow_headers));
        }
        if wpt_query_value(query, "redirect_preflight").is_none() {
            if let (Some(tok), Ok(mut stash)) = (&token, WPT_TOKEN_STASH.lock()) {
                stash_put(&mut stash, &stash_key(tok), count, "1");
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
        // redirect_preflight → 落穿：preflight 响应带重定向（fetch 侧拒绝）。
    }
    count += 1;
    // net-api-compat-v3 M3：location 用字节语义变体解码（wptserve/
    // 浏览器链路上响应头值为字节序列——redirect-location-escape 面
    // %E2%98%83 → 三字节 Latin-1 字符、%FF → U+00FF 保真）。
    if let Some(location) = wpt_query_value_bytes(query, "location") {
        // 上游：scheme ∈ {空,http,https}（urlparse 小写语义——scheme 仅
        // 认首段 `^[A-Za-z][A-Za-z0-9+.-]*:` 前缀，无前缀即空）且非
        // ?simple= 时附全量原 query（urlencode——**含 location 键自身**）
        // + count=N（自变 URL 防循环去重误判）。
        let scheme_head = location.split(['/', '?', '#']).next().unwrap_or("");
        let scheme = match scheme_head.split_once(':') {
            Some((s, _))
                if !s.is_empty()
                    && s.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                    && s.chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')) =>
            {
                s.to_ascii_lowercase()
            }
            _ => String::new(),
        };
        let scheme_httpish = scheme.is_empty() || scheme == "http" || scheme == "https";
        let mut target = location.clone();
        if scheme_httpish && wpt_query_value(query, "simple").is_none() {
            let sep = if target.contains('?') { '&' } else { '?' };
            // 原样携带 raw pair 字节（wpt_query_value 已 percent-decode，
            // 回填用 raw 段避免二次编解码漂移；上游 urlencode 大写 %XX 与
            // 语料 encodeURIComponent/escape 产物一致）。
            target.push(sep);
            target.push_str(query);
            target.push_str(&format!("&count={}", count));
        }
        headers.push(("location".into(), target));
    }
    if let Some(rp) = wpt_query_value(query, "redirect_referrerpolicy") {
        headers.push(("referrer-policy".into(), rp));
    }
    // FIXME(上游 delay 参数)：语料 fetch/api/redirect 无 delay 消费者，
    // fixture 不睡眠（wptserve time.sleep 等价待有面再接）。
    if let Some(tok) = &token
        && let Ok(mut stash) = WPT_TOKEN_STASH.lock()
    {
        stash_put(
            &mut stash,
            &stash_key(tok),
            count,
            if preflight_flag { "1" } else { "0" },
        );
        if let Some(max_count) = wpt_query_value(query, "max_count").and_then(|v| v.parse::<u64>().ok())
            && count > max_count
        {
            // 上游：超限停止重定向，200 + str(count-1)（末跳非重定向）。
            let body = (count - 1).to_string();
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
    }
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// fetch/api/resources/redirect-empty-location.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_fetch_redirect_empty_location(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api-compat-v3 M1b fixture：fetch/api/resources/redirect-empty-
    // location.py（上游逐字——https://github.com/web-platform-tests/wpt/
    // blob/3159769/fetch/api/resources/redirect-empty-location.py）：
    // 302 + 空 Location 头（fetch follow → TypeError、manual →
    // opaqueredirect）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("location".into(), String::new()));
    wpt_add_fetch_metadata(&mut headers, req, 302);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 302,
        status_text: wpt_status_text(302).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// fetch/api/resources/method.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_fetch_method(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api-compat-v3 M1b fixture：fetch/api/resources/method.py（上游
    // 逐字——https://github.com/web-platform-tests/wpt/blob/3159769/fetch/
    // api/resources/method.py）：x-request-method/content-type/content-
    // length/content-encoding/content-language/content-location 回显请求；
    // 体=请求体（redirect-method 断言重定向后方法/头/体保真）；?cors →
    // ACAO */ACAC true/ACAM/ACAH + expose x-request-method。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let req_header = |name: &str| -> String {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "NO".into())
    };
    let mut headers: Vec<(String, String)> = Vec::new();
    if wpt_query_value(query, "cors").is_some() {
        headers.push(("access-control-allow-origin".into(), "*".into()));
        headers.push(("access-control-allow-credentials".into(), "true".into()));
        headers.push(("access-control-allow-methods".into(), "GET, POST, PUT, FOO".into()));
        headers.push(("access-control-allow-headers".into(), "x-test, x-foo".into()));
        headers.push(("access-control-expose-headers".into(), "x-request-method".into()));
    }
    headers.push(("x-request-method".into(), req.method.clone()));
    headers.push(("x-request-content-type".into(), req_header("content-type")));
    headers.push(("x-request-content-length".into(), req_header("content-length")));
    headers.push(("x-request-content-encoding".into(), req_header("content-encoding")));
    headers.push(("x-request-content-language".into(), req_header("content-language")));
    headers.push(("x-request-content-location".into(), req_header("content-location")));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let body_bytes = req
        .body_bytes
        .clone()
        .or_else(|| req.body.as_ref().map(|b| b.as_bytes().to_vec()))
        .unwrap_or_default();
    let body_text = String::from_utf8_lossy(&body_bytes).to_string();
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body_text,
        body_bytes: Some(body_bytes),
    }))
}

/// echo-headers.py（Suffix 谓词引入） 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_echo_headers(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S21 fixture：request-content-length 载体——回显全部请求头
    // 为 `Name: value` 行（request-content-length.any.js 断言
    // responseText 含 `Content-Length: <len>`）。
    // net-api M2-S10：头名逐字回显（不再 title-case——request-headers-case
    // 「THIS-is-A-test: 1, 2」原始大小写断言面；引擎内部头已按 HTTP 惯例
    // Title-Case 发出，`Content-Length: <len>` 大小写敏感 includes 面由
    // 引擎侧保住）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let mut body = String::new();
    for (n, v) in &req.headers {
        body.push_str(&format!("{}: {}\n", n, v));
    }
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// delay.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_delay(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S20 fixture：xhr/resources/delay.py（上游行为等价——?ms= 延迟
    // 后回 text/plain "TEST"；延迟由 shim 侧 setTimeout 实现——runner 同步契约
    // 下 host sleep 冻结 JS，此处立即应答）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    // net-api M5-S10：OPTIONS preflight 应答（ACAO 回显 + ACAM/ACAH ACRM/ACRH
    // 并入——上游 wptserve 自动应答 preflight 的合规 CORS 服务端语义；
    // timeout-multiple-fetches「CORS preflights should not reset the timer」
    // YO 方法面——无此分支时 OPTIONS 落入本 200 应答无 CORS 头 → preflight
    // 拒 → onerror 而非 ontimeout。M5-S3 eventsource cors.py 同先例）。
    if req.method == "OPTIONS" {
        headers.push((
            "access-control-allow-origin".into(),
            req.headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| "*".into()),
        ));
        let mut allow_methods: Vec<String> = ["GET", "POST", "HEAD", "OPTIONS"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        if let Some(acrm) = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
            .map(|(_, v)| v.trim().to_ascii_uppercase())
            && !acrm.is_empty()
            && !allow_methods.contains(&acrm)
        {
            allow_methods.push(acrm);
        }
        headers.push(("access-control-allow-methods".into(), allow_methods.join(", ")));
        if let Some(acrh) = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
            .map(|(_, v)| v.trim().to_string())
            .filter(|v| !v.is_empty())
        {
            headers.push(("access-control-allow-headers".into(), acrh));
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
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "TEST".to_string(),
        body_bytes: Some(b"TEST".to_vec()),
    }))
}

/// fetch/api/resources/authentication.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_fetch_authentication(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api-compat-v3 M2 fixture：fetch/api/resources/authentication.py
    //（上游逐字——https://github.com/web-platform-tests/wpt/blob/
    // 3159769/fetch/api/resources/authentication.py。与 xhr 版 token
    // stash 挑战流不同源：固定凭据校验——Basic user:password → 200
    // "Authentication done"；否则 401 Unauthorized +
    // `WWW-Authenticate: Basic realm="?realm="（缺省 test）` + 提示体。
    // 无 CORS 头（上游同形——authentication-basic 相对 URL 同源取回，
    // resp.type 断言 "basic"）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let credentials = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("authorization"))
        .and_then(|(_, v)| {
            let b64 = v.strip_prefix("Basic ").or_else(|| v.strip_prefix("basic "))?;
            use base64::Engine as _;
            let decoded = base64::engine::general_purpose::STANDARD.decode(b64.trim()).ok()?;
            String::from_utf8_lossy(&decoded)
                .split_once(':')
                .map(|(u, p)| (u.to_string(), p.to_string()))
        });
    let realm = wpt_query_value(query, "realm").unwrap_or_else(|| "test".to_string());
    let (status, body) = match credentials {
        Some((u, p)) if u == "user" && p == "password" => (200, "Authentication done".to_string()),
        _ => (401, "Please login with credentials 'user' and 'password'".to_string()),
    };
    let mut headers: Vec<(String, String)> = vec![("content-type".into(), "text/plain".into())];
    if status == 401 {
        headers.push(("www-authenticate".into(), format!("Basic realm=\"{}\"", realm)));
    }
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// dump-authorization-header.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_dump_authorization_header(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S17 fixture：fetch/api/resources/dump-authorization-header.py
    //（上游逐字等价——回显 Authorization 或 "none"；带 Origin → ACAO 回显 +
    // ACAC true + ACAH Authorization）。net-api-compat-v3 M2 补上游
    // OPTIONS strip_auth_header=1 臂：预检 ACRH 含 authorization → 500
    // "fail"（跨源重定向后预检不得携 auth 头——xhr-authorization-
    // redirect 跨源腿断言面）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone());
    let auth = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("authorization"))
        .map(|(_, v)| v.clone());
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/html".into()));
    headers.push(("cache-control".into(), "no-cache".into()));
    if req.method == "OPTIONS"
        && wpt_query_value(query, "strip_auth_header").is_some()
        && req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
            .map(|(_, v)| v.to_ascii_lowercase().contains("authorization"))
            .unwrap_or(false)
    {
        let body = "fail".to_string();
        wpt_add_fetch_metadata(&mut headers, req, 500);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 500,
            status_text: wpt_status_text(500).to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        }));
    }
    match origin {
        Some(o) => {
            headers.push(("access-control-allow-origin".into(), o));
            headers.push(("access-control-allow-credentials".into(), "true".into()));
        }
        None => {
            headers.push(("access-control-allow-origin".into(), "*".into()));
        }
    }
    headers.push(("access-control-allow-headers".into(), "Authorization".into()));
    let body = auth.unwrap_or_else(|| "none".into());
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// fetch/api/resources/clean-stash.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_clean_stash(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S23 fixture：fetch/api/resources/clean-stash.py（上游逐字
    // 等价——stash take 命中 → "1"，否则 "0"；恒 200）。上游按 token 抹
    // 全部端点状态，runner flat stash 下同抹 redirect.py 命名空间键
    // `redirect\x1f{token}`（net-api-compat-v3 M1b）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let token = wpt_query_value(query, "token").unwrap_or_default();
    let had = WPT_TOKEN_STASH
        .lock()
        .ok()
        .map(|mut s| {
            let a = s.remove(&token).is_some();
            let b = s.remove(&format!("redirect\x1f{}", token)).is_some();
            a || b
        })
        .unwrap_or(false);
    let mut headers: Vec<(String, String)> = vec![("content-type".into(), "text/plain".into())];
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let body = if had { "1" } else { "0" };
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }))
}

/// fetch/api/resources/preflight.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_preflight(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S23 fixture：fetch/api/resources/preflight.py（上游行为
    // 等价——?origin= 多 ACAO、?clear-stash=、?credentials=、OPTIONS 分支
    // （ACRM/Accept 校验 + allow_methods/allow_headers/max_age/preflight_
    // status + stash 记录 did-preflight/ACRH/referrer/UA）、实际请求分支
    // x-did-preflight 等暴露头回读；stash 取后回存上游 take-then-put 语义）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let qval = |name: &str| wpt_query_value(query, name);
    let req_header = |name: &str| -> Option<String> {
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
    };
    let token = qval("token").unwrap_or_default();
    let mut headers: Vec<(String, String)> = vec![("content-type".into(), "text/plain".into())];
    if let Some(origin_list) = qval("origin") {
        for o in origin_list.split(", ") {
            headers.push(("access-control-allow-origin".into(), o.to_string()));
        }
    } else {
        headers.push(("access-control-allow-origin".into(), "*".into()));
    }
    if qval("clear-stash").is_some() {
        let had = WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(&token)).is_some();
        wpt_add_fetch_metadata(&mut headers, req, 200);
        let body = if had { "1" } else { "0" };
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.to_string(),
            body_bytes: Some(body.as_bytes().to_vec()),
        }));
    }
    if qval("credentials").is_some() {
        headers.push(("access-control-allow-credentials".into(), "true".into()));
    }
    // stash dict 四字段定序编码：preflight\x1fcontrol_request_headers\x1f
    // preflight_referrer\x1fpreflight_user_agent（None → "\u{0}"哨兵）。
    const NONE_SENTINEL: &str = "\u{0}";
    let encode = |c: &str, r: &str, u: &str| format!("1\x1f{}\x1f{}\x1f{}", c, r, u);
    if req.method == "OPTIONS" {
        if req_header("access-control-request-method").is_none() {
            wpt_add_fetch_metadata(&mut headers, req, 400);
            return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 400,
                status_text: wpt_status_text(400).to_string(),
                headers,
                body: "ERROR: No access-control-request-method in preflight!".to_string(),
                body_bytes: Some(b"ERROR: No access-control-request-method in preflight!".to_vec()),
            }));
        }
        if req_header("accept").map(|v| v.trim() != "*/*").unwrap_or(true) {
            wpt_add_fetch_metadata(&mut headers, req, 400);
            return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 400,
                status_text: wpt_status_text(400).to_string(),
                headers,
                body: "ERROR: Invalid access in preflight!".to_string(),
                body_bytes: Some(b"ERROR: Invalid access in preflight!".to_vec()),
            }));
        }
        // control_request_headers 旗标在 → 存 ACRH（头缺席 → None 哨兵，
        // 不发射）；旗标不在 → 缺省 b""（恒发射空值——上游默认字典同形）。
        let control_headers = if qval("control_request_headers").is_some() {
            req_header("access-control-request-headers").unwrap_or_else(|| NONE_SENTINEL.into())
        } else {
            String::new()
        };
        if let Some(ma) = qval("max_age") {
            headers.push(("access-control-max-age".into(), ma));
        }
        if let Some(ah) = qval("allow_headers") {
            headers.push(("access-control-allow-headers".into(), ah));
        }
        if let Some(am) = qval("allow_methods") {
            headers.push(("access-control-allow-methods".into(), am));
        }
        let status: u16 = qval("preflight_status").and_then(|v| v.parse().ok()).unwrap_or(200);
        let dict = encode(
            &control_headers,
            &req_header("referer").unwrap_or_default(),
            &req_header("user-agent").unwrap_or_default(),
        );
        if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
            stash.insert(token, dict);
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
    // 实际请求分支：stash 取后回存（上游 take-then-put）。
    let stored = WPT_TOKEN_STASH.lock().ok().and_then(|mut s| s.remove(&token));
    let (preflight_flag, control_headers, preflight_referrer, preflight_ua) = match stored {
        Some(dict) => {
            let parts: Vec<&str> = dict.split('\x1f').collect();
            (
                parts.first().copied().unwrap_or("0").to_string(),
                parts.get(1).copied().unwrap_or(NONE_SENTINEL).to_string(),
                parts.get(2).copied().unwrap_or_default().to_string(),
                parts.get(3).copied().unwrap_or_default().to_string(),
            )
        }
        // net-api M4-S25：control_request_headers 缺省 b""（非 None）——
        // x-control-request-headers 恒发射（上游默认字典同形）。
        None => ("0".to_string(), String::new(), String::new(), String::new()),
    };
    if qval("checkUserAgentHeaderInPreflight").is_some() {
        let ua = req_header("user-agent").unwrap_or_default();
        if ua != preflight_ua {
            wpt_add_fetch_metadata(&mut headers, req, 400);
            return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 400,
                status_text: wpt_status_text(400).to_string(),
                headers,
                body: "ERROR: No user-agent header in preflight".to_string(),
                body_bytes: Some(b"ERROR: No user-agent header in preflight".to_vec()),
            }));
        }
    }
    headers.push((
        "access-control-expose-headers".into(),
        "x-did-preflight, x-control-request-headers, x-referrer, x-preflight-referrer, x-origin".into(),
    ));
    headers.push(("x-did-preflight".into(), preflight_flag.clone()));
    if control_headers != NONE_SENTINEL {
        headers.push(("x-control-request-headers".into(), control_headers.clone()));
    }
    headers.push(("x-preflight-referrer".into(), preflight_referrer.clone()));
    headers.push(("x-referrer".into(), req_header("referer").unwrap_or_default()));
    headers.push(("x-origin".into(), req_header("origin").unwrap_or_default()));
    if let Ok(mut stash) = WPT_TOKEN_STASH.lock() {
        stash.insert(
            token,
            format!(
                "{}\x1f{}\x1f{}\x1f{}",
                preflight_flag.clone(),
                control_headers.clone(),
                preflight_referrer.clone(),
                preflight_ua.clone()
            ),
        );
    }
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// 非 GET/OPTIONS 兜底门谓词——inspect-headers.py 豁免（方法无关回显端点经
/// 下一臂承接），其余端点非 GET/OPTIONS 请求兜底拒绝（原 405 门原条件）。
pub(crate) fn pred_method_gate_non_get(clean: &str, req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
    req.method != "GET" && req.method != "OPTIONS" && !clean.ends_with("/inspect-headers.py")
}

/// OPTIONS 兜底 preflight 谓词——无 OPTIONS 分支端点的统一 200 空 preflight
/// 应答（原 blanket OPTIONS 臂原条件）。
pub(crate) fn pred_method_options(_clean: &str, req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
    req.method == "OPTIONS"
}

/// 非 GET/OPTIONS 兜底门 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_method_not_supported(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S18：非 GET 放行至各 fixture 自行分派（corsenabled/redirect-cors
    // 的 OPTIONS preflight 分支）后兜底拒绝——preflight OPTIONS 不再被 blanket 拒。
    // net-api M2-S5：inspect-headers.py 上游 wptserve handler 方法无关（任意
    // 方法回显请求头）——header-values-normalize XHR POST 腿经此回显；其余
    // 端点维持兜底拒绝。
    Some(Err(format!("method not supported: {}", req.method)))
}

/// OPTIONS 兜底 preflight 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_options_blanket(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // 无 OPTIONS 分支的端点：统一 200 + 空（preflight 判定交给 shim 侧
    // ACAM/ACAH 检查）。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push((
        "access-control-allow-origin".into(),
        req.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "*".into()),
    ));
    // net-api M5-S10：ACAM/ACAH 回显（ACRM/ACRH 请求头并入——上游 wptserve
    // 自动应答 preflight 的合规 CORS 服务端语义；timeout-multiple-fetches
    // 「CORS preflights should not reset the timer」YO 方法面——固定集不覆盖
    // 自定义方法 → preflight 拒 → onerror 而非 ontimeout。M5-S3 eventsource
    // cors.py 同先例）。
    let mut allow_methods: Vec<String> = ["GET", "POST", "HEAD", "OPTIONS"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(acrm) = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
        .map(|(_, v)| v.trim().to_ascii_uppercase())
        && !acrm.is_empty()
        && !allow_methods.contains(&acrm)
    {
        allow_methods.push(acrm);
    }
    headers.push(("access-control-allow-methods".into(), allow_methods.join(", ")));
    if let Some(acrh) = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty())
    {
        headers.push(("access-control-allow-headers".into(), acrh));
    }
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// inspect-headers.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_inspect_headers(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M3-S1 fixture：fetch/api/resources/inspect-headers.py（上游逐字
    // 等价——https://github.com/web-platform-tests/wpt/blob/3159769/fetch/api/
    // resources/inspect-headers.py）。?headers=a|b|c → 每个请求头存在则回
    // `x-request-<name>: <value>`（多值 ', ' 合并）；?cors → AC 族 + expose；
    // content-type: text/plain。header-values(-normalize) 回读面。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers: Vec<(String, String)> = Vec::new();
    let checked: Vec<String> = wpt_query_value(query, "headers")
        .map(|names| {
            names
                .split('|')
                .map(|n| n.trim().to_lowercase())
                .filter(|n| !n.is_empty())
                .collect()
        })
        .unwrap_or_default();
    for name in &checked {
        // 上游 inspect-headers.py 按「头存在」回显（wptserve `in
        // request.headers`），值可为空串——header-values-normalize strip
        // 腿（值 strip 后为 ""）期望 x-request-val1 == "" 而非缺席。
        let hits: Vec<&str> = req
            .headers
            .iter()
            .filter(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
            .collect();
        if hits.is_empty() {
            continue;
        }
        headers.push((format!("x-request-{}", name), hits.join(", ")));
    }
    if wpt_query_value(query, "cors").is_some() {
        let origin = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "*".to_string());
        headers.push(("access-control-allow-origin".into(), origin));
        headers.push(("access-control-allow-credentials".into(), "true".into()));
        headers.push(("access-control-allow-methods".into(), "GET, POST, HEAD".into()));
        if !checked.is_empty() {
            let exposed = checked
                .iter()
                .map(|n| format!("x-request-{}", n))
                .collect::<Vec<_>>()
                .join(", ");
            headers.push(("access-control-expose-headers".into(), exposed));
        }
    }
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

/// fetch/api/resources/trickle.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_fetch_trickle(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
    let count = wpt_query_value(query, "count")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let count = count.min(128);
    let body = "TEST_TRICKLE\n".repeat(count);
    let mut headers = wpt_pipe_headers(query);
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// bad-chunk-encoding.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_bad_chunk_encoding(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M5-S9：fetch/api/resources/bad-chunk-encoding.py（上游 wptserve
    // 流式端点——count×合法 chunk 后写裸 "garbage"，客户端 chunk 解析失败 →
    // 中段网络错误）。合成通道无 chunk 帧语义：body = count×TEST_CHUNK +
    // "garbage"，`x-zero-body-error-after` 内部标记头承载「N 字节后流错误」
    // 语义（shim 侧消费后从可见 header list 剥离；error-after-response
    // 两腿 = 首 read 得 TEST_CHUNK、次 read/closed 拒 TypeError）。
    // net-api M5-S11：后缀匹配扩 xhr/resources/（event-error.sub 相对 URL
    // 「resources/bad-chunk-encoding.py」→ /xhr/resources/ 面——上游两目录
    // 各有一份同型端点，语义同构）。
    let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
    let count = wpt_query_value(query, "count")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let count = count.min(128);
    let mut body_bytes = "TEST_CHUNK".repeat(count).into_bytes();
    let good_len = body_bytes.len();
    body_bytes.extend_from_slice(b"garbage");
    let mut headers = Vec::new();
    headers.push(("x-zero-body-error-after".into(), good_len.to_string()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let body = String::from_utf8_lossy(&body_bytes).into_owned();
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body,
        body_bytes: Some(body_bytes),
    }))
}
