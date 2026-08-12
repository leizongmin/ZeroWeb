//! HTML/CSS 资源提取族（从 `mod.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `extract_stylesheet_hrefs` / `extract_page_scripts` / `extract_img_resources` /
//! `extract_font_faces` / `extract_import_urls` / `extract_css_image_urls` /
//! `extract_html_style_text` 等纯解析函数（复用 `zero_dom` / `zero_css_parser`，
//! engine 不直接耦合网络）。`mod.rs` 经 `pub use extract::*;` 复用 + 保
//! `zero_engine::*` 外部路径；私有 helper 仅供族内，`strip_cdata` 例外（`pub`，
//! 供 `mod.rs::inject_print_page_dividers` 复用）。

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
    let doc = zero_dom::parse_html(html);
    let mut hrefs = Vec::new();
    for link_id in doc.get_elements_by_tag_name("link") {
        let rel = doc.get_attribute(link_id, "rel").unwrap_or_default();
        let is_stylesheet = rel.split_whitespace().any(|t| t.eq_ignore_ascii_case("stylesheet"));
        if !is_stylesheet {
            continue;
        }
        if let Some(href) = doc.get_attribute(link_id, "href") {
            let href = href.trim();
            if !href.is_empty() {
                hrefs.push(href.to_string());
            }
        }
    }
    hrefs
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
    /// 外链 ES module `src`。
    ExternalModule(String),
}

fn script_type_is_javascript(type_attr: Option<&str>) -> bool {
    match type_attr.map(str::trim).filter(|t| !t.is_empty()) {
        None => true,
        Some(t) if t.eq_ignore_ascii_case("text/javascript") => true,
        Some(t) if t.eq_ignore_ascii_case("application/javascript") => true,
        Some(t) if t.eq_ignore_ascii_case("module") => true,
        Some(t) if t.ends_with("javascript") => true,
        _ => false,
    }
}

fn script_is_module(type_attr: Option<&str>) -> bool {
    type_attr
        .map(|t| t.trim().eq_ignore_ascii_case("module"))
        .unwrap_or(false)
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
    for (this_idx, script_id) in doc.get_elements_by_tag_name("script").into_iter().enumerate() {
        let type_attr = doc.get_attribute(script_id, "type");
        if !script_type_is_javascript(type_attr.as_deref()) {
            continue;
        }
        let is_module = script_is_module(type_attr.as_deref());
        if let Some(src) = doc.get_attribute(script_id, "src") {
            let src = src.trim();
            if !src.is_empty() {
                if is_module {
                    scripts.push((PageScript::ExternalModule(src.to_string()), this_idx));
                } else {
                    scripts.push((PageScript::External(src.to_string()), this_idx));
                }
                continue;
            }
        }
        if let Some(raw) = doc.text_content(script_id) {
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
            }
        }
    }
    scripts
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
        if let Some(src) = doc.get_attribute(img_id, "src") {
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
        out.push(ImgResource { src, lazy });
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
    zero_css_parser::values::FontFeatureSettingsValue,
);

/// 从 CSS 文本提取所有有效的 `@font-face` 规则。
pub fn extract_font_faces(css: &str) -> Vec<ExtractedFontFace> {
    use zero_css_parser::ast::Rule as CssRule;
    use zero_css_parser::values::types::FontStyleValue;
    zero_css_parser::Parser::parse_stylesheet(css)
        .rules
        .iter()
        .filter_map(|rule| match rule {
            CssRule::FontFace(ff) => {
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
                    ff.feature_settings.clone(),
                ))
            }
            _ => None,
        })
        .collect()
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
/// `@font-face` 块内的 url（字体由 `extract_font_faces` 单独处理，避免重复抓取）
/// 与 `data:` URI（调用方识别，此处亦过滤以保持集合干净）。结果去重并保留首次出现顺序。
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
                if !raw.is_empty() && !raw.starts_with("data:") && !urls.iter().any(|x: &String| x == raw) {
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
/// 供 `extract_css_image_urls` 扫描图片 `url()` 引用（R1796 起覆盖 inline
/// `style="background-image: url(...)"` 等属性内引用）。与 `extract_img_srcs` 同模式：
/// 复用 `zero_dom` 解析（DOM 精确，比正则稳健）。
pub fn extract_html_style_text(html: &str) -> String {
    let doc = zero_dom::parse_html(html);
    let mut out = String::new();
    for style_id in doc.get_elements_by_tag_name("style") {
        if let Some(text) = doc.text_content(style_id) {
            out.push_str(&text);
            out.push('\n');
        }
    }
    // R1796：inline `style=` 属性值（如 `style="background-image: url(x)"`）亦是 CSS 文本，
    // 收集后交 extract_css_image_urls 扫描。通配 `"*"` 匹配所有元素。
    for elem_id in doc.get_elements_by_tag_name_ns(None, "*") {
        if let Some(style_attr) = doc.get_attribute(elem_id, "style") {
            out.push_str(&style_attr);
            out.push('\n');
        }
    }
    out
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
