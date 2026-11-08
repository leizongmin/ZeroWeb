//! py 端点路由底座（46 号 runner-net-infra M3 框架化）——fetch handler 闭包体
//! 自 testharness.rs 整体迁出的底座切片：案级 12 项 stash/状态收敛
//! [`PyEndpointCtx`]，分派入口 [`dispatch`]。臂序（先命中先返回）与行为
//! 逐字节保持（全量 A/B 门禁，见 goal 基线归因档）。注册表化（保序
//! `PyRoute` 表 + 端点注册项）按族分批落账：本切片先承载迁出体，后续
//! 批次把 `if` 臂逐族转注册项并迁四 fetcher 闭包臂（蓝图见 py 普查档 §三）。
//!
//! handler 按端点族分文件：[`site`]（站点/先例）、[`xhr`]、[`fetch`]、[`es`]、
//! [`misc`]——本文件只留底座（类型/表/分派），子模块经 glob use 反供表项。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub(crate) use crate::testharness::{
    CachePyStash, CredentialCookieStash, HARNESS_LOADED_TAIL_CALL, HARNESS_REPORTER_SOURCE, ServeCustomResponseStash,
    WPT_TOKEN_STASH, ZW_ONE_PX_PNG, ZW_S288_404_PNG, ZW_S295_BLUE_10, ZW_S295_GREEN_100,
    apply_harness_completion_patch, apply_wpt_substitutions, es_status_reconnect_response, inline_local_scripts,
    serve_interactions_stylesheet, sniff_document_encoding, wpt_add_fetch_metadata, wpt_apply_get_query_substitutions,
    wpt_cache_py_fixture, wpt_load_error_events_response, wpt_pipe_headers, wpt_pipe_status, wpt_query_value,
    wpt_query_value_bytes, wpt_sidecar_headers, wpt_static_resource_headers, wpt_status_text, wpt_url_path,
};

mod es;
mod fetch;
mod misc;
mod site;
mod xhr;
use es::*;
use fetch::*;
use misc::*;
use site::*;
use xhr::*;

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
        PyRoute {
            matches: PyRouteMatch::Exact("xhr/resources/get-set-cookie.py"),
            handle: route_get_set_cookie,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "xhr/",
                suffix: ".asis",
            },
            handle: route_xhr_asis,
        },
        PyRoute {
            matches: PyRouteMatch::DirSuffix {
                dir: "eventsource/",
                suffix: "/cors.py",
            },
            handle: route_es_cors,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/resources/clean-stash.py"),
            handle: route_clean_stash,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/resources/preflight.py"),
            handle: route_preflight,
        },
        PyRoute {
            matches: PyRouteMatch::Custom(pred_method_gate_non_get),
            handle: route_method_not_supported,
        },
        PyRoute {
            matches: PyRouteMatch::Custom(pred_method_options),
            handle: route_options_blanket,
        },
        PyRoute {
            matches: PyRouteMatch::Suffix("/inspect-headers.py"),
            handle: route_inspect_headers,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("fetch/api/resources/trickle.py"),
            handle: route_fetch_trickle,
        },
        PyRoute {
            matches: PyRouteMatch::Suffix("/bad-chunk-encoding.py"),
            handle: route_bad_chunk_encoding,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("common/redirect.py"),
            handle: route_common_redirect,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("dom/nodes/encoding.py"),
            handle: route_dom_encoding,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("service-workers/cache-storage/resources/vary.py"),
            handle: route_sw_vary,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("service-workers/cache-storage/resources/fetch-status.py"),
            handle: route_sw_fetch_status,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("service-workers/service-worker/resources/cors-approved.txt"),
            handle: route_sw_cors_approved,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("service-workers/service-worker/resources/cors-denied.txt"),
            handle: route_sw_cors_denied,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("service-workers/cache-storage/resources/redirect.py"),
            handle: route_sw_cache_redirect,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("common/blank.html"),
            handle: route_blank_html,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("websockets/security/check.py"),
            handle: route_ws_security_check,
        },
        PyRoute {
            matches: PyRouteMatch::Exact("html/syntax/xmldecl/support/trickle.py"),
            handle: route_xmldecl_trickle,
        },
        PyRoute {
            matches: PyRouteMatch::Exact(
                "html/semantics/scripting-1/the-script-element/resources/load-error-events.py",
            ),
            handle: route_load_error_events,
        },
    ]
}

/// fetch handler 分派入口——原闭包体逐字节迁出（testharness.rs 原位只剩
/// ctx 构建 + 本调用）；臂序/行为不变，后续批次在本文件内逐族转注册项。
pub(crate) fn dispatch(
    ctx: &PyEndpointCtx,
    req: &zero_engine::fetch_bridge::FetchRequest,
) -> Result<zero_engine::fetch_bridge::FetchResponse, String> {
    let root = &ctx.root;
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
