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
            query_match_selector(&snap, &sel)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_query_all",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            query_all_selector_list(&snap, &sel)
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
            collect_element_ids(&snap)
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
            query_attr_from_html(&snap, &args[0], &args[1])
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
            query_tag_from_html(&snap, &sel)
        }),
    );

    // `getComputedStyle(el).getPropertyValue(prop)`——计算样式（display/position/visibility/
    // opacity + 颜色族）。**per-snapshot 缓存**：html_key → (selector → ComputedStyle)。Document 非
    // Send（含 observer/listener 闭包 + html5ever tendril `Cell`），不能入 `Send + Sync` 闭包；故只
    // 缓存 `ComputedStyle`（纯值类型，Send）。同 html 同 selector 命中 → 仅 serialize（O(1)）；新
    // selector → parse+cascade 一次并存入——同一元素的多属性查询（`cs.display;cs.color;cs.visibility`）
    // 由 3 次全 cascade 摊销为 1 次。html 变（新 snapshot）→ 清空 per-selector 缓存。
    let html = Arc::clone(dom_html);
    let cs_cache: Arc<Mutex<Option<(String, HashMap<String, ComputedStyle>)>>> = Arc::new(Mutex::new(None));
    sandbox.register_callback(
        "__zw_get_computed_style",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            let sel = &args[0];
            let prop = &args[1];
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            let mut cache = cs_cache.lock().unwrap_or_else(|e| e.into_inner());
            // html 变 → 清空 per-selector 缓存，重置 key。
            let need_reset = cache.as_ref().is_none_or(|(h, _)| h != &*snap);
            if need_reset {
                *cache = Some(((*snap).clone(), HashMap::new()));
            }
            let (_, map) = cache.as_mut().expect("cs cache populated");
            // 同 selector 命中 → 直接 serialize（O(1)）。
            if let Some(style) = map.get(sel) {
                return serialize_computed_property(style, prop);
            }
            // 未命中：parse + cascade，提取该 selector 的 ComputedStyle 并缓存，再 serialize。
            let (doc, styles) = compute_document_styles(&snap);
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
    // 无法区分存在与空值，故 `el.checked` getter / toggle 判定用本回调）。返 "1"/"0"。
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

    // 元素全部属性名（`|` 分隔）→ shim `el.dataset` 枚举（ownKeys：data-* 属性 → camelCase 键）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_attr_names",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            element_attribute_names(&snap, &sel)
        }),
    );

    // P1a select：读 `<select>` 当前选中 option 的 value（HTML spec 语义：首个 selected option，
    // 无则首 option）。shim `select.value` getter 对 tag=SELECT 调此（非 value 属性）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_select_value",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            select_value_from_html(&snap, &sel)
        }),
    );

    // P1a select：读选中 option 的索引（shim `select.selectedIndex` getter）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_select_index",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            select_index_from_html(&snap, &sel).to_string()
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
            query_text_from_html(&snap, &sel)
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

    // `element.toggleAttribute(name, force?)`——server-side 决策（apply 时读存在性），连续 toggle
    // 正确复合。第三参 force：`"1"` 强加、`"0"` 强移、缺省切换。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_toggle_attribute",
        Box::new(move |args| {
            if args.len() >= 2 {
                let force = if args.len() >= 3 {
                    match args[2].as_str() {
                        "1" => Some(true),
                        "0" => Some(false),
                        _ => None,
                    }
                } else {
                    None
                };
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::ToggleAttribute {
                        selector: args[0].clone(),
                        name: args[1].clone(),
                        force,
                    });
            }
            "ok".into()
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
            query_inner_html_from_html(&snap, &sel)
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
            query_outer_html_from_html(&snap, &sel)
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

    // `HTMLCanvasElement.getContext('2d')`（R2795，canvas slice 1）——host 持 CanvasContext 注册表，
    // `__zw_canvas_op(handle, op, ...args)` 串参派发（详见 [`canvas_context_op`]）。getContext2d 创建
    // 上下文返 id；getImageData 返 "w:h;r,g,b,a,..."；其余 op 返 "ok"。host 未注册 → shim no-throw 回落。
    let canvas_reg: Arc<Mutex<(u64, HashMap<u64, zero_canvas::CanvasContext>)>> =
        Arc::new(Mutex::new((1, HashMap::new())));
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
