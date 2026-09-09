use super::*;

/// 回归测试：HTML 区块型元素必须默认 `display: block`。
///
/// 历史缺陷（R253 morning-work 4× 高度根因）：`article`/`aside`/`details` 等标签缺失于
/// `ua_default_display` 的 block 列表，回落到 CSS 初始值 `inline`。当此类「inline」元素含
/// 块级子元素（h2/p）时，触发 R109（CSS2 §9.2.1.1）匿名块拆分，在每对块级子元素之间插入
/// 包裹空白文本的幻影匿名块盒（继承父 node_id），把页面内容整体推开数倍高度
///（morning-work body 25301px ≈ chromium 5981px 的 4.2×）。
///
/// 此测试钉死 HTML Living Standard UA 样式表中应为 `display:block` 的「分组/分节」元素，
/// 防止再次遗漏导致同类幻影盒回归。
#[test]
fn test_html_block_level_sectioning_elements_default_to_block() {
    // R253 实证触发幻影盒的三个标签（修复前缺失）
    for tag in ["article", "aside", "details"] {
        assert_eq!(
            ua_default_display(tag),
            Some(DisplayValue::Block),
            "<{tag}> must default to display:block (was inline → R109 phantom anon blocks)"
        );
    }
    // 其余按 HTML Living Standard 应为 block 的分节/分组元素（防御性钉死）
    for tag in [
        "address",
        "blockquote",
        // R1651：<center> HTML4 块级（等价 <div align=center>）；先前缺 → inline 致 4px 盒
        // 与块子元素 overlap（legacy-html fixture 17-center struct-check FAIL 抓到）。
        "center",
        "dd",
        "dir",
        "div",
        "dl",
        "dt",
        "fieldset",
        "figcaption",
        "figure",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hgroup",
        "hr",
        "li",
        "listing",
        "main",
        "menu",
        "nav",
        "ol",
        "p",
        "plaintext",
        "pre",
        "search",
        "section",
        "summary",
        "ul",
        "xmp",
    ] {
        assert_eq!(
            ua_default_display(tag),
            Some(DisplayValue::Block),
            "<{tag}> should default to display:block per HTML UA stylesheet"
        );
    }
}

/// 内联元素（span/a/code 等）不得被错误标记为 block，否则破坏行内排版。
#[test]
fn test_inline_elements_remain_unset() {
    for tag in ["span", "a", "code", "em", "strong", "b", "i"] {
        assert_eq!(
            ua_default_display(tag),
            None,
            "<{tag}> should fall back to CSS initial inline (None), not block"
        );
    }
}

#[test]
fn test_textarea_defaults_to_bidirectional_resize() {
    let doc = zero_dom::parse_html("<textarea></textarea>");
    let textarea = doc.get_elements_by_tag_name("textarea")[0];
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    assert_eq!(styles[&textarea].resize, ResizeValue::Both);
}

/// R4174（CSS Overflow 3 §3.1 + SVG2 §7.2）：`<svg>` 根 UA `overflow: hidden`（chromium
/// UA 样式表同）。CSS 初始值 visible 使 svg 计算溢出恒为 Visible，paint 侧失去「作者
/// 声明轴级 overflow」与「UA 默认」的区分信号（overflow-clip-x-visible-y-svg 轴级语义
/// 无从落地）。作者声明可按轴覆盖（第二个断言：`overflow: visible` 覆盖回 Visible）。
#[test]
fn test_svg_root_defaults_to_overflow_hidden() {
    let doc = zero_dom::parse_html("<svg><rect width=\"10\" height=\"10\"/></svg>");
    let svg = doc.get_elements_by_tag_name("svg")[0];
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);
    assert_eq!(
        styles[&svg].overflow_x,
        OverflowValue::Hidden,
        "svg UA overflow-x 应 hidden"
    );
    assert_eq!(
        styles[&svg].overflow_y,
        OverflowValue::Hidden,
        "svg UA overflow-y 应 hidden"
    );

    let doc2 = zero_dom::parse_html("<svg style=\"overflow: visible\"><rect/></svg>");
    let svg2 = doc2.get_elements_by_tag_name("svg")[0];
    let styles2 = system.compute_styles(&doc2, &[]);
    assert_eq!(
        styles2[&svg2].overflow_x,
        OverflowValue::Visible,
        "作者 overflow:visible 应覆盖 UA hidden"
    );
}

/// 隐藏元素（script/style/noframes/noscript 等）必须 display:none。
/// `<noframes>` 内容在 frame-capable UA（含 chromium oracle，所有现代浏览器）中按
/// HTML 渲染规范隐藏；`<noscript>` 在脚本启用时同理隐藏。R1657：legacy-html fixture
/// 38-noframes 实测 ZW 误渲染 noframes 回退文本（5 行）vs chromium 隐藏（2 段）。
#[test]
fn test_hidden_elements_default_to_none() {
    for tag in [
        "script", "style", "link", "meta", "head", "title", "base", "basefont", "bgsound", "noframes", "noembed",
        "param", "noscript", "template", "dialog",
        // R1669：area（image map 区域，HTML 渲染规范 area{display:none}）+ frame（frameset 子，
        // nested browsing context 非普通 CSS 盒）。legacy-html fixture 44/46 LAYOUT_DUMP 抓到
        // 两者误渲染（area 6×24.6 盒、frame 6×24.6 断盒 @负 y）。
        "area", "frame",
        // R1675：datalist（自动补全建议容器）+ source/track（media 子元素，无盒）。legacy-html
        // fixture 47 LAYOUT_DUMP + pixel 采样抓到误渲染（datalist option 文本当 inline 渲染；
        // source/track 渲成 6×24.6 断盒致 video collapsed-container + sibling overlap）。
        "datalist", "source", "track",
        // R1676：rp（ruby 括号 fallback，ruby-capable UA display:none）。legacy-html fixture 48
        // pixel 采样抓到 ZW 误渲 "(" ")"（chrome 隐藏）。
        "rp",
    ] {
        assert_eq!(
            ua_default_display(tag),
            Some(DisplayValue::None),
            "<{tag}> should default to display:none (hidden content) per HTML UA stylesheet"
        );
    }
}

#[test]
fn hidden_attribute_suppresses_output_element() {
    let doc =
        zero_dom::parse_html(r#"<output id="visible">visible</output><output id="hidden" hidden>hidden</output>"#);
    let visible = doc.get_element_by_id("visible").expect("visible output");
    let hidden = doc.get_element_by_id("hidden").expect("hidden output");
    let mut system = StyleSystem::new();
    let styles = system.compute_styles(&doc, &[]);

    assert_ne!(styles[&visible].display, DisplayValue::None);
    assert_eq!(styles[&hidden].display, DisplayValue::None);
}
