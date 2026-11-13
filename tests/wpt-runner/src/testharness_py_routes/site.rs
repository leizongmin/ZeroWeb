//! 站点根与静态/先例端点族——链头臂（切片二首批）、s346 module-fetch-tester、
//! css/样式/404/echo-content/set-cookie/cache.py/referrer-checker/beacon/fetch-with-body-worker。

use super::*;

/// html/semantics/scripting-1/the-script-element/module-fetch-tester.py 臂——s346
/// （html-semantics）：页面 fetch() stat 通道（concurrent-failed-imports 判据取
/// `?action=stat` 回计数）。非 stat 请求按上游镜像计数 + 404/200 分派（corpus 面
/// 页面 fetch() 不取模块本体——模块取回走 script fetcher 臂，两 face 同调
/// [`crate::testharness::module_fetch_tester_serve`] 共享 [`crate::testharness::MODULE_FETCH_TESTER_STASH`]，
/// 计数/应答逐字节一致）。
/// https://github.com/web-platform-tests/wpt/blob/master/html/semantics/scripting-1/the-script-element/module-fetch-tester.py
pub(crate) fn route_module_fetch_tester(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let key = wpt_query_value(query, "key").unwrap_or_default();
    let action = wpt_query_value(query, "action").unwrap_or_default();
    let mode = wpt_query_value(query, "mode").unwrap_or_default();
    let (status, status_text, body) = match crate::testharness::module_fetch_tester_serve(&key, &action, &mode) {
        Ok(body) => (200, "OK", body),
        Err(_) => (404, "Not Found", "not found".to_string()),
    };
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: status_text.to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// 站点根（wpt.test /） 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_root_page(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // net-api M3-S2 fixture：站点根（wpt.test /）——responsetype DONE 腿
    // `open('get', '/')` 须非空响应体；通用最小根页。
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "wpt.test root\n".to_string(),
        body_bytes: Some(b"wpt.test root\n".to_vec()),
    }))
}

/// fetch/api/request/resources/cache.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_cache_py(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let cache_py_stash = &ctx.cache_py_stash;
    // http-cache goal M1-b：request-cache 族 cache.py 端点仿真（上游
    // wptserve handler 1:1 移植——token 观察记录、querystate 取走 JSON、
    // ETag/Last-Modified/Expires/Vary/ACO 按参数回显、304 条件命中、
    // 302 redirect，见 wpt_cache_py_fixture；其前的缓存语义由外层
    // wpt_cache_py_wrapped_handler 承载）。
    Some(wpt_cache_py_fixture(req, cache_py_stash, path_part))
}

/// cookies/resources/set-cookie.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_set_cookie(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let cred_cookies = &ctx.cred_cookies;
    // s338：set-cookie.py 端点仿真（上游 wptserve handler 不在语料——
    // dynamic-import credentials 判据面的置 cookie 通道；helper 以
    // fetch(credentials: 'include') 于 check-cookie import 前调用，同源
    // 相对 + cross 绝对（www2）各一枚）。写案级 cookie 表（键 (host,
    // name)，值 "1"——上游 `name=1` 字面；path/samesite/secure 参数不建模
    // ——语料 path 覆盖 check-cookie 所在目录，jar 无 path 作用域近似，
    // FIXME 挂账）。回体/Content-Type 逐字镜像上游（application/json +
    // `var dummy='value';`）；ACO 反射头 runner fetch 通道无 CORS 执行面，
    // 不建模。host 取请求 URL authority（无 scheme = origin-absolute →
    // runner 页面恒 wpt.test）。
    // https://github.com/web-platform-tests/wpt/blob/3159769/cookies/resources/set-cookie.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let name = wpt_query_value(query, "name").unwrap_or_default();
    if !name.is_empty() {
        let host = req
            .url
            .split_once("://")
            .and_then(|(_, rest)| rest.find('/').map(|cut| &rest[..cut]))
            .map(|authority| authority.split(':').next().unwrap_or(authority).to_string())
            .unwrap_or_else(|| "wpt.test".to_string());
        if let Ok(mut stash) = cred_cookies.lock() {
            stash.insert((host, name), "1".to_string());
        }
    }
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "application/json".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "var dummy='value';".to_string(),
        body_bytes: Some(b"var dummy='value';".to_vec()),
    }))
}

/// FileAPI/file/resources/echo-content-escaped.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_echo_content_escaped(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // form-submission-0 三族（urlencoded2/text-plain/multipart-formdata
    // .window.js 经 enctypes-helper.js ACTION_URL）的提交回显端点——上游
    // wptserve echo-content-escaped.py 行为等价：请求体逐字节转义为 `\xNN`
    // 文本（可打印 ASCII 与 CR/LF 字面原样；`\` 自身转义 `\x5C`，防被
    // helper 的 `\xNN` 还原正则误吞），text/plain 交付——helper
    // `unescape()` 可逆，断言面即提交编码算法本体。CR/LF 不转义是语义
    // 关键：helper `unescape()` 首步 `/\r\n?|\n/g → "\r\n"` 归一只作用于
    // 字面 CR/LF（先于 `\xNN` 解码）——若转义为 `\x0D\x0A`，解码后不再
    // 归一，text/plain `\n`→`\r\n` 展开案即错。
    let raw: Vec<u8> = req
        .body_bytes
        .clone()
        .or_else(|| req.body.as_ref().map(|b| b.as_bytes().to_vec()))
        .unwrap_or_default();
    let mut escaped = String::with_capacity(raw.len() * 4);
    for &byte in &raw {
        match byte {
            b'\\' => escaped.push_str("\\x5C"),
            b'\r' | b'\n' | 0x20..=0x7E => escaped.push(byte as char),
            _ => escaped.push_str(&format!("\\x{byte:02X}")),
        }
    }
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: escaped.clone(),
        body_bytes: Some(escaped.into_bytes()),
    }))
}

/// html/semantics/forms/form-submission-target/resources/
/// form-target-request-header-helper.py 臂——form-target-request-header 的 POST
/// 端点（s431）。上游 wptserve helper 行为等价：请求头含 Content-Type → 'OK'
/// 否则 'FAIL'；channelname 取 POST body `channelname=<name>` 首个 '=' 后段；
/// 回体为 BroadcastChannel postMessage 端点文档（text/html）。runner 的
/// py_routes 不执行上游 .py（语料拉取面排除 .py），故按行为等价仿真。
pub(crate) fn route_form_target_request_header(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let has_content_type = req.headers.iter().any(|(n, _)| n.eq_ignore_ascii_case("content-type"));
    let result = if has_content_type { "OK" } else { "FAIL" };
    let channel_name = req.body.as_deref().unwrap_or("").split('=').nth(1).unwrap_or("");
    let body = format!(
        "\n<script>\nconst channel = new BroadcastChannel('{}');\nchannel.postMessage('{}', '*');\nwindow.close();\n</script>\n",
        channel_name, result
    );
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "text/html".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// html/semantics/document-metadata/the-link-element/stylesheet.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_link_element_stylesheet(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let link_sheet_stash = &ctx.link_sheet_stash;
    // s150（html-semantics M2 切片 150）：link-multiple-load-events /
    // document-without-browsing-context 的请求计数端点——上游 wptserve
    // stylesheet.py 行为等价（take-and-clear 计数）：带 `count` 参数的
    // 请求 take 出当前计数并返回（stash 不回填——下次 count 读取归 0）；
    // 无 count 的资源请求 take 后 put(count+1) 并返回 CSS 体。
    // https://github.com/web-platform-tests/wpt/blob/master/
    // html/semantics/document-metadata/the-link-element/stylesheet.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let id = wpt_query_value(query, "id").unwrap_or_default();
    let count_read = wpt_query_value(query, "count").is_some();
    let taken = link_sheet_stash
        .lock()
        .ok()
        .and_then(|mut s| s.remove(&id))
        .unwrap_or(0);
    let (content_type, body) = if count_read {
        ("text/plain".into(), taken.to_string())
    } else {
        if let Ok(mut s) = link_sheet_stash.lock() {
            s.insert(id, taken + 1);
        }
        ("text/css".into(), "body {color: red;}".to_string())
    };
    let mut headers: Vec<(String, String)> = Vec::new();
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

/// html/semantics/document-metadata/interactions-of-styling-and-scripting/stylesheet.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_interactions_stylesheet(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // s163（html-semantics M2 切片 163）：conditionally-block-rendering-on-link-
    // media-attr 的参数化 CSS 端点——上游 stylesheet.py 行为等价（pin master 逐字
    // 核对）。响应体/延迟逻辑在 [`serve_interactions_stylesheet`]（引擎 base 槽
    // 外链 CSS 通道共用——该通道不过 fetch 桥，见其注）。
    // https://github.com/web-platform-tests/wpt/blob/master/html/semantics/
    // document-metadata/interactions-of-styling-and-scripting/stylesheet.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let (content_type, body) = serve_interactions_stylesheet(query);
    let mut headers: Vec<(String, String)> = Vec::new();
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

/// html/semantics/scripting-1/the-script-element/serve-custom-response.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_serve_custom_response(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let serve_custom_stash = &ctx.serve_custom_stash;
    // s267：serve-custom-response.py 动态端点 action 三查询臂（上游
    // wptserve handler 不在语料；模块取回臂在 [`wpt_data_script_fetcher`]
    // ——页面 fetch() 与模块取回共享 [`SERVE_CUSTOM_RESPONSE_STASH`]）。
    // stat 只读回计数（不计数）；reset-stat 归零；set 更新 code/
    // content-type/network-error（解析失败参数按缺席忽略——corpus 面
    // 恒合法）。回体 text/plain：stat = 计数十进制，reset/set = "OK"；
    // 无 action 落空体（页面 fetch() 直取端点无 corpus face）。
    // https://github.com/web-platform-tests/wpt/blob/master/html/semantics/
    // scripting-1/the-script-element/serve-custom-response.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let key = wpt_query_value(query, "key").unwrap_or_default();
    let action = wpt_query_value(query, "action").unwrap_or_default();
    let mut body = String::new();
    if let Ok(mut stash) = serve_custom_stash.lock() {
        let state = stash.entry(key).or_default();
        match action.as_str() {
            "stat" => body = state.run_count.to_string(),
            "reset-stat" => {
                state.run_count = 0;
                body = "OK".to_string();
            }
            "set" => {
                if let Some(code) = wpt_query_value(query, "code").and_then(|c| c.parse::<u16>().ok()) {
                    state.status_code = code;
                }
                if let Some(ct) = wpt_query_value(query, "content-type") {
                    state.content_type = ct;
                }
                if let Some(ne) = wpt_query_value(query, "network-error") {
                    state.network_error = ne.eq_ignore_ascii_case("true");
                }
                body = "OK".to_string();
            }
            _ => {}
        }
    }
    let mut headers: Vec<(String, String)> = Vec::new();
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

/// html/semantics/document-metadata/the-link-element/resources/css.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_link_css_py(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // s151（html-semantics M2 切片 151）：link-style-error 三文件的响应头
    // 参数化 CSS 端点——上游 css.py 行为等价（pin master 逐字核对）：
    // `content_type` 查询参数原样作 Content-Type 头（缺席 → **无 CT 头**
    // ——「no Content-Type」案的 sniff 放行面）；`nosniff` 旗标 →
    // x-content-type-options: nosniff；`cors` 旗标上游 → ACAO: *（语料
    // URL 未携带该参数，两头无条件交付已覆盖其语义）。体恒
    // `body { background:red }`、status 200。
    // https://github.com/web-platform-tests/wpt/blob/master/
    // html/semantics/document-metadata/the-link-element/resources/css.py
    // runner 通道补 ACAO/ACAH 外的 Access-Control-Allow-Origin 与
    // Access-Control-Expose-Headers `*`：shim 侧 s150 helper 经通用 fetch
    // （默认 cors 模式）取回——跨源案无 ACAO 即 reject（fetch CORS check
    // 臂）；ACAO 通配放行后 cors 过滤响应仍剥非 safelisted 头
    //（x-content-type-options 读不到）——ACEH `*` 通配全暴露。上游
    // wptserve 真环境 stylesheet 加载不走 CORS 检查，两头是通道妥协，
    // 语料不读 ACAO/ACEH，期望面不变。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(ct) = wpt_query_value(query, "content_type") {
        headers.push(("content-type".into(), ct));
    }
    if wpt_query_value(query, "nosniff").is_some() {
        headers.push(("x-content-type-options".into(), "nosniff".into()));
    }
    headers.push(("access-control-allow-origin".into(), "*".into()));
    headers.push(("access-control-expose-headers".into(), "*".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let body = "body { background:red }".to_string();
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// html/semantics/embedded-content/the-img-element/resources/referrer-checker-img.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_referrer_checker_img(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // s270（html-semantics）：referrer-checker-img.py 动态端点仿真（上游
    // wptserve handler 不在语料——`request.headers.referer` 与
    // `?expected_referrer=` 比对：匹配回 200 image/png 字节，不匹配回
    // 404 "Not found"）。runner fetcher 无 Referer 头建模（同 s116
    // module 臂简化面——真实 HTTP 头不合成），actual Referer 恒 ""：语料
    // 唯一消费者 image-loading-lazy-referrerpolicy-change 的期望值恒空串
    // （"无 Referer"）→ 引擎 img 取回不发 Referer 恒匹配。「deferred 装载
    // 用最新 referrerpolicy」的可观察差异仅在 Referer 头有无比对——引擎
    // relevant mutations 臂已重验（`__zwImgAttrMutated5277`
    // referrerpolicy/crossorigin 臂），策略值在无头通道不可观察。
    // https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data
    // https://github.com/web-platform-tests/wpt/blob/master/
    // html/semantics/embedded-content/the-img-element/resources/referrer-checker-img.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let expected = wpt_query_value(query, "expected_referrer").unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "image/png".into()));
    if expected.is_empty() {
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(ZW_ONE_PX_PNG.to_vec()),
        }));
    }
    wpt_add_fetch_metadata(&mut headers, req, 404);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 404,
        status_text: "Not Found".to_string(),
        headers,
        body: "Not found".to_string(),
        body_bytes: Some(b"Not found".to_vec()),
    }))
}

/// html/semantics/embedded-content/the-img-element/404-response-with-actual-image-data.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_404_response_with_image(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // s288（html-semantics）：404-response-with-actual-image-data.py 端点仿真
    //（上游 wptserve handler 不在语料——固定返 404 状态 + image/png 体
    // [`ZW_S288_404_PNG`]，上游 .py `decodebytes` 内嵌体逐字）。测试断言面：
    // update the image data 的响应处理只门槛网络错误，HTTP 状态不入门——
    // 404 + 可解码体亦 available（load 派发、onerror unreached）。
    // https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data
    // https://github.com/web-platform-tests/wpt/blob/master/
    // html/semantics/embedded-content/the-img-element/404-response-with-actual-image-data.py
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "image/png".into()));
    wpt_add_fetch_metadata(&mut headers, req, 404);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 404,
        status_text: "Not Found".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(ZW_S288_404_PNG.to_vec()),
    }))
}

/// html/semantics/embedded-content/the-img-element/resources/image-alternate-no-store.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_image_alternate_no_store(
    ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let alt_no_store_switch = &ctx.alt_no_store_switch;
    // s295（html-semantics）：image-alternate-no-store.py 端点仿真（上游
    // wptserve handler 不在语料——固定 stash 键布尔逐请求交替 green.png/
    // blue-10.png 回体，Content-Type: image/png + Cache-Control: no-store，
    // 首 falsy→green、次→blue-10）。语料消费者 image-alternate-no-store.
    // window.js 断言面：同 src 重指派走 list-of-available-images 复用
    //（spec updating-the-image-data step 7.4——ignore higher-layer caching
    // flag 入列）不触发额外取回 → 两次 decode 同图 naturalWidth 同值；若
    // 重取回，交替臂换图（100↔10 尺寸差）宽度断言即翻红——交替正是上游
    // 钉「不额外取回」的机制。no-store 头为 HTTP 缓存层语义，shim 复用面
    // 走 _zwAvailableImages 内存表，头只忠实携带不入门。
    // https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data
    // https://github.com/web-platform-tests/wpt/blob/
    // 315976933870b34d6ea30e3f6643403edae678ba/
    // html/semantics/embedded-content/the-img-element/resources/image-alternate-no-store.py
    let switch = alt_no_store_switch
        .lock()
        .map(|mut s| {
            let v = *s;
            *s = !v;
            v
        })
        .unwrap_or(false);
    let mut headers: Vec<(String, String)> = Vec::new();
    headers.push(("content-type".into(), "image/png".into()));
    headers.push(("cache-control".into(), "no-store".into()));
    wpt_add_fetch_metadata(&mut headers, req, 200);
    let body = if switch { ZW_S295_BLUE_10 } else { ZW_S295_GREEN_100 };
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(body.to_vec()),
    }))
}

/// html/semantics/embedded-content/the-img-element/resources/image-and-stash.py 臂——注册表首批（切片二）迁移，体逐字节保持。
pub(crate) fn route_image_and_stash(
    ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let img_and_stash = &ctx.img_and_stash;
    // s295（html-semantics）：image-and-stash.py 端点仿真（上游 wptserve
    // handler 不在语料——`?increment=<uuid>` per-uuid 计数 +1 回 image/png
    // 体、`?read=<uuid>` 只读回计数值文本（take+put-back 不删）、其余 404）。
    // 语料消费者 list-of-available-images-does-not-coalesce-in-flight-
    // requests.sub.tentative 断言面：list of available images 不合并
    // in-flight 请求（spec：列表只在 completely downloaded + load 后入列
    // ——非请求合并器）→ 同 URL 两 img 各自独立 fetch，服务端须见恰 2 请求
    // （fetch(path?read=) 回 "2"）。上游 increment 回 `resources/image.png`
    //（40x30）——页断言仅 onload 到场 + 服务端计数、无尺寸面，以
    // [`ZW_ONE_PX_PNG`] 最小合法图承载（s270 referrer 臂同款简化）。
    // `{{uuid()}}` 模板 runner 无替换（字面常量键）——案内自洽，per-case
    // 计数表案内自清语义等价；pipe=trickle(d1) 真实时序交错不可复现（在册
    // trickle pipe 不可复现），计数与 onload 断言不依赖时序面不涉。
    // https://html.spec.whatwg.org/multipage/#the-list-of-available-images
    // https://github.com/web-platform-tests/wpt/blob/
    // 315976933870b34d6ea30e3f6643403edae678ba/
    // html/semantics/embedded-content/the-img-element/resources/image-and-stash.py
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(uuid) = wpt_query_value(query, "increment") {
        if let Ok(mut stash) = img_and_stash.lock() {
            *stash.entry(uuid).or_insert(0) += 1;
        }
        headers.push(("content-type".into(), "image/png".into()));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(ZW_ONE_PX_PNG.to_vec()),
        }));
    }
    if let Some(uuid) = wpt_query_value(query, "read") {
        let value = img_and_stash
            .lock()
            .ok()
            .and_then(|stash| stash.get(&uuid).copied())
            .unwrap_or(0);
        headers.push(("content-type".into(), "text/plain".into()));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Some(Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: value.to_string(),
            body_bytes: Some(value.to_string().into_bytes()),
        }));
    }
    headers.push(("content-type".into(), "text/plain".into()));
    wpt_add_fetch_metadata(&mut headers, req, 404);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 404,
        status_text: "Not Found".to_string(),
        headers,
        body: "Not found".to_string(),
        body_bytes: Some(b"Not found".to_vec()),
    }))
}

/// beacon inspect-header/content-type-and-body/url-echo 族 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_beacon_inspect_header(
    ctx: &PyEndpointCtx,
    clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let beacon_stash = &ctx.beacon_stash;
    // page-state-compat M2 fixture：beacon/resources 三端点（上游行为等价——
    // https://github.com/web-platform-tests/wpt/blob/315976933870/beacon/resources/
    // inspect-header.py、content-type-and-body.py、url-echo.py）。共享 take 语义
    // stash（wptserve server.stash.put/take——读后即删）；beacon POST 打 put/store，
    // 断言侧 fetch 轮询打 get/stat。
    let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
    let cmd = wpt_query_value(query, "cmd")
        .map(|v| v.to_lowercase())
        .unwrap_or_default();
    let id = wpt_query_value(query, "id").unwrap_or_default();
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut status: u16 = 200;
    let mut body = String::new();
    if clean.ends_with("inspect-header.py") {
        headers.push(("content-type".into(), "text/plain".into()));
        let header = wpt_query_value(query, "header").unwrap_or_default();
        if cmd == "put" {
            // 上游 request.headers.get(header, b"")——头缺席存空串。
            let value = req
                .headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(&header))
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            if let Ok(mut stash) = beacon_stash.lock() {
                stash.insert(id, value);
            }
        } else if cmd == "get" {
            // take 语义；有值（含空串）回 x-request-<header>（pollResult 依
            // headers.get != undefined 结束轮询——无值则不发射头，轮询继续）。
            let stored = beacon_stash.lock().ok().and_then(|mut s| s.remove(&id));
            if let Some(value) = stored {
                headers.push((format!("x-request-{}", header.to_lowercase()), value));
            }
        } else {
            status = 400;
            body = "ERROR: Bad Command!".into();
        }
    } else if clean.ends_with("content-type-and-body.py") {
        headers.push(("content-type".into(), "text/plain".into()));
        if cmd == "put" {
            // 上游 stash[id] = Content-Type + ": " + body（头缺席 CT 为空串——
            // ArrayBuffer/View 无 CT 腿期望 `: <bytes>`）。
            let ct = req
                .headers
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            let body_bytes = req
                .body_bytes
                .clone()
                .or_else(|| req.body.clone().map(String::into_bytes))
                .unwrap_or_default();
            let value = format!("{}: {}", ct, String::from_utf8_lossy(&body_bytes));
            if let Ok(mut stash) = beacon_stash.lock() {
                stash.insert(id, value);
            }
        } else if cmd == "get" {
            let stored = beacon_stash.lock().ok().and_then(|mut s| s.remove(&id));
            match stored {
                Some(value) => body = value,
                // 上游缺值 set_error(400)（cmd 合法但 stash 空）。
                None => {
                    status = 400;
                    body = "ERROR: Bad Command!".into();
                }
            }
        } else {
            status = 400;
            body = "ERROR: Bad Command!".into();
        }
    } else {
        // url-echo.py：store 存原始（百分号编码形态）query 串；stat take 返回
        //（无值空体，状态恒 200——上游仅非法 cmd 置 400）。
        if cmd == "store" {
            let raw_query = req.url.split_once('?').map(|(_, q)| q).unwrap_or("");
            if let Ok(mut stash) = beacon_stash.lock() {
                stash.insert(id, raw_query.to_string());
            }
            headers.push(("content-type".into(), "text/plain".into()));
        } else if cmd == "stat" {
            headers.push(("content-type".into(), "text/plain".into()));
            body = beacon_stash
                .lock()
                .ok()
                .and_then(|mut s| s.remove(&id))
                .unwrap_or_default();
        } else {
            status = 400;
        }
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

/// html/semantics/scripting-1/the-script-element/resources/cross-origin.py 臂——
/// s383（html-semantics M2 切片 383）：script-crossorigin-network 车道。上游 1:1 移植：
/// - 请求带 Origin 头 → ACAO=origin 回显 + ACAM=GET + ACAC=true（缺省不设）；
/// - OPTIONS → 空体先行返回（CORS 头照设——上游先置头后判方法）；
/// - Content-Type: text/javascript；体按 Cookie 头 milk 值选——缺省/milk≠1 →
///   `var included = false;`，milk=1 → `var included = true;`。
///
/// cookie 送达面由 shim 附加 Cookie 头承载（part01 s383 臂——动态 classic 脚本按
/// crossOrigin 推导 credentials mode，RFC 6265bis §5.4 判据；runner 无 jar，案内
/// document.cookie 写入的 in-JS jar 经 `_zwCookieHeaderForUrl` 桥出请求头，路由只读
/// 头不持案内态——与 cors-cookie.py 的 stash 自写先例不同源，上游 request.cookies
/// 语义等价面）。css-module/text-module/json-module 三份 resources/cross-origin.py
/// 为不同上游脚本（module 载荷 + ?id/?origin 参数），不在本臂 Exact 路径内。
///
/// <https://github.com/web-platform-tests/wpt/blob/master/html/semantics/scripting-1/the-script-element/resources/cross-origin.py>
pub(crate) fn route_cross_origin_py(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    _path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    let mut headers: Vec<(String, String)> = Vec::new();
    let req_origin = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("origin"))
        .map(|(_, v)| v.clone());
    if let Some(origin) = req_origin {
        headers.push(("access-control-allow-origin".into(), origin));
        headers.push(("access-control-allow-methods".into(), "GET".into()));
        headers.push(("access-control-allow-credentials".into(), "true".into()));
    }
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
    headers.push(("content-type".into(), "text/javascript".into()));
    // 上游 request.cookies.first(b"milk")扫全部 cookie 对：None → false；
    // value == "1" → true；余 → false。
    let milk = req
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("cookie"))
        .and_then(|(_, v)| {
            v.split(';')
                .filter_map(|pair| pair.trim().split_once('='))
                .find(|(name, _)| name.eq_ignore_ascii_case("milk"))
        })
        .map(|(_, value)| value.trim().to_string());
    let included = milk.as_deref() == Some("1");
    let body = format!("var included = {};", included);
    wpt_add_fetch_metadata(&mut headers, req, 200);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}

/// service-workers fetch-with-body-worker 臂——注册表切片三迁入，体逐字节保持。
pub(crate) fn route_fetch_with_body_worker(
    _ctx: &PyEndpointCtx,
    _clean: &str,
    path_part: &str,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>> {
    // https://github.com/web-platform-tests/wpt/blob/04067ce9c7c2165e71ad7d0dde10a4c5cb394a83/service-workers/service-worker/resources/fetch-with-body-worker.py
    let has_body = req.body_bytes.as_ref().is_some_and(|bytes| !bytes.is_empty())
        || req.body.as_ref().is_some_and(|body| !body.is_empty());
    let status = if has_body { 200 } else { 400 };
    let body = if has_body { "BODY" } else { "NO BODY" };
    let mut headers = wpt_pipe_headers(path_part.split_once('?').map(|(_, query)| query).unwrap_or(""));
    wpt_add_fetch_metadata(&mut headers, req, status);
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }))
}
