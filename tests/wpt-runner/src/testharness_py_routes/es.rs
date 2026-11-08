//! EventSource 端点族——message/last-event-id/reconnect 状态机与 cors-cookie/cors（含 OPTIONS preflight 面）。

use super::*;

/// eventsource message.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_message(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S12 fixture：eventsource/resources/message.py（上游逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/
    // message.py）。?mime=（默认 text/event-stream）&message=（默认 data: data）
    // &newline=none（尾 \n\n 省略）&sleep=（ms）→ SSE 单响应。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mime = wpt_query_value(query, "mime").unwrap_or_else(|| "text/event-stream".into());
    let message = wpt_query_value(query, "message").unwrap_or_else(|| "data: data".into());
    let newline = if wpt_query_value(query, "newline").as_deref() == Some("none") {
        ""
    } else {
        "\n\n"
    };
    let sleep_ms = wpt_query_value(query, "sleep")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .min(10_000);
    if sleep_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
    }
    let body = format!("{}{}\n", message, newline);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), mime));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource message2.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_message2(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S12 fixture：eventsource/resources/message2.py（上游为无限循环
    // 流式写——headless 有限流模型取一轮循环体即覆盖页面断言的前三个事件；
    // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/
    // message2.py）。
    let body = "data:msg\ndata: msg\n\n:\nfalsefield:msg\n\nfalsefield:msg\n\n\
                Data:data\n\ndata\n\ndata:end\n\n"
        .to_string();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/event-stream".into()));
    headers.push(("cache-control".into(), "no-cache".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource last-event-id.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_last_event_id(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S12 fixture：eventsource/resources/last-event-id.py（上游逐字
    // 等价——https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/
    // resources/last-event-id.py）。带 Last-Event-ID 请求头则回 `data: <id>`，
    // 否则回 `id: <idvalue 默认 …>\nretry: 200\ndata: hello`（重连面）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let last_id = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("last-event-id"))
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    let body = if !last_id.is_empty() {
        format!("data: {}\n\n", last_id)
    } else {
        let idvalue = wpt_query_value(query, "idvalue").unwrap_or_else(|| "\u{2026}".into());
        format!("id: {}\nretry: 200\ndata: hello\n\n", idvalue)
    };
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/event-stream".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource .event_stream 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_event_stream(
    _ctx: &PyEndpointCtx,
    clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4-S12 fixture：eventsource/resources/accept.event_stream 与
    // cache-control.event_stream（上游为 wptserve 模板文件
    // `data: {{headers[<name>]}}`——按文件名回显对应请求头；
    // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/）。
    let header_name = clean
        .rsplit('/')
        .next()
        .and_then(|f| f.strip_suffix(".event_stream"))
        .unwrap_or("accept");
    let echoed = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(header_name))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let body = format!("data: {}\n\n", echoed);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/event-stream".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource reconnect-fail.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_reconnect_fail(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let es_cookie_stash = &ctx.es_cookie_stash;
    // net-api M4 fixture：eventsource/resources/reconnect-fail.py（上游逐字
    // 等价——https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/
    // resources/reconnect-fail.py）。cookie 态机：无态 → 200 `retry: 2`+
    // `data: opened`（置态 opened）；opened → 200 `data: reconnected`（置态
    // reconnected）；reconnected → 204 `data: closed`（清态——status ≠ 200，
    // ES fail the connection，体不达）。cookie 往返以 es_cookie_stash 承载。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let name = format!("recon_fail_{}", wpt_query_value(query, "id").unwrap_or_default());
    let mut jar = match es_cookie_stash.lock() {
        Ok(j) => j,
        Err(poisoned) => poisoned.into_inner(),
    };
    let (status, status_text, body) = match jar.remove(&name).as_deref() {
        Some("opened") => {
            jar.insert(name, "reconnected".to_string());
            (200u16, "RECONNECT", "data: reconnected\n\n".to_string())
        }
        Some("reconnected") => (204, "NO CONTENT (CLOSE)", "data: closed\n\n".to_string()),
        _ => {
            jar.insert(name, "opened".to_string());
            (200, "OPEN", "retry: 2\ndata: opened\n\n".to_string())
        }
    };
    drop(jar);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/event-stream".into()));
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: status_text.to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource status-reconnect.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_status_reconnect(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let es_cookie_stash = &ctx.es_cookie_stash;
    // net-api M4 fixture：eventsource/resources/status-reconnect.py（上游逐字
    // 等价——同 pin reconnect-fail.py 形态的 cookie 态机；cors.py run=
    // status-reconnect 委派同一实现）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let (status, status_text, body) = es_status_reconnect_response(es_cookie_stash, query);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/event-stream".into()));
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text,
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource last-event-id2.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_last_event_id2(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M4 fixture：eventsource/resources/last-event-id2.py（上游逐字
    // 等价——同 pin）。?type 静态三流：1 = id 持续（id:1 两条 + id:2 两条）；
    // 2 = `id:` 空 id 字段重置 lastEventId；3 = 裸 `id`（无冒号）同样重置
    // （spec §9.2.6 id 字段处理——字段值空串进缓冲）。type 4+ → invalid_test
    // （上游 else 臂）；type 解析失败 → 1（上游 except 臂）。
    let test_type = path_part
        .split_once('?')
        .map(|(_, q)| q)
        .and_then(|q| wpt_query_value(q, "type"))
        .map(|v| v.parse::<u8>().unwrap_or(1))
        .unwrap_or(1);
    let body = match test_type {
        2 => "id: 1\ndata: 1\n\nid:\ndata:2\n\ndata:3\n\n",
        3 => "id: 1\ndata: 1\n\nid\ndata:2\n\ndata:3\n\n",
        _ => "id: 1\ndata: 1\n\ndata: 2\n\nid: 2\ndata:3\n\ndata:4\n\n",
    };
    let body = if test_type > 3 {
        "data: invalid_test\n\n".to_string()
    } else {
        body.to_string()
    };
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/event-stream".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource cors-cookie.py 臂——注册表切片四迁入，体逐字节保持。
pub(crate) fn route_es_cors_cookie(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let es_cookie_stash = &ctx.es_cookie_stash;
    // net-api M4 fixture：eventsource/resources/cors-cookie.py（上游逐字等价
    // ——同 pin）。Last-Event-ID 缺省 → `id: 1`+`retry: 200`+`data: first
    // <cookie>` 并置 cookie；`Last-Event-ID: 1` → `id: 2`+`data: second
    // <cookie>` 并清 cookie（上游 expires 过去时刻——删除语义等价）；其余 →
    // Content-Type: stop。cookie 值 = 表含 ident 且请求以 credentials mode
    // include 发出 → "COOKIE"，否则 "NO_COOKIE"——runner 无 jar，跨源送达面
    // 以 FetchRequest.credentials 判定（RFC 6265bis §5.4——get-set-cookie.py
    // secure 同款先例；首次请求 cookie 未置，恒 NO_COOKIE）。
    // ACAO ← ?origin=（缺省回显请求 Origin 头；none → 不设）；ACAC ←
    // ?credentials=（缺省 true；none → 不设）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let last_event_id = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("last-event-id"))
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    let ident = wpt_query_value(query, "ident").unwrap_or_else(|| "test".into());
    let req_origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let origin = wpt_query_value(query, "origin").unwrap_or(req_origin);
    let credentials = wpt_query_value(query, "credentials").unwrap_or_else(|| "true".into());
    let mut jar = match es_cookie_stash.lock() {
        Ok(j) => j,
        Err(poisoned) => poisoned.into_inner(),
    };
    // 上游 cookie 判定先于本响应 set_cookie（request.cookies 为请求所携）。
    let cookie = if jar.contains_key(&ident) && req.credentials.as_deref() == Some("include") {
        "COOKIE"
    } else {
        "NO_COOKIE"
    };
    let mut headers: Vec<(String, String)> = Vec::new();
    if origin != "none" {
        headers.push(("access-control-allow-origin".into(), origin));
    }
    if credentials != "none" {
        headers.push(("access-control-allow-credentials".into(), credentials));
    }
    if req.method == "OPTIONS" {
        // OPTIONS preflight 先于状态机应答（ES withCredentials 跨源恒
        // preflight——Cache-Control 非 safelisted；ACAM ← 回显 ACRM、
        // ACAH ← 回显 ACRH，2xx + 空 body，状态不消费）。
        let acrm = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "GET".into());
        let acrh = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "cache-control".into());
        headers.push(("access-control-allow-methods".into(), acrm));
        headers.push(("access-control-allow-headers".into(), acrh));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    let body = if last_event_id.is_empty() {
        jar.insert(ident, "COOKIE".to_string());
        headers.push(("content-type".into(), "text/event-stream".into()));
        format!("id: 1\nretry: 200\ndata: first {}\n\n", cookie)
    } else if last_event_id == "1" {
        jar.remove(&ident);
        headers.push(("content-type".into(), "text/event-stream".into()));
        format!("id: 2\ndata: second {}\n\n", cookie)
    } else {
        headers.push(("content-type".into(), "stop".into()));
        format!("data: {}{}\n\n", last_event_id, cookie)
    };
    drop(jar);
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// eventsource/resources/cors.py 臂——注册表切片五迁入，体逐字节保持。
pub(crate) fn route_es_cors(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let es_cookie_stash = &ctx.es_cookie_stash;
    // net-api M4-S13 fixture：eventsource/resources/cors.py（上游逐字等价——
    // https://github.com/web-platform-tests/wpt/blob/3159769/eventsource/resources/
    // cors.py）。ACAO ← ?origin=（缺省回显请求 Origin 头）+ ACAC ← ?credentials=
    // （默认 true）；?run= 委派面（上游 run_other）——message（message.py 缺省
    // 参数形态）/redirect（common/redirect.py：302 + Location ← ?location）/
    // status-reconnect（cookie 态机，es_status_reconnect_response 共享实现）/
    // cache-control → text/event-stream + cache-control 模板体（回显请求
    // Cache-Control 头）。其余 run 值上游返空体。
    // net-api M5-S3：OPTIONS preflight 应答面——ES 请求恒携带 Cache-Control:
    // no-cache（非 CORS-safelisted，fetch spec §3.1.1）→ 跨源必须先过 preflight；
    // 上游 cors.py 无 OPTIONS 分支（真实 wptserve 部署由服务端配置补），本 runner
    // fixture 按「支持 ES 跨源的合规 CORS 服务端」语义应答：ACAM ← 回显 ACRM
    // （缺省 GET）+ ACAH ← 回显 ACRH（缺省 cache-control）。缺此则 ES 跨源恒
    // Failed to fetch → request-cache-control 跨源腿 onerror 未接线 → 页级
    // Timeout（M5-S2 轮归因，evidence/2026-10-14-m5-s3-*.md）。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let req_origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone())
        .unwrap_or_default();
    let acao = wpt_query_value(query, "origin").unwrap_or(req_origin);
    let acac = wpt_query_value(query, "credentials").unwrap_or_else(|| "true".into());
    let run = wpt_query_value(query, "run").unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("access-control-allow-origin".into(), acao));
    headers.push(("access-control-allow-credentials".into(), acac));
    if req.method == "OPTIONS" {
        let acrm = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-method"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "GET".into());
        let acrh = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("access-control-request-headers"))
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| "cache-control".into());
        headers.push(("access-control-allow-methods".into(), acrm));
        headers.push(("access-control-allow-headers".into(), acrh));
        // OPTIONS preflight 先于 run 委派应答（委派臂仅对实际请求生效——
        // preflight 落 302 则 2xx 门不过 → network error）。
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    if run == "redirect" {
        // 委派 common/redirect.py：302 + Location ← ?location（缺省 /）。
        // ACAO/ACAC 已先行设置——跨源 redirect 响应自身须过 CORS（fetch spec
        // §3.2 redirect 步）。
        let location = wpt_query_value(query, "location").unwrap_or_else(|| "/".into());
        headers.push(("location".into(), location));
        wpt_add_fetch_metadata(&mut headers, req, 302);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 302,
            status_text: "Redirect".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        }));
    }
    if run == "message" {
        // 委派 message.py 缺省参数形态：`data: data` + 尾 \n\n。
        headers.push(("content-type".into(), "text/event-stream".into()));
        let body = "data: data\n\n\n".to_string();
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        }));
    }
    if run == "status-reconnect" {
        let (status, status_text, body) = es_status_reconnect_response(es_cookie_stash, query);
        headers.push(("content-type".into(), "text/event-stream".into()));
        wpt_add_fetch_metadata(&mut headers, req, status);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status,
            status_text,
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        }));
    }
    let body = if run == "cache-control" {
        headers.push(("content-type".into(), "text/event-stream".into()));
        let echoed = req
            .headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case("cache-control"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        format!("data: {}\n\n", echoed)
    } else {
        String::new()
    };
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
