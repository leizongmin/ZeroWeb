//! `register_dom_callbacks` —— 向 V8 sandbox 注册全部 `__zw_*` DOM 桥接回调。从 js_dom_bridge.rs
//! 拆出（R2976，文件大小治理 slice 4）。连接 [`generate_js_dom_shim`] 产生的 JS shim 与宿主侧
//! DomMutation 收集器：JS 侧 `__zw_*` 扁平回调翻译为 DomMutation / 从 dom_html 快照查询。
//! `use super::*` 复用父模块全部类型与 helper（find_by_selector / compute_document_styles /
//! apply_dom_mutations / crypto_* / canvas_context_op / element_matches_test_selector 等，经
//! pub use 重导出 + 祖先私有项可见）。pub register_dom_callbacks 经 `pub use callbacks::*` 重导出。

use super::*;

/// 向 V8 sandbox 注册全部 `__zw_*` DOM 桥接回调。
///
/// 将 [`generate_js_dom_shim`] 产生的 JS shim 与宿主侧 [`DomMutation`] 收集器连接：
/// JS 侧 `document.querySelector`/`setAttribute`/`createElement` 等操作经
/// `__zw_*` 扁平回调翻译为 `DomMutation`，推入共享 `mutations` 向量；查询类回调
/// （`__zw_get_attr`/`__zw_get_text`/`__zw_query_*`）则从 `dom_html` 快照读取。
///
/// `dom_html` / `page_url` 用 `Arc<Mutex<String>>` 共享，使宿主能在脚本执行前
/// 经 [`V8Sandbox::execute`] 切换快照（与 browser/renderer/reftest 三处共用一致语义）。
///
/// 该函数从 renderer/browser 两个 JS worker 中抽取为共享实现，避免第三份拷贝
/// （reftest harness 也复用，见 `tests/wpt-runner`）。
pub fn register_dom_callbacks(
    sandbox: &mut dyn Sandbox,
    mutations: &Arc<std::sync::Mutex<Vec<DomMutation>>>,
    dom_html: &Arc<std::sync::Mutex<String>>,
    page_url: &Arc<std::sync::Mutex<String>>,
    canvas_registry: &Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>>,
) {
    let counter = Arc::new(AtomicU64::new(0));

    let url = Arc::clone(page_url);
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(move |_args| url.lock().unwrap_or_else(|e| e.into_inner()).clone()),
    );

    // `performance.now()`——DOMHighResTimeStamp（ms，单调时钟，自 time origin 起，子毫秒精度）。
    // analytics / 动画计时 / rAF timestamp 高频查询。time origin = 回调注册时刻（页面/脚本启动近似），
    // 回调返 elapsed ms（f64 串）。Instant 单调且 Send+Sync，闭包仅借 &origin 故为 Fn。
    let perf_origin = std::time::Instant::now();
    sandbox.register_callback(
        "__zw_performance_now",
        Box::new(move |_args| format!("{}", perf_origin.elapsed().as_secs_f64() * 1000.0)),
    );

    // `console.*` 桥接（R3256，Console Standard）——page console.log/info/warn/error/debug/trace/dir/dirxml/table
    // 经 shim `_zwConsoleEmit` 序列化 args 后调本回调，转发到宿主 `tracing` 日志（便于排障 + WPT console 断言
    // 可见）。level→tracing 宏映射：error→error / warn→warn / (info,log,table)→info / 其余→debug。返空串
    //（shim 不读返值）。失败不 panic（best-effort，console 不应阻断页面）。
    sandbox.register_callback(
        "__zw_console_log",
        Box::new(|args: &[String]| -> String {
            let level = args.first().map(String::as_str).unwrap_or("log");
            let msg = args.get(1).map(String::as_str).unwrap_or("");
            match level {
                "error" => tracing::error!("[console] {msg}"),
                "warn" => tracing::warn!("[console] {msg}"),
                "info" | "log" | "table" => tracing::info!("[console] {msg}"),
                _ => tracing::debug!("[console.{level}] {msg}"),
            }
            String::new()
        }),
    );

    // `new URL(url, base)`——WHATWG URL 解析（protocol/host/hostname/port/pathname/search/hash/origin/
    // href）。location.href 操纵 / fetch 相对 URL / 链接解析高频。委托 [`parse_url_to_json`]（spec-correct
    // via `url` crate）；解析失败返空串（shim 抛 TypeError，spec 一致）。
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            parse_url_to_json(input, base)
        }),
    );

    // URL 属性 setter——组件可写（protocol/host/hostname/port/pathname/search/hash/username/password/
    // href）。委托 [`set_url_part`]（spec-correct via `url` crate 的 Url setters）；失败返空串（shim 抛
    // TypeError，spec）。`__zw_parse_url` 已注册时本回调方有意义（shim URL setter 依赖两者）。
    sandbox.register_callback(
        "__zw_set_url_part",
        Box::new(|args: &[String]| -> String {
            let prev = args.first().map(String::as_str).unwrap_or("");
            let part = args.get(1).map(String::as_str).unwrap_or("");
            let value = args.get(2).map(String::as_str).unwrap_or("");
            set_url_part(prev, part, value)
        }),
    );

    // `window.matchMedia(query)`——响应式设计 / viewport 查询高频。委托 [`match_media_to_json`]（spec-correct
    // via `zero_css_parser::media_query`，含 min/max-width/height、orientation、prefers-color-scheme）。
    // JS 侧传 query + viewport 宽高（innerWidth/innerHeight）；返 `{"matches","media"}` JSON。
    sandbox.register_callback(
        "__zw_match_media",
        Box::new(|args: &[String]| -> String {
            let query = args.first().map(String::as_str).unwrap_or("");
            let width = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let height = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            match_media_to_json(query, width, height)
        }),
    );

    // `CSS.supports(prop, val?)`——CSS 特性检测（modern progressive enhancement 高频）。委托 [`css_supports`]
    //（known-property gate + apply_property_value_with_quirks；两参声明 / 单参条件 not/括号/声明）。
    // 返 "1"/"0"（shim 转 bool）。
    sandbox.register_callback(
        "__zw_css_supports",
        Box::new(|args: &[String]| -> String {
            let prop = args.first().map(String::as_str).unwrap_or("");
            let value = args.get(1).map(String::as_str);
            if css_supports(prop, value) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_match",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_match_selector_doc(doc, &sel))
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_all",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_all_selector_list_doc(doc, &sel))
        }),
    );

    // `DOMParser.parseFromString(str, type)`（R2790）——解析**任意 HTML 串**为只读 Document。
    // 与 `__zw_query_*`（基于 dom_html 快照）不同：html 从 arg[0] 取（DOMParser 解析的是传入串，
    // 非当前页面快照），selector 从 arg[1]，all 标志从 arg[2]（"1"=全部）。返 JSON 元素快照数组。
    // shim 包成 `_zwParsedDoc` + 只读 element-proxy（querySelector/getElementById/body/textContent/...）。
    sandbox.register_callback(
        "__zw_parse_html_query",
        Box::new(|args: &[String]| -> String {
            let html = args.first().map(String::as_str).unwrap_or("");
            let sel = args.get(1).map(String::as_str).unwrap_or("");
            let all = args.get(2).map(|s| s == "1").unwrap_or(false);
            parse_html_element_json(html, sel, all)
        }),
    );

    // `document.implementation.createHTMLDocument().body.childNodes`（R3016）——DOMPurify.sanitize 递归 walk
    // 的核心阻塞。与 `__zw_parse_html_query` 对称：html 从 arg[0]（detached 串，非 dom_html 快照），
    // elem_sel 从 arg[1]。返 child_nodes_json（element→{k:E,s:selector} / text→{k:T,v} / comment→{k:C,v}）。
    sandbox.register_callback(
        "__zw_parse_html_child_nodes",
        Box::new(|args: &[String]| -> String {
            let html = args.first().map(String::as_str).unwrap_or("");
            let sel = args.get(1).map(String::as_str).unwrap_or("");
            child_nodes_json(html, sel)
        }),
    );

    // `crypto.subtle.digest(algo, data)`（R2793）——SHA-1/256/384/512 哈希。algo 从 arg[0]（串），
    // 字节从 arg[1]（逗号分隔十进制串）。返逗号分隔十进制 hash 串（unsupported → 空，shim reject）。
    sandbox.register_callback(
        "__zw_crypto_subtle_digest",
        Box::new(|args: &[String]| -> String {
            let algo = args.first().map(String::as_str).unwrap_or("");
            let bytes = args.get(1).map(String::as_str).unwrap_or("");
            crypto_subtle_digest(algo, bytes)
        }),
    );

    // `crypto.getRandomValues` / `randomUUID` OS 随机源（R2960）——arg[0]=字节数。返逗号分隔十进制随机字节串。
    sandbox.register_callback(
        "__zw_crypto_get_random_values",
        Box::new(|args: &[String]| -> String {
            let n: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            crypto_random_bytes(n)
        }),
    );

    // `new CompressionStream(format)`（R2986）——gzip/deflate/deflate-raw 压缩。
    // arg[0]=format，arg[1]=输入字节 csv。返压缩字节 csv（unsupported → 空串，shim reject）。
    sandbox.register_callback(
        "__zw_compress",
        Box::new(|args: &[String]| -> String {
            let format = args.first().map(String::as_str).unwrap_or("");
            let data = args.get(1).map(String::as_str).unwrap_or("");
            compress_bytes(format, data)
        }),
    );
    // `new DecompressionStream(format)`（R2986）——gzip/deflate/deflate-raw 解压。
    // arg[0]=format，arg[1]=压缩字节 csv。返解压字节 csv（损坏/unsupported → 空串，shim error）。
    sandbox.register_callback(
        "__zw_decompress",
        Box::new(|args: &[String]| -> String {
            let format = args.first().map(String::as_str).unwrap_or("");
            let data = args.get(1).map(String::as_str).unwrap_or("");
            decompress_bytes(format, data)
        }),
    );

    // `crypto.subtle.sign/verify("HMAC", ...)`（R2955）——HMAC-SHA-1/256/384/512。
    // arg[0]=hash 名（"SHA-256"），arg[1]=key 字节 csv，arg[2]=data 字节 csv。返 MAC csv（unsupported → 空）。
    sandbox.register_callback(
        "__zw_crypto_subtle_hmac",
        Box::new(|args: &[String]| -> String {
            let hash = args.first().map(String::as_str).unwrap_or("");
            let key = args.get(1).map(String::as_str).unwrap_or("");
            let data = args.get(2).map(String::as_str).unwrap_or("");
            crypto_subtle_hmac(hash, key, data)
        }),
    );

    // `crypto.subtle.deriveBits("PBKDF2", ...)`（R2956）——PBKDF2-HMAC-SHA-1/256/384/512。
    // arg[0]=hash 名，arg[1]=password csv，arg[2]=salt csv，arg[3]=iterations，arg[4]=dklen（字节）。返派生密钥 csv。
    sandbox.register_callback(
        "__zw_crypto_subtle_pbkdf2",
        Box::new(|args: &[String]| -> String {
            let hash = args.first().map(String::as_str).unwrap_or("");
            let password = args.get(1).map(String::as_str).unwrap_or("");
            let salt = args.get(2).map(String::as_str).unwrap_or("");
            let iterations: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
            let dklen: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            crypto_subtle_pbkdf2(hash, password, salt, iterations, dklen)
        }),
    );

    // `crypto.subtle.encrypt/decrypt("AES-GCM", ...)`（R2957）——AES-128/256-GCM。
    // arg[0]=mode("encrypt"/"decrypt")，arg[1]=key csv，arg[2]=iv csv，arg[3]=data csv，arg[4]=aad csv。返 csv（error → 空）。
    sandbox.register_callback(
        "__zw_crypto_subtle_aes_gcm",
        Box::new(|args: &[String]| -> String {
            let mode = args.first().map(String::as_str).unwrap_or("");
            let key = args.get(1).map(String::as_str).unwrap_or("");
            let iv = args.get(2).map(String::as_str).unwrap_or("");
            let data = args.get(3).map(String::as_str).unwrap_or("");
            let aad = args.get(4).map(String::as_str).unwrap_or("");
            crypto_subtle_aes_gcm(mode, key, iv, data, aad)
        }),
    );

    // `crypto.subtle.deriveBits("HKDF", ...)`（R2958）——HKDF-SHA-1/256/384/512（RFC 5869）。
    // arg[0]=hash 名，arg[1]=ikm csv，arg[2]=salt csv，arg[3]=info csv，arg[4]=dklen（字节）。返派生密钥 csv。
    sandbox.register_callback(
        "__zw_crypto_subtle_hkdf",
        Box::new(|args: &[String]| -> String {
            let hash = args.first().map(String::as_str).unwrap_or("");
            let ikm = args.get(1).map(String::as_str).unwrap_or("");
            let salt = args.get(2).map(String::as_str).unwrap_or("");
            let info = args.get(3).map(String::as_str).unwrap_or("");
            let dklen: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
            crypto_subtle_hkdf(hash, ikm, salt, info, dklen)
        }),
    );

    // `element.matches(selector)` / `element.closest(selector)`——元素查询 API（直接消费选择器引擎，
    // 含组合器）。elem_sel = 元素唯一选择器（proxy 持有），test_sel = 待测选择器。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_matches",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let test_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if element_matches_test_selector(&snap, &elem_sel, &test_sel) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // `CSSStyleSheet.cssRules` 读（R2808）——解析 `<style>` 元素文本 → StyleRule 序列化为
    // `\x1f`（规则间）/`\x1e`（selectorText·cssText）wire。供 shim document.styleSheets[].cssRules。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_style_rules",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            style_rules_wire(&snap, &sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_closest",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let test_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            closest_matching_selector(&snap, &elem_sel, &test_sel)
        }),
    );

    // `element.querySelector(selector)` / `element.querySelectorAll(selector)`——元素**子树**作用域
    // （spec：仅后代，不含元素自身）。elem_sel = 元素唯一选择器，区别于文档作用域的 query_match/all。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_match_sub",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_match_in_subtree(&snap, &elem_sel, &sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_all_sub",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_all_in_subtree(&snap, &elem_sel, &sel)
        }),
    );

    // Form-associated listed controls，按 form owner 过滤并保持文档序。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_form_controls",
        Box::new(move |args| {
            let form = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            form_control_selectors(&snap, &form).join("|")
        }),
    );

    // 元素遍历/导航 API：children/firstElementChild/lastElementChild/childElementCount（子列表）、
    // previousElementSibling/nextElementSibling（兄弟对）、contains（后代判定）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_element_children",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            element_children_selectors(&snap, &elem_sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_element_siblings",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            element_sibling_selectors(&snap, &elem_sel)
        }),
    );

    // 节点级遍历 API（含文本/注释节点）：childNodes/firstChild/lastChild（子列表）、
    // previousSibling/nextSibling（兄弟对）。JSON 序列化（文本内容含任意字符）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_child_nodes",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            child_nodes_json(&snap, &elem_sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_sibling_nodes",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            sibling_nodes_json(&snap, &elem_sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_contains",
        Box::new(move |args| {
            let container_sel = args.first().map(String::from).unwrap_or_default();
            let other_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if element_contains(&snap, &container_sel, &other_sel) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // `element.parentNode` / `parentElement`——元素父唯一选择器（修正旧 stub 恒返 body）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_parent",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            parent_selector_for(&snap, &elem_sel)
        }),
    );

    // HTML 规范「Window 上的命名属性访问」：所有带 id 的元素作为全局变量可访问
    // （`<div id="container">` → JS 裸标识符 `container`）。shim 据此在脚本执行前
    // 安装 `globalThis[id] = getElementById(id)`（仅合法标识符、不覆盖已存在全局）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_collect_ids",
        Box::new(move |_args| {
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, collect_element_ids_doc)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_attr",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_attr_from_html_doc(doc, &args[0], &args[1]))
        }),
    );

    // R2995：sel-based `getAttribute` 专用 latest-wins 变体。区别于 `__zw_get_attr`（纯快照，供 defaultValue /
    // role / aria / value 懒初始化等反射 getter，须稳定读快照避免 .value= 脏污 defaultValue），本回调先 consult
    // 变更列表（同批 setAttribute/removeAttribute 在 render apply 前不入快照），命中 SetAttr→新值 /
    // RemoveAttr→空串（absent）；无命中回落快照。闭合 removeAttribute 后 getAttribute 仍返旧值的 stale gap（R2993）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_attr_lw",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(ov) = sel_attr_override(&list, &args[0], &args[1]) {
                return ov.unwrap_or_default();
            }
            drop(list);
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_attr_from_html_doc(doc, &args[0], &args[1]))
        }),
    );

    // P1a form input：真实 tag 名查询（shim `_tagFromSel` 对 id-only 选择器等仅启发式猜测，
    // `__zw_text_input` 需真实 tag 判 INPUT/TEXTAREA）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_tag",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_tag_from_html_doc(doc, &sel))
        }),
    );

    // `getComputedStyle(el).getPropertyValue(prop)`——计算样式（display/position/visibility/
    // opacity + 颜色族）。**per-snapshot + per-style-version 缓存**：(html_key, style_version) →
    // (selector → ComputedStyle)。Document 非 Send（含 observer/listener 闘包 + html5ever tendril
    // `Cell`），不能入 `Send + Sync` 闭包；故只缓存 `ComputedStyle`（纯值类型，Send）。同 html 同
    // selector 命中 → 仅 serialize（O(1)）；新 selector → parse+cascade 一次并存入——同一元素的多属
    // 性查询（`cs.display;cs.color;cs.visibility`）由 3 次全 cascade 摊销为 1 次。html 变（新 snapshot）
    // 或 inline style mutation 变 → 清空 per-selector 缓存。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    let cs_cache: Arc<Mutex<Option<(String, usize, HashMap<String, ComputedStyle>)>>> = Arc::new(Mutex::new(None));
    sandbox.register_callback(
        "__zw_get_computed_style",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let sel = &args[0];
            let prop = &args[1];
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            // R3030：style_version = mutations.len()（脚本内单调递增，单调反映 inline style 变更）。
            // 与快照一同作 cache key：任一变化 → 重算时把 inline style mutation 子集顺序 apply 到
            // parsed doc 后再 cascade（latest-wins，语义同 render），闭合 `el.style.X=` 后 gCS 读 stale。
            let style_version = m.lock().unwrap_or_else(|e| e.into_inner()).len();
            let mut cache = cs_cache.lock().unwrap_or_else(|e| e.into_inner());
            // html 变或 style_version 变 → 清空 per-selector 缓存，重置 key。
            let need_reset = cache
                .as_ref()
                .is_none_or(|(h, v, _)| h != &*snap || *v != style_version);
            if need_reset {
                *cache = Some(((*snap).clone(), style_version, HashMap::new()));
            }
            let (_, _, map) = cache.as_mut().expect("cs cache populated");
            // 同 selector 命中 → 直接 serialize（O(1)）。
            if let Some(style) = map.get(sel) {
                return serialize_computed_property(style, prop);
            }
            // 未命中：parse + apply inline-style overrides + cascade，提取该 selector 的 ComputedStyle
            // 并缓存，再 serialize。clone 变更列表后即释放锁，parse+cascade 不持 mutation 锁。
            let mlist = m.lock().unwrap_or_else(|e| e.into_inner()).clone();
            let (doc, styles) = compute_document_styles_with_inline_overrides(&snap, &mlist);
            let Some(node) = find_by_selector(&doc, sel) else {
                return String::new();
            };
            let Some(style) = styles.get(&node) else {
                return String::new();
            };
            let value = serialize_computed_property(style, prop);
            map.insert((*sel).clone(), style.clone());
            value
        }),
    );

    // P1a checkbox：属性存在性查询（boolean 属性 checked/disabled 靠存在性；getAttribute 返空串
    // 无法区分存在与空值，故 `el.checked` getter / toggle 判定用本回调）。返 "1"/"0"。纯快照读——
    // 反射 getter（checked/defaultChecked）须稳定，故 latest-wins 见 `__zw_has_attr_lw`（hasAttribute 专用）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_has_attr",
        Box::new(move |args| {
            if args.len() < 2 {
                return "0".to_string();
            }
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if has_attribute(&snap, &args[0], &args[1]) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // R2995：sel-based `hasAttribute` 专用 latest-wins 变体（区别于纯快照 `__zw_has_attr`，理由同
    // `__zw_get_attr_lw`）。先 consult 变更列表：命中 SetAttr→"1" / RemoveAttr→"0"；无命中回落快照。
    // 闭合 removeAttribute 后 hasAttribute 恒 true 的 stale gap（R2993 latent）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_has_attr_lw",
        Box::new(move |args| {
            if args.len() < 2 {
                return "0".to_string();
            }
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(ov) = sel_attr_override(&list, &args[0], &args[1]) {
                return if ov.is_some() { "1" } else { "0" }.to_string();
            }
            drop(list);
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if has_attribute(&snap, &args[0], &args[1]) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // 元素全部属性名（`|` 分隔）→ shim `getAttributeNames`/`hasAttributes`/`dataset` 枚举。R3002：latest-wins
    // ——在快照基底上应用 pending SetAttr/RemoveAttr（同 sel），反映同批 setAttribute/removeAttribute/dataset 设删
    // （旧纯快照 → stale，R2995 限制 ③）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_attr_names",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            let mlock = m.lock().unwrap_or_else(|e| e.into_inner());
            element_attribute_names_lw(&snap, &mlock, &sel)
        }),
    );

    // P1a select：读 `<select>` 当前选中 option 的 value（HTML spec 语义：首个 selected option，
    // 无则首 option）。shim `select.value` getter 对 tag=SELECT 调此（非 value 属性）。R3000：先 consult
    // 最新 `SelectOption` mutation（`select.value=` 编程选中），无则回落快照（旧不 consult → stale）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_select_value",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let mlock = m.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(v) = latest_select_option_value(&mlock, &sel) {
                return v.to_string();
            }
            drop(mlock);
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            select_value_from_html(&snap, &sel)
        }),
    );

    // P1a select：读选中 option 的索引（shim `select.selectedIndex` getter）。R3000：先 consult 最新
    // `SelectOption` mutation → 匹配 option 的索引；无 SelectOption / 无匹配 → 回落快照。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_select_index",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            // 先取最新编程选中值（owned，drop mutation 锁后再锁 html，避免持双锁）。
            let opt_val: Option<String> = {
                let mlock = m.lock().unwrap_or_else(|e| e.into_inner());
                latest_select_option_value(&mlock, &sel).map(str::to_owned)
            };
            if let Some(v) = opt_val {
                let snap = html.lock().unwrap_or_else(|e| e.into_inner());
                let idx = option_index_for_value(&snap, &sel, &v);
                if idx >= 0 {
                    return idx.to_string();
                }
            }
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            select_index_from_html(&snap, &sel).to_string()
        }),
    );

    // R3000：读 option 的 selected 态（shim `option.selected` getter sel 路径调此）。consult pending mutations
    // （SetAttr/RemoveAttr{selected} latest-wins + SelectOption 关联 option↔所属 select，最新适用胜出），无 → 回落
    // 快照（selected 属性存在性）。区别于通用 `__zw_has_attr_lw`：本回调感知 SelectOption（编程选中）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_option_selected",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            let mlock = m.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(b) = option_selected_resolved(&snap, &mlock, &sel) {
                return if b { "1".into() } else { "0".into() };
            }
            drop(mlock);
            // 快照回落：selected 属性**存在性**（非值——boolean 属性 selected 无值，query_attr_from_html
            // 返空串，须用 has_attribute 区分 absent vs present-empty）。
            if has_attribute(&snap, &sel, "selected") {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // P1a select：编程设 `select.value = value`——记录 SelectOption mutation（apply 时 mark
    // 匹配 option selected + deselect 兄弟）。匹配浏览器语义：编程设值不自动派 change。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_select_option",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SelectOption {
                        selector: args[0].clone(),
                        value: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_text",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_text_from_html_doc(doc, &sel))
        }),
    );

    // R3028：sel-based `textContent` getter 专用 latest-wins 变体。区别于 `__zw_get_text`（纯快照，供
    // output.defaultValue / textarea 初始 value 等反射 getter，须稳定读快照避免 `textContent=` 脏污
    // 默认值），本回调先 consult 变更列表（同批 `textContent=` 在 render apply 前不入快照），命中
    // SetText→新文本；无命中回落快照。闭合 `textContent=` 后 getter 仍返旧值的 stale gap + 供
    // MutationObserver characterDataOldValue mutate 前 old-value 读（镜像 `__zw_get_attr_lw`）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_text_lw",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(t) = sel_text_override(&list, &sel) {
                return t;
            }
            drop(list);
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_text_from_html_doc(doc, &sel))
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_attr_handle",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_attr_from_mutations(&list, &args[0], &args[1])
        }),
    );

    // create 句柄元素的属性存在性（`new Option()` 创建的句柄 option `.selected`/`.defaultSelected`
    // 读——句柄元素不在 HTML 快照，sel-based `__zw_has_attr` 对其恒 false）。返 "1"/"0"。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_has_attr_handle",
        Box::new(move |args| {
            if args.len() < 2 {
                return "0".to_string();
            }
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            if has_attr_from_mutations(&list, &args[0], &args[1]) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // create 句柄元素的全部属性名（`|` 分隔，变更序）——供 handle 元素 `el.dataset` 枚举（ownKeys）等
    // 遍历属性名场景。句柄元素不在 HTML 快照，属性名仅来自 SetAttrOnHandle/RemoveAttrOnHandle
    //（正序 latest-wins，无快照基底）。R3196：闭合 R3195 限制①（旧 handle dataset 枚举恒返 []）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_attr_names_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            attribute_names_from_mutations(&list, &handle)
        }),
    );

    // create 句柄元素的属性**真移除**（`el.removeAttribute(name)` on handle 元素——区别于 `__zw_set_attr_handle`
    // 空值残留；布尔/存在性属性须移除才 unset；R2993 闭合 hasAttribute-after-remove + CE post-remove old=null）。
    // 记 [`DomMutation::RemoveAttrOnHandle`]；query/has 函数 latest-wins 据此判 absent。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_attr_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveAttrOnHandle {
                        handle: args[0].clone(),
                        name: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_text_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_text_from_mutations(&list, &handle)
        }),
    );

    // detached createElement 句柄元素的真实 tag 名（shim `tagName`/`nodeName` 对 handle-only
    // 元素原走 `_tagFromSel` 恒猜 DIV；本回调从 CreateElement 记录取真实 tag）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_tag_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_tag_from_mutations(&list, &handle)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_attr",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock().unwrap_or_else(|e| e.into_inner()).push(DomMutation::SetAttr {
                    selector: args[0].clone(),
                    name: args[1].clone(),
                    value: args[2].clone(),
                });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_form_value",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetFormValue {
                        selector: args[0].clone(),
                        value: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // R3254-M7'：页面 `element.focus()`/`blur()` —— shim 已在 V8 内派发 focus 事件，
    // 宿主仅同步 retained 焦点状态（不写 DOM）。selector 为空串表示 blur。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_focus_changed",
        Box::new(move |args| {
            let selector = args.first().map(String::from).filter(|s| !s.is_empty());
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::FocusChanged { selector });
            "ok".into()
        }),
    );

    // `element.removeAttribute(name)` / `delete el.dataset.x` —— 真移除属性（区别于 SetAttr 空值；
    // 布尔/存在性属性须移除才 unset）。记 `DomMutation::RemoveAttr`。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_attr",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveAttr {
                        selector: args[0].clone(),
                        name: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // `element.toggleAttribute(name, force?)`——R3192：**enqueue-时解析**决策（旧 apply-时解析使连续
    // toggle 返值 stale——shim 无法预测 apply 结果）。本回调计算 latest-wins presence（pending SetAttr/
    // RemoveAttr 经 [`sel_attr_override`] + 快照 [`has_attribute`]），决定 want，入队**具体** SetAttr/
    // RemoveAttr（非 ToggleAttribute），返 `"1"`/`"0"`（post-toggle presence）。enqueue-时解析使所有 lw
    // 读（getAttribute/hasAttribute/后续 toggle）经既有 sel_attr_override 一致反映——闭合 R3191 连续 toggle
    // 返值 stale 限制。force：`"1"` 强加、`"0"` 强移、缺省切换。注意锁序：先释放 m 锁再取 html 锁（避死锁）。
    let m = Arc::clone(mutations);
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_toggle_attribute",
        Box::new(move |args| {
            if args.len() < 2 {
                return "0".into();
            }
            let force = if args.len() >= 3 {
                match args[2].as_str() {
                    "1" => Some(true),
                    "0" => Some(false),
                    _ => None,
                }
            } else {
                None
            };
            // latest-wins presence：pending SetAttr/RemoveAttr 优先（逆序首命中），无命中回落快照。
            let present = {
                let list = m.lock().unwrap_or_else(|e| e.into_inner());
                match sel_attr_override(&list, &args[0], &args[1]) {
                    Some(ov) => ov.is_some(),
                    None => {
                        drop(list);
                        let snap = html.lock().unwrap_or_else(|e| e.into_inner());
                        has_attribute(&snap, &args[0], &args[1])
                    }
                }
            };
            let want = force.unwrap_or(!present);
            let mut list = m.lock().unwrap_or_else(|e| e.into_inner());
            if want && !present {
                list.push(DomMutation::SetAttr {
                    selector: args[0].clone(),
                    name: args[1].clone(),
                    value: String::new(),
                });
            } else if !want && present {
                list.push(DomMutation::RemoveAttr {
                    selector: args[0].clone(),
                    name: args[1].clone(),
                });
            }
            drop(list);
            if want { "1".into() } else { "0".into() }
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_style",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock().unwrap_or_else(|e| e.into_inner()).push(DomMutation::SetStyle {
                    selector: args[0].clone(),
                    property: args[1].clone(),
                    value: args[2].clone(),
                });
            }
            "ok".into()
        }),
    );

    // `el.style.removeProperty(prop)` — 真移除 style 声明（SetStyle 空值仍 push，不移除）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_style",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveStyle {
                        selector: args[0].clone(),
                        property: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // R3194：`__zw_get_style_lw(sel)`——element inline style **latest-wins** 读（闭合 R3193 已知限制①：
    // sync set→read stale）。snapshot style 为基底，顺序 replay 同 sel 的 pending style-affecting mutation：
    // SetAttr('style',v) 整体覆盖 / RemoveAttr('style') 清空 / SetStyle per-prop merge / RemoveStyle per-prop
    // 移除。**保留 SetStyle/RemoveStyle 变体**（pipeline `is_paint_only_mutation` 依赖 property 粒度跳过
    // relayout——若 enqueue-时解析为 SetAttr('style',merged) 会丢 property 信息致 paint-only 优化失效）。
    let m = Arc::clone(mutations);
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_style_lw",
        Box::new(move |args| {
            if args.is_empty() {
                return String::new();
            }
            let selector = &args[0];
            // 完整顺序 replay：基底为 snapshot style，顺序应用同 sel 的全部 style-affecting mutation
            //（SetAttr/RemoveAttr on 'style' 整体覆盖/清空，SetStyle/RemoveStyle per-prop merge/remove）。
            // 后 apply 自然覆盖先 apply（含 cssText SetAttr 覆盖此前 per-prop SetStyle）——latest-wins。
            // handle 变体（SetStyleOnHandle 等）key 不同，跳过。
            let mut style = {
                let snap = html.lock().unwrap_or_else(|e| e.into_inner());
                with_query_doc(&snap, |doc| query_attr_from_html_doc(doc, selector, "style"))
            };
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            for mt in list.iter() {
                match mt {
                    DomMutation::SetAttr {
                        selector: s,
                        name,
                        value,
                    } if s == selector && name.eq_ignore_ascii_case("style") => {
                        style = value.clone();
                    }
                    DomMutation::RemoveAttr { selector: s, name }
                        if s == selector && name.eq_ignore_ascii_case("style") =>
                    {
                        style.clear();
                    }
                    DomMutation::SetStyle {
                        selector: s,
                        property,
                        value,
                    } if s == selector => {
                        style = merge_style_property(&style, property, value);
                    }
                    DomMutation::RemoveStyle { selector: s, property } if s == selector => {
                        style = remove_style_property(&style, property);
                    }
                    _ => {}
                }
            }
            drop(list);
            style
        }),
    );

    // R3199：`__zw_get_style_lw_handle(handle)`——handle 元素 inline style **latest-wins** 读（闭合 R3194 已知
    // 限制①：handle style sync set→read stale）。句柄元素无快照基底（不在 HTML），正序 replay 同 handle 的
    // style-affecting 变更（SetAttrOnHandle/RemoveAttrOnHandle on 'style' 整体覆盖/清空，SetStyleOnHandle/
    // RemoveStyleOnHandle per-prop merge/remove）——与 `__zw_get_style_lw` 同算法（R3194），区别是无快照基底
    // + 用 *OnHandle 变体。保留 SetStyleOnHandle/RemoveStyleOnHandle 变体（pipeline `is_paint_only_mutation`
    // 依赖 property 粒度跳过 relayout——同 R3194 理由）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_style_lw_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            style_from_mutations_lw(&list, &handle)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_text",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock().unwrap_or_else(|e| e.into_inner()).push(DomMutation::SetText {
                    selector: args[0].clone(),
                    text: args[1].clone(),
                });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove",
        Box::new(move |args| {
            if let Some(sel) = args.first() {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::Remove { selector: sel.clone() });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_element",
        Box::new(move |args| {
            let tag = args.first().map(String::from).unwrap_or_else(|| "div".into());
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateElement {
                    handle: handle.clone(),
                    tag,
                });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_text",
        Box::new(move |args| {
            let text = args.first().map(String::from).unwrap_or_default();
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateTextNode {
                    handle: handle.clone(),
                    text,
                });
            handle
        }),
    );

    // `__zw_create_comment(text)`——document.createComment（R2816）。镜像 `__zw_create_text`（注释 nodeType 8）。
    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_comment",
        Box::new(move |args| {
            let text = args.first().map(String::from).unwrap_or_default();
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateComment {
                    handle: handle.clone(),
                    text,
                });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    let c = Arc::clone(&counter);
    sandbox.register_callback(
        "__zw_create_document_fragment",
        Box::new(move |_args| {
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateDocumentFragment { handle: handle.clone() });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_fragment_children",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendFragmentChildren {
                        parent_selector: args[0].clone(),
                        fragment_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_fragment_children_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendFragmentChildrenByHandle {
                        parent_handle: args[0].clone(),
                        fragment_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_fragment_before",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertFragmentBefore {
                        parent_selector: args[0].clone(),
                        fragment_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_fragment_before_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertFragmentBeforeByHandle {
                        parent_handle: args[0].clone(),
                        fragment_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_child",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendChild {
                        parent_selector: args[0].clone(),
                        child_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_child_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendChildByHandle {
                        parent_handle: args[0].clone(),
                        child_handle: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_before",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertBefore {
                        parent_selector: args[0].clone(),
                        child_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_before_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertBeforeByHandle {
                        parent_handle: args[0].clone(),
                        child_handle: args[1].clone(),
                        ref_selector: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_attr_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetAttrOnHandle {
                        handle: args[0].clone(),
                        name: args[1].clone(),
                        value: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_style_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetStyleOnHandle {
                        handle: args[0].clone(),
                        property: args[1].clone(),
                        value: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // `el.style.removeProperty(prop)` 的 handle 版（detached createElement 元素）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_style_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveStyleOnHandle {
                        handle: args[0].clone(),
                        property: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_text_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetTextOnHandle {
                        handle: args[0].clone(),
                        text: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_inner_html",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_inner_html_from_html_doc(doc, &sel))
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_inner_html",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetInnerHtml {
                        selector: args[0].clone(),
                        html: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_outer_html",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap, |doc| query_outer_html_from_html_doc(doc, &sel))
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_outer_html",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetOuterHtml {
                        selector: args[0].clone(),
                        html: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_inner_html_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            query_inner_html_from_mutations(&list, &handle)
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_set_inner_html_handle",
        Box::new(move |args| {
            if args.len() >= 2 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetInnerHtmlOnHandle {
                        handle: args[0].clone(),
                        html: args[1].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_html",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentHtml {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        html: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_text",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentText {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        text: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_element",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentElement {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        child_handle: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_remove_handle",
        Box::new(move |args| {
            if let Some(handle) = args.first() {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::RemoveHandle { handle: handle.clone() });
            }
            "ok".into()
        }),
    );

    // `HTMLCanvasElement.getContext('2d')`（R2795，canvas slice 1）——host 持 CanvasRegistry（上下文表 + 渐变表），
    // `__zw_canvas_op(handle, op, ...args)` 串参派发（详见 [`canvas_context_op`]）。getContext2d 创建
    // 上下文返 id；getImageData 返 "w:h;r,g,b,a,..."；其余 op 返 "ok"。host 未注册 → shim no-throw 回落。
    // R3268：registry 由调用方创建并传入——painter 需要同一 registry 把 canvas 内容
    // 桥接为显示图元（canvas 显示链路）。
    let canvas_reg = Arc::clone(canvas_registry);
    sandbox.register_callback(
        "__zw_canvas_op",
        Box::new(move |args: &[String]| -> String {
            let handle = args.first().map(String::as_str).unwrap_or("0");
            let op = args.get(1).map(String::as_str).unwrap_or("");
            let rest = if args.len() > 2 { &args[2..] } else { &[] };
            let mut reg = canvas_reg.lock().unwrap_or_else(|e| e.into_inner());
            canvas_context_op(&mut reg, handle, op, rest)
        }),
    );
}

thread_local! {
    /// JS 查询回调的 (html, Document) 解析缓存。
    ///
    /// JS 交互时每次 DOM 查询（__zw_query_match/__zw_get_attr 等）都 parse_html(dom_html
    /// 快照) 全文档重解析（medium 页面 ~1ms/次，动画/交互页面每帧多次）。缓存键 = html
    /// 文本——mutation 应用 / load_html 后快照文本变化 → 自动失效，无需外部失效点。
    /// Document 含 Cell（非 Send，见错误 `std::cell::Cell<usize> cannot be shared`——
    /// 事件监听器/observer 存储）→ 只能 thread_local（JS 执行线程内复用，跨线程各自
    /// 缓存；回调闭包 'static 可直接访问静态）。
    static QUERY_DOC_CACHE: std::cell::RefCell<Option<(String, zero_dom::Document)>> =
        const { std::cell::RefCell::new(None) };
}

/// 在查询 doc（html → Document 缓存解析结果）上执行闭包。
///
/// 缓存键 = html 文本（mutation 应用 / load_html 后快照变化 → 自动失效）；快照相同
/// 复用解析结果（省每次查询全文档 parse_html）。RefMut 无法逃逸 thread_local::with，
/// 故查询逻辑经闭包在 with 内执行。
fn with_query_doc<R>(html: &str, f: impl FnOnce(&zero_dom::Document) -> R) -> R {
    QUERY_DOC_CACHE.with(|cache| {
        let mut guard = cache.borrow_mut();
        if guard.as_ref().map(|(h, _)| h.as_str()) != Some(html) {
            *guard = Some((html.to_string(), parse_html(html)));
        }
        let doc = &guard.as_ref().expect("cache populated").1;
        f(doc)
    })
}
