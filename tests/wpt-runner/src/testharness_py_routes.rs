//! py 端点路由底座（46 号 runner-net-infra M3 框架化）——fetch handler 闭包体
//! 自 testharness.rs 整体迁出的底座切片：案级 12 项 stash/状态收敛
//! [`PyEndpointCtx`]，分派入口 [`dispatch`]。臂序（先命中先返回）与行为
//! 逐字节保持（全量 A/B 门禁，见 goal 基线归因档）。注册表化（保序
//! `PyRoute` 表 + 端点注册项）按族分批落账：本切片先承载迁出体，后续
//! 批次把 `if` 臂逐族转注册项并迁四 fetcher 闭包臂（蓝图见 py 普查档 §三）。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::testharness::{
    CachePyStash, CredentialCookieStash, HARNESS_LOADED_TAIL_CALL, HARNESS_REPORTER_SOURCE, ServeCustomResponseStash,
    WPT_TOKEN_STASH, ZW_ONE_PX_PNG, ZW_S288_404_PNG, ZW_S295_BLUE_10, ZW_S295_GREEN_100,
    apply_harness_completion_patch, apply_wpt_substitutions, es_status_reconnect_response, inline_local_scripts,
    serve_interactions_stylesheet, sniff_document_encoding, wpt_add_fetch_metadata, wpt_apply_get_query_substitutions,
    wpt_cache_py_fixture, wpt_load_error_events_response, wpt_pipe_headers, wpt_pipe_status, wpt_query_value,
    wpt_query_value_bytes, wpt_sidecar_headers, wpt_static_resource_headers, wpt_status_text, wpt_url_path,
};

/// fetch handler 案级上下文——原闭包捕获的 12 项 per-case 状态收敛；各表
/// 语义注释随原声明保留在 testharness.rs 构建点（handler per-case 构建、
/// 案内键自清、跨案零泄漏）。
pub(crate) struct PyEndpointCtx {
    pub(crate) root: PathBuf,
    pub(crate) serve_custom_stash: ServeCustomResponseStash,
    pub(crate) cred_cookies: CredentialCookieStash,
    pub(crate) beacon_stash: Arc<Mutex<HashMap<String, String>>>,
    pub(crate) link_sheet_stash: Arc<Mutex<HashMap<String, u32>>>,
    pub(crate) alt_no_store_switch: Arc<Mutex<bool>>,
    pub(crate) img_and_stash: Arc<Mutex<HashMap<String, u32>>>,
    pub(crate) get_set_cookie_stash: Arc<Mutex<std::collections::BTreeMap<String, (String, bool)>>>,
    pub(crate) es_cookie_stash: Arc<Mutex<std::collections::BTreeMap<String, String>>>,
    pub(crate) vary_value_override: Arc<Mutex<Option<String>>>,
    pub(crate) cache_py_stash: CachePyStash,
    #[allow(dead_code)]
    // 与外层 wpt_cache_py_wrapped_handler 共享同一 Arc（包裹器承载缓存语义；dispatch 内 cache.py 臂直调 fixture）——切片对齐保留
    pub(crate) case_cache: Arc<Mutex<zero_net::HttpCache>>,
}

/// py 端点路由匹配谓词——沿既有臂口径（py 普查档 §三：路径全等；
/// 目录+文件名后缀、裸后缀与复合谓词按臂原条件 1:1 引入）。
pub(crate) enum PyRouteMatch {
    /// 路径全等（归一后 clean；空串 = 站点根）。
    Exact(&'static str),
    /// 路径全等多臂 OR（同族多端点共享 handler）。
    Exacts(&'static [&'static str]),
    /// 目录前缀 + 单文件名后缀。
    DirSuffix { dir: &'static str, suffix: &'static str },
    /// 目录前缀 + 多文件名后缀任一（原 `starts_with && (ends_with || …)` 臂）。
    DirSuffixes {
        dir: &'static str,
        suffixes: &'static [&'static str],
    },
    /// 裸文件名后缀（任意目录——原无前缀条件的 ends_with 臂）。
    Suffix(&'static str),
    /// 复合谓词（fn 指针——原组合条件臂）。
    Custom(PyPredicate),
}

impl PyRouteMatch {
    fn test(&self, clean: &str, req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
        match self {
            Self::Exact(path) => clean == *path,
            Self::Exacts(paths) => paths.contains(&clean),
            Self::DirSuffix { dir, suffix } => clean.starts_with(dir) && clean.ends_with(suffix),
            Self::DirSuffixes { dir, suffixes } => {
                clean.starts_with(dir) && suffixes.iter().any(|s| clean.ends_with(s))
            }
            Self::Suffix(suffix) => clean.ends_with(suffix),
            Self::Custom(pred) => pred(clean, req),
        }
    }
}

/// 路由处理器签名——ctx 案级状态 + clean（归一路径，无 query）+
/// path_part（归一路径，含 query）+ 原请求；`Some` = 终答（Ok/Err
/// 均交付），`None` = 未命中，落穿下一臂（承载既有臂内条件不返回
/// 即续链语义）。
pub(crate) type PyRouteHandler = fn(
    &PyEndpointCtx,
    &str,
    &str,
    &zero_engine::fetch_bridge::FetchRequest,
) -> Option<Result<zero_engine::fetch_bridge::FetchResponse, String>>;

/// 复合谓词签名（clean + 请求；纯函数，静态表内联不可表达时用）。
pub(crate) type PyPredicate = fn(&str, &zero_engine::fetch_bridge::FetchRequest) -> bool;

/// py 端点路由注册项（py 普查档 §三蓝图）——保序注册表行；表序 =
/// 既有 if 链臂序（先命中先返回），新端点 = 注册项声明。
pub(crate) struct PyRoute {
    pub(crate) matches: PyRouteMatch,
    pub(crate) handle: PyRouteHandler,
}

/// 保序注册表——首批迁入 dispatch 链头 12 条全等匹配臂（切片二，
/// 行为逐字节保持）；后续批次按臂序前缀推进注册项直至 84 臂全量
/// 迁入。
fn py_routes() -> &'static [PyRoute] {
    &[
        PyRoute {
            matches: PyRouteMatch::Exact(""),
            handle: route_root_page,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/request/resources/cache.py"),
            handle: route_cache_py,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("cookies/resources/set-cookie.py"),
            handle: route_set_cookie,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("FileAPI/file/resources/echo-content-escaped.py"),
            handle: route_echo_content_escaped,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("html/semantics/document-metadata/the-link-element/stylesheet.py"),
            handle: route_link_element_stylesheet,
        },
        PyRoute {
            matches: PyRouteMatch::Exact(
                "html/semantics/document-metadata/interactions-of-styling-and-scripting/stylesheet.py",
            ),
            handle: route_interactions_stylesheet,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("html/semantics/scripting-1/the-script-element/serve-custom-response.py"),
            handle: route_serve_custom_response,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("html/semantics/document-metadata/the-link-element/resources/css.py"),
            handle: route_link_css_py,
        },
        PyRoute {
            matches: PyRouteMatch::Exact(
                "html/semantics/embedded-content/the-img-element/resources/referrer-checker-img.py",
            ),
            handle: route_referrer_checker_img,
        },
        PyRoute {
            matches: PyRouteMatch::Exact(
                "html/semantics/embedded-content/the-img-element/404-response-with-actual-image-data.py",
            ),
            handle: route_404_response_with_image,
        },
        PyRoute {
            matches: PyRouteMatch::Exact(
                "html/semantics/embedded-content/the-img-element/resources/image-alternate-no-store.py",
            ),
            handle: route_image_alternate_no_store,
        },
        PyRoute {
            matches: PyRouteMatch::Exact(
                "html/semantics/embedded-content/the-img-element/resources/image-and-stash.py",
            ),
            handle: route_image_and_stash,
        },
        PyRoute {
            matches: PyRouteMatch::Exacts(&[
                "beacon/resources/inspect-header.py",
                "beacon/resources/content-type-and-body.py",
                "beacon/resources/url-echo.py",
            ]),
            handle: route_beacon_inspect_header,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("service-workers/service-worker/resources/fetch-with-body-worker.py"),
            handle: route_fetch_with_body_worker,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/upload.py",
            },
            handle: route_xhr_upload,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/form.py",
            },
            handle: route_xhr_form,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/echo-content-type.py",
            },
            handle: route_xhr_echo_content_type,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffixes {
                dir: "fetch/api/resources/",
                suffixes: &["/echo-content.py", "/echo-content.h2.py"],
            },
            handle: route_fetch_echo_content,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-origin-header.py",
            },
            handle: route_xhr_access_control_origin_header,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/over-1-meg.txt",
            },
            handle: route_xhr_over_1_meg,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-basic-allow-star.py",
            },
            handle: route_xhr_basic_allow_star,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffixes {
                dir: "xhr/",
                suffixes: &[
                    "/access-control-basic-allow.py",
                    "/access-control-basic-allow-no-credentials.py",
                ],
            },
            handle: route_xhr_basic_allow,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/redirect-cors.py",
            },
            handle: route_xhr_redirect_cors,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "fetch/api/resources/",
                suffix: "/redirect.py",
            },
            handle: route_fetch_redirect,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/resources/redirect-empty-location.py"),
            handle: route_fetch_redirect_empty_location,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/resources/method.py"),
            handle: route_fetch_method,
        },
        PyRoute {
            matches: PyRouteMatch::Custom(pred_xhr_authentication),
            handle: route_xhr_authentication,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/image.gif",
            },
            handle: route_xhr_image_gif,
        },
        PyRoute {
            matches: PyRouteMatch::Suffix("/echo-headers.py"),
            handle: route_echo_headers,
        },
        PyRoute {
            matches: PyRouteMatch::Suffix("/delay.py"),
            handle: route_delay,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("xhr/resources/trickle.py"),
            handle: route_xhr_trickle,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("xhr/resources/top.txt"),
            handle: route_xhr_top_txt,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-basic-put-allow.py",
            },
            handle: route_xhr_basic_put_allow,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/reset-token.py",
            },
            handle: route_xhr_reset_token,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffixes {
                dir: "xhr/",
                suffixes: &[
                    "/access-control-basic-preflight-cache.py",
                    "/access-control-basic-preflight-cache-timeout.py",
                    "/access-control-basic-preflight-cache-invalidation.py",
                ],
            },
            handle: route_xhr_preflight_cache,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffixes {
                dir: "xhr/",
                suffixes: &[
                    "/access-control-preflight-request-header-returns-origin.py",
                    "/access-control-preflight-request-allow-headers-returns-star.py",
                ],
            },
            handle: route_xhr_preflight_returns,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/headers.py",
            },
            handle: route_xhr_headers,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/chunked.py",
            },
            handle: route_xhr_chunked,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/parse-headers.py",
            },
            handle: route_xhr_parse_headers,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-basic-cors-safelisted-response-headers.py",
            },
            handle: route_xhr_safelisted_response_headers,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-basic-cors-safelisted-request-headers.py",
            },
            handle: route_xhr_safelisted_request_headers,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/no-custom-header-on-preflight.py",
            },
            handle: route_xhr_no_custom_header_on_preflight,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-preflight-request-header-lowercase.py",
            },
            handle: route_xhr_preflight_header_lowercase,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-preflight-request-header-sorted.py",
            },
            handle: route_xhr_preflight_header_sorted,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-preflight-request-headers-origin.py",
            },
            handle: route_xhr_preflight_headers_origin,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/access-control-preflight-request-invalid-status.py",
            },
            handle: route_xhr_preflight_invalid_status,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/echo-content-cors.py",
            },
            handle: route_xhr_echo_content_cors,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/resources/authentication.py"),
            handle: route_fetch_authentication,
        },
        PyRoute {
            matches: PyRouteMatch::Suffix("/dump-authorization-header.py"),
            handle: route_dump_authorization_header,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/corsenabled.py",
            },
            handle: route_xhr_corsenabled,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/redirect.py",
            },
            handle: route_xhr_redirect,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/well-formed.xml",
            },
            handle: route_xhr_well_formed_xml,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/pass.txt",
            },
            handle: route_xhr_pass_txt,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: "/content.py",
            },
            handle: route_xhr_content,
        },
        PyRoute {
            matches: PyRouteMatch::Custom(pred_status_py),
            handle: route_status_py,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/message.py",
            },
            handle: route_es_message,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/message2.py",
            },
            handle: route_es_message2,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/last-event-id.py",
            },
            handle: route_es_last_event_id,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: ".event_stream",
            },
            handle: route_es_event_stream,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/reconnect-fail.py",
            },
            handle: route_es_reconnect_fail,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/status-reconnect.py",
            },
            handle: route_es_status_reconnect,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/last-event-id2.py",
            },
            handle: route_es_last_event_id2,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/cors-cookie.py",
            },
            handle: route_es_cors_cookie,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("html/semantics/scripting-1/the-script-element/module-fetch-tester.py"),
            handle: route_module_fetch_tester,
        },
    ]
}

/// html/semantics/scripting-1/the-script-element/module-fetch-tester.py 臂——s346
/// （html-semantics）：页面 fetch() stat 通道（concurrent-failed-imports 判据取
/// `?action=stat` 回计数）。非 stat 请求按上游镜像计数 + 404/200 分派（corpus 面
/// 页面 fetch() 不取模块本体——模块取回走 script fetcher 臂，两 face 同调
/// [`crate::testharness::module_fetch_tester_serve`] 共享 [`crate::testharness::MODULE_FETCH_TESTER_STASH`]，
/// 计数/应答逐字节一致）。
/// https://github.com/web-platform-tests/wpt/blob/master/html/semantics/scripting-1/the-script-element/module-fetch-tester.py
fn route_module_fetch_tester(
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
fn route_root_page(
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
fn route_cache_py(
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
fn route_set_cookie(
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
fn route_echo_content_escaped(
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

/// html/semantics/document-metadata/the-link-element/stylesheet.py 臂——注册表首批（切片二）迁移，体逐字节保持。
fn route_link_element_stylesheet(
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
fn route_interactions_stylesheet(
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
fn route_serve_custom_response(
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
fn route_link_css_py(
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
fn route_referrer_checker_img(
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
fn route_404_response_with_image(
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
fn route_image_alternate_no_store(
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
fn route_image_and_stash(
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
fn route_beacon_inspect_header(
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
/// service-workers fetch-with-body-worker 臂——注册表切片三迁入，体逐字节保持。
fn route_fetch_with_body_worker(
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
/// xhr upload.py 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_upload(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: content.clone(),
        body_bytes: Some(content.into_bytes()),
    }))
}
/// xhr form.py 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_form(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: content.clone(),
        body_bytes: Some(content.into_bytes()),
    }))
}
/// xhr echo-content-type.py 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_echo_content_type(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: ct.clone(),
        body_bytes: Some(ct.into_bytes()),
    }))
}
/// fetch/api/resources/ echo-content(.h2) 族 臂——注册表切片三迁入，体逐字节保持。
fn route_fetch_echo_content(
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
/// xhr access-control-origin-header 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_access_control_origin_header(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr over-1-meg.txt 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_over_1_meg(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr access-control-basic-allow-star 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_basic_allow_star(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr access-control-basic-allow(-no-credentials) 族 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_basic_allow(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr redirect-cors 臂——注册表切片三迁入，体逐字节保持。
fn route_xhr_redirect_cors(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 302,
        status_text: wpt_status_text(302).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}

/// xhr authentication 族谓词——authentication.py 全等 + authN 目录三委托端点。
fn pred_xhr_authentication(clean: &str, _req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
    clean == "xhr/resources/authentication.py"
        || (clean.starts_with("xhr/resources/auth")
            && (clean.ends_with("/auth.py")
                || clean.ends_with("/corsenabled.py")
                || clean.ends_with("/corsenabled-no-authorize.py")))
}

/// status.py 谓词——xhr/ 前缀或 fetch/api/resources/status.py 全等。
fn pred_status_py(clean: &str, _req: &zero_engine::fetch_bridge::FetchRequest) -> bool {
    clean.ends_with("/status.py") && (clean.starts_with("xhr/") || clean == "fetch/api/resources/status.py")
}

/// fetch/api/resources/ redirect.py 臂——注册表切片四迁入，体逐字节保持。
fn route_fetch_redirect(
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
fn route_fetch_redirect_empty_location(
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
fn route_fetch_method(
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
/// xhr authentication.py + authN 三委托端点（Custom 谓词） 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_authentication(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr image.gif 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_image_gif(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(body),
    }))
}
/// echo-headers.py（Suffix 谓词引入） 臂——注册表切片四迁入，体逐字节保持。
fn route_echo_headers(
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
fn route_delay(
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
/// xhr trickle.py 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_trickle(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr top.txt 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_top_txt(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "top\n".to_string(),
        body_bytes: Some(b"top\n".to_vec()),
    }))
}
/// xhr access-control-basic-put-allow 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_basic_put_allow(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: format!("Wrong method: {}", req.method),
        body_bytes: Some(format!("Wrong method: {}", req.method).into_bytes()),
    }))
}
/// xhr reset-token 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_reset_token(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "PASS".to_string(),
        body_bytes: Some(b"PASS".to_vec()),
    }))
}
/// xhr preflight-cache 三联（DirSuffixes） 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_preflight_cache(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr preflight-request-*-returns-* 对（DirSuffixes） 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_preflight_returns(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 400,
        status_text: wpt_status_text(400).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}
/// xhr headers.py 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_headers(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "TEST".to_string(),
        body_bytes: Some(b"TEST".to_vec()),
    }))
}
/// xhr chunked.py 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_chunked(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }))
}
/// xhr parse-headers.py 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_parse_headers(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}
/// xhr cors-safelisted-response-headers 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_safelisted_response_headers(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.to_string(),
        body_bytes: Some(body.as_bytes().to_vec()),
    }))
}
/// xhr cors-safelisted-request-headers 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_safelisted_request_headers(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr no-custom-header-on-preflight 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_no_custom_header_on_preflight(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr preflight-request-header-lowercase 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_preflight_header_lowercase(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr preflight-request-header-sorted 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_preflight_header_sorted(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr preflight-request-headers-origin 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_preflight_headers_origin(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr preflight-request-invalid-status 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_preflight_invalid_status(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status,
        status_text: wpt_status_text(status).to_string(),
        headers,
        body: String::new(),
        body_bytes: Some(Vec::new()),
    }))
}
/// xhr echo-content-cors 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_echo_content_cors(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// fetch/api/resources/authentication.py 臂——注册表切片四迁入，体逐字节保持。
fn route_fetch_authentication(
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
fn route_dump_authorization_header(
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
/// xhr corsenabled 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_corsenabled(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "Test".to_string(),
        body_bytes: Some(b"Test".to_vec()),
    }))
}
/// xhr redirect.py 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_redirect(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: code,
        status_text: wpt_status_text(code).to_string(),
        headers,
        body: "TEST".to_string(),
        body_bytes: Some(b"TEST".to_vec()),
    }))
}
/// xhr well-formed.xml 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_well_formed_xml(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// xhr pass.txt 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_pass_txt(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: "PASS\n".to_string(),
        body_bytes: Some(b"PASS\n".to_vec()),
    }))
}
/// xhr content.py 臂——注册表切片四迁入，体逐字节保持。
fn route_xhr_content(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers,
        body: body.clone(),
        body_bytes: Some(body.into_bytes()),
    }))
}
/// status.py（xhr/ 前缀 + fetch/api 精确——Custom 谓词） 臂——注册表切片四迁入，体逐字节保持。
fn route_status_py(
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
    Some(Ok(zero_engine::fetch_bridge::FetchResponse {
        status: code,
        status_text: text,
        headers,
        body: String::from_utf8_lossy(&content_bytes).into_owned(),
        body_bytes: Some(content_bytes),
    }))
}
/// eventsource message.py 臂——注册表切片四迁入，体逐字节保持。
fn route_es_message(
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
fn route_es_message2(
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
fn route_es_last_event_id(
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
fn route_es_event_stream(
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
fn route_es_reconnect_fail(
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
fn route_es_status_reconnect(
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
fn route_es_last_event_id2(
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
fn route_es_cors_cookie(
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

/// fetch handler 分派入口——原闭包体逐字节迁出（testharness.rs 原位只剩
/// ctx 构建 + 本调用）；臂序/行为不变，后续批次在本文件内逐族转注册项。
pub(crate) fn dispatch(
    ctx: &PyEndpointCtx,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Result<zero_engine::fetch_bridge::FetchResponse, String> {
    let root = &ctx.root;
    let get_set_cookie_stash = &ctx.get_set_cookie_stash;
    let es_cookie_stash = &ctx.es_cookie_stash;
    let vary_value_override = &ctx.vary_value_override;
    let path_part = wpt_url_path(&req.url);
    let path_part = path_part.strip_prefix('/').unwrap_or(path_part);
    // net-api M4-S23：路径点段归一（上游 wptserve/URL 语义——`/a/../b` 即 `/b`。
    // cors 域以 `dirname(page)/../resources/x.py` 形态构造端点 URL——不归一则
    // fixture 分支失配（clean 带 `cors/..`），静态读库才由文件系统隐式归一）。
    let (raw_path, path_query) = path_part.split_once('?').unwrap_or((path_part, ""));
    let mut norm_segments: Vec<&str> = Vec::new();
    for segment in raw_path.split(['?', '#']).next().unwrap_or(raw_path).split('/') {
        match segment {
            ".." => {
                norm_segments.pop();
            }
            "." => {}
            s => norm_segments.push(s),
        }
    }
    let path_part = if path_query.is_empty() {
        norm_segments.join("/")
    } else {
        format!("{}?{}", norm_segments.join("/"), path_query)
    };
    let clean = path_part.split(['?', '#']).next().unwrap_or(&path_part);
    // M3 切片二：保序注册表分发（py 普查档 §三蓝图）——注册表承载已迁移
    // 臂（表序 = 既有臂序，先命中先返回）；其后为未迁移残链（既有 if
    // 臂序不变），后续批次按臂序前缀推进直至 84 臂全量迁入。handler
    // 返 None = 落穿下一臂。
    for route in py_routes() {
        if !route.matches.test(clean, req) {
            continue;
        }
        if let Some(result) = (route.handle)(ctx, clean, &path_part, req) {
            return result;
        }
    }
    if clean == "xhr/resources/get-set-cookie.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        });
    }
    if clean.starts_with("xhr/") && clean.ends_with(".asis") {
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
                return Err(format!("fixture not embedded: {}", clean));
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status,
            status_text,
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        });
    }
    if clean.starts_with("eventsource/") && clean.ends_with("/cors.py") {
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
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: String::new(),
                body_bytes: Some(Vec::new()),
            });
        }
        if run == "redirect" {
            // 委派 common/redirect.py：302 + Location ← ?location（缺省 /）。
            // ACAO/ACAC 已先行设置——跨源 redirect 响应自身须过 CORS（fetch spec
            // §3.2 redirect 步）。
            let location = wpt_query_value(query, "location").unwrap_or_else(|| "/".into());
            headers.push(("location".into(), location));
            wpt_add_fetch_metadata(&mut headers, req, 302);
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 302,
                status_text: "Redirect".to_string(),
                headers,
                body: String::new(),
                body_bytes: Some(Vec::new()),
            });
        }
        if run == "message" {
            // 委派 message.py 缺省参数形态：`data: data` + 尾 \n\n。
            headers.push(("content-type".into(), "text/event-stream".into()));
            let body = "data: data\n\n\n".to_string();
            wpt_add_fetch_metadata(&mut headers, req, 200);
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: body.clone(),
                body_bytes: Some(body.into_bytes()),
            });
        }
        if run == "status-reconnect" {
            let (status, status_text, body) = es_status_reconnect_response(es_cookie_stash, query);
            headers.push(("content-type".into(), "text/event-stream".into()));
            wpt_add_fetch_metadata(&mut headers, req, status);
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status,
                status_text,
                headers,
                body: body.clone(),
                body_bytes: Some(body.into_bytes()),
            });
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        });
    }
    if clean == "fetch/api/resources/clean-stash.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.to_string(),
            body_bytes: Some(body.as_bytes().to_vec()),
        });
    }
    if clean == "fetch/api/resources/preflight.py" {
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
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: body.to_string(),
                body_bytes: Some(body.as_bytes().to_vec()),
            });
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
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 400,
                    status_text: wpt_status_text(400).to_string(),
                    headers,
                    body: "ERROR: No access-control-request-method in preflight!".to_string(),
                    body_bytes: Some(b"ERROR: No access-control-request-method in preflight!".to_vec()),
                });
            }
            if req_header("accept").map(|v| v.trim() != "*/*").unwrap_or(true) {
                wpt_add_fetch_metadata(&mut headers, req, 400);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 400,
                    status_text: wpt_status_text(400).to_string(),
                    headers,
                    body: "ERROR: Invalid access in preflight!".to_string(),
                    body_bytes: Some(b"ERROR: Invalid access in preflight!".to_vec()),
                });
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
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status,
                status_text: wpt_status_text(status).to_string(),
                headers,
                body: String::new(),
                body_bytes: Some(Vec::new()),
            });
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
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status: 400,
                    status_text: wpt_status_text(400).to_string(),
                    headers,
                    body: "ERROR: No user-agent header in preflight".to_string(),
                    body_bytes: Some(b"ERROR: No user-agent header in preflight".to_vec()),
                });
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        });
    }
    // net-api M4-S18：非 GET 放行至各 fixture 自行分派（corsenabled/redirect-cors
    // 的 OPTIONS preflight 分支）后兜底拒绝——preflight OPTIONS 不再被 blanket 拒。
    // net-api M2-S5：inspect-headers.py 上游 wptserve handler 方法无关（任意
    // 方法回显请求头）——header-values-normalize XHR POST 腿经此回显；其余
    // 端点维持兜底拒绝。
    if req.method != "GET" && req.method != "OPTIONS" && !clean.ends_with("/inspect-headers.py") {
        return Err(format!("method not supported: {}", req.method));
    }
    if req.method == "OPTIONS" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        });
    }
    if clean.ends_with("/inspect-headers.py") {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        });
    }
    if clean == "fetch/api/resources/trickle.py" {
        let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
        let count = wpt_query_value(query, "count")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1);
        let count = count.min(128);
        let body = "TEST_TRICKLE\n".repeat(count);
        let mut headers = wpt_pipe_headers(query);
        headers.push(("content-type".into(), "text/plain".into()));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        });
    }
    // net-api M5-S9：fetch/api/resources/bad-chunk-encoding.py（上游 wptserve
    // 流式端点——count×合法 chunk 后写裸 "garbage"，客户端 chunk 解析失败 →
    // 中段网络错误）。合成通道无 chunk 帧语义：body = count×TEST_CHUNK +
    // "garbage"，`x-zero-body-error-after` 内部标记头承载「N 字节后流错误」
    // 语义（shim 侧消费后从可见 header list 剥离；error-after-response
    // 两腿 = 首 read 得 TEST_CHUNK、次 read/closed 拒 TypeError）。
    if clean.ends_with("/bad-chunk-encoding.py") {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body,
            body_bytes: Some(body_bytes),
        });
    }
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
    if clean == "common/redirect.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 302,
            status_text: "Found".to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        });
    }
    if clean == "dom/nodes/encoding.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        });
    }
    if clean == "service-workers/cache-storage/resources/vary.py" {
        let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
        let mut headers = wpt_pipe_headers(query);
        // https://github.com/web-platform-tests/wpt/blob/24197a11e8c5bd29a5cb7bdf18135a82be8a8546/service-workers/cache-storage/resources/vary.py
        if query.split('&').any(|pair| pair == "clear-vary-value-override-cookie") {
            *vary_value_override.lock().unwrap() = None;
            wpt_add_fetch_metadata(&mut headers, req, 200);
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: "vary cookie cleared".to_string(),
                body_bytes: Some(b"vary cookie cleared".to_vec()),
            });
        }
        if let Some(vary) = wpt_query_value(query, "set-vary-value-override-cookie") {
            *vary_value_override.lock().unwrap() = Some(vary);
            wpt_add_fetch_metadata(&mut headers, req, 200);
            return Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: "vary cookie set".to_string(),
                body_bytes: Some(b"vary cookie set".to_vec()),
            });
        }
        let omits_credentials = req.credentials.as_deref() == Some("omit");
        let override_vary = (!omits_credentials)
            .then(|| vary_value_override.lock().unwrap().clone())
            .flatten();
        if let Some(vary) = override_vary.or_else(|| wpt_query_value(query, "vary")) {
            headers.push(("vary".into(), vary));
        }
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "vary response".to_string(),
            body_bytes: Some(b"vary response".to_vec()),
        });
    }
    if clean == "service-workers/cache-storage/resources/fetch-status.py" {
        let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
        let status = wpt_query_value(query, "status")
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(200);
        let mut headers = wpt_pipe_headers(query);
        wpt_add_fetch_metadata(&mut headers, req, status);
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status,
            status_text: wpt_status_text(status).to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        });
    }
    if clean == "service-workers/service-worker/resources/cors-approved.txt" {
        let mut headers = vec![
            ("Access-Control-Allow-Origin".into(), "*".into()),
            ("content-type".into(), "text/plain".into()),
        ];
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "plaintext\n".to_string(),
            body_bytes: Some(b"plaintext\n".to_vec()),
        });
    }
    if clean == "service-workers/service-worker/resources/cors-denied.txt" {
        let mut headers = vec![("content-type".into(), "text/plain".into())];
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "Cross-origin request blocked by missing CORS response header\n".to_string(),
            body_bytes: Some(b"Cross-origin request blocked by missing CORS response header\n".to_vec()),
        });
    }
    if clean == "service-workers/cache-storage/resources/redirect.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status,
            status_text: wpt_status_text(status).to_string(),
            headers,
            body: String::new(),
            body_bytes: Some(Vec::new()),
        });
    }
    if clean == "common/blank.html" {
        // net-api M4-S22 fixture：common/blank.html（上游为最小空页；wpt-data 未拉
        // 静态本体——redirect 链暴露头面（expose-headers-on-redirect）与 ?pipe=
        // 管道头载体经此处等价供给）。
        let query = path_part.split_once('?').map(|(_, q)| q).unwrap_or("");
        let mut headers = wpt_pipe_headers(query);
        headers.push(("content-type".into(), "text/html".into()));
        wpt_add_fetch_metadata(&mut headers, req, 200);
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: "<!doctype html>\n<meta charset=\"utf-8\">\n<title>Blank</title>\n".to_string(),
            body_bytes: Some(b"<!doctype html>\n<meta charset=\"utf-8\">\n<title>Blank</title>\n".to_vec()),
        });
    }
    if clean == "websockets/security/check.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: body.to_string(),
            body_bytes: Some(body.as_bytes().to_vec()),
        });
    }
    if clean == "html/syntax/xmldecl/support/trickle.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: String::from_utf8_lossy(body).into_owned(),
            body_bytes: Some(body.to_vec()),
        });
    }
    if clean == "html/semantics/scripting-1/the-script-element/resources/load-error-events.py" {
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
        return Ok(zero_engine::fetch_bridge::FetchResponse {
            status,
            status_text: wpt_status_text(status).to_string(),
            headers,
            body: body.clone(),
            body_bytes: Some(body.into_bytes()),
        });
    }
    match std::fs::read(root.join(clean)) {
        Ok(bytes) => {
            let query = path_part.split_once('?').map(|(_, query)| query).unwrap_or("");
            let mut headers = wpt_pipe_headers(query);
            let default_headers = wpt_static_resource_headers(clean);
            // 同名头去重保首项（pipe 优先）：pipe 头与默认 MIME 并存时页面
            // Headers.get 按 HTTP 组合语义逗号拼接（`a, a`）——wasm-compat M3
            // streaming 面 contenttype 整值判定探针实证误挂；与 CT 解析侧
            // 「首项胜出」同向。
            for (name, value) in default_headers {
                if !headers.iter().any(|(n, _)| n.eq_ignore_ascii_case(&name)) {
                    headers.push((name, value));
                }
            }
            // wptserve `.headers` sidecar 语义（net-api-compat-v3 M3 升全量头）：
            // content-type 覆盖默认 MIME（wpt_static_resource_headers）——替换而非
            // 追加（重复 content-type 头在 CT 解析侧首项胜出，保留单头不变量）；
            // 其余头（cors-top.txt.headers ACAO `*` 等）absent 时追加——同名去重
            // 保首项（pipe 头优先，与上方 default_headers 并存序同向）。
            for (sidecar_name, sidecar_value) in wpt_sidecar_headers(root, clean) {
                if sidecar_name.eq_ignore_ascii_case("content-type") {
                    headers.retain(|(n, _)| !n.eq_ignore_ascii_case("content-type"));
                    headers.push((sidecar_name, sidecar_value));
                } else if !headers.iter().any(|(n, _)| n.eq_ignore_ascii_case(&sidecar_name)) {
                    headers.push((sidecar_name, sidecar_value));
                }
            }
            if clean == "service-workers/cache-storage/resources/simple.txt"
                && wpt_query_value(query, "zw-filtered").as_deref() == Some("cors")
            {
                headers.push(("Access-Control-Allow-Origin".into(), "*".into()));
            }
            let status = wpt_pipe_status(query).unwrap_or(200);
            wpt_add_fetch_metadata(&mut headers, req, status);
            // net-api M5-S1：`.sub.js` 服务端替换（上游 wptserve 对 .sub 文件做
            // 全集替换的语义；script fetcher 通道 M4-S22 已对齐——websockets/
            // *.html 案经 fetch 通道读 constants.sub.js，缺此则 SCHEME_DOMAIN_PORT
            // 带 '{{ports[ws][0]}}' 字面量 → Invalid URL）。
            let bytes = if clean == "resources/testharness.js" {
                // html-semantics M2 s98：write 注入 harness 的完成探测接线——
                // document.write 写入的 `<script src="/resources/testharness.js">`
                // （execution-timing/102 全家桶形）经本通道取回，此前拿到 RAW 副本：
                // 无 `__zw_mark_harness_loaded`/`__zw_harness_state` 完成探测钩子，
                // 且新文档无生命周期脚本补调（document.open 后 load 事件不重发）→
                // test_environment.all_loaded 恒 false → 末测 done 后不 complete →
                // runner 探针等满超时（hook=undefined/state=null 伪 Timeout）。
                // 此处服务与静态内联路径同款补丁源（apply_harness_completion_patch）
                // 并自带 eval 尾自调——任一装载路径拿到同一接线面。
                let patched = apply_harness_completion_patch(String::from_utf8_lossy(&bytes).into_owned());
                let mut with_tail = patched;
                with_tail.push_str(HARNESS_LOADED_TAIL_CALL);
                with_tail.into_bytes()
            } else if clean == "resources/testharnessreport.js" {
                // s98 同面：上游 testharnessreport.js 是裸 loader（无 runner 装配），
                // write 注入副本经其注册的 completion 回调到不了探针全局。服务
                // reporter 快照（setup({output:false}) + GLOBAL shim +
                // __zw_harness_results/__zw_harness_complete 装配），与静态路径
                // prepare_harness_html 移除 report 标签内联 reporter 等价。
                HARNESS_REPORTER_SOURCE.as_bytes().to_vec()
            } else if clean.ends_with(".sub.js") {
                apply_wpt_substitutions(&String::from_utf8_lossy(&bytes)).into_bytes()
            } else {
                bytes
            };
            // R3254-E2 切片 6（editing goal，2026-09-07）：**iframe 文档的相对
            // src 脚本内联**——shim iframe 装载（R115 runInlineScripts）只执行
            // inline `<script>`，外部 src 脚本被跳过（无同步 fetch 通道），iframe
            // 内 `var`/函数面缺失（selection/test-iframe.html 依赖 common.js 的
            // setupRangeTests——deleteFromDocument/getSelection iframe 簇根因）。
            // 主文档面 runner 已有 inline_local_scripts 预处理；此处对 fetch 通道
            // 返回的 .html 同样内联相对 src（绝对/`/resources/` 保留原标签）。
            // html-syntax（2026-10-20）：非 UTF-8 文档字节保真——XHR document
            // 响应的 charset 嗅探/解码消费原始字节（html/syntax/charset 22 案
            // 断言 windows-1251 等遗留编码字节面）；from_utf8_lossy 把裸字节
            // 改写为 EF BF BD，信息先于 JS 侧 TextDecoder 损失。合法 UTF-8 时
            // 维持内联路径（往返恒等）；非 UTF-8 时跳过内联、原字节直供（上游
            // charset 语料为纯标记、无相对 src 脚本；FIXME: 非 UTF-8 文档的
            // 相对脚本内联待字节级 tokenizer）。
            let body = if clean.ends_with(".html") {
                match std::str::from_utf8(&bytes) {
                    Ok(html) => {
                        // encoding-legacy-mb DC-2（2026-10-28）：合法 UTF-8 字节 +
                        // 非 UTF-8 声明 → serve 侧按声明解码。iso-2022-jp 语料
                        // 7-bit clean（字节合法 UTF-8 ≠ UTF-8 内容），plain wire
                        // 侧 shim 嗅探只落 characterSet 元数据、正文不重解码
                        //（spec encoding sniffing 的 change-encoding-and-restart
                        // 步缺失）→ 文档族字面转义全败。解码走文档管道
                        //（decode_document_bytes：BOM > meta 预扫描 > UTF-8，
                        // DC-1 read_corpus_to_string 同语义）。例外：replacement
                        // 编码（iso-2022-kr bogo 声明，xmldecl replacement.htm
                        // 支持页）不解——replacement 解码恒 U+FFFD 毁页，维持
                        // 字面 + shim characterSet 元数据面（2026-10-20
                        // html-syntax 既定行为）。UTF-8/无声明语料恒等零变化；
                        // 非 UTF-8 字节走 Err 分支（`__zw_bytes:` wire 字节保真，
                        // XHR document/iframe bytes 面零变化）。
                        // https://html.spec.whatwg.org/multipage/parsing.html#encoding-sniffing-algorithm
                        let html = match sniff_document_encoding(html) {
                            Some(name) if name != "UTF-8" && name != "replacement" => {
                                zero_dom::decode_document_bytes(&bytes)
                            }
                            _ => html.to_string(),
                        };
                        // navigation-compat M3-S4（wptserve 模板面）：fetch 通道
                        // .html 按该请求查询串做 `{{GET[name]}}` 替换——上游
                        // wptserve 对每请求资源注入（replace-before-load 族
                        // code-injector.html?code=<enc js> 载体；主文档通道
                        // run_testharness_html_inner 已替换，iframe 子文档此前拿
                        // 字面量 → 注入脚本语法错误，自导航从未发生）。替换先于
                        // 脚本内联（注入代码须进提取出的 script 源）。
                        let substituted = wpt_apply_get_query_substitutions(html, query);
                        let inlined = inline_local_scripts(
                            &substituted,
                            root,
                            clean,
                            sniff_document_encoding(&substituted).as_deref(),
                        );
                        let inlined_bytes = inlined.clone().into_bytes();
                        (inlined_bytes, inlined)
                    }
                    Err(_) => (bytes.clone(), String::from_utf8_lossy(&bytes).into_owned()),
                }
            } else {
                (bytes.clone(), String::from_utf8_lossy(&bytes).into_owned())
            };
            Ok(zero_engine::fetch_bridge::FetchResponse {
                status,
                status_text: wpt_status_text(status).to_string(),
                headers,
                body: body.1,
                body_bytes: Some(body.0),
            })
        }
        Err(_e) => {
            if clean.starts_with("service-workers/service-worker/resources/") && clean.ends_with(".html") {
                let status = 404;
                let mut headers = vec![("content-type".into(), "text/html".into())];
                wpt_add_fetch_metadata(&mut headers, req, status);
                return Ok(zero_engine::fetch_bridge::FetchResponse {
                    status,
                    status_text: wpt_status_text(status).to_string(),
                    headers,
                    body: String::new(),
                    body_bytes: Some(Vec::new()),
                });
            }
            // net-api M4-S20：文件缺失 → 404 响应（HTTP 语义——缺文件非网络错误；
            // responseurl.html「404 response has proper responseURL」面）。
            let mut headers404: Vec<(String, String)> = Vec::new();
            headers404.push(("content-type".into(), "text/plain".into()));
            wpt_add_fetch_metadata(&mut headers404, req, 404);
            Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 404,
                status_text: "Not Found".to_string(),
                headers: headers404,
                body: String::new(),
                body_bytes: Some(Vec::new()),
            })
        }
    }
}
