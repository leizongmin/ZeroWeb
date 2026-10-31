//! HTML/CSS 资源提取族（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `extract_stylesheet_hrefs` / `extract_page_scripts` / `extract_img_resources` /
//! `extract_font_faces` / `extract_import_urls` / `extract_css_image_urls` /
//! `extract_html_style_text` 等纯解析函数（复用 `zero_dom` / `zero_css_parser`，
//! engine 不直接耦合网络）。`mod.rs` 经 `pub use extract::*;` 复用 + 保
//! `zero_engine::*` 外部路径；私有 helper 仅供族内，`strip_cdata` 例外（`pub`，
//! 供 `mod.rs::inject_print_page_dividers` 复用）。

use std::cmp::Ordering;
use std::collections::HashSet;

/// 提取 HTML 中所有 `<link rel="stylesheet" href="...">` 的 href 原始值。
///
/// 用于 URL 导航路径下外链样式表的加载（goal doc P1 缺口「外部样式表加载缺失」）：
/// `collect_stylesheets` 仅收集调用方传入 CSS 与文档内 `<style>`，不抓取 `<link>`。
/// 本函数复用 `zero_dom` 解析（DOM 精确，区别于脆弱的正则扫描），返回原始 href
/// 字符串（可能是相对路径）；URL 解析与网络抓取由调用方（webview 层，持有 base URL
/// 与 http client）负责，保持 engine 不直接耦合网络。
///
/// - `rel` 以空白拆分后任一 token 等于 `stylesheet`（大小写不敏感）即匹配，
///   覆盖 `rel="stylesheet"` 与 `rel="stylesheet preload"` 等写法。
/// - 空 href 与 `rel` 不含 stylesheet 的 link（如 icon / preload 非 stylesheet）被忽略。
pub fn extract_stylesheet_hrefs(html: &str) -> Vec<String> {
    extract_stylesheet_links(html)
        .into_iter()
        .map(|(href, _media)| href)
        .collect()
}

/// [`extract_stylesheet_hrefs`] 的 media 属性携带变体（s161）：返回 `(href 原始值,
/// media 属性原文)`（属性缺失 = 空串）。供 webview 外链样式表加载路径做 media 门
/// （media 不匹配环境 → 表仍取回语义上前端不并入级联——spec processing the media
/// attribute）。rel/href 门与 [`extract_stylesheet_hrefs`] 完全一致。
/// https://html.spec.whatwg.org/multipage/semantics.html#processing-the-media-attribute
pub fn extract_stylesheet_links(html: &str) -> Vec<(String, String)> {
    let doc = zero_dom::parse_html(html);
    let mut links = Vec::new();
    for link_id in doc.get_elements_by_tag_name("link") {
        let rel = doc.get_attribute(link_id, "rel").unwrap_or_default();
        let is_stylesheet = rel.split_whitespace().any(|t| t.eq_ignore_ascii_case("stylesheet"));
        if !is_stylesheet {
            continue;
        }
        if let Some(href) = doc.get_attribute(link_id, "href") {
            let href = href.trim();
            if !href.is_empty() {
                let media = doc.get_attribute(link_id, "media").unwrap_or_default();
                links.push((href.to_string(), media));
            }
        }
    }
    links
}

/// 页面脚本来源：内联文本或 `<script src>` 原始值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageScript {
    /// 内联经典脚本。
    Inline(String),
    /// 外链经典脚本 `src`（可能为相对 URL）。
    External(String),
    /// 内联 ES module。
    InlineModule(String),
    /// 外链 ES module `src`（可能为相对 URL）；`integrity` 为 SRI 属性原文（s107——
    /// 顶层模块取回后校验面；classic 外链脚本未建模，FIXME 挂账）。
    ExternalModule {
        /// `src` 属性原文（SVG script 为 href/xlink:href）。
        src: String,
        /// `integrity` 属性原文（空格分隔 integrity metadata，空属性归 None）。
        integrity: Option<String>,
    },
}

/// HTML「ASCII whitespace」剥离（TAB/LF/FF/CR/SPACE）——s93（classic 脚本属性面）：spec 剥离
/// 仅此五种，不含 Unicode White_Space 充数；旧 `str::trim` 连 \x0B/\x85/\xA0 等一起剥，
/// `type="text/javascript\x0B"` 被误判为 JS 执行（WPT script-type-whitespace 5F 红因）。
/// https://html.spec.whatwg.org/multipage/infrastructure.html#ascii-whitespace
fn trim_ascii_html_ws(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, '\t' | '\n' | '\x0C' | '\r' | ' '))
}

/// HTML §scripting languages：JavaScript MIME type essence 集合全表（16 项——含
/// legacy text/javascript1.0-1.5 / text/jscript / text/livescript / x- 变体）。
/// essence match = 值剥首尾 ASCII whitespace 后 ASCII case-insensitive **等值**于
/// 集合成员，无参数语法：`text/javascript;charset=UTF-8` 不是 essence match →
/// data block 不执行（语料 script-type-and-language-with-params 判据；MIME 参数
/// 剥离属 mimesniff 解析，spec 此处是字符串比对）。
/// https://html.spec.whatwg.org/multipage/scripting.html#javascript-mime-type
const JS_MIME_ESSENCE_TYPES: [&str; 16] = [
    "application/ecmascript",
    "application/javascript",
    "application/x-ecmascript",
    "application/x-javascript",
    "text/ecmascript",
    "text/javascript",
    "text/javascript1.0",
    "text/javascript1.1",
    "text/javascript1.2",
    "text/javascript1.3",
    "text/javascript1.4",
    "text/javascript1.5",
    "text/jscript",
    "text/livescript",
    "text/x-ecmascript",
    "text/x-javascript",
];

/// 脚本 type 判 JS（HTML prepare-the-script-element：type 字面空串或缺省 → type
/// string = "text/javascript" → classic；否则 type string essence match → classic，
/// module 形态 → module（既有剥白口径不动）；余为 data block 不执行。s113：旧
/// `ends_with("javascript")` 子串放行 bare `javascript` / `text/javascript1.8` /
/// `\0` 沾染非法 type（语料 script-type-and-language-js 解析插入面 1 缺口），
/// 16 集合收窄后与动态面 `_zwScriptTypeClassify` 同表）。
/// pub（s110）：runner 投递面 `inline_local_scripts` 对非 JS type 外链单元按同判据整段
/// 透传（data-inline 改写剥 src，mutation 重触发面须保 src 语义）。
/// https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
pub fn script_type_is_javascript(type_attr: Option<&str>) -> bool {
    match type_attr {
        None => true,
        // 字面空串（剥白区分轨：type=" " 推导串为原值，剥后空串不在集合 → data block，
        // 语料 testTypeShouldNotRun(" ")）。
        Some("") => true,
        Some(t) => {
            JS_MIME_ESSENCE_TYPES
                .iter()
                .any(|m| trim_ascii_html_ws(t).eq_ignore_ascii_case(m))
                || script_is_module(Some(t))
        }
    }
}

/// 脚本 type 判 module（同上步骤：剥首尾 ASCII whitespace 后 ASCII case-insensitive 匹配 `module`）。
fn script_is_module(type_attr: Option<&str>) -> bool {
    type_attr
        .map(|t| trim_ascii_html_ws(t).eq_ignore_ascii_case("module"))
        .unwrap_or(false)
}

/// `nomodule` 门（s94，nomodule face）：元素带 `nomodule` 内容属性且类型为 classic → prepare
/// 中止不执行（支持 module 脚本的 UA 压制 nomodule classic 脚本，含内联/外链/动态插入）；
/// module 脚本不受 nomodule 影响（语料 module/nomodule-attribute：module+nomodule 照跑）。
/// bool 属性按内容属性在场判定，值无意义。
/// https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
fn script_suppressed_by_nomodule(nomodule_present: bool, is_module: bool) -> bool {
    nomodule_present && !is_module
}

/// `for`/`event` 门（s94，for-event face）：classic 脚本**同时**带 `for` 与 `event` 内容属性
/// 时，两值各剥首尾 ASCII whitespace 后 `for` 须 ASCII case-insensitive 匹配 `window`、
/// `event` 须 ASCII case-insensitive 匹配 `onload` 或 `onload()`，否则中止不执行；
/// 任一属性缺失则本步骤跳过照常执行（语料单属性臂 true）。NBSP 等非 ASCII 空白不剥
///（语料 `for="window\u{00A0}"` 臂 false）。
/// https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
fn script_blocked_by_for_event(for_attr: Option<&str>, event_attr: Option<&str>) -> bool {
    let (Some(for_v), Some(event_v)) = (for_attr, event_attr) else {
        return false;
    };
    let for_v = trim_ascii_html_ws(for_v);
    let event_v = trim_ascii_html_ws(event_v);
    !(for_v.eq_ignore_ascii_case("window")
        && (event_v.eq_ignore_ascii_case("onload") || event_v.eq_ignore_ascii_case("onload()")))
}

/// 判断节点是否位于 `<template>` 子树内（R328——template 内容 inert，脚本不执行）。
fn is_inside_template(doc: &zero_dom::Document, mut id: zero_dom::NodeId) -> bool {
    while let Some(parent) = doc.parent_node(id) {
        if doc.is_template_element(parent) {
            return true;
        }
        id = parent;
    }
    false
}

/// 按文档顺序提取 `<script>` 内联文本与 `src`。
pub fn extract_page_scripts(html: &str) -> Vec<PageScript> {
    extract_page_scripts_indexed(html).into_iter().map(|(s, _)| s).collect()
}

/// 与 [`extract_page_scripts`] 相同，但每个条目附带该 `<script>` 在**所有** `<script>` 元素中的序号
///（含非 JS 类型脚本，如 `<script type="application/json">`）。
///
/// 该序号与 shim `document.getElementsByTagName('script')` 的返回序对齐——后者按文档序返回
/// 全部 `<script>` 元素（不论 type），故宿主在执行 classic 脚本前后据此序号设/清
/// `document.currentScript`（HTML §4.11.3.1：classic 脚本执行期间 currentScript 指向自身元素）。
/// 序号在过滤**之前**递增（每遇一个 `<script>` 即 +1），确保与 `getElementsByTagName('script')` 一一对应。
pub fn extract_page_scripts_indexed(html: &str) -> Vec<(PageScript, usize)> {
    let doc = zero_dom::parse_html(html);
    let mut scripts = Vec::new();
    let mut script_ids = Vec::new();
    for (this_idx, script_id) in doc.get_elements_by_tag_name("script").into_iter().enumerate() {
        let type_attr = doc.get_attribute(script_id, "type");
        if !script_type_is_javascript(type_attr.as_deref()) {
            continue;
        }
        // R328（js-dom M4）：`<template>` 内容是 inert DocumentFragment（spec HTML
        // §the-template-element——内容文档 "scripts are not executed"）。html5ever 的
        // `get_template_contents` 暂返 template 元素自身，脚本留在文档树内；
        // `get_elements_by_tag_name` 全树 DFS 不跳过 template 子树，此前把 template
        // 内联脚本当页面脚本执行——WPT remove-next-sibling-during-replace-with 的
        // template 脚本在测试体前运行、抢先把 `<b>` 摘除（测试体
        // `container.querySelector('script')` 因 template 内 script 的 remove mutation
        // 与查询叠加而 miss → "Cannot read properties of null"）。序号计数不受影响：
        // this_idx 在过滤前递增，与 `getElementsByTagName('script')` 一一对应。
        // https://html.spec.whatwg.org/multipage/scripting.html#the-template-element
        if is_inside_template(&doc, script_id) {
            continue;
        }
        let is_module = script_is_module(type_attr.as_deref());
        // s94（nomodule / for-event 双门）：HTML prepare 步骤——nomodule 压制 classic
        //（module 不受影响）；for/event 双属性在场的 window/onload 门（classic 限定）。
        // 两门在提取期跳过 = 不投递执行；序号计数不受影响（this_idx 过滤前已递增，
        // 与 shim getElementsByTagName('script') 对齐不变）。
        // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
        if script_suppressed_by_nomodule(doc.get_attribute(script_id, "nomodule").is_some(), is_module) {
            continue;
        }
        if !is_module
            && script_blocked_by_for_event(
                doc.get_attribute(script_id, "for").as_deref(),
                doc.get_attribute(script_id, "event").as_deref(),
            )
        {
            continue;
        }
        // R5005（html-syntax-compat M4 残差重归因）：SVG `<script href=…>`（SVG2 外部
        // 脚本引用——HTML script 用 src，SVG script 用 href/xlink:href）此前只查 src →
        // WPT unclosed-svg-script 的 self-closing 变体不执行。仅 SVG 命名空间元素
        // 降级查 href（HTML script 无 href 属性，不放宽无效标记形态）。
        let src_attr = if doc
            .get(script_id)
            .map(|n| match n {
                zero_dom::NodeData {
                    kind: zero_dom::NodeKind::Element(e),
                    ..
                } => e.namespace() == "http://www.w3.org/2000/svg",
                _ => false,
            })
            .unwrap_or(false)
        {
            doc.get_attribute(script_id, "href")
                .or_else(|| doc.get_attribute(script_id, "src"))
        } else {
            doc.get_attribute(script_id, "src")
        };
        if let Some(src) = src_attr {
            let src = src.trim();
            if src.is_empty() {
                // s106（SVG script href 空串面）：外链形态成立即不落内联臂——SVG
                // script 带 href/xlink:href（含空串）为外链引用，空 URL 取回失败不
                // 执行内联体（WPT execution-timing/137：assert_unreached 须不可达）。
                // HTML script src="" 维持旧内联回落（无语料判据，不扩大）。
                // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
                if doc.get(script_id).is_some_and(|n| {
                    matches!(
                        n,
                        zero_dom::NodeData {
                            kind: zero_dom::NodeKind::Element(e),
                            ..
                        } if e.namespace() == "http://www.w3.org/2000/svg"
                    )
                }) {
                    continue;
                }
            } else {
                if is_module {
                    // s107（SRI）：integrity 属性随外链模块携带——webview 取回后按
                    // integrity metadata 校验（mismatch → 网络错误语义，元素 fire error
                    // + 体不执行）。classic 外链脚本未建模（无语料判据，FIXME）。
                    // https://html.spec.whatwg.org/multipage/semantics.html#integrity-metadata-attribute
                    let integrity = doc
                        .get_attribute(script_id, "integrity")
                        .map(|v| v.trim().to_string())
                        .filter(|v| !v.is_empty());
                    scripts.push((
                        PageScript::ExternalModule {
                            src: src.to_string(),
                            integrity,
                        },
                        this_idx,
                    ));
                } else {
                    scripts.push((PageScript::External(src.to_string()), this_idx));
                }
                script_ids.push(script_id);
                continue;
            }
        }
        // s106（脚本源 = child text content）：spec prepare-the-script-element 的
        // source text = script 元素**直接子**文本节点拼接（child text content），非
        // textContent 递归语义——SVG script（foreign content 按 markup 解析）可嵌套
        // script 子元素，递归版把嵌套脚本文本吞进外层源致 SyntaxError（138/139/143）。
        // HTML script 内容为 raw text 仅文本子节点，二者恒等（零行为变化）。
        // https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
        if let Some(raw) = doc.child_text_content(script_id) {
            // XHTML 脚本常以 `<![CDATA[ ... ]]>` 包裹；html5ever 按 HTML 模式解析会把 CDATA
            // 标记作为文本保留。若不剥离，传给 JS 引擎会触发 `SyntaxError: Unexpected token '<'`
            // 致整个脚本失效（函数未定义 → onload 回调再抛 ReferenceError）。CSS21 测试套件
            // 大量 .xht 用 CDATA 包裹脚本（insert-* 动态簇等）。兼容两种写法：裸 `<![CDATA[`
            //（占绝大多数）与 `//<![CDATA[`（JS 注释隐藏，HTML/XHTML 双兼容）。
            let code = strip_script_cdata(raw.trim()).trim();
            if !code.is_empty() {
                if is_module {
                    scripts.push((PageScript::InlineModule(code.to_string()), this_idx));
                } else {
                    scripts.push((PageScript::Inline(code.to_string()), this_idx));
                }
                script_ids.push(script_id);
            }
        }
    }
    // s106（SVG script 嵌套 end-tag 序）：foreign content 按 markup 解析，SVG script
    // 可嵌套 script 子元素（WPT execution-timing 138/139/140）——parser-inserted 脚本
    // 于自身 end tag 完成时执行，子脚本 end tag 先闭合先执行。提取为 pre-order（父先
    // 子后），按「pre-order 先行者是后行者祖先 → 后行者先行」重排为 end-tag 序（区间
    // 序，传递一致）。HTML script 内容为 raw text 无元素子节点，无嵌套时序不变；
    // this_idx 仍为全体 script 元素 pre-order 序（与 shim getElementsByTagName('script')
    // 对齐），执行序与序号解耦。
    // https://html.spec.whatwg.org/multipage/syntax.html#the-rules-for-parsing-tokens-in-foreign-content
    let mut entries: Vec<(zero_dom::NodeId, usize, PageScript)> = scripts
        .into_iter()
        .zip(script_ids)
        .map(|((script, idx), id)| (id, idx, script))
        .collect();
    entries.sort_by(|a, b| match a.1.cmp(&b.1) {
        Ordering::Equal => Ordering::Equal,
        Ordering::Less => {
            if is_ancestor(&doc, a.0, b.0) {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        Ordering::Greater => {
            if is_ancestor(&doc, b.0, a.0) {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
    });
    entries.into_iter().map(|(_, idx, script)| (script, idx)).collect()
}

/// 判 `anc` 是否 `desc` 的祖先（沿父链上行）——脚本 end-tag 排序用。
fn is_ancestor(doc: &zero_dom::Document, anc: zero_dom::NodeId, desc: zero_dom::NodeId) -> bool {
    let mut current = doc.parent_node(desc);
    while let Some(node) = current {
        if node == anc {
            return true;
        }
        current = doc.parent_node(node);
    }
    false
}

/// 按文档顺序提取 `<script type="importmap">` 的内联 JSON 文本。
///
/// <https://html.spec.whatwg.org/multipage/webappapis.html#import-maps> —— import map
/// 经内联 script 元素交付（type 属性为 "importmap"，子文本内容为 JSON 表示）；
/// 带 `src` 属性的不合规元素忽略；`<template>` 子树内 inert（R328 同规则）跳过。
/// 多张 map 全部返回、按文档顺序；JSON 合法性由调用方（`ImportMap::parse`）判定。
pub fn extract_import_map_json(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut maps = Vec::new();
    for script_id in doc.get_elements_by_tag_name("script") {
        let Some(type_attr) = doc.get_attribute(script_id, "type") else {
            continue;
        };
        if !type_attr.trim().eq_ignore_ascii_case("importmap") {
            continue;
        }
        if doc.get_attribute(script_id, "src").is_some() {
            continue;
        }
        if is_inside_template(&doc, script_id) {
            continue;
        }
        if let Some(raw) = doc.text_content(script_id) {
            let json = raw.trim();
            if !json.is_empty() {
                maps.push(json.to_string());
            }
        }
    }
    maps
}

/// 提取 HTML 中所有 `<img src="...">` 的 src 原始值。
///
/// 用于 URL 导航路径下图片子资源的加载（goal doc P1 缺口「图片子资源 / ImageCache
/// 未贯通」）。与 `extract_stylesheet_hrefs` 同模式：复用 `zero_dom` 解析（DOM 精确），
/// 返回原始 src 字符串（可能相对）；URL 解析、抓取与解码由调用方（webview 层）负责。
/// 空 src 过滤；`data:` URI 原样返回（由调用方识别处理）。
pub fn extract_img_srcs(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut srcs = Vec::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        // R4162：src → srcset → picture/source 有效图源（replaced-element-012）。
        if let Some(src) = effective_img_src(&doc, img_id) {
            let src = src.trim();
            if !src.is_empty() {
                srcs.push(src.to_string());
            }
        }
        // s70：srcset 全候选预抓取（spec §4.8.4.3.7 select an image source——选中面
        // 随 DPR/sizes 动态，宿主尺寸快照须覆盖每个可能选中 URL；over-collection
        // 无害）。键 = 原始属性串 token，与 shim `__zw_get_image_size` 查询同串。
        if let Some(srcset) = doc.get_attribute(img_id, "srcset") {
            push_srcset_candidates(&srcset, &mut srcs);
        }
    }
    // s70：picture `source` srcset 候选（picture 选中面的 source 分支）。
    for source_id in doc.get_elements_by_tag_name("source") {
        if let Some(srcset) = doc.get_attribute(source_id, "srcset") {
            push_srcset_candidates(&srcset, &mut srcs);
        }
    }
    // R5275（html-semantics M2 切片 68）：`<template>` contents fragment 内声明的
    // `<img>` 也进抓取清单——fragment 不在文档树（zero_dom 解析期侧表
    // template_contents_map，template.children 恒空），get_elements_by_tag_name
    // 不达；naturalWidth-naturalHeight-width-height 族全部图源声明在模板内，
    // 漏抓 → host 尺寸快照空 → shim 静态 settle 扫描全落 broken。
    for template_id in doc.get_elements_by_tag_name("template") {
        let Some(frag_id) = doc.template_contents(template_id) else {
            continue;
        };
        collect_fragment_imgs(&doc, frag_id, &mut srcs);
    }
    // R34xx：SVG `<image>` 元素源（href/xlink:href——2d.pattern.svgimage.zeroheight 的
    // `<svg><image xlink:href="/images/red-zeroheight.svg">` 须进尺寸/像素缓存，
    // createPattern 才能判 0 尺寸 → null）。
    for img_id in doc.get_elements_by_tag_name("image") {
        let src = doc
            .get_attribute(img_id, "href")
            .or_else(|| doc.get_attribute(img_id, "xlink:href"))
            .or_else(|| doc.get_attribute(img_id, "src"));
        if let Some(src) = src {
            let src = src.trim();
            if !src.is_empty() {
                srcs.push(src.to_string());
            }
        }
    }
    srcs
}

/// `<img>` 子资源（含 lazy 标记）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImgResource {
    /// `src` 原始值。
    pub src: String,
    /// `loading=lazy`。
    pub lazy: bool,
    /// HTML `fetchpriority` 提示（`high` / `low` / `auto`）。
    pub fetchpriority: Option<String>,
}

/// 需要获取以提交基础加载状态的媒体/资源元素类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaResourceElementKind {
    /// `<audio src>`.
    Audio,
    /// `<video src>`.
    Video,
    /// `<source src>`（仅限 audio/video 子元素）。
    Source,
    /// `<track src>`.
    Track,
}

impl MediaResourceElementKind {
    /// 对应的 HTML 标签名。
    pub const fn tag_name(self) -> &'static str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Source => "source",
            Self::Track => "track",
        }
    }
}

/// 媒体/资源元素及其待获取 URL。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaResourceElement {
    /// 元素类型。
    pub kind: MediaResourceElementKind,
    /// `src` 原始值。
    pub src: String,
}

/// 提取 `audio`、`video`、其直属 `source` 以及 `track` 的非空 `src`。
///
/// 本层只识别文档关系和 URL 字符串；候选选择、获取与状态提交由 WebView
/// 加载协调器负责。
/// https://html.spec.whatwg.org/multipage/media.html#concept-media-load-resource
pub fn extract_media_resources(html: &str) -> Vec<MediaResourceElement> {
    let doc = zero_dom::parse_html(html);
    let mut out = Vec::new();
    for id in doc.get_elements_by_tag_names(&["audio", "video", "source", "track"]) {
        let Some(node) = doc.get(id) else {
            continue;
        };
        let zero_dom::NodeKind::Element(element) = &node.kind else {
            continue;
        };
        let kind = match element.local_name() {
            "audio" => MediaResourceElementKind::Audio,
            "video" => MediaResourceElementKind::Video,
            "track" => MediaResourceElementKind::Track,
            "source" => {
                let Some(parent) = doc.parent_node(id).and_then(|parent| doc.get(parent)) else {
                    continue;
                };
                let zero_dom::NodeKind::Element(parent) = &parent.kind else {
                    continue;
                };
                if !matches!(parent.local_name(), "audio" | "video") {
                    continue;
                }
                if parent.get_attribute("src").is_some_and(|src| !src.trim().is_empty()) {
                    continue;
                }
                MediaResourceElementKind::Source
            }
            _ => continue,
        };
        let Some(src) = doc.get_attribute(id, "src") else {
            continue;
        };
        let src = src.trim();
        if !src.is_empty() {
            out.push(MediaResourceElement {
                kind,
                src: src.to_string(),
            });
        }
    }
    out
}

/// 从 `srcset` 属性取首个候选 URL（R2419：srcset-only `<img>` 回退 src）。
///
/// `srcset="a.jpg 1x, b.jpg 2x"` → `a.jpg`。仅取首候选首 token（最小正确性修：
/// srcset-only 图无 src 时用首 URL 作 effective src 抓取+渲染；多分辨率 DPR 选源为后续）。
pub fn srcset_first_url(srcset: &str) -> Option<String> {
    let first = srcset.split(',').next()?.trim();
    let url = first.split_whitespace().next()?;
    (!url.is_empty()).then(|| url.to_string())
}

/// R5275：template contents fragment 递归收集 `<img>`（fragment 不在文档树，
/// `get_elements_by_tag_name` 不达；嵌套模板/任意深度均覆盖）。s70：img 附带
/// srcset 候选、`source` srcset 候选一并收集（同文档树 s70 面）。
fn collect_fragment_imgs(doc: &zero_dom::Document, id: zero_dom::NodeId, srcs: &mut Vec<String>) {
    for child in doc.child_nodes(id) {
        let is_img = doc
            .get(child)
            .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "img"));
        let is_source = doc
            .get(child)
            .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "source"));
        if is_img {
            if let Some(src) = effective_img_src(doc, child) {
                let src = src.trim();
                if !src.is_empty() {
                    srcs.push(src.to_string());
                }
            }
            if let Some(srcset) = doc.get_attribute(child, "srcset") {
                push_srcset_candidates(&srcset, srcs);
            }
        } else if is_source && let Some(srcset) = doc.get_attribute(child, "srcset") {
            push_srcset_candidates(&srcset, srcs);
        }
        collect_fragment_imgs(doc, child, srcs);
    }
}

/// s70：srcset 属性候选 URL 收集（spec parse-a-srcset 的 URL 提取采集近似——
/// 逗号分组切分、每组取首个非空白 token、剥除尾部逗号（裸 URL 候选
/// `"a.png,"` 形态）；描述符 token（`1x`/`2w`）不构成分组首 token 的情形仅
/// 出现在逗号前无 URL 的病态输入，不入清单即可。over-collection 无害：
/// 宿主多抓一图只多一条尺寸快照，选中面由 shim 动态判定。
fn push_srcset_candidates(srcset: &str, srcs: &mut Vec<String>) {
    for group in srcset.split(',') {
        let Some(token) = group.trim().split_ascii_whitespace().next() else {
            continue;
        };
        let url = token.trim_end_matches(',');
        if !url.is_empty() {
            srcs.push(url.to_string());
        }
    }
}

/// `<img>` 的有效图源 URL（HTML §4.8.5 picture/source 选择）：
/// src → srcset 首 URL →（位于 `<picture>` 内且前两者皆缺）首个 `<source srcset>` 首 URL。
/// R4162：replaced-element-012——picture 内 srcset-only img（source 提供资源）此前
/// 三层全空 → 无解码尺寸 + paint 无图元（白块）。不评估 media/type 条件（match-all，
/// 覆盖 WPT 常见形态；条件评估为后续深化）。
pub fn effective_img_src(doc: &zero_dom::Document, img_id: zero_dom::NodeId) -> Option<String> {
    let own = doc
        .get_attribute(img_id, "src")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| doc.get_attribute(img_id, "srcset").and_then(|s| srcset_first_url(&s)));
    if own.is_some() {
        return own;
    }
    let parent = doc.parent_node(img_id)?;
    let is_picture = doc
        .get(parent)
        .is_some_and(|n| matches!(&n.kind, zero_dom::NodeKind::Element(e) if e.local_name() == "picture"));
    if !is_picture {
        return None;
    }
    doc.child_nodes(parent)
        .iter()
        .filter_map(|c| doc.get(*c))
        .find_map(|n| match &n.kind {
            zero_dom::NodeKind::Element(e) if e.local_name() == "source" => {
                e.get_attribute("srcset").and_then(|s| srcset_first_url(&s))
            }
            _ => None,
        })
}

/// 提取 HTML 中所有 `<img>` 的 src 与 lazy 属性。
pub fn extract_img_resources(html: &str) -> Vec<ImgResource> {
    let doc = zero_dom::parse_html(html);
    let mut out = Vec::new();
    for img_id in doc.get_elements_by_tag_name("img") {
        // R2419：src 缺失/空时回退到 srcset 首 URL（srcset-only `<img>` 正确性修）。
        let src = doc
            .get_attribute(img_id, "src")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| doc.get_attribute(img_id, "srcset").and_then(|s| srcset_first_url(&s)));
        let Some(src) = src else {
            continue;
        };
        let lazy = doc
            .get_attribute(img_id, "loading")
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("lazy"));
        out.push(ImgResource {
            src,
            lazy,
            fetchpriority: doc.get_attribute(img_id, "fetchpriority"),
        });
    }
    out
}

/// 从 CSS 文本提取所有 `@font-face` 规则的 `(family, url_sources, weight, is_italic)` 列表
/// （保留 family + weight + is_italic）。
///
/// **保留 family**（`@font-face` 的 `font-family` 描述符值）与 **weight**（`font-weight`
/// 描述符解析为绝对权重 100-900，`None` = 未指定/相对，视为 normal/400）与 **is_italic**
/// （`font-style` 描述符为 `italic`/`oblique` → true，否则 false），供生产 async 加载路径
/// 按声明族名注册字体别名（`FontLoader::register_family_alias`），并按 weight 构
/// `{family}:700` 粗体键（R2417 font-weight matching）、按 is_italic 构 `{family}:italic`
/// italic 键（R2493 font-style matching——painter `resolve_font_id` 对 `font-weight≥600`
/// 查 `{family}:700`、对 `font-style:italic/oblique` 查 `{family}:italic`）。`sources` 为
/// css-parser 解析出的 url() 项（已去 `url()` 包裹与引号，按出现顺序）；family 已去引号。
///
/// `data:` / `local()` 的过滤由抓取层（`AsyncPageLoad::begin_font_fetch`）处理，本函数仅做
/// 透传解析。解析失败或无 @font-face 规则返回空 Vec。
/// 解析后的 `@font-face` `(family, sources, weight, italic, feature settings)`。
pub type ExtractedFontFace = (
    String,
    Vec<String>,
    Option<u16>,
    bool,
    Option<f32>,
    Option<f32>,
    zero_css_parser::values::FontFeatureSettingsValue,
    zero_css_parser::values::FontVariationSettingsValue,
    Vec<(u32, u32)>,
);

/// 递归展开样式表顶层规则中**条件成立的** @media / @supports 块（CSS Conditional 3
/// §contents-of：条件规则体内的 @font-face/@keyframes/@counter-style 等语句级 at-rule
/// 按外层条件生效）。
///
/// 返回克隆展平后的规则序列（FontFace/Keyframes/CounterStyle 等 body 规则体量小，
/// 注册路径低频，克隆成本可忽略）。@media 用 `media_ctx` 求值；@supports 委托
/// style-system 的 [`zero_style_system::evaluate_supports_condition`]（属性支持表 +
/// R3879 selector() 门控 + R3880 font 谓词的单源实现）。嵌套条件块递归。
///
/// driving: at-supports-content-002/003/004、at-media-content-002/003/004——此前
/// `extract_font_faces` / `register_from_stylesheets` / `register_counter_styles`
/// 只扫顶层规则，条件块内的语句级 at-rule 全部丢失。
pub fn flatten_conditional_rules(
    stylesheets: &[zero_css_parser::Stylesheet],
    media_ctx: &zero_css_parser::media_query::MediaContext,
) -> Vec<zero_css_parser::ast::Rule> {
    use zero_css_parser::ast::{AtRuleBody, Rule};

    fn walk(rules: &[Rule], ctx: &mut MediaCtx, out: &mut Vec<Rule>) {
        for rule in rules {
            match rule {
                Rule::At(at_rule) => {
                    let AtRuleBody::Block(inner) = &at_rule.body else {
                        continue;
                    };
                    let name = at_rule.name.as_str();
                    if name.eq_ignore_ascii_case("media") {
                        let Some(queries) = zero_css_parser::media_query::parse_media_query(&at_rule.prelude) else {
                            continue;
                        };
                        if queries
                            .iter()
                            .any(|q| zero_css_parser::media_query::evaluate_media_query(q, ctx.media))
                        {
                            walk(inner, ctx, out);
                        }
                    } else if name.eq_ignore_ascii_case("supports") {
                        // 解析失败的 prelude（畸形条件）→ 条件视为不成立，整块丢弃
                        //（与 consume_supports_rule 的 None→丢弃语义一致）。
                        let Some(condition) =
                            zero_css_parser::supports_condition::parse_supports_condition(&at_rule.prelude)
                        else {
                            continue;
                        };
                        if (ctx.supports_eval)(&condition) {
                            walk(inner, ctx, out);
                        }
                    }
                    // 其余块 at-rule（@layer/@container/@page…）不展开——与各消费方
                    // 既有顶层扫描行为一致，本切片不扩。
                }
                // 顶层 @supports 经专用解析器产出 Rule::Supports（非 Rule::At）。
                Rule::Supports(supports_rule) => {
                    if (ctx.supports_eval)(&supports_rule.condition) {
                        walk(&supports_rule.rules, ctx, out);
                    }
                }
                other => out.push(other.clone()),
            }
        }
    }

    struct MediaCtx<'a> {
        media: &'a zero_css_parser::media_query::MediaContext,
        supports_eval: &'a mut dyn FnMut(&zero_css_parser::ast::SupportsCondition) -> bool,
    }

    let mut ctx = MediaCtx {
        media: media_ctx,
        supports_eval: &mut |cond| zero_style_system::evaluate_supports_condition(cond),
    };
    let mut out = Vec::new();
    for ss in stylesheets {
        walk(&ss.rules, &mut ctx, &mut out);
    }
    out
}

/// 从 CSS 文本提取所有有效的 `@font-face` 规则。
///
/// 条件块（@media/@supports）内条件成立的 @font-face 一并提取（R3881，见
/// [`flatten_conditional_rules`]）。
pub fn extract_font_faces(css: &str) -> Vec<ExtractedFontFace> {
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    extract_font_faces_from_rules(&flatten_conditional_rules(
        std::slice::from_ref(&stylesheet),
        &media_context_default(),
    ))
}

/// 无视口默认媒体上下文（font 提取的 @media 语义 = 媒体类型/all 恒真 + 静态偏好；
/// 带视口的调用方走 [`extract_font_faces_in_context`]）。
pub fn media_context_default() -> zero_css_parser::media_query::MediaContext {
    zero_css_parser::media_query::MediaContext::new(0.0, 0.0)
}

/// 从（已条件展开的）规则序列提取 `@font-face` 描述符。
fn extract_font_faces_from_rules(rules: &[zero_css_parser::ast::Rule]) -> Vec<ExtractedFontFace> {
    use zero_css_parser::values::types::FontStyleValue;
    rules
        .iter()
        .filter_map(|rule| match rule {
            zero_css_parser::ast::Rule::FontFace(ff) => {
                let is_italic = matches!(
                    ff.style,
                    Some(FontStyleValue::Italic) | Some(FontStyleValue::Oblique(_))
                );
                Some((
                    ff.family.clone(),
                    ff.sources.clone(),
                    ff.weight,
                    is_italic,
                    ff.stretch,
                    ff.size_adjust,
                    ff.feature_settings.clone(),
                    ff.variation_settings.clone(),
                    ff.unicode_ranges.clone(),
                ))
            }
            _ => None,
        })
        .collect()
}

/// 带视口/媒体上下文的 `@font-face` 提取（webview 生产路径用，媒体特性按真 viewport
/// 求值）。
pub fn extract_font_faces_in_context(
    css: &str,
    media_ctx: &zero_css_parser::media_query::MediaContext,
) -> Vec<ExtractedFontFace> {
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    extract_font_faces_from_rules(&flatten_conditional_rules(std::slice::from_ref(&stylesheet), media_ctx))
}

/// [`extract_used_font_families`] 的结果：静态可解析的「使用侧」族名集合，外加
/// var() 污染标记。
#[derive(Debug, Default, PartialEq, Eq)]
pub struct UsedFontFamilies {
    families: HashSet<String>,
    /// 存在含 `var()` 的 `font-family`/`font` 声明——该声明在 computed value 阶段
    /// 替换、静态提取解析不到最终族名，视为「可能引用任何已声明族」。
    unresolvable_reference: bool,
}

impl UsedFontFamilies {
    /// face 族名（ASCII 小写归一）是否可能被页面引用；`true` 时使用即取门禁不得按
    /// 族名跳过该 face 的抓取。
    pub fn may_reference(&self, family_lowercase: &str) -> bool {
        self.unresolvable_reference || self.families.contains(family_lowercase)
    }
}

/// 声明值是否含 `var()` 函数 token（自定义属性引用——tokenizer 解码转义后按函数名
/// ASCII 大小写不敏感匹配，非裸字符串 `starts_with`）。
fn value_references_var(value: &str) -> bool {
    use zero_css_parser::tokenizer::{Token, Tokenizer};
    Tokenizer::new(value)
        .any(|spanned| matches!(&spanned.token, Token::Function(name) if name.eq_ignore_ascii_case("var")))
}

/// 收集 CSS 中 `font-family` 声明引用的全部族名（去引号、ASCII 小写去重）。
///
/// 使用即取字体抓取门禁的「使用」侧输入：CSS Fonts Level 4 §5.2 字体匹配算法
/// （https://drafts.csswg.org/css-fonts-4/#font-style-matching）只在元素 computed
/// `font-family` 列表引用某族名时才会在该族内选 face——从未被任何 font-family 声明
/// （含 `font` 简写展开，style-system `expand_shorthands` 单源）引用的 `@font-face`
/// 永不可能被选中，抓取层据此跳过。
///
/// 覆盖面取**保守过近似**（宁多收集勿漏收集，漏收集 = 漏抓 = 可见回退字体回归）：
/// 条件块（@media/@supports/@layer/@container）不求值、全部递归展开收集（区别于
/// [`flatten_conditional_rules`] 的按条件过滤——那是声明提取的正确性要求，本函数
/// 只服务于「有没有可能被用到」的存在性判断）；`@font-face` 规则自身的 font-family
/// 是声明名而非使用，跳过；@keyframes 块内声明纳入（font-family 为 discrete 动画属性）。
///
/// 含 `var()` 的声明（含 `font` 简写）标记 [`UsedFontFamilies::may_reference`] 恒真：
/// var() 在 computed value 阶段替换、静态提取解析不到最终族名，且 `parse_font_family`
/// 对含 `(` 的值整声明拒绝——连逗号列表中的合法兄弟族名也会丢，必须在解析前检测。
pub fn extract_used_font_families(css: &str) -> UsedFontFamilies {
    use zero_css_parser::ast::{AtRuleBody, Rule};

    fn push_family_value(value: &str, out: &mut HashSet<String>) {
        for entry in zero_style_system::parse_font_family(value) {
            let name = zero_style_system::font_family_unquote(&entry)
                .trim()
                .to_ascii_lowercase();
            if !name.is_empty() {
                out.insert(name);
            }
        }
    }

    fn collect_declarations(declarations: &[zero_css_parser::ast::Declaration], out: &mut UsedFontFamilies) {
        for decl in declarations {
            let prop = decl.property.trim();
            let is_family = prop.eq_ignore_ascii_case("font-family");
            let is_shorthand = prop.eq_ignore_ascii_case("font");
            if !is_family && !is_shorthand {
                continue;
            }
            // var() 间接引用静态不可解析——该声明视为「可能引用任何已声明族」。
            if value_references_var(&decl.value) {
                out.unresolvable_reference = true;
                continue;
            }
            if is_family {
                push_family_value(&decl.value, &mut out.families);
            } else {
                // `font` 简写第五位即 font-family；展开复用 style-system 单源实现。
                for (expanded_prop, expanded_value, _, _) in zero_style_system::expand_shorthands(&[(
                    prop.to_string(),
                    decl.value.clone(),
                    decl.important,
                    (0, 0, 0),
                )]) {
                    if expanded_prop == "font-family" {
                        push_family_value(&expanded_value, &mut out.families);
                    }
                }
            }
        }
    }

    fn walk(rules: &[Rule], out: &mut UsedFontFamilies) {
        for rule in rules {
            match rule {
                Rule::Style(style) => collect_declarations(&style.declarations, out),
                Rule::Keyframes(keyframes) => {
                    for block in &keyframes.keyframes {
                        collect_declarations(&block.declarations, out);
                    }
                }
                // 条件块不求值全展开（保守过近似）；@font-face 自身的 font-family 是
                // 声明名，不属于「使用」。
                Rule::At(at_rule) => {
                    if let AtRuleBody::Block(inner) = &at_rule.body {
                        walk(inner, out);
                    }
                }
                Rule::Layer(layer) => walk(&layer.rules, out),
                Rule::Supports(supports) => walk(&supports.rules, out),
                Rule::Container(container) => walk(&container.rules, out),
                _ => {}
            }
        }
    }

    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut out = UsedFontFamilies::default();
    walk(&stylesheet.rules, &mut out);
    out
}

/// 文档是否含加载后出现字体使用的动态来源（`<script>` 元素或任一元素的 `style` 属性）。
///
/// 使用即取门禁族维度的启用前提：两者皆无时，「CSS 文本静态提取的族名集合」才可信。
/// 有脚本或内联样式属性时，页面可在加载后经 `el.style.fontFamily`、canvas `ctx.font`、
/// 动态注入节点等途径引用任何已声明 face，静态提取不可见——族维度跳过整体停用；
/// unicode-range 维度不受影响（其文本侧 [`extract_html_text`] 已含脚本字面量近似）。
pub fn html_has_dynamic_font_sources(html: &str) -> bool {
    let doc = zero_dom::parse_html(html);
    if !doc.get_elements_by_tag_name("script").is_empty() {
        return true;
    }
    doc.get_elements_by_tag_name_ns(None, "*")
        .iter()
        .any(|&elem| doc.get_attribute(elem, "style").is_some())
}

/// 提取 HTML 文档全部文本（`Node::textContent` 递归语义：含 `<script>`/`<style>`/注释文本）。
///
/// 使用即取字体抓取门禁的「文本」侧输入：unicode-range 把 face 限制到码点区间
/// （CSS Fonts Level 4 §4，https://drafts.csswg.org/css-fonts-4/#unicode-range-desc），
/// 文本无任一码点落入区间的子集永不参与匹配。取全文而非渲染文本属**保守过近似**：
/// display:none 文本、脚本字符串字面量（动态注入内容最常见来源）一并纳入命中判定，
/// 换取「静态文本命中 ⇒ 动态同文字体可用」的下界保证——本切片无渲染触发的懒抓取
/// 路径（后续切片），漏抓方向必须封死。
pub fn extract_html_text(html: &str) -> String {
    let doc = zero_dom::parse_html(html);
    doc.text_content(doc.root()).unwrap_or_default()
}

/// 将 CSS feature settings 转为字体整形层输入。
pub fn font_feature_settings_to_opentype(
    settings: &zero_css_parser::values::FontFeatureSettingsValue,
) -> Vec<zero_render_foundation::font::OpenTypeFeature> {
    match settings {
        zero_css_parser::values::FontFeatureSettingsValue::Normal => Vec::new(),
        zero_css_parser::values::FontFeatureSettingsValue::Features(features) => features
            .iter()
            .map(|feature| zero_render_foundation::font::OpenTypeFeature::new(feature.tag, feature.value))
            .collect(),
    }
}

/// 将 CSS variation settings 转为字体整形层输入。
pub fn font_variation_settings_to_opentype(
    settings: &zero_css_parser::values::FontVariationSettingsValue,
) -> Vec<zero_render_foundation::font::OpenTypeVariation> {
    match settings {
        zero_css_parser::values::FontVariationSettingsValue::Normal => Vec::new(),
        zero_css_parser::values::FontVariationSettingsValue::Settings(variations) => variations
            .iter()
            .map(|variation| zero_render_foundation::font::OpenTypeVariation::new(variation.tag, variation.value))
            .collect(),
    }
}

/// 从 CSS 文本提取所有 `@import` 规则的 URL（按出现顺序，已去引号/`url()` 包裹）。
///
/// 供生产 async 路径递归抓取被 `@import` 引入的样式表——旧版 css-parser 解析 `@import` 为
/// `Rule::Import` 但 style-system 显式跳过（注释误称「引擎处理」），实际全链路从不 fetch/应用，
/// 致 `@import` 引入的 CSS 静默丢失（与 R2406 @font-face 同类子资源 gap）。媒体查询本期不消费
/// （`@import url(x) screen` 的 `screen` 忽略，无条件导入）。解析失败或无规则返回空。
pub fn extract_import_urls(css: &str) -> Vec<String> {
    use zero_css_parser::ast::Rule as CssRule;
    zero_css_parser::Parser::parse_stylesheet(css)
        .rules
        .iter()
        .filter_map(|rule| match rule {
            CssRule::Import(imp) => Some(imp.url.clone()),
            _ => None,
        })
        .collect()
}

/// R1794：从 CSS 文本提取所有**图片类** `url(...)` 引用。
///
/// 与 `extract_font_faces` 互补：本函数扫描**全部** `url(...)`，但**排除**
/// `@font-face` 块内的 url（字体由 `extract_font_faces` 单独处理，避免重复抓取）。
/// `data:` URI 自 R5357 起纳入（fetch 层解码 + key 与 painter 查找一致，见 Url 臂内
/// 注释）。结果去重并保留首次出现顺序。
///
/// 覆盖 `background-image` / `list-style-image` / `border-image-source` 等所有
/// CSS 图片引用——它们都经 `decode_image_bytes` 解码后入 `image_cache`，painter
/// 按 `image_resource_key(url, document_url)` 查找像素。
pub fn extract_css_image_urls(css: &str) -> Vec<String> {
    // 经 tokenizer 识别 url()（CSS Syntax §5）：函数名 `url` 大小写不敏感且可含转义
    //（如 `URL(`、`U\r\4c (`——tokenizer eq_ignore_ascii_case + consume_escape 解码），
    // 内容亦可含转义（`support/\'green\ block.png`）。Url token 内容**已解码**，与
    // painter 一致（painter 经 tokenizer 解码 url 作 image key）。原 raw `find("url(")`
    // 漏转义函数名（driving：uri-015 `U\r\4c ("...")` 不抓 → painter image_cache miss
    // → 背景滞红；escaped-url-001 仅因 6 div 共享一图、div0 纯 `url()` 预抓而幸免）。
    // @font-face 块内 url 是字体引用（由字体路径抓取）；@import 的 url() 是样式表引用
    // （由 @import 路径抓取）——两者按 token 上下文跳过，不当图片处理。
    use zero_css_parser::tokenizer::{Token, Tokenizer};
    let mut urls: Vec<String> = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut font_face_depth: i32 = 0; // >0 表示当前在 @font-face 块内（记录其 brace 层）
    let mut pending_at: Option<String> = None;
    for spanned in Tokenizer::new(css) {
        match spanned.token {
            Token::AtKeyword(name) => pending_at = Some(name),
            Token::LBrace => {
                brace_depth += 1;
                if let Some(name) = pending_at.take()
                    && name.eq_ignore_ascii_case("font-face")
                {
                    font_face_depth = brace_depth;
                }
            }
            Token::RBrace => {
                brace_depth = (brace_depth - 1).max(0);
                if brace_depth < font_face_depth {
                    font_face_depth = 0;
                }
            }
            Token::Semicolon => {
                // @import / @namespace / @charset 等 at-rule prelude 以 `;` 结束，清待定 at-keyword。
                pending_at = None;
            }
            Token::Url(u)
                if font_face_depth == 0
                    && !matches!(pending_at.as_deref(), Some(n) if n.eq_ignore_ascii_case("import")) =>
            {
                // R2411：@import 的 url() 是样式表引用（由 @import 路径单独抓取），非图片——跳过，
                // 否则会被当作 background/list/border-image 重复抓取并解码失败（浪费）。
                let raw = u.trim();
                // R5357：data: URI 不再排除——R1794 过滤时调用方（fetch_image_subresources /
                // runner build_image_cache）尚未支持 data:，R1987 补齐调用方解码后此处成为
                // 陈旧缺口：CSS url(data:…) 图片（background/border-image 等）既不抓取也不
                // 入缓存，painter 按 key 查不到 → 整图静默不绘（driving:
                // border-image-repeat-005 `border-image: url("data:image/svg+xml,…") …`
                // 全白）。fetch 层 data: 解码与 key（image_resource_key 对非相对 href 原样
                // 哈希）均与 painter 查找一致，链条闭合。
                if !raw.is_empty() && !urls.iter().any(|x: &String| x == raw) {
                    urls.push(raw.to_string());
                }
            }
            _ => {}
        }
    }
    urls
}

/// R1794：提取 HTML 中所有文档级 CSS 文本并拼接——`<style>` 块 + 元素 inline `style=` 属性。
///
/// parser 消费方（`extract_font_faces` / `extract_used_font_families`）的输入形态
///（R1796 起覆盖 inline `style="background-image: url(...)"` 等属性内引用）。与
/// `extract_img_srcs` 同模式：复用 `zero_dom` 解析（DOM 精确，比正则稳健）。
/// tokenizer 消费方（图片 url 扫描）走 [`extract_html_css_image_urls`]，不走拼接。
pub fn extract_html_style_text(html: &str) -> String {
    let (style_blocks, inline_attrs) = extract_html_style_blocks(html);
    let mut out = String::new();
    for text in &style_blocks {
        out.push_str(text);
        // 括在注释内：parser 消费方（extract_font_faces / extract_used_font_families）
        // 视整段为单个 comment token，不触发错误恢复吞噬后续规则（裸 `)` 会让解析器
        // 丢弃之后全部规则——inline style 属性块因此在门禁侧全部失联）。tokenizer
        // 消费方对块尾未终止 url( 的终止不依赖分隔符——走
        // [`extract_html_css_image_urls`] 逐块 EOF 边界，见其文档。
        out.push_str("\n/* ) */\n");
    }
    for attr in &inline_attrs {
        // 使用即取字体门禁（extract_used_font_families）按样式表语法解析该文本——声明必须
        // 位于规则块内，故属性值包一层中性选择器块（对 url() 文本扫描无影响；逐属性独立
        // 块还隔离了单个属性内不配对花括号的跨属性吞噬）。
        out.push_str("zw-inline{");
        out.push_str(attr);
        out.push_str("}\n");
    }
    out
}

/// [`extract_html_style_text`] 与 [`extract_html_css_image_urls`] 的共享提取面：
/// `<style>` 块原文序列 + inline `style=` 属性值序列（均保持文档顺序）。通配 `"*"`
/// 匹配所有元素（R1796：inline 属性值亦是 CSS 文本）。
fn extract_html_style_blocks(html: &str) -> (Vec<String>, Vec<String>) {
    let doc = zero_dom::parse_html(html);
    let mut style_blocks = Vec::new();
    for style_id in doc.get_elements_by_tag_name("style") {
        if let Some(text) = doc.text_content(style_id) {
            style_blocks.push(text);
        }
    }
    let mut inline_attrs = Vec::new();
    for elem_id in doc.get_elements_by_tag_name_ns(None, "*") {
        if let Some(style_attr) = doc.get_attribute(elem_id, "style") {
            inline_attrs.push(style_attr);
        }
    }
    (style_blocks, inline_attrs)
}

/// R1794/R1796：文档级 CSS `url()` **图片**引用——`<style>` 块与 inline `style=`
/// 属性逐块独立过 [`extract_css_image_urls`]（每块一个 EOF 边界），跨块去重保首次
/// 出现序，@font-face / @import 排除与单文本扫描一致（data: 自 R5357 起纳入）。
///
/// 与「[`extract_html_style_text`] 拼接后一次扫描」的分道原因：块尾未终止 url( 的
/// 终止判定——tokenizer（`consume_url`）在字符串被块界换行截断后要求下一非空白字符
/// 恰为 `)`，注释分隔符不可见即整段判 bad-url 丢弃（`extract_css_urls_unterminated_
/// at_style_close` 回归实证）；裸 `)` 分隔符则毒化 parser 消费方（见
/// [`extract_html_style_text`] 注释）。EOF 恢复对两类消费方均无害：tokenizer 块末
/// EOF 闭合出干净 url（R2134 语义——与 painter 同 key，image cache 不 miss），parser
/// 每块本就独立无需分隔符。
pub fn extract_html_css_image_urls(html: &str) -> Vec<String> {
    fn push_unique(urls: &mut Vec<String>, css: &str) {
        for url in extract_css_image_urls(css) {
            if !urls.contains(&url) {
                urls.push(url);
            }
        }
    }
    let (style_blocks, inline_attrs) = extract_html_style_blocks(html);
    let mut urls = Vec::new();
    for text in &style_blocks {
        push_unique(&mut urls, text);
    }
    // tokenizer 只认 url( token，`zw-inline{}` 包裹仅为 parser 侧声明须位于规则块内，
    // 此处裸属性值扫描即可。
    for attr in &inline_attrs {
        push_unique(&mut urls, attr);
    }
    urls
}

/// 去除 XHTML CDATA 包装（`<![CDATA[...]]>`）。
///
/// html5ever 仅支持 HTML 模式解析，会将 `<style>` 中的 CDATA 标记
/// 作为文本内容保留。CSS 解析器遇到 `<![CDATA[` 时，错误恢复路径
/// 会贪婪吞噬后续所有 token（`[` 触发 `skip_to_rbracket()`），
/// 导致整个样式表提取 0 条规则。因此必须在传递给 CSS 解析器前去除。
pub fn strip_cdata(css: &str) -> std::borrow::Cow<'_, str> {
    if let Some(stripped) = css.strip_prefix("<![CDATA[").and_then(|s| s.strip_suffix("]]>")) {
        std::borrow::Cow::Owned(stripped.to_string())
    } else {
        std::borrow::Cow::Borrowed(css)
    }
}

/// 去除 `<script>` 内的 XHTML CDATA 包装，兼容两种写法：
/// - 裸 `<![CDATA[ ... ]]>`（CSS21 .xht 套件绝大多数）
/// - `//<![CDATA[ ... //]]>`（JS 行注释隐藏 CDATA，HTML/XHTML 双兼容写法）
///
/// 与 [`strip_cdata`]（专用于 `<style>` CSS）的区别：脚本侧另需处理 `//` 注释前缀。
/// `//` 不会出现在 CSS CDATA 中（CSS 注释是 `/* */`），故二者独立。
fn strip_script_cdata(code: &str) -> &str {
    let mut s = code;
    if let Some(rest) = s.strip_prefix("//<![CDATA[").or_else(|| s.strip_prefix("<![CDATA[")) {
        s = rest;
    }
    if let Some(rest) = s.strip_suffix("//]]>").or_else(|| s.strip_suffix("]]>")) {
        s = rest;
    }
    s
}

/// 提取文档级 CSP 政策源（security-hardening M2-s1）：`<meta
/// http-equiv="content-security-policy" content="...">` 的 content 值，文档序。
///
/// spec CSP3 §meta-element（https://www.w3.org/TR/CSP3/#delivery-html-meta-element）：
/// 仅 `<head>` 内的 meta 生效（body 内忽略）；http-equiv 匹配大小写不敏感；同一文档
/// 多个 meta → 多政策并集（任一阻止即阻止）。`Content-Security-Policy-Report-Only`
/// 由调用方单独区分（本函数只返回 enforce 面）。空 content 忽略（无政策语义）。
pub fn extract_meta_csp_policies(html: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let head_ids: Vec<zero_dom::NodeId> = doc.get_elements_by_tag_name("head");
    let mut policies = Vec::new();
    for meta_id in doc.get_elements_by_tag_name("meta") {
        let http_equiv = doc.get_attribute(meta_id, "http-equiv").unwrap_or_default();
        if !http_equiv.trim().eq_ignore_ascii_case("content-security-policy") {
            continue;
        }
        // head 祖先检查（spec：body 内 meta 不适用）。head_ids 非空且 meta 在其子树内。
        let in_head = head_ids.iter().any(|head_id| {
            let mut cur = Some(meta_id);
            while let Some(id) = cur {
                if *head_id == id {
                    return true;
                }
                cur = doc.parent_node(id);
            }
            false
        });
        if !in_head {
            continue;
        }
        if let Some(content) = doc.get_attribute(meta_id, "content") {
            let content = content.trim();
            if !content.is_empty() {
                policies.push(content.to_string());
            }
        }
    }
    policies
}

/// 单个 `<script>` 元素的 CSP 检查点输入面（security-hardening M2-s1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptElementCsp {
    /// `nonce` 属性值（缺失/空 → None）。
    pub nonce: Option<String>,
    /// `data-zw-harness` 标记（runner 注入/内联的基础设施脚本——harness、testdriver
    /// stub、inline_local_scripts 产物）。上游这些脚本为同源外链（天然过 'self'），
    /// 本地无网络投递才内联——属投递形态差异而非页面内容，CSP 检查点豁免。
    pub zw_harness: bool,
}

/// 提取每个 `<script>` 元素的 CSP 检查点输入（nonce + harness 标记）。
///
/// 返回向量按**全部** `<script>` 元素（含非 JS 类型——与
/// [`extract_page_scripts_indexed`] 的 `this_idx` 序号口径一致）文档序对齐：第 n 项
/// 即第 n 个 script 元素。宿主 CSP 检查点据此以 O(1) 取当前脚本元素的检查输入，
/// 无需改 [`extract_page_scripts`] 返回形状。
pub fn extract_script_csp_info(html: &str) -> Vec<ScriptElementCsp> {
    let doc = zero_dom::parse_html(html);
    doc.get_elements_by_tag_name("script")
        .into_iter()
        .map(|script_id| ScriptElementCsp {
            nonce: doc
                .get_attribute(script_id, "nonce")
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty()),
            zw_harness: doc.get_attribute(script_id, "data-zw-harness").is_some(),
        })
        .collect()
}

/// 提取每个 `<script>` 元素**内容起始位置**的 1-based 行/列号（security-hardening
/// M2-s2）——CSP 违规事件 `lineNumber`/`columnNumber` 源（spec CSP3
/// §securitypolicyviolationevent：被阻止的内联脚本位置 = 内容首字符在文档源的行列）。
///
/// 朴素原文扫描：按文档序找第 n 个 `<script` 开标签的 `>` 后首字符（内容起点），
/// 换算 1-based 行/列。**序号口径 = 原文 `<script` 出现序**，与
/// [`extract_page_scripts_indexed`] 的 DOM 全量 script 序号对齐（同一解析器同一文档
/// 序；分歧仅在 template 内容含脚本等 DOM/原文计数不一致形态——FIXME：遇到时改用
/// DOM 源偏移）。向量含全部 `<script` 出现（含非 JS），与 this_idx 口径一致。
pub fn extract_script_source_positions(html: &str) -> Vec<(u32, u32)> {
    let mut positions = Vec::new();
    let mut rest = html;
    let mut offset = 0;
    while let Some(start) = rest.find("<script") {
        let abs_start = offset + start;
        // 内容起点 = 开标签 `>` 后首字符（引号内 '>' 的病态形态可忽略——属性含 '>'
        // 时开标签扫描止于第一个 '>'，与 html5ever 行为分歧仅在病态源）。
        let Some(rel_gt) = rest[start..].find('>') else {
            break;
        };
        let content_abs = abs_start + rel_gt + 1;
        // 1-based 行 = 前缀 '\n' 计数 + 1；1-based 列 = 行内偏移 + 1。
        let prefix = &html[..content_abs];
        let line = prefix.bytes().filter(|b| *b == b'\n').count() as u32 + 1;
        let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = (content_abs - line_start) as u32 + 1;
        positions.push((line, column));
        offset = content_abs;
        rest = &html[content_abs..];
    }
    positions
}

/// 每个 `<script>` 元素（DOM 文档序）位置生效的文档基 URL（绝对；无 `<base>` → 页面 URL）。
///
/// s90（string-compilation base URL 面）spec
/// https://html.spec.whatwg.org/multipage/webappapis.html#concept-scripts-base-url
/// ——内联脚本的基 URL 在脚本创建（parse）时刻取文档基 URL 快照（后续 `<base>`
/// 变更不回溯）；调用方对外链经典脚本另用其绝对 src（外链脚本基 URL = 自身
/// URL），不经本表。DOM 前序遍历（与 [`extract_page_scripts_indexed`] 的
/// this_idx 同一序号空间——全部 `<script>` 元素、含 template 内计数项）。
/// `<base>` 无 href 跳过（spec first base **with an href attribute**）；href
/// 解析失败回落页面 URL。
pub fn script_effective_base_urls(html: &str, page_url: &str) -> Vec<String> {
    let doc = zero_dom::parse_html(html);
    let mut bases: Vec<String> = Vec::new();
    let mut current_base: Option<String> = None;
    // 显式栈 DFS（子节点逆序入栈 = 前序文档序）。
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        let Some(node) = doc.get(id) else { continue };
        if let zero_dom::NodeKind::Element(e) = &node.kind {
            // tag_name HTML 命名空间 ASCII 大写（node.rs tag_name 契约）——归一小写匹配。
            match e.tag_name().to_ascii_lowercase().as_str() {
                "base" => {
                    if let Some(href) = doc.get_attribute(id, "href").filter(|h| !h.is_empty()) {
                        current_base = Some(
                            url::Url::parse(page_url)
                                .ok()
                                .and_then(|u| u.join(&href).ok())
                                .map(|u| u.to_string())
                                .unwrap_or_else(|| page_url.to_string()),
                        );
                    }
                }
                "script" => {
                    bases.push(current_base.clone().unwrap_or_else(|| page_url.to_string()));
                }
                _ => {}
            }
        }
        for child in node.children.iter().rev() {
            stack.push(*child);
        }
    }
    bases
}

/// 单个 `<style>` 元素的 CSP 检查点输入面（security-hardening M2-s3）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleElementCsp {
    /// `nonce` 属性值（缺失/空 → None）。
    pub nonce: Option<String>,
    /// 内容原始文本（未 trim——hash 由调用方对 trim 后内容计算）。
    pub content: String,
    /// 内容起始字节偏移（原文）。
    pub content_start: usize,
    /// 内容结束字节偏移（原文， exclusive——`</style` 之前）。
    pub content_end: usize,
}

/// 从开标签文本提取属性值（security-hardening M2-s3 / s92 string-compilation CSP
/// nonce 面——`nonce="v"` / `nonce='v'` / `nonce=v` / 裸 `nonce`；大小写不敏感匹配
/// 名；runner 内联投递保留原标签 nonce 复用）。
pub fn attr_from_open_tag(open_tag: &str, name: &str) -> Option<String> {
    let lower = open_tag.to_ascii_lowercase();
    let needle = name.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(pos) = lower[search_from..].find(&needle) {
        let abs = search_from + pos;
        let after_lower = &lower[abs + needle.len()..];
        // 属性名边界：后随空白 / =（排除前缀撞名，如 `data-nonce`）。
        let prev_ok = abs == 0
            || open_tag[..abs]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_whitespace());
        if !prev_ok {
            search_from = abs + needle.len();
            continue;
        }
        if let Some(rest) = after_lower.trim_start().strip_prefix('=') {
            let val_raw = rest.trim_start();
            let (quote, val) = match val_raw.chars().next()? {
                '"' => ('"', &val_raw[1..]),
                '\'' => ('\'', &val_raw[1..]),
                _ => ('\0', val_raw),
            };
            let end = if quote == '\0' {
                val.find(|c: char| c.is_ascii_whitespace() || c == '>')
                    .unwrap_or(val.len())
            } else {
                val.find(quote).unwrap_or(val.len())
            };
            // s100：值面从**原文**切片（lower 定位镜像仅 ASCII 小写化、等字节长，偏移
            // 通用；值内多字节 UTF-8 序列小写化不变，find 偏移两侧一致）——旧版从
            // lower 切值把大小写敏感值降为小写（`onLoad(t1)` → `onload(t1)` 失效；
            // base64 nonce 同面）。
            let val_start = lower.len() - val.len();
            return Some(open_tag[val_start..val_start + end].trim().to_string());
        }
        // 裸属性（无值）——nonce 无值视作缺失。
        return None;
    }
    None
}

/// 开标签属性**在场**判定（s94，nomodule face 投递保真）：bool 属性裸形/空值形/任意值形
/// 一律算在场（HTML bool 属性语义——值无关，在场即 true）。与 [`attr_from_open_tag` 的
/// 分工：nonce 语义「裸形视作缺失」由该函数承担，本函数不做值判定；前后缀撞名
///（`data-nomodule`/`nomodulefoo`）不算在场。
/// https://html.spec.whatwg.org/multipage/common-microsyntaxes.html#boolean-attributes
pub fn open_tag_has_attr(open_tag: &str, name: &str) -> bool {
    let lower = open_tag.to_ascii_lowercase();
    let needle = name.to_ascii_lowercase();
    let mut search_from = 0;
    while let Some(pos) = lower[search_from..].find(&needle) {
        let abs = search_from + pos;
        let end = abs + needle.len();
        let prev_ok = abs == 0
            || open_tag[..abs]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_whitespace());
        let next_ok = lower[end..]
            .chars()
            .next()
            .is_none_or(|c| c.is_ascii_whitespace() || c == '=' || c == '>');
        if prev_ok && next_ok {
            return true;
        }
        search_from = end;
    }
    false
}

/// 提取每个 `<style>` 元素的 CSP 检查点输入（security-hardening M2-s3）。
///
/// 原文扫描（`<style` … `</style`），向量按文档序对齐全量 `<style>` 元素序——
/// 宿主据此检查 nonce/hash 并对被阻止元素**原地清空内容**（span 置空白，元素保留
/// ——DOM 中元素必须存在：targeting corpus 断言 violation target 为该 style 元素）。
pub fn extract_style_elements_csp(html: &str) -> Vec<StyleElementCsp> {
    let mut infos = Vec::new();
    // ASCII 小写镜像（等字节长）——大小写不敏感扫描 `<style` / `</style`（HTML 标签
    // 名大小写不敏感；原文切片仍取自 html）。
    let lower_all = html.to_ascii_lowercase();
    let mut cursor = 0;
    while let Some(start) = lower_all[cursor..].find("<style") {
        let abs_start = cursor + start;
        let Some(rel_gt) = lower_all[abs_start..].find('>') else {
            break;
        };
        let open_tag = &html[abs_start..=abs_start + rel_gt];
        let content_abs_start = abs_start + rel_gt + 1;
        let content_len = match lower_all[content_abs_start..].find("</style") {
            Some(end) => end,
            None => html.len() - content_abs_start,
        };
        infos.push(StyleElementCsp {
            nonce: attr_from_open_tag(open_tag, "nonce").filter(|n| !n.is_empty()),
            content: html[content_abs_start..content_abs_start + content_len].to_string(),
            content_start: content_abs_start,
            content_end: content_abs_start + content_len,
        });
        cursor = content_abs_start + content_len;
    }
    infos
}

#[cfg(test)]
mod tests {
    use super::{
        MediaResourceElementKind, PageScript, extract_media_resources, extract_meta_csp_policies,
        extract_page_scripts_indexed, extract_script_csp_info, extract_script_source_positions,
        extract_style_elements_csp, open_tag_has_attr, script_effective_base_urls,
    };

    const BASE_PAGE: &str = "https://wpt.test/a/b/page.html";

    #[test]
    fn script_type_strip_is_ascii_whitespace_only() {
        // s93（classic 脚本属性面）：type 首尾剥离仅 ASCII whitespace（TAB/LF/FF/CR/SPACE）——
        // 尾随 \x0B/\x85/\xA0/ /　 使 type 非 JavaScript MIME type essence match →
        // data block 不提取；ASCII 空白照剥（WPT script-type-whitespace 五例红因）。
        let script_type =
            |type_attr: &str| extract_page_scripts_indexed(&format!(r#"<script type="{type_attr}">x();</script>"#));
        assert!(matches!(
            script_type(" text/javascript ")[..],
            [(PageScript::Inline(_), _)]
        ));
        assert!(matches!(
            script_type("\t\n\x0C\r module")[..],
            [(PageScript::InlineModule(_), _)]
        ));
        assert!(script_type("text/javascript\u{000B}").is_empty());
        assert!(script_type("text/javascript\u{0085}").is_empty());
        assert!(script_type("text/javascript\u{00A0}").is_empty());
        assert!(script_type("text/javascript\u{1680}").is_empty());
        assert!(script_type("text/javascript\u{3000}").is_empty());
    }

    #[test]
    fn nomodule_suppresses_classic_but_not_module() {
        // s94（nomodule face）：nomodule 内容属性在场压制 classic 提取（内联/外链同门，
        // bool 属性值无意义）；module 形态不受 nomodule 影响（语料 module/nomodule-attribute：
        // module+nomodule 照跑，this.js 顶层 this=undefined 才入 log）。
        let scripts = |html: &str| extract_page_scripts_indexed(&format!(r#"<body>{html}</body>"#));
        assert!(scripts(r#"<script nomodule>x();</script>"#).is_empty());
        assert!(scripts(r#"<script nomodule="">x();</script>"#).is_empty());
        assert!(scripts(r#"<script src="a.js" nomodule></script>"#).is_empty());
        assert!(matches!(
            scripts(r#"<script type="module" nomodule>x();</script>"#)[..],
            [(PageScript::InlineModule(_), _)]
        ));
        assert!(matches!(
            scripts(r#"<script>x();</script>"#)[..],
            [(PageScript::Inline(_), _)]
        ));
    }

    #[test]
    fn script_type_js_mime_essence_set_s113() {
        // s113（type essence 16 集合）：legacy 集合成员提取（含 text/javascript1.x、
        // x- 变体、UPPER）；bare `javascript`/legacy 1.8+/\0 沾染/剥后空串/参数串
        // 一律 data block 不提取（语料 script-type-and-language-js 解析面 +
        // with-params：`text/javascript;charset=UTF-8` 不跑——essence 无参数语法）。
        // 字面空串照提取（type string = "text/javascript"）；type=" " 不提取。
        let scripts = |html: &str| extract_page_scripts_indexed(&format!(r#"<body>{html}</body>"#));
        for ty in [
            "text/javascript1.5",
            "TEXT/ECMASCRIPT",
            "application/x-javascript",
            " application/javascript ",
        ] {
            assert!(
                !scripts(&format!(r#"<script type="{ty}">x();</script>"#)).is_empty(),
                "essence 集合成员须提取: {ty}"
            );
        }
        for ty in [
            "javascript",
            "text/javascript1.8",
            "text/javascript\u{0000}",
            " ",
            "text/javascript;charset=UTF-8",
        ] {
            assert!(
                scripts(&format!(r#"<script type="{ty}">x();</script>"#)).is_empty(),
                "非 essence 值须为 data block 不提取: {ty}"
            );
        }
        assert!(matches!(
            scripts(r#"<script type="">x();</script>"#)[..],
            [(PageScript::Inline(_), _)]
        ));
    }

    #[test]
    fn for_event_gate_blocks_unless_window_onload() {
        // s94（for-event face）：双属性在场 → 剥 ASCII whitespace 后 for≈"window" 且
        // event≈"onload"|"onload()"（ASCII case-insens）才提取；任一属性缺失照常提取
        //（语料单属性臂 true）；NBSP 等非 ASCII 空白不剥（语料   沾染臂 false）。
        let blocked = |html: &str| extract_page_scripts_indexed(&format!(r#"<body>{html}</body>"#)).is_empty();
        let ran = |html: &str| {
            matches!(
                extract_page_scripts_indexed(&format!(r#"<body>{html}</body>"#))[..],
                [(PageScript::Inline(_), _)]
            )
        };
        // 阻塞臂（语料 expected false 形态）
        assert!(blocked(r#"<script for="wİndow" event="onload">x();</script>"#));
        assert!(blocked(r#"<script for="window" event="onload x">x();</script>"#));
        assert!(blocked(r#"<script for="window" event="onload(x)">x();</script>"#));
        assert!(blocked(r#"<script for="window" event="onclick">x();</script>"#));
        assert!(blocked(r#"<script for="" event="">x();</script>"#));
        assert!(blocked(r#"<script for="&#xA0;window" event="onload">x();</script>"#));
        assert!(blocked(r#"<script for="object" event="handler">x();</script>"#));
        // 放行臂（语料 expected true 形态）
        assert!(ran(r#"<script for=" window " event=" onload ">x();</script>"#));
        assert!(ran(r#"<script for=" WINDOW " event=" ONLOAD() ">x();</script>"#));
        assert!(ran(r#"<script event="handler">x();</script>"#));
        assert!(ran(r#"<script for="object">x();</script>"#));
    }

    #[test]
    fn open_tag_attr_presence_covers_bare_and_valued_forms() {
        // s94（nomodule face，投递保真）：bool 属性裸形/空值形/任意值形一律在场；
        // 前后缀撞名（data-nomodule / nomodulefoo）不算。
        assert!(open_tag_has_attr("<script nomodule src=\"a.js\">", "nomodule"));
        assert!(open_tag_has_attr("<script NOMODULE=\"\" src=\"a.js\">", "nomodule"));
        assert!(open_tag_has_attr("<script nomodule=\"x\">", "nomodule"));
        assert!(!open_tag_has_attr("<script src=\"a.js\">", "nomodule"));
        assert!(!open_tag_has_attr("<script data-nomodule=\"y\">", "nomodule"));
        assert!(!open_tag_has_attr("<script nomodulefoo=\"y\">", "nomodule"));
    }

    #[test]
    fn script_bases_capture_parse_time_base_and_skip_hrefless() {
        // s90：脚本前 `<base>` 捕获（脚本**执行中**动态换 base 不入解析树序之外——
        // 表纯函数于原文）；无 href 的 base 跳过；无 base 回落页面 URL。
        let html = r#"<html><head><base id="x"><base href="scripts/foo/"></head>
<body><script>a</script><script>b</script></body></html>"#;
        let bases = script_effective_base_urls(html, BASE_PAGE);
        assert_eq!(
            bases,
            ["https://wpt.test/a/b/scripts/foo/", "https://wpt.test/a/b/scripts/foo/"]
        );
        assert_eq!(
            script_effective_base_urls("<p>no scripts</p>", BASE_PAGE),
            Vec::<String>::new()
        );
        assert_eq!(script_effective_base_urls("<script>a</script>", BASE_PAGE), [BASE_PAGE]);
    }

    #[test]
    fn script_bases_count_template_and_later_base_only_for_later_scripts() {
        // base 仅影响其树序**之后**的脚本（前序遍历顺序语义）；template 内 script
        // 计数不跳过（与 extract_page_scripts_indexed 的 this_idx 同空间）。
        let html = r#"<script>a</script><base href="../"><script>b</script>"#;
        let bases = script_effective_base_urls(html, BASE_PAGE);
        assert_eq!(bases, [BASE_PAGE, "https://wpt.test/a/"]);
    }

    #[test]
    fn media_resource_extraction_respects_owner_and_direct_src() {
        let resources = extract_media_resources(
            r#"<audio src="audio.mp3"><source src="ignored-audio.ogg"></audio>
               <video><source src="fallback.webm"></video>
               <picture><source src="image.webp"><img src="image.png"></picture>
               <track src="captions.vtt">"#,
        );
        assert_eq!(
            resources
                .iter()
                .map(|resource| (resource.kind, resource.src.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (MediaResourceElementKind::Audio, "audio.mp3"),
                (MediaResourceElementKind::Source, "fallback.webm"),
                (MediaResourceElementKind::Track, "captions.vtt"),
            ]
        );
    }

    /// R328（js-dom M4）：`<template>` 内容 inert——内联/外链脚本不进页面脚本序列，
    /// template 外脚本正常提取；序号（含 template 内 script）仍按全文档 script 序递增。
    /// Driving WPT：dom/nodes/remove-next-sibling-during-replace-with（template 脚本
    /// 抢跑使测试体 `container.querySelector('script')` miss）。
    /// https://html.spec.whatwg.org/multipage/scripting.html#the-template-element
    #[test]
    fn extract_page_scripts_skips_template_contents_r328() {
        let html = "<html><body><div id=\"container\"><div id=\"target\"></div><b></b></div>\
            <template><span>New </span><script>window.__inTemplate=1;</script><span>content</span></template>\
            <script>window.__topLevel=1;</script></body></html>";
        let scripts = extract_page_scripts_indexed(html);
        // 仅 template 外的顶层脚本进入执行序列。
        assert_eq!(scripts.len(), 1, "template inline script must not execute");
        assert!(matches!(&scripts[0].0, PageScript::Inline(c) if c.contains("__topLevel")));
        // WC-M2 收敛：contents 真实化后 template 内 script 不在文档树——
        // `get_elements_by_tag_name`（与 shim gEBTN 一致）不含它，序号从 0 起。
        // （旧断言基于 parser 内联占位——template 内 script 误占 index 0。spec：
        // contents 非文档树后代，真实浏览器 gEBTN 亦不含，两者一致。）
        assert_eq!(scripts[0].1, 0, "template scripts are not in the document tree");
    }

    /// R5005（html-syntax-compat M4 残差重归因）：SVG `<script href=…>` 外部脚本提取
    ///——SVG2 script 以 href（非 src）引用外链，仅 SVG 命名空间元素降级查 href；
    /// HTML `<script href>`（无效形态）不放宽。Driving WPT：unclosed-svg-script
    /// 「SVG scripts with self-closing start tag should run」。
    #[test]
    fn extract_page_scripts_svg_href_external_r5005() {
        // SVG script href → External；HTML script href → 不视为外链（无效形态不放宽）。
        let html = "<svg><script href=\"support/svg-script-self-closing.js\"/></svg>\
            <script href=\"x.js\">console.log(1)</script>";
        let scripts = extract_page_scripts_indexed(html);
        assert_eq!(scripts.len(), 2, "SVG href 外链 + HTML 内联各一条");
        assert!(
            matches!(&scripts[0].0, PageScript::External(s) if s == "support/svg-script-self-closing.js"),
            "SVG script href 应按外部脚本提取，实得 {:?}",
            scripts[0].0
        );
        assert!(
            matches!(&scripts[1].0, PageScript::Inline(c) if c.contains("console.log")),
            "HTML script href 不应视为外链（内联仍执行），实得 {:?}",
            scripts[1].0
        );
    }

    /// s106 回归钉：SVG script 嵌套（foreign content 按 markup 解析，嵌套 script 为
    /// 真实子元素）——①外层源 = child text content（不含嵌套脚本文本，旧递归
    /// textContent 吞入致 SyntaxError）；②执行序 = end-tag 序（子脚本先闭合先执行，
    /// 旧 pre-order 父先子后）；③序号仍为全体 script pre-order 序。
    /// https://html.spec.whatwg.org/multipage/scripting.html#prepare-the-script-element
    #[test]
    fn svg_nested_script_child_source_and_end_tag_order_s106() {
        let html = "<svg><script>\nlog(\"outer-start\");\nvar a = {\n  <script>\n    log(\"inner\")\n  </script>\na:1}\nlog(\"outer-end\");\n</script></svg>";
        let scripts = extract_page_scripts_indexed(html);
        assert_eq!(scripts.len(), 2, "嵌套内外两条脚本均提取");
        assert!(
            matches!(&scripts[0].0, PageScript::Inline(c) if c.contains("inner") && !c.contains("outer")),
            "end-tag 序：嵌套子脚本先执行，实得 {:?}",
            scripts[0].0
        );
        assert!(
            matches!(&scripts[1].0, PageScript::Inline(c) if c.contains("outer-start") && c.contains("a:1}") && !c.contains("inner")),
            "外层源 = child text content（不吞嵌套脚本文本），实得 {:?}",
            scripts[1].0
        );
        assert!(
            scripts[0].1 > scripts[1].1,
            "序号保持全体 script 元素 pre-order 序（父小子大）：{:?}",
            (scripts[0].1, scripts[1].1)
        );
    }

    /// s106 回归钉：外链 SVG script 嵌套内联（140 形态）——end-tag 序子先父后，
    /// 外链经 href 提取。
    #[test]
    fn svg_external_script_nested_inline_end_tag_order_s106() {
        let html = "<svg><script xlink:href=\"scripts/include-1.js\">\n  <script>\n    log(\"inline script #1\")\n  </script>\n</script></svg>";
        let scripts = extract_page_scripts_indexed(html);
        assert_eq!(scripts.len(), 2);
        assert!(
            matches!(&scripts[0].0, PageScript::Inline(c) if c.contains("inline script #1")),
            "嵌套内联先执行，实得 {:?}",
            scripts[0].0
        );
        assert!(
            matches!(&scripts[1].0, PageScript::External(s) if s == "scripts/include-1.js"),
            "外链父脚本后执行，实得 {:?}",
            scripts[1].0
        );
    }

    /// s106 回归钉：SVG script href/xlink:href 空串——外链形态成立不落内联臂（137
    /// 判据 assert_unreached 须不可达）；HTML script src="" 维持旧内联回落。
    #[test]
    fn svg_script_empty_href_skips_inline_body_s106() {
        let svg_html = "<svg><script xlink:href=\"\">log(\"should-not-run\")</script></svg>";
        assert!(
            extract_page_scripts_indexed(svg_html).is_empty(),
            "SVG script 空href 不提取（不执行内联体）"
        );
        let html_html = "<script src=\"\">log(\"html-fallback\")</script>";
        assert_eq!(
            extract_page_scripts_indexed(html_html).len(),
            1,
            "HTML script src=\"\" 维持旧内联回落（不扩大）"
        );
    }

    /// s106 回归钉：无嵌套（HTML script raw text）时提取序与序号零变化。
    #[test]
    fn classic_scripts_order_unchanged_without_nesting_s106() {
        let html = "<script>a()</script><script src=\"x.js\"></script><script>c()</script>";
        let scripts = extract_page_scripts_indexed(html);
        assert_eq!(scripts.len(), 3);
        assert!(matches!(&scripts[0].0, PageScript::Inline(c) if c == "a()"));
        assert!(matches!(&scripts[1].0, PageScript::External(s) if s == "x.js"));
        assert!(matches!(&scripts[2].0, PageScript::Inline(c) if c == "c()"));
        assert_eq!(
            (scripts[0].1, scripts[1].1, scripts[2].1),
            (0, 1, 2),
            "无嵌套时序号与执行序均不变"
        );
    }

    /// s107 回归钉（SRI）：外链模块 integrity 属性随 PageScript 携带（原文、trim、
    /// 空属性归 None）；classic 外链脚本不带。
    #[test]
    fn external_module_integrity_attr_extraction_s107() {
        let html = "<script type=\"module\" src=\"m.js\" integrity=\"sha384-kc1K2K\"></script>\
                    <script type=\"module\" src=\"n.js\" integrity=\"  \"></script>\
                    <script type=\"module\" src=\"o.js\"></script>\
                    <script src=\"p.js\" integrity=\"sha256-AA\"></script>";
        let scripts = extract_page_scripts_indexed(html);
        assert_eq!(scripts.len(), 4);
        assert!(
            matches!(&scripts[0].0, PageScript::ExternalModule { src, integrity: Some(v) }
                if src == "m.js" && v == "sha384-kc1K2K"),
            "integrity 原文随外链模块携带"
        );
        assert!(
            matches!(&scripts[1].0, PageScript::ExternalModule { integrity: None, .. }),
            "空白 integrity 归 None"
        );
        assert!(
            matches!(&scripts[2].0, PageScript::ExternalModule { integrity: None, .. }),
            "无 integrity 归 None"
        );
        assert!(
            matches!(&scripts[3].0, PageScript::External(_)),
            "classic 外链脚本不带 integrity（FIXME 挂账）"
        );
    }

    /// script 内容起始行列提取（security-hardening M2-s2）：1-based 行列 + 全量
    /// script 序号口径。blockeduri-inline 期望形态：`<script>` 开标签同行内容起点
    /// = line 15 / column 9。
    #[test]
    fn extract_script_source_positions_one_based_sh1_m2s2() {
        let html = "<!doctype html>\n<meta http-equiv=\"x\">\n<script nonce=\"a\" src=\"t.js\"></script>\n<script>alert(1)</script>\n";
        // script#0：`<script nonce="a" src="t.js">` 共 38 字符，内容起点在其后同行。
        let positions = extract_script_source_positions(html);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0], (3, 30), "内容起点 = 开标签 > 后首字符（1-based）");
        assert_eq!(
            positions[1],
            (4, 9),
            "<script> 8 字符 → 内容起点列 9（blockeduri-inline 形态）"
        );
    }

    /// style 元素 CSP 输入提取（security-hardening M2-s3）：nonce 提取 + 内容 span
    /// 偏移 + 大小写不敏感闭合标签。
    #[test]
    fn extract_style_elements_csp_spans_sh1_m2s3() {
        let html = "<html><head>\
            <style nonce=\"ok\">body{color:red}</style>\
            <STYLE>p { color: blue }</STYLE>\
            </head><body></body></html>";
        let infos = extract_style_elements_csp(html);
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].nonce.as_deref(), Some("ok"));
        assert_eq!(infos[0].content, "body{color:red}");
        assert_eq!(&html[infos[0].content_start..infos[0].content_end], "body{color:red}");
        assert_eq!(infos[1].nonce, None);
        assert_eq!(infos[1].content, "p { color: blue }");
        assert_eq!(&html[infos[1].content_start..infos[1].content_end], "p { color: blue }");
    }

    /// meta CSP 政策提取（security-hardening M2-s1）：head 内生效、body 内忽略、
    /// http-equiv 大小写不敏感、多 meta 多政策、空 content 忽略。
    #[test]
    fn extract_meta_csp_policies_head_scoped_sh1_m2s1() {
        let html = "<html><head>\
            <meta http-equiv=\"Content-Security-Policy\" content=\"script-src 'self'\">\
            <meta HTTP-EQUIV=\"content-security-policy\" content=\"img-src 'none'\">\
            <meta http-equiv=\"x-other\" content=\"ignored\">\
            <meta http-equiv=\"content-security-policy\" content=\"\">\
            </head><body>\
            <meta http-equiv=\"content-security-policy\" content=\"default-src 'none'\">\
            </body></html>";
        assert_eq!(
            extract_meta_csp_policies(html),
            vec!["script-src 'self'", "img-src 'none'"],
            "head 内两个政策按文档序提取；body 内 / 非 CSP / 空 content 忽略"
        );
    }

    /// script CSP 输入提取序号对齐（security-hardening M2-s1）：全量 script 元素口径
    /// 与 extract_page_scripts_indexed 的 this_idx 一致（含非 JS type）；harness 标记
    /// 按属性判定。
    #[test]
    fn extract_script_csp_info_aligns_with_script_ordinal_sh1_m2s1() {
        let html = "<html><head>\
            <script nonce=\"abc\" src=\"/resources/testharness.js\"></script>\
            <script type=\"application/json\">{\"x\":1}</script>\
            </head><body>\
            <script data-zw-harness>alert(1)</script>\
            <script src=\"x.js\"></script>\
            </body></html>";
        let infos = extract_script_csp_info(html);
        assert_eq!(
            infos,
            vec![
                super::ScriptElementCsp {
                    nonce: Some("abc".to_string()),
                    zw_harness: false
                },
                super::ScriptElementCsp {
                    nonce: None,
                    zw_harness: false
                },
                super::ScriptElementCsp {
                    nonce: None,
                    zw_harness: true
                },
                super::ScriptElementCsp {
                    nonce: None,
                    zw_harness: false
                },
            ],
            "nonce + harness 标记按全量 script 文档序对齐（含非 JS type 占位）"
        );
        let scripts = extract_page_scripts_indexed(html);
        // 页面脚本序列 3 条（JSON type 过滤，占位 1 号），this_idx 与 info 向量对位。
        assert_eq!(scripts.len(), 3);
        assert_eq!(scripts[0].1, 0);
        assert_eq!(infos[scripts[0].1].nonce.as_deref(), Some("abc"));
        assert_eq!(scripts[1].1, 2);
        assert!(infos[scripts[1].1].zw_harness);
        assert_eq!(scripts[2].1, 3);
        assert!(!infos[scripts[2].1].zw_harness);
    }
}
