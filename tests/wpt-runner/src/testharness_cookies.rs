//! Cookie 兼容 goal（cookies-compat M1 / DC-1）—— document.cookie 读写/属性/可见性
//! 语义上游 testharness window 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_webmessaging
//! 先例），经 `pub(crate)` 复用其执行底座（`run_testharness_html_inner` /
//! `apply_wpt_substitutions`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/18-cookies-compat.sh` 按需拉到 `wpt-data/cookies/`
//! （gitignored，WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。运行面 = 目录
//! 扫描（web-components/webmessaging 先例，skip 规则两侧同源防漂移）+
//! [`cookies_case_skipped`] 内容规则筛减。skip 域（goal Mission 排除面 + 支撑页）
//! 见该函数文档与 fetch 脚本头注释。
//!
//! 语料形态注记（2026-10-12 M1 盘点）：cookies corpus 主体按「cookie 由谁写入」分层
//! —— **HTTP 头驱动**（`httpCookieTest` → cookie.py 服务端 Set-Cookie；`*.headers`
//! 服务端预置；set.py/list.py 传输验证）在静态语料 runner 上**不可执行**（无服务端
//! 注入通道，goal 契约「静态语料口径诚实」记分母说明）；**DOM 驱动**
//! （`domCookieTest` → document.cookie 读写）为可执行验收面。混合案（expires /
//! name-ctl / value-ctl / prefix 系）两类并存：DOM 腿运行、HTTP 腿经 [`helper 内联
//! 覆盖`](HEADER_DRIVEN_OVERRIDE) 就地转标记性 Fail（`zw-static-corpus:` 前缀
//! message），基线按 message 前缀分类记账，不静默丢弃。
//!
//! 运行环境前提差异：runner 恒 `https://wpt.test`（`isSecureContext === true`）——
//! 上游非安全上下文前提案（文件含 "Non-secure" 期望的 `.html` 非 https 变体）前提
//! 不可达，整案 skip（规则 [`cookies_case_skipped`] 显式清单）；`.https.` 变体前提
//! 满足，照常运行。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, apply_wpt_substitutions, corpus_case_timeout, run_testharness_html_inner,
};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// cookies corpus 扫描子目录（goal Mission 排除面整目录不扫：`samesite/`、
/// `samesite-none-secure/`、`schemeful-same-site/`、`partitioned-cookies/`、
/// `third-party-cookies/`（SameSite/CHIPS/3PC——传输策略域）；`domain/`、
/// `origin-bound-cookies/`（服务端 .headers 预置 + set.py/list.py 传输验证——
/// jar 本体既有域）。helper 目录 `resources/` 不在扫描面（经 inline extras 内联）。
pub const COOKIES_TEST_SUBDIRS: &[&str] = &[
    "cookies",
    "cookies/attributes",
    "cookies/encoding",
    "cookies/name",
    "cookies/ordering",
    "cookies/path",
    "cookies/prefix",
    "cookies/secure",
    "cookies/size",
    "cookies/value",
];

/// corpus helper 脚本路径（inline extras 内联——外部 src 脚本提取器不加载）。
const HELPER_COOKIE_TEST: &str = "/cookies/resources/cookie-test.js";
const HELPER_COOKIE_HELPER: &str = "/cookies/resources/cookie-helper.sub.js";
const HELPER_TESTHARNESS_HELPERS: &str = "/cookies/resources/testharness-helpers.js";

/// HTTP 头驱动测试形态的就地覆盖（内联在 helper 体尾部）。静态语料无服务端
/// Set-Cookie 通道（cookie.py/set.py 均不落地），对应子测试转**标记性 Fail**
/// （message 以 `zw-static-corpus:` 开头）——基线分类时与可执行面 Fail 区分，
/// 记入不可执行面分母说明（goal 契约「不静默丢弃」）。不用 skip()：pinned
/// testharness.js（5207 行副本）未暴露 skip()。
const HEADER_DRIVEN_OVERRIDE: &str = r#"
;(function() {
  function zwSkipHeaderDriven(name) {
    return promise_test(function() {
      assert_true(false, 'zw-static-corpus: HTTP-header-driven cookie set is not executable in static corpus (no server-side Set-Cookie channel)');
    }, name);
  }
  if (typeof window.httpCookieTest === 'function') {
    window.httpCookieTest = function(cookie, expected, name) { return zwSkipHeaderDriven(name); };
  }
  if (typeof window.httpRedirectCookieTest === 'function') {
    window.httpRedirectCookieTest = function(cookie, expected, name) { return zwSkipHeaderDriven(name); };
  }
  if (typeof window.set_prefixed_cookie_via_http_test === 'function') {
    window.set_prefixed_cookie_via_http_test = function(options) { return zwSkipHeaderDriven(options.title); };
  }
  // 混合案（cookies-compat M2，2026-10-12）：set_prefixed_cookie_via_dom_test =
  // DOM 腿（document.cookie 写入 + DOM 可见性断言——可执行）+ list.py 服务端回执腿
  // （credFetch 404 → Unhandled rejection——静态语料不可执行；且「请求附带/响应入
  // jar」属 goal Mission 排除的传输域）。替换为**仅 DOM 面计分**版本：assert_dom_cookie
  // 语义逐字保留（含 erase 前置/清理），回执腿不执行。回执面不可执行性在
  // evidence / wpt-suites.csv note 记账，不静默。
  if (typeof window.set_prefixed_cookie_via_dom_test === 'function') {
    window.set_prefixed_cookie_via_dom_test = function(options) {
      promise_test(function(t) {
        var name = options.prefix + "prefixtestcookie";
        erase_cookie_from_js(name, options.params);
        t.add_cleanup(function() { erase_cookie_from_js(name, options.params); });
        var value = "foo" + ++window.dom_prefix_counter;
        document.cookie = name + "=" + value + ";" + options.params;
        assert_dom_cookie(name, value, options.shouldExistInDOM);
        // pinned 老版 testharness 的 promise_test 强制返回 thenable（原 helper 的
        // credFetch 链即 thenable；DOM 面同步断言后补 resolved promise）。
        return Promise.resolve();
      }, options.title);
    };
  }
})();
"#;

/// [`COOKIES_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文（内容规则按标记筛减）。
///
/// - 内容标记 `fetch_tests_from_window`（第二 window 测试聚合面——runner 无多
///   window 管道，webmessaging 同门槛）：attributes/domain.sub、
///   attributes/secure-non-secure
/// - 内容标记 `window.open`（第二文档面，同 iframe 门槛）：ordering/ordering.sub
/// - 内容标记 `iframe`（frame 树管道缺失，navigation-compat M3 门槛后随其重入）：
///   cookie-enabled-noncookie-frame、navigated-away、path/default、path/match
/// - 内容标记 `new WebSocket(`/`wss:`（WebSocket 握手 Set-Cookie 面——workers/
///   ws 管道域）：secure/set-from-ws、set-from-wss
/// - 内容标记 `Non-secure`（非安全上下文前提案——runner 恒 https，前提不可达）：
///   prefix 非 .https 变体（document-cookie.non-secure、__host/-secure
///   document-cookie/header 的 `.html` 变体）
/// - 显式清单 `secure/set-from-dom.sub.html`（非安全上下文前提——源无 "Non-secure"
///   字面量，断言面同上：Secure cookie 于非安全来源拒设；.https 姊妹案照常运行）
/// - 显式清单 `secure/set-from-http*`（服务端 .headers 预置 Set-Cookie 前提——
///   静态语料 cookie 未入 jar，连 DOM 断言都无意义）
/// - `.html` 不含 `testharness`（同目录无 harness 支撑页；helper 目录不在扫描面）
pub(crate) fn cookies_case_skipped(relative: &str, source: &str) -> bool {
    if relative.ends_with(".html") && !source.contains("testharness") {
        return true;
    }
    if source.contains("fetch_tests_from_window")
        || source.contains("window.open")
        || source.contains("iframe")
        || source.contains("new WebSocket(")
        || source.contains("wss:")
        || source.contains("Non-secure")
    {
        return true;
    }
    relative.starts_with("cookies/secure/set-from-http") || relative == "cookies/secure/set-from-dom.sub.html"
}

/// Run the imported upstream cookies window subset
/// （cookies-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `prefix/`、
/// `attributes/`——基线按面分类）。
pub fn run_cookies_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    // corpus helper 源（{{host}} 等服务端模板就地替换——上游 wptserve 对 .sub.js 同
    // 语义）；缺失按 Fail 记账不静默（引用它的用例会拿到失败子测试，账面可见）。
    let mut helpers: Vec<(&str, String)> = Vec::new();
    for path in [HELPER_COOKIE_TEST, HELPER_COOKIE_HELPER, HELPER_TESTHARNESS_HELPERS] {
        let source = read_corpus_to_string(wpt_root.join(path.trim_start_matches('/')))
            .map(|source| apply_wpt_substitutions(&source));
        helpers.push((
            path,
            source.unwrap_or_else(|error| format!("/* helper load failed: {error} */")),
        ));
    }
    // HTTP 头驱动覆盖追加到对应 helper 尾部（replace_script_source 原位换标签——
    // 覆盖在页内联脚本之前执行，时序与上游 helper 加载一致）。
    let mut extras: Vec<(&str, String)> = Vec::with_capacity(helpers.len());
    for (path, mut source) in helpers {
        if path == HELPER_COOKIE_TEST || path == HELPER_COOKIE_HELPER {
            source.push_str(HEADER_DRIVEN_OVERRIDE);
        }
        extras.push((path, source));
    }
    let extra_refs: Vec<(&str, &str)> = extras.iter().map(|(path, source)| (*path, source.as_str())).collect();

    let mut cases = Vec::new();
    for subdir in COOKIES_TEST_SUBDIRS {
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            // cookies corpus 全部为 .html 形态（无 .any.js/.window.js 变体）。
            if !file_name.ends_with(".html") {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if cookies_case_skipped(&relative, &source) {
                continue;
            }
            // 用例源同走服务端模板替换（.sub 形态——attributes-ctl/size 案含 {{host}}）。
            let substituted = apply_wpt_substitutions(&source);
            let results = run_testharness_html_inner(
                wpt_root,
                &relative,
                &substituted,
                &harness_source,
                &extra_refs,
                corpus_case_timeout(),
            );
            cases.push((relative, results));
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}
