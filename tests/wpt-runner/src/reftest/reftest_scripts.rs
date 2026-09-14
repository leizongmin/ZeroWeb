//! Reftest 脚本辅助 —— 执行 reftest 页面中会修改 DOM 的 JS（harness JS vein）。
//!
//! 很多 WPT reftest 依赖页面脚本动态设置条件（生成元素、改 class、等待事件等）。
//! 这些函数在渲染前 best-effort 执行内联/外链脚本与 `<body onload>` handler，把
//! V8 sandbox 记录到的 `DomMutation` 应用回 HTML，使最终截图反映脚本执行后的状态。

use std::path::Path;

use zero_engine::pipeline::PageScript;
use zero_engine::{
    DomMutation, apply_mutations_to_html, extract_page_scripts, generate_js_dom_shim, register_dom_callbacks,
};

/// 返回 (JS 后最终 HTML, 页面最终焦点 selector)。焦点 selector 取 FocusChanged 记录的
/// **最后一条**（spec 焦点转移语义：最后一次 focus/blur 胜出），供渲染管线
/// `set_focused_selector` 注入 `:focus`/`:focus-within` 样式判定（R4241——serialize→re-parse
/// 边界不携带焦点，须显式跨接）。
pub(super) fn apply_scripted_dom_mutations(
    html: &str,
    base_dir: Option<&Path>,
    wpt_root: Option<&Path>,
    canvas_registry: &std::sync::Arc<std::sync::Mutex<zero_engine::js_dom_bridge::CanvasRegistry>>,
) -> (String, Option<String>) {
    let scripts = extract_page_scripts(html);
    let onload_handlers = extract_onload_handlers(html);
    if scripts.is_empty() && onload_handlers.is_empty() {
        return (html.to_string(), None);
    }

    use std::sync::Arc;
    use std::sync::Mutex;
    use zero_script_sandbox::SandboxConfig;

    let config = SandboxConfig {
        // DOM-mutating reftest 脚本通常很短；与既有 reftest JS 超时一致。
        timeout_ms: 5000,
        persistent_context: true,
        ..Default::default()
    };
    #[cfg(feature = "v8")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> = match zero_script_sandbox::V8Sandbox::with_config(config) {
        Ok(s) => Box::new(s),
        Err(_) => return (html.to_string(), None),
    };
    #[cfg(feature = "quickjs")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        match zero_script_sandbox::QuickJSSandbox::with_config(config) {
            Ok(s) => Box::new(s),
            Err(_) => return (html.to_string(), None),
        };

    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("about:blank")));
    register_dom_callbacks(&mut *sandbox, &mutations, &dom_html, &page_url, canvas_registry, None);
    // R4344：清除潜在跨 case 泄漏的 live 查询文档（本函数末尾同样清）——live 文档
    // 生命周期严格限于本 case 的脚本阶段。
    zero_engine::publish_live_query_doc(None);

    if let Err(e) = sandbox.execute(generate_js_dom_shim()) {
        eprintln!("  [reftest JS] DOM shim init warning: {e}");
        return (html.to_string(), None);
    }
    // reftest harness 自有更完整的 <body>/<frameset>/<html> onload 处理（下方直接执行 handler 体 + 派发
    // 'load'）；禁用 R2946 body→window 反射以避免双 fire（重复 mutation 致 apply_mutations_to_html 失败）。
    let _ = sandbox.execute("globalThis.__zw_no_body_reflect = true;");

    // 按文档序执行每个脚本。外链脚本从 base_dir 读取本地文件（reftest 离线运行）。
    // R4321：每个脚本执行后读 shim `document.write` 写缓冲（见
    // `take_document_write_buffer`），非空则记 (脚本全序号, 缓冲) 供末尾合成
    // SetOuterHtml 替换——WPT ref 页「解析期 write() 无 close」用法（计数器 ref 页
    // system-symbolic / system-additive 等）此前缓冲永不落地 → ref 页缺失 write 行。
    let scripts_indexed = zero_engine::extract_page_scripts_indexed(html);
    let mut doc_write_hits: Vec<(usize, String)> = Vec::new();
    // R4344：累计的结构性 delta（锚定重写后）——live 视图刷新时按文档序整批重放。
    let mut applied_deltas: Vec<Vec<DomMutation>> = Vec::new();
    let mut prev_cnt = 0usize;
    for (exec_pos, script) in scripts.iter().enumerate() {
        let code: Option<String> = match script {
            PageScript::Inline(c) | PageScript::InlineModule(c) => Some(c.clone()),
            PageScript::External(src) | PageScript::ExternalModule(src) => {
                match fetch_external_script(src, base_dir, wpt_root) {
                    Ok(c) => Some(c),
                    Err(e) => {
                        eprintln!("  [reftest JS] external script {src}: {e}");
                        None
                    }
                }
            }
        };
        let Some(code) = code else { continue };
        if code.trim().is_empty() {
            continue;
        }
        // module 语义需要编译管线；reftest 视角下按经典脚本 best-effort 执行。
        // R57（M3）：module 脚本按**块作用域**包裹——真 module 每个文件独立作用域，
        // 经典脚本同全局作用域会让多个 module 脚本的顶层 `const canvas = ...`
        // 重声明互撞（canvas-grid 用例 10 格 canvas 各一个 module 脚本，第 2 格起
        // 全 SyntaxError 中止 → 格子全空白，2d.gradient.colorInterpolationMethod
        // oracle A/B 10.7%）。块包裹近似 module 隔离（import/export 仍不支持——
        // best-effort，parse 失败按既有 warning 流程）。
        let is_module = matches!(script, PageScript::InlineModule(_) | PageScript::ExternalModule(_));
        let full = if is_module {
            format!("__zw_begin_script && __zw_begin_script();\n{{\n{code}\n}}")
        } else {
            format!("__zw_begin_script && __zw_begin_script();\n{code}")
        };
        if let Err(e) = sandbox.execute(&full) {
            eprintln!("  [reftest JS] Script execution warning: {e}");
        }
        // R4344：解析期位置锚定 + live 视图增量刷新。本脚本执行后新增的 mutation 属于
        // 该脚本——先把其中解析期 appendChild 重写为脚本解析位置锚定（文本子 →
        // InsertAdjacentText@script.afterend），再把累计的结构性 delta 在**树级**
        // 重放并发布 live 查询文档（R102 `publish_live_query_doc`）+ 换代 pending
        // 状态（`__zw_reset_pending_state`，R358 快照换代钩子）。
        //
        // 为何走树级 live 文档而非改写 dom_html 字符串：字符串快照的
        // parse→serialize 往返会把**相邻文本节点合并**（序列化无节点边界标记）——
        // basic-004 门卫断言的「两个独立追加文本节点」边界被合并后 previousSibling
        // 链仍断在 SCRIPT 上。树级发布保留节点边界；锚定重放保证追加节点落在解析
        // 位置（脚本之后、解析器后续输出之前），与真浏览器 DOM 序一致。任一 delta
        // 重放失败 → 回退整批（保留旧视图 = 旧行为兜底，不发布新文档）。
        let cur_cnt = mutations.lock().unwrap_or_else(|e| e.into_inner()).len();
        if cur_cnt > prev_cnt {
            let script_ord = scripts_indexed.get(exec_pos).map(|(_, ord)| *ord);
            let structural = {
                let mut rec = mutations.lock().unwrap_or_else(|e| e.into_inner());
                anchor_parse_phase_appends(html, &mut rec, prev_cnt..cur_cnt, script_ord);
                rec[prev_cnt..cur_cnt].iter().any(is_structural_mutation)
            };
            if structural {
                let delta: Vec<DomMutation> =
                    mutations.lock().unwrap_or_else(|e| e.into_inner())[prev_cnt..cur_cnt].to_vec();
                applied_deltas.push(delta);
                // 树级重放：每次从原始 html 重 parse，按文档序重放全部累计 delta——
                // 节点边界在树内保持分离（无序列化往返）。
                let mut doc = zero_dom::parse_html(html);
                let mut chain_err: Option<String> = None;
                for d in &applied_deltas {
                    if let Err(e) = zero_engine::apply_dom_mutations(&mut doc, d) {
                        chain_err = Some(e);
                        break;
                    }
                }
                match chain_err {
                    None => {
                        zero_engine::publish_live_query_doc(Some(std::rc::Rc::new(std::cell::RefCell::new(doc))));
                        let _ = sandbox
                            .execute("if (typeof __zw_reset_pending_state === 'function') __zw_reset_pending_state();");
                        if std::env::var("REFTEST_DEBUG").is_ok() {
                            eprintln!(
                                "  [reftest JS] live doc refreshed (+{} mutation(s), {} delta batch(es))",
                                cur_cnt - prev_cnt,
                                applied_deltas.len(),
                            );
                        }
                    }
                    Some(e) => {
                        applied_deltas.pop();
                        eprintln!("  [reftest JS] live doc refresh warning: {e}");
                    }
                }
            }
            prev_cnt = cur_cnt;
        }
        // 本脚本的全序号：extract_page_scripts 与 extract_page_scripts_indexed 对同一
        // html 的过滤序一致，按执行位次对位取全序号（含非 JS type 的 script）。
        if let (Some(&ord), Some(buf)) = (
            scripts_indexed.get(exec_pos).map(|(_, i)| i),
            take_document_write_buffer(&mut *sandbox),
        ) {
            doc_write_hits.push((ord, buf));
        }
    }

    // 派发 load 事件：(a) 直接执行 `<body onload>` 属性 handler 体；
    // shim 的 setTimeout 经 microtask 立即跑（V8 execute 返回前排空 microtask）。
    for handler in onload_handlers.iter().filter(|h| !h.trim().is_empty()) {
        let full = format!("__zw_begin_script && __zw_begin_script();\n{handler}");
        if let Err(e) = sandbox.execute(&full) {
            eprintln!("  [reftest JS] onload handler warning: {e}");
        }
    }
    // (b) 派发 window 'load' 事件，触发 `addEventListener('load', …)` 监听器（best-effort）。
    // 注：`window.onload = fn` 直写全局属性**无需**再显式调用——R2932 通用 on* IDL
    // accessor（shim part06）把它注册为 window 'load' listener，本派发即触发。再显式
    // 调用 = 双 fire（R4340 A/B 净 −11 的伪影源头：block-between-002 实测 12 条 = 2×6 条记录）。
    let _ = sandbox.execute(
        "if (typeof __zw_dispatch_event === 'function') { try { __zw_dispatch_event('html','load',null); } catch(_e){} }",
    );

    let recorded = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if std::env::var("REFTEST_DEBUG").is_ok() {
        eprintln!("  [reftest JS] recorded {} mutation(s): {:?}", recorded.len(), recorded);
    }
    if std::env::var("REFTEST_DEBUG_HTML").is_ok() {
        let sample = html.chars().take(800).collect::<String>();
        eprintln!("  [reftest JS] original html (first 800): {sample}");
    }
    // R4321：先应用 recorded mutations（既有行为，作用于原始 html），再两段式落地
    // document.write 缓冲（合成 SetOuterHtml 作用于上一步产物）。两段分离保证合成
    // 选择器失配等 Err 只回落 write 兜底（= 旧行为），不殃及 recorded mutations。
    let focus_selector = recorded
        .iter()
        .filter_map(|m| match m {
            DomMutation::FocusChanged { selector } => Some(selector.clone()),
            _ => None,
        })
        .next_back()
        .flatten();
    let mut current = html.to_string();
    if !recorded.is_empty() {
        match apply_mutations_to_html(html, &recorded) {
            Ok(new_html) => {
                if std::env::var("REFTEST_DEBUG_HTML").is_ok() {
                    let sample = new_html.chars().take(2000).collect::<String>();
                    eprintln!("  [reftest JS] mutated html (first 2000): {sample}");
                }
                current = new_html;
            }
            Err(e) => {
                eprintln!("  [reftest JS] apply mutations warning: {e}");
            }
        }
    }
    if !doc_write_hits.is_empty() {
        current = apply_document_write_flushes(current, &doc_write_hits);
    }
    // R4344：脚本阶段结束——清 live 查询文档（防跨 case 泄漏；渲染管线走 apply 后
    // 的 html 字符串，不消费 live 文档）。
    zero_engine::publish_live_query_doc(None);
    (current, focus_selector)
}

/// R4344：解析期 `appendChild` 位置锚定——把单个内联脚本执行期间（`range`）记录的
/// `AppendChild { parent_selector, child_handle }` 重写为脚本解析位置插入：
/// - 文本子（同 range 内 `CreateTextNode` 配对）→ `InsertAdjacentText { script,
///   "afterend", text }`——真浏览器 DOM 序为 `[.., script, 追加文本.., 解析器后续
///   输出..]`（basic-004 门卫断言 previousSibling 链跨越「 intervening 空白文本」，
///   锚必须是紧贴脚本之后的**任意**节点位置，next-element-sibling 不够）；
/// - 其余（元素子）→ `InsertBefore { ref_selector = 下一个元素兄弟 }`（渲染等价
///   保守路径）。
///
/// 依据（解析流语义）：内联脚本执行时脚本之后的静态内容尚未解析——脚本此刻的
/// `appendChild` 落在**当前解析位置**，解析器后续输出排在其后。harness 两段式模型
/// （快照 → 记录 → 末尾统一 apply）把 append 固定在父容器**末尾**，与真浏览器
/// DOM 序分歧。与 R4321 document.write flush 的「插入于脚本自身位置」同原则。
///
/// 守卫（逐条独立判定，不满足保留原 AppendChild 旧行为）：
/// ① 仅解析期调用（onload handler 的 append 由调用方以不调本函数区分——其发生在
///   整树解析后，父容器末尾即真位置）；
/// ② 仅 parent 为脚本元素**祖先**的 append——追加到文档中已完整解析的其他容器
///   （如 getElementById 命中的先前兄弟容器），容器末尾就是 spec 位置，不锚定；
/// ③ 脚本元素可定位且唯一选择器可计算（同 doc-write flush 口径）。
///
/// 顺序保持：`afterend` 连续插入逐条紧贴脚本之后 → 后者先插入会排前面，故同一
/// range 内的文本重写按**逆序**赋位，保持原 append 先后序。
fn anchor_parse_phase_appends(
    html: &str,
    recorded: &mut [DomMutation],
    range: std::ops::Range<usize>,
    script_ord: Option<usize>,
) {
    let Some(ord) = script_ord else { return };
    let doc = zero_dom::parse_html(html);
    let Some(&script_id) = doc.get_elements_by_tag_name("script").get(ord) else {
        return;
    };
    let Some(script_sel) = zero_engine::unique_selector_for_node(&doc, script_id) else {
        return;
    };
    let root = doc.root();
    // 同 range 的 CreateTextNode 配对表（handle → 文本内容）。
    let text_of: std::collections::HashMap<String, String> = recorded
        .get(range.clone())
        .unwrap_or(&[])
        .iter()
        .filter_map(|m| match m {
            DomMutation::CreateTextNode { handle, text } => Some((handle.clone(), text.clone())),
            _ => None,
        })
        .collect();
    // 先收集重写决策（index → 新记录），文本路径逆序赋位。
    let mut text_slots: Vec<(usize, DomMutation)> = Vec::new();
    for i in range.clone() {
        let Some(m) = recorded.get(i) else { break };
        let DomMutation::AppendChild {
            parent_selector,
            child_handle,
        } = m
        else {
            continue;
        };
        // 守卫②：parent 须是脚本元素的祖先（含直父）——沿 parent 链上行判定。
        let parent_hit = doc
            .query_selector(root, zero_dom::trim_ascii_ws(parent_selector))
            .is_some_and(|p| {
                let mut anc = doc.parent_node(script_id);
                while let Some(a) = anc {
                    if a == p {
                        return true;
                    }
                    anc = doc.parent_node(a);
                }
                false
            });
        if !parent_hit {
            continue;
        }
        if let Some(text) = text_of.get(child_handle) {
            text_slots.push((
                i,
                DomMutation::InsertAdjacentText {
                    selector: script_sel.clone(),
                    position: "afterend".into(),
                    text: text.clone(),
                },
            ));
            continue;
        }
        // 元素子：下一个**元素**兄弟（跳过文本/注释）为锚。
        let mut sib = doc.next_sibling(script_id);
        let anchor = loop {
            match sib {
                Some(s) => {
                    if matches!(doc.get(s).map(|n| &n.kind), Some(zero_dom::NodeKind::Element(_))) {
                        break Some(s);
                    }
                    sib = doc.next_sibling(s);
                }
                None => break None,
            }
        };
        let Some(anchor_id) = anchor else { continue };
        let Some(ref_selector) = zero_engine::unique_selector_for_node(&doc, anchor_id) else {
            continue;
        };
        text_slots.push((
            i,
            DomMutation::InsertBefore {
                parent_selector: parent_selector.clone(),
                child_handle: child_handle.clone(),
                ref_selector,
            },
        ));
    }
    // 文本重写逆序赋位（afterend 连续插入的序补偿）；元素路径无序敏感，统一处理无碍。
    for ((slot, _), (_, m)) in text_slots.iter().zip(text_slots.iter().rev()) {
        recorded[*slot] = m.clone();
    }
}

/// R4344：结构性 mutation 判定——解析期视图增量刷新的门（仅结构性变更值得
/// 重建视图；属性/表单类 delta 维持旧自洽面，缩小行为变更半径）。
fn is_structural_mutation(m: &DomMutation) -> bool {
    matches!(
        m,
        DomMutation::AppendChild { .. }
            | DomMutation::AppendChildByHandle { .. }
            | DomMutation::InsertBefore { .. }
            | DomMutation::InsertBeforeByHandle { .. }
            | DomMutation::InsertBeforeByHandleHandle { .. }
            | DomMutation::InsertAdjacentHtml { .. }
            | DomMutation::InsertAdjacentText { .. }
            | DomMutation::InsertAdjacentElement { .. }
            | DomMutation::Remove { .. }
            | DomMutation::SetInnerHtml { .. }
            | DomMutation::SetOuterHtml { .. }
    )
}

/// R4321：读 shim `document.write` 写缓冲（`js_dom_shim/part06.js`：write() 只缓冲、
/// close() 才应用——S16 简化语义）。非空字符串 = 有未 close 的 write → 返回并清空；
/// null/undefined（未 write 或 close 已消费）→ None。
fn take_document_write_buffer(sandbox: &mut dyn zero_script_sandbox::Sandbox) -> Option<String> {
    let result = sandbox
        .execute_json("(typeof __zwDocWriteBuffer === 'string') ? __zwDocWriteBuffer : null")
        .ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&result.value).ok()?;
    match parsed {
        serde_json::Value::String(s) if !s.is_empty() => {
            let _ = sandbox.execute("globalThis.__zwDocWriteBuffer = '';");
            Some(s)
        }
        _ => None,
    }
}

/// R4321：把 (脚本全序号, 缓冲) 落地为 `SetOuterHtml`（脚本元素整体替换为缓冲内容，
/// ≈ 解析流「write 插入于脚本自身位置」语义）。倒序应用——移除后置脚本不影响前置
/// 脚本的结构选择器（nth 路径只数前驱兄弟）。单个脚本落地失败仅告警跳过，不阻断
/// 其余（harness 兜底，数据完整性不受影响）。
///
/// 实体单次解码：解析流语义下缓冲中的字符引用（`&#x2685;` 等——script 源文本不解码，
/// write 进解析流才解码）恰好解码一次。SetOuterHtml 的片段路径（含 `<`）经
/// html5ever 解析自带解码；**纯文本快路径**（无 `<` → create_text_node）不解码——
/// 此处对无 `<` 缓冲先行解码（经同一 html5ever），保证两路均为单次解码。
fn apply_document_write_flushes(html: String, hits: &[(usize, String)]) -> String {
    let doc = zero_dom::parse_html(&html);
    let script_ids = doc.get_elements_by_tag_name("script");
    let mut mutations = Vec::new();
    for (ord, buf) in hits.iter().rev() {
        let Some(&script_id) = script_ids.get(*ord) else {
            eprintln!("  [reftest JS] document.write flush: script ord {ord} not found, skipped");
            continue;
        };
        let Some(selector) = zero_engine::unique_selector_for_node(&doc, script_id) else {
            eprintln!("  [reftest JS] document.write flush: no unique selector for script ord {ord}, skipped");
            continue;
        };
        let decoded = if buf.contains('<') {
            buf.clone()
        } else {
            let frag = zero_dom::parse_html_fragment(buf, "http://www.w3.org/1999/xhtml", "body");
            frag.text_content(frag.root()).unwrap_or_else(|| buf.clone())
        };
        mutations.push(DomMutation::SetOuterHtml {
            selector,
            html: decoded,
        });
    }
    if mutations.is_empty() {
        return html;
    }
    match apply_mutations_to_html(&html, &mutations) {
        Ok(new_html) => new_html,
        Err(e) => {
            eprintln!("  [reftest JS] document.write flush warning: {e}");
            html
        }
    }
}

/// 提取 `<body>`/`<frameset>`/`<html>` 上 `onload` 属性的 handler 体（JS 源码）。
///
/// 这些属性在 `load` 事件触发时由浏览器编译为函数体执行；reftest 直接把属性值当
/// JS 源码运行（与 browser 侧 shim 语义一致：shim 的 setTimeout 会立即经 microtask 跑）。
pub(super) fn extract_onload_handlers(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut out = Vec::new();
    for tag in ["body", "frameset", "html"] {
        for id in doc.get_elements_by_tag_name(tag) {
            if let Some(h) = doc.get_attribute(id, "onload")
                && !h.trim().is_empty()
            {
                out.push(h);
            }
        }
    }
    out
}

/// 解析并读取外链脚本（reftest 离线运行，失败返回 `Err` 由调用方跳过、不阻塞）。
///
/// WPT URL 语义（R546/R551 谱系补齐，2026-08-07）：
/// - 以 `/` 开头的 src（如 `/common/reftest-wait.js`）是**套件根相对 URL**，
///   相对 wpt_root（wpt-data 根）解析——不能按文件系统绝对路径处理；
/// - src 可能带 query（如 `foo.js?x=1`，少见），剥离后再加载。
pub(super) fn fetch_external_script(
    src: &str,
    base_dir: Option<&Path>,
    wpt_root: Option<&Path>,
) -> Result<String, String> {
    let src = src.split('?').next().unwrap_or(src);
    let resolved = if src.starts_with('/') {
        match wpt_root {
            Some(root) => root.join(src.trim_start_matches('/')),
            None => {
                return Err(format!(
                    "absolute WPT path {src} but wpt_root not configured (set ReftestConfig::wpt_root)"
                ));
            }
        }
    } else if let Some(base) = base_dir {
        base.join(src)
    } else {
        std::path::Path::new(src).to_path_buf()
    };
    std::fs::read_to_string(&resolved).map_err(|e| format!("{}: {e}", resolved.display()))
}
