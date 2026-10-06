//! `register_dom_callbacks` —— 向 V8 sandbox 注册全部 `__zw_*` DOM 桥接回调。从 js_dom_bridge.rs
//! 拆出（R2976，文件大小治理 slice 4）。连接 [`generate_js_dom_shim`] 产生的 JS shim 与宿主侧
//! DomMutation 收集器：JS 侧 `__zw_*` 扁平回调翻译为 DomMutation / 从 dom_html 快照查询。
//! `use super::*` 复用父模块全部类型与 helper（find_by_selector / compute_document_styles /
//! apply_dom_mutations / crypto_* / canvas_context_op / element_matches_test_selector 等，经
//! pub use 重导出 + 祖先私有项可见）。pub register_dom_callbacks 经 `pub use callbacks::*` 重导出。

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    /// 查询视图缓存（R-baidu3 文档直读 + O(1) 命中键）。
    ///
    /// `view_doc` 承载**已应用视图的解析文档**：查询闭包直接在它上执行，消掉
    /// 字符串视图的每步 `outer_html` serialize + `parse_html` re-parse 往返。
    /// 内容恒等式 = `parse(snap)` + 队列 `[0..count)` 内全部 InsertAdjacentHtml
    /// 烘焙（与 R57 字符串路径同一 [`apply_dom_mutations`] applier，只省序列化
    /// 往返）。单条目——文档换代/队列清空即整体换血，无跨视图污染。
    ///
    /// **thread_local 而非共享 `Arc<Mutex>`**：`zero_dom::Document` 非 Send
    ///（tendril NonAtomic / observer `dyn Fn`），进不得 sandbox 回调的 Send 捕获
    /// 边界；而同 epoch 视图文档只被创建它的 V8 线程读写（回调与注册同线程，
    /// 与 [`LIVE_QUERY_DOC`] 同一约束面），thread_local 即正确归属。
    ///
    /// 命中键 = `(dom_arc, mut_arc, count, drain_gen, view_gen)`——registration epoch 内
    /// `dom_html` 内容不可变（写入点只有注册时初值与 R348 重绑，两者都安装
    /// **新 Arc**），`mutations` 只增不减 ⇒ 键相同 ⇒ 查询视图输入逐字节相同
    ///（R-baidu2 的 `src == *snap` 全文比较被此恒等式取代，baidu 类 436KB 页面
    /// 每回调省一次 O(html_len) memcmp）。**键存 Arc 克隆而非 `Arc::as_ptr`**——
    /// 条目持有引用使旧 epoch 的分配地址不可能被复用（ABA 免疫；裸指针键在跨
    /// 注册地址复用时会服务 stale 视图）。
    static VIEW_DOC_CACHE:
        RefCell<Option<(ViewDocKey, zero_dom::Document)>> = const { RefCell::new(None) };
}

/// [`VIEW_DOC_CACHE`] 条目键（命中判定字段 + 增量链前提）。
struct ViewDocKey {
    dom_arc: Arc<std::sync::Mutex<String>>,
    mut_arc: Arc<std::sync::Mutex<Vec<DomMutation>>>,
    count: usize,
    /// 条目构建时的 drain 代际（[`MUT_DRAIN_GEN`]）——增量链的成立前提。
    drain_gen: usize,
    /// 条目构建时的快照换代代际（[`DOM_VIEW_GEN`]）——就地换代后旧条目/旧链作废。
    view_gen: usize,
}

/// R57（FV M3）：查询前把 pending mutations 应用到**快照副本**——同批 mutation
///（insertAdjacentHTML 等）未应用时查询快照 stale（form-requestsubmit 的
/// insertAdjacentHTML 后 querySelector 返回 null）。**不修改存储快照、不清队列**：
/// 队列由 host 在脚本运行结束后统一读取+清空并应用到活 DOM（查询时清空会丢失
/// latest-wins 兜底数据——radio 组 valueMissing 的 .checked= 同批查询回归）；
/// apply 失败回落未应用快照（当前元素属性由 `__zw_has_attr_lw` latest-wins 兜底）。
///
/// **应用范围 = 结构级 mutation only**（InsertAdjacentHtml/SetInnerHtml/SetOuterHtml/
/// Remove）：
/// - 属性级（SetAttr/RemoveAttr/SetStyle/SetFormValue/SetAttrOnHandle…）**不应用**——
///   属性读取经 latest-wins 队列（`__zw_has_attr_lw`/`__zw_get_attr_lw`）已覆盖，且
///   部分既有测试断言「查询反映 pending 前」语义（R3190 `[data-x]` 存在性选择器）；
/// - handle 链（CreateElement/AppendChild/SetAttrOnHandle…）**不应用**——pending 元素
///   由 shim 侧本地 registry 回落（R51c：querySelector('#fresh') === el 须同一 proxy
///   身份——host 命中会包成 sel-based proxy 破坏 `===`）。
///
/// R-baidu3 文档直读形态：在缓存条目维护的**已应用视图文档**上执行查询闭包
/// `f`（消字符串视图的 serialize + re-parse 往返）。`live_ok` = 队列无 pending
/// structural mutations（js-dom M1 L2 R102：live_ok 时查询可直读已发布的 live
/// doc，见 [`with_query_doc_live_aware`]）。
///
/// 重入约束：`f` 在持有 cache 锁期间执行——`f` 必须是纯文档遍历（不得再入
/// `__zw_*` 查询回调；与 [`with_query_doc`] 的 thread_local 借用同一约束面）。
fn with_query_view_doc<R>(
    html: &Arc<std::sync::Mutex<String>>,
    mutations: &Arc<std::sync::Mutex<Vec<DomMutation>>>,
    f: impl FnOnce(&zero_dom::Document) -> R,
) -> R {
    // uievents-compat 尾簇 13：视图基座**钉定**在首见时点 html（按 drain_gen 代际——
    // 导航清史即重钉）。dom_html Arc 存在换代写入点（R55 dispatch_event 每次重注册
    // 换新 Arc = 最新 cached_html、user_actions 批末内容更新、R348 重绑）——换代后
    // 基座已含「已 apply 的落地拷贝」（InsertAdjacentHtml 物化结果），而重放区间
    // [0..count) 仍含同一 op → **双计**（WPT image-map img-resized 双案：视图残留
    // 已移除 area → hit test 命中幽灵、宿主派发回落 img）。钉定后基座恒为首见快照
    // + [0..count] 重放 = R100 原始设计语义（dom_html 不可变 + 队列全史）；native
    // 写/live 读经 live-first 路径（publish_live_query_doc）覆盖，不依赖视图基座。
    let count = mutations.lock().unwrap_or_else(|e| e.into_inner()).len();
    // live_ok 判定：队列 [0..count) 无 pending structural mutations——与 R57 字符串
    // 路径的 structural.is_empty()/base_live 传播逐点一致（结构性插入一旦入队即
    // false；全空即 true）。true 且 live doc 已发布 → 闭包直读 live（js-dom M1 L2
    // R102，[`with_query_doc_live_aware`] 同语义），视图文档根本不用建。
    let live_ok = {
        let mut_guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        !mut_guard[..count]
            .iter()
            .any(|m| matches!(m, DomMutation::InsertAdjacentHtml { .. }))
    };
    // FnOnce 单次调用：live 命中即消费；miss 时经 Option 还回落。
    let mut f = Some(f);
    if live_ok {
        let live_hit = LIVE_QUERY_DOC.with(|slot| {
            slot.borrow()
                .as_ref()
                .and_then(|rc| rc.try_borrow().ok())
                .and_then(|doc| f.take().map(|f| f(&doc)))
        });
        if let Some(r) = live_hit {
            return r;
        }
    }
    let f = f.expect("live miss path: f not consumed");
    let view_gen = DOM_VIEW_GEN.load(std::sync::atomic::Ordering::Relaxed);
    // R-baidu3 增量视图维护（文档直读形态）：同 epoch（Arc 身份相同）且队列只增长
    //（count > 上一条目的 count，drain 代际未变）时，从**上一条目的 view_doc**原地
    // 只应用新增区间 [prev_count, count) 的 structural mutations。baidu 加载期
    // loader 每次插入推进 count，字符串路径每步 parse(732KB)+serialize 全量重放
    // × 数百步 = 宿主侧 15s 重解析风暴主力（V8 GC 风暴清零后的残余 CPU 自旋）；
    // 文档路径每步只 parse 插入片段本身。顺序应用可结合 ⇒ 结果与全量重放逐节点
    // 一致。队列清空（drain_gen 递增）或换 epoch → 全量重建基座。
    //
    // O(1) 命中：epoch Arc 身份 + 队列长度唯一决定视图输入（见 [`VIEW_DOC_CACHE`]
    // 文档的不变式）。Arc::ptr_eq 对着条目持有的克隆比较——旧 epoch 的条目活着
    // 时地址不可能被新 Arc 复用（ABA 免疫）。借用跨 `f`：同 [`with_query_doc`]
    // 的 thread_local 约束面（`f` 纯文档遍历，不得再入查询回调）。
    VIEW_DOC_CACHE.with(|slot| {
        let mut cache_guard = slot.borrow_mut();
        if let Some((key, doc)) = cache_guard.as_ref()
            && Arc::ptr_eq(&key.dom_arc, html)
            && Arc::ptr_eq(&key.mut_arc, mutations)
            && key.count == count
            && key.drain_gen == MUT_DRAIN_GEN.load(std::sync::atomic::Ordering::Relaxed)
            && key.view_gen == view_gen
        {
            return f(doc);
        }
        let prev_count = cache_guard
            .as_ref()
            .and_then(|(key, _)| {
                (Arc::ptr_eq(&key.dom_arc, html)
                    && Arc::ptr_eq(&key.mut_arc, mutations)
                    && count > key.count
                    && key.drain_gen == MUT_DRAIN_GEN.load(std::sync::atomic::Ordering::Relaxed)
                    && key.view_gen == view_gen)
                    .then_some(key.count)
            })
            .unwrap_or(0);
        let chain_hit = prev_count > 0;
        // 结构级 mutation 子集（见 register_dom_callbacks 处 R57 文档——属性级/handle
        // 链/Remove/SetInnerHtml 不应用）。Remove 排除：被移除元素的 proxy 属性读取
        //（old.id / removedNodes[].tagName——R3029/replace_child_e2e）须回落快照；且
        // R47 断言 remove 后 querySelector 仍命中。SetInnerHtml/SetOuterHtml 排除同理
        //（R3029：innerHTML= 替换后 removedNodes[] 的 tagName 读旧子）；form-
        // requestsubmit 的需求（同批 insertAdjacentHTML 后 querySelector 命中）由
        // InsertAdjacentHtml 覆盖。
        // 增量步：条目已归我们所有（take 出来原地改，零 clone）；否则全新 parse 基座。
        let mut doc = if chain_hit {
            cache_guard.take().expect("chain_hit implies entry").1
        } else {
            parse_html(&html.lock().unwrap_or_else(|e| e.into_inner()))
        };
        let structural: Vec<DomMutation> = {
            let mut_guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
            mut_guard[prev_count..count]
                .iter()
                .filter(|m| matches!(m, DomMutation::InsertAdjacentHtml { .. }))
                .filter(|m| {
                    // uievents-compat 尾簇 13：**基座已反映去重**——dom_html 存在换代
                    // 写入点（R55 dispatch_event 每次重注册换新 Arc = 最新 cached_html、
                    // user_actions 批末更新），基座可能已含「已 apply 的落地拷贝」；
                    // 全量重放再插一次即双计（WPT image_map img-resized 双案：视图
                    // 幽灵 → hit test 命中残影）。fragment 首元素带 id 且基座已有同
                    // id → 视该 op 已反映，跳过；无 id 片段照旧重放（无法判重，保守）。
                    if let DomMutation::InsertAdjacentHtml { html: frag, .. } = m
                        && let Some(start) = frag.find("id=")
                    {
                        let rest = &frag[start + 4..];
                        let id = rest.strip_prefix('"').and_then(|r| r.find('"').map(|i| &r[..i]));
                        if let Some(id) = id
                            && !id.is_empty()
                        {
                            let has = doc.query_selector(doc.root(), &format!("#{}", id)).is_some();
                            return !has;
                        }
                    }
                    true
                })
                .cloned()
                .collect()
        };
        if !structural.is_empty() {
            // apply 失败回落：增量步保留旧基座（缺新增插入——字符串路径
            // `unwrap_or(base)` 同型；全量步回落未应用 parse——`unwrap_or_else(|_| snap)`
            // 同型）。当前元素属性由 `__zw_has_attr_lw` latest-wins 兜底。
            let _ = apply_dom_mutations(&mut doc, &structural);
        }
        *cache_guard = Some((
            ViewDocKey {
                dom_arc: Arc::clone(html),
                mut_arc: Arc::clone(mutations),
                count,
                drain_gen: MUT_DRAIN_GEN.load(std::sync::atomic::Ordering::Relaxed),
                view_gen,
            },
            doc,
        ));
        f(&cache_guard.as_ref().expect("entry stored").1)
    })
}

/// R-baidu3：注册代际计数器——`register_dom_callbacks` 每次（每 execute 重注册）递增。
/// 视图戳的 ABA 防御项：旧 epoch 的 dom_html Arc 释放后新 Arc 落到同地址时，
/// (ptr, count) 可能碰撞；epoch 项保证跨注册永不碰撞（见 `__zw_dom_view_stamp`）。
static REG_EPOCH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// R-baidu3：mutations 队列 drain 代际——队列排空站点（渲染进程 js_worker、
/// webview user_actions、renderer page_scripts `apply_recorded_mutations` 等，
/// [`bump_mut_drain_gen`] 调用点为权威清单）时递增。drain ⇒ bump 是全视图缓存
/// 键（[`VIEW_DOC_CACHE`]/TAG_MEMO/TAGGED_ALL_CACHE/`__zw_dom_view_stamp`）的
/// 不变式前提：drain 后同批重新增长回旧 count 时 (ptr, count) 键会被误判为
/// 「只增长」/「精确命中」，没有 gen 项会把 pre-drain 视图端出（错视图）。
/// 未配对 view_gen 换代的 drain 站点（不推快照的排空路径）必须直接 bump 本代际。
pub static MUT_DRAIN_GEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// mutations 队列 drain 站点调用（见 [`MUT_DRAIN_GEN`]）。
pub fn bump_mut_drain_gen() {
    MUT_DRAIN_GEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// R358/R3243：dom_html 快照**就地换代**代际——快照写入点（renderer `SetDomSnapshot`
/// 的 `*snap = html`、webview user_actions 批末 `*dom_html.lock() = cached_html`、R348
/// 重绑刷新）调用 [`bump_dom_view_gen`]。快照 Arc 被回调闭包捕获（Box<dyn Fn> 不可达
/// ⇒ 无法换装新 Arc），内容更新只能就地写 ⇒ `(dom_arc, count)` 键对换代视而不见：
/// 换代后同 count 查询命中换代前解析的视图（R358 `children[0].id` 读到旧 span、
/// R3243 insertRow 行数取自旧视图）。代际项进全部宿主侧视图/备忘缓存键（视图文档、
/// TAG_MEMO、TAGGED_ALL_CACHE、NS_MEMO、`__zw_dom_view_stamp`），换代即整体失效。
pub static DOM_VIEW_GEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// dom_html 快照就地写入站点调用（见 [`DOM_VIEW_GEN`]）。
pub fn bump_dom_view_gen() {
    DOM_VIEW_GEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

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
    rect_snapshot_opt: Option<&crate::rect_bridge::LayoutRectSnapshot>,
) {
    // js-dom M3 R100：handle 计数器改 thread-local 单调持久——旧版每次
    // `register_dom_callbacks`（run_page_scripts / execute_script_with_dom / dispatch_event
    // 各自注册）都从 0 重启，跨注册的 `__zw_create_element` 返回碰撞的 `__n0`——
    // `_wrapHandle` 经 `_proxyCache['@__n0']` 返回**旧元素 proxy**，后续 execute 中
    // 恢复的异步框架链（lit performUpdate 的 awaited continuation 在 execute 的
    // microtask checkpoint 处恢复）新建的节点错挂到旧 registry 条目（lit e2e 首渲染
    // 插值丢失实证：p-text "Hello, !" 缺 `${name}`）。单调计数保证 handle 名在页面
    // 生命周期内唯一（导航换页由 `__zw_reset_form_state` 侧 JS 态负责，host 侧计数
    // 复用无害——旧页 handle 不再被引用）。
    // R57（FV M3）：查询视图缓存（同一 execute 内共享——dom_html Arc + 队列同源）。

    // js-dom M4 R55：注册即 dom_html 换代（dispatch_event 每次重注册拿最新 cached_html 的快照
    // Arc）→ JS 侧基底缓存全量失效（childNodes `_zwChildBaseCache` / sibling `_zwSiblingBaseCache`，
    // part05/part04）。幂等 no-op 脚本（函数不存在时静默——shim 未装的首注册先于 shim 执行，
    // 此时尚无缓存可失效）。
    let _ = sandbox.execute(
        "if (typeof _zwChildBaseInvalidateAll === 'function') _zwChildBaseInvalidateAll();\
         if (typeof _zwSiblingBaseInvalidateAll === 'function') _zwSiblingBaseInvalidateAll();",
    );

    let url = Arc::clone(page_url);
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(move |_args| url.lock().unwrap_or_else(|e| e.into_inner()).clone()),
    );

    // `performance.now()`——DOMHighResTimeStamp（ms，单调时钟，自 time origin 起，子毫秒精度）。
    // analytics / 动画计时 / rAF timestamp 高频查询。time origin = 回调注册时刻（页面/脚本启动近似），
    // 回调返 elapsed ms（f64 串）。Instant 单调且 Send+Sync，闭包仅借 &origin 故为 Fn。
    // js-dom M4 R138：origin 提为线程本地共享（`shared_perf_origin`）——native Event.timeStamp
    //（dom_bindings event.rs `perf_now_ms`）此前用**自有 origin**（首次构造 Event 时 Instant::now），
    // 与 performance.now() 的 origin 独立 → WPT Event-timestamp-high-resolution 断言
    // `ev.timeStamp >= before = performance.now()` 恒 false（native timeStamp 从更晚的 origin
    // 起算，数值远小）。共享 origin 后两钟同源（spec「same time origin as performance.now()」）。
    let perf_origin = shared_perf_origin();
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

    // js-dom R150（M4）：`__zw_getBoundingClientRect(selector)`——进程内 gBCR。rect 从
    // 共享 layout snapshot（webview render 后刷新的 `LayoutRectSnapshot`，注册方传入）
    // 读取；selector→NodeId 经 LIVE_QUERY_DOC（live doc 优先，回落查询快照 re-parse）。
    // 多进程路径的 renderer js_worker 自带 rect_bridge 版注册（同回调名，先注册者生效
    // ——进程内 webview 不走 renderer，无冲突）。返 "x,y,w,h"；miss → 空串（shim 回落零 rect）。
    let rect_snapshot = rect_snapshot_opt
        .cloned()
        .unwrap_or_else(|| std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())));
    let rs_html = Arc::clone(dom_html);
    let rs_mut = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(move |args: &[String]| -> String {
            let sel = args.first().map(String::as_str).unwrap_or("");
            if sel.is_empty() {
                return String::new();
            }
            // selector → NodeId：live doc 优先（进程内 webview 常驻发布），
            // miss 回落查询快照 re-parse（与 __zw_query_match 同源逻辑）。
            let node_id = LIVE_QUERY_DOC.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .and_then(|rc| rc.try_borrow().ok())
                    .and_then(|doc| {
                        crate::js_dom_bridge::find_by_selector(&doc, sel).map(crate::hit_test::node_id_to_u64)
                    })
            });
            let node_id = match node_id {
                Some(id) => id,
                None => {
                    let hit = with_query_view_doc(&rs_html, &rs_mut, |doc| {
                        crate::js_dom_bridge::find_by_selector(doc, sel).map(crate::hit_test::node_id_to_u64)
                    });
                    match hit {
                        Some(id) => id,
                        None => return String::new(),
                    }
                }
            };
            let map = rect_snapshot.lock().unwrap_or_else(|e| e.into_inner());
            match map.get(&node_id) {
                Some((x, y, w, h)) => format!("{x},{y},{w},{h}"),
                None => String::new(),
            }
        }),
    );

    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_query_match",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            with_query_view_doc(&html, &m, |doc| query_match_selector_doc(doc, &sel))
        }),
    );

    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_query_all",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            with_query_view_doc(&html, &m, |doc| query_all_selector_list_doc(doc, &sel))
        }),
    );

    // R-baidu3：`__zw_query_all` 的批量 tag 形态——`getElementsByTagName('*')` 全文档
    // 枚举（jQuery/Sizzle）逐元素读 tagName 原本打一次 `__zw_get_tag` 宿主回调
    //（baidu 页单事件 6 万+ 次往返 → exec 超时风暴）；本回调一次往返返回
    // `sel\x1ftag|…`，shim 侧建 sel→tag 缓存（`_realTag` 命中零往返）。payload 按视图
    // 键单条目缓存——同视图内重复枚举零重算（jQuery 每事件多次 gETN('*')）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_query_all_tagged",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let count = m.lock().unwrap_or_else(|e| e.into_inner()).len();
            let drain_gen = MUT_DRAIN_GEN.load(Ordering::Relaxed);
            let view_gen = DOM_VIEW_GEN.load(Ordering::Relaxed);
            if let Some(hit) = TAGGED_ALL_CACHE.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .filter(|(k, _)| {
                        Arc::ptr_eq(&k.0, &html)
                            && Arc::ptr_eq(&k.1, &m)
                            && k.2 == count
                            && k.3 == drain_gen
                            && k.4 == view_gen
                            && k.5 == sel
                    })
                    .map(|(_, payload)| payload.clone())
            }) {
                return hit;
            }
            let payload = with_query_view_doc(&html, &m, |doc| query_all_tagged_list_doc(doc, &sel));
            TAGGED_ALL_CACHE.with(|slot| {
                *slot.borrow_mut() = Some((
                    (Arc::clone(&html), Arc::clone(&m), count, drain_gen, view_gen, sel),
                    payload.clone(),
                ));
            });
            payload
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
            // R160：arg[3] 可选 URL（iframe 子文档的 `:target` fragment 判定）。
            // R161：arg[4] 可选 filter_synthetic（元素子树查询剔合成 html/body 容器）。
            let url = args.get(3).map(String::as_str);
            let filter_syn = args.get(4).map(|s| s == "1").unwrap_or(false);
            parse_html_element_json_full(html, sel, all, url, filter_syn)
        }),
    );

    // R156（js-dom M4）：选择器有效性判定——`element.matches(selector)` /
    // `querySelector` 的 spec 语义是非法选择器抛 SyntaxError DOMException（WPT
    // Element-matches invalidSelectors 簇：Unknown pseudo-class / Undeclared
    // namespace / Invalid combinator 等 33 形态）。查询回调对非法输入静默返空，
    // 无法区分「无匹配」与「非法」——本探针返 "1"/"0" 供 shim matches 先验后抛。
    sandbox.register_callback(
        "__zw_selector_valid",
        Box::new(|args: &[String]| -> String {
            let sel = args.first().map(String::as_str).unwrap_or("");
            if zero_dom::selector_is_valid(sel) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // R5001 M3 片 a 收口（html-syntax-compat）：`Range.createContextualFragment`——
    // markup 从 arg[0]，context tag 从 arg[1]（spec §8.5.7 的 context 元素），scripting
    // 从 arg[2]（'0' = inert 文档）。返片段顶层子 JSON（`__zw_parse_html_child_nodes`
    // foreign 分支同形态）。
    sandbox.register_callback(
        "__zw_parse_fragment_children",
        Box::new(|args: &[String]| -> String {
            let html = args.first().map(String::as_str).unwrap_or("");
            let ctx_tag = args.get(1).map(String::as_str).unwrap_or("body");
            let scripting = args.get(2).map(|s| s != "0").unwrap_or(true);
            fragment_children_json(html, ctx_tag, scripting)
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
            // R5000 片 b：arg[2] 可选 context namespace（foreign context 的本地视图
            // 解析面——createElementNS 容器 innerHTML 的 CDATA/插入模式按 context）。
            // R5001 M3 片 a：arg[3] 可选 scripting（'0' = detached/inert 文档——
            // DOMParser/createHTMLDocument 本地视图，noscript 按 markup 解析）。
            let ctx_ns = args.get(2).map(String::as_str).filter(|s| !s.is_empty());
            let scripting = args.get(3).map(|s| s != "0").unwrap_or(true);
            child_nodes_json_full(html, sel, ctx_ns, scripting)
        }),
    );

    // WC-M2：参数化 html 串的 template contents children（iframe/detached shim 文档的 JS 树
    // 构建数据源——html 是文档自身串，非主文档快照；与 __zw_template_contents 的主文档域区分）。
    // 深形态（children 嵌套）——contents 子非文档树节点，selector 二次定位不可达。
    sandbox.register_callback(
        "__zw_parse_template_contents",
        Box::new(|args: &[String]| -> String {
            let html = args.first().map(String::as_str).unwrap_or("");
            let sel = args.get(1).map(String::as_str).unwrap_or("");
            parse_template_contents_deep_json(html, sel)
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

    // `new TextDecoder(label)`（encoding-compat M2）——labels 标签匹配 + legacy 编码解码
    // host 面（encoding_rs WHATWG tables；模块 doc 记架构）。encoding_of 返规范名
    // （未知 → 空串 shim RangeError；"replacement" → shim 拒绝构造，XHR 解码面照用）。
    // decoder_new 建有状态 decoder 返 handle（跨 decode({stream}) 调用驻留半截多字节
    // lead / iso-2022-jp ESC 模式机）；decoder_decode 字节 csv → JSON {text, err}
    // （err = fatal malformed，shim 抛 TypeError）。未注册时（engine/reftest/polyfill
    // 无 script-runtime 或 shim 先于注册）shim 留守 utf-8 纯 JS 路径，零回归。
    sandbox.register_callback(
        "__zw_text_encoding_of",
        Box::new(|args: &[String]| -> String { text_encoding_of(args.first().map(String::as_str).unwrap_or("")) }),
    );
    sandbox.register_callback(
        "__zw_text_decoder_new",
        Box::new(|args: &[String]| -> String {
            let label = args.first().map(String::as_str).unwrap_or("");
            let ignore_bom = args.get(1).map(String::as_str) == Some("1");
            text_decoder_new(label, ignore_bom)
        }),
    );
    sandbox.register_callback(
        "__zw_text_decoder_decode",
        Box::new(|args: &[String]| -> String {
            let handle: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let data = args.get(1).map(String::as_str).unwrap_or("");
            let fatal = args.get(2).map(String::as_str) == Some("1");
            let last = args.get(3).map(String::as_str) != Some("0"); // 缺省 flush
            text_decoder_decode(handle, data, fatal, last)
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

    // js-dom M4 R113：`CSSStyleSheet.cssRules` 的 handle 版——createElement('style') 后 append 入
    // head 的 CSS-in-JS / WPT prefixed-animation 形态（无 selector，owner 是 handle）。规则源 =
    // mutation 历史（CreateElement 后的 SetTextOnHandle / SetInnerHtmlOnHandle latest-wins），
    // 经既有 `query_inner_html_from_mutations` 取文本 + `style_rules_text` 解析。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_style_rules_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            let mut text = query_inner_html_from_mutations(&list, &handle);
            if text.is_empty() {
                text = query_history_text(&handle);
            }
            if text.is_empty() {
                return String::new();
            }
            style_rules_text(&text)
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
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_query_match_sub",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let sel = args.get(1).map(String::from).unwrap_or_default();
            // js-dom M1 L2 R103：helper 类查询 live 化（_doc 变体）。
            with_query_view_doc(&html, &m, |doc| query_match_in_subtree_doc(doc, &elem_sel, &sel))
        }),
    );

    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_query_all_sub",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let sel = args.get(1).map(String::from).unwrap_or_default();
            with_query_view_doc(&html, &m, |doc| query_all_in_subtree_doc(doc, &elem_sel, &sel))
        }),
    );

    // Form-associated listed controls，按 form owner 过滤并保持文档序。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_form_controls",
        Box::new(move |args| {
            let form = args.first().map(String::from).unwrap_or_default();
            with_query_view_doc(&html, &m, |doc| form_control_selectors_doc(doc, &form).join("|"))
        }),
    );

    // 元素遍历/导航 API：children/firstElementChild/lastElementChild/childElementCount（子列表）、
    // previousElementSibling/nextElementSibling（兄弟对）、contains（后代判定）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_element_children",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            with_query_view_doc(&html, &m, |doc| element_children_selectors_doc(doc, &elem_sel))
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_element_siblings",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, |doc| element_sibling_selectors_doc(doc, &elem_sel))
        }),
    );

    // WC-M2（web-components goal）：template contents 视图——shim template.content 的
    // 子节点数据源（contents fragment 的 children，非 template 的 children——后者解析后
    // 恒空）。深形态 JSON（children 嵌套——contents 子 selector 二次定位不可达）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_template_contents_deep",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, |doc| template_contents_json_doc(doc, &elem_sel))
        }),
    );

    // 主文档浅版（保留兼容命名；消费面已切深版）。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_template_contents",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, |doc| template_contents_json_doc(doc, &elem_sel))
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
            // js-dom R52：DOM 遍历族回调接 `with_query_doc` 缓存（旧每次全文档 re-parse，
            // `body.firstChild` 实测 1.2ms/次——testharness mega-case per-op 主成本）。
            with_query_doc_live_aware(&snap, true, |doc| child_nodes_json_doc(doc, &elem_sel))
        }),
    );

    // 文档根的前导注释（R317）——spec：doctype/文档元素前的 comment/PI 是 document 子节点。
    // 供 part06 `document.childNodes` getter 合成 [comments..., doctype, html]。
    // 文档 doctype 元数据（R317）——host 解析树真实 name/publicId/systemId。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_doc_doctype_json",
        Box::new(move |_args| {
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, doc_doctype_json_doc)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_doc_comments",
        Box::new(move |_args| {
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, doc_top_level_comments_json_doc)
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_sibling_nodes",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, |doc| sibling_nodes_json_doc(doc, &elem_sel))
        }),
    );

    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_contains",
        Box::new(move |args| {
            let container_sel = args.first().map(String::from).unwrap_or_default();
            let other_sel = args.get(1).map(String::from).unwrap_or_default();
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            if with_query_doc_live_aware(&snap, true, |doc| element_contains_doc(doc, &container_sel, &other_sel)) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // `element.parentNode` / `parentElement`——元素父唯一选择器（修正旧 stub 恒返 body）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_parent",
        Box::new(move |args| {
            let elem_sel = args.first().map(String::from).unwrap_or_default();
            with_query_view_doc(&html, &m, |doc| parent_selector_for_doc(doc, &elem_sel))
        }),
    );

    // HTML 规范「Window 上的命名属性访问」：带 id 的元素与四 name-able 元素（embed/
    // form/img/object 非空 name——slice28 RP-1 建；iframe 不入本面，其名走 child
    // navigable 通道由 R139 `__zwRegisterNamedIframes` 委托 contentWindow 值，
    // slice33 I-1 曾误收、缺陷轮 B-1 撤出）作为全局变量可访问
    // （`<div id="container">` → JS 裸标识符 `container`）。shim 据此在脚本执行前
    // 安装 `globalThis[id] = getElementById(id)`（仅合法标识符、不覆盖已存在全局）。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_collect_ids",
        Box::new(move |_args| {
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, collect_element_ids_doc)
        }),
    );
    // slice30（RP-1 同名多命中面）：多命中名清单（≥2 named object 同名，树序首现）。
    // shim `_installNamedAccess` 对这些名安装 HTMLCollection（spec 取值算法多命中返
    // 集合）；单命中名仍走 `__zw_collect_ids` 元素路径。
    // https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_collect_ids_multi",
        Box::new(move |_args| {
            let snap = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc_live_aware(&snap, true, collect_element_ids_multi_doc)
        }),
    );
    let _ = sandbox.execute("if (typeof __zwInstallNamedAccess === 'function') __zwInstallNamedAccess();");

    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_attr",
        Box::new(move |args| {
            if args.len() < 2 {
                return String::new();
            }
            // R57（FV M3）：快照读用 applied view——同批插入（insertAdjacentHTML 等）的
            // 元素属性可见（type/required 等约束读取；SetFormValue 在 apply 中 no-op——
            // .value= 不脏污 defaultValue 语义保持）。
            with_query_view_doc(&html, &m, |doc| query_attr_from_html_doc(doc, &args[0], &args[1]))
        }),
    );

    // R2995：sel-based `getAttribute` 专用 latest-wins 变体。区别于 `__zw_get_attr`（纯快照，供 defaultValue /
    // role / aria / value 懒初始化等反射 getter，须稳定读快照避免 .value= 脏污 defaultValue——SetFormValue 在
    // apply 中 no-op，applied view 同样稳定），本回调先 consult 变更列表（同批 setAttribute/removeAttribute
    // 在 render apply 前不入快照），命中 SetAttr→新值 / RemoveAttr→空串（absent）；无命中回落 **applied view**
    //（R57 FV M3：同批插入元素的结构属性可见）。闭合 removeAttribute 后 getAttribute 仍返旧值的 stale gap（R2993）。
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
            with_query_view_doc(&html, &m, |doc| query_attr_from_html_doc(doc, &args[0], &args[1]))
        }),
    );

    // P1a form input：真实 tag 名查询（shim `_tagFromSel` 对 id-only 选择器等仅启发式猜测，
    // `__zw_text_input` 需真实 tag 判 INPUT/TEXTAREA）。
    // R-baidu3：`__zw_get_tag` 是 jQuery/Sizzle 风暴的最热回调（单事件 6 万+ 次），
    // 走两层 O(1) 化——TAG_MEMO 备忘（同视图键 sel 直接命中）→ with_query_view_doc
    // （epoch 键 O(1) 视图文档命中 + live 直读）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_tag",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let count = m.lock().unwrap_or_else(|e| e.into_inner()).len();
            let drain_gen = MUT_DRAIN_GEN.load(Ordering::Relaxed);
            let view_gen = DOM_VIEW_GEN.load(Ordering::Relaxed);
            if let Some(tag) = TAG_MEMO.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .filter(|(k, _)| {
                        Arc::ptr_eq(&k.0, &html)
                            && Arc::ptr_eq(&k.1, &m)
                            && k.2 == count
                            && k.3 == drain_gen
                            && k.4 == view_gen
                    })
                    .and_then(|(_, map)| map.get(&sel).cloned())
            }) {
                return tag;
            }
            let tag = with_query_view_doc(&html, &m, |doc| query_tag_from_html_doc(doc, &sel));
            TAG_MEMO.with(|slot| {
                let mut guard = slot.borrow_mut();
                let stale = guard
                    .as_ref()
                    .map(|(k, _)| {
                        !(Arc::ptr_eq(&k.0, &html)
                            && Arc::ptr_eq(&k.1, &m)
                            && k.2 == count
                            && k.3 == drain_gen
                            && k.4 == view_gen)
                    })
                    .unwrap_or(true);
                if stale {
                    *guard = Some((
                        (Arc::clone(&html), Arc::clone(&m), count, drain_gen, view_gen),
                        HashMap::new(),
                    ));
                }
                guard.as_mut().expect("entry ensured").1.insert(sel, tag.clone());
            });
            tag
        }),
    );

    // R185（js-dom M4）：sel 元素的 namespace 查询（svg/MathML 等非 HTML ns 的
    // namespaceURI getter + cloneNode ns 保留）。
    // R-baidu3：NS_MEMO 备忘 + 快照零拷贝借用（原实现每次 436KB clone——风暴路径
    // getPrototypeOf 8 万次/事件的内存 churn 主力之一）。
    // R-baidu3 视图戳：`__zw_dom_view_stamp` → "epoch:ptr:count:drain:gen" 小字符串，
    // 同视图恒同串。shim 侧全树枚举（baidu scanAndDoRender 每次 resolve 重扫）以它为
    // 缓存键：戳不变 ⇒ 视图未变 ⇒ 直接复用上次枚举的代理数组/tag 表，**免 400KB
    // payload 调用与 2 万代理重建**（V8 主 GC 风暴——MarkCompact 100% CPU 自旋百秒
    // 级——的分配源头）。ptr 项 = dom_html Arc 身份（同注册内换代必换 Arc）；count 项
    // = mutations 尾长；epoch 项 = REG_EPOCH（跨注册 Arc 地址复用 ABA 防御）；drain 项
    // = MUT_DRAIN_GEN（drain 后重长回同 count 内容可不同——与视图缓存精确命中键同
    // 族，缺项会把 pre-drain 枚举当同视图复用）；gen 项 = DOM_VIEW_GEN（就地换代必
    // 换——R358/R3243，Arc 身份对 `*snap = html` 视而不见）。
    {
        let html = Arc::clone(dom_html);
        let m = Arc::clone(mutations);
        let epoch = REG_EPOCH.fetch_add(1, Ordering::Relaxed);
        sandbox.register_callback(
            "__zw_dom_view_stamp",
            Box::new(move |_args| {
                let ptr = Arc::as_ptr(&html) as usize;
                let count = m.lock().unwrap_or_else(|e| e.into_inner()).len();
                let drain_gen = MUT_DRAIN_GEN.load(Ordering::Relaxed);
                let view_gen = DOM_VIEW_GEN.load(Ordering::Relaxed);
                format!("{epoch:x}:{ptr:x}:{count}:{drain_gen:x}:{view_gen}")
            }),
        );
    }
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_ns",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            let view_gen = DOM_VIEW_GEN.load(Ordering::Relaxed);
            if let Some(ns) = NS_MEMO.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .filter(|(k, _)| Arc::ptr_eq(&k.0, &html) && k.1 == view_gen)
                    .and_then(|(_, map)| map.get(&sel).cloned())
            }) {
                return ns;
            }
            // R185 修正：**纯快照查询**——不走 with_query_view_doc（live 视图构建有
            // 状态副作用：namespaceURI getter 会在 live-collection 过滤（getElementsByTagName
            // 的 matches）中被调用，pending-apply 会消费 mutation 队列破坏查询状态，
            // Attr-prefix g 元素丢失实证）。ns 对静态解析元素恒定，快照足够。
            // 快照经 MutexGuard 直接借用进 with_query_doc（回调闭包与 doc 查询路径
            // 均不再锁 dom_html，无重入死锁面），免 436KB clone。
            let ns = {
                let snap_guard = html.lock().unwrap_or_else(|e| e.into_inner());
                with_query_doc(&snap_guard, |doc| query_ns_from_html_doc(doc, &sel))
            };
            NS_MEMO.with(|slot| {
                let mut guard = slot.borrow_mut();
                if guard
                    .as_ref()
                    .map(|(k, _)| !(Arc::ptr_eq(&k.0, &html) && k.1 == view_gen))
                    .unwrap_or(true)
                {
                    *guard = Some(((Arc::clone(&html), view_gen), HashMap::new()));
                }
                guard.as_mut().expect("entry ensured").1.insert(sel, ns.clone());
            });
            ns
        }),
    );

    // R5000 片 c（html-syntax-compat P5）：`document.characterSet`——dom_html 快照的
    // encoding_label（解析期 `<meta charset>` 预扫描产物，spec encoding sniffing meta
    // prescan 片段）经 encoding_rs 归一为编码名；无标签回落 UTF-8。shim 主文档视图的
    // 旧常量 'UTF-8' 遮蔽原生 getter（part06）——shim 改读本回调。
    let html = Arc::clone(dom_html);
    sandbox.register_callback(
        "__zw_get_character_set",
        Box::new(move |_args: &[String]| -> String {
            let snap_guard = html.lock().unwrap_or_else(|e| e.into_inner());
            with_query_doc(&snap_guard, |doc| {
                doc.encoding_label()
                    .and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes()))
                    .map(|e| e.name().to_string())
                    .unwrap_or_else(|| "UTF-8".to_string())
            })
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
    // 无法区分存在与空值，故 `el.checked` getter / toggle 判定用本回调）。返 "1"/"0"。applied view
    // 读（R57 FV M3：同批插入元素的属性存在性可见；checked/defaultChecked 的稳定语义同
    // __zw_get_attr——SetFormValue no-op），latest-wins 见 `__zw_has_attr_lw`（hasAttribute 专用）。
    let html = Arc::clone(dom_html);
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_has_attr",
        Box::new(move |args| {
            if args.len() < 2 {
                return "0".to_string();
            }
            if with_query_view_doc(&html, &m, |doc| {
                find_by_selector(doc, &args[0])
                    .map(|n| doc.get_attribute(n, &args[1]).is_some())
                    .unwrap_or(false)
            }) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );

    // R2995：sel-based `hasAttribute` 专用 latest-wins 变体（区别于纯快照 `__zw_has_attr`，理由同
    // `__zw_get_attr_lw`）。先 consult 变更列表：命中 SetAttr→"1" / RemoveAttr→"0"；无命中回落
    // **applied view**（R57 FV M3：同批插入元素的属性存在性可见）。闭合 removeAttribute 后
    // hasAttribute 恒 true 的 stale gap（R2993 latent）。
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
            if with_query_view_doc(&html, &m, |doc| {
                find_by_selector(doc, &args[0])
                    .map(|n| doc.get_attribute(n, &args[1]).is_some())
                    .unwrap_or(false)
            }) {
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
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_text",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            // R57（FV M3）：applied view——同批插入元素的结构文本可见。
            with_query_view_doc(&html, &m, |doc| query_text_from_html_doc(doc, &sel))
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
            {
                // 作用域收窄：snap guard（html 锁）须在查询视图构建（会再锁 html——
                // std Mutex 非重入，持锁调用即自死锁——FV M3 实测 textContent= 后
                // getter 挂起）前释放。
                let snap = html.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(text) = query_text_from_pending_mutations(&snap, &list, &sel) {
                    return text;
                }
            }
            drop(list);
            // R57（FV M3）：applied view——同批插入元素的结构文本可见。
            with_query_view_doc(&html, &m, |doc| query_text_from_html_doc(doc, &sel))
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
    // R100：跨 execute 读回落（同 `__zw_get_tag_handle`——当前批记录 miss 的旧 handle
    // 经持久正置表锚回 selector，从查询快照读 textContent）。
    let txt_html = Arc::clone(dom_html);
    let txt_sel_map = sel_handle_map_snapshot();
    sandbox.register_callback(
        "__zw_get_text_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            let text = query_text_from_mutations(&list, &handle);
            if !text.is_empty() {
                return text;
            }
            // R100：当前批 miss → 线程本地已应用历史（跨注册 latest-wins 重放）。
            let hist = query_history_text(&handle);
            if !hist.is_empty() {
                return hist;
            }
            if let Some(sel) = txt_sel_map.lock().unwrap_or_else(|e| e.into_inner()).get(&handle) {
                let sel = sel.clone();
                let snap = txt_html.lock().unwrap_or_else(|e| e.into_inner());
                return with_query_doc_live_aware(&snap, true, |doc| {
                    find_by_selector(doc, &sel)
                        .and_then(|id| doc.text_content(id))
                        .unwrap_or_default()
                });
            }
            text
        }),
    );

    // detached createElement 句柄元素的真实 tag 名（shim `tagName`/`nodeName` 对 handle-only
    // 元素原走 `_tagFromSel` 恒猜 DIV；本回调从 CreateElement 记录取真实 tag）。
    // js-dom M3 R100：当前批记录 miss（跨 execute 引用的旧 handle——本批 mutations 不含
    // 其 CreateElement）时，经持久 selector→handle 反查表锚回 selector，从查询快照读 tag
    //（闭包捕获 dom_html Arc）。两处都 miss → 空串（shim fallback，原行为）。
    let m = Arc::clone(mutations);
    let tag_html = Arc::clone(dom_html);
    let tag_sel_map = sel_handle_map_snapshot();
    sandbox.register_callback(
        "__zw_get_tag_handle",
        Box::new(move |args| {
            let handle = args.first().map(String::from).unwrap_or_default();
            let list = m.lock().unwrap_or_else(|e| e.into_inner());
            let tag = query_tag_from_mutations(&list, &handle);
            if !tag.is_empty() {
                return tag;
            }
            // R100：当前批 miss → 线程本地已应用历史。
            let hist = query_history_tag(&handle);
            if !hist.is_empty() {
                return hist;
            }
            // R100 持久反查：handle → selector → 快照 tag。
            if let Some(sel) = tag_sel_map.lock().unwrap_or_else(|e| e.into_inner()).get(&handle) {
                let sel = sel.clone();
                let snap = tag_html.lock().unwrap_or_else(|e| e.into_inner());
                return with_query_doc_live_aware(&snap, true, |doc| query_tag_selector_doc(doc, &sel));
            }
            String::new()
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
                with_query_doc_live_aware(&snap, true, |doc| query_attr_from_html_doc(doc, selector, "style"))
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
    let c = HANDLE_COUNTER.with(|h| h.clone());
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

    // `__zw_create_element_ns(namespace, qualifiedName)`——document.createElementNS（js-dom M4 / R18）。
    // 区别 `__zw_create_element`：经 `DomMutation::CreateElementNS` → `doc.create_element_ns`，**大小写敏感**
    // 保留原 qualified name（spec createElementNS 不小写），且记录 namespace 供 `namespaceURI` getter。
    // shim `createElementNS` 调本回调，并把句柄记入 `_nsHandles`（存原 qualified name + ns），
    // 使 `tagName`/`prefix`/`localName`/`namespaceURI` getter 返大小写敏感正确值。
    let m = Arc::clone(mutations);
    let c = HANDLE_COUNTER.with(|h| h.clone());
    sandbox.register_callback(
        "__zw_create_element_ns",
        Box::new(move |args| {
            let namespace = args.first().map(String::from).unwrap_or_default();
            let qualified = args.get(1).map(String::from).unwrap_or_else(|| "div".into());
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateElementNS {
                    handle: handle.clone(),
                    namespace,
                    qualified_name: qualified,
                });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    let c = HANDLE_COUNTER.with(|h| h.clone());
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
    let c = HANDLE_COUNTER.with(|h| h.clone());
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

    // `__zw_create_processing_instruction(target, data)`——document.createProcessingInstruction（js-dom M4，
    // spec `dom-document-createprocessinginstruction`）。PI 节点 nodeType 7。镜像 `__zw_create_comment`。
    // spec 校验（非法 target Name / data 含 `?>`）已在 JS 桥（shim）同步抛 DOMException，此处仅收合法值。
    let m = Arc::clone(mutations);
    let c = HANDLE_COUNTER.with(|h| h.clone());
    sandbox.register_callback(
        "__zw_create_processing_instruction",
        Box::new(move |args| {
            let target = args.first().map(String::from).unwrap_or_default();
            let data = args.get(1).map(String::from).unwrap_or_default();
            let n = c.fetch_add(1, Ordering::Relaxed);
            let handle = format!("__n{n}");
            m.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(DomMutation::CreateProcessingInstruction {
                    handle: handle.clone(),
                    target,
                    data,
                });
            handle
        }),
    );

    let m = Arc::clone(mutations);
    let c = HANDLE_COUNTER.with(|h| h.clone());
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

    // t7（js-dom P15 修复）：plain 祖先 append 的路径寻址回调——path 由 shim 以
    // 逗号连接的 child-index 串传入（如 "0,2,1"），此处解析为 Vec<u32>。
    let parse_child_path =
        |s: &str| -> Vec<u32> { s.split(',').filter_map(|p| p.trim().parse::<u32>().ok()).collect() };
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_child_handle_path",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendChildByHandlePath {
                        parent_handle: args[0].clone(),
                        path: parse_child_path(&args[1]),
                        child_handle: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_append_child_sel_path",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::AppendChildBySelPath {
                        parent_selector: args[0].clone(),
                        path: parse_child_path(&args[1]),
                        child_handle: args[2].clone(),
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

    // js-dom M3 R101：全 handle 形态 insertBefore（父/子/ref 都是 create 句柄）——
    // Vue v-for 的 li 挂接形态（anchor comment 无 selector 可翻译）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_before_handle_handle",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertBeforeByHandleHandle {
                        parent_handle: args[0].clone(),
                        child_handle: args[1].clone(),
                        ref_handle: args[2].clone(),
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

    let m = Arc::clone(mutations);
    sandbox.register_callback(
        // js-dom M4 R48：parsed 文本/注释子节点的 CharacterData 编辑——按父 sel + child 索引定位。
        "__zw_set_child_text",
        Box::new(move |args| {
            if args.len() >= 3
                && let Ok(idx) = args[1].parse::<usize>()
            {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::SetChildText {
                        parent_selector: args[0].clone(),
                        child_index: idx,
                        text: args[2].clone(),
                    });
            }
            "ok".into()
        }),
    );

    // siteopt slice24：parsed 非元素子节点（文本/注释/PI）移除——`Node.removeChild`
    // sel-based 路径的 host 真相同步（与 `__zw_set_child_text` 同「父 sel + child 索引」
    // 定位口径；此前四条移除分支均只认 handle/sel 身份，parsed CharacterData 静默穿透，
    // baidu san 水合的 s-data 注释移除失效即此形态）。kill-switch：`ZW_REMOVE_CHILD_AT=0`
    // 回退（off 臂不注册回调，shim 分支的 typeof guard 落回旧形态），照
    // `ZW_MO_HOST_TRIGGER` 先例。
    // https://dom.spec.whatwg.org/#dom-node-removechild
    if std::env::var("ZW_REMOVE_CHILD_AT").as_deref() != Ok("0") {
        let m = Arc::clone(mutations);
        sandbox.register_callback(
            "__zw_remove_child_at",
            Box::new(move |args| {
                if args.len() >= 2
                    && let Ok(idx) = args[1].parse::<usize>()
                {
                    m.lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(DomMutation::RemoveChildAt {
                            parent_selector: args[0].clone(),
                            child_index: idx,
                        });
                }
                "ok".into()
            }),
        );
    }

    let html = Arc::clone(dom_html);
    let muts_ih = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_get_inner_html",
        Box::new(move |args| {
            let sel = args.first().map(String::from).unwrap_or_default();
            // R5002 M3 片 b（html-syntax-compat）：读链切**查询视图文档**——旧
            // live-aware 直读 live doc，同 turn InsertAdjacentHtml 烘焙的子树（含
            // template contents）缺席（查询面 view doc 有、读链 live 无 → 查询命中
            // 元素的 innerHTML/子树读恒空，WPT template.html 两断言 + ambiguous-
            // ampersand 写入子树读链根因）。视图文档无 pending structural mutations
            // 时走同一 live fast path（with_query_view_doc 的 live_ok 臂），空闲期
            // 零行为/零成本变化；Remove/SetInnerHtml 仍按 R57/R3029 约定排除在
            // 烘焙范围外（removed 子树读回落快照语义不变）。
            with_query_view_doc(&html, &muts_ih, |doc| query_inner_html_from_html_doc(doc, &sel))
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
            with_query_doc_live_aware(&snap, true, |doc| query_outer_html_from_html_doc(doc, &sel))
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

    // R182（js-dom M4）：sel 子形态 insertAdjacentElement（静态页面元素移动）。
    let m = Arc::clone(mutations);
    sandbox.register_callback(
        "__zw_insert_adjacent_sel_element",
        Box::new(move |args| {
            if args.len() >= 3 {
                m.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(DomMutation::InsertAdjacentSelElement {
                        selector: args[0].clone(),
                        position: args[1].clone(),
                        child_selector: args[2].clone(),
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
    /// js-dom M4 R138：performance.now() 与 native Event.timeStamp 的**共享 time origin**
    ///（spec DOM：Event timeStamp 与 performance.now() 同 time origin——WPT
    /// Event-timestamp-high-resolution 断言 `ev.timeStamp >= performance.now()` 取自
    /// 创建前）。`shared_perf_origin()` 懒初始化（进程级近似：两消费者 whichever first）。
    static SHARED_PERF_ORIGIN: std::cell::OnceCell<std::time::Instant> = const { std::cell::OnceCell::new() };
    /// js-dom M3 R100：已应用 mutation 的线程本地历史（read 回调跨注册回落源）。
    ///
    /// 旧架构下 `run_page_scripts` 注册的回调闭包持有**未清空**的 mutations Vec，后续
    /// `execute_script`（无重注册）的读回调仍能从中查到历史 CreateElement/TextNode/
    /// SetTextOnHandle 记录；`execute_script_with_dom` 引入按次重注册后，新回调绑空
    /// Vec，旧 handle 的读（lit 插值文本 `data`、tagName 等）全空。webview 在每次
    /// apply 前把该批 `recorded` append 进本历史，读回调在当前批 miss 时查询此处
    /// ——语义等价 HEAD 的「未清空 Vec」，且显式、可换代清理（load_html 清）。
    static MUTATION_HISTORY: std::cell::RefCell<Vec<DomMutation>> = const { std::cell::RefCell::new(Vec::new()) };
    /// JS 查询回调的 (html, Document) 解析缓存。
    ///
    /// JS 交互时每次 DOM 查询（__zw_query_match/__zw_get_attr 等）都 parse_html(dom_html
    /// 快照) 全文档重解析（medium 页面 ~1ms/次，动画/交互页面每帧多次）。缓存键 = html
    /// 文本——mutation 应用 / load_html 后快照文本变化 → 自动失效，无需外部失效点。
    /// Document 含 Cell（非 Send，见错误 `std::cell::Cell<usize> cannot be shared`——
    /// 事件监听器/observer 存储）→ 只能 thread_local（JS 执行线程内复用，跨线程各自
    /// 缓存；回调闭包 'static 可直接访问静态）。
    static QUERY_DOC_CACHE: std::cell::RefCell<Option<(Arc<String>, zero_dom::Document)>> =
        const { std::cell::RefCell::new(None) };
    /// R-baidu2 查询重解析风暴预算（见 [`query_reparse_guard`]）。
    static QUERY_REPARSE: std::cell::RefCell<Option<QueryReparseGuard>> = const { std::cell::RefCell::new(None) };
    /// js-dom M1 L2（R102）：查询回调的 **live Document** 源——pipeline
    /// `cached_doc` 共享句柄（webview 每次 execute/apply 前发布最新；load_html 换代
    /// 发布 None）。发布后无 pending structural mutation 的查询**直接读 live doc**
    ///（只读 borrow），消 `parse_html(dom_html)` 全文档 re-parse——L2「polyfill 桥读
    /// live Document」的查询层落点。镜像 dom_bindings gc.rs 的 DOM_SOURCE 模式。
    static LIVE_QUERY_DOC: std::cell::RefCell<Option<Rc<RefCell<zero_dom::Document>>>> =
        const { std::cell::RefCell::new(None) };
    /// js-dom M3 R100：跨 `register_dom_callbacks` 持久的 handle 计数器（见注册处
    /// 文档——防跨注册 `__n{n}` 名碰撞）。`Arc` 承载（AtomicU64 非 Clone，闭包捕获
    /// 需 'static + 每注册共享同一实例）。
    static HANDLE_COUNTER: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
    /// js-dom M3 R100：**正置** handle→selector 持久表（webview `selector_handle_map`
    /// 的正置镜像——`__zw_handle_for_selector` 注册的倒置表方向相反，host 侧回调
    /// （如 `__zw_get_tag_handle` 的跨 execute 回落）需要正置查询）。生产方
    /// [`publish_forward_handle_map`]（webview 每次 mutation 应用 merge 后发布）。
    /// `RefCell<Option<..>>` 承载（thread_local 值语义不可变，需换柄发布）。
    static FORWARD_HANDLE_MAP: std::cell::RefCell<Option<Arc<Mutex<HashMap<String, String>>>>> =
        const { std::cell::RefCell::new(None) };
}

/// js-dom M4 R138：performance.now() 回调与 native Event.timeStamp 的共享 time origin
///（懒初始化，两消费者 whichever first 锁定）。spec DOM 要求 Event timeStamp 与
/// performance.now() **同 time origin**（WPT Event-timestamp-high-resolution 断言
/// `ev.timeStamp >= performance.now()` 取自创建前——独立 origin 时 native timeStamp
/// 从更晚起点算，数值恒小于 performance.now() 而断言失败）。
pub fn shared_perf_origin() -> std::time::Instant {
    SHARED_PERF_ORIGIN.with(|slot| *slot.get_or_init(std::time::Instant::now))
}

/// js-dom M3 R100：发布正置 handle→selector 表（webview 在 mutation 应用后调用；传
/// `None` 清空——文档换代）。同线程（JS 执行线程）内 publish/读共享同一 Arc。
pub fn publish_forward_handle_map(map: Option<Arc<Mutex<HashMap<String, String>>>>) {
    FORWARD_HANDLE_MAP.with(|slot| {
        *slot.borrow_mut() = Some(map.unwrap_or_else(|| Arc::new(Mutex::new(HashMap::new()))));
    });
}

/// js-dom M1 L2（R102）：发布查询回调的 live Document 源（webview 在沙箱注册/apply
/// 前调用——pipeline `cached_doc_shared()`；`None` = 文档换代清空）。发布后无 pending
/// structural mutation 的查询读 live（消 re-parse），未发布/有 pending 走原快照路径。
pub fn publish_live_query_doc(doc: Option<Rc<RefCell<zero_dom::Document>>>) {
    LIVE_QUERY_DOC.with(|slot| *slot.borrow_mut() = doc);
}

/// js-dom M3 R100：把一批已应用的 mutations append 进线程本地历史（webview apply 前调用）。
pub fn append_mutation_history(batch: &[DomMutation]) {
    MUTATION_HISTORY.with(|h| h.borrow_mut().extend(batch.iter().cloned()));
}

/// js-dom M3 R100：清空 mutation 历史（文档换代——`load_html` 调用；旧页记录在新页无效）。
pub fn clear_mutation_history() {
    MUTATION_HISTORY.with(|h| h.borrow_mut().clear());
}

/// js-dom M3 R100：读回调的跨注册回落——当前批 miss 时查历史批（CreateElement/TextNode/
/// SetTextOnHandle 等 latest-wins 重放语义与 HEAD 的未清空 Vec 一致）。
fn query_history_text(handle: &str) -> String {
    MUTATION_HISTORY.with(|h| query_text_from_mutations(&h.borrow(), handle))
}

/// [`query_history_text`] 的 tag 版。
fn query_history_tag(handle: &str) -> String {
    MUTATION_HISTORY.with(|h| query_tag_from_mutations(&h.borrow(), handle))
}
fn sel_handle_map_snapshot() -> Arc<Mutex<HashMap<String, String>>> {
    FORWARD_HANDLE_MAP.with(|slot| {
        slot.borrow()
            .clone()
            .unwrap_or_else(|| Arc::new(Mutex::new(HashMap::new())))
    })
}

/// js-dom M3 R100：selector → 快照文档中该元素的 tag 名（小写 local name——与
/// `query_tag_from_mutations` 返回形态一致，shim `_zwAsciiUpper` 后呈现）。
/// selector 失配 → 空串（调用方 fallback）。
fn query_tag_selector_doc(doc: &zero_dom::Document, selector: &str) -> String {
    find_by_selector(doc, selector)
        .and_then(|id| {
            let node = doc.get(id)?;
            match &node.kind {
                zero_dom::NodeKind::Element(e) => Some(e.local_name().to_string()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

/// 查询重解析风暴预算守卫（R-baidu2）。
///
/// baidu 类页面以「mutation + query 交替」高频运行：每个 mutation 都使
/// [`QUERY_DOC_CACHE`] 失效，后续每次查询触发**全文档 `parse_html`**，单脚本
/// 执行被拖到分钟级；页面脚本在 renderer 主循环上同步执行（`run_page_scripts`
/// 的 mpmc recv 等待），整条 IPC 管线（自动化响应/子资源推进/绘制发布）随之
/// 冻结。V8 watchdog 的 `terminate_execution` 无法打断宿主回调内的原生解析，
/// 故在解析入口做**预算**：
///
/// - 滚动窗口（5s）内累计重解析耗时 < [`QUERY_REPARSE_BUDGET`]：正常重解析，
///   零语义变化（小文档/常规交互页永远不会触达预算）。
/// - 预算耗尽：进入指数退避（100ms 起，×2 封顶 2s）——退避到期的那次查询
///   仍刷新文档，期间的查询**服务上一次解析的文档**（有界过期快照）。
///   https://dom.spec.whatwg.org/#dom-parentelement-queryselector 要求查询
///   反映当前 DOM；此处是「冻结 vs 有界过期」的显式权衡，仅病理页面触达。
#[derive(Debug)]
struct QueryReparseGuard {
    window_start: std::time::Instant,
    window_spent: std::time::Duration,
    backoff: std::time::Duration,
    last_parse: std::time::Instant,
}

#[derive(Debug, PartialEq, Eq)]
enum ReparseDecision {
    /// 全量重解析（缓存键未变或预算内）。
    Parse,
    /// 服务上一次解析的文档（预算耗尽 + 退避未到期）。
    Stale,
}

const QUERY_REPARSE_WINDOW: std::time::Duration = Duration::from_secs(5);
const QUERY_REPARSE_BUDGET: std::time::Duration = Duration::from_millis(1200);
const QUERY_REPARSE_BACKOFF_START: std::time::Duration = Duration::from_millis(100);
const QUERY_REPARSE_BACKOFF_MAX: std::time::Duration = Duration::from_secs(2);
use std::time::Duration;

impl QueryReparseGuard {
    fn new(now: std::time::Instant) -> Self {
        Self {
            window_start: now,
            window_spent: Duration::ZERO,
            backoff: Duration::ZERO,
            last_parse: now,
        }
    }

    /// 缓存 miss 时的决策：`Parse`（随后必须 [`Self::record`]）或 `Stale`。
    fn decide(&mut self, now: std::time::Instant) -> ReparseDecision {
        if now.duration_since(self.window_start) >= QUERY_REPARSE_WINDOW {
            self.window_start = now;
            self.window_spent = Duration::ZERO;
            self.backoff = Duration::ZERO;
        }
        if self.window_spent < QUERY_REPARSE_BUDGET {
            return ReparseDecision::Parse;
        }
        // 预算耗尽：退避到期放行一次刷新（并阶梯加倍），否则服务有界过期快照。
        if now.duration_since(self.last_parse) >= self.backoff {
            self.backoff = (self.backoff * 2)
                .max(QUERY_REPARSE_BACKOFF_START)
                .min(QUERY_REPARSE_BACKOFF_MAX);
            ReparseDecision::Parse
        } else {
            ReparseDecision::Stale
        }
    }

    fn record(&mut self, now: std::time::Instant, cost: Duration) {
        if now.duration_since(self.window_start) >= QUERY_REPARSE_WINDOW {
            self.window_start = now;
            self.window_spent = Duration::ZERO;
        }
        self.window_spent += cost;
        self.last_parse = now;
    }
}

/// 在查询 doc（html → Document 缓存解析结果）上执行闭包。
///
/// 缓存键 = html 文本（mutation 应用 / load_html 后快照变化 → 自动失效）；快照相同
/// 复用解析结果（省每次查询全文档 parse_html）。RefMut 无法逃逸 thread_local::with，
/// 故查询逻辑经闭包在 with 内执行。
///
/// R-baidu2：缓存 miss 时先问 [`QUERY_REPARSE`] 预算——超预算且退避未到期的
/// 查询服务上一次解析的文档（`f` 收到的 doc 落后于 `html`，有界过期）。
fn with_query_doc<R>(html: &str, f: impl FnOnce(&zero_dom::Document) -> R) -> R {
    QUERY_DOC_CACHE.with(|cache| {
        let mut guard = cache.borrow_mut();
        if guard.as_ref().map(|(h, _)| h.as_str()) != Some(html) {
            let miss_at = std::time::Instant::now();
            let stale_doc = QUERY_REPARSE.with(|slot| {
                let decision = match slot.borrow_mut().as_mut() {
                    Some(g) => g.decide(miss_at),
                    // 首次查询：无旧文档可回退，必须解析。
                    None => ReparseDecision::Parse,
                };
                match decision {
                    ReparseDecision::Parse => None,
                    ReparseDecision::Stale => {
                        if guard.as_ref().is_some() {
                            Some(())
                        } else {
                            // 无旧文档（不应发生：decide 首查即 Parse），强制解析。
                            if let Some(g) = slot.borrow_mut().as_mut() {
                                g.record(miss_at, Duration::ZERO);
                            }
                            None
                        }
                    }
                }
            });
            if stale_doc.is_none() {
                let parse_start = std::time::Instant::now();
                let doc = parse_html(html);
                let cost = parse_start.elapsed();
                QUERY_REPARSE.with(|slot| {
                    slot.borrow_mut()
                        .get_or_insert_with(|| QueryReparseGuard::new(parse_start))
                        .record(parse_start, cost)
                });
                *guard = Some((Arc::new(html.to_string()), doc));
            }
        }
        let doc = &guard.as_ref().expect("cache populated").1;
        f(doc)
    })
}

/// js-dom M1 L2（R102）：live 感知的查询执行——`live_ok`（= 本查询视图无 pending
/// structural mutations，`with_query_view_doc` 判定）且 live doc 已发布时直接读
/// live（消 re-parse），否则原快照路径。查询语义等价前提：apply 后 webview 同步
/// `cached_html` 与 live（`apply_pending_shared_mutations` 的 outer_html 快照），
/// 无 pending 时两者内容一致。
fn with_query_doc_live_aware<R>(html: &str, live_ok: bool, f: impl FnOnce(&zero_dom::Document) -> R) -> R {
    // FnOnce 单次调用：live 命中即消费；miss 时经 Option 还回落（闭包 move 进 with，
    // 用 Some(f) 包装 + take 保证两条路径恰用一次）。
    let mut f = Some(f);
    if live_ok {
        let live_hit = LIVE_QUERY_DOC.with(|slot| {
            slot.borrow()
                .as_ref()
                .and_then(|rc| rc.try_borrow().ok())
                .and_then(|doc| f.take().map(|f| f(&doc)))
        });
        if let Some(r) = live_hit {
            return r;
        }
    }
    with_query_doc(html, f.expect("live miss path: f not consumed"))
}

// __zw_get_tag 热路径备忘缓存（R-baidu3）：`(视图键, sel) → tag`。视图键 =
// (dom_arc, mut_arc, count, drain_gen, view_gen)（与 QueryViewEntry 命中键同一
// 恒等式）——键内 sel→tag 是查询视图的纯函数。jQuery/Sizzle 每次选择器操作经
// getElementsByTagName('*') 全文档枚举，每个元素 proxy 属性读（tagName/localName/…）
// 都打一次 __zw_get_tag 宿主回调；备忘把重复 sel 的宿主往返（含 find_by_selector
// 文档遍历）塌缩为每视图一次。R348 重绑换 dom_html Arc / mutation 追加 → 键变化
// → 旧表整体作废（无跨视图污染）。键存 Arc 克隆（非裸指针）——条目持有引用
// 使旧 epoch 地址不可复用（ABA 免疫，见 QueryViewEntry 文档）。
thread_local! {
    static TAG_MEMO: std::cell::RefCell<
        Option<((Arc<std::sync::Mutex<String>>, Arc<std::sync::Mutex<Vec<DomMutation>>>, usize, usize, usize), std::collections::HashMap<String, String>)>,
    > = const { std::cell::RefCell::new(None) };
}

// __zw_query_all_tagged 的 payload 单条目缓存（R-baidu3）：(dom_arc, mut_arc, count,
// drain_gen, view_gen, sel) → payload。视图键恒等式同 QueryViewEntry（epoch Arc 身份 + 队列长度
// + drain 代际唯一决定视图输入——drain 后重长到同 count 内容可不同，缺 drain_gen 会
// 命中 pre-drain 条目）；sel 或视图变化即换条目。jQuery/Sizzle 每事件多次
// getElementsByTagName('*')，同视图重枚举免 O(匹配数) 的重复 sel+tag 构建。
thread_local! {
    static TAGGED_ALL_CACHE: std::cell::RefCell<
        Option<((Arc<std::sync::Mutex<String>>, Arc<std::sync::Mutex<Vec<DomMutation>>>, usize, usize, usize, String), String)>,
    > = const { std::cell::RefCell::new(None) };
}

// __zw_get_ns 的 sel→ns 备忘（R-baidu3）：键 = dom_html epoch Arc——R185 语义下 ns
// 是**纯快照**函数（不走视图重放），epoch 内快照不可变（见 QueryViewEntry 不变式）
// ⇒ epoch 键即可。风暴路径：proxy getPrototypeOf（HTMLElement/SVGElement 原型选择）
// 每元素属性读触发 ns 查询（baidu 单事件 8 万次/事件，旧实现每次 436KB 快照 clone +
// 全文缓存键比较 → 内存churn 风暴主力之一）。键存 Arc 克隆（ABA 免疫）。
thread_local! {
    static NS_MEMO: std::cell::RefCell<Option<((Arc<std::sync::Mutex<String>>, usize), std::collections::HashMap<String, String>)>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
mod query_reparse_tests {
    use super::*;

    /// 预算内：miss 一律重解析（零语义变化——常规页面永不触达预算）。
    #[test]
    fn budget_not_tripped_allows_every_reparse() {
        let t0 = std::time::Instant::now();
        let mut g = QueryReparseGuard::new(t0);
        assert_eq!(g.decide(t0), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(5), Duration::from_millis(5));
        // 每轮 miss → parse(5ms) × 100 = 累计 500ms < 1200ms 预算，全部放行。
        for i in 1..=100 {
            let now = t0 + Duration::from_millis(5 + i * 10);
            assert_eq!(g.decide(now), ReparseDecision::Parse);
            g.record(now, Duration::from_millis(5));
        }
    }

    /// 预算耗尽：进入指数退避——退避内服务过期快照，到期放行一次刷新并加倍退避。
    #[test]
    fn budget_exhausted_escalates_backoff_with_periodic_refresh() {
        let t0 = std::time::Instant::now();
        let mut g = QueryReparseGuard::new(t0);
        // 打满预算（单窗口 1200ms）。
        g.record(t0 + Duration::from_millis(1), QUERY_REPARSE_BUDGET);
        // 预算刚好耗尽后的首次 miss：退避尚未设置（0）→ 立即放行一次刷新，退避升至 START。
        assert_eq!(g.decide(t0 + Duration::from_millis(2)), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(2), Duration::ZERO);
        assert_eq!(g.backoff, QUERY_REPARSE_BACKOFF_START);
        // 退避窗口内 → 过期快照。
        assert_eq!(g.decide(t0 + Duration::from_millis(60)), ReparseDecision::Stale);
        // 退避到期 → 放行刷新，退避翻倍。
        assert_eq!(g.decide(t0 + Duration::from_millis(105)), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(105), Duration::ZERO);
        assert_eq!(g.backoff, QUERY_REPARSE_BACKOFF_START * 2);
        assert_eq!(g.decide(t0 + Duration::from_millis(250)), ReparseDecision::Stale);
        assert_eq!(g.decide(t0 + Duration::from_millis(310)), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(310), Duration::ZERO);
        assert_eq!(g.backoff, QUERY_REPARSE_BACKOFF_START * 4);
        // 同一窗口内（<5s）继续：退避到期逐次放行并翻倍，直至 2s 封顶。
        assert_eq!(g.decide(t0 + Duration::from_millis(560)), ReparseDecision::Stale);
        assert_eq!(g.decide(t0 + Duration::from_millis(810)), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(810), Duration::ZERO);
        assert_eq!(g.backoff, QUERY_REPARSE_BACKOFF_START * 8);
        assert_eq!(g.decide(t0 + Duration::from_millis(1610)), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(1610), Duration::ZERO);
        assert_eq!(g.backoff, QUERY_REPARSE_BACKOFF_START * 16);
        // 再翻倍触及 2s 封顶。
        assert_eq!(g.decide(t0 + Duration::from_millis(3220)), ReparseDecision::Parse);
        g.record(t0 + Duration::from_millis(3220), Duration::ZERO);
        assert_eq!(g.backoff, QUERY_REPARSE_BACKOFF_MAX);
        // 封顶后（仍在窗口内）：退避未到期 → 过期快照。
        assert_eq!(g.decide(t0 + Duration::from_millis(4220)), ReparseDecision::Stale);
        // 窗口滚过（≥5s）：预算与退避复位 → 恢复正常重解析语义。
        assert_eq!(g.decide(t0 + Duration::from_millis(5230)), ReparseDecision::Parse);
        assert_eq!(g.window_spent, Duration::ZERO);
        assert_eq!(g.backoff, Duration::ZERO);
    }

    /// 滚动窗口：窗口滚过后预算与退避复位（风暴平息后自动恢复正常语义）。
    #[test]
    fn window_roll_resets_budget_and_backoff() {
        let t0 = std::time::Instant::now();
        let mut g = QueryReparseGuard::new(t0);
        g.record(t0 + Duration::from_millis(1), QUERY_REPARSE_BUDGET);
        assert_eq!(g.decide(t0 + Duration::from_millis(2)), ReparseDecision::Parse);
        assert!(g.backoff >= QUERY_REPARSE_BACKOFF_START);
        // 窗口（5s）过期后的第一次 decide：spent/backoff 复位。
        assert_eq!(
            g.decide(t0 + QUERY_REPARSE_WINDOW + Duration::from_millis(10)),
            ReparseDecision::Parse
        );
        assert_eq!(g.window_spent, Duration::ZERO);
        assert_eq!(g.backoff, Duration::ZERO);
    }

    /// 集成路径：预算耗尽时 `with_query_doc` 服务过期文档（查询结果落后于最新 html），
    /// 退避到期后恢复刷新。操纵 QUERY_REPARSE thread_local 以确定性触达预算。
    #[test]
    fn with_query_doc_serves_stale_doc_while_budget_tripped() {
        let html_a = "<html><body><div id=\"a\"></div></body></html>";
        let html_b = "<html><body><div id=\"a\"></div><div id=\"b\"></div></body></html>";
        let count_divs = |doc: &zero_dom::Document| {
            // `query_all_selector_list_doc` 返回「唯一选择器 | 串」——div 数 = 段数。
            crate::js_dom_bridge::query_all_selector_list_doc(doc, "div")
                .split('|')
                .filter(|s| !s.is_empty())
                .count()
        };
        // 首查填充缓存（html_a）。
        let seen_a = with_query_doc(html_a, count_divs);
        assert_eq!(seen_a, 1);
        // 打满预算 + 刚解析过（backoff 窗口内）→ html_b 的查询服务 html_a 的旧文档。
        QUERY_REPARSE.with(|slot| {
            let mut g = slot.borrow_mut();
            let now = std::time::Instant::now();
            let guard = g.get_or_insert_with(|| QueryReparseGuard::new(now));
            guard.window_spent = QUERY_REPARSE_BUDGET;
            guard.last_parse = now;
            guard.backoff = QUERY_REPARSE_BACKOFF_MAX;
        });
        let seen_stale = with_query_doc(html_b, count_divs);
        assert_eq!(seen_stale, 1, "budget-tripped miss must serve the stale cached doc");
        // 退避到期 → 刷新到 html_b。
        QUERY_REPARSE.with(|slot| {
            if let Some(g) = slot.borrow_mut().as_mut() {
                g.backoff = Duration::ZERO;
            }
        });
        let seen_fresh = with_query_doc(html_b, count_divs);
        assert_eq!(seen_fresh, 2, "post-backoff miss must reparse to the fresh doc");
    }
}
